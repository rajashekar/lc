//! Search functionality commands matching the documentation

use crate::cli::{SearchCommands, SearchProviderCommands};
use crate::search::{SearchConfig, SearchEngine, SearchProviderType};
use anyhow::Result;
use colored::*;

/// Handle search-related commands
pub async fn handle(command: SearchCommands) -> Result<()> {
    match command {
        SearchCommands::Provider { command } => handle_provider(command).await,
        SearchCommands::Query {
            provider,
            query,
            format,
            count,
        } => handle_query(provider, query, format, count).await,
    }
}

async fn handle_provider(command: SearchProviderCommands) -> Result<()> {
    let mut config = SearchConfig::load()?;

    match command {
        SearchProviderCommands::Add { name, url } => {
            println!(
                "{} Adding search provider '{}' with URL: {}",
                "🔍".blue(),
                name.bold(),
                url.dimmed()
            );

            // Auto-detect provider type from URL
            match SearchProviderType::detect_from_url(&url) {
                Ok(provider_type) => {
                    println!(
                        "  {} Auto-detected provider type: {}",
                        "✓".green(),
                        format!("{:?}", provider_type).cyan()
                    );

                    config.add_provider(name.clone(), url, provider_type.clone())?;
                    config.save()?;

                    println!(
                        "{} Search provider '{}' added successfully",
                        "✓".green(),
                        name.bold()
                    );

                    // Show provider-specific setup instructions
                    show_provider_setup_help(&name, provider_type);
                }
                Err(e) => {
                    eprintln!(
                        "{} Failed to detect provider type from URL: {}",
                        "✗".red(),
                        e
                    );
                    eprintln!(
                        "  {} Make sure the URL matches one of the supported patterns:",
                        "ℹ".blue()
                    );
                    eprintln!("    • Brave: api.search.brave.com");
                    eprintln!("    • Exa: api.exa.ai");
                    eprintln!("    • Serper: google.serper.dev");
                    eprintln!("    • SerpApi: serpapi.com");
                    eprintln!("    • DuckDuckGo: api.duckduckgo.com");
                    eprintln!("    • Jina: s.jina.ai");
                    eprintln!("    • Tavily: api.tavily.com");
                }
            }
        }
        SearchProviderCommands::List => {
            let providers = config.list_providers();

            if providers.is_empty() {
                println!(
                    "{} No search providers configured\n{} Add one with: {}",
                    "📋".blue(),
                    "💡".yellow(),
                    "lc search provider add <name> <url>".bold()
                );
            } else {
                println!("{} Configured search providers:", "📋".blue());
                println!();

                for (name, provider) in providers {
                    let is_default = config.get_default_provider() == Some(name);
                    let default_marker = if is_default { " (default)" } else { "" };

                    println!("  {} {}{}", "•".cyan(), name.bold(), default_marker.green());
                    println!("    Type: {:?}", provider.provider_type);
                    println!("    URL: {}", provider.url.dimmed());

                    if !provider.headers.is_empty() {
                        println!("    Headers:");
                        for key in provider.headers.keys() {
                            println!("      - {}: ***", key.yellow());
                        }
                    } else {
                        println!("    Headers: {}", "None configured".dimmed());
                    }
                    println!();
                }
            }
        }
        SearchProviderCommands::Delete { name } => {
            println!(
                "{} Removing search provider '{}'...",
                "🗑".red(),
                name.bold()
            );

            config.delete_provider(&name)?;
            config.save()?;

            println!(
                "{} Search provider '{}' removed successfully",
                "✓".green(),
                name
            );
        }
        SearchProviderCommands::Set {
            provider,
            header_name,
            header_value,
        } => {
            println!(
                "{} Setting header '{}' for provider '{}'",
                "⚙".blue(),
                header_name.yellow(),
                provider.bold()
            );

            config.set_header(&provider, header_name.clone(), header_value)?;
            config.save()?;

            println!(
                "{} Header '{}' set successfully for '{}'",
                "✓".green(),
                header_name.yellow(),
                provider.bold()
            );
        }
    }

    Ok(())
}

async fn handle_query(provider: String, query: String, format: String, count: usize) -> Result<()> {
    println!(
        "{} Searching with '{}' for: {}",
        "🔍".blue(),
        provider.bold(),
        query.cyan()
    );

    let engine = SearchEngine::new()?;
    let results = engine.search(&provider, &query, Some(count)).await?;

    match format.as_str() {
        "json" => {
            println!("{}", engine.format_results_json(&results)?);
        }
        "md" | "markdown" => {
            println!("{}", engine.format_results_markdown(&results));
        }
        _ => {
            eprintln!(
                "{} Invalid format '{}'. Use 'json' or 'md'/'markdown'",
                "✗".red(),
                format
            );
        }
    }

    Ok(())
}

fn show_provider_setup_help(name: &str, provider_type: SearchProviderType) {
    println!();
    println!("{} Next steps:", "ℹ".blue());

    match provider_type {
        SearchProviderType::Brave => {
            println!("  1. Get your API key from: https://brave.com/search/api/");
            println!("  2. Set the API key:");
            println!(
                "     {}",
                format!(
                    "lc search provider set {} X-Subscription-Token YOUR_API_KEY",
                    name
                )
                .bold()
            );
        }
        SearchProviderType::Exa => {
            println!("  1. Get your API key from: https://exa.ai/");
            println!("  2. Set the API key:");
            println!(
                "     {}",
                format!("lc search provider set {} x-api-key YOUR_API_KEY", name).bold()
            );
        }
        SearchProviderType::Serper => {
            println!("  1. Get your API key from: https://serper.dev/");
            println!("  2. Set the API key:");
            println!(
                "     {}",
                format!("lc search provider set {} X-API-KEY YOUR_API_KEY", name).bold()
            );
        }
        SearchProviderType::SerpApi => {
            println!("  1. Get your API key from: https://serpapi.com/");
            println!("  2. Set the API key:");
            println!(
                "     {}",
                format!("lc search provider set {} api_key YOUR_API_KEY", name).bold()
            );
        }
        SearchProviderType::DuckDuckGo => {
            println!(
                "  {} No API key required! You can start searching immediately:",
                "✓".green()
            );
            println!(
                "     {}",
                format!("lc search query {} \"your query\"", name).bold()
            );
        }
        SearchProviderType::Jina => {
            println!("  1. Get your API key from: https://jina.ai/");
            println!("  2. Set the API key:");
            println!(
                "     {}",
                format!(
                    "lc search provider set {} Authorization \"Bearer YOUR_API_KEY\"",
                    name
                )
                .bold()
            );
            println!("  3. (Optional) Enable full content reading:");
            println!(
                "     {}",
                format!("lc search provider set {} X-Engine direct", name).bold()
            );
            println!("  4. (Optional) Enable JSON response:");
            println!(
                "     {}",
                format!("lc search provider set {} Accept application/json", name).bold()
            );
        }
        SearchProviderType::Tavily => {
            println!("  1. Get your API key from: https://tavily.com/");
            println!("  2. Set the API key:");
            println!(
                "     {}",
                format!("lc search provider set {} api-key YOUR_API_KEY", name).bold()
            );
        }
    }

    println!();
}
