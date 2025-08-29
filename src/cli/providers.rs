//! Provider management commands

use anyhow::Result;
use crate::cli::{ProviderCommands, HeaderCommands, ProviderVarsCommands, ProviderPathCommands};
use crate::provider_installer::{AuthType, ProviderInstaller};
use crate::{chat, config, debug_log};
use colored::Colorize;

/// Handle provider-related commands
pub async fn handle(command: ProviderCommands) -> Result<()> {
    match command {
        ProviderCommands::Install { name, force } => {
            let installer = ProviderInstaller::new()?;
            installer.install_provider(&name, force).await?;
        }
        ProviderCommands::Upgrade { name } => {
            let installer = ProviderInstaller::new()?;
            if let Some(provider_name) = name.as_deref() {
                installer.update_provider(&provider_name).await?;
            } else {
                installer.update_all_providers().await?;
            }
        }
        ProviderCommands::Uninstall { name } => {
            let installer = ProviderInstaller::new()?;
            installer.uninstall_provider(&name)?;
        }
        ProviderCommands::Available { official, tag } => {
            let installer = ProviderInstaller::new()?;
            let providers = installer.list_available().await?;
            
            println!("\n{}", "Available Providers:".bold().blue());
            
            let mut displayed_count = 0;
            for (id, metadata) in providers {
                // Apply filters
                if official && !metadata.official {
                    continue;
                }
                if let Some(ref filter_tag) = tag {
                    if !metadata.tags.contains(filter_tag) {
                        continue;
                    }
                }
                
                displayed_count += 1;
                
                print!("  {} {} - {}", "•".blue(), id.bold(), metadata.name);
                
                if metadata.official {
                    print!(" {}", "✓ official".green());
                }
                
                if !metadata.tags.is_empty() {
                    print!(" [{}]", metadata.tags.join(", ").dimmed());
                }
                
                println!("\n    {}", metadata.description.dimmed());
                
                // Show auth type
                let auth_str = match metadata.auth_type {
                    AuthType::ApiKey => "API Key",
                    AuthType::ServiceAccount => "Service Account",
                    AuthType::OAuth => "OAuth",
                    AuthType::Token => "Token",
                    AuthType::Headers => "Custom Headers",
                    AuthType::None => "None",
                };
                println!("    Auth: {}", auth_str.yellow());
            }
            
            if displayed_count == 0 {
                if official {
                    println!("No official providers found.");
                } else if tag.is_some() {
                    println!("No providers found with the specified tag.");
                } else {
                    println!("No providers available.");
                }
            } else {
                println!("\n{} Use 'lc providers install <name>' to install a provider", "💡".yellow());
            }
        }
        ProviderCommands::Add {
            name,
            url,
            models_path,
            chat_path,
        } => {
            let mut config = config::Config::load()?;
            config.add_provider_with_paths(name.clone(), url, models_path, chat_path)?;
            config.save()?;
            println!("{} Provider '{}' added successfully", "✓".green(), name);
        }
        ProviderCommands::Update { name, url } => {
            let mut config = config::Config::load()?;
            if !config.has_provider(&name) {
                anyhow::bail!("Provider '{}' not found", name);
            }
            config.add_provider(name.clone(), url)?; // add_provider also updates
            config.save()?;
            println!("{} Provider '{}' updated successfully", "✓".green(), name);
        }
        ProviderCommands::Remove { name } => {
            let mut config = config::Config::load()?;
            if !config.has_provider(&name) {
                anyhow::bail!("Provider '{}' not found", name);
            }
            config.providers.remove(&name);
            config.save()?;
            println!("{} Provider '{}' removed successfully", "✓".green(), name);
        }
        ProviderCommands::List => {
            let config = config::Config::load()?;
            if config.providers.is_empty() {
                println!("No providers configured.");
                return Ok(());
            }

            println!("\n{}", "Configured Providers:".bold().blue());

            // Load keys config to check authentication status
            let keys = crate::keys::KeysConfig::load().unwrap_or_else(|_| crate::keys::KeysConfig::new());

            // Sort providers by name for easier lookup
            let mut sorted_providers: Vec<_> = config.providers.iter().collect();
            sorted_providers.sort_by(|a, b| a.0.cmp(b.0));

            for (name, provider_config) in sorted_providers {
                // Check if provider has authentication in keys.toml
                let has_key = keys.has_auth(name);
                let key_status = if has_key { "✓".green() } else { "✗".red() };
                println!(
                    "  {} {} - {} (API Key: {})",
                    "•".blue(),
                    name.bold(),
                    provider_config.endpoint,
                    key_status
                );
            }
        }
        ProviderCommands::Models { name, refresh } => {
            debug_log!(
                "Handling provider models command for '{}', refresh: {}",
                name,
                refresh
            );

            let config = config::Config::load()?;
            let _provider_config = config.get_provider(&name)?;

            debug_log!("Provider '{}' found in config", name);

            // Use unified cache system
            match crate::unified_cache::UnifiedCache::fetch_and_cache_provider_models(
                &name, refresh,
            )
            .await
            {
                Ok(models) => {
                    debug_log!(
                        "Successfully fetched {} models for provider '{}'",
                        models.len(),
                        name
                    );
                    println!("\n{} Available models:", "Models:".bold());
                    display_provider_models(&models)?;
                }
                Err(e) => {
                    debug_log!("Unified cache failed for provider '{}': {}", name, e);
                    eprintln!("Error fetching models from provider '{}': {}", name, e);

                    // Fallback to basic listing if unified cache fails
                    debug_log!(
                        "Attempting fallback to basic client listing for provider '{}'",
                        name
                    );
                    let mut config_mut = config.clone();
                    match chat::create_authenticated_client(&mut config_mut, &name).await {
                        Ok(client) => {
                            debug_log!("Created fallback client for provider '{}'", name);
                            // Save config if tokens were updated
                            if config_mut.get_cached_token(&name) != config.get_cached_token(&name)
                            {
                                debug_log!("Tokens updated for provider '{}', saving config", name);
                                config_mut.save()?;
                            }

                            match client.list_models().await {
                                Ok(models) => {
                                    debug_log!(
                                        "Fallback client returned {} models for provider '{}'",
                                        models.len(),
                                        name
                                    );
                                    println!(
                                        "\n{} Available models (basic listing):",
                                        "Models:".bold()
                                    );
                                    for model in models {
                                        println!("  • {}", model.id);
                                    }
                                }
                                Err(e2) => {
                                    debug_log!(
                                        "Fallback client failed for provider '{}': {}",
                                        name,
                                        e2
                                    );
                                    anyhow::bail!("Failed to fetch models: {}", e2);
                                }
                            }
                        }
                        Err(e2) => {
                            debug_log!(
                                "Failed to create fallback client for provider '{}': {}",
                                name,
                                e2
                            );
                            anyhow::bail!("Failed to create client: {}", e2);
                        }
                    }
                }
            }
        }
        ProviderCommands::Headers { provider, command } => {
            let mut config = config::Config::load()?;

            if !config.has_provider(&provider) {
                anyhow::bail!("Provider '{}' not found", provider);
            }

            match command {
                HeaderCommands::Add { name, value } => {
                    config.add_header(provider.clone(), name.clone(), value.clone())?;
                    config.save()?;
                    println!(
                        "{} Header '{}' added to provider '{}'",
                        "✓".green(),
                        name,
                        provider
                    );
                }
                HeaderCommands::Delete { name } => {
                    config.remove_header(provider.clone(), name.clone())?;
                    config.save()?;
                    println!(
                        "{} Header '{}' removed from provider '{}'",
                        "✓".green(),
                        name,
                        provider
                    );
                }
                HeaderCommands::List => {
                    let headers = config.list_headers(&provider)?;
                    if headers.is_empty() {
                        println!("No custom headers configured for provider '{}'", provider);
                    } else {
                        println!(
                            "\n{} Custom headers for provider '{}':",
                            "Headers:".bold().blue(),
                            provider
                        );
                        for (name, value) in headers {
                            println!("  {} {}: {}", "•".blue(), name.bold(), value);
                        }
                    }
                }
            }
        }
        ProviderCommands::TokenUrl { provider, url } => {
            let mut config = config::Config::load()?;

            if !config.has_provider(&provider) {
                anyhow::bail!("Provider '{}' not found", provider);
            }

            config.set_token_url(provider.clone(), url.clone())?;
            config.save()?;
            println!("{} Token URL set for provider '{}'", "✓".green(), provider);
        }
        ProviderCommands::Vars { provider, command } => {
            let mut config = config::Config::load()?;
            if !config.has_provider(&provider) {
                anyhow::bail!("Provider '{}' not found", provider);
            }
            match command {
                ProviderVarsCommands::Set { key, value } => {
                    config.set_provider_var(&provider, &key, &value)?;
                    config.save()?;
                    println!(
                        "{} Set var '{}'='{}' for provider '{}'",
                        "✓".green(),
                        key,
                        value,
                        provider
                    );
                }
                ProviderVarsCommands::Get { key } => {
                    match config.get_provider_var(&provider, &key) {
                        Some(val) => println!("{}", val),
                        None => anyhow::bail!("Var '{}' not set for provider '{}'", key, provider),
                    }
                }
                ProviderVarsCommands::List => {
                    let vars = config.list_provider_vars(&provider)?;
                    if vars.is_empty() {
                        println!("No vars set for provider '{}'", provider);
                    } else {
                        println!(
                            "\n{} Vars for provider '{}':",
                            "Vars:".bold().blue(),
                            provider
                        );
                        for (k, v) in vars {
                            println!("  {} {} = {}", "•".blue(), k.bold(), v);
                        }
                    }
                }
            }
        }
        ProviderCommands::Paths { provider, command } => {
            let mut config = config::Config::load()?;
            if !config.has_provider(&provider) {
                anyhow::bail!("Provider '{}' not found", provider);
            }
            match command {
                ProviderPathCommands::Add {
                    models_path,
                    chat_path,
                    images_path,
                    embeddings_path,
                } => {
                    let mut updated = false;
                    if let Some(path) = models_path.as_deref() {
                        config.set_provider_models_path(&provider, &path)?;
                        println!(
                            "{} Models path set to '{}' for provider '{}'",
                            "✓".green(),
                            path,
                            provider
                        );
                        updated = true;
                    }
                    if let Some(path) = chat_path.as_deref() {
                        config.set_provider_chat_path(&provider, &path)?;
                        println!(
                            "{} Chat path set to '{}' for provider '{}'",
                            "✓".green(),
                            path,
                            provider
                        );
                        updated = true;
                    }
                    if let Some(path) = images_path.as_deref() {
                        config.set_provider_images_path(&provider, &path)?;
                        println!(
                            "{} Images path set to '{}' for provider '{}'",
                            "✓".green(),
                            path,
                            provider
                        );
                        updated = true;
                    }
                    if let Some(path) = embeddings_path.as_deref() {
                        config.set_provider_embeddings_path(&provider, &path)?;
                        println!(
                            "{} Embeddings path set to '{}' for provider '{}'",
                            "✓".green(),
                            path,
                            provider
                        );
                        updated = true;
                    }
                    if !updated {
                        anyhow::bail!("No paths specified. Use -m, -c, -i, or -e to set paths.");
                    }
                    config.save()?;
                }
                ProviderPathCommands::Delete {
                    models,
                    chat,
                    images,
                    embeddings,
                } => {
                    let mut updated = false;
                    if models {
                        config.reset_provider_models_path(&provider)?;
                        println!(
                            "{} Models path reset to default for provider '{}'",
                            "✓".green(),
                            provider
                        );
                        updated = true;
                    }
                    if chat {
                        config.reset_provider_chat_path(&provider)?;
                        println!(
                            "{} Chat path reset to default for provider '{}'",
                            "✓".green(),
                            provider
                        );
                        updated = true;
                    }
                    if images {
                        config.reset_provider_images_path(&provider)?;
                        println!(
                            "{} Images path reset to default for provider '{}'",
                            "✓".green(),
                            provider
                        );
                        updated = true;
                    }
                    if embeddings {
                        config.reset_provider_embeddings_path(&provider)?;
                        println!(
                            "{} Embeddings path reset to default for provider '{}'",
                            "✓".green(),
                            provider
                        );
                        updated = true;
                    }
                    if !updated {
                        anyhow::bail!("No paths specified for deletion. Use -m, -c, -i, or -e to delete paths.");
                    }
                    config.save()?;
                }
                ProviderPathCommands::List => {
                    let paths = config.list_provider_paths(&provider)?;
                    println!(
                        "\n{} API paths for provider '{}':",
                        "Paths:".bold().blue(),
                        provider
                    );
                    println!("  {} Models: {}", "•".blue(), paths.models_path.bold());
                    println!("  {} Chat: {}", "•".blue(), paths.chat_path.bold());
                    if let Some(ref images_path) = paths.images_path {
                        println!("  {} Images: {}", "•".blue(), images_path.bold());
                    } else {
                        println!("  {} Images: {}", "•".blue(), "not set".dimmed());
                    }
                    if let Some(ref embeddings_path) = paths.embeddings_path {
                        println!("  {} Embeddings: {}", "•".blue(), embeddings_path.bold());
                    } else {
                        println!("  {} Embeddings: {}", "•".blue(), "not set".dimmed());
                    }
                }
            }
        }
    }
    Ok(())
}

// Display provider models with metadata
fn display_provider_models(models: &[crate::model_metadata::ModelMetadata]) -> Result<()> {
    use colored::Colorize;

    for model in models {
        // Safety check: log if all capability flags are false to catch defaulting bugs
        if !model.supports_tools
            && !model.supports_function_calling
            && !model.supports_vision
            && !model.supports_audio
            && !model.supports_reasoning
            && !model.supports_code
        {
            debug_log!("All capability flags are false for model '{}' - this might indicate a defaulting bug", model.id);
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
        if let Some(max_out) = model.max_output_tokens {
            if max_out >= 1000 {
                info_parts.push(format!("{}k out", max_out / 1000));
            } else {
                info_parts.push(format!("{} out", max_out));
            }
        }
        if let Some(input_price) = model.input_price_per_m {
            info_parts.push(format!("${:.2}/M in", input_price));
        }
        if let Some(output_price) = model.output_price_per_m {
            info_parts.push(format!("${:.2}/M out", output_price));
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

        if !capabilities.is_empty() {
            let capability_strings: Vec<String> =
                capabilities.iter().map(|c| c.to_string()).collect();
            print!(" [{}]", capability_strings.join(" "));
        }

        if !info_parts.is_empty() {
            print!(" ({})", info_parts.join(", ").dimmed());
        }

        println!();
    }

    Ok(())
}