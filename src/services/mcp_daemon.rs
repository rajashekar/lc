//! MCP Daemon Service for persistent MCP connections across CLI invocations
//!
//! This module provides a background daemon that maintains persistent MCP server
//! connections, allowing browser sessions and other stateful resources to persist
//! across multiple CLI command invocations.

#[cfg(all(unix, feature = "unix-sockets"))]
use crate::mcp::{
    create_sse_server_config, create_stdio_server_config, McpConfig, McpServerType, SdkMcpManager,
};
#[cfg(all(unix, feature = "unix-sockets"))]
use anyhow::anyhow;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[cfg(all(unix, feature = "unix-sockets"))]
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
#[cfg(all(unix, feature = "unix-sockets"))]
use tokio::io::{AsyncReadExt, AsyncWriteExt};
#[cfg(all(unix, feature = "unix-sockets"))]
use tokio::net::{UnixListener, UnixStream};

#[derive(Debug, Serialize, Deserialize)]
pub enum DaemonRequest {
    ListTools {
        server_name: String,
    },
    CallTool {
        server_name: String,
        tool_name: String,
        arguments: serde_json::Value,
    },
    EnsureServerConnected {
        server_name: String,
    },
    CloseServer {
        server_name: String,
    },
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

#[cfg(all(unix, feature = "unix-sockets"))]
pub struct McpDaemon {
    manager: SdkMcpManager,
    socket_path: PathBuf,
}

// Unix stub when unix-sockets feature is disabled
#[cfg(all(unix, not(feature = "unix-sockets")))]
pub struct McpDaemon {
    _phantom: std::marker::PhantomData<()>,
}

#[cfg(windows)]
pub struct McpDaemon {
    _phantom: std::marker::PhantomData<()>,
}

#[cfg(all(unix, not(feature = "unix-sockets")))]
impl McpDaemon {
    /// Creates a new MCP daemon instance.
    ///
    /// **Note**: MCP daemon functionality requires the "unix-sockets" feature to be enabled.
    pub fn new() -> Result<Self> {
        Err(anyhow::anyhow!(
            "MCP daemon functionality requires the 'unix-sockets' feature to be enabled. \
             Enable it in Cargo.toml or use direct MCP connections without the daemon."
        ))
    }

    /// Returns the socket path for the daemon.
    ///
    /// **Note**: This returns an error when the unix-sockets feature is disabled.
    pub fn get_socket_path() -> Result<PathBuf> {
        Err(anyhow::anyhow!(
            "Unix socket functionality requires the 'unix-sockets' feature to be enabled"
        ))
    }

    /// Starts the daemon service.
    ///
    /// **Note**: This returns an error when the unix-sockets feature is disabled.
    pub async fn start(&mut self) -> Result<()> {
        Err(anyhow::anyhow!(
            "MCP daemon requires the 'unix-sockets' feature to be enabled"
        ))
    }
}

#[cfg(windows)]
impl McpDaemon {
    /// Creates a new MCP daemon instance.
    ///
    /// **Note**: MCP daemon functionality is not supported on Windows.
    /// This returns an error indicating unsupported operation.
    #[allow(dead_code)]
    pub fn new() -> Result<Self> {
        Err(anyhow::anyhow!(
            "MCP daemon functionality is not supported on Windows. \
             The daemon requires Unix domain sockets which are not available on Windows. \
             Consider using direct MCP connections without the daemon."
        ))
    }

    /// Returns the socket path for the daemon.
    ///
    /// **Note**: This always returns an error on Windows as Unix sockets are not supported.
    #[allow(dead_code)]
    pub fn get_socket_path() -> Result<PathBuf> {
        Err(anyhow::anyhow!(
            "Unix socket paths are not supported on Windows"
        ))
    }

    /// Starts the daemon service.
    ///
    /// **Note**: This always returns an error on Windows.
    #[allow(dead_code)]
    pub async fn start(&mut self) -> Result<()> {
        Err(anyhow::anyhow!(
            "MCP daemon cannot be started on Windows due to lack of Unix socket support"
        ))
    }
}

#[cfg(all(unix, feature = "unix-sockets"))]
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
            tokio::fs::remove_file(&self.socket_path).await?;
        }

        let listener = UnixListener::bind(&self.socket_path)?;

        // Set permissions to 0o600 (read/write for owner only) to prevent unauthorized access
        // to the MCP daemon socket which provides access to tools
        let mut perms = std::fs::metadata(&self.socket_path)?.permissions();
        perms.set_mode(0o600);
        std::fs::set_permissions(&self.socket_path, perms)?;

        crate::debug_log!("MCP Daemon started, listening on {:?}", self.socket_path);

        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    // Handle each client sequentially to maintain shared state
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
        // Read request with timeout and larger buffer
        let mut buffer = vec![0; 32768];

        // Add timeout for read operation
        let n = tokio::time::timeout(std::time::Duration::from_secs(30), stream.read(&mut buffer))
            .await??;

        if n == 0 {
            return Ok(());
        }

        // Deserialize in a separate task to avoid blocking
        let request_data = buffer[..n].to_vec();
        let request: DaemonRequest =
            tokio::task::spawn_blocking(move || serde_json::from_slice(&request_data)).await??;

        crate::debug_log!("Daemon received request: {:?}", request);

        let response = self.process_request(request).await;

        // Serialize response in a separate task to avoid blocking
        let response_data =
            tokio::task::spawn_blocking(move || serde_json::to_vec(&response)).await??;

        // Write response with timeout
        let response_len = response_data.len() as u32;
        tokio::time::timeout(std::time::Duration::from_secs(30), async {
            stream.write_all(&response_len.to_le_bytes()).await?;
            stream.write_all(&response_data).await?;
            stream.flush().await
        })
        .await??;

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
                // First ensure the server is connected
                if let Err(e) = self.ensure_server_connected(&server_name).await {
                    return DaemonResponse::Error(format!(
                        "Failed to connect to server '{}': {}",
                        server_name, e
                    ));
                }

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
            DaemonRequest::CallTool {
                server_name,
                tool_name,
                arguments,
            } => {
                match self
                    .manager
                    .call_tool(&server_name, &tool_name, arguments)
                    .await
                {
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
            crate::debug_log!(
                "DAEMON: MCP server '{}' already connected. Total connections: {}",
                server_name,
                self.manager.clients.len()
            );
            return Ok(());
        }

        crate::debug_log!(
            "DAEMON: Loading MCP configuration for server '{}'",
            server_name
        );

        // Load configuration and connect to server
        let config = McpConfig::load().await?;
        if let Some(server_config) = config.get_server(server_name) {
            crate::debug_log!(
                "DAEMON: Found server config for '{}': {:?} ({})",
                server_name,
                server_config.server_type,
                server_config.command_or_url
            );

            let sdk_config = match server_config.server_type {
                McpServerType::Stdio => {
                    let parts: Vec<String> = server_config
                        .command_or_url
                        .split_whitespace()
                        .map(|s| s.to_string())
                        .collect();
                    crate::debug_log!(
                        "DAEMON: Creating STDIO config with command parts: {:?}",
                        parts
                    );
                    let env = if server_config.env.is_empty() {
                        crate::debug_log!("DAEMON: No environment variables to add");
                        None
                    } else {
                        crate::debug_log!(
                            "DAEMON: Adding {} environment variables",
                            server_config.env.len()
                        );
                        for (key, value) in &server_config.env {
                            crate::debug_log!("DAEMON: Env var: {}={}", key, value);
                        }
                        Some(server_config.env.clone())
                    };
                    create_stdio_server_config(server_name.to_string(), parts, env, None)
                }
                McpServerType::Sse => {
                    crate::debug_log!(
                        "DAEMON: Creating SSE config with URL: {}",
                        server_config.command_or_url
                    );
                    create_sse_server_config(
                        server_name.to_string(),
                        server_config.command_or_url.clone(),
                    )
                }
                McpServerType::Streamable => {
                    crate::debug_log!(
                        "DAEMON: Creating Streamable config (treating as SSE) with URL: {}",
                        server_config.command_or_url
                    );
                    // For now, treat as SSE (closest equivalent)
                    create_sse_server_config(
                        server_name.to_string(),
                        server_config.command_or_url.clone(),
                    )
                }
            };

            crate::debug_log!(
                "DAEMON: Attempting to connect to MCP server '{}'",
                server_name
            );
            match self.manager.add_server(sdk_config).await {
                Ok(_) => {
                    crate::debug_log!(
                        "DAEMON: Successfully connected to MCP server '{}'. Total connections: {}",
                        server_name,
                        self.manager.clients.len()
                    );
                    Ok(())
                }
                Err(e) => {
                    crate::debug_log!(
                        "DAEMON: Failed to connect to MCP server '{}': {}",
                        server_name,
                        e
                    );
                    Err(e)
                }
            }
        } else {
            crate::debug_log!(
                "DAEMON: Server '{}' not found in configuration",
                server_name
            );
            Err(anyhow!(
                "MCP server '{}' not found in configuration",
                server_name
            ))
        }
    }
}

// Client functions for CLI to communicate with daemon
#[cfg(all(unix, feature = "unix-sockets"))]
pub struct DaemonClient {
    socket_path: PathBuf,
}

// Unix stub when unix-sockets feature is disabled
#[cfg(all(unix, not(feature = "unix-sockets")))]
pub struct DaemonClient {
    _phantom: std::marker::PhantomData<()>,
}

#[cfg(windows)]
pub struct DaemonClient {
    _phantom: std::marker::PhantomData<()>,
}

#[cfg(all(unix, not(feature = "unix-sockets")))]
impl DaemonClient {
    /// Creates a new daemon client.
    ///
    /// **Note**: MCP daemon functionality requires the "unix-sockets" feature to be enabled.
    pub fn new() -> Result<Self> {
        Err(anyhow::anyhow!(
            "MCP daemon client requires the 'unix-sockets' feature to be enabled"
        ))
    }

    /// Checks if the daemon is running.
    ///
    /// **Note**: Always returns false when unix-sockets feature is disabled.
    pub async fn is_daemon_running(&self) -> bool {
        false
    }

    /// Attempts to start the daemon if needed.
    ///
    /// **Note**: Always returns an error when unix-sockets feature is disabled.
    pub async fn start_daemon_if_needed(&self) -> Result<()> {
        Err(anyhow::anyhow!(
            "Cannot start MCP daemon - 'unix-sockets' feature is required"
        ))
    }

    /// Sends a request to the daemon.
    ///
    /// **Note**: Always returns an error when unix-sockets feature is disabled.
    pub async fn send_request(&self, _request: DaemonRequest) -> Result<DaemonResponse> {
        Err(anyhow::anyhow!(
            "Cannot communicate with MCP daemon - 'unix-sockets' feature is required"
        ))
    }

    /// Ensures a server is connected via the daemon.
    ///
    /// **Note**: Always returns an error when unix-sockets feature is disabled.
    pub async fn ensure_server_connected(&self, _server_name: &str) -> Result<()> {
        Err(anyhow::anyhow!(
            "MCP daemon server connections require the 'unix-sockets' feature"
        ))
    }

    /// Calls a tool via the daemon.
    ///
    /// **Note**: Always returns an error when unix-sockets feature is disabled.
    pub async fn call_tool(
        &self,
        _server_name: &str,
        _tool_name: &str,
        _arguments: serde_json::Value,
    ) -> Result<serde_json::Value> {
        Err(anyhow::anyhow!(
            "MCP daemon tool calls require the 'unix-sockets' feature"
        ))
    }

    /// Lists tools via the daemon.
    ///
    /// **Note**: Always returns an error when unix-sockets feature is disabled.
    pub async fn list_tools(
        &self,
        _server_name: &str,
    ) -> Result<HashMap<String, Vec<rmcp::model::Tool>>> {
        Err(anyhow::anyhow!(
            "MCP daemon tool listing requires the 'unix-sockets' feature"
        ))
    }

    /// Closes a server connection via the daemon.
    ///
    /// **Note**: Always returns an error when unix-sockets feature is disabled.
    pub async fn close_server(&self, _server_name: &str) -> Result<()> {
        Err(anyhow::anyhow!(
            "MCP daemon server closing requires the 'unix-sockets' feature"
        ))
    }
}

#[cfg(windows)]
impl DaemonClient {
    /// Creates a new daemon client.
    ///
    /// **Note**: MCP daemon functionality is not supported on Windows.
    pub fn new() -> Result<Self> {
        Err(anyhow::anyhow!(
            "MCP daemon client is not supported on Windows"
        ))
    }

    /// Checks if the daemon is running.
    ///
    /// **Note**: Always returns false on Windows.
    #[allow(dead_code)]
    pub async fn is_daemon_running(&self) -> bool {
        false
    }

    /// Attempts to start the daemon if needed.
    ///
    /// **Note**: Always returns an error on Windows.
    #[allow(dead_code)]
    pub async fn start_daemon_if_needed(&self) -> Result<()> {
        Err(anyhow::anyhow!(
            "Cannot start MCP daemon on Windows - Unix sockets not supported"
        ))
    }

    /// Sends a request to the daemon.
    ///
    /// **Note**: Always returns an error on Windows.
    #[allow(dead_code)]
    pub async fn send_request(&self, _request: DaemonRequest) -> Result<DaemonResponse> {
        Err(anyhow::anyhow!(
            "Cannot communicate with MCP daemon on Windows"
        ))
    }

    /// Ensures a server is connected via the daemon.
    ///
    /// **Note**: Always returns an error on Windows.
    pub async fn ensure_server_connected(&self, _server_name: &str) -> Result<()> {
        Err(anyhow::anyhow!(
            "MCP daemon server connections not supported on Windows"
        ))
    }

    /// Calls a tool via the daemon.
    ///
    /// **Note**: Always returns an error on Windows.
    pub async fn call_tool(
        &self,
        _server_name: &str,
        _tool_name: &str,
        _arguments: serde_json::Value,
    ) -> Result<serde_json::Value> {
        Err(anyhow::anyhow!(
            "MCP daemon tool calls not supported on Windows"
        ))
    }

    /// Lists tools via the daemon.
    ///
    /// **Note**: Always returns an error on Windows.
    pub async fn list_tools(
        &self,
        _server_name: &str,
    ) -> Result<HashMap<String, Vec<rmcp::model::Tool>>> {
        Err(anyhow::anyhow!(
            "MCP daemon tool listing not supported on Windows"
        ))
    }

    /// Closes a server connection via the daemon.
    ///
    /// **Note**: Always returns an error on Windows.
    pub async fn close_server(&self, _server_name: &str) -> Result<()> {
        Err(anyhow::anyhow!(
            "MCP daemon server closing not supported on Windows"
        ))
    }
}

#[cfg(all(unix, feature = "unix-sockets"))]
impl DaemonClient {
    pub fn new() -> Result<Self> {
        Ok(Self {
            socket_path: McpDaemon::get_socket_path()?,
        })
    }

    pub async fn is_daemon_running(&self) -> bool {
        self.socket_path.exists()
            && self
                .send_request(DaemonRequest::ListConnectedServers)
                .await
                .is_ok()
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

        match self
            .send_request(DaemonRequest::EnsureServerConnected {
                server_name: server_name.to_string(),
            })
            .await?
        {
            DaemonResponse::ServerConnected => Ok(()),
            DaemonResponse::Error(e) => Err(anyhow!(e)),
            _ => Err(anyhow!("Unexpected response from daemon")),
        }
    }

    pub async fn call_tool(
        &self,
        server_name: &str,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value> {
        match self
            .send_request(DaemonRequest::CallTool {
                server_name: server_name.to_string(),
                tool_name: tool_name.to_string(),
                arguments,
            })
            .await?
        {
            DaemonResponse::ToolResult(result) => Ok(result),
            DaemonResponse::Error(e) => Err(anyhow!(e)),
            _ => Err(anyhow!("Unexpected response from daemon")),
        }
    }

    pub async fn list_tools(
        &self,
        server_name: &str,
    ) -> Result<HashMap<String, Vec<rmcp::model::Tool>>> {
        crate::debug_log!(
            "DaemonClient: Requesting tools for server '{}'",
            server_name
        );
        match self
            .send_request(DaemonRequest::ListTools {
                server_name: server_name.to_string(),
            })
            .await?
        {
            DaemonResponse::Tools(tools) => {
                crate::debug_log!(
                    "DaemonClient: Received tools response with {} servers",
                    tools.len()
                );
                for (name, server_tools) in &tools {
                    crate::debug_log!(
                        "DaemonClient: Server '{}' has {} tools",
                        name,
                        server_tools.len()
                    );
                }
                Ok(tools)
            }
            DaemonResponse::Error(e) => {
                crate::debug_log!("DaemonClient: Received error response: {}", e);
                Err(anyhow!(e))
            }
            response => {
                crate::debug_log!("DaemonClient: Received unexpected response: {:?}", response);
                Err(anyhow!("Unexpected response from daemon"))
            }
        }
    }

    pub async fn close_server(&self, server_name: &str) -> Result<()> {
        match self
            .send_request(DaemonRequest::CloseServer {
                server_name: server_name.to_string(),
            })
            .await?
        {
            DaemonResponse::ServerClosed => Ok(()),
            DaemonResponse::Error(e) => Err(anyhow!(e)),
            _ => Err(anyhow!("Unexpected response from daemon")),
        }
    }
}
