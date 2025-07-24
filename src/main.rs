mod cli;
mod config;
mod database;
mod provider;
mod chat;
mod error;
mod models_cache;

use anyhow::Result;
use cli::{Cli, Commands};
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Handle direct prompt or subcommands
    match (cli.prompt, cli.command) {
        (Some(prompt), None) => {
            // Direct prompt provided as argument
            cli::handle_direct_prompt(prompt, cli.provider, cli.model).await?;
        }
        (None, Some(Commands::Providers { command })) => {
            cli::handle_provider_command(command).await?;
        }
        (None, Some(Commands::Keys { command })) => {
            cli::handle_key_command(command).await?;
        }
        (None, Some(Commands::Logs { command })) => {
            cli::handle_log_command(command).await?;
        }
        (None, Some(Commands::Config { command })) => {
            cli::handle_config_command(command).await?;
        }
        (None, Some(Commands::Chat { model, cid })) => {
            cli::handle_chat_command(model, cid).await?;
        }
        (None, Some(Commands::Models { command, query })) => {
            cli::handle_models_command(command, query).await?;
        }
        (None, None) => {
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
                        cli::handle_direct_prompt(prompt, cli.provider, cli.model).await?;
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
        (Some(_), Some(_)) => {
            // Both prompt and subcommand provided, this is an error
            anyhow::bail!("Cannot provide both a direct prompt and a subcommand");
        }
    }
    
    Ok(())
}