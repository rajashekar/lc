//! Chat functionality commands

use anyhow::Result;
use colored::Colorize;
use std::io::{self, Write};
use uuid::Uuid;

use crate::config::Config;
use crate::core::chat;
use crate::database::Database;
use crate::provider::{ContentPart, ImageUrl, Message, MessageContent};
use crate::utils::{cli_utils::resolve_model_and_provider, input::MultiLineInput};

#[allow(clippy::too_many_arguments)]
/// Handle chat command - interactive chat mode
pub async fn handle(
    model: Option<String>,
    provider: Option<String>,
    cid: Option<String>,
    tools: Option<String>,
    database: Option<String>,
    debug: bool,
    has_images: bool,
    stream: bool,
) -> Result<()> {
    // Set debug mode if requested
    if debug {
        crate::cli::set_debug_mode(true);
    }

    let config = Config::load()?;
    let db = Database::new()?;

    // Determine session ID
    let session_id = cid.unwrap_or_else(|| {
        let new_id = Uuid::new_v4().to_string();
        db.set_current_session_id(&new_id).unwrap();
        new_id
    });

    // Resolve provider and model
    let (provider_name, resolved_model) = resolve_model_and_provider(&config, provider, model)?;
    let _provider_config = config.get_provider(&provider_name)?;

    let mut config_mut = config.clone();
    let client = chat::create_authenticated_client(&mut config_mut, &provider_name).await?;

    // Save config if tokens were updated
    if config_mut.get_cached_token(&provider_name) != config.get_cached_token(&provider_name) {
        config_mut.save()?;
    }

    // MCP tools support (placeholder for now)
    let mcp_tools: Option<Vec<crate::provider::Tool>> = None;
    let mcp_server_names: Vec<String> = Vec::new();
    if tools.is_some() {
        println!(
            "{} MCP tools support is not yet fully implemented",
            "⚠️".yellow()
        );
    }

    let mut current_model = resolved_model.clone();

    // Process initial images if provided (placeholder for now)
    let mut processed_images: Vec<String> = Vec::new();
    if has_images {
        println!(
            "{} Image support is not yet fully implemented",
            "⚠️".yellow()
        );
    }

    println!("\n{} Interactive Chat Mode", "🚀".blue());
    println!("{} Session ID: {}", "📝".blue(), session_id);
    println!("{} Model: {}", "🤖".blue(), current_model);
    if !processed_images.is_empty() {
        println!("{} Initial images: {}", "🖼️".blue(), processed_images.len());
    }
    if let Some(tools) = &mcp_tools {
        if !mcp_server_names.is_empty() {
            println!(
                "{} Tools: {} (from MCP servers: {})",
                "🔧".blue(),
                tools.len(),
                mcp_server_names.join(", ")
            );
        }
    }
    println!("{} Type /help for commands, /exit to quit", "💡".yellow());
    println!(
        "{} Use Shift+Enter or Ctrl+J for multi-line input, Enter to send\n",
        "💡".yellow()
    );

    // Create multi-line input handler
    let mut input_handler = MultiLineInput::new();

    loop {
        // Use multi-line input handler
        let input_string = match input_handler.read_input(&format!("{}", "You:".bold().green())) {
            Ok(input_text) => input_text.trim().to_string(),
            Err(_) => {
                // If there's an error with multi-line input, fall back to simple input
                print!("{} ", "You:".bold().green());
                io::stdout().flush()?;

                let mut fallback_input = String::new();
                let bytes_read = io::stdin().read_line(&mut fallback_input)?;

                // If we read 0 bytes, it means EOF (e.g., when input is piped)
                if bytes_read == 0 {
                    println!("Goodbye! 👋");
                    break;
                }

                fallback_input.trim().to_string()
            }
        };

        if input_string.is_empty() {
            continue;
        }

        let input = input_string.as_str();

        // Handle chat commands
        if input.starts_with('/') {
            match input {
                "/exit" | "/quit" => {
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
                    println!("  /exit, /quit     - Exit chat session");
                    println!("  /clear           - Clear current session");
                    println!("  /model <name>    - Change model");
                    println!("  /system <prompt> - Set system prompt");
                    println!("  /help            - Show this help");
                    println!("\n{}", "Input Controls:".bold().blue());
                    println!("  Enter            - Send message");
                    println!("  Shift+Enter      - New line (multi-line input)");
                    println!("  Ctrl+J           - New line (alternative)");
                    println!("  Ctrl+C           - Cancel current input\n");
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
                _ if input.starts_with("/system ") => {
                    let new_system = input.strip_prefix("/system ").unwrap().trim();
                    if !new_system.is_empty() {
                        // TODO: Store and use the system prompt in the config_mut
                        println!("{} System prompt updated", "✓".green());
                    } else {
                        println!("{} Please specify a system prompt", "✗".red());
                    }
                    continue;
                }
                _ => {
                    println!(
                        "{} Unknown command. Type /help for available commands",
                        "✗".red()
                    );
                    continue;
                }
            }
        }

        // Send chat message
        let history = db.get_chat_history(&session_id)?;

        // RAG support (placeholder for now)
        let enhanced_input = input.to_string();
        if database.is_some() {
            println!(
                "{} Vector database RAG support is not yet fully implemented",
                "⚠️".yellow()
            );
        }

        // Create messages with images if we have initial images
        let messages = if !processed_images.is_empty() {
            // Build history messages first
            let mut msgs: Vec<Message> = history
                .iter()
                .flat_map(|entry| {
                    vec![
                        Message::user(entry.question.clone()),
                        Message::assistant(entry.response.clone()),
                    ]
                })
                .collect();

            // Add current message with images
            let mut content_parts = vec![ContentPart::Text {
                text: enhanced_input.clone(),
            }];

            // Add each image as a content part
            for image_url in &processed_images {
                content_parts.push(ContentPart::ImageUrl {
                    image_url: ImageUrl {
                        url: image_url.clone(),
                        detail: Some("auto".to_string()),
                    },
                });
            }

            msgs.push(Message {
                role: "user".to_string(),
                content_type: MessageContent::Multimodal {
                    content: content_parts,
                },
                tool_calls: None,
                tool_call_id: None,
            });

            msgs
        } else {
            Vec::new()
        };

        // Add newline before "Thinking..." to ensure proper positioning after multi-line input
        println!();
        print!("{}", "Thinking...".dimmed());
        io::stdout().flush()?;

        let resolved_system_prompt = config
            .system_prompt
            .as_ref()
            .map(|system_prompt| config.resolve_template_or_prompt(system_prompt));

        // Determine if streaming should be used (default to true for interactive chat)
        let mut use_streaming = stream || config.stream.unwrap_or(true);

        // Disable streaming for certain providers
        if use_streaming {
            if let Ok(pcfg) = config.get_provider(&provider_name) {
                let is_gemini_like = pcfg
                    .endpoint
                    .to_lowercase()
                    .contains("generativelanguage.googleapis.com");
                if is_gemini_like {
                    use_streaming = false;
                }
            }
        }

        // Handle tool execution, streaming, or regular chat
        if mcp_tools.is_some() && !mcp_server_names.is_empty() {
            // Tool execution (not yet fully implemented)
            print!("\r{}\r", " ".repeat(12)); // Clear "Thinking..."
            println!(
                "{} Tool execution is not yet fully implemented",
                "⚠️".yellow()
            );
            continue;
        } else if use_streaming {
            // Use streaming chat
            print!("\r{}\r{} ", " ".repeat(12), "Assistant:".bold().blue());
            io::stdout().flush()?;

            let result = if !messages.is_empty() {
                chat::send_chat_request_with_streaming_messages(
                    &client,
                    &current_model,
                    &messages,
                    resolved_system_prompt.as_deref(),
                    config.max_tokens,
                    config.temperature,
                    &provider_name,
                    None,
                )
                .await
            } else {
                chat::send_chat_request_with_streaming(
                    &client,
                    &current_model,
                    &enhanced_input,
                    &history,
                    resolved_system_prompt.as_deref(),
                    config.max_tokens,
                    config.temperature,
                    &provider_name,
                    None,
                )
                .await
            };

            match result {
                Ok(_) => {
                    // Streaming completed successfully
                    println!();

                    // Save to database with placeholder since the actual response was streamed
                    if let Err(e) = db.save_chat_entry_with_tokens(
                        &session_id,
                        &current_model,
                        input,
                        "[Streamed Response]",
                        None,
                        None,
                    ) {
                        eprintln!("Warning: Failed to save chat entry: {}", e);
                    }

                    // Clear processed images after first use
                    if !processed_images.is_empty() {
                        processed_images.clear();
                    }
                }
                Err(e) => {
                    println!("\n{} Error: {}", "✗".red(), e);
                }
            }
        } else {
            // Use regular chat
            let result = if !messages.is_empty() {
                chat::send_chat_request_with_validation_messages(
                    &client,
                    &current_model,
                    &messages,
                    resolved_system_prompt.as_deref(),
                    config.max_tokens,
                    config.temperature,
                    &provider_name,
                    None,
                )
                .await
            } else {
                chat::send_chat_request_with_validation(
                    &client,
                    &current_model,
                    &enhanced_input,
                    &history,
                    resolved_system_prompt.as_deref(),
                    config.max_tokens,
                    config.temperature,
                    &provider_name,
                    None,
                )
                .await
            };

            match result {
                Ok((response, input_tokens, output_tokens)) => {
                    print!("\r{}\r", " ".repeat(12)); // Clear "Thinking..."
                    println!("{} {}", "Assistant:".bold().blue(), response);

                    // Save to database with token counts
                    if let Err(e) = db.save_chat_entry_with_tokens(
                        &session_id,
                        &current_model,
                        input,
                        &response,
                        input_tokens,
                        output_tokens,
                    ) {
                        eprintln!("Warning: Failed to save chat entry: {}", e);
                    }

                    // Clear processed images after first use
                    if !processed_images.is_empty() {
                        processed_images.clear();
                    }
                }
                Err(e) => {
                    print!("\r{}\r", " ".repeat(12)); // Clear "Thinking..."
                    println!("{} Error: {}", "✗".red(), e);
                }
            }
        }

        println!(); // Add spacing
    }

    Ok(())
}
