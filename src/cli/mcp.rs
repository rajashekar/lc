//! MCP (Model Context Protocol) commands

use crate::cli::McpCommands;
use anyhow::Result;
use colored::*;

/// Handle MCP-related commands
pub async fn handle(command: McpCommands) -> Result<()> {
    match command {
        McpCommands::Start { name, command, args } => {
            println!(
                "{} Starting MCP server '{}'...",
                "🚀".cyan(),
                name.bold()
            );
            
            // First, save the server config
            use crate::services::mcp::{McpConfig, McpServerType};
            use std::collections::HashMap;
            
            let mut config = McpConfig::load().await?;
            let full_command = if args.is_empty() {
                command.clone()
            } else {
                format!("{} {}", command, args.join(" "))
            };
            
            // Determine server type based on command pattern
            let server_type = if command.starts_with("http://") || command.starts_with("https://") {
                McpServerType::Sse
            } else {
                McpServerType::Stdio
            };
            
            config.add_server_with_env(
                name.clone(),
                full_command.clone(),
                server_type,
                HashMap::new(),
            )?;
            config.save().await?;
            
            // Now connect via daemon
            let daemon_client = crate::services::mcp_daemon::DaemonClient::new()?;
            
            match daemon_client.ensure_server_connected(&name).await {
                Ok(_) => {
                    println!(
                        "{} MCP server '{}' started successfully",
                        "✓".green(),
                        name
                    );
                    println!("  Command: {}", full_command.dimmed());
                }
                Err(e) => {
                    anyhow::bail!("Failed to start MCP server '{}': {}", name, e);
                }
            }
        }
        McpCommands::Stop { name } => {
            println!(
                "{} Stopping MCP server '{}'...",
                "🛑".red(),
                name.bold()
            );
            
            let daemon_client = crate::services::mcp_daemon::DaemonClient::new()?;
            match daemon_client.close_server(&name).await {
                Ok(_) => {
                    println!(
                        "{} MCP server '{}' stopped successfully",
                        "✓".green(),
                        name
                    );
                    
                    // Also remove from config
                    use crate::services::mcp::McpConfig;
                    let mut config = McpConfig::load().await?;
                    if config.delete_server(&name).is_ok() {
                        let _ = config.save().await;
                    }
                }
                Err(e) => {
                    println!(
                        "{} Failed to stop MCP server '{}': {}",
                        "⚠️".yellow(),
                        name,
                        e
                    );
                }
            }
        }
        McpCommands::List => {
            println!("{} MCP servers:", "📋".blue());
            
            // Load MCP config to show configured servers
            use crate::services::mcp::McpConfig;
            let config = McpConfig::load().await?;
            let servers = config.list_servers();
            
            if servers.is_empty() {
                println!("  No MCP servers configured.");
                println!(
                    "\n{}",
                    "Add one with: lc mcp start <name> <command>".italic().dimmed()
                );
            } else {
                // Check daemon for active connections
                let daemon_client = crate::services::mcp_daemon::DaemonClient::new();
                let mut active_servers = vec![];
                
                // Try to get connection status for each server
                if let Ok(client) = daemon_client {
                    for (name, _) in &servers {
                        // Try to list tools as a way to check if connected
                        if client.list_tools(name).await.is_ok() {
                            active_servers.push(name.clone());
                        }
                    }
                }
                
                for (name, server_config) in servers {
                    let status = if active_servers.contains(&name) {
                        format!("{} (connected)", "✓".green())
                    } else {
                        "".to_string()
                    };
                    
                    println!(
                        "  {} {} - {:?} ({}) {}",
                        "•".blue(),
                        name.bold(),
                        server_config.server_type,
                        server_config.command_or_url.dimmed(),
                        status
                    );
                }
            }
        }
        McpCommands::Status { name } => {
            if let Some(server_name) = name {
                println!(
                    "{} Checking status of MCP server '{}'...",
                    "🔍".blue(),
                    server_name.bold()
                );
                
                // Check if server is configured
                use crate::services::mcp::McpConfig;
                let config = McpConfig::load().await?;
                
                if let Some(server_config) = config.get_server(&server_name) {
                    println!("  Configuration:");
                    println!("    Type: {:?}", server_config.server_type);
                    println!("    Command/URL: {}", server_config.command_or_url);
                    
                    // Check daemon connection status
                    let daemon_client = crate::services::mcp_daemon::DaemonClient::new()?;
                    
                    // Try to list tools as a connection test
                    match daemon_client.list_tools(&server_name).await {
                        Ok(tools_map) => {
                            println!("    Status: {} Connected", "✓".green());
                            if let Some(tools) = tools_map.get(&server_name) {
                                println!("    Available tools: {}", tools.len());
                                if !tools.is_empty() {
                                    println!("    Tools:");
                                    for tool in tools.iter().take(5) {
                                        println!("      - {}", tool.name);
                                    }
                                    if tools.len() > 5 {
                                        println!("      ... and {} more", tools.len() - 5);
                                    }
                                }
                            }
                        }
                        Err(_) => {
                            println!("    Status: {} Not connected", "✗".red());
                            println!("    Use 'lc mcp start {}' to connect", server_name);
                        }
                    }
                } else {
                    println!("{} MCP server '{}' not found", "✗".red(), server_name);
                    println!("\nAvailable servers:");
                    let servers = config.list_servers();
                    if servers.is_empty() {
                        println!("  No servers configured");
                    } else {
                        for (name, _) in servers {
                            println!("  - {}", name);
                        }
                    }
                }
            } else {
                // Show status of all servers
                println!("{} MCP server status:", "📊".blue());
                
                use crate::services::mcp::McpConfig;
                let config = McpConfig::load().await?;
                let servers = config.list_servers();
                
                if servers.is_empty() {
                    println!("  No MCP servers configured.");
                } else {
                    let daemon_client = crate::services::mcp_daemon::DaemonClient::new();
                    
                    for (name, server_config) in servers {
                        let status = if let Ok(client) = &daemon_client {
                            if client.list_tools(&name).await.is_ok() {
                                format!("{} Connected", "✓".green())
                            } else {
                                format!("{} Not connected", "✗".red())
                            }
                        } else {
                            format!("{} Daemon unavailable", "⚠️".yellow())
                        };
                        
                        println!(
                            "  {} {} ({:?}) - {}",
                            "•".blue(),
                            name.bold(),
                            server_config.server_type,
                            status
                        );
                    }
                }
            }
        }
    }
    Ok(())
}
