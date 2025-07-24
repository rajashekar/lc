use clap::{Parser, Subcommand};
use anyhow::Result;
use crate::{config, provider, chat, database};
use colored::Colorize;
use rpassword::read_password;
use std::io::{self, Write};

#[derive(Parser)]
#[command(name = "lc")]
#[command(about = "LLM Client - A fast Rust-based LLM CLI tool")]
#[command(version = "0.1.0")]
pub struct Cli {
    /// Direct prompt to send to the default model
    #[arg(value_name = "PROMPT")]
    pub prompt: Option<String>,
    
    /// Provider to use for the prompt
    #[arg(short = 'p', long = "provider")]
    pub provider: Option<String>,
    
    /// Model to use for the prompt
    #[arg(short = 'm', long = "model")]
    pub model: Option<String>,
    
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Provider management (alias: p)
    #[command(alias = "p")]
    Providers {
        #[command(subcommand)]
        command: ProviderCommands,
    },
    /// API key management (alias: k)
    #[command(alias = "k")]
    Keys {
        #[command(subcommand)]
        command: KeyCommands,
    },
    /// Log management (alias: l)
    #[command(alias = "l")]
    Logs {
        #[command(subcommand)]
        command: LogCommands,
    },
    /// Configuration management (alias: co)
    #[command(alias = "co")]
    Config {
        #[command(subcommand)]
        command: Option<ConfigCommands>,
    },
    /// Interactive chat mode (alias: c)
    #[command(alias = "c")]
    Chat {
        /// Model to use for the chat
        #[arg(short, long)]
        model: String,
        /// Chat ID to use or continue
        #[arg(long)]
        cid: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum ProviderCommands {
    /// Add a new provider (alias: a)
    #[command(alias = "a")]
    Add {
        /// Provider name
        name: String,
        /// Provider endpoint URL
        url: String,
    },
    /// Update an existing provider (alias: u)
    #[command(alias = "u")]
    Update {
        /// Provider name
        name: String,
        /// Provider endpoint URL
        url: String,
    },
    /// Remove a provider (alias: r)
    #[command(alias = "r")]
    Remove {
        /// Provider name
        name: String,
    },
    /// List all providers (alias: l)
    #[command(alias = "l")]
    List,
    /// List available models for a provider (alias: m)
    #[command(alias = "m")]
    Models {
        /// Provider name
        name: String,
    },
}

#[derive(Subcommand)]
pub enum KeyCommands {
    /// Add API key for a provider (alias: a)
    #[command(alias = "a")]
    Add {
        /// Provider name
        name: String,
    },
    /// List providers with API keys (alias: l)
    #[command(alias = "l")]
    List,
    /// Get API key for a provider (alias: g)
    #[command(alias = "g")]
    Get {
        /// Provider name
        name: String,
    },
    /// Remove API key for a provider (alias: r)
    #[command(alias = "r")]
    Remove {
        /// Provider name
        name: String,
    },
}

#[derive(Subcommand)]
pub enum LogCommands {
    /// Show all logs (alias: sh)
    #[command(alias = "sh")]
    Show {
        /// Show minimal table format
        #[arg(long)]
        minimal: bool,
    },
    /// Show recent logs (alias: r)
    #[command(alias = "r")]
    Recent {
        #[command(subcommand)]
        command: Option<RecentCommands>,
        /// Number of recent entries to show
        #[arg(short, long, default_value = "10")]
        count: usize,
    },
    /// Show current session logs (alias: c)
    #[command(alias = "c")]
    Current,
    /// Show database statistics (alias: s)
    #[command(alias = "s")]
    Stats,
    /// Purge all logs (alias: p)
    #[command(alias = "p")]
    Purge {
        /// Confirm purge without prompt
        #[arg(long)]
        yes: bool,
    },
}

#[derive(Subcommand)]
pub enum RecentCommands {
    /// Get last answer from LLM (alias: a)
    #[command(alias = "a")]
    Answer {
        #[command(subcommand)]
        command: Option<AnswerCommands>,
    },
    /// Get last question/prompt asked to LLM (alias: q)
    #[command(alias = "q")]
    Question,
    /// Get model used in last interaction (alias: m)
    #[command(alias = "m")]
    Model,
    /// Get session ID of last interaction (alias: s)
    #[command(alias = "s")]
    Session,
}

#[derive(Subcommand)]
pub enum AnswerCommands {
    /// Extract code blocks from last answer (alias: c)
    #[command(alias = "c")]
    Code,
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Set configuration values (alias: s)
    #[command(alias = "s")]
    Set {
        #[command(subcommand)]
        command: SetCommands,
    },
    /// Get configuration values (alias: g)
    #[command(alias = "g")]
    Get {
        #[command(subcommand)]
        command: GetCommands,
    },
    /// Show configuration directory path (alias: p)
    #[command(alias = "p")]
    Path,
}

#[derive(Subcommand)]
pub enum SetCommands {
    /// Set default provider (alias: p)
    #[command(alias = "p")]
    Provider {
        /// Provider name
        name: String,
    },
    /// Set default model (alias: m)
    #[command(alias = "m")]
    Model {
        /// Model name
        name: String,
    },
}

#[derive(Subcommand)]
pub enum GetCommands {
    /// Get default provider (alias: p)
    #[command(alias = "p")]
    Provider,
    /// Get default model (alias: m)
    #[command(alias = "m")]
    Model,
}

// Helper function to extract code blocks from markdown text
fn extract_code_blocks(text: &str) -> Vec<String> {
    let mut code_blocks = Vec::new();
    let mut in_code_block = false;
    let mut current_block = String::new();
    
    for line in text.lines() {
        if line.starts_with("```") {
            if in_code_block {
                // End of code block
                if !current_block.trim().is_empty() {
                    code_blocks.push(current_block.trim().to_string());
                }
                current_block.clear();
                in_code_block = false;
            } else {
                // Start of code block
                in_code_block = true;
            }
        } else if in_code_block {
            current_block.push_str(line);
            current_block.push('\n');
        }
    }
    
    // Handle case where code block doesn't end properly
    if in_code_block && !current_block.trim().is_empty() {
        code_blocks.push(current_block.trim().to_string());
    }
    
    code_blocks
}

// Provider command handlers
pub async fn handle_provider_command(command: ProviderCommands) -> Result<()> {
    match command {
        ProviderCommands::Add { name, url } => {
            let mut config = config::Config::load()?;
            config.add_provider(name.clone(), url)?;
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
            for (name, provider_config) in &config.providers {
                let has_key = provider_config.api_key.is_some();
                let key_status = if has_key { "✓".green() } else { "✗".red() };
                println!("  {} {} - {} (API Key: {})", 
                    "•".blue(), 
                    name.bold(), 
                    provider_config.endpoint,
                    key_status
                );
            }
        }
        ProviderCommands::Models { name } => {
            let config = config::Config::load()?;
            let provider_config = config.get_provider(&name)?;
            
            let client = provider::OpenAIClient::new(
                provider_config.endpoint.clone(),
                provider_config.api_key.clone().unwrap_or_default(),
            );
            
            println!("Fetching models from provider '{}'...", name);
            let models = client.list_models().await?;
            
            println!("\n{} Available models:", "Models:".bold());
            for model in models {
                println!("  • {}", model.id);
            }
        }
    }
    Ok(())
}

// Key command handlers
pub async fn handle_key_command(command: KeyCommands) -> Result<()> {
    match command {
        KeyCommands::Add { name } => {
            let mut config = config::Config::load()?;
            
            if !config.has_provider(&name) {
                anyhow::bail!("Provider '{}' not found. Add it first with 'lc providers add'", name);
            }
            
            print!("Enter API key for {}: ", name);
            io::stdout().flush()?;
            let key = read_password()?;
            
            config.set_api_key(name.clone(), key)?;
            config.save()?;
            println!("{} API key set for provider '{}'", "✓".green(), name);
        }
        KeyCommands::Get { name } => {
            let config = config::Config::load()?;
            
            if !config.has_provider(&name) {
                anyhow::bail!("Provider '{}' not found", name);
            }
            
            let provider_config = config.get_provider(&name)?;
            if let Some(api_key) = &provider_config.api_key {
                println!("{}", api_key);
            } else {
                anyhow::bail!("No API key configured for provider '{}'", name);
            }
        }
        KeyCommands::List => {
            let config = config::Config::load()?;
            if config.providers.is_empty() {
                println!("No providers configured.");
                return Ok(());
            }
            
            println!("\n{}", "API Key Status:".bold().blue());
            for (name, provider_config) in &config.providers {
                let status = if provider_config.api_key.is_some() {
                    "✓ Configured".green()
                } else {
                    "✗ Missing".red()
                };
                println!("  {} {} - {}", "•".blue(), name.bold(), status);
            }
        }
        KeyCommands::Remove { name } => {
            let mut config = config::Config::load()?;
            
            if !config.has_provider(&name) {
                anyhow::bail!("Provider '{}' not found", name);
            }
            
            if let Some(provider_config) = config.providers.get_mut(&name) {
                provider_config.api_key = None;
            }
            config.save()?;
            println!("{} API key removed for provider '{}'", "✓".green(), name);
        }
    }
    Ok(())
}

// Log command handlers
pub async fn handle_log_command(command: LogCommands) -> Result<()> {
    let db = database::Database::new()?;
    
    match command {
        LogCommands::Show { minimal } => {
            let entries = db.get_all_logs()?;
            
            if entries.is_empty() {
                println!("No chat logs found.");
                return Ok(());
            }
            
            if minimal {
                use tabled::{Table, Tabled};
                
                #[derive(Tabled)]
                struct LogEntry {
                    #[tabled(rename = "Chat ID")]
                    chat_id: String,
                    #[tabled(rename = "Model")]
                    model: String,
                    #[tabled(rename = "Question")]
                    question: String,
                    #[tabled(rename = "Time")]
                    time: String,
                }
                
                let table_data: Vec<LogEntry> = entries.into_iter().map(|entry| {
                    LogEntry {
                        chat_id: entry.chat_id[..8].to_string(),
                        model: entry.model,
                        question: if entry.question.len() > 50 {
                            format!("{}...", &entry.question[..50])
                        } else {
                            entry.question
                        },
                        time: entry.timestamp.format("%m-%d %H:%M").to_string(),
                    }
                }).collect();
                
                let table = Table::new(table_data);
                println!("{}", table);
            } else {
                println!("\n{}", "Chat Logs:".bold().blue());
                
                for entry in entries {
                    println!("\n{} {} ({})", 
                        "Session:".bold(),
                        &entry.chat_id[..8],
                        entry.timestamp.format("%Y-%m-%d %H:%M:%S")
                    );
                    println!("{} {}", "Model:".bold(), entry.model);
                    println!("{} {}", "Q:".yellow(), entry.question);
                    println!("{} {}", "A:".green(), 
                        if entry.response.len() > 200 {
                            format!("{}...", &entry.response[..200])
                        } else {
                            entry.response
                        }
                    );
                    println!("{}", "─".repeat(80).dimmed());
                }
            }
        }
        LogCommands::Recent { command, count } => {
            match command {
                Some(RecentCommands::Answer { command }) => {
                    let entries = db.get_all_logs()?;
                    if let Some(entry) = entries.first() {
                        match command {
                            Some(AnswerCommands::Code) => {
                                let code_blocks = extract_code_blocks(&entry.response);
                                if code_blocks.is_empty() {
                                    anyhow::bail!("No code blocks found in the last answer");
                                } else {
                                    for block in code_blocks {
                                        println!("{}", block);
                                    }
                                }
                            }
                            None => {
                                println!("{}", entry.response);
                            }
                        }
                    } else {
                        anyhow::bail!("No recent logs found");
                    }
                }
                Some(RecentCommands::Question) => {
                    let entries = db.get_all_logs()?;
                    if let Some(entry) = entries.first() {
                        println!("{}", entry.question);
                    } else {
                        anyhow::bail!("No recent logs found");
                    }
                }
                Some(RecentCommands::Model) => {
                    let entries = db.get_all_logs()?;
                    if let Some(entry) = entries.first() {
                        println!("{}", entry.model);
                    } else {
                        anyhow::bail!("No recent logs found");
                    }
                }
                Some(RecentCommands::Session) => {
                    let entries = db.get_all_logs()?;
                    if let Some(entry) = entries.first() {
                        println!("{}", entry.chat_id);
                    } else {
                        anyhow::bail!("No recent logs found");
                    }
                }
                None => {
                    // Default behavior - show recent logs
                    let mut entries = db.get_all_logs()?;
                    entries.truncate(count);
                    
                    if entries.is_empty() {
                        println!("No recent logs found.");
                        return Ok(());
                    }
                    
                    println!("\n{} (showing {} entries)", "Recent Logs:".bold().blue(), entries.len());
                    
                    for entry in entries {
                        println!("\n{} {} ({})",
                            "Session:".bold(),
                            &entry.chat_id[..8],
                            entry.timestamp.format("%Y-%m-%d %H:%M:%S")
                        );
                        println!("{} {}", "Model:".bold(), entry.model);
                        println!("{} {}", "Q:".yellow(), entry.question);
                        println!("{} {}", "A:".green(),
                            if entry.response.len() > 150 {
                                format!("{}...", &entry.response[..150])
                            } else {
                                entry.response
                            }
                        );
                        println!("{}", "─".repeat(60).dimmed());
                    }
                }
            }
        }
        LogCommands::Current => {
            if let Some(session_id) = db.get_current_session_id()? {
                let history = db.get_chat_history(&session_id)?;
                
                println!("\n{} {}", "Current Session:".bold().blue(), session_id);
                println!("{} {} messages", "Messages:".bold(), history.len());
                
                for (i, entry) in history.iter().enumerate() {
                    println!("\n{} {} ({})", 
                        format!("Message {}:", i + 1).bold(),
                        entry.model,
                        entry.timestamp.format("%H:%M:%S")
                    );
                    println!("{} {}", "Q:".yellow(), entry.question);
                    println!("{} {}", "A:".green(), 
                        if entry.response.len() > 100 {
                            format!("{}...", &entry.response[..100])
                        } else {
                            entry.response.clone()
                        }
                    );
                }
            } else {
                println!("No current session found.");
            }
        }
        LogCommands::Stats => {
            let stats = db.get_stats()?;
            
            println!("\n{}", "Database Statistics:".bold().blue());
            println!();
            
            // Basic stats
            println!("{} {}", "Total Entries:".bold(), stats.total_entries);
            println!("{} {}", "Unique Sessions:".bold(), stats.unique_sessions);
            
            // File size formatting
            let file_size_str = if stats.file_size_bytes < 1024 {
                format!("{} bytes", stats.file_size_bytes)
            } else if stats.file_size_bytes < 1024 * 1024 {
                format!("{:.1} KB", stats.file_size_bytes as f64 / 1024.0)
            } else {
                format!("{:.1} MB", stats.file_size_bytes as f64 / (1024.0 * 1024.0))
            };
            println!("{} {}", "Database Size:".bold(), file_size_str);
            
            // Date range
            if let Some((earliest, latest)) = stats.date_range {
                println!("{} {} to {}",
                    "Date Range:".bold(),
                    earliest.format("%Y-%m-%d %H:%M:%S"),
                    latest.format("%Y-%m-%d %H:%M:%S")
                );
            } else {
                println!("{} {}", "Date Range:".bold(), "No entries".dimmed());
            }
            
            // Model usage
            if !stats.model_usage.is_empty() {
                println!("\n{}", "Model Usage:".bold().blue());
                for (model, count) in stats.model_usage {
                    let percentage = if stats.total_entries > 0 {
                        (count as f64 / stats.total_entries as f64) * 100.0
                    } else {
                        0.0
                    };
                    println!("  {} {} ({} - {:.1}%)",
                        "•".blue(),
                        model.bold(),
                        count,
                        percentage
                    );
                }
            }
        }
        LogCommands::Purge { yes } => {
            if !yes {
                print!("Are you sure you want to purge all logs? This cannot be undone. (y/N): ");
                io::stdout().flush()?;
                
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                
                if !input.trim().to_lowercase().starts_with('y') {
                    println!("Purge cancelled.");
                    return Ok(());
                }
            }
            
            db.purge_all_logs()?;
            println!("{} All logs purged successfully", "✓".green());
        }
    }
    Ok(())
}

// Config command handlers
pub async fn handle_config_command(command: Option<ConfigCommands>) -> Result<()> {
    match command {
        Some(ConfigCommands::Set { command }) => {
            match command {
                SetCommands::Provider { name } => {
                    let mut config = config::Config::load()?;
                    
                    if !config.has_provider(&name) {
                        anyhow::bail!("Provider '{}' not found. Add it first with 'lc providers add'", name);
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
            }
        }
        Some(ConfigCommands::Get { command }) => {
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
            }
        }
        Some(ConfigCommands::Path) => {
            let config_dir = config::Config::config_dir()?;
            println!("\n{}", "Configuration Directory:".bold().blue());
            println!("{}", config_dir.display());
            println!("\n{}", "Files:".bold().blue());
            println!("  {} config.toml", "•".blue());
            println!("  {} logs.db", "•".blue());
        }
        None => {
            // Show current configuration
            let config = config::Config::load()?;
            println!("\n{}", "Current Configuration:".bold().blue());
            
            if let Some(provider) = &config.default_provider {
                println!("provider {}", provider);
            } else {
                println!("provider {}", "not set".dimmed());
            }
            
            if let Some(model) = &config.default_model {
                println!("model {}", model);
            } else {
                println!("model {}", "not set".dimmed());
            }
        }
    }
    Ok(())
}

// Direct prompt handler
pub async fn handle_direct_prompt(prompt: String, provider_override: Option<String>, model_override: Option<String>) -> Result<()> {
    let config = config::Config::load()?;
    let db = database::Database::new()?;
    
    // Determine provider and model to use
    let provider_name = if let Some(provider) = provider_override {
        // Validate that the provider exists
        if !config.has_provider(&provider) {
            anyhow::bail!("Provider '{}' not found. Add it first with 'lc providers add'", provider);
        }
        provider
    } else {
        config.default_provider.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No default provider configured. Set one with 'lc config set provider <name>' or use -p flag"))?
            .clone()
    };
    
    let model_name = if let Some(model) = model_override {
        model
    } else {
        config.default_model.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No default model configured. Set one with 'lc config set model <name>' or use -m flag"))?
            .clone()
    };
    
    // Get provider config
    let provider_config = config.get_provider(&provider_name)?;
    
    if provider_config.api_key.is_none() {
        anyhow::bail!("No API key configured for provider '{}'. Add one with 'lc keys add {}'", provider_name, provider_name);
    }
    
    let client = provider::OpenAIClient::new(
        provider_config.endpoint.clone(),
        provider_config.api_key.clone().unwrap(),
    );
    
    // Generate a session ID for this direct prompt
    let session_id = uuid::Uuid::new_v4().to_string();
    db.set_current_session_id(&session_id)?;
    
    // Send the prompt
    print!("{} ", "Thinking...".dimmed());
    io::stdout().flush()?;
    
    match chat::send_chat_request(&client, &model_name, &prompt, &[]).await {
        Ok(response) => {
            print!("\r{}\r", " ".repeat(20)); // Clear "Thinking..."
            println!("{}", response);
            
            // Save to database
            if let Err(e) = db.save_chat_entry(&session_id, &model_name, &prompt, &response) {
                eprintln!("Warning: Failed to save chat entry: {}", e);
            }
        }
        Err(e) => {
            print!("\r{}\r", " ".repeat(20)); // Clear "Thinking..."
            anyhow::bail!("Error: {}", e);
        }
    }
    
    Ok(())
}

// Interactive chat mode
pub async fn handle_chat_command(model: String, cid: Option<String>) -> Result<()> {
    let config = config::Config::load()?;
    let db = database::Database::new()?;
    
    // Determine session ID
    let session_id = cid.unwrap_or_else(|| {
        let new_id = uuid::Uuid::new_v4().to_string();
        db.set_current_session_id(&new_id).unwrap();
        new_id
    });
    
    // Find provider for the model
    let provider_name = config.find_provider_for_model(&model)?;
    let provider_config = config.get_provider(&provider_name)?;
    
    let client = provider::OpenAIClient::new(
        provider_config.endpoint.clone(),
        provider_config.api_key.clone().unwrap_or_default(),
    );
    
    let mut current_model = model;
    
    println!("\n{} Interactive Chat Mode", "🚀".blue());
    println!("{} Session ID: {}", "📝".blue(), session_id);
    println!("{} Model: {}", "🤖".blue(), current_model);
    println!("{} Type /help for commands, /exit to quit\n", "💡".yellow());
    
    loop {
        print!("{} ", "You:".bold().green());
        io::stdout().flush()?;
        
        let mut input = String::new();
        let bytes_read = io::stdin().read_line(&mut input)?;
        
        // If we read 0 bytes, it means EOF (e.g., when input is piped)
        if bytes_read == 0 {
            println!("Goodbye! 👋");
            break;
        }
        
        let input = input.trim();
        
        if input.is_empty() {
            continue;
        }
        
        // Handle chat commands
        if input.starts_with('/') {
            match input {
                "/exit" => {
                    println!("Goodbye! 👋");
                    break;
                }
                "/clear" => {
                    db.clear_session(&session_id)?;
                    println!("{} Session cleared", "✓".green());
                    continue;
                }
                "/help" => {
                    println!("\n{}", "Available Commands:".bold().blue());
                    println!("  /exit          - Exit chat session");
                    println!("  /clear         - Clear current session");
                    println!("  /model <name>  - Change model");
                    println!("  /help          - Show this help\n");
                    continue;
                }
                _ if input.starts_with("/model ") => {
                    let new_model = input.strip_prefix("/model ").unwrap().trim();
                    if !new_model.is_empty() {
                        current_model = new_model.to_string();
                        println!("{} Model changed to: {}", "✓".green(), current_model);
                    } else {
                        println!("{} Please specify a model name", "✗".red());
                    }
                    continue;
                }
                _ => {
                    println!("{} Unknown command. Type /help for available commands", "✗".red());
                    continue;
                }
            }
        }
        
        // Send chat message
        let history = db.get_chat_history(&session_id)?;
        
        print!("{} ", "Thinking...".dimmed());
        io::stdout().flush()?;
        
        match chat::send_chat_request(&client, &current_model, input, &history).await {
            Ok(response) => {
                print!("\r{}\r", " ".repeat(20)); // Clear "Thinking..."
                println!("{} {}", "Assistant:".bold().blue(), response);
                
                // Save to database
                if let Err(e) = db.save_chat_entry(&session_id, &current_model, input, &response) {
                    eprintln!("Warning: Failed to save chat entry: {}", e);
                }
            }
            Err(e) => {
                print!("\r{}\r", " ".repeat(20)); // Clear "Thinking..."
                println!("{} Error: {}", "✗".red(), e);
            }
        }
        
        println!(); // Add spacing
    }
    
    Ok(())
}