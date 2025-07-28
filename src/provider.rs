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
    pub model: String,
    pub usage: EmbeddingUsage,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EmbeddingData {
    pub embedding: Vec<f64>,
    pub index: u32,
    pub object: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EmbeddingUsage {
    pub prompt_tokens: u32,
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
    pub role: String,
    pub content: LlamaContent,
}

#[derive(Debug, Deserialize)]
pub struct LlamaContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct CohereChatResponse {
    pub message: CohereMessage,
}

#[derive(Debug, Deserialize)]
pub struct CohereMessage {
    pub role: String,
    pub content: Vec<CohereContentItem>,
}

#[derive(Debug, Deserialize)]
pub struct CohereContentItem {
    #[serde(rename = "type")]
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
    pub status: String,
    #[serde(default)]
    pub supports_tools: bool,
    #[serde(default)]
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
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .expect("Failed to create HTTP client");
        
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
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json");
        
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
                            
                            // TODO: Actually execute the MCP function call here
                            response.push_str("*Note: Tool execution not yet implemented - this shows the LLM successfully recognized and attempted to use the MCP tools.*\n\n");
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
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json");
        
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
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json");
        
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
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json");
        
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