//! Configuration management commands

use crate::cli::ConfigCommands;
use crate::cli::{DeleteCommands, GetCommands, SetCommands};
use crate::config;
use anyhow::Result;
use colored::Colorize;

/// Handle config-related commands
pub async fn handle(command: Option<ConfigCommands>) -> Result<()> {
    match command {
        Some(ConfigCommands::Set { command }) => handle_set_command(command).await,
        Some(ConfigCommands::Get { command }) => handle_get_command(command).await,
        Some(ConfigCommands::Delete { command }) => handle_delete_command(command).await,
        Some(ConfigCommands::Path) => handle_path_command().await,
        None => handle_show_current_config().await,
    }
}

async fn handle_set_command(command: SetCommands) -> Result<()> {
    match command {
        SetCommands::Provider { name } => {
            let mut config = config::Config::load()?;

            if !config.has_provider(&name) {
                anyhow::bail!(
                    "Provider '{}' not found. Add it first with 'lc providers add'",
                    name
                );
            }

            config.default_provider = Some(name.clone());
            config.save()?;
            println!("{} Default provider set to '{}'", "✓".green(), name);
        }
        SetCommands::Model { name } => {
            let mut config = config::Config::load()?;
            config.default_model = Some(name.clone());
            config.save()?;
            println!("{} Default model set to '{}'", "✓".green(), name);
        }
        SetCommands::SystemPrompt { prompt } => {
            let mut config = config::Config::load()?;
            let resolved_prompt = config.resolve_template_or_prompt(&prompt);
            config.system_prompt = Some(resolved_prompt);
            config.save()?;
            println!("{} System prompt set", "✓".green());
        }
        SetCommands::MaxTokens { value } => {
            let mut config = config::Config::load()?;
            let parsed_value = config::Config::parse_max_tokens(&value)?;
            config.max_tokens = Some(parsed_value);
            config.save()?;
            println!("{} Max tokens set to {}", "✓".green(), parsed_value);
        }
        SetCommands::Temperature { value } => {
            let mut config = config::Config::load()?;
            let parsed_value = config::Config::parse_temperature(&value)?;
            config.temperature = Some(parsed_value);
            config.save()?;
            println!("{} Temperature set to {}", "✓".green(), parsed_value);
        }
        SetCommands::Search { name } => {
            let mut search_config = crate::search::SearchConfig::load()?;

            if !search_config.has_provider(&name) {
                anyhow::bail!(
                    "Search provider '{}' not found. Add it first with 'lc search provider add'",
                    name
                );
            }

            search_config.set_default_provider(name.clone())?;
            search_config.save()?;
            println!("{} Default search provider set to '{}'", "✓".green(), name);
        }
        SetCommands::Stream { value } => {
            let mut config = config::Config::load()?;
            let stream_value = match value.to_lowercase().as_str() {
                "true" | "1" | "yes" | "on" => true,
                "false" | "0" | "no" | "off" => false,
                _ => anyhow::bail!("Invalid stream value '{}'. Use 'true' or 'false'", value),
            };
            config.stream = Some(stream_value);
            config.save()?;
            println!("{} Streaming mode set to {}", "✓".green(), stream_value);
        }
    }
    Ok(())
}

async fn handle_get_command(command: GetCommands) -> Result<()> {
    let config = config::Config::load()?;
    match command {
        GetCommands::Provider => {
            if let Some(provider) = &config.default_provider {
                println!("{}", provider);
            } else {
                anyhow::bail!("No default provider configured");
            }
        }
        GetCommands::Model => {
            if let Some(model) = &config.default_model {
                println!("{}", model);
            } else {
                anyhow::bail!("No default model configured");
            }
        }
        GetCommands::SystemPrompt => {
            if let Some(system_prompt) = &config.system_prompt {
                println!("{}", system_prompt);
            } else {
                anyhow::bail!("No system prompt configured");
            }
        }
        GetCommands::MaxTokens => {
            if let Some(max_tokens) = &config.max_tokens {
                println!("{}", max_tokens);
            } else {
                anyhow::bail!("No max tokens configured");
            }
        }
        GetCommands::Temperature => {
            if let Some(temperature) = &config.temperature {
                println!("{}", temperature);
            } else {
                anyhow::bail!("No temperature configured");
            }
        }
        GetCommands::Search => {
            let search_config = crate::search::SearchConfig::load()?;
            if let Some(provider) = search_config.get_default_provider() {
                println!("{}", provider);
            } else {
                anyhow::bail!("No default search provider configured");
            }
        }
        GetCommands::Stream => {
            if let Some(stream) = &config.stream {
                println!("{}", stream);
            } else {
                anyhow::bail!("No streaming mode configured");
            }
        }
    }
    Ok(())
}

async fn handle_delete_command(command: DeleteCommands) -> Result<()> {
    let mut config = config::Config::load()?;
    match command {
        DeleteCommands::Provider => {
            if config.default_provider.is_some() {
                config.default_provider = None;
                config.save()?;
                println!("{} Default provider deleted", "✓".green());
            } else {
                anyhow::bail!("No default provider configured to delete");
            }
        }
        DeleteCommands::Model => {
            if config.default_model.is_some() {
                config.default_model = None;
                config.save()?;
                println!("{} Default model deleted", "✓".green());
            } else {
                anyhow::bail!("No default model configured to delete");
            }
        }
        DeleteCommands::SystemPrompt => {
            if config.system_prompt.is_some() {
                config.system_prompt = None;
                config.save()?;
                println!("{} System prompt deleted", "✓".green());
            } else {
                anyhow::bail!("No system prompt configured to delete");
            }
        }
        DeleteCommands::MaxTokens => {
            if config.max_tokens.is_some() {
                config.max_tokens = None;
                config.save()?;
                println!("{} Max tokens deleted", "✓".green());
            } else {
                anyhow::bail!("No max tokens configured to delete");
            }
        }
        DeleteCommands::Temperature => {
            if config.temperature.is_some() {
                config.temperature = None;
                config.save()?;
                println!("{} Temperature deleted", "✓".green());
            } else {
                anyhow::bail!("No temperature configured to delete");
            }
        }
        DeleteCommands::Search => {
            let mut search_config = crate::search::SearchConfig::load()?;
            if search_config.get_default_provider().is_some() {
                search_config.set_default_provider(String::new())?;
                search_config.save()?;
                println!("{} Default search provider deleted", "✓".green());
            } else {
                anyhow::bail!("No default search provider configured to delete");
            }
        }
        DeleteCommands::Stream => {
            let mut config = config::Config::load()?;
            if config.stream.is_some() {
                config.stream = None;
                config.save()?;
                println!("{} Streaming mode deleted", "✓".green());
            } else {
                anyhow::bail!("No streaming mode configured to delete");
            }
        }
    }
    Ok(())
}

async fn handle_path_command() -> Result<()> {
    let config_dir = config::Config::config_dir()?;
    println!("\n{}", "Configuration Directory:".bold().blue());
    println!("{}", config_dir.display());
    println!("\n{}", "Files:".bold().blue());
    println!("  {} config.toml", "•".blue());
    println!("  {} logs.db (synced to cloud)", "•".blue());
    println!("\n{}", "Database Management:".bold().blue());
    println!(
        "  {} Purge old logs: {}",
        "•".blue(),
        "lc logs purge --older-than-days 30".dimmed()
    );
    println!(
        "  {} Keep recent logs: {}",
        "•".blue(),
        "lc logs purge --keep-recent 1000".dimmed()
    );
    println!(
        "  {} Size-based purge: {}",
        "•".blue(),
        "lc logs purge --max-size-mb 50".dimmed()
    );
    Ok(())
}

async fn handle_show_current_config() -> Result<()> {
    // Show current configuration with enhanced model metadata
    let config = config::Config::load()?;
    println!("\n{}", "Current Configuration:".bold().blue());

    if let Some(provider) = &config.default_provider {
        println!("provider {}", provider);
    } else {
        println!("provider {}", "not set".dimmed());
    }

    if let Some(model) = &config.default_model {
        // Try to find model metadata to display rich information
        if let Some(provider) = &config.default_provider {
            match load_provider_enhanced_models(provider).await {
                Ok(models) => {
                    // Find the specific model
                    if let Some(model_metadata) = models.iter().find(|m| m.id == *model) {
                        // Display model with metadata
                        let _model_info = vec![model.clone()];

                        // Build capability indicators
                        let mut capabilities = Vec::new();
                        if model_metadata.supports_tools || model_metadata.supports_function_calling
                        {
                            capabilities.push("🔧 tools".blue());
                        }
                        if model_metadata.supports_vision {
                            capabilities.push("👁 vision".magenta());
                        }
                        if model_metadata.supports_audio {
                            capabilities.push("🔊 audio".yellow());
                        }
                        if model_metadata.supports_reasoning {
                            capabilities.push("🧠 reasoning".cyan());
                        }
                        if model_metadata.supports_code {
                            capabilities.push("💻 code".green());
                        }

                        // Build context and pricing info
                        let mut info_parts = Vec::new();
                        if let Some(ctx) = model_metadata.context_length {
                            if ctx >= 1000000 {
                                info_parts.push(format!("{}m ctx", ctx / 1000000));
                            } else if ctx >= 1000 {
                                info_parts.push(format!("{}k ctx", ctx / 1000));
                            } else {
                                info_parts.push(format!("{} ctx", ctx));
                            }
                        }
                        if let Some(input_price) = model_metadata.input_price_per_m {
                            info_parts.push(format!("${:.2}/M in", input_price));
                        }
                        if let Some(output_price) = model_metadata.output_price_per_m {
                            info_parts.push(format!("${:.2}/M out", output_price));
                        }

                        // Display model name with metadata
                        let model_display =
                            if let Some(ref display_name) = model_metadata.display_name {
                                if display_name != &model_metadata.id {
                                    format!("{} ({})", model, display_name)
                                } else {
                                    model.clone()
                                }
                            } else {
                                model.clone()
                            };

                        print!("model {}", model_display);

                        if !capabilities.is_empty() {
                            let capability_strings: Vec<String> =
                                capabilities.iter().map(|c| c.to_string()).collect();
                            print!(" [{}]", capability_strings.join(" "));
                        }

                        if !info_parts.is_empty() {
                            print!(" ({})", info_parts.join(", ").dimmed());
                        }

                        println!();
                    } else {
                        // Model not found in metadata, show basic info
                        println!("model {}", model);
                    }
                }
                Err(_) => {
                    // Failed to load metadata, show basic info
                    println!("model {}", model);
                }
            }
        } else {
            // No provider set, show basic info
            println!("model {}", model);
        }
    } else {
        println!("model {}", "not set".dimmed());
    }

    if let Some(system_prompt) = &config.system_prompt {
        println!("system_prompt {}", system_prompt);
    } else {
        println!("system_prompt {}", "not set".dimmed());
    }

    if let Some(max_tokens) = &config.max_tokens {
        println!("max_tokens {}", max_tokens);
    } else {
        println!("max_tokens {}", "not set".dimmed());
    }

    if let Some(temperature) = &config.temperature {
        println!("temperature {}", temperature);
    } else {
        println!("temperature {}", "not set".dimmed());
    }

    if let Some(stream) = &config.stream {
        println!("stream {}", stream);
    } else {
        println!("stream {}", "not set".dimmed());
    }

    Ok(())
}

// Helper function to load enhanced models for a specific provider
async fn load_provider_enhanced_models(
    provider_name: &str,
) -> Result<Vec<crate::model_metadata::ModelMetadata>> {
    use crate::model_metadata::MetadataExtractor;
    use std::fs;

    let filename = format!("models/{}.json", provider_name);

    if !std::path::Path::new(&filename).exists() {
        return Ok(Vec::new());
    }

    match fs::read_to_string(&filename) {
        Ok(json_content) => {
            match MetadataExtractor::extract_from_provider(provider_name, &json_content) {
                Ok(models) => Ok(models),
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to extract metadata from {}: {}",
                        provider_name, e
                    );
                    Ok(Vec::new())
                }
            }
        }
        Err(e) => {
            eprintln!("Warning: Failed to read {}: {}", filename, e);
            Ok(Vec::new())
        }
    }
}
