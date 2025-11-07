use anyhow::Result;
use colored::Colorize;
use std::io::{self, Write};

use crate::cli::KeyCommands;
use crate::config;

/// Handle key command operations
pub async fn handle(command: KeyCommands) -> Result<()> {
    match command {
        KeyCommands::Add { name } => add_key(name).await,
        KeyCommands::Get { name } => get_key(name).await,
        KeyCommands::List => list_keys().await,
        KeyCommands::Remove { name } => remove_key(name).await,
    }
}

async fn add_key(name: String) -> Result<()> {
    let mut config = config::Config::load()?;

    if !config.has_provider(&name) {
        anyhow::bail!(
            "Provider '{}' not found. Add it first with 'lc providers add'",
            name
        );
    }

    // Detect Google SA JWT providers and prompt for Service Account JSON
    let provider_cfg = config.get_provider(&name)?;
    let is_google_sa = provider_cfg.auth_type.as_deref() == Some("google_sa_jwt")
        || provider_cfg.endpoint.contains("aiplatform.googleapis.com");

    if is_google_sa {
        println!("Detected Google Vertex AI provider. Please provide the Service Account JSON.");
        println!("Options:");
        println!("  1. Paste the base64 version directly (ex: cat sa.json | base64)");
        println!("  2. Provide the path to the JSON file (ex: /path/to/sa.json)");
        print!("Base64 Service Account JSON or file path for {}: ", name);
        io::stdout().flush()?;

        // Use regular stdin reading instead of rpassword for large inputs
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        let sa_json = if input.starts_with('/') || input.ends_with(".json") {
            // Treat as file path
            match std::fs::read_to_string(input) {
                Ok(file_content) => file_content,
                Err(e) => {
                    anyhow::bail!("Failed to read service account file '{}': {}", input, e)
                }
            }
        } else {
            // Treat as base64 input - clean whitespace and newlines
            let sa_json_b64 = input
                .trim()
                .replace("\n", "")
                .replace("\r", "")
                .replace(" ", "");

            // Decode base64
            use base64::{engine::general_purpose, Engine as _};
            match general_purpose::STANDARD.decode(&sa_json_b64) {
                Ok(decoded_bytes) => match String::from_utf8(decoded_bytes) {
                    Ok(json_str) => json_str,
                    Err(_) => anyhow::bail!("Invalid UTF-8 in decoded base64 data"),
                },
                Err(_) => anyhow::bail!("Invalid base64 format"),
            }
        };

        // Minimal validation
        let parsed: serde_json::Value =
            serde_json::from_str(&sa_json).map_err(|e| anyhow::anyhow!("Invalid JSON: {}", e))?;
        let sa_type = parsed.get("type").and_then(|v| v.as_str()).unwrap_or("");
        let client_email = parsed
            .get("client_email")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let private_key = parsed
            .get("private_key")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if sa_type != "service_account" {
            anyhow::bail!("Service Account JSON must have \"type\": \"service_account\"");
        }
        if client_email.is_empty() {
            anyhow::bail!("Service Account JSON missing 'client_email'");
        }
        if private_key.is_empty() {
            anyhow::bail!("Service Account JSON missing 'private_key'");
        }

        // Store full JSON string in api_key field (used by JWT mint flow)
        config.set_api_key(name.clone(), sa_json)?;
        config.save()?;
        println!(
            "{} Service Account stored for provider '{}'",
            "✓".green(),
            name
        );
    } else {
        print!("Enter API key for {}: ", name);
        io::stdout().flush()?;
        let key = rpassword::read_password()?;

        config.set_api_key(name.clone(), key)?;
        config.save()?;
        println!("{} API key set for provider '{}'", "✓".green(), name);
    }

    Ok(())
}

async fn get_key(name: String) -> Result<()> {
    let config = config::Config::load()?;

    if !config.has_provider(&name) {
        anyhow::bail!("Provider '{}' not found", name);
    }

    // Use centralized keys instead of provider config
    let keys = crate::keys::KeysConfig::load()?;
    if let Some(auth) = keys.get_auth(&name) {
        match auth {
            crate::keys::ProviderAuth::ApiKey(key) => println!("{}", key),
            crate::keys::ProviderAuth::ServiceAccount(sa_json) => println!("{}", sa_json),
            crate::keys::ProviderAuth::Token(token) => println!("{}", token),
            crate::keys::ProviderAuth::OAuthToken(oauth) => println!("{}", oauth),
            crate::keys::ProviderAuth::Headers(headers) => {
                for (k, v) in headers {
                    println!("{}={}", k, v);
                }
            }
        }
    } else {
        anyhow::bail!("No API key configured for provider '{}'", name);
    }

    Ok(())
}

async fn list_keys() -> Result<()> {
    let config = config::Config::load()?;
    if config.providers.is_empty() {
        println!("No providers configured.");
        return Ok(());
    }

    println!("\n{}", "API Key Status:".bold().blue());

    // Load keys from centralized keys.toml
    let keys = crate::keys::KeysConfig::load().unwrap_or_else(|_| crate::keys::KeysConfig::new());

    for name in config.providers.keys() {
        // Check if provider has authentication in centralized keys
        let has_auth = keys.has_auth(name);
        let status = if has_auth {
            "✓ Configured".green()
        } else {
            "✗ Missing".red()
        };
        println!("  {} {} - {}", "•".blue(), name.bold(), status);
    }

    Ok(())
}

async fn remove_key(name: String) -> Result<()> {
    let mut config = config::Config::load()?;

    if !config.has_provider(&name) {
        anyhow::bail!("Provider '{}' not found", name);
    }

    if let Some(provider_config) = config.providers.get_mut(&name) {
        provider_config.api_key = None;
    }
    config.save()?;
    println!("{} API key removed for provider '{}'", "✓".green(), name);

    Ok(())
}
