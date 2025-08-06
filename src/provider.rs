use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use futures_util::StreamExt;
use std::io::{self, Write};

#[derive(Debug, Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

// Chat request without model field for providers that specify model in URL
#[derive(Debug, Serialize)]
pub struct ChatRequestWithoutModel {
    pub messages: Vec<Message>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

impl From<&ChatRequest> for ChatRequestWithoutModel {
    fn from(request: &ChatRequest) -> Self {
        Self {
            messages: request.messages.clone(),
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            tools: request.tools.clone(),
            stream: request.stream,
        }
    }
}

// Bedrock-specific request structure
#[derive(Debug, Serialize)]
pub struct BedrockChatRequest {
    pub messages: Vec<BedrockMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct BedrockMessage {
    pub role: String,
    pub content: Vec<BedrockContentPart>,
}

#[derive(Debug, Serialize)]
pub struct BedrockContentPart {
    pub text: String,
}

impl BedrockMessage {
    pub fn from_message(message: &Message) -> Self {
        let text_content = match &message.content_type {
            MessageContent::Text { content } => {
                content.as_ref().unwrap_or(&String::new()).clone()
            }
            MessageContent::Multimodal { content } => {
                // Extract text from multimodal content
                content.iter()
                    .filter_map(|part| match part {
                        ContentPart::Text { text } => Some(text.clone()),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            }
        };

        Self {
            role: message.role.clone(),
            content: vec![BedrockContentPart { text: text_content }],
        }
    }
}

impl BedrockChatRequest {
    pub fn from_chat_request(request: &ChatRequest) -> Self {
        Self {
            messages: request.messages.iter().map(BedrockMessage::from_message).collect(),
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            tools: request.tools.clone(),
            stream: request.stream,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct EmbeddingRequest {
    pub model: String,
    pub input: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding_format: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ImageGenerationRequest {
    pub prompt: String,
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ImageGenerationResponse {
    pub data: Vec<ImageData>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ImageData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub b64_json: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revised_prompt: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct EmbeddingResponse {
    pub data: Vec<EmbeddingData>,
    pub usage: EmbeddingUsage,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EmbeddingData {
    pub embedding: Vec<f64>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EmbeddingUsage {
    pub total_tokens: u32,
}

#[derive(Debug, Serialize, Clone)]
pub struct Tool {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: Function,
}

#[derive(Debug, Serialize, Clone)]
pub struct Function {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

// Updated Message struct to support multimodal content
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: String,
    #[serde(flatten)]
    pub content_type: MessageContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

// New enum to support both text and multimodal content
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum MessageContent {
    Text {
        content: Option<String>,
    },
    Multimodal {
        content: Vec<ContentPart>,
    },
}

// Content part for multimodal messages
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum ContentPart {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image_url")]
    ImageUrl { image_url: ImageUrl },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImageUrl {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>, // "low", "high", or "auto"
}

impl Message {
    pub fn user(content: String) -> Self {
        Self {
            role: "user".to_string(),
            content_type: MessageContent::Text { content: Some(content) },
            tool_calls: None,
            tool_call_id: None,
        }
    }
    
    #[allow(dead_code)]
    pub fn user_with_image(text: String, image_data: String, detail: Option<String>) -> Self {
        Self {
            role: "user".to_string(),
            content_type: MessageContent::Multimodal {
                content: vec![
                    ContentPart::Text { text },
                    ContentPart::ImageUrl {
                        image_url: ImageUrl {
                            url: image_data,
                            detail,
                        },
                    },
                ],
            },
            tool_calls: None,
            tool_call_id: None,
        }
    }
    
    pub fn assistant(content: String) -> Self {
        Self {
            role: "assistant".to_string(),
            content_type: MessageContent::Text { content: Some(content) },
            tool_calls: None,
            tool_call_id: None,
        }
    }
    
    pub fn assistant_with_tool_calls(tool_calls: Vec<ToolCall>) -> Self {
        Self {
            role: "assistant".to_string(),
            content_type: MessageContent::Text { content: None },
            tool_calls: Some(tool_calls),
            tool_call_id: None,
        }
    }
    
    pub fn tool_result(tool_call_id: String, content: String) -> Self {
        Self {
            role: "tool".to_string(),
            content_type: MessageContent::Text { content: Some(content) },
            tool_calls: None,
            tool_call_id: Some(tool_call_id),
        }
    }
    
    // Helper method to get text content for backward compatibility
    pub fn get_text_content(&self) -> Option<&String> {
        match &self.content_type {
            MessageContent::Text { content } => content.as_ref(),
            MessageContent::Multimodal { content } => {
                // Return the first text content if available
                content.iter().find_map(|part| match part {
                    ContentPart::Text { text } => Some(text),
                    _ => None,
                })
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    pub choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
pub struct Choice {
    pub message: ResponseMessage,
}

#[derive(Debug, Deserialize)]
pub struct ResponseMessage {
    #[allow(dead_code)]
    pub role: String,
    pub content: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: FunctionCall,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Deserialize)]
pub struct LlamaChatResponse {
    pub completion_message: LlamaMessage,
}

#[derive(Debug, Deserialize)]
pub struct LlamaMessage {
    #[allow(dead_code)]
    pub role: String,
    pub content: LlamaContent,
}

#[derive(Debug, Deserialize)]
pub struct LlamaContent {
    #[serde(rename = "type")]
    #[allow(dead_code)]
    pub content_type: String,
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct CohereChatResponse {
    pub message: CohereMessage,
}

#[derive(Debug, Deserialize)]
pub struct CohereMessage {
    #[allow(dead_code)]
    pub role: String,
    pub content: Vec<CohereContentItem>,
}

#[derive(Debug, Deserialize)]
pub struct CohereContentItem {
    #[serde(rename = "type")]
    #[allow(dead_code)]
    pub content_type: String,
    pub text: String,
}

// Cloudflare-specific response structures
#[derive(Debug, Deserialize)]
pub struct CloudflareChatResponse {
    pub result: CloudflareResult,
    pub success: bool,
    #[allow(dead_code)]
    pub errors: Vec<serde_json::Value>,
    #[allow(dead_code)]
    pub messages: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct CloudflareResult {
    pub response: String,
    #[allow(dead_code)]
    pub tool_calls: Vec<serde_json::Value>,
    #[allow(dead_code)]
    pub usage: CloudflareUsage,
}

#[derive(Debug, Deserialize)]
pub struct CloudflareUsage {
    #[allow(dead_code)]
    pub prompt_tokens: u32,
    #[allow(dead_code)]
    pub completion_tokens: u32,
    #[allow(dead_code)]
    pub total_tokens: u32,
}

// Bedrock-specific response structures
#[derive(Debug, Deserialize)]
pub struct BedrockChatResponse {
    pub output: BedrockOutput,
    #[serde(rename = "stopReason")]
    #[allow(dead_code)]
    pub stop_reason: String,
    #[allow(dead_code)]
    pub usage: BedrockUsage,
    #[allow(dead_code)]
    pub metrics: BedrockMetrics,
}

#[derive(Debug, Deserialize)]
pub struct BedrockOutput {
    pub message: BedrockResponseMessage,
}

#[derive(Debug, Deserialize)]
pub struct BedrockResponseMessage {
    pub content: Vec<BedrockResponseContentPart>,
    #[allow(dead_code)]
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct BedrockResponseContentPart {
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct BedrockUsage {
    #[serde(rename = "inputTokens")]
    #[allow(dead_code)]
    pub input_tokens: u32,
    #[serde(rename = "outputTokens")]
    #[allow(dead_code)]
    pub output_tokens: u32,
    #[serde(rename = "totalTokens")]
    #[allow(dead_code)]
    pub total_tokens: u32,
}

#[derive(Debug, Deserialize)]
pub struct BedrockMetrics {
    #[serde(rename = "latencyMs")]
    #[allow(dead_code)]
    pub latency_ms: u32,
}

#[derive(Debug, Deserialize)]
pub struct ModelsResponse {
    #[serde(alias = "models")]
    pub data: Vec<Model>,
}

#[derive(Debug, Deserialize)]
pub struct CohereModelsResponse {
    pub models: Vec<CohereModel>,
}

#[derive(Debug, Deserialize)]
pub struct CohereModel {
    pub name: String,
    #[serde(default = "default_object_type")]
    pub object: String,
}

#[derive(Debug, Deserialize)]
pub struct Provider {
    pub provider: String,
    #[allow(dead_code)]
    pub status: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub supports_tools: bool,
    #[serde(default)]
    #[allow(dead_code)]
    pub supports_structured_output: bool,
}

#[derive(Debug, Deserialize)]
pub struct Model {
    pub id: String,
    #[serde(default = "default_object_type")]
    pub object: String,
    #[serde(default)]
    pub providers: Vec<Provider>,
}

fn default_object_type() -> String {
    "model".to_string()
}

#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    pub token: String,
    pub expires_at: i64, // Unix timestamp
}

pub struct OpenAIClient {
    client: Client,
    streaming_client: Client,  // Separate client optimized for streaming
    base_url: String,
    api_key: String,
    models_path: String,
    chat_path: String,
    custom_headers: std::collections::HashMap<String, String>,
    provider_config: Option<crate::config::ProviderConfig>,
}

impl OpenAIClient {
    
    pub fn new_with_headers(base_url: String, api_key: String, models_path: String, chat_path: String, custom_headers: std::collections::HashMap<String, String>) -> Self {
        // Create optimized HTTP client with connection pooling and keep-alive settings
        // This client keeps compression enabled for regular requests
        let client = Client::builder()
            .pool_max_idle_per_host(10) // Keep up to 10 idle connections per host
            .pool_idle_timeout(Duration::from_secs(90)) // Keep connections alive for 90 seconds
            .tcp_keepalive(Duration::from_secs(60)) // TCP keep-alive every 60 seconds
            .timeout(Duration::from_secs(60)) // Total request timeout
            .connect_timeout(Duration::from_secs(10)) // Connection establishment timeout
            .user_agent(concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("Failed to create optimized HTTP client");

        // Create a separate streaming-optimized client
        let streaming_client = Client::builder()
            .timeout(Duration::from_secs(300))  // Longer timeout for streaming
            .build()
            .expect("Failed to create streaming-optimized HTTP client");

        Self {
            client,
            streaming_client,
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
            models_path,
            chat_path,
            custom_headers,
            provider_config: None,
        }
    }
    
    pub fn new_with_provider_config(base_url: String, api_key: String, models_path: String, chat_path: String, custom_headers: std::collections::HashMap<String, String>, provider_config: crate::config::ProviderConfig) -> Self {
        // Create optimized HTTP client with connection pooling and keep-alive settings
        // This client keeps compression enabled for regular requests
        let client = Client::builder()
            .pool_max_idle_per_host(10) // Keep up to 10 idle connections per host
            .pool_idle_timeout(Duration::from_secs(90)) // Keep connections alive for 90 seconds
            .tcp_keepalive(Duration::from_secs(60)) // TCP keep-alive every 60 seconds
            .timeout(Duration::from_secs(60)) // Total request timeout
            .connect_timeout(Duration::from_secs(10)) // Connection establishment timeout
            .user_agent(concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("Failed to create optimized HTTP client");

        // Create a separate streaming-optimized client
        let streaming_client = Client::builder()
            .timeout(Duration::from_secs(300))  // Longer timeout for streaming
            .build()
            .expect("Failed to create streaming-optimized HTTP client");

        Self {
            client,
            streaming_client,
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
            models_path,
            chat_path,
            custom_headers,
            provider_config: Some(provider_config),
        }
    }
    
    /// Get the chat URL, handling both traditional paths and full URLs with model replacement
    fn get_chat_url(&self, model: &str) -> String {
        if let Some(ref config) = self.provider_config {
            // Use the provider config's URL generation method which handles template variables
            config.get_chat_url(model)
        } else {
            // Fallback to original logic for backward compatibility
            if self.chat_path.starts_with("https://") {
                // Full URL with model replacement
                self.chat_path.replace("{model_name}", model).replace("{model}", model)
            } else {
                // Traditional path-based approach
                format!("{}{}", self.base_url, self.chat_path)
            }
        }
    }
    
    /// Check if the URL is for Amazon Bedrock
    fn is_bedrock_url(&self, url: &str) -> bool {
        url.contains("bedrock") || url.contains("amazonaws.com")
    }
    
    /// Check if the URL is for Cloudflare
    fn is_cloudflare_url(&self, url: &str) -> bool {
        url.contains("api.cloudflare.com")
    }
    
    pub async fn chat(&self, request: &ChatRequest) -> Result<String> {
        let url = self.get_chat_url(&request.model);
        
        let mut req = self.client
            .post(&url)
            .header("Content-Type", "application/json");
        
        // Disable compression for streaming requests
        if request.stream == Some(true) {
            req = req.header("Accept-Encoding", "identity");
        }
        
        // Add Authorization header only if no custom headers are present
        // This allows providers like Gemini to use custom authentication headers
        if self.custom_headers.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", self.api_key));
        }
        
        // Add custom headers
        for (name, value) in &self.custom_headers {
            req = req.header(name, value);
        }
        
        // Check if we should exclude model from payload (when model is in URL path)
        let should_exclude_model = if let Some(ref config) = self.provider_config {
            config.chat_path.contains("{model}")
        } else {
            self.chat_path.contains("{model}")
        };
        
        // Use Bedrock-specific format if this is a Bedrock URL
        let response = if self.is_bedrock_url(&url) {
            let bedrock_request = BedrockChatRequest::from_chat_request(request);
            req.json(&bedrock_request).send().await?
        } else if should_exclude_model {
            // Use ChatRequestWithoutModel for providers like Cloudflare
            let request_without_model = ChatRequestWithoutModel::from(request);
            req.json(&request_without_model).send().await?
        } else {
            req.json(request).send().await?
        };
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("API request failed with status {}: {}", status, text);
        }
        
        // Get the response text first to handle different formats
        let response_text = response.text().await?;
        
        // Try to parse as Cloudflare format first (with "result" wrapper)
        if let Ok(cloudflare_response) = serde_json::from_str::<CloudflareChatResponse>(&response_text) {
            if cloudflare_response.success {
                return Ok(cloudflare_response.result.response);
            } else {
                anyhow::bail!("Cloudflare API request failed");
            }
        }
        
        // Try to parse as standard OpenAI format (with "choices" array)
        if let Ok(chat_response) = serde_json::from_str::<ChatResponse>(&response_text) {
            if let Some(choice) = chat_response.choices.first() {
                // Handle tool calls - check if tool_calls exists AND is not empty
                if let Some(tool_calls) = &choice.message.tool_calls {
                    if !tool_calls.is_empty() {
                        let mut response = String::new();
                        response.push_str("🔧 **Tool Calls Made:**\n\n");
                        
                        for tool_call in tool_calls {
                            response.push_str(&format!("**Function:** `{}`\n", tool_call.function.name));
                            response.push_str(&format!("**Arguments:** `{}`\n\n", tool_call.function.arguments));
                            
                            // Note: Tool execution is handled by the chat module's tool execution loop
                            response.push_str("*Tool calls detected - execution handled by chat module*\n\n");
                        }
                        
                        return Ok(response);
                    }
                    // If tool_calls is empty array, fall through to check content
                }
                
                // Handle content (either no tool_calls or empty tool_calls array)
                if let Some(content) = &choice.message.content {
                    return Ok(content.clone());
                } else {
                    anyhow::bail!("No content or tool calls in response");
                }
            } else {
                anyhow::bail!("No response from API");
            }
        }
        
        // Try to parse as Llama format (with "completion_message")
        if let Ok(llama_response) = serde_json::from_str::<LlamaChatResponse>(&response_text) {
            return Ok(llama_response.completion_message.content.text);
        }
        
        // Try to parse as Cohere format (with "message" and content array)
        if let Ok(cohere_response) = serde_json::from_str::<CohereChatResponse>(&response_text) {
            if let Some(content_item) = cohere_response.message.content.first() {
                return Ok(content_item.text.clone());
            } else {
                anyhow::bail!("No content in Cohere response");
            }
        }
        
        // Try to parse as Bedrock format (with "output" and nested message structure)
        if let Ok(bedrock_response) = serde_json::from_str::<BedrockChatResponse>(&response_text) {
            if let Some(content_part) = bedrock_response.output.message.content.first() {
                return Ok(content_part.text.clone());
            } else {
                anyhow::bail!("No content in Bedrock response");
            }
        }
        
        // If all fail, return an error with the response text for debugging
        anyhow::bail!("Failed to parse chat response. Response: {}", response_text);
    }
    
    pub async fn list_models(&self) -> Result<Vec<Model>> {
        let url = format!("{}{}", self.base_url, self.models_path);
        
        let mut req = self.client
            .get(&url)
            .header("Content-Type", "application/json");
        
        // Add Authorization header only if no custom headers are present
        if self.custom_headers.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", self.api_key));
        }
        
        // Add custom headers
        for (name, value) in &self.custom_headers {
            req = req.header(name, value);
        }
        
        let response = req
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("API request failed with status {}: {}", status, text);
        }
        
        // Get the response text first to handle different formats
        let response_text = response.text().await?;
        
        // Try to parse as ModelsResponse first (with "data" field)
        let models = if let Ok(models_response) = serde_json::from_str::<ModelsResponse>(&response_text) {
            models_response.data
        } else if let Ok(cohere_response) = serde_json::from_str::<CohereModelsResponse>(&response_text) {
            // Try to parse as CohereModelsResponse (with "models" field and "name" instead of "id")
            cohere_response.models.into_iter().map(|cohere_model| Model {
                id: cohere_model.name,
                object: cohere_model.object,
                providers: vec![],
            }).collect()
        } else if let Ok(parsed_models) = serde_json::from_str::<Vec<Model>>(&response_text) {
            // If that fails, try to parse as direct array of models
            parsed_models
        } else {
            // If all fail, return an error with the response text for debugging
            anyhow::bail!("Failed to parse models response. Response: {}", response_text);
        };
        
        // Expand models with providers into separate entries
        let mut expanded_models = Vec::new();
        
        for model in models {
            if model.providers.is_empty() {
                // No providers, add the model as-is
                expanded_models.push(model);
            } else {
                // Has providers, create a model entry for each provider
                for provider in &model.providers {
                    let expanded_model = Model {
                        id: format!("{}:{}", model.id, provider.provider),
                        object: model.object.clone(),
                        providers: vec![], // Clear providers for the expanded model
                    };
                    expanded_models.push(expanded_model);
                }
            }
            
        }
        
        Ok(expanded_models)
    }
    
    // New method that returns the full parsed response for tool handling
    pub async fn chat_with_tools(&self, request: &ChatRequest) -> Result<ChatResponse> {
        let url = self.get_chat_url(&request.model);
        
        let mut req = self.client
            .post(&url)
            .header("Content-Type", "application/json");
        
        // Disable compression for streaming requests
        if request.stream == Some(true) {
            req = req.header("Accept-Encoding", "identity");
        }
        
        // Add Authorization header only if no custom headers are present
        if self.custom_headers.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", self.api_key));
        }
        
        // Add custom headers
        for (name, value) in &self.custom_headers {
            req = req.header(name, value);
        }
        
        // Check if we should exclude model from payload (when model is in URL path)
        let should_exclude_model = if let Some(ref config) = self.provider_config {
            config.chat_path.contains("{model}")
        } else {
            self.chat_path.contains("{model}")
        };
        
        // Use Bedrock-specific format if this is a Bedrock URL
        let response = if self.is_bedrock_url(&url) {
            let bedrock_request = BedrockChatRequest::from_chat_request(request);
            req.json(&bedrock_request).send().await?
        } else if should_exclude_model {
            // Use ChatRequestWithoutModel for providers like Cloudflare
            let request_without_model = ChatRequestWithoutModel::from(request);
            req.json(&request_without_model).send().await?
        } else {
            req.json(request).send().await?
        };
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("API request failed with status {}: {}", status, text);
        }
        
        // Get the response text first to handle different formats
        let response_text = response.text().await?;
        
        // Try to parse as Cloudflare format first (with "result" wrapper)
        if let Ok(cloudflare_response) = serde_json::from_str::<CloudflareChatResponse>(&response_text) {
            if cloudflare_response.success {
                // Convert Cloudflare response to standard ChatResponse format
                let choice = Choice {
                    message: ResponseMessage {
                        role: "assistant".to_string(),
                        content: Some(cloudflare_response.result.response),
                        tool_calls: None,
                    },
                };
                return Ok(ChatResponse {
                    choices: vec![choice],
                });
            } else {
                anyhow::bail!("Cloudflare API request failed");
            }
        }
        
        // Try to parse as standard OpenAI format (with "choices" array)
        if let Ok(chat_response) = serde_json::from_str::<ChatResponse>(&response_text) {
            return Ok(chat_response);
        }
        
        // If parsing fails, return an error with the response text for debugging
        anyhow::bail!("Failed to parse chat response. Response: {}", response_text);
    }
    
    pub async fn get_token_from_url(&self, token_url: &str) -> Result<TokenResponse> {
        let mut req = self.client
            .get(token_url)
            .header("Authorization", format!("token {}", self.api_key))
            .header("Content-Type", "application/json");
        
        // Add custom headers
        for (name, value) in &self.custom_headers {
            req = req.header(name, value);
        }
        
        let response = req.send().await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Token request failed with status {}: {}", status, text);
        }
        
        let token_response: TokenResponse = response.json().await?;
        Ok(token_response)
    }
    
    pub async fn embeddings(&self, request: &EmbeddingRequest) -> Result<EmbeddingResponse> {
        let url = format!("{}/embeddings", self.base_url);
        
        let mut req = self.client
            .post(&url)
            .header("Content-Type", "application/json");
        
        // Add Authorization header only if no custom headers are present
        if self.custom_headers.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", self.api_key));
        }
        
        // Add custom headers
        for (name, value) in &self.custom_headers {
            req = req.header(name, value);
        }
        
        let response = req
            .json(request)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Embeddings API request failed with status {}: {}", status, text);
        }
        
        let embedding_response: EmbeddingResponse = response.json().await?;
        Ok(embedding_response)
    }
    
    pub async fn generate_images(&self, request: &ImageGenerationRequest) -> Result<ImageGenerationResponse> {
        let url = format!("{}/images/generations", self.base_url);
        
        let mut req = self.client
            .post(&url)
            .header("Content-Type", "application/json");
        
        // Add Authorization header only if no custom headers are present
        if self.custom_headers.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", self.api_key));
        }
        
        // Add custom headers
        for (name, value) in &self.custom_headers {
            req = req.header(name, value);
        }
        
        let response = req
            .json(request)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Image generation API request failed with status {}: {}", status, text);
        }
        
        let image_response: ImageGenerationResponse = response.json().await?;
        Ok(image_response)
    }
    
    pub async fn chat_stream(&self, request: &ChatRequest) -> Result<()> {
        use std::io::{stdout, Write};
        
        let url = self.get_chat_url(&request.model);
        
        // Use the streaming-optimized client for streaming requests
        let mut req = self.streaming_client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Accept", "text/event-stream")  // Explicitly request SSE format
            .header("Cache-Control", "no-cache")  // Prevent caching for streaming
            .header("Accept-Encoding", "identity");  // Explicitly request no compression
        
        // Wrap stdout in BufWriter for efficiency
        let stdout = stdout();
        let mut handle = std::io::BufWriter::new(stdout.lock());
        
        // Add Authorization header only if no custom headers are present
        if self.custom_headers.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", self.api_key));
        }
        
        // Add custom headers
        for (name, value) in &self.custom_headers {
            req = req.header(name, value);
        }
        
        // Check if we should exclude model from payload (when model is in URL path)
        let should_exclude_model = if let Some(ref config) = self.provider_config {
            config.chat_path.contains("{model}")
        } else {
            self.chat_path.contains("{model}")
        };
        
        // Use Bedrock-specific format if this is a Bedrock URL
        let response = if self.is_bedrock_url(&url) {
            let bedrock_request = BedrockChatRequest::from_chat_request(request);
            req.json(&bedrock_request).send().await?
        } else if should_exclude_model {
            // Use ChatRequestWithoutModel for providers like Cloudflare
            let request_without_model = ChatRequestWithoutModel::from(request);
            req.json(&request_without_model).send().await?
        } else {
            req.json(request).send().await?
        };
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("API request failed with status {}: {}", status, text);
        }
        
        // Check for compression headers (silent check for potential issues)
        let headers = response.headers();
        if headers.get("content-encoding").is_some() {
            // Content encoding detected - may cause buffering delays but continue silently
        }
        
        let mut stream = response.bytes_stream();
        
        let mut buffer = String::new();
        
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            
            let chunk_str = String::from_utf8_lossy(&chunk);
            buffer.push_str(&chunk_str);
            
            // Process complete lines from buffer
            while let Some(newline_pos) = buffer.find('\n') {
                let line = buffer[..newline_pos].to_string();
                buffer.drain(..=newline_pos);
                
                // Handle Server-Sent Events format
                if line.starts_with("data: ") {
                    let data = &line[6..]; // Remove "data: " prefix
                    
                    if data.trim() == "[DONE]" {
                        handle.write_all(b"\n")?;
                        handle.flush()?;
                        return Ok(());
                    }
                    
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                        // Try Cloudflare streaming format first (direct "response" field)
                        if let Some(response) = json.get("response") {
                            if let Some(text) = response.as_str() {
                                if !text.is_empty() {
                                    handle.write_all(text.as_bytes())?;
                                    handle.flush()?;
                                }
                            }
                        }
                        // Try standard OpenAI streaming format
                        else if let Some(choices) = json.get("choices") {
                            if let Some(choice) = choices.get(0) {
                                if let Some(delta) = choice.get("delta") {
                                    if let Some(content) = delta.get("content") {
                                        if let Some(text) = content.as_str() {
                                            // Write directly to stdout and flush immediately
                                            handle.write_all(text.as_bytes())?;
                                            handle.flush()?;
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else if line.trim().is_empty() {
                    // Skip empty lines in SSE format
                    continue;
                } else {
                    // Handle non-SSE format (direct JSON stream)
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&line) {
                        // Try Cloudflare streaming format first (direct "response" field)
                        if let Some(response) = json.get("response") {
                            if let Some(text) = response.as_str() {
                                if !text.is_empty() {
                                    handle.write_all(text.as_bytes())?;
                                    handle.flush()?;
                                }
                            }
                        }
                        // Try standard OpenAI streaming format
                        else if let Some(choices) = json.get("choices") {
                            if let Some(choice) = choices.get(0) {
                                if let Some(delta) = choice.get("delta") {
                                    if let Some(content) = delta.get("content") {
                                        if let Some(text) = content.as_str() {
                                            handle.write_all(text.as_bytes())?;
                                            handle.flush()?;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Process any remaining data in buffer
        if !buffer.trim().is_empty() {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&buffer) {
                // Try Cloudflare streaming format first (direct "response" field)
                if let Some(response) = json.get("response") {
                    if let Some(text) = response.as_str() {
                        if !text.is_empty() {
                            handle.write_all(text.as_bytes())?;
                            handle.flush()?;
                        }
                    }
                }
                // Try standard OpenAI streaming format
                else if let Some(choices) = json.get("choices") {
                    if let Some(choice) = choices.get(0) {
                        if let Some(delta) = choice.get("delta") {
                            if let Some(content) = delta.get("content") {
                                if let Some(text) = content.as_str() {
                                    handle.write_all(text.as_bytes())?;
                                    handle.flush()?;
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Add newline at the end
        handle.write_all(b"\n")?;
        handle.flush()?;
        Ok(())
    }
}

// Gemini-specific structures
#[derive(Debug, Serialize)]
pub struct GeminiChatRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_instruction: Option<GeminiSystemInstruction>,
    pub contents: Vec<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<GeminiTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_config: Option<GeminiGenerationConfig>,
}

#[derive(Debug, Serialize)]
pub struct GeminiSystemInstruction {
    pub parts: GeminiPart,
}

#[derive(Debug, Serialize)]
pub struct GeminiContent {
    pub role: String,
    pub parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum GeminiPart {
    Text { text: String },
    InlineData { inline_data: GeminiInlineData },
    FunctionCall { function_call: GeminiFunctionCall },
    FunctionResponse { function_response: GeminiFunctionResponse },
}

// New struct for Gemini inline image data
#[derive(Debug, Serialize)]
pub struct GeminiInlineData {
    pub mime_type: String,
    pub data: String, // base64 encoded
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeminiFunctionCall {
    pub name: String,
    pub args: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct GeminiFunctionResponse {
    pub name: String,
    pub response: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct GeminiTool {
    pub function_declarations: Vec<GeminiFunctionDeclaration>,
}

#[derive(Debug, Serialize)]
pub struct GeminiFunctionDeclaration {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct GeminiGenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct GeminiChatResponse {
    pub candidates: Vec<GeminiCandidate>,
}

#[derive(Debug, Deserialize)]
pub struct GeminiCandidate {
    pub content: GeminiResponseContent,
    #[serde(rename = "finishReason")]
    #[allow(dead_code)]
    pub finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GeminiResponseContent {
    pub parts: Vec<GeminiResponsePart>,
    pub role: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum GeminiResponsePart {
    Text { text: String },
    FunctionCall {
        #[serde(rename = "functionCall")]
        function_call: GeminiFunctionCall
    },
}

#[derive(Debug, Deserialize)]
pub struct GeminiModelsResponse {
    #[allow(dead_code)]
    pub models: Vec<GeminiModel>,
}

#[derive(Debug, Deserialize)]
pub struct GeminiModel {
    #[allow(dead_code)]
    pub name: String,
    #[serde(rename = "displayName")]
    #[allow(dead_code)]
    pub display_name: String,
    #[allow(dead_code)]
    pub description: Option<String>,
    #[serde(rename = "inputTokenLimit")]
    #[allow(dead_code)]
    pub input_token_limit: Option<u32>,
    #[serde(rename = "outputTokenLimit")]
    #[allow(dead_code)]
    pub output_token_limit: Option<u32>,
    #[serde(rename = "supportedGenerationMethods")]
    #[allow(dead_code)]
    pub supported_generation_methods: Vec<String>,
    #[allow(dead_code)]
    pub temperature: Option<f32>,
    #[serde(rename = "topP")]
    #[allow(dead_code)]
    pub top_p: Option<f32>,
    #[serde(rename = "topK")]
    #[allow(dead_code)]
    pub top_k: Option<u32>,
    #[serde(rename = "maxTemperature")]
    #[allow(dead_code)]
    pub max_temperature: Option<f32>,
}

pub struct GeminiClient {
    client: Client,
    base_url: String,
    #[allow(dead_code)]
    api_key: String,
    models_path: String,
    chat_path_template: String, // Template with <model> placeholder
    custom_headers: std::collections::HashMap<String, String>,
    provider_config: Option<crate::config::ProviderConfig>,
}

impl GeminiClient {
    #[allow(dead_code)]
    pub fn new(base_url: String, api_key: String, models_path: String, chat_path_template: String) -> Self {
        let client = Client::builder()
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(Duration::from_secs(90))
            .tcp_keepalive(Duration::from_secs(60))
            .timeout(Duration::from_secs(60))
            .connect_timeout(Duration::from_secs(10))
            .user_agent(concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("Failed to create optimized HTTP client");
        
        Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
            models_path,
            chat_path_template,
            custom_headers: std::collections::HashMap::new(),
            provider_config: None,
        }
    }
    
    #[allow(dead_code)]
    pub fn new_with_headers(base_url: String, api_key: String, models_path: String, chat_path_template: String, custom_headers: std::collections::HashMap<String, String>) -> Self {
        let client = Client::builder()
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(Duration::from_secs(90))
            .tcp_keepalive(Duration::from_secs(60))
            .timeout(Duration::from_secs(60))
            .connect_timeout(Duration::from_secs(10))
            .user_agent(concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("Failed to create optimized HTTP client");
        
        Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
            models_path,
            chat_path_template,
            custom_headers,
            provider_config: None,
        }
    }
    
    pub fn new_with_provider_config(base_url: String, api_key: String, models_path: String, chat_path_template: String, custom_headers: std::collections::HashMap<String, String>, provider_config: crate::config::ProviderConfig) -> Self {
        let client = Client::builder()
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(Duration::from_secs(90))
            .tcp_keepalive(Duration::from_secs(60))
            .timeout(Duration::from_secs(60))
            .connect_timeout(Duration::from_secs(10))
            .user_agent(concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("Failed to create optimized HTTP client");
        
        Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
            models_path,
            chat_path_template,
            custom_headers,
            provider_config: Some(provider_config),
        }
    }
    
    pub async fn chat(&self, request: &GeminiChatRequest, model: &str) -> Result<String> {
        // Use provider config for URL generation if available, otherwise fallback to template replacement
        let url = if let Some(ref config) = self.provider_config {
            let generated_url = config.get_chat_url(model);
            crate::debug_log!("GeminiClient: Using provider config URL generation, generated URL: {}", generated_url);
            generated_url
        } else {
            // Replace <model> placeholder in chat path
            let chat_path = self.chat_path_template.replace("<model>", model);
            let fallback_url = format!("{}{}", self.base_url, chat_path);
            crate::debug_log!("GeminiClient: Using fallback URL generation, generated URL: {}", fallback_url);
            fallback_url
        };
        
        let mut req = self.client
            .post(&url)
            .header("Content-Type", "application/json");
        
        // Add custom headers (including x-goog-api-key from config)
        for (name, value) in &self.custom_headers {
            req = req.header(name, value);
        }
        
        let response = req
            .json(request)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Gemini API request failed with status {}: {}", status, text);
        }
        
        let response_text = response.text().await?;
        let gemini_response: GeminiChatResponse = serde_json::from_str(&response_text)?;
        
        if let Some(candidate) = gemini_response.candidates.first() {
            // Handle tool calls
            for part in &candidate.content.parts {
                if let GeminiResponsePart::FunctionCall { function_call } = part {
                    let mut response = String::new();
                    response.push_str("🔧 **Tool Calls Made:**\n\n");
                    response.push_str(&format!("**Function:** `{}`\n", function_call.name));
                    response.push_str(&format!("**Arguments:** `{}`\n\n", serde_json::to_string(&function_call.args)?));
                    response.push_str("*Tool calls detected - execution handled by chat module*\n\n");
                    return Ok(response);
                }
            }
            
            // Handle text content
            for part in &candidate.content.parts {
                if let GeminiResponsePart::Text { text } = part {
                    return Ok(text.clone());
                }
            }
            
            anyhow::bail!("No text content in Gemini response");
        } else {
            anyhow::bail!("No candidates in Gemini response");
        }
    }
    
    pub async fn chat_with_tools(&self, request: &GeminiChatRequest, model: &str) -> Result<GeminiChatResponse> {
        // Use provider config for URL generation if available, otherwise fallback to template replacement
        let url = if let Some(ref config) = self.provider_config {
            let generated_url = config.get_chat_url(model);
            crate::debug_log!("GeminiClient: Using provider config URL generation for tools, generated URL: {}", generated_url);
            generated_url
        } else {
            let chat_path = self.chat_path_template.replace("<model>", model);
            let fallback_url = format!("{}{}", self.base_url, chat_path);
            crate::debug_log!("GeminiClient: Using fallback URL generation for tools, generated URL: {}", fallback_url);
            fallback_url
        };
        
        let mut req = self.client
            .post(&url)
            .header("Content-Type", "application/json");
        
        // Add custom headers (including x-goog-api-key from config)
        for (name, value) in &self.custom_headers {
            req = req.header(name, value);
        }
        
        let response = req
            .json(request)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Gemini API request failed with status {}: {}", status, text);
        }
        
        let response_text = response.text().await?;
        let gemini_response: GeminiChatResponse = serde_json::from_str(&response_text)?;
        Ok(gemini_response)
    }
    
    pub async fn list_models(&self) -> Result<Vec<Model>> {
        let url = format!("{}{}", self.base_url, self.models_path);
        
        let mut req = self.client
            .get(&url)
            .header("Content-Type", "application/json");
        
        // Add custom headers (including x-goog-api-key from config)
        for (name, value) in &self.custom_headers {
            req = req.header(name, value);
        }
        
        let response = req
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Gemini API request failed with status {}: {}", status, text);
        }
        
        let response_text = response.text().await?;
        
        // Use the rich metadata extraction system
        use crate::model_metadata::MetadataExtractor;
        let metadata_models = MetadataExtractor::extract_from_provider("gemini", &response_text)
            .map_err(|e| anyhow::anyhow!("Failed to extract Gemini model metadata: {}", e))?;
        
        // Convert ModelMetadata back to Model format for compatibility
        let models = metadata_models.into_iter().map(|metadata| {
            let provider = Provider {
                provider: "gemini".to_string(),
                status: "active".to_string(),
                supports_tools: metadata.supports_tools,
                supports_structured_output: metadata.supports_json_mode,
            };
            
            Model {
                id: format!("{}:{}", metadata.id, "gemini"), // Format as model:provider for consistency
                object: "model".to_string(),
                providers: vec![provider],
            }
        }).collect();
        
        Ok(models)
    }
    
    pub async fn chat_stream(&self, request: &GeminiChatRequest, model: &str) -> Result<()> {
        // For now, Gemini streaming will fall back to regular chat
        // This can be enhanced later with proper Gemini streaming support
        let response = self.chat(request, model).await?;
        print!("{}", response);
        io::stdout().flush()?;
        println!();
        Ok(())
    }
}