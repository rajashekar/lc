use anyhow::Result;
use crate::provider::{OpenAIClient, ChatRequest, Message, MessageContent, ContentPart, GeminiClient, GeminiChatRequest, GeminiContent, GeminiPart, GeminiInlineData, GeminiSystemInstruction, GeminiTool, GeminiFunctionDeclaration, GeminiGenerationConfig, GeminiChatResponse, GeminiResponsePart};
use crate::database::ChatEntry;
use crate::config::Config;
use crate::token_utils::TokenCounter;
use crate::model_metadata::MetadataExtractor;
use chrono::{DateTime, Utc};


pub async fn send_chat_request_with_validation(
    client: &LLMClient,
    model: &str,
    prompt: &str,
    history: &[ChatEntry],
    system_prompt: Option<&str>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
    provider_name: &str,
    tools: Option<Vec<crate::provider::Tool>>,
) -> Result<(String, Option<i32>, Option<i32>)> {
    crate::debug_log!("Sending chat request - provider: '{}', model: '{}', prompt length: {}, history entries: {}",
                      provider_name, model, prompt.len(), history.len());
    crate::debug_log!("Request parameters - max_tokens: {:?}, temperature: {:?}", max_tokens, temperature);
    
    // Try to get model metadata for context validation
    crate::debug_log!("Loading model metadata for provider '{}', model '{}'", provider_name, model);
    let model_metadata = get_model_metadata(provider_name, model).await;
    
    if let Some(ref metadata) = model_metadata {
        crate::debug_log!("Found metadata for model '{}' - context_length: {:?}, max_output: {:?}",
                          model, metadata.context_length, metadata.max_output_tokens);
    } else {
        crate::debug_log!("No metadata found for model '{}'", model);
    }
    
    // Create token counter
    crate::debug_log!("Creating token counter for model '{}'", model);
    let token_counter = match TokenCounter::new(model) {
        Ok(counter) => {
            crate::debug_log!("Successfully created token counter for model '{}'", model);
            Some(counter)
        }
        Err(e) => {
            crate::debug_log!("Failed to create token counter for model '{}': {}", model, e);
            eprintln!("Warning: Failed to create token counter for model '{}': {}", model, e);
            None
        }
    };
    
    let mut final_prompt = prompt.to_string();
    let mut final_history = history.to_vec();
    let mut input_tokens = None;
    
    // Validate context size if we have both metadata and token counter
    if let (Some(metadata), Some(ref counter)) = (&model_metadata, &token_counter) {
        if let Some(context_limit) = metadata.context_length {
            // Check if input exceeds context limit
            if counter.exceeds_context_limit(prompt, system_prompt, history, context_limit) {
                println!("⚠️  Input exceeds model context limit ({}k tokens). Truncating...", context_limit / 1000);
                
                // Truncate to fit within context limit
                let (truncated_prompt, truncated_history) = counter.truncate_to_fit(
                    prompt,
                    system_prompt,
                    history,
                    context_limit,
                    metadata.max_output_tokens
                );
                
                final_prompt = truncated_prompt;
                final_history = truncated_history;
                
                if final_history.len() < history.len() {
                    println!("📝 Truncated conversation history from {} to {} messages", history.len(), final_history.len());
                }
                
                if final_prompt.len() < prompt.len() {
                    println!("✂️  Truncated prompt from {} to {} characters", prompt.len(), final_prompt.len());
                }
            }
            
            // Calculate input tokens after potential truncation
            input_tokens = Some(counter.estimate_chat_tokens(&final_prompt, system_prompt, &final_history) as i32);
        }
    } else if let Some(ref counter) = token_counter {
        // No metadata available, but we can still count tokens
        input_tokens = Some(counter.estimate_chat_tokens(&final_prompt, system_prompt, &final_history) as i32);
    }
    
    // Build messages for the request
    let mut messages = Vec::new();
    
    // Add system prompt if provided
    if let Some(sys_prompt) = system_prompt {
        messages.push(Message {
            role: "system".to_string(),
            content_type: MessageContent::Text { content: Some(sys_prompt.to_string()) },
            tool_calls: None,
            tool_call_id: None,
        });
    }
    
    // Add conversation history
    for entry in &final_history {
        messages.push(Message::user(entry.question.clone()));
        messages.push(Message::assistant(entry.response.clone()));
    }
    
    // Add current prompt
    messages.push(Message::user(final_prompt));
    
    let request = ChatRequest {
        model: model.to_string(),
        messages: messages.clone(),
        max_tokens: max_tokens.or(Some(1024)),
        temperature: temperature.or(Some(0.7)),
        tools,
        stream: None, // Non-streaming request
    };
    
    crate::debug_log!("Sending chat request with {} messages, max_tokens: {:?}, temperature: {:?}",
                      messages.len(), request.max_tokens, request.temperature);
    
    // Send the request
    crate::debug_log!("Making API call to chat endpoint...");
    let response = client.chat(&request, model).await?;
    
    crate::debug_log!("Received response from chat API ({} characters)", response.len());
    
    // Calculate output tokens if we have a token counter
    let output_tokens = if let Some(ref counter) = token_counter {
        Some(counter.count_tokens(&response) as i32)
    } else {
        None
    };
    
    // Display token usage if available
    if let (Some(input), Some(output)) = (input_tokens, output_tokens) {
        println!("📊 Token usage: {} input + {} output = {} total",
                 input, output, input + output);
        
        // Show cost estimate if we have pricing info
        if let Some(metadata) = &model_metadata {
            if let (Some(input_price), Some(output_price)) = (metadata.input_price_per_m, metadata.output_price_per_m) {
                let input_cost = (input as f64 / 1_000_000.0) * input_price;
                let output_cost = (output as f64 / 1_000_000.0) * output_price;
                let total_cost = input_cost + output_cost;
                println!("💰 Estimated cost: ${:.6} (${:.6} input + ${:.6} output)",
                         total_cost, input_cost, output_cost);
            }
        }
    }
    
    Ok((response, input_tokens, output_tokens))
}

pub async fn send_chat_request_with_streaming(
    client: &LLMClient,
    model: &str,
    prompt: &str,
    history: &[ChatEntry],
    system_prompt: Option<&str>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
    provider_name: &str,
    tools: Option<Vec<crate::provider::Tool>>,
) -> Result<()> {
    crate::debug_log!("Sending streaming chat request - provider: '{}', model: '{}', prompt length: {}, history entries: {}",
                      provider_name, model, prompt.len(), history.len());
    crate::debug_log!("Request parameters - max_tokens: {:?}, temperature: {:?}", max_tokens, temperature);
    
    // Try to get model metadata for context validation
    crate::debug_log!("Loading model metadata for provider '{}', model '{}'", provider_name, model);
    let model_metadata = get_model_metadata(provider_name, model).await;
    
    if let Some(ref metadata) = model_metadata {
        crate::debug_log!("Found metadata for model '{}' - context_length: {:?}, max_output: {:?}",
                          model, metadata.context_length, metadata.max_output_tokens);
    } else {
        crate::debug_log!("No metadata found for model '{}'", model);
    }
    
    // Create token counter
    crate::debug_log!("Creating token counter for model '{}'", model);
    let token_counter = match TokenCounter::new(model) {
        Ok(counter) => {
            crate::debug_log!("Successfully created token counter for model '{}'", model);
            Some(counter)
        }
        Err(e) => {
            crate::debug_log!("Failed to create token counter for model '{}': {}", model, e);
            eprintln!("Warning: Failed to create token counter for model '{}': {}", model, e);
            None
        }
    };
    
    let mut final_prompt = prompt.to_string();
    let mut final_history = history.to_vec();
    
    // Validate context size if we have both metadata and token counter
    if let (Some(metadata), Some(ref counter)) = (&model_metadata, &token_counter) {
        if let Some(context_limit) = metadata.context_length {
            // Check if input exceeds context limit
            if counter.exceeds_context_limit(prompt, system_prompt, history, context_limit) {
                println!("⚠️  Input exceeds model context limit ({}k tokens). Truncating...", context_limit / 1000);
                
                // Truncate to fit within context limit
                let (truncated_prompt, truncated_history) = counter.truncate_to_fit(
                    prompt,
                    system_prompt,
                    history,
                    context_limit,
                    metadata.max_output_tokens
                );
                
                final_prompt = truncated_prompt;
                final_history = truncated_history;
                
                if final_history.len() < history.len() {
                    println!("📝 Truncated conversation history from {} to {} messages", history.len(), final_history.len());
                }
                
                if final_prompt.len() < prompt.len() {
                    println!("✂️  Truncated prompt from {} to {} characters", prompt.len(), final_prompt.len());
                }
            }
        }
    }
    
    // Build messages for the request
    let mut messages = Vec::new();
    
    // Add system prompt if provided
    if let Some(sys_prompt) = system_prompt {
        messages.push(Message {
            role: "system".to_string(),
            content_type: MessageContent::Text { content: Some(sys_prompt.to_string()) },
            tool_calls: None,
            tool_call_id: None,
        });
    }
    
    // Add conversation history
    for entry in &final_history {
        messages.push(Message::user(entry.question.clone()));
        messages.push(Message::assistant(entry.response.clone()));
    }
    
    // Add current prompt
    messages.push(Message::user(final_prompt));
    
    let request = ChatRequest {
        model: model.to_string(),
        messages: messages.clone(),
        max_tokens: max_tokens.or(Some(1024)),
        temperature: temperature.or(Some(0.7)),
        tools,
        stream: Some(true), // Enable streaming
    };
    
    crate::debug_log!("Sending streaming chat request with {} messages, max_tokens: {:?}, temperature: {:?}",
                      messages.len(), request.max_tokens, request.temperature);
    
    // Send the streaming request
    crate::debug_log!("Making streaming API call to chat endpoint...");
    client.chat_stream(&request, model).await?;
    
    Ok(())
}

async fn get_model_metadata(provider_name: &str, model_name: &str) -> Option<crate::model_metadata::ModelMetadata> {
    use std::fs;
    
    let filename = format!("models/{}.json", provider_name);
    
    if !std::path::Path::new(&filename).exists() {
        return None;
    }
    
    match fs::read_to_string(&filename) {
        Ok(json_content) => {
            match MetadataExtractor::extract_from_provider(provider_name, &json_content) {
                Ok(models) => {
                    models.into_iter().find(|m| m.id == model_name)
                }
                Err(_) => None,
            }
        }
        Err(_) => None,
    }
}

pub async fn get_or_refresh_token(config: &mut Config, provider_name: &str, client: &OpenAIClient) -> Result<String> {
    // If provider is configured for Google SA JWT (Vertex AI), use JWT Bearer flow
    let provider = config.get_provider(provider_name)?.clone();
    let is_vertex = provider.endpoint.to_lowercase().contains("aiplatform.googleapis.com")
        || provider.auth_type.as_deref() == Some("google_sa_jwt");

    // If we have a valid cached token, use it (30s skew)
    if let Some(cached_token) = config.get_cached_token(provider_name) {
        if Utc::now() < cached_token.expires_at {
            return Ok(cached_token.token.clone());
        }
    }

    if is_vertex {
        // Google OAuth 2.0 JWT Bearer flow
        let token_url = provider
            .token_url
            .clone()
            .unwrap_or_else(|| "https://oauth2.googleapis.com/token".to_string());

        // Parse Service Account JSON from api_key
        let api_key_raw = provider.api_key.clone()
            .ok_or_else(|| anyhow::anyhow!("Service Account JSON not set for '{}'. Run lc k a {} and paste SA JSON.", provider_name, provider_name))?;
        #[derive(serde::Deserialize)]
        struct GoogleSA { #[serde(rename="type")] sa_type: String, client_email: String, private_key: String }
        let sa: GoogleSA = serde_json::from_str(&api_key_raw)
            .map_err(|e| anyhow::anyhow!("Invalid Service Account JSON: {}", e))?;
        if sa.sa_type != "service_account" {
            anyhow::bail!("Provided key is not a service_account");
        }

        // Build JWT
        #[derive(serde::Serialize)]
        struct Claims<'a> {
            iss: &'a str,
            scope: &'a str,
            aud: &'a str,
            exp: i64,
            iat: i64,
        }
        let now = Utc::now().timestamp();
        let claims = Claims {
            iss: &sa.client_email,
            scope: "https://www.googleapis.com/auth/cloud-platform",
            aud: &token_url,
            iat: now,
            exp: now + 3600,
        };
        let header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256);
        let key = jsonwebtoken::EncodingKey::from_rsa_pem(sa.private_key.as_bytes())
            .map_err(|e| anyhow::anyhow!("Failed to load RSA key: {}", e))?;
        let assertion = jsonwebtoken::encode(&header, &claims, &key)
            .map_err(|e| anyhow::anyhow!("JWT encode failed: {}", e))?;

        // Exchange for access token
        #[derive(serde::Deserialize)]
        struct GoogleTokenResp {
            access_token: String,
            expires_in: i64,
            #[allow(dead_code)]
            token_type: String,
        }
        let http = reqwest::Client::new();
        let resp = http.post(&token_url)
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
                ("assertion", assertion.as_str()),
            ])
            .send().await
            .map_err(|e| anyhow::anyhow!("Token exchange error: {}", e))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let txt = resp.text().await.unwrap_or_default();
            anyhow::bail!("Token exchange failed ({}): {}", status, txt);
        }
        let token_json: GoogleTokenResp = resp.json().await
            .map_err(|e| anyhow::anyhow!("Failed to parse token response: {}", e))?;
        let expires_at = DateTime::from_timestamp(now + token_json.expires_in - 60, 0)
            .ok_or_else(|| anyhow::anyhow!("Invalid expires timestamp"))?;
        config.set_cached_token(provider_name.to_string(), token_json.access_token.clone(), expires_at)?;
        config.save()?;
        return Ok(token_json.access_token);
    }

    // Fallback: GitHub-style token endpoint using existing client helper
    let token_url = match config.get_token_url(provider_name) {
        Some(url) => url.clone(),
        None => {
            let provider_config = config.get_provider(provider_name)?;
            return provider_config.api_key.clone()
                .ok_or_else(|| anyhow::anyhow!("No API key or token URL configured for provider '{}'", provider_name));
        }
    };

    let token_response = client.get_token_from_url(&token_url).await?;
    let expires_at = DateTime::from_timestamp(token_response.expires_at, 0)
        .ok_or_else(|| anyhow::anyhow!("Invalid expires_at timestamp: {}", token_response.expires_at))?;
    config.set_cached_token(provider_name.to_string(), token_response.token.clone(), expires_at)?;
    config.save()?;
    Ok(token_response.token)
}

// Helper function to detect if a provider is Gemini-based
fn is_gemini_provider(provider_config: &crate::config::ProviderConfig) -> bool {
    provider_config.endpoint.contains("generativelanguage.googleapis.com") ||
    provider_config.chat_path.contains(":generateContent")
}

// Enum to represent different client types
pub enum LLMClient {
    OpenAI(OpenAIClient),
    Gemini(GeminiClient),
}

impl LLMClient {
    pub async fn chat(&self, request: &ChatRequest, model: &str) -> Result<String> {
        match self {
            LLMClient::OpenAI(client) => client.chat(request).await,
            LLMClient::Gemini(client) => {
                // Convert OpenAI format to Gemini format
                let gemini_request = convert_to_gemini_request(request)?;
                client.chat(&gemini_request, model).await
            }
        }
    }
    
    pub async fn chat_with_tools(&self, request: &ChatRequest, model: &str) -> Result<crate::provider::ChatResponse> {
        match self {
            LLMClient::OpenAI(client) => client.chat_with_tools(request).await,
            LLMClient::Gemini(client) => {
                let gemini_request = convert_to_gemini_request(request)?;
                let gemini_response = client.chat_with_tools(&gemini_request, model).await?;
                convert_from_gemini_response(gemini_response)
            }
        }
    }
    
    pub async fn list_models(&self) -> Result<Vec<crate::provider::Model>> {
        match self {
            LLMClient::OpenAI(client) => client.list_models().await,
            LLMClient::Gemini(client) => client.list_models().await,
        }
    }
    
    pub async fn embeddings(&self, request: &crate::provider::EmbeddingRequest) -> Result<crate::provider::EmbeddingResponse> {
        match self {
            LLMClient::OpenAI(client) => client.embeddings(request).await,
            LLMClient::Gemini(_client) => {
                // Gemini doesn't support embeddings through the same API
                // For now, return an error - this could be extended later
                anyhow::bail!("Embeddings not supported for Gemini provider. Use a dedicated embedding model.")
            }
        }
    }
    
    pub async fn generate_images(&self, request: &crate::provider::ImageGenerationRequest) -> Result<crate::provider::ImageGenerationResponse> {
        match self {
            LLMClient::OpenAI(client) => client.generate_images(request).await,
            LLMClient::Gemini(_client) => {
                // Gemini doesn't support image generation through the same API
                // For now, return an error - this could be extended later
                anyhow::bail!("Image generation not supported for Gemini provider. Use a dedicated image generation model.")
            }
        }
    }
    
    pub async fn chat_stream(&self, request: &ChatRequest, model: &str) -> Result<()> {
        match self {
            LLMClient::OpenAI(client) => client.chat_stream(request).await,
            LLMClient::Gemini(client) => {
                // Convert OpenAI format to Gemini format
                let gemini_request = convert_to_gemini_request(request)?;
                client.chat_stream(&gemini_request, model).await
            }
        }
    }
}

// Convert OpenAI ChatRequest to Gemini format
fn convert_to_gemini_request(request: &ChatRequest) -> Result<GeminiChatRequest> {
    let mut contents = Vec::new();
    let mut system_instruction = None;
    
    for message in &request.messages {
        match message.role.as_str() {
            "system" => {
                if let Some(content) = message.get_text_content() {
                    system_instruction = Some(GeminiSystemInstruction {
                        parts: GeminiPart::Text { text: content.clone() },
                    });
                }
            }
            "user" => {
                match &message.content_type {
                    MessageContent::Text { content } => {
                        if let Some(text) = content {
                            contents.push(GeminiContent {
                                role: "user".to_string(),
                                parts: vec![GeminiPart::Text { text: text.clone() }],
                            });
                        }
                    }
                    MessageContent::Multimodal { content } => {
                        let mut parts = Vec::new();
                        for part in content {
                            match part {
                                ContentPart::Text { text } => {
                                    parts.push(GeminiPart::Text { text: text.clone() });
                                }
                                ContentPart::ImageUrl { image_url } => {
                                    // Convert image URL to Gemini inline data format
                                    if image_url.url.starts_with("data:") {
                                        // Extract mime type and base64 data from data URL
                                        if let Some(comma_pos) = image_url.url.find(',') {
                                            let header = &image_url.url[5..comma_pos]; // Skip "data:"
                                            let data = &image_url.url[comma_pos + 1..];
                                            
                                            let mime_type = if let Some(semi_pos) = header.find(';') {
                                                header[..semi_pos].to_string()
                                            } else {
                                                header.to_string()
                                            };
                                            
                                            parts.push(GeminiPart::InlineData {
                                                inline_data: GeminiInlineData {
                                                    mime_type,
                                                    data: data.to_string(),
                                                },
                                            });
                                        }
                                    }
                                }
                            }
                        }
                        if !parts.is_empty() {
                            contents.push(GeminiContent {
                                role: "user".to_string(),
                                parts,
                            });
                        }
                    }
                }
            }
            "assistant" => {
                let mut parts = Vec::new();
                
                if let Some(content) = message.get_text_content() {
                    parts.push(GeminiPart::Text { text: content.clone() });
                }
                
                if let Some(tool_calls) = &message.tool_calls {
                    for tool_call in tool_calls {
                        let args: serde_json::Value = serde_json::from_str(&tool_call.function.arguments)?;
                        parts.push(GeminiPart::FunctionCall {
                            function_call: crate::provider::GeminiFunctionCall {
                                name: tool_call.function.name.clone(),
                                args,
                            },
                        });
                    }
                }
                
                if !parts.is_empty() {
                    contents.push(GeminiContent {
                        role: "model".to_string(),
                        parts,
                    });
                }
            }
            "tool" => {
                if let (Some(content), Some(tool_call_id)) = (message.get_text_content(), &message.tool_call_id) {
                    // For Gemini, we need to find the function name from the tool_call_id
                    // This is a simplified approach - in practice, you might need to track this
                    let response_value: serde_json::Value = serde_json::from_str(content)
                        .unwrap_or_else(|_| serde_json::json!({"result": content}));
                    
                    contents.push(GeminiContent {
                        role: "user".to_string(),
                        parts: vec![GeminiPart::FunctionResponse {
                            function_response: crate::provider::GeminiFunctionResponse {
                                name: tool_call_id.clone(), // Simplified - should be function name
                                response: response_value,
                            },
                        }],
                    });
                }
            }
            _ => {} // Ignore unknown roles
        }
    }
    
    // Convert tools if present
    let gemini_tools = if let Some(tools) = &request.tools {
        let function_declarations: Vec<GeminiFunctionDeclaration> = tools.iter().map(|tool| {
            GeminiFunctionDeclaration {
                name: tool.function.name.clone(),
                description: tool.function.description.clone(),
                parameters: tool.function.parameters.clone(),
            }
        }).collect();
        
        if !function_declarations.is_empty() {
            Some(vec![GeminiTool { function_declarations }])
        } else {
            None
        }
    } else {
        None
    };
    
    let generation_config = if request.temperature.is_some() || request.max_tokens.is_some() {
        Some(GeminiGenerationConfig {
            temperature: request.temperature,
            max_output_tokens: request.max_tokens,
        })
    } else {
        None
    };
    
    Ok(GeminiChatRequest {
        system_instruction,
        contents,
        tools: gemini_tools,
        generation_config,
    })
}

// Convert Gemini response to OpenAI format
fn convert_from_gemini_response(gemini_response: GeminiChatResponse) -> Result<crate::provider::ChatResponse> {
    let mut choices = Vec::new();
    
    for candidate in gemini_response.candidates {
        let mut content = None;
        let mut tool_calls = Vec::new();
        
        for part in candidate.content.parts {
            match part {
                GeminiResponsePart::Text { text } => {
                    content = Some(text);
                }
                GeminiResponsePart::FunctionCall { function_call } => {
                    tool_calls.push(crate::provider::ToolCall {
                        id: format!("call_{}", uuid::Uuid::new_v4().to_string()[..8].to_string()),
                        call_type: "function".to_string(),
                        function: crate::provider::FunctionCall {
                            name: function_call.name,
                            arguments: serde_json::to_string(&function_call.args)?,
                        },
                    });
                }
            }
        }
        
        let choice = crate::provider::Choice {
            message: crate::provider::ResponseMessage {
                role: candidate.content.role,
                content,
                tool_calls: if tool_calls.is_empty() { None } else { Some(tool_calls) },
            },
        };
        
        choices.push(choice);
    }
    
    Ok(crate::provider::ChatResponse { choices })
}

pub async fn create_authenticated_client(config: &mut Config, provider_name: &str) -> Result<LLMClient> {
    crate::debug_log!("Creating authenticated client for provider '{}'", provider_name);
    let provider_config = config.get_provider(provider_name)?.clone();

    crate::debug_log!(
        "Provider '{}' config - endpoint: {}, models_path: {}, chat_path: {}",
        provider_name,
        provider_config.endpoint,
        provider_config.models_path,
        provider_config.chat_path
    );

    // Normalize chat_path placeholders: support both {model} and legacy {model_name}
    let normalized_chat_path = provider_config
        .chat_path
        .replace("{model_name}", "{model}");
    let provider_config = crate::config::ProviderConfig {
        chat_path: normalized_chat_path,
        ..provider_config
    };

    // Check if this is a Vertex AI provider that should use Gemini format
    // Vertex AI Llama endpoints (ending with /chat/completions) use OpenAI format
    let is_vertex_ai_gemini = (provider_config.endpoint.contains("aiplatform.googleapis.com") ||
                              provider_config.auth_type.as_deref() == Some("google_sa_jwt")) &&
                              !provider_config.chat_path.ends_with("/chat/completions");

    // Gemini (generativelanguage) still uses API key header semantics
    // Vertex AI Gemini endpoints use Gemini format but with OAuth tokens
    // Vertex AI Llama endpoints (/chat/completions) use OpenAI format
    if is_gemini_provider(&provider_config) || is_vertex_ai_gemini {
        crate::debug_log!("Detected Gemini/Vertex AI provider, creating GeminiClient");
        
        if is_vertex_ai_gemini {
            // For Vertex AI, we need to get an OAuth token
            let temp_client = OpenAIClient::new_with_headers(
                provider_config.endpoint.clone(),
                provider_config.api_key.clone().unwrap_or_default(),
                provider_config.models_path.clone(),
                provider_config.chat_path.clone(),
                provider_config.headers.clone(),
            );
            
            let auth_token = get_or_refresh_token(config, provider_name, &temp_client).await?;
            
            // Create custom headers with Authorization for Vertex AI
            let mut vertex_headers = provider_config.headers.clone();
            vertex_headers.insert("Authorization".to_string(), format!("Bearer {}", auth_token));
            
            let client = GeminiClient::new_with_provider_config(
                provider_config.endpoint.clone(),
                auth_token, // Pass token as api_key for compatibility
                provider_config.models_path.clone(),
                provider_config.chat_path.clone(),
                vertex_headers,
                provider_config.clone(),
            );
            return Ok(LLMClient::Gemini(client));
        } else {
            // Regular Gemini with API key
            let api_key = provider_config.api_key.clone()
                .ok_or_else(|| anyhow::anyhow!("No API key configured for Gemini provider '{}'", provider_name))?;
            let client = GeminiClient::new_with_provider_config(
                provider_config.endpoint.clone(),
                api_key,
                provider_config.models_path.clone(),
                provider_config.chat_path.clone(),
                provider_config.headers.clone(),
                provider_config.clone(),
            );
            return Ok(LLMClient::Gemini(client));
        }
    }

    // OpenAI-compatible flow
    // Check if this is a Vertex AI Llama endpoint (needs OAuth but OpenAI format)
    let is_vertex_ai_llama = (provider_config.endpoint.contains("aiplatform.googleapis.com") ||
                             provider_config.auth_type.as_deref() == Some("google_sa_jwt")) &&
                             provider_config.chat_path.ends_with("/chat/completions");

    if is_vertex_ai_llama {
        // Vertex AI Llama: OAuth authentication with OpenAI format
        let temp_client = OpenAIClient::new_with_headers(
            provider_config.endpoint.clone(),
            provider_config.api_key.clone().unwrap_or_default(),
            provider_config.models_path.clone(),
            provider_config.chat_path.clone(),
            provider_config.headers.clone(),
        );
        
        let auth_token = get_or_refresh_token(config, provider_name, &temp_client).await?;
        
        // Create custom headers with Authorization for Vertex AI Llama
        let mut vertex_headers = provider_config.headers.clone();
        vertex_headers.insert("Authorization".to_string(), format!("Bearer {}", auth_token));
        
        let client = OpenAIClient::new_with_provider_config(
            provider_config.endpoint.clone(),
            auth_token,
            provider_config.models_path.clone(),
            provider_config.chat_path.clone(),
            vertex_headers,
            provider_config.clone(),
        );
        
        return Ok(LLMClient::OpenAI(client));
    }

    // Regular OpenAI-compatible flow
    // Build temp client for token retrieval fallbacks that still use simple GET token_url
    let temp_client = OpenAIClient::new_with_headers(
        provider_config.endpoint.clone(),
        provider_config.api_key.clone().unwrap_or_default(),
        provider_config.models_path.clone(),
        provider_config.chat_path.clone(),
        provider_config.headers.clone(),
    );

    let auth_token = get_or_refresh_token(config, provider_name, &temp_client).await?;

    let client = OpenAIClient::new_with_provider_config(
        provider_config.endpoint.clone(),
        auth_token,
        provider_config.models_path.clone(),
        provider_config.chat_path.clone(),
        provider_config.headers.clone(),
        provider_config.clone(),
    );

    Ok(LLMClient::OpenAI(client))
}

// New function to handle tool execution loop
pub async fn send_chat_request_with_tool_execution(
    client: &LLMClient,
    model: &str,
    prompt: &str,
    history: &[ChatEntry],
    system_prompt: Option<&str>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
    _provider_name: &str,
    tools: Option<Vec<crate::provider::Tool>>,
    mcp_server_names: &[&str],
) -> Result<(String, Option<i32>, Option<i32>)> {
    use crate::provider::{Message, ChatRequest};
    use crate::token_utils::TokenCounter;
    
    let mut conversation_messages = Vec::new();
    let mut total_input_tokens = 0i32;
    let mut total_output_tokens = 0i32;
    
    // Create token counter for tracking usage
    let token_counter = TokenCounter::new(model).ok();
    
    // Add system prompt if provided
    if let Some(sys_prompt) = system_prompt {
        conversation_messages.push(Message {
            role: "system".to_string(),
            content_type: MessageContent::Text { content: Some(sys_prompt.to_string()) },
            tool_calls: None,
            tool_call_id: None,
        });
    }
    
    // Add conversation history
    for entry in history {
        conversation_messages.push(Message::user(entry.question.clone()));
        conversation_messages.push(Message::assistant(entry.response.clone()));
    }
    
    // Add current prompt
    conversation_messages.push(Message::user(prompt.to_string()));
    let max_iterations = 10; // Prevent infinite loops
    let mut iteration = 0;
    
    loop {
        iteration += 1;
        if iteration > max_iterations {
            anyhow::bail!("Maximum tool execution iterations reached ({})", max_iterations);
        }
        
        crate::debug_log!("Tool execution iteration {}/{}", iteration, max_iterations);
        
        let request = ChatRequest {
            model: model.to_string(),
            messages: conversation_messages.clone(),
            max_tokens: max_tokens.or(Some(1024)),
            temperature: temperature.or(Some(0.7)),
            tools: tools.clone(),
            stream: None, // Non-streaming request for tool execution
        };
        
        // Make the API call
        let response = client.chat_with_tools(&request, model).await?;
        
        // Track token usage if we have a counter
        if let Some(ref counter) = token_counter {
            let input_tokens = counter.estimate_chat_tokens("", system_prompt, &[]) as i32;
            total_input_tokens += input_tokens;
        }
        
        if let Some(choice) = response.choices.first() {
            crate::debug_log!("Response choice - tool_calls: {}, content: {}",
                             choice.message.tool_calls.as_ref().map_or(0, |tc| tc.len()),
                             choice.message.content.as_ref().map_or("None", |c|
                                 if c.len() > 50 { &c[..50] } else { c }));
            
            // Check if the LLM made tool calls
            if let Some(tool_calls) = &choice.message.tool_calls {
                if !tool_calls.is_empty() {
                    crate::debug_log!("LLM made {} tool calls in iteration {}", tool_calls.len(), iteration);
                    
                    // Add the assistant's tool call message to conversation
                    conversation_messages.push(Message::assistant_with_tool_calls(tool_calls.clone()));
                
                    // Execute each tool call
                    for (i, tool_call) in tool_calls.iter().enumerate() {
                        crate::debug_log!("Executing tool call {}/{}: {} with args: {}",
                                         i + 1, tool_calls.len(), tool_call.function.name, tool_call.function.arguments);
                        
                        // Find which MCP server has this function using daemon client
                        let daemon_client = crate::mcp_daemon::DaemonClient::new()?;
                        let mut tool_result = None;
                        for server_name in mcp_server_names {
                            // Parse arguments as JSON value instead of Vec<String>
                            let args_value: serde_json::Value = serde_json::from_str(&tool_call.function.arguments)?;
                            match daemon_client.call_tool(
                                server_name,
                                &tool_call.function.name,
                                args_value
                            ).await {
                                Ok(result) => {
                                    crate::debug_log!("Tool call successful on server '{}': {}", server_name,
                                                     serde_json::to_string(&result).unwrap_or_else(|_| "invalid json".to_string()));
                                    tool_result = Some(format_tool_result(&result));
                                    break;
                                }
                                Err(e) => {
                                    crate::debug_log!("Tool call failed on server '{}': {}", server_name, e);
                                    continue;
                                }
                            }
                        }
                        
                        let result_content = tool_result.unwrap_or_else(|| {
                            format!("Error: Function '{}' not found on any MCP server", tool_call.function.name)
                        });
                        
                        crate::debug_log!("Tool result for {}: {}", tool_call.function.name,
                                         if result_content.len() > 100 {
                                             format!("{}...", &result_content[..100])
                                         } else {
                                             result_content.clone()
                                         });
                        
                        // Add tool result to conversation
                        conversation_messages.push(Message::tool_result(
                            tool_call.id.clone(),
                            result_content
                        ));
                    }
                    
                    // Continue the loop to get the LLM's response to the tool results
                    continue;
                } else {
                    // Empty tool_calls array - check if we have content (final answer)
                    if let Some(content) = &choice.message.content {
                        if !content.trim().is_empty() {
                            crate::debug_log!("LLM provided final answer with empty tool_calls after {} iterations: {}",
                                             iteration, if content.len() > 100 {
                                                 format!("{}...", &content[..100])
                                             } else {
                                                 content.clone()
                                             });
                            
                            // Track output tokens
                            if let Some(ref counter) = token_counter {
                                total_output_tokens += counter.count_tokens(content) as i32;
                            }
                            
                            // Exit immediately when LLM provides content (final answer)
                            return Ok((content.clone(), Some(total_input_tokens), Some(total_output_tokens)));
                        }
                    }
                }
            } else if let Some(content) = &choice.message.content {
                // LLM provided a final answer without tool calls field
                crate::debug_log!("LLM provided final answer without tool_calls field after {} iterations: {}",
                                 iteration, if content.len() > 100 {
                                     format!("{}...", &content[..100])
                                 } else {
                                     content.clone()
                                 });
                
                // Track output tokens
                if let Some(ref counter) = token_counter {
                    total_output_tokens += counter.count_tokens(content) as i32;
                }
                
                // Exit immediately when LLM provides content (final answer)
                return Ok((content.clone(), Some(total_input_tokens), Some(total_output_tokens)));
            } else {
                // LLM provided neither tool calls nor content - this shouldn't happen
                crate::debug_log!("LLM provided neither tool calls nor content in iteration {}", iteration);
                anyhow::bail!("No content or tool calls in response from LLM in iteration {}", iteration);
            }
        } else {
            anyhow::bail!("No response from API");
        }
    }
}


// Helper function to format tool result for display
fn format_tool_result(result: &serde_json::Value) -> String {
    if let Some(content_array) = result.get("content") {
        if let Some(content_items) = content_array.as_array() {
            let mut formatted = String::new();
            for item in content_items {
                if let Some(text) = item.get("text") {
                    if let Some(text_str) = text.as_str() {
                        formatted.push_str(text_str);
                        formatted.push('\n');
                    }
                }
            }
            return formatted.trim().to_string();
        }
    }
    
    // Fallback to pretty-printed JSON
    serde_json::to_string_pretty(result).unwrap_or_else(|_| "Error formatting result".to_string())
}

// Message-based versions of the chat functions for handling multimodal content

pub async fn send_chat_request_with_validation_messages(
    client: &LLMClient,
    model: &str,
    messages: &[Message],
    system_prompt: Option<&str>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
    provider_name: &str,
    tools: Option<Vec<crate::provider::Tool>>,
) -> Result<(String, Option<i32>, Option<i32>)> {
    crate::debug_log!("Sending chat request with messages - provider: '{}', model: '{}', messages: {}",
                      provider_name, model, messages.len());
    
    // Build final messages including system prompt if needed
    let mut final_messages = Vec::new();
    
    // Add system prompt if provided and not already in messages
    if let Some(sys_prompt) = system_prompt {
        let has_system = messages.iter().any(|m| m.role == "system");
        if !has_system {
            final_messages.push(Message {
                role: "system".to_string(),
                content_type: MessageContent::Text { content: Some(sys_prompt.to_string()) },
                tool_calls: None,
                tool_call_id: None,
            });
        }
    }
    
    // Add all provided messages
    final_messages.extend_from_slice(messages);
    
    let request = ChatRequest {
        model: model.to_string(),
        messages: final_messages,
        max_tokens: max_tokens.or(Some(1024)),
        temperature: temperature.or(Some(0.7)),
        tools,
        stream: None,
    };
    
    let response = client.chat(&request, model).await?;
    
    // For now, return None for token counts as we'd need to implement multimodal token counting
    Ok((response, None, None))
}

pub async fn send_chat_request_with_streaming_messages(
    client: &LLMClient,
    model: &str,
    messages: &[Message],
    system_prompt: Option<&str>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
    provider_name: &str,
    tools: Option<Vec<crate::provider::Tool>>,
) -> Result<()> {
    crate::debug_log!("Sending streaming chat request with messages - provider: '{}', model: '{}', messages: {}",
                      provider_name, model, messages.len());
    
    // Build final messages including system prompt if needed
    let mut final_messages = Vec::new();
    
    // Add system prompt if provided and not already in messages
    if let Some(sys_prompt) = system_prompt {
        let has_system = messages.iter().any(|m| m.role == "system");
        if !has_system {
            final_messages.push(Message {
                role: "system".to_string(),
                content_type: MessageContent::Text { content: Some(sys_prompt.to_string()) },
                tool_calls: None,
                tool_call_id: None,
            });
        }
    }
    
    // Add all provided messages
    final_messages.extend_from_slice(messages);
    
    let request = ChatRequest {
        model: model.to_string(),
        messages: final_messages,
        max_tokens: max_tokens.or(Some(1024)),
        temperature: temperature.or(Some(0.7)),
        tools,
        stream: Some(true),
    };
    
    client.chat_stream(&request, model).await?;
    
    Ok(())
}

pub async fn send_chat_request_with_tool_execution_messages(
    client: &LLMClient,
    model: &str,
    messages: &[Message],
    system_prompt: Option<&str>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
    provider_name: &str,
    tools: Option<Vec<crate::provider::Tool>>,
    mcp_server_names: &[&str],
) -> Result<(String, Option<i32>, Option<i32>)> {
    crate::debug_log!("Sending chat request with tool execution and messages - provider: '{}', model: '{}', messages: {}",
                      provider_name, model, messages.len());
    
    let mut conversation_messages = Vec::new();
    
    // Add system prompt if provided and not already in messages
    if let Some(sys_prompt) = system_prompt {
        let has_system = messages.iter().any(|m| m.role == "system");
        if !has_system {
            conversation_messages.push(Message {
                role: "system".to_string(),
                content_type: MessageContent::Text { content: Some(sys_prompt.to_string()) },
                tool_calls: None,
                tool_call_id: None,
            });
        }
    }
    
    // Add all provided messages
    conversation_messages.extend_from_slice(messages);
    
    let max_iterations = 10;
    let mut iteration = 0;
    
    loop {
        iteration += 1;
        if iteration > max_iterations {
            anyhow::bail!("Maximum tool execution iterations reached ({})", max_iterations);
        }
        
        let request = ChatRequest {
            model: model.to_string(),
            messages: conversation_messages.clone(),
            max_tokens: max_tokens.or(Some(1024)),
            temperature: temperature.or(Some(0.7)),
            tools: tools.clone(),
            stream: None,
        };
        
        let response = client.chat_with_tools(&request, model).await?;
        
        if let Some(choice) = response.choices.first() {
            if let Some(tool_calls) = &choice.message.tool_calls {
                if !tool_calls.is_empty() {
                    conversation_messages.push(Message::assistant_with_tool_calls(tool_calls.clone()));
                    
                    for tool_call in tool_calls {
                        let daemon_client = crate::mcp_daemon::DaemonClient::new()?;
                        let mut tool_result = None;
                        
                        for server_name in mcp_server_names {
                            let args_value: serde_json::Value = serde_json::from_str(&tool_call.function.arguments)?;
                            match daemon_client.call_tool(server_name, &tool_call.function.name, args_value).await {
                                Ok(result) => {
                                    tool_result = Some(format_tool_result(&result));
                                    break;
                                }
                                Err(_) => continue,
                            }
                        }
                        
                        let result_content = tool_result.unwrap_or_else(|| {
                            format!("Error: Function '{}' not found on any MCP server", tool_call.function.name)
                        });
                        
                        conversation_messages.push(Message::tool_result(
                            tool_call.id.clone(),
                            result_content
                        ));
                    }
                    
                    continue;
                }
            }
            
            if let Some(content) = &choice.message.content {
                return Ok((content.clone(), None, None));
            }
        }
        
        anyhow::bail!("No response from API");
    }
}