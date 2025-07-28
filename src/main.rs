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
mod vector_db;

use anyhow::Result;
use cli::{Cli, Commands};
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
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
                        handle_prompt_with_optional_piped_input(user_prompt, Some(template_content.clone()), piped_input, cli.provider, cli.model, cli.max_tokens, cli.temperature, cli.attachments, cli.tools, cli.vectordb).await?;
                    } else {
                        // Use template content as the prompt (no additional user prompt)
                        handle_prompt_with_optional_piped_input(template_content.clone(), cli.system_prompt, piped_input, cli.provider, cli.model, cli.max_tokens, cli.temperature, cli.attachments, cli.tools, cli.vectordb).await?;
                    }
                } else {
                    anyhow::bail!("Template '{}' not found", template_name);
                }
            } else {
                // Regular direct prompt - join all arguments
                let prompt = cli.prompt.join(" ");
                handle_prompt_with_optional_piped_input(prompt, cli.system_prompt, piped_input, cli.provider, cli.model, cli.max_tokens, cli.temperature, cli.attachments, cli.tools, cli.vectordb).await?;
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
        (true, Some(Commands::Chat { model, provider, cid, tools, database, debug })) => {
            cli::handle_chat_command(model, provider, cid, tools, database, debug).await?;
        }
        (true, Some(Commands::Models { command, query, tools, reasoning, vision, audio, code, context_length, input_length, output_length, input_price, output_price })) => {
            cli::handle_models_command(command, query, tools, reasoning, vision, audio, code, context_length, input_length, output_length, input_price, output_price).await?;
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
        (true, None) => {
            // No subcommand or prompt provided, check if input is piped
            if let Some(piped_content) = piped_input {
                // Input was piped, use it as a direct prompt
                if !piped_content.trim().is_empty() {
                    cli::handle_direct_prompt_with_piped_input(piped_content, cli.provider, cli.model, cli.system_prompt, cli.max_tokens, cli.temperature, cli.attachments, cli.tools, cli.vectordb).await?;
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
    tools: Option<String>,
    vectordb: Option<String>,
) -> Result<()> {
    if let Some(piped_content) = piped_input {
        // Combine prompt with piped input
        let combined_prompt = format!("{}\n\n=== Piped Input ===\n{}", prompt, piped_content);
        cli::handle_direct_prompt(combined_prompt, provider, model, system_prompt, max_tokens, temperature, attachments, tools, vectordb).await
    } else {
        // No piped input, use regular prompt handling
        cli::handle_direct_prompt(prompt, provider, model, system_prompt, max_tokens, temperature, attachments, tools, vectordb).await
    }
}