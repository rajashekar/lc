use crate::{chat, config::Config, models_cache::ModelsCache, provider::ChatRequest};
use anyhow::Result;
use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::Json,
    routing::{get, post},
    Router,
};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

#[derive(Clone)]
pub struct ProxyState {
    pub config: Config,
    pub api_key: Option<String>,
    pub provider_filter: Option<String>,
    pub model_filter: Option<String>,
}

#[derive(Deserialize)]
pub struct ProxyModelsQuery {
    #[serde(default)]
    pub provider: Option<String>,
}

#[derive(Serialize)]
pub struct ProxyModelsResponse {
    pub object: String,
    pub data: Vec<ProxyModel>,
}

#[derive(Serialize)]
pub struct ProxyModel {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub owned_by: String,
}

#[derive(Deserialize)]
pub struct ProxyChatRequest {
    pub model: String,
    pub messages: Vec<crate::provider::Message>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

#[derive(Serialize)]
pub struct ProxyChatResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ProxyChoice>,
    pub usage: ProxyUsage,
}

#[derive(Serialize)]
pub struct ProxyChoice {
    pub index: u32,
    pub message: crate::provider::Message,
    pub finish_reason: String,
}

#[derive(Serialize)]
pub struct ProxyUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

pub async fn start_proxy_server(
    host: String,
    port: u16,
    provider_filter: Option<String>,
    model_filter: Option<String>,
    api_key: Option<String>,
) -> Result<()> {
    let config = Config::load()?;

    // Generate API key if requested
    let final_api_key = if api_key.is_some() { api_key } else { None };

    let state = ProxyState {
        config,
        api_key: final_api_key.clone(),
        provider_filter,
        model_filter,
    };

    let app = Router::new()
        .route("/models", get(list_models))
        .route("/v1/models", get(list_models))
        .route("/chat/completions", post(chat_completions))
        .route("/v1/chat/completions", post(chat_completions))
        .layer(CorsLayer::permissive())
        .with_state(Arc::new(state));

    let addr = format!("{}:{}", host, port);
    println!("{} Starting proxy server on {}", "🚀".blue(), addr.bold());

    if let Some(ref key) = final_api_key {
        println!(
            "{} Authentication enabled with API key: {}",
            "🔐".yellow(),
            key
        );
    } else {
        println!("{} No authentication required", "⚠️".yellow());
    }

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    println!("{} Server listening on http://{}", "✓".green(), addr);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn authenticate(headers: &HeaderMap, state: &ProxyState) -> Result<(), StatusCode> {
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

async fn list_models(
    Query(query): Query<ProxyModelsQuery>,
    State(state): State<Arc<ProxyState>>,
    headers: HeaderMap,
) -> Result<Json<ProxyModelsResponse>, StatusCode> {
    // Authenticate if API key is configured
    authenticate(&headers, &state).await?;

    let mut models = Vec::new();
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Use models cache for fast response
    let cache = ModelsCache::load().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Check if cache needs refresh and refresh in background if needed
    if cache.needs_refresh() {
        // Refresh cache in background, but don't block the response
        tokio::spawn(async {
            if let Ok(mut bg_cache) = ModelsCache::load() {
                let _ = bg_cache.refresh().await;
            }
        });
    }

    // Get cached models
    let cached_models = cache.get_all_models();

    for cached_model in cached_models {
        let provider_name = &cached_model.provider;
        let model_name = &cached_model.model;
        let model_id = format!("{}:{}", provider_name, model_name);

        // Apply provider filter if specified
        if let Some(ref provider_filter) = state.provider_filter {
            if provider_name != provider_filter {
                continue;
            }
        }

        // Apply query provider filter if specified
        if let Some(ref query_provider) = query.provider {
            if provider_name != query_provider {
                continue;
            }
        }

        // Apply model filter if specified
        if let Some(ref model_filter) = state.model_filter {
            if !model_id.contains(model_filter) && model_name != model_filter {
                continue;
            }
        }

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

    Ok(Json(response))
}

async fn chat_completions(
    State(state): State<Arc<ProxyState>>,
    headers: HeaderMap,
    Json(request): Json<ProxyChatRequest>,
) -> Result<Json<ProxyChatResponse>, StatusCode> {
    // Authenticate if API key is configured
    authenticate(&headers, &state).await?;

    // Parse the model to determine provider and model name
    let (provider_name, model_name) =
        parse_model_string(&request.model, &state.config).map_err(|_| StatusCode::BAD_REQUEST)?;

    // Check if provider is allowed by filter
    if let Some(ref provider_filter) = state.provider_filter {
        if provider_name != *provider_filter {
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    // Check if model is allowed by filter
    if let Some(ref model_filter) = state.model_filter {
        if !request.model.contains(model_filter) && model_name != *model_filter {
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    // Create client for the provider
    let mut config_mut = state.config.clone();
    let client = chat::create_authenticated_client(&mut config_mut, &provider_name)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Convert to internal chat request format
    let chat_request = ChatRequest {
        model: model_name.clone(),
        messages: request.messages,
        max_tokens: request.max_tokens,
        temperature: request.temperature,
        tools: None,  // Proxy doesn't support tools yet
        stream: None, // Proxy doesn't support streaming yet
    };

    // Send the request
    let response_text = client
        .chat(&chat_request)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Create response in OpenAI format
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let response = ProxyChatResponse {
        id: format!("chatcmpl-{}", uuid::Uuid::new_v4()),
        object: "chat.completion".to_string(),
        created: current_time,
        model: request.model,
        choices: vec![ProxyChoice {
            index: 0,
            message: crate::provider::Message {
                role: "assistant".to_string(),
                content_type: crate::provider::MessageContent::Text {
                    content: Some(response_text),
                },
                tool_calls: None,
                tool_call_id: None,
            },
            finish_reason: "stop".to_string(),
        }],
        usage: ProxyUsage {
            prompt_tokens: 0, // We don't track token usage currently
            completion_tokens: 0,
            total_tokens: 0,
        },
    };

    Ok(Json(response))
}

pub fn parse_model_string(model: &str, config: &Config) -> Result<(String, String)> {
    // Check if it's an alias first
    if let Some(alias_target) = config.get_alias(model) {
        if alias_target.contains(':') {
            let parts: Vec<&str> = alias_target.splitn(2, ':').collect();
            if parts.len() == 2 {
                return Ok((parts[0].to_string(), parts[1].to_string()));
            }
        }
        return Err(anyhow::anyhow!("Invalid alias target format"));
    }

    // Check if it contains provider:model format
    if model.contains(':') {
        let parts: Vec<&str> = model.splitn(2, ':').collect();
        if parts.len() == 2 {
            let provider_name = parts[0].to_string();
            let model_name = parts[1].to_string();

            // Validate provider exists
            if config.has_provider(&provider_name) {
                return Ok((provider_name, model_name));
            }
        }
    }

    // If no provider specified, use default provider
    if let Some(default_provider) = &config.default_provider {
        return Ok((default_provider.clone(), model.to_string()));
    }

    Err(anyhow::anyhow!(
        "Could not determine provider for model: {}",
        model
    ))
}

pub fn generate_api_key() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();

    let key: String = (0..32)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    format!("sk-{}", key)
}
