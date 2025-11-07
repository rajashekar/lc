//! Model management commands

use crate::cli::{ModelsCommands, ModelsPathCommands, ModelsTagsCommands};
use crate::{chat, config, debug_log};
use anyhow::Result;
use colored::Colorize;

#[allow(clippy::too_many_arguments)]
/// Handle model-related commands
pub async fn handle(
    command: Option<ModelsCommands>,
    query: Option<String>,
    tags: Option<String>,
    context_length: Option<u64>,
    input_length: Option<u64>,
    output_length: Option<u64>,
    input_price: Option<f64>,
    output_price: Option<f64>,
) -> Result<()> {
    // Convert Option<u64> to Option<String> as expected by the implementation
    let context_length_str = context_length.map(|v| v.to_string());
    let input_length_str = input_length.map(|v| v.to_string());
    let output_length_str = output_length.map(|v| v.to_string());

    handle_models_command(
        command,
        query,
        tags,
        context_length_str,
        input_length_str,
        output_length_str,
        input_price,
        output_price,
    )
    .await
}

// Models command handlers
#[allow(clippy::too_many_arguments)]
async fn handle_models_command(
    command: Option<ModelsCommands>,
    query: Option<String>,
    tags: Option<String>,
    context_length: Option<String>,
    input_length: Option<String>,
    output_length: Option<String>,
    input_price: Option<f64>,
    output_price: Option<f64>,
) -> Result<()> {
    match command {
        Some(ModelsCommands::Refresh) => {
            crate::unified_cache::UnifiedCache::refresh_all_providers().await?;
        }
        Some(ModelsCommands::Info) => {
            debug_log!("Handling models info command");

            let models_dir = crate::unified_cache::UnifiedCache::models_dir()?;
            debug_log!("Models cache directory: {}", models_dir.display());

            println!("\n{}", "Models Cache Information:".bold().blue());
            println!("Cache Directory: {}", models_dir.display());

            if !models_dir.exists() {
                debug_log!("Cache directory does not exist");
                println!("Status: No cache directory found");
                return Ok(());
            }

            let entries = std::fs::read_dir(&models_dir)?;
            let mut provider_count = 0;
            let mut total_models = 0;

            debug_log!("Reading cache directory entries");

            // Collect provider information first
            let mut provider_info = Vec::new();
            for entry in entries {
                let entry = entry?;
                let path = entry.path();

                if let Some(extension) = path.extension() {
                    if extension == "json" {
                        if let Some(provider_name) = path.file_stem().and_then(|s| s.to_str()) {
                            debug_log!("Processing cache file for provider: {}", provider_name);
                            provider_count += 1;
                            match crate::unified_cache::UnifiedCache::load_provider_models(
                                provider_name,
                            )
                            .await
                            {
                                Ok(models) => {
                                    let count = models.len();
                                    total_models += count;
                                    debug_log!(
                                        "Provider '{}' has {} cached models",
                                        provider_name,
                                        count
                                    );

                                    let age_display =
                                        crate::unified_cache::UnifiedCache::get_cache_age_display(
                                            provider_name,
                                        )
                                        .await
                                        .unwrap_or_else(|_| "Unknown".to_string());
                                    let is_fresh =
                                        crate::unified_cache::UnifiedCache::is_cache_fresh(
                                            provider_name,
                                        )
                                        .await
                                        .unwrap_or(false);
                                    debug_log!(
                                        "Provider '{}' cache age: {}, fresh: {}",
                                        provider_name,
                                        age_display,
                                        is_fresh
                                    );

                                    let status = if is_fresh {
                                        age_display.green()
                                    } else {
                                        format!("{} (expired)", age_display).red()
                                    };
                                    provider_info.push((provider_name.to_string(), count, status));
                                }
                                Err(e) => {
                                    debug_log!(
                                        "Error loading cache for provider '{}': {}",
                                        provider_name,
                                        e
                                    );
                                    provider_info.push((
                                        provider_name.to_string(),
                                        0,
                                        "Error loading cache".red(),
                                    ));
                                }
                            }
                        }
                    }
                }
            }

            debug_log!("Sorting {} providers alphabetically", provider_info.len());

            // Sort providers alphabetically by name
            provider_info.sort_by(|a, b| a.0.cmp(&b.0));

            println!("\nCached Providers:");
            for (provider_name, count, status) in provider_info {
                if count > 0 {
                    println!(
                        "  {} {} - {} models ({})",
                        "•".blue(),
                        provider_name.bold(),
                        count,
                        status
                    );
                } else {
                    println!("  {} {} - {}", "•".blue(), provider_name.bold(), status);
                }
            }

            debug_log!(
                "Cache summary: {} providers, {} total models",
                provider_count,
                total_models
            );

            println!("\nSummary:");
            println!("  Providers: {}", provider_count);
            println!("  Total Models: {}", total_models);
        }
        Some(ModelsCommands::Dump) => {
            dump_models_data().await?;
        }
        Some(ModelsCommands::Embed) => {
            debug_log!("Handling embedding models command");

            // Use unified cache for embedding models command
            debug_log!("Loading all cached models from unified cache");
            let enhanced_models =
                crate::unified_cache::UnifiedCache::load_all_cached_models().await?;

            debug_log!("Loaded {} models from cache", enhanced_models.len());

            // If no cached models found, refresh all providers
            if enhanced_models.is_empty() {
                debug_log!("No cached models found, refreshing all providers");
                println!("No cached models found. Refreshing all providers...");
                crate::unified_cache::UnifiedCache::refresh_all_providers().await?;
                let enhanced_models =
                    crate::unified_cache::UnifiedCache::load_all_cached_models().await?;

                debug_log!("After refresh, loaded {} models", enhanced_models.len());

                if enhanced_models.is_empty() {
                    debug_log!("Still no models found after refresh");
                    println!("No models found after refresh.");
                    return Ok(());
                }
            }

            debug_log!("Filtering for embedding models");

            // Filter for embedding models only
            let embedding_models: Vec<_> = enhanced_models
                .into_iter()
                .filter(|model| {
                    matches!(
                        model.model_type,
                        crate::model_metadata::ModelType::Embedding
                    )
                })
                .collect();

            debug_log!("Found {} embedding models", embedding_models.len());

            if embedding_models.is_empty() {
                println!("No embedding models found.");
                return Ok(());
            }

            // Display results
            debug_log!("Displaying {} embedding models", embedding_models.len());
            display_embedding_models(&embedding_models)?;
        }
        Some(ModelsCommands::Path { command }) => match command {
            ModelsPathCommands::List => {
                crate::model_metadata::list_model_paths()?;
            }
            ModelsPathCommands::Add { path } => {
                crate::model_metadata::add_model_path(path)?;
            }
            ModelsPathCommands::Delete { path } => {
                crate::model_metadata::remove_model_path(path)?;
            }
        },
        Some(ModelsCommands::Tags { command }) => {
            match command {
                ModelsTagsCommands::List => {
                    crate::model_metadata::list_tags()?;
                }
                ModelsTagsCommands::Add { tag, rule } => {
                    // For simplicity, we'll add a single path rule
                    crate::model_metadata::add_tag(tag, vec![rule], "string".to_string(), None)?;
                }
            }
        }
        Some(ModelsCommands::Filter { tags: filter_tags }) => {
            // Load all models
            let models = crate::unified_cache::UnifiedCache::load_all_cached_models().await?;

            // Parse tags
            let required_tags: Vec<&str> = filter_tags.split(',').map(|s| s.trim()).collect();

            // Filter models based on tags
            let filtered: Vec<_> = models
                .into_iter()
                .filter(|model| {
                    for tag in &required_tags {
                        match *tag {
                            "tools" => {
                                if !model.supports_tools && !model.supports_function_calling {
                                    return false;
                                }
                            }
                            "vision" => {
                                if !model.supports_vision {
                                    return false;
                                }
                            }
                            "audio" => {
                                if !model.supports_audio {
                                    return false;
                                }
                            }
                            "reasoning" => {
                                if !model.supports_reasoning {
                                    return false;
                                }
                            }
                            "code" => {
                                if !model.supports_code {
                                    return false;
                                }
                            }
                            _ => {
                                // Check for context length filters like "ctx>100k"
                                if tag.starts_with("ctx") {
                                    if let Some(ctx) = model.context_length {
                                        if tag.contains('>') {
                                            if let Some(min_str) = tag.split('>').nth(1) {
                                                if let Ok(min_ctx) = parse_token_count(min_str) {
                                                    if ctx < min_ctx {
                                                        return false;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    true
                })
                .collect();

            if filtered.is_empty() {
                println!("No models found with tags: {}", filter_tags);
            } else {
                println!(
                    "\n{} Models with tags [{}] ({} found):",
                    "Filtered Results:".bold().blue(),
                    filter_tags,
                    filtered.len()
                );

                let mut current_provider = String::new();
                for model in filtered {
                    if model.provider != current_provider {
                        current_provider = model.provider.clone();
                        println!("\n{}", format!("{}:", current_provider).bold().green());
                    }

                    print!("  {} {}", "•".blue(), model.id.bold());

                    // Show capabilities
                    let mut capabilities = Vec::new();
                    if model.supports_tools || model.supports_function_calling {
                        capabilities.push("🔧 tools".blue());
                    }
                    if model.supports_vision {
                        capabilities.push("👁 vision".magenta());
                    }
                    if model.supports_audio {
                        capabilities.push("🔊 audio".yellow());
                    }
                    if model.supports_reasoning {
                        capabilities.push("🧠 reasoning".cyan());
                    }
                    if model.supports_code {
                        capabilities.push("💻 code".green());
                    }

                    if !capabilities.is_empty() {
                        let capability_strings: Vec<String> =
                            capabilities.iter().map(|c| c.to_string()).collect();
                        print!(" [{}]", capability_strings.join(" "));
                    }

                    // Show context info
                    if let Some(ctx) = model.context_length {
                        if ctx >= 1000 {
                            print!(" ({}k ctx)", ctx / 1000);
                        } else {
                            print!(" ({} ctx)", ctx);
                        }
                    }

                    println!();
                }
            }
        }
        None => {
            debug_log!("Handling global models command");

            // Use unified cache for global models command
            debug_log!("Loading all cached models from unified cache");
            let enhanced_models =
                crate::unified_cache::UnifiedCache::load_all_cached_models().await?;

            debug_log!("Loaded {} models from cache", enhanced_models.len());

            // If no cached models found, refresh all providers
            if enhanced_models.is_empty() {
                debug_log!("No cached models found, refreshing all providers");
                println!("No cached models found. Refreshing all providers...");
                crate::unified_cache::UnifiedCache::refresh_all_providers().await?;
                let enhanced_models =
                    crate::unified_cache::UnifiedCache::load_all_cached_models().await?;

                debug_log!("After refresh, loaded {} models", enhanced_models.len());

                if enhanced_models.is_empty() {
                    debug_log!("Still no models found after refresh");
                    println!("No models found after refresh.");
                    return Ok(());
                }
            }

            debug_log!("Applying filters to {} models", enhanced_models.len());

            // Parse tags if provided
            let tag_filters = if let Some(ref tag_str) = tags {
                let tags_vec: Vec<String> =
                    tag_str.split(',').map(|s| s.trim().to_string()).collect();
                Some(tags_vec)
            } else {
                None
            };

            // Apply filters
            let filtered_models = apply_model_filters_with_tags(
                enhanced_models,
                &query,
                tag_filters,
                &context_length,
                &input_length,
                &output_length,
                input_price,
                output_price,
            )?;

            debug_log!("After filtering, {} models remain", filtered_models.len());

            if filtered_models.is_empty() {
                debug_log!("No models match the specified criteria");
                println!("No models found matching the specified criteria.");
                return Ok(());
            }

            // Display results
            debug_log!("Displaying {} filtered models", filtered_models.len());
            display_enhanced_models(&filtered_models, &query)?;
        }
    }

    Ok(())
}

// Dump models data function
async fn dump_models_data() -> Result<()> {
    println!("{} Dumping /models for each provider...", "🔍".blue());

    // Load configuration
    let config = config::Config::load()?;

    // Create models directory if it doesn't exist
    std::fs::create_dir_all("models")?;

    let mut successful_dumps = 0;
    let mut total_providers = 0;

    for (provider_name, provider_config) in &config.providers {
        total_providers += 1;

        // Skip providers without API keys
        if provider_config.api_key.is_none() {
            println!("{} Skipping {} (no API key)", "⚠️".yellow(), provider_name);
            continue;
        }

        println!("{} Fetching models from {}...", "📡".blue(), provider_name);

        // Create authenticated client
        let mut config_mut = config.clone();
        match chat::create_authenticated_client(&mut config_mut, provider_name).await {
            Ok(client) => {
                // Make raw request to get full JSON response
                match fetch_raw_models_response(&client, provider_config).await {
                    Ok(raw_response) => {
                        // Save raw response to file
                        let filename = format!("models/{}.json", provider_name);
                        match std::fs::write(&filename, &raw_response) {
                            Ok(_) => {
                                println!(
                                    "{} Saved {} models data to {}",
                                    "✅".green(),
                                    provider_name,
                                    filename
                                );
                                successful_dumps += 1;
                            }
                            Err(e) => {
                                println!(
                                    "{} Failed to save {} models data: {}",
                                    "❌".red(),
                                    provider_name,
                                    e
                                );
                            }
                        }
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
            Err(e) => {
                println!(
                    "{} Failed to create client for {}: {}",
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
    println!("   Models data saved to: ./models/");

    if successful_dumps > 0 {
        println!("\n{} Model data collection complete!", "🎉".green());
        println!("   Next step: Analyze the JSON files to extract metadata patterns");
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn apply_model_filters_with_tags(
    models: Vec<crate::model_metadata::ModelMetadata>,
    query: &Option<String>,
    tag_filters: Option<Vec<String>>,
    context_length: &Option<String>,
    input_length: &Option<String>,
    output_length: &Option<String>,
    input_price: Option<f64>,
    output_price: Option<f64>,
) -> Result<Vec<crate::model_metadata::ModelMetadata>> {
    let mut filtered = models;

    // Apply text search filter
    if let Some(ref search_query) = query {
        let query_lower = search_query.to_lowercase();
        filtered.retain(|model| {
            model.id.to_lowercase().contains(&query_lower)
                || model
                    .display_name
                    .as_ref()
                    .is_some_and(|name| name.to_lowercase().contains(&query_lower))
                || model
                    .description
                    .as_ref()
                    .is_some_and(|desc| desc.to_lowercase().contains(&query_lower))
        });
    }

    // Apply tag filters if provided
    if let Some(tags) = tag_filters {
        for tag in tags {
            match tag.as_str() {
                "tools" => {
                    filtered
                        .retain(|model| model.supports_tools || model.supports_function_calling);
                }
                "reasoning" => {
                    filtered.retain(|model| model.supports_reasoning);
                }
                "vision" => {
                    filtered.retain(|model| model.supports_vision);
                }
                "audio" => {
                    filtered.retain(|model| model.supports_audio);
                }
                "code" => {
                    filtered.retain(|model| model.supports_code);
                }
                _ => {
                    // Ignore unknown tags
                }
            }
        }
    }

    // Apply context length filter
    if let Some(ref ctx_str) = context_length {
        let min_ctx = parse_token_count(ctx_str)?;
        filtered.retain(|model| model.context_length.is_some_and(|ctx| ctx >= min_ctx));
    }

    // Apply input length filter
    if let Some(ref input_str) = input_length {
        let min_input = parse_token_count(input_str)?;
        filtered.retain(|model| {
            model
                .max_input_tokens
                .is_some_and(|input| input >= min_input)
                || model.context_length.is_some_and(|ctx| ctx >= min_input)
        });
    }

    // Apply output length filter
    if let Some(ref output_str) = output_length {
        let min_output = parse_token_count(output_str)?;
        filtered.retain(|model| {
            model
                .max_output_tokens
                .is_some_and(|output| output >= min_output)
        });
    }

    // Apply price filters
    if let Some(max_input_price) = input_price {
        filtered.retain(|model| {
            model
                .input_price_per_m
                .is_none_or(|price| price <= max_input_price)
        });
    }

    if let Some(max_output_price) = output_price {
        filtered.retain(|model| {
            model
                .output_price_per_m
                .is_none_or(|price| price <= max_output_price)
        });
    }

    // Sort by provider, then by model name
    filtered.sort_by(|a, b| a.provider.cmp(&b.provider).then(a.id.cmp(&b.id)));

    Ok(filtered)
}

fn parse_token_count(input: &str) -> Result<u32> {
    let input = input.to_lowercase();
    if let Some(num_str) = input.strip_suffix('k') {
        let num: f32 = num_str
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid token count format: '{}'", input))?;
        Ok((num * 1000.0) as u32)
    } else if let Some(num_str) = input.strip_suffix('m') {
        let num: f32 = num_str
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid token count format: '{}'", input))?;
        Ok((num * 1000000.0) as u32)
    } else {
        input
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid token count format: '{}'", input))
    }
}

fn display_enhanced_models(
    models: &[crate::model_metadata::ModelMetadata],
    query: &Option<String>,
) -> Result<()> {
    if let Some(ref search_query) = query {
        println!(
            "\n{} Models matching '{}' ({} found):",
            "Search Results:".bold().blue(),
            search_query,
            models.len()
        );
    } else {
        println!(
            "\n{} Available models ({} total):",
            "Models:".bold().blue(),
            models.len()
        );
    }

    let mut current_provider = String::new();
    for model in models {
        if model.provider != current_provider {
            current_provider = model.provider.clone();
            println!("\n{}", format!("{}:", current_provider).bold().green());
        }

        // Build capability indicators
        let mut capabilities = Vec::new();
        if model.supports_tools || model.supports_function_calling {
            capabilities.push("🔧 tools".blue());
        }
        if model.supports_vision {
            capabilities.push("👁 vision".magenta());
        }
        if model.supports_audio {
            capabilities.push("🔊 audio".yellow());
        }
        if model.supports_reasoning {
            capabilities.push("🧠 reasoning".cyan());
        }
        if model.supports_code {
            capabilities.push("💻 code".green());
        }

        // Build context info
        let mut context_info = Vec::new();
        if let Some(ctx) = model.context_length {
            context_info.push(format!("{}k ctx", ctx / 1000));
        }
        if let Some(max_out) = model.max_output_tokens {
            context_info.push(format!("{}k out", max_out / 1000));
        }

        // Display model with metadata
        let model_display = if let Some(ref display_name) = model.display_name {
            format!("{} ({})", model.id, display_name)
        } else {
            model.id.clone()
        };

        print!("  {} {}", "•".blue(), model_display.bold());

        if !capabilities.is_empty() {
            let capability_strings: Vec<String> =
                capabilities.iter().map(|c| c.to_string()).collect();
            print!(" [{}]", capability_strings.join(" "));
        }

        if !context_info.is_empty() {
            print!(" ({})", context_info.join(", ").dimmed());
        }

        println!();
    }

    Ok(())
}

async fn fetch_raw_models_response(
    _client: &crate::chat::LLMClient,
    provider_config: &crate::config::ProviderConfig,
) -> Result<String> {
    use serde_json::Value;

    // Create optimized HTTP client with connection pooling and keep-alive settings
    let http_client = reqwest::Client::builder()
        .pool_max_idle_per_host(10)
        .pool_idle_timeout(std::time::Duration::from_secs(90))
        .tcp_keepalive(std::time::Duration::from_secs(60))
        .timeout(std::time::Duration::from_secs(60))
        .connect_timeout(std::time::Duration::from_secs(10))
        .build()?;

    let url = provider_config.get_models_url();

    debug_log!("Making API request to: {}", url);
    debug_log!("Request timeout: 60 seconds");

    let mut req = http_client
        .get(&url)
        .header("Content-Type", "application/json");

    debug_log!("Added Content-Type: application/json header");

    // Add custom headers first
    let mut has_custom_headers = false;
    for (name, value) in &provider_config.headers {
        debug_log!("Adding custom header: {}: {}", name, value);
        req = req.header(name, value);
        has_custom_headers = true;
    }

    // Only add Authorization header if no custom headers are present
    if !has_custom_headers {
        if let Some(api_key) = provider_config.api_key.as_ref() {
            req = req.header("Authorization", format!("Bearer {}", api_key));
            debug_log!("Added Authorization header with API key");
        } else {
            debug_log!("No API key configured and no custom headers provided; cannot add Authorization header");
            // Return a clear error instead of panicking
            anyhow::bail!("No API key configured and no custom headers set for models request");
        }
    } else {
        debug_log!("Skipping Authorization header due to custom headers present");
    }

    debug_log!("Sending HTTP GET request...");
    let response = req.send().await?;

    let status = response.status();
    debug_log!("Received response with status: {}", status);

    if !status.is_success() {
        let text = response.text().await.unwrap_or_default();
        debug_log!("API request failed with error response: {}", text);
        anyhow::bail!("API request failed with status {}: {}", status, text);
    }

    let response_text = response.text().await?;
    debug_log!("Received response body ({} bytes)", response_text.len());

    // Pretty print the JSON for better readability
    match serde_json::from_str::<Value>(&response_text) {
        Ok(json_value) => {
            debug_log!("Response is valid JSON, pretty-printing");
            Ok(serde_json::to_string_pretty(&json_value)?)
        }
        Err(_) => {
            debug_log!("Response is not valid JSON, returning as-is");
            // If it's not valid JSON, return as-is
            Ok(response_text)
        }
    }
}

// Display embedding models with metadata
fn display_embedding_models(models: &[crate::model_metadata::ModelMetadata]) -> Result<()> {
    println!(
        "\n{} Available embedding models ({} total):",
        "Embedding Models:".bold().blue(),
        models.len()
    );

    let mut current_provider = String::new();
    for model in models {
        if model.provider != current_provider {
            current_provider = model.provider.clone();
            println!("\n{}", format!("{}:", current_provider).bold().green());
        }

        // Build context and pricing info
        let mut info_parts = Vec::new();
        if let Some(ctx) = model.context_length {
            if ctx >= 1000000 {
                info_parts.push(format!("{}m ctx", ctx / 1000000));
            } else if ctx >= 1000 {
                info_parts.push(format!("{}k ctx", ctx / 1000));
            } else {
                info_parts.push(format!("{} ctx", ctx));
            }
        }
        if let Some(input_price) = model.input_price_per_m {
            info_parts.push(format!("${:.2}/M", input_price));
        }

        // Display model with metadata
        let model_display = if let Some(ref display_name) = model.display_name {
            if display_name != &model.id {
                format!("{} ({})", model.id, display_name)
            } else {
                model.id.clone()
            }
        } else {
            model.id.clone()
        };

        print!("  {} {}", "•".blue(), model_display.bold());

        if !info_parts.is_empty() {
            print!(" ({})", info_parts.join(", ").dimmed());
        }

        println!();
    }

    Ok(())
}
