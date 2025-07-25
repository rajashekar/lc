use clap::{Parser, Subcommand};
use anyhow::Result;
use crate::{config, chat, database};
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
    pub prompt: Vec<String>,
    
    /// Provider to use for the prompt
    #[arg(short = 'p', long = "provider")]
    pub provider: Option<String>,
    
    /// Model to use for the prompt
    #[arg(short = 'm', long = "model")]
    pub model: Option<String>,
    
    /// System prompt to use (when used with direct prompt)
    #[arg(short = 's', long = "system")]
    pub system_prompt: Option<String>,
    
    /// Max tokens override (supports 'k' suffix, e.g., '2k' for 2000)
    #[arg(long = "max-tokens")]
    pub max_tokens: Option<String>,
    
    /// Temperature override (0.0 to 2.0)
    #[arg(long = "temperature")]
    pub temperature: Option<String>,
    
    /// Attach file(s) to the prompt
    #[arg(short = 'a', long = "attach")]
    pub attachments: Vec<String>,
    
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
        /// Provider to use for the chat
        #[arg(short, long)]
        provider: Option<String>,
        /// Chat ID to use or continue
        #[arg(long)]
        cid: Option<String>,
    },
    /// Global models management (alias: m)
    #[command(alias = "m")]
    Models {
        #[command(subcommand)]
        command: Option<ModelsCommands>,
        /// Search query for models (case-insensitive)
        #[arg(short = 'q', long = "query")]
        query: Option<String>,
        /// Filter models that support tools/function calling
        #[arg(long = "tools")]
        tools: bool,
        /// Filter models that support reasoning
        #[arg(long = "reasoning")]
        reasoning: bool,
        /// Filter models that support vision
        #[arg(long = "vision")]
        vision: bool,
        /// Filter models that support audio
        #[arg(long = "audio")]
        audio: bool,
        /// Filter models that support code generation
        #[arg(long = "code")]
        code: bool,
        /// Filter models with minimum context length (e.g., 128k)
        #[arg(long = "ctx")]
        context_length: Option<String>,
        /// Filter models with minimum input token length (e.g., 128k)
        #[arg(long = "input")]
        input_length: Option<String>,
        /// Filter models with minimum output token length (e.g., 128k)
        #[arg(long = "output")]
        output_length: Option<String>,
        /// Filter models with maximum input price per million tokens
        #[arg(long = "input-price")]
        input_price: Option<f64>,
        /// Filter models with maximum output price per million tokens
        #[arg(long = "output-price")]
        output_price: Option<f64>,
    },
    /// Model alias management (alias: a)
    #[command(alias = "a")]
    Alias {
        #[command(subcommand)]
        command: AliasCommands,
    },
    /// Template management (alias: t)
    #[command(alias = "t")]
    Templates {
        #[command(subcommand)]
        command: TemplateCommands,
    },
    /// Proxy server (alias: pr)
    #[command(alias = "pr")]
    Proxy {
        /// Port to listen on
        #[arg(short = 'p', long = "port", default_value = "6789")]
        port: u16,
        /// Host to bind to
        #[arg(short = 'h', long = "host", default_value = "127.0.0.1")]
        host: String,
        /// Filter by provider
        #[arg(long = "provider")]
        provider: Option<String>,
        /// Filter by specific model (can be provider:model or alias)
        #[arg(short = 'm', long = "model")]
        model: Option<String>,
        /// API key for authentication
        #[arg(short = 'k', long = "key")]
        api_key: Option<String>,
        /// Generate a random API key
        #[arg(short = 'g', long = "generate-key")]
        generate_key: bool,
    },
}

#[derive(Subcommand)]
pub enum ModelsCommands {
    /// Refresh the models cache (alias: r)
    #[command(alias = "r")]
    Refresh,
    /// Show cache information (alias: i)
    #[command(alias = "i")]
    Info,
    /// Dump raw /models responses to JSON files (alias: d)
    #[command(alias = "d")]
    Dump,
}

#[derive(Subcommand)]
pub enum AliasCommands {
    /// Add a new alias (alias: a)
    #[command(alias = "a")]
    Add {
        /// Alias name
        name: String,
        /// Provider and model in format provider:model
        target: String,
    },
    /// Remove an alias (alias: d)
    #[command(alias = "d")]
    Delete {
        /// Alias name to remove
        name: String,
    },
    /// List all aliases (alias: l)
    #[command(alias = "l")]
    List,
}

#[derive(Subcommand)]
pub enum TemplateCommands {
    /// Add a new template (alias: a)
    #[command(alias = "a")]
    Add {
        /// Template name
        name: String,
        /// Template prompt content
        prompt: String,
    },
    /// Remove a template (alias: d)
    #[command(alias = "d")]
    Delete {
        /// Template name to remove
        name: String,
    },
    /// List all templates (alias: l)
    #[command(alias = "l")]
    List,
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
        /// Custom models endpoint path (default: /models)
        #[arg(short = 'm', long = "models-path")]
        models_path: Option<String>,
        /// Custom chat completions endpoint path (default: /chat/completions)
        #[arg(short = 'c', long = "chat-path")]
        chat_path: Option<String>,
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
    /// Manage custom headers for a provider (alias: h)
    #[command(alias = "h")]
    Headers {
        /// Provider name
        provider: String,
        #[command(subcommand)]
        command: HeaderCommands,
    },
    /// Set token URL for a provider (alias: t)
    #[command(alias = "t")]
    TokenUrl {
        /// Provider name
        provider: String,
        /// Token URL for dynamic token retrieval
        url: String,
    },
}

#[derive(Subcommand)]
pub enum HeaderCommands {
    /// Add a custom header (alias: a)
    #[command(alias = "a")]
    Add {
        /// Header name
        name: String,
        /// Header value
        value: String,
    },
    /// Remove a custom header (alias: d)
    #[command(alias = "d")]
    Delete {
        /// Header name
        name: String,
    },
    /// List all custom headers (alias: l)
    #[command(alias = "l")]
    List,
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
    /// Delete/unset configuration values (alias: d)
    #[command(alias = "d")]
    Delete {
        #[command(subcommand)]
        command: DeleteCommands,
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
    /// Set system prompt (alias: s)
    #[command(alias = "s")]
    SystemPrompt {
        /// System prompt text
        prompt: String,
    },
    /// Set max tokens (alias: mt)
    #[command(alias = "mt")]
    MaxTokens {
        /// Max tokens value (supports 'k' suffix, e.g., '2k' for 2000)
        value: String,
    },
    /// Set temperature (alias: te)
    #[command(alias = "te")]
    Temperature {
        /// Temperature value (0.0 to 2.0)
        value: String,
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
    /// Get system prompt (alias: s)
    #[command(alias = "s")]
    SystemPrompt,
    /// Get max tokens (alias: mt)
    #[command(alias = "mt")]
    MaxTokens,
    /// Get temperature (alias: te)
    #[command(alias = "te")]
    Temperature,
}

#[derive(Subcommand)]
pub enum DeleteCommands {
    /// Delete default provider (alias: p)
    #[command(alias = "p")]
    Provider,
    /// Delete default model (alias: m)
    #[command(alias = "m")]
    Model,
    /// Delete system prompt (alias: s)
    #[command(alias = "s")]
    SystemPrompt,
    /// Delete max tokens (alias: mt)
    #[command(alias = "mt")]
    MaxTokens,
    /// Delete temperature (alias: te)
    #[command(alias = "te")]
    Temperature,
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
        ProviderCommands::Add { name, url, models_path, chat_path } => {
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
            
            // Sort providers by name for easier lookup
            let mut sorted_providers: Vec<_> = config.providers.iter().collect();
            sorted_providers.sort_by(|a, b| a.0.cmp(b.0));
            
            for (name, provider_config) in sorted_providers {
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
            let _provider_config = config.get_provider(&name)?;
            
            let mut config_mut = config.clone();
            let client = chat::create_authenticated_client(&mut config_mut, &name).await?;
            
            // Save config if tokens were updated
            if config_mut.get_cached_token(&name) != config.get_cached_token(&name) {
                config_mut.save()?;
            }
            
            println!("Fetching models from provider '{}'...", name);
            
            // Try to load enhanced metadata for this provider
            let enhanced_models = load_provider_enhanced_models(&name).await?;
            
            if !enhanced_models.is_empty() {
                // Display with rich metadata
                println!("\n{} Available models:", "Models:".bold());
                display_provider_models(&enhanced_models)?;
            } else {
                // Fallback to basic listing
                let models = client.list_models().await?;
                println!("\n{} Available models:", "Models:".bold());
                for model in models {
                    println!("  • {}", model.id);
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
                    println!("{} Header '{}' added to provider '{}'", "✓".green(), name, provider);
                }
                HeaderCommands::Delete { name } => {
                    config.remove_header(provider.clone(), name.clone())?;
                    config.save()?;
                    println!("{} Header '{}' removed from provider '{}'", "✓".green(), name, provider);
                }
                HeaderCommands::List => {
                    let headers = config.list_headers(&provider)?;
                    if headers.is_empty() {
                        println!("No custom headers configured for provider '{}'", provider);
                    } else {
                        println!("\n{} Custom headers for provider '{}':", "Headers:".bold().blue(), provider);
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
                    
                    // Show token usage if available
                    if let (Some(input_tokens), Some(output_tokens)) = (entry.input_tokens, entry.output_tokens) {
                        println!("{} {} input + {} output = {} total tokens",
                                 "Tokens:".bold(), input_tokens, output_tokens, input_tokens + output_tokens);
                    }
                    
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
                        
                        // Show token usage if available
                        if let (Some(input_tokens), Some(output_tokens)) = (entry.input_tokens, entry.output_tokens) {
                            println!("{} {} input + {} output = {} total tokens",
                                     "Tokens:".bold(), input_tokens, output_tokens, input_tokens + output_tokens);
                        }
                        
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
            }
        }
        Some(ConfigCommands::Delete { command }) => {
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
                                let mut model_info = vec![model.clone()];
                                
                                // Build capability indicators
                                let mut capabilities = Vec::new();
                                if model_metadata.supports_tools || model_metadata.supports_function_calling {
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
                                let model_display = if let Some(ref display_name) = model_metadata.display_name {
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
                                    let capability_strings: Vec<String> = capabilities.iter().map(|c| c.to_string()).collect();
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
        }
    }
    Ok(())
}

// Helper function to resolve model and provider from various inputs
fn resolve_model_and_provider(
    config: &config::Config,
    provider_override: Option<String>,
    model_override: Option<String>
) -> Result<(String, String)> {
    // Parse provider and model from model_override if it contains ":" or resolve alias
    // BUT if provider_override is already provided, treat model_override as literal
    let (final_provider_override, final_model_override) = if let Some(model) = &model_override {
        if provider_override.is_some() {
            // Provider is explicitly provided, treat model as literal (don't parse colons)
            (provider_override, model_override)
        } else if model.contains(':') {
            // No explicit provider, try to parse provider:model format
            let parts: Vec<&str> = model.splitn(2, ':').collect();
            if parts.len() == 2 {
                (Some(parts[0].to_string()), Some(parts[1].to_string()))
            } else {
                (provider_override, model_override)
            }
        } else {
            // Check if it's an alias
            if let Some(alias_target) = config.get_alias(model) {
                // Alias found, parse the target
                if alias_target.contains(':') {
                    let parts: Vec<&str> = alias_target.splitn(2, ':').collect();
                    if parts.len() == 2 {
                        (Some(parts[0].to_string()), Some(parts[1].to_string()))
                    } else {
                        anyhow::bail!("Invalid alias target format: '{}'. Expected 'provider:model'", alias_target);
                    }
                } else {
                    anyhow::bail!("Invalid alias target format: '{}'. Expected 'provider:model'", alias_target);
                }
            } else {
                // Not an alias, treat as regular model name
                (provider_override, model_override)
            }
        }
    } else {
        (provider_override, model_override)
    };
    
    // Determine provider and model to use
    let provider_name = if let Some(provider) = final_provider_override {
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
    
    let model_name = if let Some(model) = final_model_override {
        model
    } else {
        config.default_model.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No default model configured. Set one with 'lc config set model <name>' or use -m flag"))?
            .clone()
    };
    
    Ok((provider_name, model_name))
}

// Helper function to read and format file contents
fn read_and_format_attachments(attachments: &[String]) -> Result<String> {
    if attachments.is_empty() {
        return Ok(String::new());
    }
    
    let mut formatted_content = String::new();
    
    for (i, file_path) in attachments.iter().enumerate() {
        match std::fs::read_to_string(file_path) {
            Ok(content) => {
                if i > 0 {
                    formatted_content.push_str("\n\n");
                }
                
                // Determine file extension for better formatting
                let extension = std::path::Path::new(file_path)
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("");
                
                formatted_content.push_str(&format!("=== File: {} ===\n", file_path));
                
                // Add language hint for code files
                if !extension.is_empty() && is_code_file(extension) {
                    formatted_content.push_str(&format!("```{}\n{}\n```", extension, content));
                } else {
                    formatted_content.push_str(&content);
                }
            }
            Err(e) => {
                anyhow::bail!("Failed to read file '{}': {}", file_path, e);
            }
        }
    }
    
    Ok(formatted_content)
}

// Helper function to determine if a file extension represents code
fn is_code_file(extension: &str) -> bool {
    matches!(extension.to_lowercase().as_str(),
        "rs" | "py" | "js" | "ts" | "java" | "cpp" | "c" | "h" | "hpp" |
        "go" | "rb" | "php" | "swift" | "kt" | "scala" | "sh" | "bash" |
        "zsh" | "fish" | "ps1" | "bat" | "cmd" | "html" | "css" | "scss" |
        "sass" | "less" | "xml" | "json" | "yaml" | "yml" | "toml" | "ini" |
        "cfg" | "conf" | "sql" | "r" | "m" | "mm" | "pl" | "pm" | "lua" |
        "vim" | "dockerfile" | "makefile" | "cmake" | "gradle" | "maven"
    )
}

// Direct prompt handler
pub async fn handle_direct_prompt(prompt: String, provider_override: Option<String>, model_override: Option<String>, system_prompt_override: Option<String>, max_tokens_override: Option<String>, temperature_override: Option<String>, attachments: Vec<String>) -> Result<()> {
    let config = config::Config::load()?;
    let db = database::Database::new()?;
    
    // Read and format attachments
    let attachment_content = read_and_format_attachments(&attachments)?;
    
    // Combine prompt with attachments
    let final_prompt = if attachment_content.is_empty() {
        prompt.clone()
    } else {
        format!("{}\n\n{}", prompt, attachment_content)
    };
    
    // Determine system prompt to use (CLI override takes precedence over config)
    let system_prompt = if let Some(override_prompt) = &system_prompt_override {
        Some(config.resolve_template_or_prompt(override_prompt))
    } else if let Some(config_prompt) = &config.system_prompt {
        Some(config.resolve_template_or_prompt(config_prompt))
    } else {
        None
    };
    let system_prompt = system_prompt.as_deref();
    
    // Determine max_tokens to use (CLI override takes precedence over config)
    let max_tokens = if let Some(override_tokens) = &max_tokens_override {
        Some(config::Config::parse_max_tokens(override_tokens)?)
    } else {
        config.max_tokens
    };
    
    // Determine temperature to use (CLI override takes precedence over config)
    let temperature = if let Some(override_temp) = &temperature_override {
        Some(config::Config::parse_temperature(override_temp)?)
    } else {
        config.temperature
    };
    
    // Resolve provider and model
    let (provider_name, model_name) = resolve_model_and_provider(&config, provider_override, model_override)?;
    
    // Get provider config
    let provider_config = config.get_provider(&provider_name)?;
    
    if provider_config.api_key.is_none() {
        anyhow::bail!("No API key configured for provider '{}'. Add one with 'lc keys add {}'", provider_name, provider_name);
    }
    
    let mut config_mut = config.clone();
    let client = chat::create_authenticated_client(&mut config_mut, &provider_name).await?;
    
    // Save config if tokens were updated
    if config_mut.get_cached_token(&provider_name) != config.get_cached_token(&provider_name) {
        config_mut.save()?;
    }
    
    // Generate a session ID for this direct prompt
    let session_id = uuid::Uuid::new_v4().to_string();
    db.set_current_session_id(&session_id)?;
    
    // Send the prompt
    print!("{} ", "Thinking...".dimmed());
    io::stdout().flush()?;
    
    match chat::send_chat_request_with_validation(&client, &model_name, &final_prompt, &[], system_prompt, max_tokens, temperature, &provider_name).await {
        Ok((response, input_tokens, output_tokens)) => {
            print!("\r{}\r", " ".repeat(20)); // Clear "Thinking..."
            println!("{}", response);
            
            // Save to database with token counts (save original prompt for cleaner logs)
            if let Err(e) = db.save_chat_entry_with_tokens(&session_id, &model_name, &prompt, &response, input_tokens, output_tokens) {
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

// Direct prompt handler for piped input (treats piped content as attachment)
pub async fn handle_direct_prompt_with_piped_input(piped_content: String, provider_override: Option<String>, model_override: Option<String>, system_prompt_override: Option<String>, max_tokens_override: Option<String>, temperature_override: Option<String>, attachments: Vec<String>) -> Result<()> {
    // For piped input, we need to determine if there's a prompt in the arguments
    // Since we're called from main.rs when there's no prompt argument, we'll treat the piped content as both prompt and attachment
    // But we should provide a way to specify a prompt when piping content
    
    // For now, let's treat piped content as an attachment and ask for clarification
    let prompt = "Please analyze the following content:".to_string();
    
    // Create a temporary "attachment" from piped content
    let all_attachments = attachments;
    
    // Format piped content as an attachment
    let piped_attachment = format!("=== Piped Input ===\n{}", piped_content);
    
    let config = config::Config::load()?;
    let db = database::Database::new()?;
    
    // Read and format file attachments
    let file_attachment_content = read_and_format_attachments(&all_attachments)?;
    
    // Combine prompt with piped content and file attachments
    let final_prompt = if file_attachment_content.is_empty() {
        format!("{}\n\n{}", prompt, piped_attachment)
    } else {
        format!("{}\n\n{}\n\n{}", prompt, piped_attachment, file_attachment_content)
    };
    
    // Determine system prompt to use (CLI override takes precedence over config)
    let system_prompt = if let Some(override_prompt) = &system_prompt_override {
        Some(config.resolve_template_or_prompt(override_prompt))
    } else if let Some(config_prompt) = &config.system_prompt {
        Some(config.resolve_template_or_prompt(config_prompt))
    } else {
        None
    };
    let system_prompt = system_prompt.as_deref();
    
    // Determine max_tokens to use (CLI override takes precedence over config)
    let max_tokens = if let Some(override_tokens) = &max_tokens_override {
        Some(config::Config::parse_max_tokens(override_tokens)?)
    } else {
        config.max_tokens
    };
    
    // Determine temperature to use (CLI override takes precedence over config)
    let temperature = if let Some(override_temp) = &temperature_override {
        Some(config::Config::parse_temperature(override_temp)?)
    } else {
        config.temperature
    };
    
    // Resolve provider and model
    let (provider_name, model_name) = resolve_model_and_provider(&config, provider_override, model_override)?;
    
    // Get provider config
    let provider_config = config.get_provider(&provider_name)?;
    
    if provider_config.api_key.is_none() {
        anyhow::bail!("No API key configured for provider '{}'. Add one with 'lc keys add {}'", provider_name, provider_name);
    }
    
    let mut config_mut = config.clone();
    let client = chat::create_authenticated_client(&mut config_mut, &provider_name).await?;
    
    // Save config if tokens were updated
    if config_mut.get_cached_token(&provider_name) != config.get_cached_token(&provider_name) {
        config_mut.save()?;
    }
    
    // Generate a session ID for this direct prompt
    let session_id = uuid::Uuid::new_v4().to_string();
    db.set_current_session_id(&session_id)?;
    
    // Send the prompt
    print!("{} ", "Thinking...".dimmed());
    io::stdout().flush()?;
    
    match chat::send_chat_request_with_validation(&client, &model_name, &final_prompt, &[], system_prompt, max_tokens, temperature, &provider_name).await {
        Ok((response, input_tokens, output_tokens)) => {
            print!("\r{}\r", " ".repeat(20)); // Clear "Thinking..."
            println!("{}", response);
            
            // Save to database with token counts (save a shortened version for cleaner logs)
            let log_prompt = if piped_content.len() > 100 {
                format!("{}... (piped content)", &piped_content[..100])
            } else {
                format!("{} (piped content)", piped_content)
            };
            
            if let Err(e) = db.save_chat_entry_with_tokens(&session_id, &model_name, &log_prompt, &response, input_tokens, output_tokens) {
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
pub async fn handle_chat_command(model: String, provider: Option<String>, cid: Option<String>) -> Result<()> {
    let config = config::Config::load()?;
    let db = database::Database::new()?;
    
    // Determine session ID
    let session_id = cid.unwrap_or_else(|| {
        let new_id = uuid::Uuid::new_v4().to_string();
        db.set_current_session_id(&new_id).unwrap();
        new_id
    });
    
    // Resolve provider and model using the same logic as direct prompts
    let (provider_name, resolved_model) = resolve_model_and_provider(&config, provider, Some(model))?;
    let _provider_config = config.get_provider(&provider_name)?;
    
    let mut config_mut = config.clone();
    let client = chat::create_authenticated_client(&mut config_mut, &provider_name).await?;
    
    // Save config if tokens were updated
    if config_mut.get_cached_token(&provider_name) != config.get_cached_token(&provider_name) {
        config_mut.save()?;
    }
    
    let mut current_model = resolved_model.clone();
    
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
        
        let resolved_system_prompt = if let Some(system_prompt) = &config.system_prompt {
            Some(config.resolve_template_or_prompt(system_prompt))
        } else {
            None
        };
        
        match chat::send_chat_request_with_validation(&client, &current_model, input, &history, resolved_system_prompt.as_deref(), config.max_tokens, config.temperature, &provider_name).await {
            Ok((response, input_tokens, output_tokens)) => {
                print!("\r{}\r", " ".repeat(20)); // Clear "Thinking..."
                println!("{} {}", "Assistant:".bold().blue(), response);
                
                // Save to database with token counts
                if let Err(e) = db.save_chat_entry_with_tokens(&session_id, &current_model, input, &response, input_tokens, output_tokens) {
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

// Models command handlers
pub async fn handle_models_command(
    command: Option<ModelsCommands>,
    query: Option<String>,
    tools: bool,
    reasoning: bool,
    vision: bool,
    audio: bool,
    code: bool,
    context_length: Option<String>,
    input_length: Option<String>,
    output_length: Option<String>,
    input_price: Option<f64>,
    output_price: Option<f64>,
) -> Result<()> {
    use crate::models_cache::ModelsCache;
    use crate::model_metadata::{MetadataExtractor, ModelMetadata};
    use colored::Colorize;
    
    match command {
        Some(ModelsCommands::Refresh) => {
            let mut cache = ModelsCache::load()?;
            cache.refresh().await?;
        }
        Some(ModelsCommands::Info) => {
            let cache = ModelsCache::load()?;
            let info = cache.get_cache_info()?;
            println!("\n{}", "Models Cache Information:".bold().blue());
            println!("{}", info);
        }
        Some(ModelsCommands::Dump) => {
            dump_models_data().await?;
        }
        None => {
            // List models with enhanced metadata and filtering
            let enhanced_models = load_enhanced_models().await?;
            
            // Apply filters
            let filtered_models = apply_model_filters(
                enhanced_models,
                &query,
                tools,
                reasoning,
                vision,
                audio,
                code,
                &context_length,
                &input_length,
                &output_length,
                input_price,
                output_price,
            )?;
            
            if filtered_models.is_empty() {
                println!("No models found matching the specified criteria.");
                return Ok(());
            }
            
            // Display results
            display_enhanced_models(&filtered_models, &query)?;
        }
    }
    
    Ok(())
}

// Template command handlers
pub async fn handle_template_command(command: TemplateCommands) -> Result<()> {
    use colored::Colorize;
    
    match command {
        TemplateCommands::Add { name, prompt } => {
            let mut config = config::Config::load()?;
            config.add_template(name.clone(), prompt.clone())?;
            config.save()?;
            println!("{} Template '{}' added", "✓".green(), name);
        }
        TemplateCommands::Delete { name } => {
            let mut config = config::Config::load()?;
            config.remove_template(name.clone())?;
            config.save()?;
            println!("{} Template '{}' removed", "✓".green(), name);
        }
        TemplateCommands::List => {
            let config = config::Config::load()?;
            let templates = config.list_templates();
            
            if templates.is_empty() {
                println!("No templates configured.");
            } else {
                println!("\n{}", "Templates:".bold().blue());
                for (name, prompt) in templates {
                    let display_prompt = if prompt.len() > 60 {
                        format!("{}...", &prompt[..60])
                    } else {
                        prompt.clone()
                    };
                    println!("  {} {} -> {}", "•".blue(), name.bold(), display_prompt);
                }
            }
        }
    }
    
    Ok(())
}

// Proxy command handler
pub async fn handle_proxy_command(
    port: u16,
    host: String,
    provider: Option<String>,
    model: Option<String>,
    api_key: Option<String>,
    generate_key: bool,
) -> Result<()> {
    use crate::proxy;
    
    // Handle API key generation
    let final_api_key = if generate_key {
        let generated_key = proxy::generate_api_key();
        println!("{} Generated API key: {}", "🔑".green(), generated_key.bold());
        Some(generated_key)
    } else {
        api_key
    };
    
    // Validate provider if specified
    if let Some(ref provider_name) = provider {
        let config = config::Config::load()?;
        if !config.has_provider(provider_name) {
            anyhow::bail!("Provider '{}' not found. Add it first with 'lc providers add'", provider_name);
        }
    }
    
    // Validate model if specified (could be alias or provider:model format)
    if let Some(ref model_name) = model {
        let config = config::Config::load()?;
        
        // Check if it's an alias
        if let Some(_alias_target) = config.get_alias(model_name) {
            // Valid alias
        } else if model_name.contains(':') {
            // Check provider:model format
            let parts: Vec<&str> = model_name.splitn(2, ':').collect();
            if parts.len() == 2 {
                let provider_name = parts[0];
                if !config.has_provider(provider_name) {
                    anyhow::bail!("Provider '{}' not found in model specification '{}'", provider_name, model_name);
                }
            }
        } else {
            // Assume it's a model name for the default or specified provider
            // This will be validated when requests come in
        }
    }
    
    // Show configuration summary
    println!("\n{}", "Proxy Server Configuration:".bold().blue());
    println!("  {} {}:{}", "Address:".bold(), host, port);
    
    if let Some(ref provider_filter) = provider {
        println!("  {} {}", "Provider Filter:".bold(), provider_filter.green());
    } else {
        println!("  {} {}", "Provider Filter:".bold(), "All providers".dimmed());
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
    println!("  {} http://{}:{}/models", "•".blue(), host, port);
    println!("  {} http://{}:{}/v1/models", "•".blue(), host, port);
    println!("  {} http://{}:{}/chat/completions", "•".blue(), host, port);
    println!("  {} http://{}:{}/v1/chat/completions", "•".blue(), host, port);
    
    println!("\n{} Press Ctrl+C to stop the server\n", "💡".yellow());
    
    // Start the proxy server
    proxy::start_proxy_server(host, port, provider, model, final_api_key).await?;
    
    Ok(())
}

// Dump models data function
async fn dump_models_data() -> Result<()> {
    use crate::{config::Config, provider::OpenAIClient, chat};
    use reqwest::Client;
    use serde_json::Value;
    use std::time::Duration;
    
    println!("{} Dumping /models for each provider...", "🔍".blue());
    
    // Load configuration
    let config = Config::load()?;
    
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
                                println!("{} Saved {} models data to {}", "✅".green(), provider_name, filename);
                                successful_dumps += 1;
                            }
                            Err(e) => {
                                println!("{} Failed to save {} models data: {}", "❌".red(), provider_name, e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("{} Failed to fetch models from {}: {}", "❌".red(), provider_name, e);
                    }
                }
            }
            Err(e) => {
                println!("{} Failed to create client for {}: {}", "❌".red(), provider_name, e);
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

// Enhanced models functionality
async fn load_enhanced_models() -> Result<Vec<crate::model_metadata::ModelMetadata>> {
    use crate::model_metadata::MetadataExtractor;
    use std::fs;
    
    let mut all_models = Vec::new();
    
    // Check if models directory exists
    if !std::path::Path::new("models").exists() {
        println!("{} Models metadata not found. Run 'lc models dump' first to collect model data.", "⚠".yellow());
        return Ok(all_models);
    }
    
    // Read all JSON files from models directory
    let entries = fs::read_dir("models")?;
    
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        
        if let Some(extension) = path.extension() {
            if extension == "json" {
                if let Some(provider_name) = path.file_stem().and_then(|s| s.to_str()) {
                    match fs::read_to_string(&path) {
                        Ok(json_content) => {
                            match MetadataExtractor::extract_from_provider(provider_name, &json_content) {
                                Ok(mut models) => {
                                    all_models.append(&mut models);
                                }
                                Err(e) => {
                                    eprintln!("Warning: Failed to extract metadata from {}: {}", provider_name, e);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Warning: Failed to read {}: {}", path.display(), e);
                        }
                    }
                }
            }
        }
    }
    
    Ok(all_models)
}

fn apply_model_filters(
    models: Vec<crate::model_metadata::ModelMetadata>,
    query: &Option<String>,
    tools: bool,
    reasoning: bool,
    vision: bool,
    audio: bool,
    code: bool,
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
            model.id.to_lowercase().contains(&query_lower) ||
            model.display_name.as_ref().map_or(false, |name| name.to_lowercase().contains(&query_lower)) ||
            model.description.as_ref().map_or(false, |desc| desc.to_lowercase().contains(&query_lower))
        });
    }
    
    // Apply capability filters
    if tools {
        filtered.retain(|model| model.supports_tools || model.supports_function_calling);
    }
    
    if reasoning {
        filtered.retain(|model| model.supports_reasoning);
    }
    
    if vision {
        filtered.retain(|model| model.supports_vision);
    }
    
    if audio {
        filtered.retain(|model| model.supports_audio);
    }
    
    if code {
        filtered.retain(|model| model.supports_code);
    }
    
    // Apply context length filter
    if let Some(ref ctx_str) = context_length {
        let min_ctx = parse_token_count(ctx_str)?;
        filtered.retain(|model| {
            model.context_length.map_or(false, |ctx| ctx >= min_ctx)
        });
    }
    
    // Apply input length filter
    if let Some(ref input_str) = input_length {
        let min_input = parse_token_count(input_str)?;
        filtered.retain(|model| {
            model.max_input_tokens.map_or(false, |input| input >= min_input) ||
            model.context_length.map_or(false, |ctx| ctx >= min_input)
        });
    }
    
    // Apply output length filter
    if let Some(ref output_str) = output_length {
        let min_output = parse_token_count(output_str)?;
        filtered.retain(|model| {
            model.max_output_tokens.map_or(false, |output| output >= min_output)
        });
    }
    
    // Apply price filters
    if let Some(max_input_price) = input_price {
        filtered.retain(|model| {
            model.input_price_per_m.map_or(true, |price| price <= max_input_price)
        });
    }
    
    if let Some(max_output_price) = output_price {
        filtered.retain(|model| {
            model.output_price_per_m.map_or(true, |price| price <= max_output_price)
        });
    }
    
    // Sort by provider, then by model name
    filtered.sort_by(|a, b| {
        a.provider.cmp(&b.provider).then(a.id.cmp(&b.id))
    });
    
    Ok(filtered)
}

fn parse_token_count(input: &str) -> Result<u32> {
    let input = input.to_lowercase();
    if let Some(num_str) = input.strip_suffix('k') {
        let num: f32 = num_str.parse()
            .map_err(|_| anyhow::anyhow!("Invalid token count format: '{}'", input))?;
        Ok((num * 1000.0) as u32)
    } else if let Some(num_str) = input.strip_suffix('m') {
        let num: f32 = num_str.parse()
            .map_err(|_| anyhow::anyhow!("Invalid token count format: '{}'", input))?;
        Ok((num * 1000000.0) as u32)
    } else {
        input.parse()
            .map_err(|_| anyhow::anyhow!("Invalid token count format: '{}'", input))
    }
}

fn display_enhanced_models(models: &[crate::model_metadata::ModelMetadata], query: &Option<String>) -> Result<()> {
    use colored::Colorize;
    
    if let Some(ref search_query) = query {
        println!("\n{} Models matching '{}' ({} found):", "Search Results:".bold().blue(), search_query, models.len());
    } else {
        println!("\n{} Available models ({} total):", "Models:".bold().blue(), models.len());
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
            let capability_strings: Vec<String> = capabilities.iter().map(|c| c.to_string()).collect();
            print!(" [{}]", capability_strings.join(" "));
        }
        
        if !context_info.is_empty() {
            print!(" ({})", context_info.join(", ").dimmed());
        }
        
        println!();
    }
    
    Ok(())
}

async fn fetch_raw_models_response(client: &crate::provider::OpenAIClient, provider_config: &crate::config::ProviderConfig) -> Result<String> {
    use reqwest::Client;
    use serde_json::Value;
    use std::time::Duration;
    
    let http_client = Client::builder()
        .timeout(Duration::from_secs(60))
        .build()?;
    
    let url = format!("{}{}", provider_config.endpoint.trim_end_matches('/'), provider_config.models_path);
    
    let mut req = http_client
        .get(&url)
        .header("Authorization", format!("Bearer {}", provider_config.api_key.as_ref().unwrap()))
        .header("Content-Type", "application/json");
    
    // Add custom headers
    for (name, value) in &provider_config.headers {
        req = req.header(name, value);
    }
    
    let response = req.send().await?;
    
    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        anyhow::bail!("API request failed with status {}: {}", status, text);
    }
    
    let response_text = response.text().await?;
    
    // Pretty print the JSON for better readability
    match serde_json::from_str::<Value>(&response_text) {
        Ok(json_value) => {
            Ok(serde_json::to_string_pretty(&json_value)?)
        }
        Err(_) => {
            // If it's not valid JSON, return as-is
            Ok(response_text)
        }
    }
}

// Alias command handlers
pub async fn handle_alias_command(command: AliasCommands) -> Result<()> {
    use colored::Colorize;
    
    match command {
        AliasCommands::Add { name, target } => {
            let mut config = config::Config::load()?;
            config.add_alias(name.clone(), target.clone())?;
            config.save()?;
            println!("{} Alias '{}' added for '{}'", "✓".green(), name, target);
        }
        AliasCommands::Delete { name } => {
            let mut config = config::Config::load()?;
            config.remove_alias(name.clone())?;
            config.save()?;
            println!("{} Alias '{}' removed", "✓".green(), name);
        }
        AliasCommands::List => {
            let config = config::Config::load()?;
            let aliases = config.list_aliases();
            
            if aliases.is_empty() {
                println!("No aliases configured.");
            } else {
                println!("\n{}", "Model Aliases:".bold().blue());
                for (alias, target) in aliases {
                    println!("  {} {} -> {}", "•".blue(), alias.bold(), target);
                }
            }
        }
    }
    
    Ok(())
}

// Load enhanced models for a specific provider
async fn load_provider_enhanced_models(provider_name: &str) -> Result<Vec<crate::model_metadata::ModelMetadata>> {
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
                    eprintln!("Warning: Failed to extract metadata from {}: {}", provider_name, e);
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

// Display provider models with metadata
fn display_provider_models(models: &[crate::model_metadata::ModelMetadata]) -> Result<()> {
    use colored::Colorize;
    
    for model in models {
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
            let capability_strings: Vec<String> = capabilities.iter().map(|c| c.to_string()).collect();
            print!(" [{}]", capability_strings.join(" "));
        }
        
        if !info_parts.is_empty() {
            print!(" ({})", info_parts.join(", ").dimmed());
        }
        
        println!();
    }
    
    Ok(())
}