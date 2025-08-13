use anyhow::Result;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Json,
    routing::{get, post},
    Router,
};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use uuid::Uuid;

// Configuration for webchatproxy
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WebChatProxyConfig {
    pub providers: HashMap<String, WebChatProxyProviderConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WebChatProxyProviderConfig {
    pub auth_token: Option<String>,
}

impl WebChatProxyConfig {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_file_path()?;

        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            let config: WebChatProxyConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            // Create default config
            let config = WebChatProxyConfig {
                providers: HashMap::new(),
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
        let content = toml::to_string_pretty(self)?;
        fs::write(&config_path, content)?;
        Ok(())
    }

    pub fn set_provider_auth(&mut self, provider: &str, auth_token: &str) -> Result<()> {
        let provider_config = WebChatProxyProviderConfig {
            auth_token: Some(auth_token.to_string()),
        };
        self.providers.insert(provider.to_string(), provider_config);
        Ok(())
    }

    pub fn get_provider_auth(&self, provider: &str) -> Option<&String> {
        self.providers.get(provider)?.auth_token.as_ref()
    }

    fn config_file_path() -> Result<PathBuf> {
        let config_dir =
            dirs::config_dir().ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;

        Ok(config_dir.join("lc").join("webchatproxy.toml"))
    }
}

// Server state
#[derive(Clone)]
pub struct WebChatProxyState {
    pub provider: String,
    pub api_key: Option<String>,
    pub config: WebChatProxyConfig,
}

// OpenAI-compatible request/response structures
#[derive(Deserialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[allow(dead_code)]
    pub max_tokens: Option<u32>,
    #[allow(dead_code)]
    pub temperature: Option<f32>,
    #[allow(dead_code)]
    pub stream: Option<bool>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Serialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ChatChoice>,
    pub usage: ChatUsage,
}

#[derive(Serialize)]
pub struct ChatChoice {
    pub index: u32,
    pub message: ChatMessage,
    pub finish_reason: String,
}

#[derive(Serialize)]
pub struct ChatUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

// OpenAI-compatible models response structures
#[derive(Serialize)]
pub struct ModelsListResponse {
    pub object: String,
    pub data: Vec<ModelInfo>,
}

#[derive(Serialize)]
pub struct ModelInfo {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub owned_by: String,
}

// Kagi-specific structures
#[derive(Serialize)]
pub struct KagiRequest {
    pub focus: KagiFocus,
    pub profile: KagiProfile,
}

#[derive(Serialize)]
pub struct KagiFocus {
    pub thread_id: Option<String>,
    pub branch_id: String,
    pub prompt: String,
}

#[derive(Serialize)]
pub struct KagiProfile {
    pub id: Option<String>,
    pub personalizations: bool,
    pub internet_access: bool,
    pub model: String,
    pub lens_id: Option<String>,
}

// Kagi models structures
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KagiModelsResponse {
    pub profiles: Vec<KagiModelProfile>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KagiModelProfile {
    pub id: Option<String>,
    pub name: String,
    pub model: String,
    pub model_name: String,
    pub model_provider: String,
    pub model_input_limit: Option<u32>,
    pub scorecard: KagiScorecard,
    pub model_provider_name: String,
    pub internet_access: bool,
    pub personalizations: bool,
    pub shortcut: String,
    pub is_default_profile: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KagiScorecard {
    pub speed: f32,
    pub accuracy: f32,
    pub cost: f32,
    pub context_window: f32,
    pub privacy: f32,
    pub description: Option<String>,
    pub recommended: bool,
}

// Daemon management structures
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DaemonInfo {
    pub pid: u32,
    pub host: String,
    pub port: u16,
    pub provider: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DaemonRegistry {
    pub daemons: HashMap<String, DaemonInfo>,
}

impl DaemonRegistry {
    pub fn load() -> Result<Self> {
        let registry_path = Self::registry_file_path()?;

        if registry_path.exists() {
            let content = fs::read_to_string(&registry_path)?;
            let registry: DaemonRegistry = toml::from_str(&content)?;
            Ok(registry)
        } else {
            Ok(DaemonRegistry {
                daemons: HashMap::new(),
            })
        }
    }

    pub fn save(&self) -> Result<()> {
        let registry_path = Self::registry_file_path()?;

        // Ensure directory exists
        if let Some(parent) = registry_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        fs::write(&registry_path, content)?;
        Ok(())
    }

    pub fn add_daemon(&mut self, provider: String, info: DaemonInfo) {
        self.daemons.insert(provider, info);
    }

    pub fn remove_daemon(&mut self, provider: &str) -> Option<DaemonInfo> {
        self.daemons.remove(provider)
    }

    #[allow(dead_code)]
    pub fn get_daemon(&self, provider: &str) -> Option<&DaemonInfo> {
        self.daemons.get(provider)
    }

    pub fn list_daemons(&self) -> &HashMap<String, DaemonInfo> {
        &self.daemons
    }

    fn registry_file_path() -> Result<PathBuf> {
        let config_dir =
            dirs::config_dir().ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;

        Ok(config_dir.join("lc").join("webchatproxy_daemons.toml"))
    }
}

// Start the webchatproxy server
pub async fn start_webchatproxy_server(
    host: String,
    port: u16,
    provider: String,
    api_key: Option<String>,
) -> Result<()> {
    let config = WebChatProxyConfig::load()?;

    let state = WebChatProxyState {
        provider: provider.clone(),
        api_key,
        config,
    };

    let app = Router::new()
        .route("/chat/completions", post(chat_completions))
        .route("/v1/chat/completions", post(chat_completions))
        .route("/models", get(list_models))
        .route("/v1/models", get(list_models))
        .layer(CorsLayer::permissive())
        .with_state(Arc::new(state));

    let addr = format!("{}:{}", host, port);
    println!(
        "{} Starting webchatproxy server on {}",
        "🚀".blue(),
        addr.bold()
    );

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    println!("{} Server listening on http://{}", "✓".green(), addr);

    axum::serve(listener, app).await?;

    Ok(())
}

// Authentication middleware
async fn authenticate(headers: &HeaderMap, state: &WebChatProxyState) -> Result<(), StatusCode> {
    if let Some(expected_key) = &state.api_key {
        if let Some(auth_header) = headers.get("authorization") {
            if let Ok(auth_str) = auth_header.to_str() {
                if let Some(token) = auth_str.strip_prefix("Bearer ") {
                    if token == expected_key {
                        return Ok(());
                    }
                }
            }
        }
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(())
}

// Main chat completions endpoint
async fn chat_completions(
    State(state): State<Arc<WebChatProxyState>>,
    headers: HeaderMap,
    Json(request): Json<ChatCompletionRequest>,
) -> Result<Json<ChatCompletionResponse>, StatusCode> {
    println!(
        "🔄 Received chat completion request for provider: {}",
        state.provider
    );

    // Authenticate if API key is configured
    if let Err(e) = authenticate(&headers, &state).await {
        println!("❌ Authentication failed");
        return Err(e);
    }

    match state.provider.as_str() {
        "kagi" => handle_kagi_request(&state, request).await,
        _ => {
            println!("❌ Unsupported provider: {}", state.provider);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

// List models endpoint
async fn list_models(
    State(state): State<Arc<WebChatProxyState>>,
    headers: HeaderMap,
) -> Result<Json<ModelsListResponse>, StatusCode> {
    println!(
        "🔄 Received models list request for provider: {}",
        state.provider
    );

    // Authenticate if API key is configured
    if let Err(e) = authenticate(&headers, &state).await {
        println!("❌ Authentication failed");
        return Err(e);
    }

    match state.provider.as_str() {
        "kagi" => handle_kagi_models_request(&state).await,
        _ => {
            println!("❌ Unsupported provider: {}", state.provider);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

// Handle Kagi models list request
async fn handle_kagi_models_request(
    _state: &WebChatProxyState,
) -> Result<Json<ModelsListResponse>, StatusCode> {
    match fetch_kagi_models().await {
        Ok(kagi_models) => {
            let current_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();

            let models: Vec<ModelInfo> = kagi_models
                .into_iter()
                .map(|model| ModelInfo {
                    id: model.model.clone(),
                    object: "model".to_string(),
                    created: current_time,
                    owned_by: model.model_provider_name.clone(),
                })
                .collect();

            let response = ModelsListResponse {
                object: "list".to_string(),
                data: models,
            };

            println!(
                "✅ Successfully fetched {} Kagi models",
                response.data.len()
            );
            Ok(Json(response))
        }
        Err(e) => {
            println!("❌ Failed to fetch Kagi models: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Handle Kagi-specific requests
async fn handle_kagi_request(
    state: &WebChatProxyState,
    request: ChatCompletionRequest,
) -> Result<Json<ChatCompletionResponse>, StatusCode> {
    // Get Kagi auth token
    let auth_token = state
        .config
        .get_provider_auth("kagi")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Extract the user message (last message with role "user")
    let user_message = request
        .messages
        .iter()
        .rev()
        .find(|msg| msg.role == "user")
        .ok_or(StatusCode::BAD_REQUEST)?;

    // Create Kagi request
    let kagi_request = KagiRequest {
        focus: KagiFocus {
            thread_id: None,
            branch_id: "00000000-0000-4000-0000-000000000000".to_string(),
            prompt: user_message.content.clone(),
        },
        profile: KagiProfile {
            id: None,
            personalizations: false,
            internet_access: true,
            model: request.model.clone(),
            lens_id: None,
        },
    };

    // Make request to Kagi using optimized client with connection pooling
    let client = reqwest::Client::builder()
        .pool_max_idle_per_host(10)
        .pool_idle_timeout(std::time::Duration::from_secs(90))
        .tcp_keepalive(std::time::Duration::from_secs(60))
        .timeout(std::time::Duration::from_secs(60))
        .connect_timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let response = client
        .post("https://kagi.com/assistant/prompt")
        .header("Content-Type", "application/json")
        .header("x-kagi-authorization", auth_token)
        .json(&kagi_request)
        .send()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !response.status().is_success() {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let response_text = response
        .text()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Parse Kagi response
    let assistant_response =
        parse_kagi_response(&response_text).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Create OpenAI-compatible response
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let openai_response = ChatCompletionResponse {
        id: format!("chatcmpl-{}", Uuid::new_v4()),
        object: "chat.completion".to_string(),
        created: current_time,
        model: request.model,
        choices: vec![ChatChoice {
            index: 0,
            message: ChatMessage {
                role: "assistant".to_string(),
                content: assistant_response,
            },
            finish_reason: "stop".to_string(),
        }],
        usage: ChatUsage {
            prompt_tokens: 0, // Kagi doesn't provide token counts
            completion_tokens: 0,
            total_tokens: 0,
        },
    };

    println!("✅ Successfully processed Kagi request");
    Ok(Json(openai_response))
}

// Parse Kagi's HTML response to extract the assistant's message
fn parse_kagi_response(html: &str) -> Result<String> {
    let lines: Vec<&str> = html.lines().collect();

    // Look for any <div hidden> tags that contain JSON with message content
    for line in lines.iter() {
        if line.contains("<div hidden>") && line.contains("{") {
            // Extract content between <div hidden> and </div>
            if let Some(start) = line.find("<div hidden>") {
                let content_start = start + 12; // Length of '<div hidden>'
                if let Some(end) = line[content_start..].find("</div>") {
                    let json_content = &line[content_start..content_start + end];

                    // Decode HTML entities
                    let decoded_json = json_content
                        .replace("&quot;", "\"")
                        .replace("&lt;", "<")
                        .replace("&gt;", ">")
                        .replace("&amp;", "&")
                        .replace("&#39;", "'");

                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&decoded_json) {
                        // Check if this has state "done" - this is the final response
                        if let Some(state) = parsed.get("state").and_then(|v| v.as_str()) {
                            if state == "done" {
                                // First try to get the markdown content
                                if let Some(md_content) = parsed.get("md").and_then(|v| v.as_str())
                                {
                                    if !md_content.trim().is_empty() {
                                        return Ok(md_content.to_string());
                                    }
                                }

                                // Fallback to reply content (HTML)
                                if let Some(reply_content) =
                                    parsed.get("reply").and_then(|v| v.as_str())
                                {
                                    if !reply_content.trim().is_empty() {
                                        let stripped = strip_html_tags(reply_content);
                                        return Ok(stripped);
                                    }
                                }
                            }
                        }

                        // Also check for any JSON that has "md" or "reply" fields with substantial content
                        if let Some(md_content) = parsed.get("md").and_then(|v| v.as_str()) {
                            if !md_content.trim().is_empty() && md_content.len() > 10 {
                                return Ok(md_content.to_string());
                            }
                        }

                        if let Some(reply_content) = parsed.get("reply").and_then(|v| v.as_str()) {
                            if !reply_content.trim().is_empty() && reply_content.len() > 10 {
                                let stripped = strip_html_tags(reply_content);
                                return Ok(stripped);
                            }
                        }
                    }
                }
            }
        }
    }

    anyhow::bail!("Could not parse Kagi response - no meaningful content found")
}

// Simple HTML tag stripper
fn strip_html_tags(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;

    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }

    // Decode common HTML entities
    result
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#x27;", "'")
}
// Daemon management functions
pub async fn start_webchatproxy_daemon(
    host: String,
    port: u16,
    provider: String,
    api_key: Option<String>,
) -> Result<()> {
    use std::env;
    use std::fs::OpenOptions;

    // Get the current executable path
    let current_exe = env::current_exe()?;

    // Create log directory
    let log_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
        .join("lc");
    fs::create_dir_all(&log_dir)?;

    let log_file = log_dir.join(format!("{}.log", provider));

    // Build command arguments - remove the --daemon flag to prevent infinite recursion
    let mut args = vec![
        "w".to_string(),
        "start".to_string(),
        provider.clone(),
        "--port".to_string(),
        port.to_string(),
        "--host".to_string(),
        host.clone(),
    ];

    if let Some(ref key) = api_key {
        args.push("--key".to_string());
        args.push(key.clone());
    }

    // Create log file handles
    let log_file_handle = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file)?;

    // Start the daemon process with proper detachment
    let child = Command::new(&current_exe)
        .args(&args)
        .stdout(Stdio::from(log_file_handle.try_clone()?))
        .stderr(Stdio::from(log_file_handle))
        .stdin(Stdio::null())
        .spawn()?;

    let pid = child.id();

    // Give the process a moment to start
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Check if the process is still running
    #[cfg(unix)]
    {
        use nix::sys::signal;
        use nix::unistd::Pid;

        let process_pid = Pid::from_raw(pid as i32);
        match signal::kill(process_pid, None) {
            Ok(_) => {
                // Process is running, register it
                let mut registry = DaemonRegistry::load()?;
                let daemon_info = DaemonInfo {
                    pid,
                    host: host.clone(),
                    port,
                    provider: provider.clone(),
                    started_at: chrono::Utc::now(),
                };

                registry.add_daemon(provider.clone(), daemon_info);
                registry.save()?;

                println!(
                    "{} WebChatProxy daemon started for '{}' (PID: {})",
                    "✓".green(),
                    provider,
                    pid
                );
                println!("{} Server running on {}:{}", "🚀".blue(), host, port);
                println!("{} Logs: {}", "📝".blue(), log_file.display());

                Ok(())
            }
            Err(_) => {
                anyhow::bail!("Failed to start daemon process - process died immediately");
            }
        }
    }

    #[cfg(not(unix))]
    {
        // On non-Unix systems, assume the process started successfully
        let mut registry = DaemonRegistry::load()?;
        let daemon_info = DaemonInfo {
            pid,
            host: host.clone(),
            port,
            provider: provider.clone(),
            started_at: chrono::Utc::now(),
        };

        registry.add_daemon(provider.clone(), daemon_info);
        registry.save()?;

        println!(
            "{} WebChatProxy daemon started for '{}' (PID: {})",
            "✓".green(),
            provider,
            pid
        );
        println!("{} Server running on {}:{}", "🚀".blue(), host, port);
        println!("{} Logs: {}", "📝".blue(), log_file.display());

        Ok(())
    }
}

pub async fn stop_webchatproxy_daemon(provider: &str) -> Result<()> {
    let mut registry = DaemonRegistry::load()?;

    if let Some(_daemon_info) = registry.remove_daemon(provider) {
        // Try to kill the process
        #[cfg(unix)]
        {
            use nix::sys::signal::{self, Signal};
            use nix::unistd::Pid;

            let pid = Pid::from_raw(_daemon_info.pid as i32);
            match signal::kill(pid, Signal::SIGTERM) {
                Ok(_) => {
                    registry.save()?;
                    Ok(())
                }
                Err(e) => {
                    // Process might already be dead, remove from registry anyway
                    registry.save()?;
                    Err(anyhow::anyhow!(
                        "Failed to kill process {}: {}",
                        _daemon_info.pid,
                        e
                    ))
                }
            }
        }

        #[cfg(not(unix))]
        {
            // On non-Unix systems, just remove from registry
            registry.save()?;
            Ok(())
        }
    } else {
        anyhow::bail!("No running daemon found for provider '{}'", provider);
    }
}

pub async fn list_webchatproxy_daemons() -> Result<HashMap<String, DaemonInfo>> {
    let mut registry = DaemonRegistry::load()?;
    let mut active_daemons = HashMap::new();
    let dead_processes: Vec<String> = Vec::new();

    // Check which processes are still alive
    for (provider, daemon_info) in registry.list_daemons().clone() {
        #[cfg(unix)]
        {
            use nix::sys::signal;
            use nix::unistd::Pid;

            let pid = Pid::from_raw(daemon_info.pid as i32);
            match signal::kill(pid, None) {
                Ok(_) => {
                    // Process is alive
                    active_daemons.insert(provider, daemon_info);
                }
                Err(_) => {
                    // Process is dead, mark for removal
                    dead_processes.push(provider);
                }
            }
        }

        #[cfg(not(unix))]
        {
            // On non-Unix systems, assume all registered daemons are active
            active_daemons.insert(provider, daemon_info);
        }
    }

    // Remove dead processes from registry
    for provider in dead_processes {
        registry.remove_daemon(&provider);
    }

    // Save updated registry
    registry.save()?;

    Ok(active_daemons)
}

// Function to fetch Kagi models from the profile_list endpoint
pub async fn fetch_kagi_models() -> Result<Vec<KagiModelProfile>> {
    let config = WebChatProxyConfig::load()?;

    // Get Kagi auth token
    let auth_token = config.get_provider_auth("kagi").ok_or_else(|| {
        anyhow::anyhow!("No Kagi authentication token configured. Set one with 'lc w p kagi auth'")
    })?;

    // Make request to Kagi profile_list endpoint using optimized client with connection pooling
    let client = reqwest::Client::builder()
        .pool_max_idle_per_host(10)
        .pool_idle_timeout(std::time::Duration::from_secs(90))
        .tcp_keepalive(std::time::Duration::from_secs(60))
        .timeout(std::time::Duration::from_secs(60))
        .connect_timeout(std::time::Duration::from_secs(10))
        .build()?;
    let response = client
        .post("https://kagi.com/assistant/profile_list")
        .header("Content-Type", "application/json")
        .header("Cookie", format!("kagi_session={}", auth_token))
        .json(&serde_json::json!({}))
        .send()
        .await?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to fetch Kagi models: HTTP {}", response.status());
    }

    let response_text = response.text().await?;

    // Parse the HTML response to extract JSON data
    parse_kagi_models_response(&response_text)
}

// Parse Kagi's HTML response to extract model profiles
fn parse_kagi_models_response(html: &str) -> Result<Vec<KagiModelProfile>> {
    let lines: Vec<&str> = html.lines().collect();

    // Look for the <div hidden> tag that contains the profiles JSON
    for line in lines.iter() {
        if line.contains("<div hidden>") && line.contains("profiles") {
            // Extract content between <div hidden> and </div>
            if let Some(start) = line.find("<div hidden>") {
                let content_start = start + 12; // Length of '<div hidden>'
                if let Some(end) = line[content_start..].find("</div>") {
                    let json_content = &line[content_start..content_start + end];

                    // Decode HTML entities
                    let decoded_json = json_content
                        .replace("&quot;", "\"")
                        .replace("&lt;", "<")
                        .replace("&gt;", ">")
                        .replace("&amp;", "&")
                        .replace("&#39;", "'");

                    if let Ok(parsed) = serde_json::from_str::<KagiModelsResponse>(&decoded_json) {
                        return Ok(parsed.profiles);
                    }
                }
            }
        }
    }

    anyhow::bail!("Could not parse Kagi models response - no profiles data found")
}
