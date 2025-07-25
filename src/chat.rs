use anyhow::Result;
use crate::provider::{OpenAIClient, ChatRequest, Message};
use crate::database::ChatEntry;
use crate::config::Config;
use chrono::{DateTime, Utc};

pub async fn send_chat_request(
    client: &OpenAIClient,
    model: &str,
    prompt: &str,
    history: &[ChatEntry],
    system_prompt: Option<&str>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
) -> Result<String> {
    let mut messages = Vec::new();
    
    // Add system prompt if provided
    if let Some(sys_prompt) = system_prompt {
        messages.push(Message {
            role: "system".to_string(),
            content: sys_prompt.to_string(),
        });
    }
    
    // Add conversation history
    for entry in history {
        messages.push(Message {
            role: "user".to_string(),
            content: entry.question.clone(),
        });
        messages.push(Message {
            role: "assistant".to_string(),
            content: entry.response.clone(),
        });
    }
    
    // Add current prompt
    messages.push(Message {
        role: "user".to_string(),
        content: prompt.to_string(),
    });
    
    let request = ChatRequest {
        model: model.to_string(),
        messages,
        max_tokens: max_tokens.or(Some(1024)),
        temperature: temperature.or(Some(0.7)),
    };
    
    client.chat(&request).await
}

pub async fn get_or_refresh_token(config: &mut Config, provider_name: &str, client: &OpenAIClient) -> Result<String> {
    // Check if provider has a token_url configured
    let token_url = match config.get_token_url(provider_name) {
        Some(url) => url.clone(),
        None => {
            // No token_url, use regular API key
            let provider_config = config.get_provider(provider_name)?;
            return provider_config.api_key.clone()
                .ok_or_else(|| anyhow::anyhow!("No API key or token URL configured for provider '{}'", provider_name));
        }
    };
    
    // Check if we have a valid cached token
    if let Some(cached_token) = config.get_cached_token(provider_name) {
        if Utc::now() < cached_token.expires_at {
            // Token is still valid
            return Ok(cached_token.token.clone());
        }
    }
    
    // Token is expired or doesn't exist, fetch a new one
    let token_response = client.get_token_from_url(&token_url).await?;
    
    // Convert Unix timestamp to DateTime<Utc>
    let expires_at = DateTime::from_timestamp(token_response.expires_at, 0)
        .ok_or_else(|| anyhow::anyhow!("Invalid expires_at timestamp: {}", token_response.expires_at))?;
    
    // Cache the new token
    config.set_cached_token(provider_name.to_string(), token_response.token.clone(), expires_at)?;
    config.save()?;
    
    Ok(token_response.token)
}

pub async fn create_authenticated_client(config: &mut Config, provider_name: &str) -> Result<OpenAIClient> {
    // Clone the provider config to avoid borrowing issues
    let provider_config = config.get_provider(provider_name)?.clone();
    
    // Create a temporary client with the API key for token retrieval
    let temp_client = OpenAIClient::new_with_headers(
        provider_config.endpoint.clone(),
        provider_config.api_key.clone().unwrap_or_default(),
        provider_config.models_path.clone(),
        provider_config.chat_path.clone(),
        provider_config.headers.clone(),
    );
    
    // Get the appropriate authentication token
    let auth_token = get_or_refresh_token(config, provider_name, &temp_client).await?;
    
    // Create the final client with the authentication token
    let client = OpenAIClient::new_with_headers(
        provider_config.endpoint,
        auth_token,
        provider_config.models_path,
        provider_config.chat_path,
        provider_config.headers,
    );
    
    Ok(client)
}