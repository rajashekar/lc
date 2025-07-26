use anyhow::Result;
use crate::provider::{OpenAIClient, ChatRequest, Message};
use crate::database::ChatEntry;
use crate::config::Config;
use crate::token_utils::TokenCounter;
use crate::model_metadata::MetadataExtractor;
use chrono::{DateTime, Utc};

pub async fn send_chat_request(
    client: &OpenAIClient,
    model: &str,
    prompt: &str,
    history: &[ChatEntry],
    system_prompt: Option<&str>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
    tools: Option<Vec<crate::provider::Tool>>,
) -> Result<String> {
    let mut messages = Vec::new();
    
    // Add system prompt if provided
    if let Some(sys_prompt) = system_prompt {
        messages.push(Message {
            role: "system".to_string(),
            content: Some(sys_prompt.to_string()),
            tool_calls: None,
            tool_call_id: None,
        });
    }
    
    // Add conversation history
    for entry in history {
        messages.push(Message::user(entry.question.clone()));
        messages.push(Message::assistant(entry.response.clone()));
    }
    
    // Add current prompt
    messages.push(Message::user(prompt.to_string()));
    
    let request = ChatRequest {
        model: model.to_string(),
        messages,
        max_tokens: max_tokens.or(Some(1024)),
        temperature: temperature.or(Some(0.7)),
        tools,
    };
    
    client.chat(&request).await
}

pub async fn send_chat_request_with_validation(
    client: &OpenAIClient,
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
            content: Some(sys_prompt.to_string()),
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
    };
    
    crate::debug_log!("Sending chat request with {} messages, max_tokens: {:?}, temperature: {:?}",
                      messages.len(), request.max_tokens, request.temperature);
    
    // Send the request
    crate::debug_log!("Making API call to chat endpoint...");
    let response = client.chat(&request).await?;
    
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
    crate::debug_log!("Creating authenticated client for provider '{}'", provider_name);
    
    // Clone the provider config to avoid borrowing issues
    let provider_config = config.get_provider(provider_name)?.clone();
    
    crate::debug_log!("Provider '{}' config - endpoint: {}, models_path: {}, chat_path: {}",
                      provider_name, provider_config.endpoint, provider_config.models_path, provider_config.chat_path);
    crate::debug_log!("Provider '{}' has {} custom headers", provider_name, provider_config.headers.len());
    
    // Create a temporary client with the API key for token retrieval
    crate::debug_log!("Creating temporary client for token retrieval");
    let temp_client = OpenAIClient::new_with_headers(
        provider_config.endpoint.clone(),
        provider_config.api_key.clone().unwrap_or_default(),
        provider_config.models_path.clone(),
        provider_config.chat_path.clone(),
        provider_config.headers.clone(),
    );
    
    // Get the appropriate authentication token
    crate::debug_log!("Getting authentication token for provider '{}'", provider_name);
    let auth_token = get_or_refresh_token(config, provider_name, &temp_client).await?;
    
    crate::debug_log!("Successfully obtained auth token for provider '{}' (length: {})", provider_name, auth_token.len());
    
    // Create the final client with the authentication token
    crate::debug_log!("Creating final authenticated client for provider '{}'", provider_name);
    let client = OpenAIClient::new_with_headers(
        provider_config.endpoint,
        auth_token,
        provider_config.models_path,
        provider_config.chat_path,
        provider_config.headers,
    );
    
    crate::debug_log!("Successfully created authenticated client for provider '{}'", provider_name);
    Ok(client)
}

// New function to handle tool execution loop
pub async fn send_chat_request_with_tool_execution(
    client: &OpenAIClient,
    model: &str,
    prompt: &str,
    history: &[ChatEntry],
    system_prompt: Option<&str>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
    provider_name: &str,
    tools: Option<Vec<crate::provider::Tool>>,
    mcp_server_names: &[&str],
) -> Result<(String, Option<i32>, Option<i32>)> {
    use crate::provider::{Message, ChatRequest};
    use crate::mcp::McpManager;
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
            content: Some(sys_prompt.to_string()),
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
    
    let mut manager = McpManager::new();
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
        };
        
        // Make the API call
        let response = client.chat_with_tools(&request).await?;
        
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
                crate::debug_log!("LLM made {} tool calls in iteration {}", tool_calls.len(), iteration);
                
                // Add the assistant's tool call message to conversation
                conversation_messages.push(Message::assistant_with_tool_calls(tool_calls.clone()));
                
                // Execute each tool call
                for (i, tool_call) in tool_calls.iter().enumerate() {
                    crate::debug_log!("Executing tool call {}/{}: {} with args: {}",
                                     i + 1, tool_calls.len(), tool_call.function.name, tool_call.function.arguments);
                    
                    // Find which MCP server has this function
                    let mut tool_result = None;
                    for server_name in mcp_server_names {
                        match manager.invoke_function(
                            server_name,
                            &tool_call.function.name,
                            &parse_function_arguments(&tool_call.function.arguments)?
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
            } else if let Some(content) = &choice.message.content {
                // LLM provided a final answer without tool calls
                crate::debug_log!("LLM provided final answer after {} iterations: {}",
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

// Helper function to parse function arguments from JSON string
fn parse_function_arguments(args_json: &str) -> Result<Vec<String>> {
    let args_value: serde_json::Value = serde_json::from_str(args_json)?;
    let mut args_vec = Vec::new();
    
    if let Some(obj) = args_value.as_object() {
        for (key, value) in obj {
            let arg_str = match value {
                serde_json::Value::String(s) => format!("{}={}", key, s),
                _ => format!("{}={}", key, value.to_string()),
            };
            args_vec.push(arg_str);
        }
    }
    
    Ok(args_vec)
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