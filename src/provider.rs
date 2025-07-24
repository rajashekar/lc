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
pub struct ModelsResponse {
    pub data: Vec<Model>,
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
}

impl OpenAIClient {
    pub fn new(base_url: String, api_key: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .expect("Failed to create HTTP client");
        
        Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
        }
    }
    
    pub async fn chat(&self, request: &ChatRequest) -> Result<String> {
        let url = format!("{}/chat/completions", self.base_url);
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(request)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("API request failed with status {}: {}", status, text);
        }
        
        let chat_response: ChatResponse = response.json().await?;
        
        if let Some(choice) = chat_response.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            anyhow::bail!("No response from API");
        }
    }
    
    pub async fn list_models(&self) -> Result<Vec<Model>> {
        let url = format!("{}/models", self.base_url);
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("API request failed with status {}: {}", status, text);
        }
        
        // Get the response text first to handle different formats
        let response_text = response.text().await?;
        
        let mut models = Vec::new();
        
        // Try to parse as ModelsResponse first (with "data" field)
        if let Ok(models_response) = serde_json::from_str::<ModelsResponse>(&response_text) {
            models = models_response.data;
        } else if let Ok(parsed_models) = serde_json::from_str::<Vec<Model>>(&response_text) {
            // If that fails, try to parse as direct array of models
            models = parsed_models;
        } else {
            // If both fail, return an error with the response text for debugging
            anyhow::bail!("Failed to parse models response. Response: {}", response_text);
        }
        
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