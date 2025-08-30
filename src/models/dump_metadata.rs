use anyhow::Result;
use colored::Colorize;
use serde_json::Value;
use tokio::fs;

pub struct MetadataDumper;

impl MetadataDumper {
    /// Dump fresh raw metadata for all configured providers
    pub async fn dump_all_metadata() -> Result<()> {
        use crate::config::Config;

        println!(
            "{} Dumping fresh raw metadata for all configured providers...",
            "🔍".blue()
        );

        let config = Config::load()?;
        let models_raw_dir = Self::get_models_raw_dir()?;

        // Create models_raw directory if it doesn't exist
        if !models_raw_dir.exists() {
            fs::create_dir_all(&models_raw_dir).await?;
            println!(
                "{} Created directory: {}",
                "📁".blue(),
                models_raw_dir.display()
            );
        }

        println!(
            "{} Raw models directory: {}",
            "📁".blue(),
            models_raw_dir.display()
        );

        let mut total_providers = 0;
        let mut successful_dumps = 0;

        // Sort providers by name for consistent output
        let mut sorted_providers: Vec<_> = config.providers.iter().collect();
        sorted_providers.sort_by(|a, b| a.0.cmp(b.0));

        for (provider_name, provider_config) in sorted_providers {
            total_providers += 1;

            // Skip providers without API keys
            if provider_config.api_key.is_none() {
                println!("{} Skipping {} (no API key)", "⚠️".yellow(), provider_name);
                continue;
            }

            println!(
                "{} Fetching fresh models from {}...",
                "📡".blue(),
                provider_name
            );

            match Self::fetch_and_save_raw_metadata(&config, provider_name, &models_raw_dir).await {
                Ok(_) => {
                    println!("{} Saved {} raw models data", "✅".green(), provider_name);
                    successful_dumps += 1;
                }
                Err(e) => {
                    println!(
                        "{} Failed to fetch models from {}: {}",
                        "❌".red(),
                        provider_name,
                        e
                    );
                }
            }
        }

        println!("\n{} Summary:", "📊".blue());
        println!("   Total providers: {}", total_providers);
        println!("   Successful dumps: {}", successful_dumps);
        println!("   Raw data saved to: {}", models_raw_dir.display());

        if successful_dumps > 0 {
            println!("\n{} Raw metadata dump complete!", "🎉".green());
            println!("   Next step: Analyze the JSON files to debug metadata patterns");
        }

        Ok(())
    }

    /// Dump fresh raw metadata for a specific provider by name
    pub async fn dump_provider_by_name(provider_name: &str) -> Result<()> {
        use crate::config::Config;

        println!(
            "{} Dumping fresh raw metadata for provider: {}",
            "🔍".blue(),
            provider_name
        );

        let config = Config::load()?;

        // Check if provider exists
        if !config.has_provider(provider_name) {
            anyhow::bail!("Provider '{}' not found in configuration. Use 'lc providers list' to see available providers.", provider_name);
        }

        let models_raw_dir = Self::get_models_raw_dir()?;

        // Create models_raw directory if it doesn't exist
        if !models_raw_dir.exists() {
            fs::create_dir_all(&models_raw_dir).await?;
            println!(
                "{} Created directory: {}",
                "📁".blue(),
                models_raw_dir.display()
            );
        }

        match Self::fetch_and_save_raw_metadata(&config, provider_name, &models_raw_dir).await {
            Ok(_) => {
                println!(
                    "\n{} Successfully dumped fresh raw metadata for {}",
                    "✅".green(),
                    provider_name
                );
            }
            Err(e) => {
                anyhow::bail!("Failed to dump raw metadata for {}: {}", provider_name, e);
            }
        }

        Ok(())
    }

    /// List available raw metadata files
    pub async fn list_cached_metadata() -> Result<()> {
        let models_raw_dir = Self::get_models_raw_dir()?;

        println!("{} Available raw metadata files:", "📋".blue());
        println!();

        if !models_raw_dir.exists() {
            println!(
                "No models_raw directory found at: {}",
                models_raw_dir.display()
            );
            println!("Run 'lc dump' to fetch fresh raw metadata from providers.");
            return Ok(());
        }

        let mut entries = fs::read_dir(&models_raw_dir).await?;
        let mut files = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if let Some(extension) = path.extension() {
                if extension == "json" {
                    if let Some(provider_name) = path.file_stem().and_then(|s| s.to_str()) {
                        let metadata = entry.metadata().await?;
                        let size = Self::format_file_size(metadata.len());
                        files.push((provider_name.to_string(), path.clone(), size));
                    }
                }
            }
        }

        if files.is_empty() {
            println!("No raw metadata files found.");
            println!("Run 'lc dump' to fetch fresh raw metadata from providers.");
            return Ok(());
        }

        // Sort files by provider name
        files.sort_by(|a, b| a.0.cmp(&b.0));

        for (provider_name, path, size) in files {
            println!(
                "  {} {} - {} ({})",
                "•".blue(),
                provider_name,
                path.display(),
                size
            );
        }

        println!(
            "\n{} Use 'lc dump <provider>' to fetch fresh raw data for a specific provider",
            "💡".yellow()
        );

        Ok(())
    }

    /// Fetch fresh raw metadata from a provider and save it
    async fn fetch_and_save_raw_metadata(
        config: &crate::config::Config,
        provider_name: &str,
        models_raw_dir: &std::path::Path,
    ) -> Result<()> {
        use crate::chat;

        // Create authenticated client
        let mut config_mut = config.clone();
        let client = chat::create_authenticated_client(&mut config_mut, provider_name).await?;

        // Get provider config for raw API call
        let provider_config = config.get_provider(provider_name)?;

        // Make raw request to get full JSON response
        let raw_response = Self::fetch_raw_models_response(&client, provider_config).await?;

        // Save raw response to file
        let filename = format!("{}.json", provider_name);
        let filepath = models_raw_dir.join(&filename);

        fs::write(&filepath, &raw_response).await?;

        println!("{} Saved raw data to: {}", "💾".green(), filepath.display());

        Ok(())
    }

    /// Fetch raw models response from provider API
    async fn fetch_raw_models_response(
        _client: &crate::chat::LLMClient,
        provider_config: &crate::config::ProviderConfig,
    ) -> Result<String> {
        // No need to import debug_log, it's a macro exported from lib.rs

        // Create optimized HTTP client with connection pooling and keep-alive settings
        let http_client = reqwest::Client::builder()
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(std::time::Duration::from_secs(90))
            .tcp_keepalive(std::time::Duration::from_secs(60))
            .timeout(std::time::Duration::from_secs(60))
            .connect_timeout(std::time::Duration::from_secs(10))
            .build()?;

        let url = format!(
            "{}{}",
            provider_config.endpoint.trim_end_matches('/'),
            provider_config.models_path
        );

        crate::debug_log!("Making API request to: {}", url);
        crate::debug_log!("Request timeout: 60 seconds");

        let mut req = http_client
            .get(&url)
            .header("Content-Type", "application/json");

        crate::debug_log!("Added Content-Type: application/json header");

        // Add custom headers first
        let mut has_custom_headers = false;
        for (name, value) in &provider_config.headers {
            crate::debug_log!("Adding custom header: {}: {}", name, value);
            req = req.header(name, value);
            has_custom_headers = true;
        }

        // Only add Authorization header if no custom headers are present
        if !has_custom_headers {
            let api_key = provider_config
                .api_key
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("API key is required but not found for provider"))?;
            req = req.header("Authorization", format!("Bearer {}", api_key));
            crate::debug_log!("Added Authorization header with API key");
        } else {
            crate::debug_log!("Skipping Authorization header due to custom headers present");
        }

        crate::debug_log!("Sending HTTP GET request...");
        let response = req.send().await?;

        let status = response.status();
        crate::debug_log!("Received response with status: {}", status);

        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            crate::debug_log!("API request failed with error response: {}", text);
            anyhow::bail!("API request failed with status {}: {}", status, text);
        }

        let response_text = response.text().await?;
        crate::debug_log!("Received response body ({} bytes)", response_text.len());

        // Pretty print the JSON for better readability
        match serde_json::from_str::<Value>(&response_text) {
            Ok(json_value) => {
                crate::debug_log!("Response is valid JSON, pretty-printing");
                Ok(serde_json::to_string_pretty(&json_value)?)
            }
            Err(_) => {
                crate::debug_log!("Response is not valid JSON, returning as-is");
                // If it's not valid JSON, return as-is
                Ok(response_text)
            }
        }
    }

    /// Get the models_raw directory path
    fn get_models_raw_dir() -> Result<std::path::PathBuf> {
        use crate::config::Config;
        let config_dir = Config::config_dir()?;
        Ok(config_dir.join("models_raw"))
    }

    /// Format file size in human-readable format
    fn format_file_size(bytes: u64) -> String {
        if bytes < 1024 {
            format!("{} B", bytes)
        } else if bytes < 1024 * 1024 {
            format!("{:.1} KB", bytes as f64 / 1024.0)
        } else {
            format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
        }
    }
}
