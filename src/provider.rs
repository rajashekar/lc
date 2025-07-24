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
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    pub choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
pub struct Choice {
    pub message: Message,
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

pub struct OpenAIClient {
    client: Client,
    base_url: String,
    api_key: String,
    models_path: String,
    chat_path: String,
    custom_headers: std::collections::HashMap<String, String>,
}

impl OpenAIClient {
    pub fn new(base_url: String, api_key: String) -> Self {
        Self::new_with_paths(base_url, api_key, "/models".to_string(), "/chat/completions".to_string())
    }
    
    pub fn new_with_paths(base_url: String, api_key: String, models_path: String, chat_path: String) -> Self {
        Self::new_with_headers(base_url, api_key, models_path, chat_path, std::collections::HashMap::new())
    }
    
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
                return Ok(choice.message.content.clone());
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
}