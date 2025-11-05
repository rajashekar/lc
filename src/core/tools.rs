//! MCP tools integration for LLM function calling

use anyhow::Result;

// Re-export the Tool type from provider module for consistency
pub use crate::core::provider::Tool;

/// Fetch tools from specified MCP servers
pub async fn fetch_mcp_tools(tools_str: &str) -> Result<(Option<Vec<Tool>>, Vec<String>)> {
    use crate::services::mcp::McpConfig;
    use crate::services::mcp_daemon::DaemonClient;

    let server_names: Vec<&str> = tools_str.split(',').map(|s| s.trim()).collect();
    let mut all_tools = Vec::new();
    let mut valid_server_names = Vec::new();

    // Load MCP configuration
    let config = McpConfig::load().await?;

    // Use daemon client for persistent connections
    let daemon_client = DaemonClient::new()?;

    for server_name in server_names {
        if server_name.is_empty() {
            continue;
        }

        crate::debug_log!("Fetching tools from MCP server '{}'", server_name);

        // Check if server exists in configuration
        if config.get_server(server_name).is_some() {
            // Ensure server is connected via daemon
            match daemon_client.ensure_server_connected(server_name).await {
                Ok(_) => {
                    crate::debug_log!("Successfully connected to MCP server '{}'", server_name);
                    valid_server_names.push(server_name.to_string());
                }
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to connect to MCP server '{}': {}",
                        server_name, e
                    );
                    continue;
                }
            }
        } else {
            eprintln!(
                "Warning: MCP server '{}' not found in configuration",
                server_name
            );
            continue;
        }
    }

    // Get all tools from connected servers using daemon client
    for server_name in &valid_server_names {
        match daemon_client.list_tools(server_name).await {
            Ok(server_tools) => {
                if let Some(tools) = server_tools.get(server_name) {
                    crate::debug_log!(
                        "Retrieved {} tools from server '{}'",
                        tools.len(),
                        server_name
                    );

                    for tool in tools {
                        // Convert MCP tool to OpenAI tool format
                        // Simplify the schema to reduce token usage
                        let mut simplified_schema = serde_json::Map::new();

                        // Copy only essential fields from input_schema
                        if let Some(properties) = tool.input_schema.get("properties") {
                            simplified_schema
                                .insert("type".to_string(), serde_json::json!("object"));
                            simplified_schema.insert("properties".to_string(), properties.clone());

                            if let Some(required) = tool.input_schema.get("required") {
                                simplified_schema.insert("required".to_string(), required.clone());
                            }
                        } else {
                            // If no properties, use minimal schema
                            simplified_schema
                                .insert("type".to_string(), serde_json::json!("object"));
                            simplified_schema
                                .insert("properties".to_string(), serde_json::json!({}));
                        }

                        let openai_tool = crate::core::provider::Tool {
                            tool_type: "function".to_string(),
                            function: crate::core::provider::Function {
                                name: tool.name.to_string(),
                                description: tool
                                    .description
                                    .as_ref()
                                    .map(|s| s.to_string())
                                    .unwrap_or_else(|| "No description".to_string()),
                                parameters: serde_json::Value::Object(simplified_schema),
                            },
                        };

                        all_tools.push(openai_tool);
                        crate::debug_log!(
                            "Added tool '{}' from server '{}'",
                            tool.name,
                            server_name
                        );
                    }
                }
            }
            Err(e) => {
                eprintln!(
                    "Warning: Failed to list tools from MCP server '{}': {}",
                    server_name, e
                );
            }
        }
    }

    // Connections persist in daemon - no cleanup needed

    if all_tools.is_empty() {
        crate::debug_log!("No tools found from any specified MCP servers");
        Ok((None, valid_server_names))
    } else {
        crate::debug_log!("Total {} tools fetched from MCP servers", all_tools.len());
        Ok((Some(all_tools), valid_server_names))
    }
}

/// Execute a tool call via MCP
pub async fn execute_mcp_tool(
    server_name: &str,
    tool_name: &str,
    arguments: serde_json::Value,
) -> Result<serde_json::Value> {
    use crate::services::mcp_daemon::DaemonClient;

    let daemon_client = DaemonClient::new()?;

    crate::debug_log!(
        "Executing tool '{}' on server '{}' with arguments: {}",
        tool_name,
        server_name,
        arguments
    );

    match daemon_client
        .call_tool(server_name, tool_name, arguments)
        .await
    {
        Ok(result) => {
            crate::debug_log!("Tool execution successful");
            Ok(result)
        }
        Err(e) => {
            crate::debug_log!("Tool execution failed: {}", e);
            Err(e)
        }
    }
}
