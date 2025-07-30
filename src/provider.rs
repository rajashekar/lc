use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
}

#[derive(Debug, Serialize)]
pub struct EmbeddingRequest {
    pub model: String,
    pub input: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding_format: Option<String>,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl Message {
    pub fn user(content: String) -> Self {
        Self {
            role: "user".to_string(),
            content: Some(content),
            tool_calls: None,
            tool_call_id: None,
        }
    }
    
    pub fn assistant(content: String) -> Self {
        Self {
            role: "assistant".to_string(),
            content: Some(content),
            tool_calls: None,
            tool_call_id: None,
        }
    }
    
    pub fn assistant_with_tool_calls(tool_calls: Vec<ToolCall>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: None,
            tool_calls: Some(tool_calls),
            tool_call_id: None,
        }
    }
    
    pub fn tool_result(tool_call_id: String, content: String) -> Self {
        Self {
            role: "tool".to_string(),
            content: Some(content),
            tool_calls: None,
            tool_call_id: Some(tool_call_id),
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
    base_url: String,
    api_key: String,
    models_path: String,
    chat_path: String,
    custom_headers: std::collections::HashMap<String, String>,
}

impl OpenAIClient {
    
    pub fn new_with_headers(base_url: String, api_key: String, models_path: String, chat_path: String, custom_headers: std::collections::HashMap<String, String>) -> Self {
        // Create optimized HTTP client with connection pooling and keep-alive settings
        let client = Client::builder()
            .pool_max_idle_per_host(10) // Keep up to 10 idle connections per host
            .pool_idle_timeout(Duration::from_secs(90)) // Keep connections alive for 90 seconds
            .tcp_keepalive(Duration::from_secs(60)) // TCP keep-alive every 60 seconds
            .timeout(Duration::from_secs(60)) // Total request timeout
            .connect_timeout(Duration::from_secs(10)) // Connection establishment timeout
            .user_agent(concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("Failed to create optimized HTTP client");
        
        Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
            models_path,
            chat_path,
            custom_headers,
        }
    }
    
    pub async fn chat(&self, request: &ChatRequest) -> Result<String> {
        let url = format!("{}{}", self.base_url, self.chat_path);
        
        let mut req = self.client
            .post(&url)
            .header("Content-Type", "application/json");
        
        // Add Authorization header only if no custom headers are present
        // This allows providers like Gemini to use custom authentication headers
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
            anyhow::bail!("API request failed with status {}: {}", status, text);
        }
        
        // Get the response text first to handle different formats
        let response_text = response.text().await?;
        
        // Try to parse as standard OpenAI format first (with "choices" array)
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
        let url = format!("{}{}", self.base_url, self.chat_path);
        
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
            anyhow::bail!("API request failed with status {}: {}", status, text);
        }
        
        // Get the response text first to handle different formats
        let response_text = response.text().await?;
        
        // Try to parse as standard OpenAI format first (with "choices" array)
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
    FunctionCall { function_call: GeminiFunctionCall },
    FunctionResponse { function_response: GeminiFunctionResponse },
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
    pub models: Vec<GeminiModel>,
}

#[derive(Debug, Deserialize)]
pub struct GeminiModel {
    pub name: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub description: Option<String>,
    #[serde(rename = "inputTokenLimit")]
    pub input_token_limit: Option<u32>,
    #[serde(rename = "outputTokenLimit")]
    pub output_token_limit: Option<u32>,
    #[serde(rename = "supportedGenerationMethods")]
    pub supported_generation_methods: Vec<String>,
    pub temperature: Option<f32>,
    #[serde(rename = "topP")]
    pub top_p: Option<f32>,
    #[serde(rename = "topK")]
    pub top_k: Option<u32>,
    #[serde(rename = "maxTemperature")]
    pub max_temperature: Option<f32>,
}

pub struct GeminiClient {
    client: Client,
    base_url: String,
    api_key: String,
    models_path: String,
    chat_path_template: String, // Template with <model> placeholder
    custom_headers: std::collections::HashMap<String, String>,
}

impl GeminiClient {
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
        }
    }
    
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
        }
    }
    
    pub async fn chat(&self, request: &GeminiChatRequest, model: &str) -> Result<String> {
        // Replace <model> placeholder in chat path
        let chat_path = self.chat_path_template.replace("<model>", model);
        let url = format!("{}{}", self.base_url, chat_path);
        
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
        let chat_path = self.chat_path_template.replace("<model>", model);
        let url = format!("{}{}", self.base_url, chat_path);
        
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
}