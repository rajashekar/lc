//! Proxy server commands

use anyhow::Result;
use colored::*;

/// Handle proxy-related commands
pub async fn handle(
    port: Option<u16>,
    host: Option<String>,
    provider: Option<String>,
    model: Option<String>,
    api_key: Option<String>,
    generate_key: bool,
) -> Result<()> {
    // Defaults
    let port_val = port.unwrap_or(8080);
    let host_str = host.unwrap_or_else(|| "127.0.0.1".to_string());

    // Handle API key generation
    let final_api_key = if generate_key {
        let generated_key = crate::services::proxy::generate_api_key();
        println!(
            "{} Generated API key: {}",
            "🔑".green(),
            generated_key.bold()
        );
        Some(generated_key)
    } else {
        api_key
    };

    // Validate provider if specified
    if let Some(ref provider_name) = provider {
        let config = crate::config::Config::load()?;
        if !config.has_provider(provider_name) {
            anyhow::bail!(
                "Provider '{}' not found. Add it first with 'lc providers add'",
                provider_name
            );
        }
    }

    // Validate model if specified (could be alias or provider:model format)
    if let Some(ref model_name) = model {
        let config = crate::config::Config::load()?;

        // Check if it's an alias
        if let Some(_alias_target) = config.get_alias(model_name) {
            // Valid alias
        } else if model_name.contains(':') {
            // Check provider:model format
            let parts: Vec<&str> = model_name.splitn(2, ':').collect();
            if parts.len() == 2 {
                let provider_name = parts[0];
                if !config.has_provider(provider_name) {
                    anyhow::bail!(
                        "Provider '{}' not found in model specification '{}'",
                        provider_name,
                        model_name
                    );
                }
            }
        } else {
            // Assume it's a model name for the default or specified provider
            // This will be validated when requests come in
        }
    }

    // Show configuration summary
    println!("\n{}", "Proxy Server Configuration:".bold().blue());
    println!("  {} {}:{}", "Address:".bold(), host_str, port_val);

    if let Some(ref provider_filter) = provider {
        println!(
            "  {} {}",
            "Provider Filter:".bold(),
            provider_filter.green()
        );
    } else {
        println!(
            "  {} {}",
            "Provider Filter:".bold(),
            "All providers".dimmed()
        );
    }

    if let Some(ref model_filter) = model {
        println!("  {} {}", "Model Filter:".bold(), model_filter.green());
    } else {
        println!("  {} {}", "Model Filter:".bold(), "All models".dimmed());
    }

    if final_api_key.is_some() {
        println!("  {} {}", "Authentication:".bold(), "Enabled".green());
    } else {
        println!("  {} {}", "Authentication:".bold(), "Disabled".yellow());
    }

    println!("\n{}", "Available endpoints:".bold().blue());
    println!("  {} http://{}:{}/models", "•".blue(), host_str, port_val);
    println!(
        "  {} http://{}:{}/v1/models",
        "•".blue(),
        host_str,
        port_val
    );
    println!(
        "  {} http://{}:{}/chat/completions",
        "•".blue(),
        host_str,
        port_val
    );
    println!(
        "  {} http://{}:{}/v1/chat/completions",
        "•".blue(),
        host_str,
        port_val
    );

    println!("\n{} Press Ctrl+C to stop the server\n", "💡".yellow());

    // Start the proxy server
    crate::services::proxy::start_proxy_server(host_str, port_val, provider, model, final_api_key)
        .await?;

    Ok(())
}
