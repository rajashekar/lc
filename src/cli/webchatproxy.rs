//! Web chat proxy commands

use crate::cli::WebChatProxyCommands;
use anyhow::Result;
use colored::*;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, Json},
    routing::{get, post},
    Router,
};
use std::sync::Arc;

// App state for the web server
#[derive(Clone)]
struct AppState {
    config: crate::config::Config,
}

/// Handle webchat proxy commands
pub async fn handle(command: WebChatProxyCommands) -> Result<()> {
    match command {
        WebChatProxyCommands::Start { port, host, cors } => {
            handle_start(port, host, cors).await
        }
    }
}

async fn handle_start(port: u16, host: String, cors: bool) -> Result<()> {
    println!(
        "{} Starting Web Chat Proxy server...",
        "🌐".blue()
    );
    println!("  {} {}:{}", "Address:".bold(), host, port);
    println!("  {} {}", "CORS:".bold(), if cors { "Enabled".green() } else { "Disabled".yellow() });
    
    println!("\n{}", "Available endpoints:".bold().blue());
    println!("  {} http://{}:{}/", "•".blue(), host, port);
    println!("    Web interface for chat");
    println!("  {} http://{}:{}/models", "•".blue(), host, port);
    println!("    List available models");
    println!("  {} http://{}:{}/v1/models", "•".blue(), host, port);
    println!("    OpenAI-compatible models endpoint");
    println!("  {} http://{}:{}/chat/completions", "•".blue(), host, port);
    println!("    Chat completions endpoint");
    println!("  {} http://{}:{}/v1/chat/completions", "•".blue(), host, port);
    println!("    OpenAI-compatible chat endpoint");
    
    println!("\n{} Press Ctrl+C to stop the server\n", "💡".yellow());
    
    // Start the webchat proxy server
    start_webchat_server(host, port, cors).await
}

async fn start_webchat_server(host: String, port: u16, cors: bool) -> Result<()> {
    let config = crate::config::Config::load()?;
    let state = Arc::new(AppState { config });
    
    // Build the router
    let mut app = Router::new()
        .route("/", get(serve_index))
        .route("/models", get(list_models))
        .route("/v1/models", get(list_models))
        .route("/chat/completions", post(chat_completions))
        .route("/v1/chat/completions", post(chat_completions))
        .with_state(state);
    
    // Add CORS if enabled
    if cors {
        use tower_http::cors::CorsLayer;
        app = app.layer(CorsLayer::permissive());
    }
    
    let addr = format!("{}:{}", host, port);
    println!("{} Server listening on http://{}", "✓".green(), addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

// Serve a simple HTML interface
async fn serve_index() -> Html<&'static str> {
    
    Html(r#"
<!DOCTYPE html>
<html>
<head>
    <title>LC Web Chat</title>
    <style>
        body {
            font-family: system-ui, -apple-system, sans-serif;
            max-width: 800px;
            margin: 0 auto;
            padding: 20px;
            background: #f5f5f5;
        }
        h1 {
            color: #333;
        }
        .container {
            background: white;
            border-radius: 8px;
            padding: 20px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }
        .chat-box {
            height: 400px;
            overflow-y: auto;
            border: 1px solid #ddd;
            border-radius: 4px;
            padding: 10px;
            margin-bottom: 10px;
            background: #fafafa;
        }
        .message {
            margin: 10px 0;
            padding: 10px;
            border-radius: 4px;
        }
        .user-message {
            background: #007bff;
            color: white;
            text-align: right;
        }
        .assistant-message {
            background: #e9ecef;
            color: #333;
        }
        .input-group {
            display: flex;
            gap: 10px;
        }
        input[type="text"] {
            flex: 1;
            padding: 10px;
            border: 1px solid #ddd;
            border-radius: 4px;
        }
        button {
            padding: 10px 20px;
            background: #007bff;
            color: white;
            border: none;
            border-radius: 4px;
            cursor: pointer;
        }
        button:hover {
            background: #0056b3;
        }
        button:disabled {
            background: #ccc;
            cursor: not-allowed;
        }
        .model-select {
            margin-bottom: 10px;
        }
        select {
            padding: 8px;
            border: 1px solid #ddd;
            border-radius: 4px;
            width: 100%;
        }
    </style>
</head>
<body>
    <h1>🤖 LC Web Chat</h1>
    <div class="container">
        <div class="model-select">
            <label for="model">Model:</label>
            <select id="model">
                <option value="gpt-4o">gpt-4o</option>
                <option value="gpt-4o-mini">gpt-4o-mini</option>
                <option value="claude-3-5-sonnet-latest">claude-3-5-sonnet-latest</option>
                <option value="claude-3-5-haiku-latest">claude-3-5-haiku-latest</option>
            </select>
        </div>
        <div id="chat" class="chat-box"></div>
        <div class="input-group">
            <input type="text" id="message" placeholder="Type your message..." autofocus>
            <button id="send" onclick="sendMessage()">Send</button>
        </div>
    </div>
    
    <script>
        // Load available models
        fetch('/models')
            .then(res => res.json())
            .then(data => {
                const select = document.getElementById('model');
                select.innerHTML = '';
                data.data.forEach(model => {
                    const option = document.createElement('option');
                    option.value = model.id;
                    option.textContent = model.id;
                    select.appendChild(option);
                });
            })
            .catch(err => console.error('Failed to load models:', err));
        
        const messages = [];
        
        function addMessage(role, content) {
            messages.push({ role, content });
            const chat = document.getElementById('chat');
            const div = document.createElement('div');
            div.className = `message ${role}-message`;
            div.textContent = content;
            chat.appendChild(div);
            chat.scrollTop = chat.scrollHeight;
        }
        
        async function sendMessage() {
            const input = document.getElementById('message');
            const button = document.getElementById('send');
            const model = document.getElementById('model').value;
            const message = input.value.trim();
            
            if (!message) return;
            
            input.value = '';
            button.disabled = true;
            
            addMessage('user', message);
            
            try {
                const response = await fetch('/chat/completions', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify({
                        model: model,
                        messages: messages,
                    }),
                });
                
                if (!response.ok) {
                    throw new Error(`HTTP ${response.status}`);
                }
                
                const data = await response.json();
                if (data.choices && data.choices[0]) {
                    addMessage('assistant', data.choices[0].message.content);
                }
            } catch (error) {
                addMessage('assistant', `Error: ${error.message}`);
            } finally {
                button.disabled = false;
                input.focus();
            }
        }
        
        document.getElementById('message').addEventListener('keypress', (e) => {
            if (e.key === 'Enter' && !e.shiftKey) {
                e.preventDefault();
                sendMessage();
            }
        });
    </script>
</body>
</html>
    "#)
}

// List available models
async fn list_models(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    use crate::services::proxy::{ProxyModel, ProxyModelsResponse};
    use crate::models::cache::ModelsCache;
    
    let mut models = Vec::new();
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or(std::time::Duration::from_secs(0))
        .as_secs();
    
    // Use models cache for fast response
    let cache = ModelsCache::load().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Get cached models
    let cached_models = cache.get_all_models();
    
    for cached_model in cached_models {
        let provider_name = &cached_model.provider;
        let model_name = &cached_model.model;
        let model_id = format!("{}:{}", provider_name, model_name);
        
        models.push(ProxyModel {
            id: model_id,
            object: "model".to_string(),
            created: current_time,
            owned_by: provider_name.clone(),
        });
    }
    
    let response = ProxyModelsResponse {
        object: "list".to_string(),
        data: models,
    };
    
    Ok(Json(serde_json::to_value(response).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?))
}

// Handle chat completions
async fn chat_completions(
    State(state): State<Arc<AppState>>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    use crate::services::proxy::{ProxyChatRequest, ProxyChatResponse, ProxyChoice, ProxyUsage};
    use crate::core::provider::{ChatRequest, Message, MessageContent};
    
    // Parse the request
    let proxy_request: ProxyChatRequest = serde_json::from_value(request)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    // Parse model string to get provider and model
    let (provider_name, model_name) = crate::services::proxy::parse_model_string(&proxy_request.model, &state.config)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    // Create client for the provider
    let mut config_mut = state.config.clone();
    let client = crate::core::chat::create_authenticated_client(&mut config_mut, &provider_name)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Convert to internal chat request format
    let chat_request = ChatRequest {
        model: model_name.clone(),
        messages: proxy_request.messages,
        max_tokens: proxy_request.max_tokens,
        temperature: proxy_request.temperature,
        tools: None,
        stream: None,
    };
    
    // Send the request
    let response_text = client
        .chat(&chat_request)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Create response in OpenAI format
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or(std::time::Duration::from_secs(0))
        .as_secs();
    
    let response = ProxyChatResponse {
        id: format!("chatcmpl-{}", uuid::Uuid::new_v4()),
        object: "chat.completion".to_string(),
        created: current_time,
        model: proxy_request.model,
        choices: vec![ProxyChoice {
            index: 0,
            message: Message {
                role: "assistant".to_string(),
                content_type: MessageContent::Text {
                    content: Some(response_text),
                },
                tool_calls: None,
                tool_call_id: None,
            },
            finish_reason: "stop".to_string(),
        }],
        usage: ProxyUsage {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
        },
    };
    
    Ok(Json(serde_json::to_value(response).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?))
}

