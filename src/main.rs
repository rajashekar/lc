mod cli;
mod config;
mod database;
mod provider;
mod chat;
mod error;
mod models_cache;
mod model_metadata;
mod proxy;
mod token_utils;
mod unified_cache;
mod mcp;
mod mcp_daemon;
mod vector_db;
mod webchatproxy;
mod sync;
mod search;
mod readers;
mod image_utils;
mod dump_metadata;

use anyhow::Result;
use cli::{Cli, Commands};
use clap::Parser;
use database::{Database, ChatEntry};

#[derive(Debug, Clone)]
struct ChatMessage {
    role: String,
    content: String,
    model: Option<String>,
}

impl ChatMessage {
    fn new_user(content: String, model: Option<String>) -> Self {
        Self {
            role: "user".to_string(),
            content,
            model,
        }
    }
    
    fn new_assistant(content: String, model: Option<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content,
            model,
        }
    }
}

// Helper functions for database operations
async fn get_current_session() -> Result<Option<String>> {
    let db = Database::new()?;
    db.get_current_session_id()
}

async fn get_conversation_history(session_id: &str) -> Result<Vec<ChatMessage>> {
    let db = Database::new()?;
    let entries = db.get_chat_history(session_id)?;
    
    // Pre-allocate with known capacity to avoid reallocations
    let mut messages = Vec::with_capacity(entries.len() * 2);
    
    for entry in entries {
        let model_ref = Some(entry.model.clone());
        
        // Add user message - avoid cloning model twice
        messages.push(ChatMessage::new_user(entry.question, model_ref.clone()));
        
        // Add assistant message - reuse the cloned model
        messages.push(ChatMessage::new_assistant(entry.response, model_ref));
    }
    
    Ok(messages)
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize model metadata configuration files
    if let Err(e) = model_metadata::initialize_model_metadata_config() {
        eprintln!("Warning: Failed to initialize model metadata config: {}", e);
    }
    
    // Check for daemon mode first
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "--mcp-daemon" {
        // Run in daemon mode
        let mut daemon = mcp_daemon::McpDaemon::new()?;
        daemon.start().await?;
        return Ok(());
    }
    
    let cli = Cli::parse();
    
    // Set debug mode if flag is provided
    cli::set_debug_mode(cli.debug);
    
    // Check for piped input first
    let piped_input = check_for_piped_input()?;
    
    // Handle direct prompt or subcommands
    match (cli.prompt.is_empty(), cli.command) {
        (false, None) => {
            // Direct prompt(s) provided as arguments
            let first_arg = &cli.prompt[0];
            
            // Check if first argument is a template reference (t:template_name)
            if let Some(template_name) = first_arg.strip_prefix("t:") {
                // Load config to resolve template
                let config = config::Config::load()?;
                if let Some(template_content) = config.get_template(template_name) {
                    if cli.prompt.len() > 1 {
                        // Use template as system prompt and remaining args as user prompt
                        let user_prompt = cli.prompt[1..].join(" ");
                        handle_prompt_with_optional_piped_input(user_prompt, Some(template_content.clone()), piped_input, cli.provider, cli.model, cli.max_tokens, cli.temperature, cli.attachments, cli.images, cli.tools, cli.vectordb, cli.continue_session, cli.chat_id, cli.use_search, cli.stream).await?;
                    } else {
                        // Use template content as the prompt (no additional user prompt)
                        handle_prompt_with_optional_piped_input(template_content.clone(), cli.system_prompt, piped_input, cli.provider, cli.model, cli.max_tokens, cli.temperature, cli.attachments, cli.images, cli.tools, cli.vectordb, cli.continue_session, cli.chat_id, cli.use_search, cli.stream).await?;
                    }
                } else {
                    anyhow::bail!("Template '{}' not found", template_name);
                }
            } else {
                // Regular direct prompt - join all arguments
                let prompt = cli.prompt.join(" ");
                handle_prompt_with_optional_piped_input(prompt, cli.system_prompt, piped_input, cli.provider, cli.model, cli.max_tokens, cli.temperature, cli.attachments, cli.images, cli.tools, cli.vectordb, cli.continue_session, cli.chat_id, cli.use_search, cli.stream).await?;
            }
        }
        (true, Some(Commands::Providers { command })) => {
            cli::handle_provider_command(command).await?;
        }
        (true, Some(Commands::Keys { command })) => {
            cli::handle_key_command(command).await?;
        }
        (true, Some(Commands::Logs { command })) => {
            cli::handle_log_command(command).await?;
        }
        (true, Some(Commands::Config { command })) => {
            cli::handle_config_command(command).await?;
        }
        (true, Some(Commands::Chat { model, provider, cid, tools, database, debug, images })) => {
            cli::handle_chat_command(model, provider, cid, tools, database, debug, images, cli.stream).await?;
        }
        (true, Some(Commands::Models { command, query, tools, reasoning, vision, audio, code, context_length, input_length, output_length, input_price, output_price })) => {
            // Convert individual boolean flags to tags string
            let mut tags = Vec::new();
            if tools { tags.push("tools"); }
            if reasoning { tags.push("reasoning"); }
            if vision { tags.push("vision"); }
            if audio { tags.push("audio"); }
            if code { tags.push("code"); }
            
            let tags_string = if tags.is_empty() {
                None
            } else {
                Some(tags.join(","))
            };
            
            cli::handle_models_command(command, query, tags_string, context_length, input_length, output_length, input_price, output_price).await?;
        }
        (true, Some(Commands::Alias { command })) => {
            cli::handle_alias_command(command).await?;
        }
        (true, Some(Commands::Templates { command })) => {
            cli::handle_template_command(command).await?;
        }
        (true, Some(Commands::Proxy { port, host, provider, model, api_key, generate_key })) => {
            cli::handle_proxy_command(port, host, provider, model, api_key, generate_key).await?;
        }
        (true, Some(Commands::Mcp { command })) => {
            cli::handle_mcp_command(command).await?;
        }
        (true, Some(Commands::Embed { model, provider, database, files, text, debug })) => {
            cli::handle_embed_command(model, provider, database, files, text, debug).await?;
        }
        (true, Some(Commands::Similar { model, provider, database, limit, query })) => {
            cli::handle_similar_command(model, provider, database, limit, query).await?;
        }
        (true, Some(Commands::Vectors { command })) => {
            cli::handle_vectors_command(command).await?;
        }
        (true, Some(Commands::WebChatProxy { command })) => {
            cli::handle_webchatproxy_command(command).await?;
        }
        (true, Some(Commands::Sync { command })) => {
            cli::handle_sync_command(command).await?;
        }
        (true, Some(Commands::Search { command })) => {
            cli::handle_search_command(command).await?;
        }
        (true, Some(Commands::Image { prompt, model, provider, size, count, output, debug })) => {
            cli::handle_image_command(prompt, model, provider, size, count, output, debug).await?;
        }
        (true, Some(Commands::DumpMetadata { provider, list })) => {
            cli::handle_dump_metadata_command(provider, list).await?;
        }
        (true, None) => {
            // No subcommand or prompt provided, check if input is piped
            if let Some(piped_content) = piped_input {
                // Input was piped, use it as a direct prompt
                if !piped_content.trim().is_empty() {
                    handle_prompt_with_optional_piped_input_continue(piped_content, cli.system_prompt, cli.provider, cli.model, cli.max_tokens, cli.temperature, cli.attachments, cli.images, cli.tools, cli.vectordb, cli.continue_session, cli.chat_id, cli.use_search, cli.stream).await?;
                } else {
                    use clap::CommandFactory;
                    let mut cmd = Cli::command();
                    cmd.print_help()?;
                }
            } else {
                // No input available, show help
                use clap::CommandFactory;
                let mut cmd = Cli::command();
                cmd.print_help()?;
            }
        }
        (false, Some(_)) => {
            // Both prompt and subcommand provided, this is an error
            anyhow::bail!("Cannot provide both a direct prompt and a subcommand");
        }
    }
    
    Ok(())
}

// Helper function to check for piped input
fn check_for_piped_input() -> Result<Option<String>> {
    use std::io::{self, Read};
    
    // Check if stdin is a terminal (interactive) or piped
    if atty::is(atty::Stream::Stdin) {
        // stdin is a terminal, no piped input
        return Ok(None);
    }
    
    // stdin is piped, read the content
    let mut stdin = io::stdin();
    let mut buffer = String::new();
    
    match stdin.read_to_string(&mut buffer) {
        Ok(0) => Ok(None), // No input available
        Ok(_) => Ok(Some(buffer)), // Input was piped
        Err(_) => Ok(None), // Error reading stdin
    }
}

// Helper function to handle prompt with optional piped input
async fn handle_prompt_with_optional_piped_input(
    prompt: String,
    system_prompt: Option<String>,
    piped_input: Option<String>,
    provider: Option<String>,
    model: Option<String>,
    max_tokens: Option<String>,
    temperature: Option<String>,
    attachments: Vec<String>,
    images: Vec<String>,
    tools: Option<String>,
    vectordb: Option<String>,
    continue_session: bool,
    chat_id: Option<String>,
    use_search: Option<String>,
    stream: bool,
) -> Result<()> {
    if let Some(piped_content) = piped_input {
        // Combine prompt with piped input
        let combined_prompt = format!("{}\n\n=== Piped Input ===\n{}", prompt, piped_content);
        handle_direct_prompt_with_session(combined_prompt, provider, model, system_prompt, max_tokens, temperature, attachments, images, tools, vectordb, continue_session, chat_id, use_search, stream).await
    } else {
        // No piped input, use regular prompt handling
        handle_direct_prompt_with_session(prompt, provider, model, system_prompt, max_tokens, temperature, attachments, images, tools, vectordb, continue_session, chat_id, use_search, stream).await
    }
}

// Helper function to handle piped input with continue support
async fn handle_prompt_with_optional_piped_input_continue(
    piped_content: String,
    system_prompt: Option<String>,
    provider: Option<String>,
    model: Option<String>,
    max_tokens: Option<String>,
    temperature: Option<String>,
    attachments: Vec<String>,
    images: Vec<String>,
    tools: Option<String>,
    vectordb: Option<String>,
    continue_session: bool,
    chat_id: Option<String>,
    use_search: Option<String>,
    stream: bool,
) -> Result<()> {
    if continue_session || chat_id.is_some() {
        // Use piped content as prompt with session continuation
        handle_direct_prompt_with_session(piped_content, provider, model, system_prompt, max_tokens, temperature, attachments, images, tools, vectordb, continue_session, chat_id, use_search, stream).await
    } else {
        // Use existing piped input handler
        cli::handle_direct_prompt_with_piped_input(piped_content, provider, model, system_prompt, max_tokens, temperature, attachments, images, tools, vectordb, use_search, stream).await
    }
}

async fn handle_direct_prompt_with_session(
    prompt: String,
    provider: Option<String>,
    model: Option<String>,
    system_prompt: Option<String>,
    max_tokens: Option<String>,
    temperature: Option<String>,
    attachments: Vec<String>,
    images: Vec<String>,
    tools: Option<String>,
    vectordb: Option<String>,
    continue_session: bool,
    chat_id: Option<String>,
    use_search: Option<String>,
    stream: bool,
) -> Result<()> {
    if continue_session {
        // Get or create session ID
        let session_id = if let Some(cid) = chat_id {
            cid
        } else {
            // Get current session from database
            match get_current_session().await {
                Ok(Some(session)) => session,
                Ok(None) => {
                    eprintln!("No current session found. Start a new conversation first.");
                    return Ok(());
                }
                Err(e) => {
                    eprintln!("Error retrieving current session: {}", e);
                    return Ok(());
                }
            }
        };

        // Get conversation history
        let history = match get_conversation_history(&session_id).await {
            Ok(history) => history,
            Err(e) => {
                eprintln!("Error retrieving conversation history: {}", e);
                return Ok(());
            }
        };

        if history.is_empty() {
            eprintln!("No conversation history found for session: {}", session_id);
            return Ok(());
        }

        // Use provided model/provider if available, otherwise try to infer from history
        let final_model = model.or_else(|| {
            // Get model from the first message in history
            history.first().and_then(|msg| msg.model.clone())
        });

        let final_provider = provider.or_else(|| {
            // If model contains provider prefix, extract it
            if let Some(ref m) = final_model {
                if m.contains(':') {
                    Some(m.split(':').next().unwrap().to_string())
                } else {
                    None
                }
            } else {
                None
            }
        });

        if final_model.is_none() {
            eprintln!("Could not determine model. Please specify with -m/--model");
            return Ok(());
        }

        if final_provider.is_none() {
            eprintln!("Could not determine provider. Please specify with -p/--provider or use full model format (provider:model)");
            return Ok(());
        }

        handle_session_prompt(prompt, final_provider, final_model, system_prompt, max_tokens, temperature, attachments, images, tools, vectordb, session_id, history, use_search, stream).await
    } else {
        // Use regular prompt handling
        cli::handle_direct_prompt(prompt, provider, model, system_prompt, max_tokens, temperature, attachments, images, tools, vectordb, use_search, stream).await
    }
}

async fn handle_session_prompt(
    prompt: String,
    provider: Option<String>,
    model: Option<String>,
    system_prompt: Option<String>,
    max_tokens: Option<String>,
    temperature: Option<String>,
    _attachments: Vec<String>,
    _images: Vec<String>,
    _tools: Option<String>,
    _vectordb: Option<String>,
    _session_id: String,
    history: Vec<ChatMessage>,
    _use_search: Option<String>,
    _stream: bool,
) -> Result<()> {
    // Convert ChatMessage history to ChatEntry format expected by the chat module
    let mut chat_entries = Vec::new();
    let mut i = 0;
    while i < history.len() {
        if i + 1 < history.len() && history[i].role == "user" && history[i + 1].role == "assistant" {
            // We have a user-assistant pair
            let entry = ChatEntry {
                chat_id: "temp".to_string(),
                model: history[i].model.clone().unwrap_or_default(),
                question: history[i].content.clone(),
                response: history[i + 1].content.clone(),
                timestamp: chrono::Utc::now(),
                input_tokens: None,
                output_tokens: None,
            };
            chat_entries.push(entry);
            i += 2;
        } else {
            i += 1;
        }
    }
    
    // Parse parameters
    let max_tokens_parsed = max_tokens.as_ref().and_then(|s| s.parse().ok());
    let temperature_parsed = temperature.as_ref().and_then(|s| s.parse().ok());
    
    // Get provider and model - if not provided, try to infer from history
    let (provider_name, model_name) = if let (Some(p), Some(m)) = (&provider, &model) {
        // Both provided explicitly
        (p.clone(), m.clone())
    } else if let Some(first_msg) = history.first() {
        if let Some(full_model) = &first_msg.model {
            if full_model.contains(':') {
                // Model is in format "provider:model"
                let parts: Vec<&str> = full_model.split(':').collect();
                let inferred_provider = parts[0].to_string();
                let inferred_model = full_model.clone(); // Keep full model name
                
                (
                    provider.unwrap_or(inferred_provider),
                    model.unwrap_or(inferred_model)
                )
            } else {
                // Model doesn't contain provider prefix
                (
                    provider.unwrap_or_else(|| "openai".to_string()),
                    model.unwrap_or_else(|| full_model.clone())
                )
            }
        } else {
            // No model in history
            (
                provider.unwrap_or_else(|| "openai".to_string()),
                model.unwrap_or_else(|| "gpt-3.5-turbo".to_string())
            )
        }
    } else {
        // No history available
        (
            provider.unwrap_or_else(|| "openai".to_string()),
            model.unwrap_or_else(|| "gpt-3.5-turbo".to_string())
        )
    };
    
    // Create authenticated client
    let mut config = config::Config::load()?;
    let client = chat::create_authenticated_client(&mut config, &provider_name).await?;
    
    // Strip provider prefix from model name for API call
    let api_model_name = if model_name.contains(':') {
        model_name.split(':').nth(1).unwrap_or(&model_name).to_string()
    } else {
        model_name.clone()
    };
    
    // Send chat request with history
    let (response, _input_tokens, _output_tokens) = chat::send_chat_request_with_validation(
        &client,
        &api_model_name,
        &prompt,
        &chat_entries,
        system_prompt.as_deref(),
        max_tokens_parsed,
        temperature_parsed,
        &provider_name,
        None, // No tools for now
    ).await?;
    
    // Print the response
    println!("{}", response);
    
    Ok(())
}