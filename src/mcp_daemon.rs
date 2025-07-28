//! MCP Daemon Service for persistent MCP connections across CLI invocations
//!
//! This module provides a background daemon that maintains persistent MCP server
//! connections, allowing browser sessions and other stateful resources to persist
//! across multiple CLI command invocations.

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::net::{UnixListener, UnixStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::mcp::{SdkMcpManager, McpConfig, McpServerType, create_stdio_server_config, create_sse_server_config};

#[derive(Debug, Serialize, Deserialize)]
pub enum DaemonRequest {
    ListTools { server_name: String },
    CallTool { server_name: String, tool_name: String, arguments: serde_json::Value },
    EnsureServerConnected { server_name: String },
    CloseServer { server_name: String },
    ListConnectedServers,
    Shutdown,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DaemonResponse {
    Tools(HashMap<String, Vec<rmcp::model::Tool>>),
    ToolResult(serde_json::Value),
    ServerConnected,
    ServerClosed,
    ConnectedServers(Vec<String>),
    Success,
    Error(String),
}

pub struct McpDaemon {
    manager: SdkMcpManager,
    socket_path: PathBuf,
}

impl McpDaemon {
    pub fn new() -> Result<Self> {
        let socket_path = Self::get_socket_path()?;
        Ok(Self {
            manager: SdkMcpManager::new(),
            socket_path,
        })
    }

    pub fn get_socket_path() -> Result<PathBuf> {
        let config_dir = crate::config::Config::config_dir()?;
        Ok(config_dir.join("mcp_daemon.sock"))
    }

    pub async fn start(&mut self) -> Result<()> {
        // Remove existing socket if it exists
        if self.socket_path.exists() {
            std::fs::remove_file(&self.socket_path)?;
        }

        let listener = UnixListener::bind(&self.socket_path)?;
        crate::debug_log!("MCP Daemon started, listening on {:?}", self.socket_path);

        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    if let Err(e) = self.handle_client(stream).await {
                        crate::debug_log!("Error handling client: {}", e);
                    }
                }
                Err(e) => {
                    crate::debug_log!("Error accepting connection: {}", e);
                }
            }
        }
    }

    async fn handle_client(&mut self, mut stream: UnixStream) -> Result<()> {
        // Read request with larger buffer
        let mut buffer = vec![0; 32768]; // Increased from 8192 to 32768
        let n = stream.read(&mut buffer).await?;
        
        if n == 0 {
            return Ok(());
        }

        let request: DaemonRequest = serde_json::from_slice(&buffer[..n])?;
        crate::debug_log!("Daemon received request: {:?}", request);

        let response = self.process_request(request).await;
        let response_data = serde_json::to_vec(&response)?;
        
        // Write response length first, then response data
        let response_len = response_data.len() as u32;
        stream.write_all(&response_len.to_le_bytes()).await?;
        stream.write_all(&response_data).await?;
        stream.flush().await?;

        Ok(())
    }

    async fn process_request(&mut self, request: DaemonRequest) -> DaemonResponse {
        match request {
            DaemonRequest::EnsureServerConnected { server_name } => {
                match self.ensure_server_connected(&server_name).await {
                    Ok(_) => DaemonResponse::ServerConnected,
                    Err(e) => DaemonResponse::Error(e.to_string()),
                }
            }
            DaemonRequest::ListTools { server_name } => {
                match self.manager.list_all_tools().await {
                    Ok(tools) => {
                        if let Some(server_tools) = tools.get(&server_name) {
                            let mut result = HashMap::new();
                            result.insert(server_name, server_tools.clone());
                            DaemonResponse::Tools(result)
                        } else {
                            DaemonResponse::Tools(HashMap::new())
                        }
                    }
                    Err(e) => DaemonResponse::Error(e.to_string()),
                }
            }
            DaemonRequest::CallTool { server_name, tool_name, arguments } => {
                match self.manager.call_tool(&server_name, &tool_name, arguments).await {
                    Ok(result) => DaemonResponse::ToolResult(result),
                    Err(e) => DaemonResponse::Error(e.to_string()),
                }
            }
            DaemonRequest::CloseServer { server_name } => {
                // Remove the server from the manager
                if let Some(client) = self.manager.clients.remove(&server_name) {
                    let _ = client.cancel().await;
                    crate::debug_log!("Daemon closed connection to MCP server '{}'", server_name);
                    DaemonResponse::ServerClosed
                } else {
                    DaemonResponse::Error(format!("Server '{}' not found", server_name))
                }
            }
            DaemonRequest::ListConnectedServers => {
                let servers: Vec<String> = self.manager.clients.keys().cloned().collect();
                DaemonResponse::ConnectedServers(servers)
            }
            DaemonRequest::Shutdown => {
                crate::debug_log!("Daemon shutdown requested");
                std::process::exit(0);
            }
        }
    }

    async fn ensure_server_connected(&mut self, server_name: &str) -> Result<()> {
        // Check if server is already connected
        if self.manager.clients.contains_key(server_name) {
            crate::debug_log!("DAEMON: MCP server '{}' already connected. Total connections: {}", server_name, self.manager.clients.len());
            return Ok(());
        }

        // Load configuration and connect to server
        let config = McpConfig::load()?;
        if let Some(server_config) = config.get_server(server_name) {
            let sdk_config = match server_config.server_type {
                McpServerType::Stdio => {
                    let parts: Vec<String> = server_config.command_or_url.split_whitespace()
                        .map(|s| s.to_string())
                        .collect();
                    create_stdio_server_config(server_name.to_string(), parts, None, None)
                }
                McpServerType::Sse => {
                    create_sse_server_config(server_name.to_string(), server_config.command_or_url.clone())
                }
                McpServerType::Streamable => {
                    // For now, treat as SSE (closest equivalent)
                    create_sse_server_config(server_name.to_string(), server_config.command_or_url.clone())
                }
            };

            crate::debug_log!("DAEMON: Connecting to MCP server '{}' (not already connected)", server_name);
            self.manager.add_server(sdk_config).await?;
            crate::debug_log!("DAEMON: Successfully connected to MCP server '{}'. Total connections: {}", server_name, self.manager.clients.len());
            Ok(())
        } else {
            Err(anyhow!("MCP server '{}' not found in configuration", server_name))
        }
    }
}

// Client functions for CLI to communicate with daemon
pub struct DaemonClient {
    socket_path: PathBuf,
}

impl DaemonClient {
    pub fn new() -> Result<Self> {
        Ok(Self {
            socket_path: McpDaemon::get_socket_path()?,
        })
    }

    pub async fn is_daemon_running(&self) -> bool {
        self.socket_path.exists() && self.send_request(DaemonRequest::ListConnectedServers).await.is_ok()
    }

    pub async fn start_daemon_if_needed(&self) -> Result<()> {
        if !self.is_daemon_running().await {
            crate::debug_log!("Starting MCP daemon...");
            
            // Start daemon in background
            let daemon_binary = std::env::current_exe()?;
            tokio::process::Command::new(daemon_binary)
                .arg("--mcp-daemon")
                .spawn()?;

            // Wait a bit for daemon to start
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            
            // Verify daemon started
            let mut retries = 10;
            while retries > 0 && !self.is_daemon_running().await {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                retries -= 1;
            }
            
            if !self.is_daemon_running().await {
                return Err(anyhow!("Failed to start MCP daemon"));
            }
            
            crate::debug_log!("MCP daemon started successfully");
        }
        Ok(())
    }

    pub async fn send_request(&self, request: DaemonRequest) -> Result<DaemonResponse> {
        let mut stream = UnixStream::connect(&self.socket_path).await?;
        
        let request_data = serde_json::to_vec(&request)?;
        stream.write_all(&request_data).await?;
        stream.flush().await?;

        // Read response length first
        let mut len_buffer = [0u8; 4];
        stream.read_exact(&mut len_buffer).await?;
        let response_len = u32::from_le_bytes(len_buffer) as usize;
        
        // Read the actual response data
        let mut response_buffer = vec![0; response_len];
        stream.read_exact(&mut response_buffer).await?;

        let response: DaemonResponse = serde_json::from_slice(&response_buffer)?;
        Ok(response)
    }

    pub async fn ensure_server_connected(&self, server_name: &str) -> Result<()> {
        self.start_daemon_if_needed().await?;
        
        match self.send_request(DaemonRequest::EnsureServerConnected { 
            server_name: server_name.to_string() 
        }).await? {
            DaemonResponse::ServerConnected => Ok(()),
            DaemonResponse::Error(e) => Err(anyhow!(e)),
            _ => Err(anyhow!("Unexpected response from daemon")),
        }
    }

    pub async fn call_tool(&self, server_name: &str, tool_name: &str, arguments: serde_json::Value) -> Result<serde_json::Value> {
        match self.send_request(DaemonRequest::CallTool {
            server_name: server_name.to_string(),
            tool_name: tool_name.to_string(),
            arguments,
        }).await? {
            DaemonResponse::ToolResult(result) => Ok(result),
            DaemonResponse::Error(e) => Err(anyhow!(e)),
            _ => Err(anyhow!("Unexpected response from daemon")),
        }
    }

    pub async fn list_tools(&self, server_name: &str) -> Result<HashMap<String, Vec<rmcp::model::Tool>>> {
        match self.send_request(DaemonRequest::ListTools {
            server_name: server_name.to_string(),
        }).await? {
            DaemonResponse::Tools(tools) => Ok(tools),
            DaemonResponse::Error(e) => Err(anyhow!(e)),
            _ => Err(anyhow!("Unexpected response from daemon")),
        }
    }

    pub async fn close_server(&self, server_name: &str) -> Result<()> {
        match self.send_request(DaemonRequest::CloseServer {
            server_name: server_name.to_string(),
        }).await? {
            DaemonResponse::ServerClosed => Ok(()),
            DaemonResponse::Error(e) => Err(anyhow!(e)),
            _ => Err(anyhow!("Unexpected response from daemon")),
        }
    }
}