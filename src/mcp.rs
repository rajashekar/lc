//! Model Context Protocol (MCP) client implementation using the official Anthropic Rust SDK
//!
//! This module provides functionality to connect to and interact with MCP servers,
//! supporting both STDIO and SSE transports. It maintains backward compatibility
//! with the legacy configuration format while using the modern rmcp SDK internally.

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::process::Command;
use rmcp::{
    ServiceExt, service::RunningService, service::RoleClient,
    model::{CallToolRequestParam, ClientCapabilities, ClientInfo, Implementation, Tool},
    transport::{SseClientTransport, TokioChildProcess, ConfigureCommandExt},
};

// Legacy configuration structures for backward compatibility
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum McpServerType {
    Stdio,
    Sse,
    Streamable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub name: String,
    pub server_type: McpServerType,
    pub command_or_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    pub servers: HashMap<String, McpServerConfig>,
}

impl McpConfig {
    pub fn new() -> Self {
        Self {
            servers: HashMap::new(),
        }
    }

    pub fn load() -> Result<Self> {
        let config_dir = crate::config::Config::config_dir()?;
        let mcp_config_path = config_dir.join("mcp.toml");
        
        if !mcp_config_path.exists() {
            return Ok(Self::new());
        }
        
        let content = std::fs::read_to_string(&mcp_config_path)?;
        let config: McpConfig = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let config_dir = crate::config::Config::config_dir()?;
        std::fs::create_dir_all(&config_dir)?;
        
        let mcp_config_path = config_dir.join("mcp.toml");
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&mcp_config_path, content)?;
        Ok(())
    }

    pub fn add_server(&mut self, name: String, command_or_url: String, server_type: McpServerType) -> Result<()> {
        let server_config = McpServerConfig {
            name: name.clone(),
            server_type,
            command_or_url,
        };
        self.servers.insert(name, server_config);
        Ok(())
    }

    pub fn delete_server(&mut self, name: &str) -> Result<()> {
        if self.servers.remove(name).is_none() {
            return Err(anyhow!("MCP server '{}' not found", name));
        }
        Ok(())
    }

    pub fn get_server(&self, name: &str) -> Option<&McpServerConfig> {
        self.servers.get(name)
    }

    pub fn list_servers(&self) -> HashMap<String, &McpServerConfig> {
        self.servers.iter().map(|(k, v)| (k.clone(), v)).collect()
    }
}

// Legacy structures removed - functionality moved to SdkMcpManager

// Modern SDK-based implementation
pub struct SdkMcpManager {
    pub clients: HashMap<String, RunningService<RoleClient, ClientInfo>>,
}

// Global manager instance for persistent connections
use std::sync::Arc;
use tokio::sync::Mutex;

lazy_static::lazy_static! {
    static ref GLOBAL_MCP_MANAGER: Arc<Mutex<SdkMcpManager>> = Arc::new(Mutex::new(SdkMcpManager::new()));
}

// Global manager access functions - used by mcp_daemon module
#[allow(dead_code)]
pub async fn get_global_manager() -> Arc<Mutex<SdkMcpManager>> {
    GLOBAL_MCP_MANAGER.clone()
}

#[allow(dead_code)]
pub async fn ensure_server_connected(server_name: &str, config: SdkMcpServerConfig) -> Result<()> {
    let manager = get_global_manager().await;
    let mut manager_lock = manager.lock().await;
    
    // Check if server is already connected
    if !manager_lock.clients.contains_key(server_name) {
        crate::debug_log!("GLOBAL_MANAGER: Connecting to MCP server '{}' (not already connected)", server_name);
        manager_lock.add_server(config).await?;
        crate::debug_log!("GLOBAL_MANAGER: Successfully connected to MCP server '{}'. Total connections: {}", server_name, manager_lock.clients.len());
    } else {
        crate::debug_log!("GLOBAL_MANAGER: MCP server '{}' already connected. Total connections: {}", server_name, manager_lock.clients.len());
    }
    
    Ok(())
}

#[allow(dead_code)]
pub async fn call_global_tool(server_name: &str, tool_name: &str, arguments: serde_json::Value) -> Result<serde_json::Value> {
    let manager = get_global_manager().await;
    let manager_lock = manager.lock().await;
    
    crate::debug_log!("GLOBAL_MANAGER: Calling tool '{}' on server '{}'. Total connections: {}", tool_name, server_name, manager_lock.clients.len());
    
    if !manager_lock.clients.contains_key(server_name) {
        crate::debug_log!("GLOBAL_MANAGER: ERROR - Server '{}' not found in global manager!", server_name);
        return Err(anyhow::anyhow!("Server '{}' not found in global manager", server_name));
    }
    
    let result = manager_lock.call_tool(server_name, tool_name, arguments).await;
    
    crate::debug_log!("GLOBAL_MANAGER: Tool call completed. Connection still active: {}", manager_lock.clients.contains_key(server_name));
    
    result
}

#[allow(dead_code)]
pub async fn list_global_tools() -> Result<HashMap<String, Vec<Tool>>> {
    let manager = get_global_manager().await;
    let manager_lock = manager.lock().await;
    manager_lock.list_all_tools().await
}

#[allow(dead_code)]
pub async fn close_global_server(server_name: &str) -> Result<()> {
    let manager = get_global_manager().await;
    let mut manager_lock = manager.lock().await;
    
    if let Some(client) = manager_lock.clients.remove(server_name) {
        let _ = client.cancel().await;
        crate::debug_log!("Closed connection to MCP server '{}'", server_name);
    }
    
    Ok(())
}

impl SdkMcpManager {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }

    pub async fn add_server(&mut self, config: SdkMcpServerConfig) -> Result<()> {
        let client_info = ClientInfo {
            protocol_version: Default::default(),
            capabilities: ClientCapabilities::default(),
            client_info: Implementation {
                name: "lc-mcp-client".to_string(),
                version: "0.1.0".to_string(),
            },
        };

        let client = match config.transport {
            SdkMcpTransport::Stdio { command, args, env, cwd } => {
                let mut cmd = Command::new(&command);
                if let Some(args) = args {
                    cmd.args(args);
                }
                if let Some(env) = env {
                    for (key, value) in env {
                        cmd.env(key, value);
                    }
                }
                if let Some(cwd) = cwd {
                    cmd.current_dir(cwd);
                }

                let transport = TokioChildProcess::new(cmd.configure(|_| {}))?;
                client_info.serve(transport).await?
            }
            SdkMcpTransport::Sse { url } => {
                let transport = SseClientTransport::start(url.as_str()).await?;
                client_info.serve(transport).await?
            }
        };

        self.clients.insert(config.name, client);
        Ok(())
    }

    pub async fn list_all_tools(&self) -> Result<HashMap<String, Vec<Tool>>> {
        let mut all_tools = HashMap::new();

        for (server_name, client) in &self.clients {
            match client.list_tools(Default::default()).await {
                Ok(tools_result) => {
                    all_tools.insert(server_name.clone(), tools_result.tools);
                }
                Err(e) => {
                    eprintln!("Warning: Failed to list tools from server '{}': {}", server_name, e);
                }
            }
        }

        Ok(all_tools)
    }

    pub async fn call_tool(&self, server_name: &str, tool_name: &str, arguments: serde_json::Value) -> Result<serde_json::Value> {
        let client = self.clients.get(server_name)
            .ok_or_else(|| anyhow!("Server '{}' not found", server_name))?;

        let result = client.call_tool(CallToolRequestParam {
            name: tool_name.to_string().into(),
            arguments: arguments.as_object().cloned(),
        }).await?;

        // Convert the result to a JSON value
        Ok(serde_json::to_value(result)?)
    }

}

// SDK configuration structures
#[derive(Debug, Clone)]
pub struct SdkMcpServerConfig {
    pub name: String,
    pub transport: SdkMcpTransport,
}

#[derive(Debug, Clone)]
pub enum SdkMcpTransport {
    Stdio {
        command: String,
        args: Option<Vec<String>>,
        env: Option<HashMap<String, String>>,
        cwd: Option<PathBuf>,
    },
    Sse {
        url: String,
    },
}

// Helper functions to create SDK configurations
pub fn create_stdio_server_config(
    name: String,
    command_parts: Vec<String>,
    env: Option<HashMap<String, String>>,
    cwd: Option<PathBuf>,
) -> SdkMcpServerConfig {
    let (command, args) = if command_parts.is_empty() {
        ("echo".to_string(), None)
    } else if command_parts.len() == 1 {
        (command_parts[0].clone(), None)
    } else {
        (command_parts[0].clone(), Some(command_parts[1..].to_vec()))
    };

    SdkMcpServerConfig {
        name,
        transport: SdkMcpTransport::Stdio {
            command,
            args,
            env,
            cwd,
        },
    }
}

pub fn create_sse_server_config(name: String, url: String) -> SdkMcpServerConfig {
    SdkMcpServerConfig {
        name,
        transport: SdkMcpTransport::Sse { url },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_config_creation() {
        let config = McpConfig::new();
        assert!(config.servers.is_empty());
    }

    #[test]
    fn test_add_server() {
        let mut config = McpConfig::new();
        config.add_server(
            "test-server".to_string(),
            "echo test".to_string(),
            McpServerType::Stdio,
        ).unwrap();

        assert_eq!(config.servers.len(), 1);
        let server = config.get_server("test-server").unwrap();
        assert_eq!(server.name, "test-server");
        assert_eq!(server.command_or_url, "echo test");
        assert_eq!(server.server_type, McpServerType::Stdio);
    }

    #[test]
    fn test_sdk_manager_creation() {
        let manager = SdkMcpManager::new();
        assert!(manager.clients.is_empty());
    }

    #[test]
    fn test_create_stdio_config() {
        let config = create_stdio_server_config(
            "test".to_string(),
            vec!["echo".to_string(), "hello".to_string()],
            None,
            None,
        );
        assert_eq!(config.name, "test");
        match config.transport {
            SdkMcpTransport::Stdio { command, args, .. } => {
                assert_eq!(command, "echo");
                assert_eq!(args, Some(vec!["hello".to_string()]));
            }
            _ => panic!("Expected Stdio transport"),
        }
    }

    #[test]
    fn test_create_sse_config() {
        let config = create_sse_server_config(
            "test".to_string(),
            "http://localhost:8080/sse".to_string(),
        );
        assert_eq!(config.name, "test");
        match config.transport {
            SdkMcpTransport::Sse { url } => {
                assert_eq!(url, "http://localhost:8080/sse");
            }
            _ => panic!("Expected SSE transport"),
        }
    }
}