mod cli;
mod config;
mod database;
mod provider;
mod chat;
mod error;
mod models_cache;
mod proxy;

use anyhow::Result;
use cli::{Cli, Commands};
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
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
                        cli::handle_direct_prompt(user_prompt, cli.provider, cli.model, Some(template_content.clone()), cli.max_tokens, cli.temperature, cli.attachments).await?;
                    } else {
                        // Use template content as the prompt (no additional user prompt)
                        cli::handle_direct_prompt(template_content.clone(), cli.provider, cli.model, cli.system_prompt, cli.max_tokens, cli.temperature, cli.attachments).await?;
                    }
                } else {
                    anyhow::bail!("Template '{}' not found", template_name);
                }
            } else {
                // Regular direct prompt - join all arguments
                let prompt = cli.prompt.join(" ");
                cli::handle_direct_prompt(prompt, cli.provider, cli.model, cli.system_prompt, cli.max_tokens, cli.temperature, cli.attachments).await?;
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
        (true, Some(Commands::Chat { model, provider, cid })) => {
            cli::handle_chat_command(model, provider, cid).await?;
        }
        (true, Some(Commands::Models { command, query })) => {
            cli::handle_models_command(command, query).await?;
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
        (true, None) => {
            // No subcommand or prompt provided, check if input is piped
            use std::io::{self, Read};
            
            // Check if stdin has data (piped input)
            let mut stdin = io::stdin();
            let mut buffer = String::new();
            
            // Try to read from stdin with a timeout-like approach
            // If stdin is a terminal (not piped), this will block
            // If stdin is piped, this will read the data
            match stdin.read_to_string(&mut buffer) {
                Ok(0) => {
                    // No input available, show help
                    use clap::CommandFactory;
                    let mut cmd = Cli::command();
                    cmd.print_help()?;
                }
                Ok(_) => {
                    // Input was piped, use it as a direct prompt
                    let prompt = buffer.trim().to_string();
                    if !prompt.is_empty() {
                        cli::handle_direct_prompt_with_piped_input(prompt, cli.provider, cli.model, cli.system_prompt, cli.max_tokens, cli.temperature, cli.attachments).await?;
                    } else {
                        use clap::CommandFactory;
                        let mut cmd = Cli::command();
                        cmd.print_help()?;
                    }
                }
                Err(_) => {
                    // Error reading stdin, show help
                    use clap::CommandFactory;
                    let mut cmd = Cli::command();
                    cmd.print_help()?;
                }
            }
        }
        (false, Some(_)) => {
            // Both prompt and subcommand provided, this is an error
            anyhow::bail!("Cannot provide both a direct prompt and a subcommand");
        }
    }
    
    Ok(())
}