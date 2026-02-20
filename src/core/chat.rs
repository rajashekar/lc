use crate::config::Config;
use crate::database::ChatEntry;
use crate::model_metadata::MetadataExtractor;
use crate::provider::{ChatRequest, Message, MessageContent, OpenAIClient};
use crate::token_utils::TokenCounter;
use anyhow::Result;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

// Agent execution constants
const DEFAULT_MAX_ITERATIONS: u32 = 10;
const TOOL_EXECUTION_TIMEOUT_SECS: u64 = 30;
const MAX_TOOL_RESULT_LENGTH: usize = 10000;
const IMAGE_TOKEN_ESTIMATE: i32 = 85; // Approximate tokens for low-detail image

#[allow(clippy::too_many_arguments)]
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
    crate::debug_log!(
        "Request parameters - max_tokens: {:?}, temperature: {:?}",
        max_tokens,
        temperature
    );

    // Try to get model metadata for context validation
    crate::debug_log!(
        "Loading model metadata for provider '{}', model '{}'",
        provider_name,
        model
    );
    let model_metadata = get_model_metadata(provider_name, model).await;

    if let Some(ref metadata) = model_metadata {
        crate::debug_log!(
            "Found metadata for model '{}' - context_length: {:?}, max_output: {:?}",
            model,
            metadata.context_length,
            metadata.max_output_tokens
        );
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
            crate::debug_log!(
                "Failed to create token counter for model '{}': {}",
                model,
                e
            );
            eprintln!(
                "Warning: Failed to create token counter for model '{}': {}",
                model, e
            );
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
                println!(
                    "⚠️  Input exceeds model context limit ({}k tokens). Truncating...",
                    context_limit / 1000
                );

                // Truncate to fit within context limit
                let (truncated_prompt, truncated_history) = counter.truncate_to_fit(
                    prompt,
                    system_prompt,
                    history,
                    context_limit,
                    metadata.max_output_tokens,
                );

                final_prompt = truncated_prompt;
                final_history = truncated_history;

                if final_history.len() < history.len() {
                    println!(
                        "📝 Truncated conversation history from {} to {} messages",
                        history.len(),
                        final_history.len()
                    );
                }

                if final_prompt.len() < prompt.len() {
                    println!(
                        "✂️  Truncated prompt from {} to {} characters",
                        prompt.len(),
                        final_prompt.len()
                    );
                }
            }

            // Calculate input tokens after potential truncation
            input_tokens = Some(counter.estimate_chat_tokens(
                &final_prompt,
                system_prompt,
                &final_history,
            ) as i32);
        }
    } else if let Some(ref counter) = token_counter {
        // No metadata available, but we can still count tokens
        input_tokens =
            Some(counter.estimate_chat_tokens(&final_prompt, system_prompt, &final_history) as i32);
    }

    // Build messages for the request
    let mut messages = Vec::new();

    // Add system prompt if provided
    if let Some(sys_prompt) = system_prompt {
        messages.push(Message {
            role: "system".to_string(),
            content_type: MessageContent::Text {
                content: Some(sys_prompt.to_string()),
            },
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

    crate::debug_log!(
        "Sending chat request with {} messages, max_tokens: {:?}, temperature: {:?}",
        messages.len(),
        request.max_tokens,
        request.temperature
    );

    // Send the request
    crate::debug_log!("Making API call to chat endpoint...");
    let response = client.chat(&request).await?;

    crate::debug_log!(
        "Received response from chat API ({} characters)",
        response.len()
    );

    // Calculate output tokens if we have a token counter
    let output_tokens = token_counter
        .as_ref()
        .map(|counter| counter.count_tokens(&response) as i32);

    // Display token usage if available
    if let (Some(input), Some(output)) = (input_tokens, output_tokens) {
        println!(
            "📊 Token usage: {} input + {} output = {} total",
            input,
            output,
            input + output
        );

        // Show cost estimate if we have pricing info
        if let Some(metadata) = &model_metadata {
            if let (Some(input_price), Some(output_price)) =
                (metadata.input_price_per_m, metadata.output_price_per_m)
            {
                let input_cost = (input as f64 / 1_000_000.0) * input_price;
                let output_cost = (output as f64 / 1_000_000.0) * output_price;
                let total_cost = input_cost + output_cost;
                println!(
                    "💰 Estimated cost: ${:.6} (${:.6} input + ${:.6} output)",
                    total_cost, input_cost, output_cost
                );
            }
        }
    }

    Ok((response, input_tokens, output_tokens))
}

#[allow(clippy::too_many_arguments)]
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
    on_connect: Option<Box<dyn FnMut() + Send>>,
) -> Result<()> {
    crate::debug_log!("Sending streaming chat request - provider: '{}', model: '{}', prompt length: {}, history entries: {}",
                      provider_name, model, prompt.len(), history.len());
    crate::debug_log!(
        "Request parameters - max_tokens: {:?}, temperature: {:?}",
        max_tokens,
        temperature
    );

    // Try to get model metadata for context validation
    crate::debug_log!(
        "Loading model metadata for provider '{}', model '{}'",
        provider_name,
        model
    );
    let model_metadata = get_model_metadata(provider_name, model).await;

    if let Some(ref metadata) = model_metadata {
        crate::debug_log!(
            "Found metadata for model '{}' - context_length: {:?}, max_output: {:?}",
            model,
            metadata.context_length,
            metadata.max_output_tokens
        );
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
            crate::debug_log!(
                "Failed to create token counter for model '{}': {}",
                model,
                e
            );
            eprintln!(
                "Warning: Failed to create token counter for model '{}': {}",
                model, e
            );
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
                println!(
                    "⚠️  Input exceeds model context limit ({}k tokens). Truncating...",
                    context_limit / 1000
                );

                // Truncate to fit within context limit
                let (truncated_prompt, truncated_history) = counter.truncate_to_fit(
                    prompt,
                    system_prompt,
                    history,
                    context_limit,
                    metadata.max_output_tokens,
                );

                final_prompt = truncated_prompt;
                final_history = truncated_history;

                if final_history.len() < history.len() {
                    println!(
                        "📝 Truncated conversation history from {} to {} messages",
                        history.len(),
                        final_history.len()
                    );
                }

                if final_prompt.len() < prompt.len() {
                    println!(
                        "✂️  Truncated prompt from {} to {} characters",
                        prompt.len(),
                        final_prompt.len()
                    );
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
            content_type: MessageContent::Text {
                content: Some(sys_prompt.to_string()),
            },
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

    crate::debug_log!(
        "Sending streaming chat request with {} messages, max_tokens: {:?}, temperature: {:?}",
        messages.len(),
        request.max_tokens,
        request.temperature
    );

    // Send the streaming request
    crate::debug_log!("Making streaming API call to chat endpoint...");
    client.chat_stream(&request, on_connect).await?;

    Ok(())
}

// Cache for provider model metadata to avoid repeated file reads and parsing
static PROVIDER_METADATA_CACHE: OnceLock<
    RwLock<HashMap<String, Vec<crate::model_metadata::ModelMetadata>>>,
> = OnceLock::new();

async fn get_model_metadata(
    provider_name: &str,
    model_name: &str,
) -> Option<crate::model_metadata::ModelMetadata> {
    // Initialize cache if needed
    let cache = PROVIDER_METADATA_CACHE.get_or_init(|| RwLock::new(HashMap::new()));

    // Check cache first (read lock)
    {
        if let Ok(guard) = cache.read() {
            if let Some(models) = guard.get(provider_name) {
                return models.iter().find(|m| m.id == model_name).cloned();
            }
        }
    }

    // Not in cache, load from file
    let filename = format!("models/{}.json", provider_name);

    if !std::path::Path::new(&filename).exists() {
        return None;
    }

    match tokio::fs::read_to_string(&filename).await {
        Ok(json_content) => {
            match MetadataExtractor::extract_from_provider(provider_name, &json_content) {
                Ok(models) => {
                    // Update cache (write lock)
                    if let Ok(mut guard) = cache.write() {
                        guard.insert(provider_name.to_string(), models.clone());
                    }

                    models.into_iter().find(|m| m.id == model_name)
                }
                Err(_) => None,
            }
        }
        Err(_) => None,
    }
}

pub async fn get_or_refresh_token(
    config: &mut Config,
    provider_name: &str,
    client: &OpenAIClient,
) -> Result<String> {
    // If provider is configured for Google SA JWT (Vertex AI), use JWT Bearer flow
    let provider = config.get_provider_with_auth(provider_name)?.clone();
    let is_vertex = provider
        .endpoint
        .to_lowercase()
        .contains("aiplatform.googleapis.com")
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
        let api_key_raw = provider.api_key.clone().ok_or_else(|| {
            anyhow::anyhow!(
                "Service Account JSON not set for '{}'. Run lc k a {} and paste SA JSON.",
                provider_name,
                provider_name
            )
        })?;
        #[derive(serde::Deserialize)]
        struct GoogleSA {
            #[serde(rename = "type")]
            sa_type: String,
            client_email: String,
            private_key: String,
        }
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
        let resp = http
            .post(&token_url)
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
                ("assertion", assertion.as_str()),
            ])
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Token exchange error: {}", e))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let txt = resp.text().await.unwrap_or_default();
            anyhow::bail!("Token exchange failed ({}): {}", status, txt);
        }
        let token_json: GoogleTokenResp = resp
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to parse token response: {}", e))?;
        let expires_at = DateTime::from_timestamp(now + token_json.expires_in - 60, 0)
            .ok_or_else(|| anyhow::anyhow!("Invalid expires timestamp"))?;
        config.set_cached_token(
            provider_name.to_string(),
            token_json.access_token.clone(),
            expires_at,
        )?;
        config.save()?;
        return Ok(token_json.access_token);
    }

    // Fallback: GitHub-style token endpoint using existing client helper
    let token_url = match config.get_token_url(provider_name) {
        Some(url) => url.clone(),
        None => {
            let provider_config = config.get_provider_with_auth(provider_name)?;
            return provider_config.api_key.clone().ok_or_else(|| {
                anyhow::anyhow!(
                    "No API key or token URL configured for provider '{}'",
                    provider_name
                )
            });
        }
    };

    let token_response = client.get_token_from_url(&token_url).await?;
    let expires_at = DateTime::from_timestamp(token_response.expires_at, 0).ok_or_else(|| {
        anyhow::anyhow!(
            "Invalid expires_at timestamp: {}",
            token_response.expires_at
        )
    })?;
    config.set_cached_token(
        provider_name.to_string(),
        token_response.token.clone(),
        expires_at,
    )?;
    config.save()?;
    Ok(token_response.token)
}

// All providers now use OpenAIClient with template-based transformations
pub type LLMClient = OpenAIClient;

// Hardcoded conversion functions removed - now using template-based transformations

pub async fn create_authenticated_client(
    config: &mut Config,
    provider_name: &str,
) -> Result<LLMClient> {
    crate::debug_log!(
        "Creating authenticated client for provider '{}'",
        provider_name
    );

    // Get provider config with authentication from centralized keys
    let mut provider_config = config.get_provider_with_auth(provider_name)?;

    crate::debug_log!(
        "Provider '{}' config - endpoint: {}, models_path: {}, chat_path: {}",
        provider_name,
        provider_config.endpoint,
        provider_config.models_path,
        provider_config.chat_path
    );

    // Normalize chat_path placeholders: support both {model} and legacy {model_name}
    let normalized_chat_path = provider_config.chat_path.replace("{model_name}", "{model}");
    provider_config.chat_path = normalized_chat_path;

    // All providers now use OpenAIClient with template-based transformations
    // Check if this needs OAuth authentication (Vertex AI)
    let needs_oauth = provider_config
        .endpoint
        .contains("aiplatform.googleapis.com")
        || provider_config.auth_type.as_deref() == Some("google_sa_jwt");

    if needs_oauth {
        // OAuth authentication flow (Vertex AI)
        let temp_client = OpenAIClient::new_with_headers(
            provider_config.endpoint.clone(),
            provider_config.api_key.clone().unwrap_or_default(),
            provider_config.models_path.clone(),
            provider_config.chat_path.clone(),
            provider_config.headers.clone(),
        );

        let auth_token = get_or_refresh_token(config, provider_name, &temp_client).await?;

        // Create custom headers with Authorization
        let mut oauth_headers = provider_config.headers.clone();
        oauth_headers.insert(
            "Authorization".to_string(),
            format!("Bearer {}", auth_token),
        );

        let client = OpenAIClient::new_with_provider_config(
            provider_config.endpoint.clone(),
            auth_token,
            provider_config.models_path.clone(),
            provider_config.chat_path.clone(),
            oauth_headers,
            provider_config.clone(),
        );

        return Ok(client);
    }

    // Regular authentication flow (API key or token URL)
    // Special-case: if headers already contain resolved auth (e.g., x-goog-api-key), we don't need a token
    let header_has_resolved_key = provider_config.headers.iter().any(|(k, v)| {
        let k_l = k.to_lowercase();
        // Heuristic: header name suggests auth AND value is not a placeholder and not empty
        (k_l.contains("key") || k_l.contains("token") || k_l.contains("auth"))
            && !v.trim().is_empty()
            && !v.contains("${api_key}")
    });

    if provider_config.api_key.is_none() && header_has_resolved_key {
        // Header-based auth present (e.g., Gemini x-goog-api-key). No token retrieval needed.
        // Pass empty api_key since Authorization won't be used when custom headers exist.
        let client = OpenAIClient::new_with_provider_config(
            provider_config.endpoint.clone(),
            String::new(),
            provider_config.models_path.clone(),
            provider_config.chat_path.clone(),
            provider_config.headers.clone(),
            provider_config.clone(),
        );
        return Ok(client);
    }

    // Fallback: API key in Authorization or token URL-based auth
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

    Ok(client)
}

// New function to handle tool execution loop
#[allow(clippy::too_many_arguments)]
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
    max_iterations: Option<u32>,
) -> Result<(String, Option<i32>, Option<i32>)> {
    use crate::provider::{ChatRequest, Message};
    use crate::token_utils::TokenCounter;

    let mut conversation_messages = Vec::new();
    let mut total_input_tokens = 0i32;
    let mut total_output_tokens = 0i32;

    // Create token counter for tracking usage
    let token_counter = TokenCounter::new(model).ok();

    // Build tool-to-server mapping for O(1) lookups
    let tool_server_map = build_tool_server_map(&tools, mcp_server_names).await;

    // Add system prompt if provided
    if let Some(sys_prompt) = system_prompt {
        conversation_messages.push(Message {
            role: "system".to_string(),
            content_type: MessageContent::Text {
                content: Some(sys_prompt.to_string()),
            },
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

    // Use provided max_iterations or default
    let max_iterations = max_iterations.unwrap_or(DEFAULT_MAX_ITERATIONS);
    let mut iteration = 0;

    loop {
        iteration += 1;
        if iteration > max_iterations {
            anyhow::bail!(
                "Maximum tool execution iterations reached ({})",
                max_iterations
            );
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
        let response = client.chat_with_tools(&request).await?;

        // Track input token usage if we have a counter
        if let Some(ref counter) = token_counter {
            // Count tokens for all messages in the request
            let mut input_tokens = 0i32;
            for msg in &request.messages {
                match &msg.content_type {
                    MessageContent::Text { content } => {
                        if let Some(text) = content {
                            input_tokens += counter.count_tokens(text) as i32;
                        }
                    }
                    MessageContent::Multimodal { content } => {
                        // Count text tokens from multimodal content
                        for part in content {
                            match part {
                                crate::provider::ContentPart::Text { text } => {
                                    input_tokens += counter.count_tokens(text) as i32;
                                }
                                crate::provider::ContentPart::ImageUrl { .. } => {
                                    // Images are harder to count, use approximation
                                    // Typical vision models charge ~85 tokens per low-detail image
                                    input_tokens += IMAGE_TOKEN_ESTIMATE;
                                }
                            }
                        }
                    }
                }
            }
            total_input_tokens += input_tokens;
            crate::debug_log!("Iteration {} input tokens: {}", iteration, input_tokens);
        }

        if let Some(choice) = response.choices.first() {
            // Track output tokens for this response
            if let Some(ref counter) = token_counter {
                if let Some(content) = &choice.message.content {
                    let output_tokens = counter.count_tokens(content) as i32;
                    total_output_tokens += output_tokens;
                    crate::debug_log!("Iteration {} output tokens: {}", iteration, output_tokens);
                }
            }
            crate::debug_log!(
                "Response choice - tool_calls: {}, content: {}",
                choice.message.tool_calls.as_ref().map_or(0, |tc| tc.len()),
                choice
                    .message
                    .content
                    .as_ref()
                    .map_or("None", |c| if c.len() > 50 { &c[..50] } else { c })
            );

            // Check if the LLM made tool calls
            if let Some(tool_calls) = &choice.message.tool_calls {
                if !tool_calls.is_empty() {
                    crate::debug_log!(
                        "LLM made {} tool calls in iteration {}",
                        tool_calls.len(),
                        iteration
                    );

                    // Add the assistant's tool call message to conversation
                    conversation_messages
                        .push(Message::assistant_with_tool_calls(tool_calls.clone()));

                    // Execute tool calls concurrently for better performance
                    crate::debug_log!("Executing {} tool calls concurrently", tool_calls.len());

                    let mut futures = Vec::new();
                    for tool_call in tool_calls.iter() {
                        let future = execute_single_tool_call(
                            tool_call,
                            tools.as_ref(),
                            mcp_server_names,
                            &tool_server_map,
                        );
                        futures.push(future);
                    }

                    // Wait for all tool calls to complete
                    let results = futures_util::future::join_all(futures).await;

                    // Add all tool results to conversation
                    for result in results {
                        match result {
                            Ok(exec_result) => {
                                conversation_messages.push(Message::tool_result(
                                    exec_result.tool_call_id,
                                    exec_result.result_content,
                                ));
                            }
                            Err(e) => {
                                eprintln!("⚠️  Tool execution error: {}", e);
                                crate::debug_log!("Tool execution error: {}", e);
                            }
                        }
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

                            // Output tokens already tracked above (line 725-730)
                            // Exit immediately when LLM provides content (final answer)
                            return Ok((
                                content.clone(),
                                Some(total_input_tokens),
                                Some(total_output_tokens),
                            ));
                        }
                    }
                }
            } else if let Some(content) = &choice.message.content {
                // LLM provided a final answer without tool calls field
                crate::debug_log!(
                    "LLM provided final answer without tool_calls field after {} iterations: {}",
                    iteration,
                    if content.len() > 100 {
                        format!("{}...", &content[..100])
                    } else {
                        content.clone()
                    }
                );

                // Output tokens already tracked above (line 725-730)
                // Exit immediately when LLM provides content (final answer)
                return Ok((
                    content.clone(),
                    Some(total_input_tokens),
                    Some(total_output_tokens),
                ));
            } else {
                // LLM provided neither tool calls nor content - this shouldn't happen
                crate::debug_log!(
                    "LLM provided neither tool calls nor content in iteration {}",
                    iteration
                );
                anyhow::bail!(
                    "No content or tool calls in response from LLM in iteration {}",
                    iteration
                );
            }
        } else {
            anyhow::bail!("No response from API");
        }
    }
}

/// Result of a single tool execution
struct ToolExecutionResult {
    tool_call_id: String,
    result_content: String,
}

/// Execute a single tool call with validation, timeout, and error handling
async fn execute_single_tool_call(
    tool_call: &crate::provider::ToolCall,
    tools: Option<&Vec<crate::provider::Tool>>,
    mcp_server_names: &[&str],
    tool_server_map: &std::collections::HashMap<String, String>,
) -> Result<ToolExecutionResult> {
    use std::time::Duration;

    crate::debug_log!(
        "Executing tool call: {} with args: {}",
        tool_call.function.name,
        tool_call.function.arguments
    );

    // Parse arguments as JSON value
    let args_value: serde_json::Value = serde_json::from_str(&tool_call.function.arguments)?;

    // Validate arguments against tool schema
    if let Some(tool_defs) = tools {
        if let Err(e) = validate_tool_arguments(&tool_call.function.name, &args_value, tool_defs) {
            let error_msg = format!(
                "Tool argument validation failed for '{}': {}",
                tool_call.function.name, e
            );
            eprintln!("⚠️  {}", error_msg);
            crate::debug_log!("{}", error_msg);

            return Ok(ToolExecutionResult {
                tool_call_id: tool_call.id.clone(),
                result_content: error_msg,
            });
        }
        crate::debug_log!(
            "Tool '{}' arguments validated successfully",
            tool_call.function.name
        );
    }

    // Find which MCP server has this function
    let daemon_client = crate::mcp_daemon::DaemonClient::new()?;
    let mut tool_result = None;

    // Use mapping if available for O(1) lookup, otherwise iterate
    let servers_to_try: Vec<&str> =
        if let Some(server_name) = tool_server_map.get(&tool_call.function.name) {
            vec![server_name.as_str()]
        } else {
            mcp_server_names.to_vec()
        };

    for server_name in servers_to_try {
        // Add timeout to prevent hanging
        let call_future =
            daemon_client.call_tool(server_name, &tool_call.function.name, args_value.clone());

        match tokio::time::timeout(
            Duration::from_secs(TOOL_EXECUTION_TIMEOUT_SECS),
            call_future,
        )
        .await
        {
            Ok(Ok(result)) => {
                crate::debug_log!(
                    "Tool call successful on server '{}': {}",
                    server_name,
                    serde_json::to_string(&result).unwrap_or_else(|_| "invalid json".to_string())
                );
                tool_result = Some(format_tool_result(&result));
                break;
            }
            Ok(Err(e)) => {
                crate::debug_log!("Tool call failed on server '{}': {}", server_name, e);
                continue;
            }
            Err(_) => {
                let timeout_msg = format!(
                    "Tool call to '{}' on server '{}' timed out after {} seconds",
                    tool_call.function.name, server_name, TOOL_EXECUTION_TIMEOUT_SECS
                );
                eprintln!("⚠️  {}", timeout_msg);
                crate::debug_log!("{}", timeout_msg);
                continue;
            }
        }
    }

    let result_content = tool_result.unwrap_or_else(|| {
        format!(
            "Error: Function '{}' not found on any MCP server",
            tool_call.function.name
        )
    });

    crate::debug_log!(
        "Tool result for {}: {}",
        tool_call.function.name,
        if result_content.len() > 100 {
            format!("{}...", &result_content[..100])
        } else {
            result_content.clone()
        }
    );

    Ok(ToolExecutionResult {
        tool_call_id: tool_call.id.clone(),
        result_content,
    })
}

/// Build a mapping of tool names to server names for O(1) lookups
async fn build_tool_server_map(
    tools: &Option<Vec<crate::provider::Tool>>,
    mcp_server_names: &[&str],
) -> std::collections::HashMap<String, String> {
    use std::collections::HashMap;

    let mut map = HashMap::new();

    if tools.is_some() {
        // Use daemon client to get tools from each server
        if let Ok(daemon_client) = crate::mcp_daemon::DaemonClient::new() {
            for server_name in mcp_server_names {
                if let Ok(server_tools) = daemon_client.list_tools(server_name).await {
                    if let Some(tools_from_server) = server_tools.get(*server_name) {
                        for tool in tools_from_server {
                            // Map tool name to server name
                            map.insert(tool.name.to_string(), server_name.to_string());
                        }
                    }
                }
            }
        }
    }

    crate::debug_log!("Built tool-to-server mapping with {} entries", map.len());

    map
}

// Helper function to validate tool arguments against schema
fn validate_tool_arguments(
    tool_name: &str,
    arguments: &serde_json::Value,
    tools: &[crate::provider::Tool],
) -> Result<()> {
    // Find the tool definition
    let tool_def = tools
        .iter()
        .find(|t| t.function.name == tool_name)
        .ok_or_else(|| anyhow::anyhow!("Tool '{}' not found in tool definitions", tool_name))?;

    let schema = &tool_def.function.parameters;

    // Ensure arguments is an object
    let args_obj = arguments.as_object().ok_or_else(|| {
        anyhow::anyhow!("Tool arguments must be a JSON object, got: {}", arguments)
    })?;

    // Check required fields
    if let Some(required) = schema.get("required").and_then(|r| r.as_array()) {
        for req_field in required {
            if let Some(field_name) = req_field.as_str() {
                if !args_obj.contains_key(field_name) {
                    return Err(anyhow::anyhow!(
                        "Tool '{}' missing required argument: '{}'",
                        tool_name,
                        field_name
                    ));
                }
            }
        }
    }

    // Validate field types if properties are defined
    if let Some(properties) = schema.get("properties").and_then(|p| p.as_object()) {
        for (arg_name, arg_value) in args_obj {
            // Check if this property is defined in the schema
            if let Some(prop_schema) = properties.get(arg_name) {
                // Validate type if specified
                if let Some(expected_type) = prop_schema.get("type").and_then(|t| t.as_str()) {
                    let actual_type = match arg_value {
                        serde_json::Value::Null => "null",
                        serde_json::Value::Bool(_) => "boolean",
                        serde_json::Value::Number(_) => "number",
                        serde_json::Value::String(_) => "string",
                        serde_json::Value::Array(_) => "array",
                        serde_json::Value::Object(_) => "object",
                    };

                    // Handle integer as a special case of number
                    if expected_type == "integer" && arg_value.is_number() {
                        if let Some(num) = arg_value.as_f64() {
                            if num.fract() != 0.0 {
                                return Err(anyhow::anyhow!(
                                    "Tool '{}' argument '{}': expected integer, got number with decimal: {}",
                                    tool_name,
                                    arg_name,
                                    num
                                ));
                            }
                        }
                    } else if expected_type != actual_type
                        && !(expected_type == "number" && actual_type == "number")
                    {
                        return Err(anyhow::anyhow!(
                            "Tool '{}' argument '{}': expected type '{}', got '{}'",
                            tool_name,
                            arg_name,
                            expected_type,
                            actual_type
                        ));
                    }
                }

                // Validate enum if specified
                if let Some(enum_values) = prop_schema.get("enum").and_then(|e| e.as_array()) {
                    if !enum_values.contains(arg_value) {
                        return Err(anyhow::anyhow!(
                            "Tool '{}' argument '{}': value must be one of {:?}, got: {}",
                            tool_name,
                            arg_name,
                            enum_values,
                            arg_value
                        ));
                    }
                }
            }
        }
    }

    crate::debug_log!("Tool '{}' arguments validated successfully", tool_name);

    Ok(())
}

// Helper function to format tool result for display
fn format_tool_result(result: &serde_json::Value) -> String {
    if let Some(content_array) = result.get("content") {
        if let Some(content_items) = content_array.as_array() {
            let mut formatted = String::new();
            let mut original_length = 0usize;

            for item in content_items {
                if let Some(text) = item.get("text") {
                    if let Some(text_str) = text.as_str() {
                        original_length += text_str.len();

                        // Check if adding this text would exceed the limit
                        if formatted.len() + text_str.len() > MAX_TOOL_RESULT_LENGTH {
                            // Add as much as we can
                            let remaining = MAX_TOOL_RESULT_LENGTH.saturating_sub(formatted.len());
                            if remaining > 0 {
                                formatted.push_str(&text_str[..remaining.min(text_str.len())]);
                            }

                            // Add detailed truncation message
                            let truncation_msg = format!(
                                "\n\n[TRUNCATED: Result too large. Showing first {} bytes of {} total. Consider requesting smaller chunks or specific fields.]",
                                MAX_TOOL_RESULT_LENGTH,
                                original_length
                            );
                            formatted.push_str(&truncation_msg);
                            break; // Stop processing more items
                        } else {
                            formatted.push_str(text_str);
                            formatted.push('\n');
                        }
                    }
                }
            }
            return formatted.trim().to_string();
        }
    }

    // Fallback to pretty-printed JSON (also with truncation)
    let json_result = serde_json::to_string_pretty(result)
        .unwrap_or_else(|_| "Error formatting result".to_string());

    if json_result.len() > MAX_TOOL_RESULT_LENGTH {
        let truncation_msg = format!(
            "\n\n[TRUNCATED: Result too large. Showing first {} bytes of {} total.]",
            MAX_TOOL_RESULT_LENGTH,
            json_result.len()
        );
        format!(
            "{}{}",
            &json_result[..MAX_TOOL_RESULT_LENGTH],
            truncation_msg
        )
    } else {
        json_result
    }
}

// Message-based versions of the chat functions for handling multimodal content

#[allow(clippy::too_many_arguments)]
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
    crate::debug_log!(
        "Sending chat request with messages - provider: '{}', model: '{}', messages: {}",
        provider_name,
        model,
        messages.len()
    );

    // Build final messages including system prompt if needed
    let mut final_messages = Vec::new();

    // Add system prompt if provided and not already in messages
    if let Some(sys_prompt) = system_prompt {
        let has_system = messages.iter().any(|m| m.role == "system");
        if !has_system {
            final_messages.push(Message {
                role: "system".to_string(),
                content_type: MessageContent::Text {
                    content: Some(sys_prompt.to_string()),
                },
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

    let response = client.chat(&request).await?;

    // For now, return None for token counts as we'd need to implement multimodal token counting
    Ok((response, None, None))
}

#[allow(clippy::too_many_arguments)]
pub async fn send_chat_request_with_streaming_messages(
    client: &LLMClient,
    model: &str,
    messages: &[Message],
    system_prompt: Option<&str>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
    provider_name: &str,
    tools: Option<Vec<crate::provider::Tool>>,
    on_connect: Option<Box<dyn FnMut() + Send>>,
) -> Result<()> {
    crate::debug_log!(
        "Sending streaming chat request with messages - provider: '{}', model: '{}', messages: {}",
        provider_name,
        model,
        messages.len()
    );

    // Build final messages including system prompt if needed
    let mut final_messages = Vec::new();

    // Add system prompt if provided and not already in messages
    if let Some(sys_prompt) = system_prompt {
        let has_system = messages.iter().any(|m| m.role == "system");
        if !has_system {
            final_messages.push(Message {
                role: "system".to_string(),
                content_type: MessageContent::Text {
                    content: Some(sys_prompt.to_string()),
                },
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

    client.chat_stream(&request, on_connect).await?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
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
    max_iterations: Option<u32>,
) -> Result<(String, Option<i32>, Option<i32>)> {
    crate::debug_log!("Sending chat request with tool execution and messages - provider: '{}', model: '{}', messages: {}",
                      provider_name, model, messages.len());

    let mut conversation_messages = Vec::new();

    // Build tool-to-server mapping for O(1) lookups
    let tool_server_map = build_tool_server_map(&tools, mcp_server_names).await;

    // Add system prompt if provided and not already in messages
    if let Some(sys_prompt) = system_prompt {
        let has_system = messages.iter().any(|m| m.role == "system");
        if !has_system {
            conversation_messages.push(Message {
                role: "system".to_string(),
                content_type: MessageContent::Text {
                    content: Some(sys_prompt.to_string()),
                },
                tool_calls: None,
                tool_call_id: None,
            });
        }
    }

    // Add all provided messages
    conversation_messages.extend_from_slice(messages);

    // Use provided max_iterations or default
    let max_iterations = max_iterations.unwrap_or(DEFAULT_MAX_ITERATIONS);
    let mut iteration = 0;

    loop {
        iteration += 1;
        if iteration > max_iterations {
            anyhow::bail!(
                "Maximum tool execution iterations reached ({})",
                max_iterations
            );
        }

        let request = ChatRequest {
            model: model.to_string(),
            messages: conversation_messages.clone(),
            max_tokens: max_tokens.or(Some(1024)),
            temperature: temperature.or(Some(0.7)),
            tools: tools.clone(),
            stream: None,
        };

        let response = client.chat_with_tools(&request).await?;

        if let Some(choice) = response.choices.first() {
            if let Some(tool_calls) = &choice.message.tool_calls {
                if !tool_calls.is_empty() {
                    conversation_messages
                        .push(Message::assistant_with_tool_calls(tool_calls.clone()));

                    // Execute tool calls concurrently for better performance
                    crate::debug_log!("Executing {} tool calls concurrently", tool_calls.len());

                    let mut futures = Vec::new();
                    for tool_call in tool_calls.iter() {
                        let future = execute_single_tool_call(
                            tool_call,
                            tools.as_ref(),
                            mcp_server_names,
                            &tool_server_map,
                        );
                        futures.push(future);
                    }

                    // Wait for all tool calls to complete
                    let results = futures_util::future::join_all(futures).await;

                    // Add all tool results to conversation
                    for result in results {
                        match result {
                            Ok(exec_result) => {
                                conversation_messages.push(Message::tool_result(
                                    exec_result.tool_call_id,
                                    exec_result.result_content,
                                ));
                            }
                            Err(e) => {
                                eprintln!("⚠️  Tool execution error: {}", e);
                                crate::debug_log!("Tool execution error: {}", e);
                            }
                        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::{Function, Tool};

    #[test]
    fn test_validate_tool_arguments_success() {
        let tools = vec![Tool {
            tool_type: "function".to_string(),
            function: Function {
                name: "test_tool".to_string(),
                description: "A test tool".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "name": {"type": "string"},
                        "age": {"type": "integer"},
                        "active": {"type": "boolean"}
                    },
                    "required": ["name", "age"]
                }),
            },
        }];

        let valid_args = serde_json::json!({
            "name": "John",
            "age": 30,
            "active": true
        });

        let result = validate_tool_arguments("test_tool", &valid_args, &tools);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_tool_arguments_missing_required() {
        let tools = vec![Tool {
            tool_type: "function".to_string(),
            function: Function {
                name: "test_tool".to_string(),
                description: "A test tool".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "name": {"type": "string"},
                        "age": {"type": "integer"}
                    },
                    "required": ["name", "age"]
                }),
            },
        }];

        let invalid_args = serde_json::json!({
            "name": "John"
            // Missing required "age" field
        });

        let result = validate_tool_arguments("test_tool", &invalid_args, &tools);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("missing required argument"));
    }

    #[test]
    fn test_validate_tool_arguments_wrong_type() {
        let tools = vec![Tool {
            tool_type: "function".to_string(),
            function: Function {
                name: "test_tool".to_string(),
                description: "A test tool".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "name": {"type": "string"},
                        "age": {"type": "integer"}
                    }
                }),
            },
        }];

        let invalid_args = serde_json::json!({
            "name": "John",
            "age": "thirty" // Should be integer
        });

        let result = validate_tool_arguments("test_tool", &invalid_args, &tools);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("expected type"));
    }

    #[test]
    fn test_validate_tool_arguments_integer_vs_float() {
        let tools = vec![Tool {
            tool_type: "function".to_string(),
            function: Function {
                name: "test_tool".to_string(),
                description: "A test tool".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "count": {"type": "integer"}
                    }
                }),
            },
        }];

        // Should fail - float when integer expected
        let invalid_args = serde_json::json!({
            "count": 30.5
        });

        let result = validate_tool_arguments("test_tool", &invalid_args, &tools);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("expected integer"));

        // Should succeed - integer value
        let valid_args = serde_json::json!({
            "count": 30
        });

        let result = validate_tool_arguments("test_tool", &valid_args, &tools);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_tool_arguments_enum_constraint() {
        let tools = vec![Tool {
            tool_type: "function".to_string(),
            function: Function {
                name: "test_tool".to_string(),
                description: "A test tool".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "status": {
                            "type": "string",
                            "enum": ["active", "inactive", "pending"]
                        }
                    }
                }),
            },
        }];

        // Should succeed - valid enum value
        let valid_args = serde_json::json!({
            "status": "active"
        });
        let result = validate_tool_arguments("test_tool", &valid_args, &tools);
        assert!(result.is_ok());

        // Should fail - invalid enum value
        let invalid_args = serde_json::json!({
            "status": "completed"
        });
        let result = validate_tool_arguments("test_tool", &invalid_args, &tools);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must be one of"));
    }

    #[test]
    fn test_validate_tool_arguments_tool_not_found() {
        let tools = vec![Tool {
            tool_type: "function".to_string(),
            function: Function {
                name: "test_tool".to_string(),
                description: "A test tool".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            },
        }];

        let args = serde_json::json!({});
        let result = validate_tool_arguments("nonexistent_tool", &args, &tools);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_validate_tool_arguments_not_object() {
        let tools = vec![Tool {
            tool_type: "function".to_string(),
            function: Function {
                name: "test_tool".to_string(),
                description: "A test tool".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            },
        }];

        let invalid_args = serde_json::json!("not an object");
        let result = validate_tool_arguments("test_tool", &invalid_args, &tools);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must be a JSON object"));
    }

    #[test]
    fn test_format_tool_result_with_text_content() {
        let result = serde_json::json!({
            "content": [
                {"text": "Hello, world!"},
                {"text": "This is a test."}
            ]
        });

        let formatted = format_tool_result(&result);
        assert!(formatted.contains("Hello, world!"));
        assert!(formatted.contains("This is a test."));
    }

    #[test]
    fn test_format_tool_result_truncation() {
        // Create a large text that exceeds 10KB
        let large_text = "A".repeat(15000);
        let result = serde_json::json!({
            "content": [
                {"text": large_text}
            ]
        });

        let formatted = format_tool_result(&result);
        // Allow for longer truncation message (up to 200 chars for the detailed message)
        assert!(formatted.len() <= MAX_TOOL_RESULT_LENGTH + 200);
        assert!(formatted.contains("[TRUNCATED"));
        assert!(formatted.contains("bytes"));
    }

    #[test]
    fn test_format_tool_result_fallback_to_json() {
        let result = serde_json::json!({
            "status": "success",
            "data": {
                "id": 123,
                "name": "test"
            }
        });

        let formatted = format_tool_result(&result);
        // Should be pretty-printed JSON when no content array
        assert!(formatted.contains("\"status\""));
        assert!(formatted.contains("\"success\""));
    }

    #[test]
    fn test_format_tool_result_multiple_items_truncation() {
        // Create multiple items where the sum exceeds 10KB
        let item1 = "B".repeat(6000);
        let item2 = "C".repeat(6000);
        let result = serde_json::json!({
            "content": [
                {"text": item1},
                {"text": item2}
            ]
        });

        let formatted = format_tool_result(&result);
        // Allow for longer truncation message (up to 200 chars for the detailed message)
        assert!(formatted.len() <= MAX_TOOL_RESULT_LENGTH + 200);
        assert!(formatted.contains("[TRUNCATED"));
        assert!(formatted.contains("bytes"));
        // First item should be included
        assert!(formatted.chars().filter(|&c| c == 'B').count() > 0);
    }
}
