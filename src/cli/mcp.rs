//! MCP (Model Context Protocol) commands

use crate::cli::{McpCommands, McpServerType as CliMcpServerType};
use anyhow::Result;
use colored::*;
use std::collections::HashMap;

/// Handle MCP-related commands
pub async fn handle(command: McpCommands) -> Result<()> {
    use crate::services::mcp::{McpConfig, McpServerType as ConfigMcpServerType};

    match command {
        McpCommands::Add {
            name,
            command_or_url,
            server_type,
            env,
        } => {
            let mut config = McpConfig::load().await?;

            // Convert CLI enum to config enum
            let config_server_type = match server_type {
                CliMcpServerType::Stdio => ConfigMcpServerType::Stdio,
                CliMcpServerType::Sse => ConfigMcpServerType::Sse,
                CliMcpServerType::Streamable => ConfigMcpServerType::Streamable,
            };

            // Convert env vec to HashMap
            let env_map: HashMap<String, String> = env.into_iter().collect();

            // For npx commands without -y, add it to ensure package download
            let final_command_or_url =
                if command_or_url.starts_with("npx ") && !command_or_url.contains(" -y ") {
                    command_or_url.replacen("npx ", "npx -y ", 1)
                } else {
                    command_or_url.clone()
                };

            config.add_server_with_env(
                name.clone(),
                final_command_or_url.clone(),
                config_server_type,
                env_map.clone(),
            )?;
            config.save().await?;

            println!("{} MCP server '{}' added successfully", "✓".green(), name);
            println!("  Type: {:?}", server_type);
            println!("  Command/URL: {}", final_command_or_url);
            if !env_map.is_empty() {
                println!("  Environment variables:");
                for (key, _) in env_map {
                    println!("    - {}", key);
                }
            }
        }
        McpCommands::Delete { name } => {
            let mut config = McpConfig::load().await?;

            if config.get_server(&name).is_none() {
                anyhow::bail!("MCP server '{}' not found", name);
            }

            config.delete_server(&name)?;
            config.save().await?;

            println!("{} MCP server '{}' deleted successfully", "✓".green(), name);
        }
        McpCommands::Functions { name } => {
            let config = McpConfig::load().await?;

            if config.get_server(&name).is_some() {
                println!(
                    "{} Listing functions for MCP server '{}'...",
                    "🔍".blue(),
                    name
                );

                let daemon_client = crate::services::mcp_daemon::DaemonClient::new()?;

                // First ensure the server is connected
                match daemon_client.ensure_server_connected(&name).await {
                    Ok(_) => {
                        // Now list the tools/functions
                        match daemon_client.list_tools(&name).await {
                            Ok(tools_map) => {
                                if let Some(tools) = tools_map.get(&name) {
                                    if tools.is_empty() {
                                        println!("  No functions exposed by this server.");
                                    } else {
                                        println!(
                                            "\n{} Functions exposed by '{}':",
                                            "Functions:".bold().blue(),
                                            name
                                        );
                                        for tool in tools {
                                            let description = tool
                                                .description
                                                .as_ref()
                                                .map(|d| d.to_string())
                                                .unwrap_or_else(|| {
                                                    "No description available".to_string()
                                                });
                                            println!(
                                                "  {} {} - {}",
                                                "•".blue(),
                                                tool.name.bold(),
                                                description
                                            );
                                            if !tool.input_schema.is_empty() {
                                                // Pretty print the schema as JSON Value
                                                let schema_value = serde_json::Value::Object(
                                                    (*tool.input_schema).clone(),
                                                );
                                                if let Ok(pretty) =
                                                    serde_json::to_string_pretty(&schema_value)
                                                {
                                                    for line in pretty.lines() {
                                                        println!("      {}", line.dimmed());
                                                    }
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    println!("  No functions exposed by this server.");
                                }
                            }
                            Err(e) => {
                                anyhow::bail!("Failed to list functions for '{}': {}", name, e);
                            }
                        }
                    }
                    Err(e) => {
                        anyhow::bail!("Failed to connect to MCP server '{}': {}", name, e);
                    }
                }
            } else {
                anyhow::bail!("MCP server '{}' not found in configuration", name);
            }
        }
        McpCommands::Invoke {
            name,
            function,
            args,
        } => {
            let config = McpConfig::load().await?;

            if config.get_server(&name).is_some() {
                println!(
                    "{} Invoking function '{}' on MCP server '{}'...",
                    "⚡".yellow(),
                    function.bold(),
                    name.bold()
                );

                let daemon_client = crate::services::mcp_daemon::DaemonClient::new()?;

                // First ensure the server is connected
                match daemon_client.ensure_server_connected(&name).await {
                    Ok(_) => {
                        // Parse arguments as JSON
                        let args_json = if args.is_empty() {
                            serde_json::json!({})
                        } else if args.len() == 1 {
                            // Try to parse as JSON directly
                            match serde_json::from_str::<serde_json::Value>(&args[0]) {
                                Ok(json) => json,
                                Err(_) => {
                                    // Check if it's a key=value format
                                    if args[0].contains('=') {
                                        let mut obj = serde_json::Map::new();
                                        let parts: Vec<&str> = args[0].splitn(2, '=').collect();
                                        if parts.len() == 2 {
                                            obj.insert(
                                                parts[0].to_string(),
                                                serde_json::Value::String(parts[1].to_string()),
                                            );
                                        }
                                        serde_json::Value::Object(obj)
                                    } else {
                                        // If not valid JSON and not key=value, treat as string value
                                        serde_json::json!({ "value": args[0] })
                                    }
                                }
                            }
                        } else {
                            // Multiple args - check if they are key=value pairs
                            let mut obj = serde_json::Map::new();
                            let mut all_key_value = true;

                            for arg in args.iter() {
                                if arg.contains('=') {
                                    let parts: Vec<&str> = arg.splitn(2, '=').collect();
                                    if parts.len() == 2 {
                                        // Try to parse the value as JSON first (for nested objects/arrays)
                                        let value = match serde_json::from_str::<serde_json::Value>(
                                            parts[1],
                                        ) {
                                            Ok(json_val) => json_val,
                                            Err(_) => {
                                                serde_json::Value::String(parts[1].to_string())
                                            }
                                        };
                                        obj.insert(parts[0].to_string(), value);
                                    } else {
                                        all_key_value = false;
                                        break;
                                    }
                                } else {
                                    all_key_value = false;
                                    break;
                                }
                            }

                            if !all_key_value {
                                // Fallback to indexed keys if not all args are key=value
                                obj.clear();
                                for (i, arg) in args.iter().enumerate() {
                                    obj.insert(
                                        format!("arg{}", i),
                                        serde_json::Value::String(arg.clone()),
                                    );
                                }
                            }

                            serde_json::Value::Object(obj)
                        };

                        // Invoke the function
                        match daemon_client.call_tool(&name, &function, args_json).await {
                            Ok(result) => {
                                println!("{} Function invoked successfully\n", "✓".green());

                                // Pretty print the result
                                if let Ok(pretty) = serde_json::to_string_pretty(&result) {
                                    println!("Result:");
                                    for line in pretty.lines() {
                                        println!("  {}", line);
                                    }
                                } else {
                                    println!("Result: {:?}", result);
                                }
                            }
                            Err(e) => {
                                anyhow::bail!("Failed to invoke function: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        anyhow::bail!("Failed to connect to MCP server '{}': {}", name, e);
                    }
                }
            } else {
                anyhow::bail!("MCP server '{}' not found in configuration", name);
            }
        }
        McpCommands::Start {
            name,
            command,
            args,
        } => {
            use crate::services::mcp::{McpConfig, McpServerType};
            use std::collections::HashMap;

            let mut config = McpConfig::load().await?;

            // Check if command is provided or use existing configuration
            let (full_command, server_type) = if let Some(cmd) = command {
                // New server or override existing
                println!(
                    "{} Configuring and starting MCP server '{}'...",
                    "🚀".cyan(),
                    name.bold()
                );

                let full_cmd = if args.is_empty() {
                    cmd.clone()
                } else {
                    format!("{} {}", cmd, args.join(" "))
                };

                // Determine server type based on command pattern
                let srv_type = if cmd.starts_with("http://") || cmd.starts_with("https://") {
                    McpServerType::Sse
                } else {
                    McpServerType::Stdio
                };

                // Save the configuration
                config.add_server_with_env(
                    name.clone(),
                    full_cmd.clone(),
                    srv_type.clone(),
                    HashMap::new(),
                )?;
                config.save().await?;

                (full_cmd, srv_type)
            } else {
                // Use existing configuration
                if let Some(server_config) = config.get_server(&name) {
                    println!("{} Starting MCP server '{}'...", "🚀".cyan(), name.bold());
                    (
                        server_config.command_or_url.clone(),
                        server_config.server_type.clone(),
                    )
                } else {
                    anyhow::bail!(
                        "MCP server '{}' not found in configuration. Use 'lc mcp add' to configure it first, or provide a command.",
                        name
                    );
                }
            };

            // Now connect via daemon
            let daemon_client = crate::services::mcp_daemon::DaemonClient::new()?;

            match daemon_client.ensure_server_connected(&name).await {
                Ok(_) => {
                    println!("{} MCP server '{}' started successfully", "✓".green(), name);
                    println!("  Command: {}", full_command.dimmed());
                    println!("  Type: {:?}", server_type);
                }
                Err(e) => {
                    anyhow::bail!("Failed to start MCP server '{}': {}", name, e);
                }
            }
        }
        McpCommands::Stop { name } => {
            println!("{} Stopping MCP server '{}'...", "🛑".red(), name.bold());

            let daemon_client = crate::services::mcp_daemon::DaemonClient::new()?;
            match daemon_client.close_server(&name).await {
                Ok(_) => {
                    println!("{} MCP server '{}' stopped successfully", "✓".green(), name);

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
                    "Add one with: lc mcp start <name> <command>"
                        .italic()
                        .dimmed()
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
