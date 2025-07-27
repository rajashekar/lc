use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
#[cfg(unix)]
use std::os::unix::fs::{FileTypeExt};
#[cfg(unix)]
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use serde_json::{json, Value};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct McpConfig {
    pub servers: HashMap<String, McpServerConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct McpServerConfig {
    pub name: String,
    pub server_type: McpServerType,
    pub command_or_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum McpServerType {
    Stdio,
    Sse,
    Streamable,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpFunction {
    pub name: String,
    pub description: String,
    pub parameters: Option<Value>,
}

#[derive(Debug, Clone)]
pub struct RunningServerInfo {
    pub server_type: String,
    pub pid: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessRegistryEntry {
    pub name: String,
    pub pid: u32,
    pub server_type: String,
    pub command: String,
    pub session_id: Option<String>, // For SSE servers
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessRegistry {
    pub servers: HashMap<String, ProcessRegistryEntry>,
}

pub struct McpManager {
    // Simplified manager that uses the process registry for tracking servers
}

impl McpConfig {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_file_path()?;
        
        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            let config: McpConfig = serde_json::from_str(&content)?;
            Ok(config)
        } else {
            // Create default config
            let config = McpConfig {
                servers: HashMap::new(),
            };
            
            // Ensure config directory exists
            if let Some(parent) = config_path.parent() {
                fs::create_dir_all(parent)?;
            }
            
            config.save()?;
            Ok(config)
        }
    }
    
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_file_path()?;
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&config_path, content)?;
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
        if self.servers.remove(name).is_some() {
            Ok(())
        } else {
            anyhow::bail!("MCP server '{}' not found", name);
        }
    }
    
    pub fn get_server(&self, name: &str) -> Option<&McpServerConfig> {
        self.servers.get(name)
    }
    
    pub fn list_servers(&self) -> &HashMap<String, McpServerConfig> {
        &self.servers
    }
    
    fn config_file_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
        
        let lc_dir = config_dir.join("lc");
        fs::create_dir_all(&lc_dir)?;
        Ok(lc_dir.join("mcp.json"))
    }
}

impl ProcessRegistry {
    pub fn load() -> Result<Self> {
        let registry_path = Self::registry_file_path()?;
        
        if registry_path.exists() {
            let content = fs::read_to_string(&registry_path)?;
            let registry: ProcessRegistry = serde_json::from_str(&content)?;
            Ok(registry)
        } else {
            let registry = ProcessRegistry {
                servers: HashMap::new(),
            };
            
            // Ensure config directory exists
            if let Some(parent) = registry_path.parent() {
                fs::create_dir_all(parent)?;
            }
            
            registry.save()?;
            Ok(registry)
        }
    }
    
    pub fn save(&self) -> Result<()> {
        let registry_path = Self::registry_file_path()?;
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&registry_path, content)?;
        Ok(())
    }
    
    pub fn add_server(&mut self, name: String, pid: u32, server_type: String, command: String) -> Result<()> {
        self.add_server_with_session(name, pid, server_type, command, None)
    }
    
    pub fn add_server_with_session(&mut self, name: String, pid: u32, server_type: String, command: String, session_id: Option<String>) -> Result<()> {
        let entry = ProcessRegistryEntry {
            name: name.clone(),
            pid,
            server_type,
            command,
            session_id,
        };
        
        self.servers.insert(name, entry);
        Ok(())
    }
    
    pub fn remove_server(&mut self, name: &str) -> Result<()> {
        self.servers.remove(name);
        Ok(())
    }
    
    pub fn get_running_servers(&self) -> HashMap<String, RunningServerInfo> {
        let mut result = HashMap::new();
        
        for (name, entry) in &self.servers {
            // For Streamable servers (PID 0), consider them running if they have a session ID
            // For other servers, check if process is still running
            let is_running = if entry.pid == 0 {
                // Streamable server - check if it has a session ID
                entry.session_id.is_some()
            } else {
                // Regular process - check if it's still running
                Self::is_process_running(entry.pid)
            };
            
            if is_running {
                result.insert(name.clone(), RunningServerInfo {
                    server_type: entry.server_type.clone(),
                    pid: entry.pid,
                });
            }
        }
        
        result
    }
    
    pub fn cleanup_dead_processes(&mut self) -> Result<()> {
        let mut to_remove = Vec::new();
        
        for (name, entry) in &self.servers {
            // For Streamable servers (PID 0), don't remove them during cleanup
            // They should only be removed explicitly via stop_server
            if entry.pid != 0 && !Self::is_process_running(entry.pid) {
                to_remove.push(name.clone());
            }
        }
        
        for name in to_remove {
            self.servers.remove(&name);
        }
        
        Ok(())
    }
    
    fn is_process_running(pid: u32) -> bool {
        // On Unix systems, we can check if a process is running by sending signal 0
        #[cfg(unix)]
        {
            let output = std::process::Command::new("kill")
                .args(&["-0", &pid.to_string()])
                .output();
            
            match output {
                Ok(output) => output.status.success(),
                Err(_) => false,
            }
        }
        
        #[cfg(windows)]
        {
            let output = std::process::Command::new("tasklist")
                .args(&["/FI", &format!("PID eq {}", pid)])
                .output();
            
            match output {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    stdout.contains(&pid.to_string())
                }
                Err(_) => false,
            }
        }
    }
    
    fn registry_file_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
        
        let lc_dir = config_dir.join("lc");
        fs::create_dir_all(&lc_dir)?;
        Ok(lc_dir.join("mcp_processes.json"))
    }
}

impl McpManager {
    pub fn new() -> Self {
        Self {}
    }
    
    fn get_log_file_path(server_name: &str) -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
        
        let lc_dir = config_dir.join("lc");
        fs::create_dir_all(&lc_dir)?;
        Ok(lc_dir.join(format!("{}.log", server_name)))
    }
    
    fn get_stdin_pipe_path(server_name: &str) -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
        
        let lc_dir = config_dir.join("lc");
        fs::create_dir_all(&lc_dir)?;
        Ok(lc_dir.join(format!("{}.stdin.pipe", server_name)))
    }

    fn get_stdout_pipe_path(server_name: &str) -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;

        let lc_dir = config_dir.join("lc");
        fs::create_dir_all(&lc_dir)?;
        Ok(lc_dir.join(format!("{}.stdout.pipe", server_name)))
    }
    
    pub async fn start_server(&mut self, name: &str, server_config: &McpServerConfig, debug: bool) -> Result<()> {
        // Check if server is already running in registry
        let mut registry = ProcessRegistry::load()?;
        registry.cleanup_dead_processes()?;
        
        let running_servers = registry.get_running_servers();
        if running_servers.contains_key(name) {
            anyhow::bail!("MCP server '{}' is already running", name);
        }
        
        match server_config.server_type {
            McpServerType::Stdio => {
                self.start_stdio_server(name, server_config, &mut registry, debug).await
            }
            McpServerType::Sse => {
                self.start_sse_server(name, server_config, &mut registry, debug).await
            }
            McpServerType::Streamable => {
                self.start_streamable_server(name, server_config, &mut registry, debug).await
            }
        }
    }
    
    async fn start_stdio_server(
        &mut self,
        name: &str,
        server_config: &McpServerConfig,
        registry: &mut ProcessRegistry,
        debug: bool,
    ) -> Result<()> {
        let log_path = Self::get_log_file_path(name)?;
        let stdin_pipe_path = Self::get_stdin_pipe_path(name)?;
        let stdout_pipe_path = Self::get_stdout_pipe_path(name)?;

        // Create named pipes for stdin and stdout
        for pipe_path in [&stdin_pipe_path, &stdout_pipe_path] {
            if pipe_path.exists() {
                #[cfg(unix)]
                {
                    let metadata = fs::metadata(&pipe_path)?;
                    if !metadata.file_type().is_fifo() {
                        fs::remove_file(&pipe_path)?;
                        nix::unistd::mkfifo(pipe_path, nix::sys::stat::Mode::S_IRWXU)?;
                    }
                }
            } else {
                nix::unistd::mkfifo(pipe_path, nix::sys::stat::Mode::S_IRWXU)?;
            }
        }

        // The server reads from stdin_pipe and writes to stdout_pipe.
        // We also tee the output to a log file for debugging.
        let command = format!(
            "nohup sh -c 'while true; do cat \"{}\"; done | {} 2> \"{}\" > \"{}\"' > /dev/null 2>&1 & echo $!",
            stdin_pipe_path.display(),
            server_config.command_or_url,
            log_path.display(),
            stdout_pipe_path.display()
        );

        if debug {
            println!("Debug: Executing command: {}", command);
        }
        
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(&command)
            .output()?;
        
        if debug {
            println!("Debug: Command exit status: {}", output.status);
            println!("Debug: Command stdout: '{}'", String::from_utf8_lossy(&output.stdout));
            println!("Debug: Command stderr: '{}'", String::from_utf8_lossy(&output.stderr));
        }
        
        if !output.status.success() {
            anyhow::bail!("Failed to start MCP server: {}", String::from_utf8_lossy(&output.stderr));
        }
        
        let stdout_str = String::from_utf8_lossy(&output.stdout);
        // The output might contain multiple lines, we want the last line which should be the PID
        let pid_str = stdout_str.lines().last().unwrap_or("").trim().to_string();
        
        if debug {
            println!("Debug: Parsed PID string: '{}'", pid_str);
        }
        
        let pid: u32 = pid_str.parse()
            .map_err(|e| anyhow::anyhow!("Failed to parse process ID '{}': {}", pid_str, e))?;
        
        if debug {
            println!("Debug: Parsed PID: {}", pid);
        }
        
        // Register the process
        registry.add_server(
            name.to_string(),
            pid,
            format!("{:?}", server_config.server_type),
            server_config.command_or_url.clone(),
        )?;
        registry.save()?;
        
        if debug {
            println!("Debug: Process registered in registry");
        }
        
        // Give the process a moment to start up
        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
        
        // Verify the process is actually running
        let is_running = ProcessRegistry::is_process_running(pid);
        if debug {
            println!("Debug: Process {} running status: {}", pid, is_running);
        }
        
        if !is_running {
            // Clean up the registry entry if the process failed to start
            registry.remove_server(name)?;
            registry.save()?;
            
            // Try to read the log file to provide better error information
            let log_content = match fs::read_to_string(&log_path) {
                Ok(content) => {
                    if content.trim().is_empty() {
                        "No output in log file".to_string()
                    } else {
                        // Get last 10 lines of log
                        let lines: Vec<&str> = content.lines().collect();
                        let start = if lines.len() > 10 { lines.len() - 10 } else { 0 };
                        lines[start..].join("\n")
                    }
                }
                Err(_) => "Could not read log file".to_string()
            };
            
            anyhow::bail!("MCP server failed to start or exited immediately. Check log at {}: {}", log_path.display(), log_content);
        }
        
        println!("Logs are being written to: {}", log_path.display());
        
        Ok(())
    }
    
    async fn start_sse_server(
        &mut self,
        name: &str,
        server_config: &McpServerConfig,
        registry: &mut ProcessRegistry,
        debug: bool,
    ) -> Result<()> {
        let url = &server_config.command_or_url;
        let log_path = Self::get_log_file_path(name)?;
        
        if debug {
            println!("Debug: Starting SSE connection to: {}", url);
        }
        
        // Start a persistent curl process to maintain the SSE connection and capture output
        let command = format!(
            "nohup curl -N -H 'Accept: text/event-stream' '{}' > '{}' 2>&1 & echo $!",
            url, log_path.display()
        );
        
        if debug {
            println!("Debug: Executing command: {}", command);
        }
        
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(&command)
            .output()?;
        
        if debug {
            println!("Debug: Command exit status: {}", output.status);
            println!("Debug: Command stdout: '{}'", String::from_utf8_lossy(&output.stdout));
            println!("Debug: Command stderr: '{}'", String::from_utf8_lossy(&output.stderr));
        }
        
        if !output.status.success() {
            anyhow::bail!("Failed to start SSE connection: {}", String::from_utf8_lossy(&output.stderr));
        }
        
        let stdout_str = String::from_utf8_lossy(&output.stdout);
        let pid_str = stdout_str.lines().last().unwrap_or("").trim().to_string();
        
        if debug {
            println!("Debug: Parsed PID string: '{}'", pid_str);
        }
        
        let pid: u32 = pid_str.parse()
            .map_err(|e| anyhow::anyhow!("Failed to parse process ID '{}': {}", pid_str, e))?;
        
        if debug {
            println!("Debug: Parsed PID: {}", pid);
        }
        
        // Give the connection a moment to establish
        tokio::time::sleep(std::time::Duration::from_millis(3000)).await;
        
        // Read the log file to extract session ID
        let log_content = match std::fs::read_to_string(&log_path) {
            Ok(content) => content,
            Err(_) => {
                // Clean up the process if we can't read the log
                let _ = std::process::Command::new("kill").arg(pid.to_string()).output();
                anyhow::bail!("Failed to read SSE connection log at {}", log_path.display());
            }
        };
        
        if debug {
            println!("Debug: SSE log content: {}", log_content);
        }
        
        // Extract session ID from the log content
        let session_id = if let Some(data_line) = log_content.lines().find(|line| line.starts_with("data: ")) {
            let data = data_line.strip_prefix("data: ").unwrap_or("");
            if let Some(session_start) = data.find("sessionId=") {
                let session_part = &data[session_start + 10..];
                if let Some(session_end) = session_part.find(|c: char| !c.is_alphanumeric() && c != '-') {
                    session_part[..session_end].to_string()
                } else {
                    session_part.to_string()
                }
            } else {
                // Clean up the process if no session ID found
                let _ = std::process::Command::new("kill").arg(pid.to_string()).output();
                anyhow::bail!("No session ID found in SSE response");
            }
        } else {
            // Clean up the process if no data line found
            let _ = std::process::Command::new("kill").arg(pid.to_string()).output();
            anyhow::bail!("No data line found in SSE response");
        };
        
        if debug {
            println!("Debug: Extracted session ID: {}", session_id);
        }
        
        // Register the process with session ID
        registry.add_server_with_session(
            name.to_string(),
            pid,
            format!("{:?}", server_config.server_type),
            server_config.command_or_url.clone(),
            Some(session_id.clone()),
        )?;
        registry.save()?;
        
        if debug {
            println!("Debug: SSE server registered in registry with session ID: {}", session_id);
        }
        
        // Verify the process is actually running
        let is_running = ProcessRegistry::is_process_running(pid);
        if debug {
            println!("Debug: Process {} running status: {}", pid, is_running);
        }
        
        if !is_running {
            // Clean up the registry entry if the process failed to start
            registry.remove_server(name)?;
            registry.save()?;
            anyhow::bail!("SSE connection process failed to start or exited immediately");
        }
        
        println!("✓ SSE connection established with session ID: {}", session_id);
        println!("Logs are being written to: {}", log_path.display());
        
        Ok(())
    }
    
    async fn start_streamable_server(
        &mut self,
        name: &str,
        server_config: &McpServerConfig,
        registry: &mut ProcessRegistry,
        debug: bool,
    ) -> Result<()> {
        let url = &server_config.command_or_url;
        
        if debug {
            println!("Debug: Initializing Streamable HTTP connection to: {}", url);
        }
        
        // For Streamable servers, we need to send an initialize request to get a session ID
        let initialize_request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "lc-client",
                    "version": "1.0"
                }
            },
            "id": 0
        });
        
        let client = reqwest::Client::new();
        let response = client
            .post(url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/event-stream")
            .json(&initialize_request)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to initialize Streamable server: {}", e))?;
        
        if !response.status().is_success() {
            anyhow::bail!("Streamable server returned error status: {}", response.status());
        }
        
        // Extract session ID from response headers
        let session_id = response
            .headers()
            .get("mcp-session-id")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| anyhow::anyhow!("No mcp-session-id header found in response"))?
            .to_string();
        
        if debug {
            println!("Debug: Extracted session ID: {}", session_id);
        }
        
        // Read the response body to ensure initialization completed
        let response_text = response
            .text()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to read initialization response: {}", e))?;
        
        if debug {
            println!("Debug: Initialization response: {}", response_text);
        }
        
        // For Streamable servers, we don't have a persistent process, so we use PID 0
        // The session ID is the important part for communication
        registry.add_server_with_session(
            name.to_string(),
            0, // No actual process for Streamable servers
            format!("{:?}", server_config.server_type),
            server_config.command_or_url.clone(),
            Some(session_id.clone()),
        )?;
        registry.save()?;
        
        if debug {
            println!("Debug: Streamable server registered with session ID: {}", session_id);
        }
        
        println!("✓ Streamable HTTP session established with session ID: {}", session_id);
        
        Ok(())
    }
    
    pub async fn stop_server(&mut self, name: &str) -> Result<()> {
        let mut registry = ProcessRegistry::load()?;
        registry.cleanup_dead_processes()?;
        
        if let Some(entry) = registry.servers.get(name) {
            let pid = entry.pid;
            
            // For Streamable servers (PID 0), we don't need to kill a process
            if pid != 0 {
                // Kill the process using system commands
                #[cfg(unix)]
                {
                    // First, try to kill the entire process group to clean up all child processes
                    let pgid_output = std::process::Command::new("ps")
                        .args(&["-o", "pgid=", "-p", &pid.to_string()])
                        .output();
                    
                    let mut killed_group = false;
                    if let Ok(output) = pgid_output {
                        if let Ok(pgid_str) = String::from_utf8(output.stdout) {
                            if let Ok(pgid) = pgid_str.trim().parse::<i32>() {
                                // Kill the entire process group
                                let group_kill_output = std::process::Command::new("kill")
                                    .args(&["-TERM", &format!("-{}", pgid)])
                                    .output();
                                
                                if let Ok(output) = group_kill_output {
                                    if output.status.success() {
                                        killed_group = true;
                                        // Wait a moment for graceful termination
                                        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
                                        
                                        // If processes are still running, force kill the group
                                        let _ = std::process::Command::new("kill")
                                            .args(&["-KILL", &format!("-{}", pgid)])
                                            .output();
                                    }
                                }
                            }
                        }
                    }
                    
                    // Fallback: kill individual process if group kill failed
                    if !killed_group {
                        let output = std::process::Command::new("kill")
                            .args(&["-TERM", &pid.to_string()])
                            .output();
                        
                        match output {
                            Ok(output) => {
                                if !output.status.success() {
                                    // Try SIGKILL if SIGTERM failed
                                    let _ = std::process::Command::new("kill")
                                        .args(&["-KILL", &pid.to_string()])
                                        .output();
                                }
                            }
                            Err(_) => {
                                anyhow::bail!("Failed to kill process {}", pid);
                            }
                        }
                    }
                    
                    // Additional cleanup: find and kill any remaining processes related to this server
                    let cleanup_command = format!(
                        "pkill -f 'Application.Support.*lc.*{}'",
                        name
                    );
                    let _ = std::process::Command::new("sh")
                        .arg("-c")
                        .arg(&cleanup_command)
                        .output();
                }
                
                #[cfg(windows)]
                {
                    let output = std::process::Command::new("taskkill")
                        .args(&["/PID", &pid.to_string(), "/F"])
                        .output();
                    
                    match output {
                        Ok(output) => {
                            if !output.status.success() {
                                anyhow::bail!("Failed to kill process {}", pid);
                            }
                        }
                        Err(_) => {
                            anyhow::bail!("Failed to kill process {}", pid);
                        }
                    }
                }
            }
            
            // Remove from registry
            registry.remove_server(name)?;
            registry.save()?;

            // Clean up named pipes
            for pipe_path in [Self::get_stdin_pipe_path(name)?, Self::get_stdout_pipe_path(name)?] {
                if pipe_path.exists() {
                    let _ = fs::remove_file(pipe_path);
                }
            }
            
            Ok(())
        } else {
            anyhow::bail!("MCP server '{}' is not running", name);
        }
    }
    
    pub async fn list_running_servers(&self) -> Result<HashMap<String, RunningServerInfo>> {
        let mut registry = ProcessRegistry::load()?;
        registry.cleanup_dead_processes()?;
        registry.save()?; // Save after cleanup
        
        Ok(registry.get_running_servers())
    }
    
    pub async fn list_functions(&mut self, name: &str) -> Result<Vec<McpFunction>> {
        let request = json!({
            "jsonrpc": "2.0",
            "method": "tools/list",
            "id": "1"
        });
        
        let response = self.send_request(name, &request).await?;
        self.parse_tools_response(&response)
    }
    
    pub async fn invoke_function(&mut self, name: &str, function_name: &str, args: &[String]) -> Result<Value> {
        // Parse args as key=value pairs
        let params = if args.is_empty() {
            json!({})
        } else {
            let mut params_obj = serde_json::Map::new();
            for arg in args {
                if let Some((key, value)) = arg.split_once('=') {
                    params_obj.insert(key.to_string(), json!(value));
                } else {
                    anyhow::bail!("Invalid argument format: '{}'. Expected 'key=value'", arg);
                }
            }
            json!(params_obj)
        };
        
        let request = json!({
            "jsonrpc": "2.0",
            "method": "tools/call",
            "params": {
                "name": function_name,
                "arguments": params
            },
            "id": "1"
        });
        
        let response = self.send_request(name, &request).await?;
        let parsed: Value = serde_json::from_str(&response)?;
        
        if let Some(result) = parsed.get("result") {
            Ok(result.clone())
        } else if let Some(error) = parsed.get("error") {
            anyhow::bail!("MCP server error: {}", error);
        } else {
            anyhow::bail!("Invalid response from MCP server");
        }
    }
    
    async fn send_request(&mut self, name: &str, request: &Value) -> Result<String> {
        // Get server config to know how to connect
        let config = McpConfig::load()?;
        let server_config = config.get_server(name)
            .ok_or_else(|| anyhow::anyhow!("MCP server '{}' not found in configuration", name))?;
        
        match server_config.server_type {
            McpServerType::Stdio => {
                // Check if stdio server is running in registry
                let registry = ProcessRegistry::load()?;
                let running_servers = registry.get_running_servers();
                
                if !running_servers.contains_key(name) {
                    anyhow::bail!("MCP server '{}' is not running", name);
                }
                
                self.send_request_to_stdio_server(name, server_config, request).await
            }
            McpServerType::Sse => {
                // SSE servers don't need to be "running" - they're accessed directly via HTTP
                self.send_request_to_sse_server(name, server_config, request).await
            }
            McpServerType::Streamable => {
                // Streamable servers use session-based HTTP communication
                self.send_request_to_streamable_server(name, server_config, request).await
            }
        }
    }
    
    async fn send_request_to_stdio_server(
        &mut self,
        name: &str,
        _server_config: &McpServerConfig,
        request: &Value,
    ) -> Result<String> {
        // This function will now use named pipes for both stdin and stdout
        // to ensure cross-platform compatibility (especially for macOS).
        #[cfg(unix)]
        {
            use tokio::fs::OpenOptions;

            let stdin_pipe_path = Self::get_stdin_pipe_path(name)?;
            let stdout_pipe_path = Self::get_stdout_pipe_path(name)?;
            let stdin_pipe_path_clone = stdin_pipe_path.clone();

            // Open stdin pipe for writing, with a timeout.
            let open_stdin_task = tokio::task::spawn(async move {
                OpenOptions::new()
                    .write(true)
                    .open(&stdin_pipe_path_clone)
                    .await
            });

            let mut stdin = match tokio::time::timeout(std::time::Duration::from_secs(5), open_stdin_task).await {
                Ok(Ok(Ok(file))) => file,
                Ok(Ok(Err(e))) => return Err(anyhow::anyhow!("Failed to open stdin pipe '{}': {}", stdin_pipe_path.display(), e)),
                Ok(Err(e)) => return Err(anyhow::anyhow!("Failed to join stdin open task: {}", e)),
                Err(_) => return Err(anyhow::anyhow!("Timeout opening stdin pipe '{}'", stdin_pipe_path.display())),
            };

            // Open stdout pipe for reading.
            let stdout_pipe_path_clone = stdout_pipe_path.clone();
            let open_stdout_task = tokio::task::spawn(async move {
                OpenOptions::new()
                    .read(true)
                    .open(&stdout_pipe_path_clone)
                    .await
            });

            let stdout = match tokio::time::timeout(std::time::Duration::from_secs(5), open_stdout_task).await {
                Ok(Ok(Ok(file))) => file,
                Ok(Ok(Err(e))) => return Err(anyhow::anyhow!("Failed to open stdout pipe '{}': {}", stdout_pipe_path.display(), e)),
                Ok(Err(e)) => return Err(anyhow::anyhow!("Failed to join stdout open task: {}", e)),
                Err(_) => return Err(anyhow::anyhow!("Timeout opening stdout pipe '{}'", stdout_pipe_path.display())),
            };

            let mut reader = BufReader::new(stdout);

            // Send request
            let request_str = request.to_string() + "\n";
            stdin.write_all(request_str.as_bytes()).await?;
            stdin.flush().await?;

            // Read response
            let mut response_line = String::new();
            let timeout_duration = std::time::Duration::from_secs(30);

            let result = tokio::time::timeout(
                timeout_duration,
                reader.read_line(&mut response_line)
            ).await;

            match result {
                Ok(Ok(0)) => anyhow::bail!("MCP server closed connection"),
                Ok(Ok(_)) => {
                    // Filter out spinner characters and other non-JSON content
                    if let Some(json_start) = response_line.find('{') {
                        Ok(response_line[json_start..].trim().to_string())
                    } else {
                        if response_line.trim().is_empty() {
                            anyhow::bail!("MCP server returned empty response");
                        }
                        Ok(response_line.trim().to_string())
                    }
                }
                Ok(Err(e)) => anyhow::bail!("Failed to read from MCP server: {}", e),
                Err(_) => anyhow::bail!("Request timed out after 30 seconds"),
            }
        }

        #[cfg(not(unix))]
        {
            anyhow::bail!("Persistent stdio servers are currently only supported on Unix-like systems.")
        }
    }
    
    async fn send_request_to_sse_server(
        &mut self,
        name: &str,
        server_config: &McpServerConfig,
        request: &Value,
    ) -> Result<String> {
        // Get the session ID from the registry
        let registry = ProcessRegistry::load()?;
        let server_entry = registry.servers.get(name)
            .ok_or_else(|| anyhow::anyhow!("SSE server '{}' not found in registry", name))?;
        
        let session_id = server_entry.session_id.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No session ID found for SSE server '{}'", name))?;
        
        let log_path = Self::get_log_file_path(name)?;
        
        // Get the current size of the log file to know where to start reading from
        let initial_size = match std::fs::metadata(&log_path) {
            Ok(metadata) => metadata.len(),
            Err(_) => 0,
        };
        
        // For SSE servers, we send HTTP POST requests to the server URL with session ID
        let client = reqwest::Client::new();
        let url_with_session = format!("{}?sessionId={}", server_config.command_or_url, session_id);
        
        let response = client
            .post(&url_with_session)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/event-stream")
            .json(request)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send request to SSE server: {}", e))?;
        
        if !response.status().is_success() {
            anyhow::bail!("SSE server returned error status: {}", response.status());
        }
        
        // The response should be "Accepted" - the actual response is streamed to the log file
        let response_text = response
            .text()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to read response from SSE server: {}", e))?;
        
        if response_text.trim() != "Accepted" {
            anyhow::bail!("Unexpected response from SSE server: {}", response_text);
        }
        
        // Wait a moment for the response to be written to the log file
        tokio::time::sleep(std::time::Duration::from_millis(2000)).await;
        
        // Read the entire log file and find the most recent valid JSON response
        let log_content = match std::fs::read_to_string(&log_path) {
            Ok(content) => content,
            Err(e) => anyhow::bail!("Failed to read log file: {}", e),
        };
        
        // Use a more robust approach to find JSON responses
        // Look for lines that start with "data: {" (JSON responses)
        let mut latest_response = None;
        
        for line in log_content.lines() {
            if let Some(data_line) = line.strip_prefix("data: ") {
                // Check if this looks like a JSON response (starts with '{')
                if data_line.starts_with('{') {
                    // Try to parse as JSON to validate it's a complete response
                    if let Ok(_) = serde_json::from_str::<Value>(data_line) {
                        latest_response = Some(data_line.to_string());
                    }
                }
            }
        }
        
        match latest_response {
            Some(response) => Ok(response),
            None => anyhow::bail!("No valid JSON response found in SSE log"),
        }
    }
    
    async fn send_request_to_streamable_server(
        &mut self,
        name: &str,
        server_config: &McpServerConfig,
        request: &Value,
    ) -> Result<String> {
        // Get the session ID from the registry
        let registry = ProcessRegistry::load()?;
        let server_entry = registry.servers.get(name)
            .ok_or_else(|| anyhow::anyhow!("Streamable server '{}' not found in registry", name))?;
        
        let session_id = server_entry.session_id.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No session ID found for Streamable server '{}'", name))?;
        
        // For Streamable servers, we send HTTP POST requests with session ID in header
        let client = reqwest::Client::new();
        let url = &server_config.command_or_url;
        
        let response = client
            .post(url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/event-stream")
            .header("mcp-session-id", session_id)
            .json(request)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send request to Streamable server: {}", e))?;
        
        if !response.status().is_success() {
            anyhow::bail!("Streamable server returned error status: {}", response.status());
        }
        
        // Read the response body
        let response_text = response
            .text()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to read response from Streamable server: {}", e))?;
        
        // For Streamable servers, the response should contain SSE data
        // Extract the JSON from the SSE format
        if let Some(data_line) = response_text.lines().find(|line| line.starts_with("data: ")) {
            if let Some(json_data) = data_line.strip_prefix("data: ") {
                // Validate it's proper JSON
                if let Ok(_) = serde_json::from_str::<Value>(json_data) {
                    Ok(json_data.to_string())
                } else {
                    Ok(json_data.to_string()) // Return as-is if not JSON
                }
            } else {
                Ok(response_text)
            }
        } else {
            // If no SSE format, return the response as-is
            Ok(response_text)
        }
    }
    
    fn parse_tools_response(&self, response: &str) -> Result<Vec<McpFunction>> {
        let parsed: Value = serde_json::from_str(response)?;
        
        if let Some(error) = parsed.get("error") {
            anyhow::bail!("MCP server error: {}", error);
        }
        
        let tools = parsed
            .get("result")
            .and_then(|r| r.get("tools"))
            .and_then(|t| t.as_array())
            .ok_or_else(|| anyhow::anyhow!("Invalid tools response format"))?;
        
        let mut functions = Vec::new();
        for tool in tools {
            if let (Some(name), Some(description)) = (
                tool.get("name").and_then(|n| n.as_str()),
                tool.get("description").and_then(|d| d.as_str())
            ) {
                functions.push(McpFunction {
                    name: name.to_string(),
                    description: description.to_string(),
                    parameters: tool.get("inputSchema").cloned(),
                });
            }
        }
        
        Ok(functions)
    }
}