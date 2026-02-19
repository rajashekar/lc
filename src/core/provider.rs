use anyhow::Result;
use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::template_processor::TemplateProcessor;

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

#[derive(Debug, Serialize)]
pub struct AudioTranscriptionRequest {
    pub file: String, // Base64 encoded audio or URL
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<String>, // json, text, srt, verbose_json, vtt
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
}

#[derive(Debug, Deserialize)]
pub struct AudioTranscriptionResponse {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[allow(dead_code)]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[allow(dead_code)]
    pub duration: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[allow(dead_code)]
    pub segments: Option<Vec<TranscriptionSegment>>,
}

#[derive(Debug, Deserialize)]
pub struct TranscriptionSegment {
    #[allow(dead_code)]
    pub id: i32,
    #[allow(dead_code)]
    pub start: f32,
    #[allow(dead_code)]
    pub end: f32,
    #[allow(dead_code)]
    pub text: String,
}

#[derive(Debug, Serialize)]
pub struct AudioSpeechRequest {
    pub model: String, // tts-1, tts-1-hd
    pub input: String, // Text to convert to speech
    pub voice: String, // alloy, echo, fable, onyx, nova, shimmer
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<String>, // mp3, opus, aac, flac, wav, pcm
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<f32>, // 0.25 to 4.0
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
    Text { content: Option<String> },
    Multimodal { content: Vec<ContentPart> },
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
            content_type: MessageContent::Text {
                content: Some(content),
            },
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
            content_type: MessageContent::Text {
                content: Some(content),
            },
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
            content_type: MessageContent::Text {
                content: Some(content),
            },
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
pub struct ModelsResponse {
    #[serde(alias = "models")]
    pub data: Vec<Model>,
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
    streaming_client: Client, // Separate client optimized for streaming
    base_url: String,
    api_key: String,
    models_path: String,
    chat_path: String,
    custom_headers: std::collections::HashMap<String, String>,
    provider_config: Option<crate::config::ProviderConfig>,
    template_processor: Option<TemplateProcessor>,
}

impl OpenAIClient {
    /// Creates a new OpenAI client with optional provider configuration
    /// This is the unified factory method that consolidates HTTP client creation logic
    pub fn create_http_client(
        base_url: String,
        api_key: String,
        models_path: String,
        chat_path: String,
        custom_headers: std::collections::HashMap<String, String>,
        provider_config: Option<crate::config::ProviderConfig>,
    ) -> Result<Self> {
        // Create default headers including the required tracking headers
        let default_headers = Self::create_default_headers();

        // Create optimized HTTP client with connection pooling and keep-alive settings
        let client = Self::build_http_client(default_headers.clone(), Duration::from_secs(60))?;

        // Create a separate streaming-optimized client with longer timeout
        let streaming_client = Self::build_http_client(default_headers, Duration::from_secs(300))?;

        // Create template processor if provider config has templates
        let template_processor = provider_config
            .as_ref()
            .and_then(Self::create_template_processor);

        Ok(Self {
            client,
            streaming_client,
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
            models_path,
            chat_path,
            custom_headers,
            provider_config,
            template_processor,
        })
    }

    /// Legacy method for backward compatibility - delegates to create_http_client
    pub fn new_with_headers(
        base_url: String,
        api_key: String,
        models_path: String,
        chat_path: String,
        custom_headers: std::collections::HashMap<String, String>,
    ) -> Self {
        Self::create_http_client(
            base_url,
            api_key,
            models_path,
            chat_path,
            custom_headers,
            None,
        )
        .expect("Failed to create OpenAI client")
    }

    /// Legacy method for backward compatibility - delegates to create_http_client
    pub fn new_with_provider_config(
        base_url: String,
        api_key: String,
        models_path: String,
        chat_path: String,
        custom_headers: std::collections::HashMap<String, String>,
        provider_config: crate::config::ProviderConfig,
    ) -> Self {
        Self::create_http_client(
            base_url,
            api_key,
            models_path,
            chat_path,
            custom_headers,
            Some(provider_config),
        )
        .expect("Failed to create OpenAI client with provider config")
    }

    /// Creates the default headers for all HTTP clients
    fn create_default_headers() -> reqwest::header::HeaderMap {
        use reqwest::header::{HeaderName, HeaderValue};

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            HeaderName::from_static("http-referer"),
            HeaderValue::from_static("https://lc.viwq.dev/"),
        );
        headers.insert(
            HeaderName::from_static("x-title"),
            HeaderValue::from_static("lc"),
        );
        headers
    }

    /// Builds an HTTP client with the specified configuration
    fn build_http_client(
        default_headers: reqwest::header::HeaderMap,
        timeout: Duration,
    ) -> Result<Client> {
        let mut builder = Client::builder()
            .pool_max_idle_per_host(10) // Keep up to 10 idle connections per host
            .pool_idle_timeout(Duration::from_secs(90)) // Keep connections alive for 90 seconds
            .tcp_keepalive(Duration::from_secs(60)) // TCP keep-alive every 60 seconds
            .timeout(timeout)
            .connect_timeout(Duration::from_secs(10)) // Connection establishment timeout
            .user_agent(concat!(
                env!("CARGO_PKG_NAME"),
                "/",
                env!("CARGO_PKG_VERSION")
            ))
            .default_headers(default_headers);

        // Disable certificate verification for development/debugging (e.g., with Proxyman)
        if std::env::var("LC_DISABLE_TLS_VERIFY").is_ok() {
            builder = builder.danger_accept_invalid_certs(true);
        }

        builder
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to create HTTP client: {}", e))
    }

    /// Creates a template processor if any templates are configured
    fn create_template_processor(
        config: &crate::config::ProviderConfig,
    ) -> Option<TemplateProcessor> {
        let has_templates = config.chat_templates.is_some()
            || config.images_templates.is_some()
            || config.embeddings_templates.is_some()
            || config.models_templates.is_some()
            || config.speech_templates.is_some();

        if has_templates {
            match TemplateProcessor::new() {
                Ok(processor) => Some(processor),
                Err(e) => {
                    eprintln!("Warning: Failed to create template processor: {}", e);
                    None
                }
            }
        } else {
            None
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
                self.chat_path
                    .replace("{model_name}", model)
                    .replace("{model}", model)
            } else {
                // Traditional path-based approach
                format!("{}{}", self.base_url, self.chat_path)
            }
        }
    }

    /// Helper method to build URLs with optional model replacement
    fn build_url(&self, endpoint_type: &str, model: &str, default_path: &str) -> String {
        match endpoint_type {
            "models" => format!("{}{}", self.base_url, self.models_path),
            "embeddings" => {
                if let Some(ref config) = self.provider_config {
                    config.get_embeddings_url(model)
                } else {
                    format!("{}/embeddings", self.base_url)
                }
            }
            "images" => {
                if let Some(ref config) = self.provider_config {
                    config.get_images_url(model)
                } else {
                    format!("{}/images/generations", self.base_url)
                }
            }
            "audio_transcriptions" => {
                if let Some(ref config) = self.provider_config {
                    format!(
                        "{}{}",
                        self.base_url,
                        config
                            .audio_path
                            .as_deref()
                            .unwrap_or("/audio/transcriptions")
                    )
                } else {
                    format!("{}/audio/transcriptions", self.base_url)
                }
            }
            "audio_speech" => {
                if let Some(ref config) = self.provider_config {
                    config.get_speech_url(model)
                } else {
                    format!("{}/audio/speech", self.base_url)
                }
            }
            _ => {
                // Generic endpoint building
                format!("{}{}", self.base_url, default_path)
            }
        }
    }

    /// Helper method to add standard headers to a request builder
    fn add_standard_headers(&self, mut req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        // Add Authorization header unless a custom Authorization header is already present
        if !self.custom_headers.contains_key("Authorization")
            && !self.custom_headers.contains_key("authorization")
        {
            req = req.header("Authorization", format!("Bearer {}", self.api_key));
        }

        // Add custom headers
        for (name, value) in &self.custom_headers {
            req = req.header(name, value);
        }

        req
    }

    pub async fn chat(&self, request: &ChatRequest) -> Result<String> {
        let url = self.get_chat_url(&request.model);

        let mut req = self
            .client
            .post(&url)
            .header("Content-Type", "application/json");

        // Disable compression for streaming requests
        if request.stream == Some(true) {
            req = req.header("Accept-Encoding", "identity");
        }

        // Add Authorization header unless a custom Authorization header is already present
        // This allows providers like Gemini to use custom authentication headers
        if !self.custom_headers.contains_key("Authorization")
            && !self.custom_headers.contains_key("authorization")
        {
            req = req.header("Authorization", format!("Bearer {}", self.api_key));
        }

        // Add custom headers
        for (name, value) in &self.custom_headers {
            req = req.header(name, value);
        }

        // Check if we have a template for this provider/model/endpoint
        let request_body = if let Some(ref config) = &self.provider_config {
            if let Some(ref processor) = &self.template_processor {
                // Get template for chat endpoint
                let template = config.get_endpoint_template("chat", &request.model);

                if let Some(template_str) = template {
                    // Clone the processor to avoid mutable borrow issues
                    let mut processor_clone = processor.clone();
                    // Use template to transform request
                    match processor_clone.process_request(request, &template_str, &config.vars) {
                        Ok(json_value) => Some(json_value),
                        Err(e) => {
                            eprintln!("Warning: Failed to process request template: {}. Falling back to default.", e);
                            None
                        }
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        // Send request with template-processed body or fall back to default logic
        let response = if let Some(json_body) = request_body {
            req.json(&json_body).send().await?
        } else {
            // Fall back to existing logic
            // Check if we should exclude model from payload (when model is in URL path)
            let should_exclude_model = if let Some(ref config) = self.provider_config {
                config.chat_path.contains("{model}")
            } else {
                self.chat_path.contains("{model}")
            };

            if should_exclude_model {
                // Use ChatRequestWithoutModel for providers that specify model in URL
                let request_without_model = ChatRequestWithoutModel::from(request);
                req.json(&request_without_model).send().await?
            } else {
                req.json(request).send().await?
            }
        };

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("API request failed with status {}: {}", status, text);
        }

        // Get the response text first to handle different formats
        let response_text = response.text().await?;

        // Check if we have a response template for this provider/model/endpoint
        if let Some(ref config) = &self.provider_config {
            if let Some(ref processor) = &self.template_processor {
                // Get response template for chat endpoint
                let template = config.get_endpoint_response_template("chat", &request.model);

                if let Some(template_str) = template {
                    // Parse response as JSON
                    if let Ok(response_json) =
                        serde_json::from_str::<serde_json::Value>(&response_text)
                    {
                        // Clone the processor to avoid mutable borrow issues
                        let mut processor_clone = processor.clone();
                        // Use template to extract content
                        match processor_clone.process_response(&response_json, &template_str) {
                            Ok(extracted) => {
                                // Extract content from the template result
                                if let Some(content) =
                                    extracted.get("content").and_then(|v| v.as_str())
                                {
                                    return Ok(content.to_string());
                                } else if let Some(tool_calls) =
                                    extracted.get("tool_calls").and_then(|v| v.as_array())
                                {
                                    if !tool_calls.is_empty() {
                                        let mut response = String::new();
                                        response.push_str("🔧 **Tool Calls Made:**\n\n");
                                        response
                                            .push_str(&format!("Tool calls: {:?}\n\n", tool_calls));
                                        response.push_str("*Tool calls detected - execution handled by chat module*\n\n");
                                        return Ok(response);
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("Warning: Failed to process response template: {}. Falling back to default parsing.", e);
                            }
                        }
                    }
                }
            }
        }

        // Fall back to existing parsing logic
        // Try to parse as standard OpenAI format (with "choices" array)
        if let Ok(chat_response) = serde_json::from_str::<ChatResponse>(&response_text) {
            if let Some(choice) = chat_response.choices.first() {
                // Handle tool calls - check if tool_calls exists AND is not empty
                if let Some(tool_calls) = &choice.message.tool_calls {
                    if !tool_calls.is_empty() {
                        let mut response = String::new();
                        response.push_str("🔧 **Tool Calls Made:**\n\n");

                        for tool_call in tool_calls {
                            response.push_str(&format!(
                                "**Function:** `{}`\n",
                                tool_call.function.name
                            ));
                            response.push_str(&format!(
                                "**Arguments:** `{}`\n\n",
                                tool_call.function.arguments
                            ));

                            // Note: Tool execution is handled by the chat module's tool execution loop
                            response.push_str(
                                "*Tool calls detected - execution handled by chat module*\n\n",
                            );
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

        // If all fail, return an error with the response text for debugging
        anyhow::bail!("Failed to parse chat response. Response: {}", response_text);
    }

    pub async fn list_models(&self) -> Result<Vec<Model>> {
        let url = format!("{}{}", self.base_url, self.models_path);

        // Debug log the URL being requested
        crate::debug_log!("Requesting models from URL: {}", url);

        let mut req = self
            .client
            .get(&url)
            .header("Content-Type", "application/json");

        // Add standard headers using helper method
        req = self.add_standard_headers(req);

        let response = req.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            crate::debug_log!("API request failed with status {}: {}", status, text);
            anyhow::bail!("API request failed with status {}: {}", status, text);
        }

        // Get the response text first to handle different formats
        let response_text = response.text().await?;

        // Debug log the full response
        crate::debug_log!(
            "Received models response ({} bytes): {}",
            response_text.len(),
            response_text
        );

        // Try to parse as ModelsResponse first (with "data" field)
        let models = if let Ok(models_response) =
            serde_json::from_str::<ModelsResponse>(&response_text)
        {
            models_response.data
        } else if let Ok(parsed_models) = serde_json::from_str::<Vec<Model>>(&response_text) {
            // If that fails, try to parse as direct array of models
            parsed_models
        } else {
            // Try to parse as Gemini format with "models" field containing different structure
            if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&response_text) {
                if let Some(models_array) = json_value.get("models").and_then(|v| v.as_array()) {
                    // Convert Gemini models to our Model struct
                    let mut converted_models = Vec::new();
                    for model_json in models_array {
                        // Extract the model name/id - Gemini uses "name" field like "models/gemini-1.5-pro"
                        if let Some(name) = model_json.get("name").and_then(|v| v.as_str()) {
                            // Remove "models/" prefix if present
                            let id = name.strip_prefix("models/").unwrap_or(name);

                            converted_models.push(Model {
                                id: id.to_string(),
                                object: "model".to_string(),
                                providers: vec![], // Gemini doesn't have providers field
                            });
                        }
                    }

                    if !converted_models.is_empty() {
                        crate::debug_log!(
                            "Successfully parsed {} Gemini models",
                            converted_models.len()
                        );
                        converted_models
                    } else {
                        anyhow::bail!(
                            "Failed to parse models response. Response: {}",
                            response_text
                        );
                    }
                } else {
                    // If all fail, return an error with the response text for debugging
                    anyhow::bail!(
                        "Failed to parse models response. Response: {}",
                        response_text
                    );
                }
            } else {
                anyhow::bail!(
                    "Failed to parse models response. Response: {}",
                    response_text
                );
            }
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

        let mut req = self
            .client
            .post(&url)
            .header("Content-Type", "application/json");

        // Disable compression for streaming requests
        if request.stream == Some(true) {
            req = req.header("Accept-Encoding", "identity");
        }

        // Add Authorization header unless a custom Authorization header is already present
        if !self.custom_headers.contains_key("Authorization")
            && !self.custom_headers.contains_key("authorization")
        {
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

        let response = if should_exclude_model {
            // Use ChatRequestWithoutModel for providers that specify model in URL
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

        // Try to parse as standard OpenAI format (with "choices" array)
        if let Ok(chat_response) = serde_json::from_str::<ChatResponse>(&response_text) {
            return Ok(chat_response);
        }

        // If parsing fails, return an error with the response text for debugging
        anyhow::bail!("Failed to parse chat response. Response: {}", response_text);
    }

    pub async fn get_token_from_url(&self, token_url: &str) -> Result<TokenResponse> {
        let mut req = self
            .client
            .get(token_url)
            .header("Authorization", format!("token {}", self.api_key))
            .header("Content-Type", "application/json");

        // Add custom headers only (this method uses token auth instead of Bearer)
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
        // Use helper method to build URL
        let url = self.build_url("embeddings", &request.model, "/embeddings");

        let mut req = self
            .client
            .post(&url)
            .header("Content-Type", "application/json");

        // Add standard headers using helper method
        req = self.add_standard_headers(req);

        // Check if we have a template for this provider/model/endpoint
        let request_body = if let Some(ref config) = &self.provider_config {
            if let Some(ref processor) = &self.template_processor {
                // Get template for embeddings endpoint
                let template = config.get_endpoint_template("embeddings", &request.model);

                if let Some(template_str) = template {
                    // Clone the processor to avoid mutable borrow issues
                    let mut processor_clone = processor.clone();
                    // Use template to transform request
                    match processor_clone.process_embeddings_request(
                        request,
                        &template_str,
                        &config.vars,
                    ) {
                        Ok(json_value) => Some(json_value),
                        Err(e) => {
                            eprintln!("Warning: Failed to process embeddings request template: {}. Falling back to default.", e);
                            None
                        }
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        // Send request with template-processed body or fall back to default logic
        let response = if let Some(json_body) = request_body {
            req.json(&json_body).send().await?
        } else {
            req.json(request).send().await?
        };

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!(
                "Embeddings API request failed with status {}: {}",
                status,
                text
            );
        }

        // Get the response text first to handle different formats
        let response_text = response.text().await?;

        // Check if we have a response template for this provider/model/endpoint
        if let Some(ref config) = &self.provider_config {
            if let Some(ref processor) = &self.template_processor {
                // Get response template for embeddings endpoint
                let template = config.get_endpoint_response_template("embeddings", &request.model);

                if let Some(template_str) = template {
                    // Parse response as JSON
                    if let Ok(response_json) =
                        serde_json::from_str::<serde_json::Value>(&response_text)
                    {
                        // Clone the processor to avoid mutable borrow issues
                        let mut processor_clone = processor.clone();
                        // Use template to transform response
                        match processor_clone.process_response(&response_json, &template_str) {
                            Ok(transformed) => {
                                // Try to parse the transformed response as EmbeddingResponse
                                if let Ok(embedding_response) =
                                    serde_json::from_value::<EmbeddingResponse>(transformed)
                                {
                                    return Ok(embedding_response);
                                }
                            }
                            Err(e) => {
                                eprintln!("Warning: Failed to process embeddings response template: {}. Falling back to default parsing.", e);
                            }
                        }
                    }
                }
            }
        }

        // Fall back to default parsing
        let embedding_response: EmbeddingResponse = serde_json::from_str(&response_text)?;
        Ok(embedding_response)
    }

    pub async fn generate_images(
        &self,
        request: &ImageGenerationRequest,
    ) -> Result<ImageGenerationResponse> {
        // Use helper method to build URL
        let model_name = request.model.as_deref().unwrap_or("");
        let url = self.build_url("images", model_name, "/images/generations");

        let mut req = self
            .client
            .post(&url)
            .header("Content-Type", "application/json");

        // Add standard headers using helper method
        req = self.add_standard_headers(req);

        // Check if we have a template for this provider/model/endpoint
        let request_body = if let Some(ref config) = &self.provider_config {
            if let Some(ref processor) = &self.template_processor {
                // Get template for images endpoint
                let model_name = request.model.as_deref().unwrap_or("");
                let template = config.get_endpoint_template("images", model_name);

                if let Some(template_str) = template {
                    // Clone the processor to avoid mutable borrow issues
                    let mut processor_clone = processor.clone();
                    // Use template to transform request
                    match processor_clone.process_image_request(
                        request,
                        &template_str,
                        &config.vars,
                    ) {
                        Ok(json_value) => Some(json_value),
                        Err(e) => {
                            eprintln!("Warning: Failed to process image request template: {}. Falling back to default.", e);
                            None
                        }
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        // Send request with template-processed body or fall back to default logic
        let response = if let Some(json_body) = request_body {
            req.json(&json_body).send().await?
        } else {
            req.json(request).send().await?
        };

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!(
                "Image generation API request failed with status {}: {}",
                status,
                text
            );
        }

        // Get the response text first to handle different formats
        let response_text = response.text().await?;

        // Check if we have a response template for this provider/model/endpoint
        if let Some(ref config) = &self.provider_config {
            if let Some(ref processor) = &self.template_processor {
                // Get response template for images endpoint
                let model_name = request.model.as_deref().unwrap_or("");
                let template = config.get_endpoint_response_template("images", model_name);

                if let Some(template_str) = template {
                    // Parse response as JSON
                    if let Ok(response_json) =
                        serde_json::from_str::<serde_json::Value>(&response_text)
                    {
                        // Clone the processor to avoid mutable borrow issues
                        let mut processor_clone = processor.clone();
                        // Use template to transform response
                        match processor_clone.process_response(&response_json, &template_str) {
                            Ok(transformed) => {
                                // Try to parse the transformed response as ImageGenerationResponse
                                if let Ok(image_response) =
                                    serde_json::from_value::<ImageGenerationResponse>(transformed)
                                {
                                    return Ok(image_response);
                                }
                            }
                            Err(e) => {
                                eprintln!("Warning: Failed to process image response template: {}. Falling back to default parsing.", e);
                            }
                        }
                    }
                }
            }
        }

        // Fall back to default parsing
        let image_response: ImageGenerationResponse = serde_json::from_str(&response_text)?;
        Ok(image_response)
    }
    pub async fn transcribe_audio(
        &self,
        request: &AudioTranscriptionRequest,
    ) -> Result<AudioTranscriptionResponse> {
        use reqwest::multipart;

        // Use helper method to build URL
        let url = self.build_url(
            "audio_transcriptions",
            &request.model,
            "/audio/transcriptions",
        );

        // Decode base64 audio data
        use base64::Engine;
        let audio_bytes = if request.file.starts_with("data:") {
            // Handle data URL format
            let parts: Vec<&str> = request.file.splitn(2, ',').collect();
            if parts.len() == 2 {
                base64::engine::general_purpose::STANDARD.decode(parts[1])?
            } else {
                anyhow::bail!("Invalid data URL format");
            }
        } else {
            // Assume it's raw base64
            base64::engine::general_purpose::STANDARD.decode(&request.file)?
        };

        // Determine file extension based on the audio format
        // We'll try to detect from the data URL or default to wav
        let file_extension = if request.file.starts_with("data:audio/") {
            let mime_part = request.file.split(';').next().unwrap_or("");
            match mime_part {
                "data:audio/mpeg" | "data:audio/mp3" => "mp3",
                "data:audio/wav" | "data:audio/wave" => "wav",
                "data:audio/flac" => "flac",
                "data:audio/ogg" => "ogg",
                "data:audio/webm" => "webm",
                "data:audio/mp4" => "mp4",
                _ => "wav",
            }
        } else {
            "wav" // Default extension
        };

        // Create multipart form
        let mut form = multipart::Form::new()
            .text("model", request.model.clone())
            .part(
                "file",
                multipart::Part::bytes(audio_bytes)
                    .file_name(format!("audio.{}", file_extension))
                    .mime_str(&format!(
                        "audio/{}",
                        if file_extension == "mp3" {
                            "mpeg"
                        } else {
                            file_extension
                        }
                    ))?,
            );

        // Add optional parameters
        if let Some(language) = &request.language {
            form = form.text("language", language.clone());
        }
        if let Some(prompt) = &request.prompt {
            form = form.text("prompt", prompt.clone());
        }
        if let Some(response_format) = &request.response_format {
            form = form.text("response_format", response_format.clone());
        }
        if let Some(temperature) = request.temperature {
            form = form.text("temperature", temperature.to_string());
        }

        let mut req = self.client.post(&url);

        // Add standard headers using helper method
        req = self.add_standard_headers(req);

        // Send multipart form request
        let response = req.multipart(form).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!(
                "Audio transcription API request failed with status {}: {}",
                status,
                text
            );
        }

        // Get the response text first to handle different formats
        let response_text = response.text().await?;

        // Check if we have a response template for this provider/model/endpoint
        if let Some(ref config) = &self.provider_config {
            if let Some(ref processor) = &self.template_processor {
                // Get response template for audio endpoint
                let template = config.get_endpoint_response_template("audio", &request.model);

                if let Some(template_str) = template {
                    // Parse response as JSON
                    if let Ok(response_json) =
                        serde_json::from_str::<serde_json::Value>(&response_text)
                    {
                        // Clone the processor to avoid mutable borrow issues
                        let mut processor_clone = processor.clone();
                        // Use template to transform response
                        match processor_clone.process_response(&response_json, &template_str) {
                            Ok(transformed) => {
                                // Try to parse the transformed response as AudioTranscriptionResponse
                                if let Ok(audio_response) =
                                    serde_json::from_value::<AudioTranscriptionResponse>(
                                        transformed,
                                    )
                                {
                                    return Ok(audio_response);
                                }
                            }
                            Err(e) => {
                                eprintln!("Warning: Failed to process audio response template: {}. Falling back to default parsing.", e);
                            }
                        }
                    }
                }
            }
        }

        // Fall back to default parsing
        // OpenAI can return just plain text for response_format=text
        if response_text.starts_with('{') {
            // JSON response
            let audio_response: AudioTranscriptionResponse = serde_json::from_str(&response_text)?;
            Ok(audio_response)
        } else {
            // Plain text response
            Ok(AudioTranscriptionResponse {
                text: response_text.trim().to_string(),
                language: None,
                duration: None,
                segments: None,
            })
        }
    }

    pub async fn generate_speech(&self, request: &AudioSpeechRequest) -> Result<Vec<u8>> {
        // Use helper method to build URL
        let url = self.build_url("audio_speech", &request.model, "/audio/speech");

        let mut req = self
            .client
            .post(&url)
            .header("Content-Type", "application/json");

        // Add standard headers using helper method
        req = self.add_standard_headers(req);

        // Check if we have a template for this provider/model/endpoint
        let request_body = if let Some(ref config) = &self.provider_config {
            if let Some(ref processor) = &self.template_processor {
                // Get template for speech endpoint
                let template = config.get_endpoint_template("speech", &request.model);

                if let Some(template_str) = template {
                    // Clone the processor to avoid mutable borrow issues
                    let mut processor_clone = processor.clone();
                    // Use template to transform request
                    match processor_clone.process_speech_request(
                        request,
                        &template_str,
                        &config.vars,
                    ) {
                        Ok(json_value) => Some(json_value),
                        Err(e) => {
                            eprintln!("Warning: Failed to process speech request template: {}. Falling back to default.", e);
                            None
                        }
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        // Send request with template-processed body or fall back to default logic
        let response = if let Some(json_body) = request_body {
            req.json(&json_body).send().await?
        } else {
            req.json(request).send().await?
        };

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!(
                "Speech generation API request failed with status {}: {}",
                status,
                text
            );
        }

        // Get the response text first to handle different formats
        let response_text = response.text().await?;

        // Check if we have a response template for this provider/model/endpoint
        if let Some(ref config) = &self.provider_config {
            if let Some(ref processor) = &self.template_processor {
                // Get response template for speech endpoint
                let template = config.get_endpoint_response_template("speech", &request.model);

                if let Some(template_str) = template {
                    // Parse response as JSON
                    if let Ok(response_json) =
                        serde_json::from_str::<serde_json::Value>(&response_text)
                    {
                        // Clone the processor to avoid mutable borrow issues
                        let mut processor_clone = processor.clone();
                        // Use template to extract base64 data
                        match processor_clone.process_response(&response_json, &template_str) {
                            Ok(extracted) => {
                                // The template should return the base64 string directly
                                if let Some(base64_data) = extracted.as_str() {
                                    // Decode base64 to bytes
                                    use base64::Engine;
                                    match base64::engine::general_purpose::STANDARD
                                        .decode(base64_data)
                                    {
                                        Ok(audio_bytes) => return Ok(audio_bytes),
                                        Err(e) => {
                                            eprintln!("Warning: Failed to decode base64 audio data: {}. Falling back to default parsing.", e);
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("Warning: Failed to process speech response template: {}. Falling back to default parsing.", e);
                            }
                        }
                    }
                }
            }
        }

        // Fall back to default parsing - assume response is raw audio bytes
        // Try to parse as base64 first (for providers that return base64 in plain text)
        if response_text
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=')
        {
            use base64::Engine;
            if let Ok(audio_bytes) =
                base64::engine::general_purpose::STANDARD.decode(&response_text)
            {
                return Ok(audio_bytes);
            }
        }

        // If not base64, treat as raw bytes
        Ok(response_text.into_bytes())
    }

    pub async fn chat_stream(&self, request: &ChatRequest) -> Result<()> {
        use std::io::{stdout, Write};

        let url = self.get_chat_url(&request.model);

        // Use the streaming-optimized client for streaming requests
        let mut req = self
            .streaming_client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Accept", "text/event-stream") // Explicitly request SSE format
            .header("Cache-Control", "no-cache") // Prevent caching for streaming
            .header("Accept-Encoding", "identity"); // Explicitly request no compression

        // Wrap stdout in BufWriter for efficiency
        let stdout = stdout();
        let mut handle = std::io::BufWriter::new(stdout.lock());

        // Add standard headers using helper method
        req = self.add_standard_headers(req);

        // Build request body using template if available (same logic as non-streaming chat)
        let request_body = if let Some(ref config) = &self.provider_config {
            if let Some(ref processor) = &self.template_processor {
                // Get template for chat endpoint
                let template = config.get_endpoint_template("chat", &request.model);

                if let Some(template_str) = template {
                    // Clone the processor to avoid mutable borrow issues
                    let mut processor_clone = processor.clone();
                    // Use template to transform request
                    match processor_clone.process_request(request, &template_str, &config.vars) {
                        Ok(json_value) => Some(json_value),
                        Err(e) => {
                            eprintln!("Warning: Failed to process request template: {}. Falling back to default.", e);
                            None
                        }
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        // Check if we should exclude model from payload (when model is in URL path)
        let should_exclude_model = if let Some(ref config) = self.provider_config {
            config.chat_path.contains("{model}")
        } else {
            self.chat_path.contains("{model}")
        };

        // Send request with template-processed body or fall back to default logic
        let response = if let Some(json_body) = request_body {
            req.json(&json_body).send().await?
        } else if should_exclude_model {
            // Use ChatRequestWithoutModel for providers that specify model in URL
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
                if let Some(data) = line.strip_prefix("data: ") {
                    // Remove "data: " prefix

                    if data.trim() == "[DONE]" {
                        handle.write_all(b"\n")?;
                        handle.flush()?;
                        return Ok(());
                    }

                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                        // Try direct "response" field format first
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
                        // Try direct "response" field format first
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
                // Try direct "response" field format first
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_user() {
        let content = "Hello, world!".to_string();
        let message = Message::user(content.clone());
        assert_eq!(message.role, "user");
        match message.content_type {
            MessageContent::Text { content: c } => assert_eq!(c, Some(content)),
            _ => panic!("Expected Text content"),
        }
        assert!(message.tool_calls.is_none());
        assert!(message.tool_call_id.is_none());
    }

    #[test]
    fn test_message_user_with_image() {
        let text = "What is this image?".to_string();
        let image_data = "data:image/png;base64,...".to_string();
        let detail = Some("high".to_string());
        let message = Message::user_with_image(text.clone(), image_data.clone(), detail.clone());
        assert_eq!(message.role, "user");
        match message.content_type {
            MessageContent::Multimodal { content } => {
                assert_eq!(content.len(), 2);
                match &content[0] {
                    ContentPart::Text { text: t } => assert_eq!(t, &text),
                    _ => panic!("Expected Text content part"),
                }
                match &content[1] {
                    ContentPart::ImageUrl { image_url } => {
                        assert_eq!(image_url.url, image_data);
                        assert_eq!(image_url.detail, detail);
                    }
                    _ => panic!("Expected ImageUrl content part"),
                }
            }
            _ => panic!("Expected Multimodal content"),
        }
    }

    #[test]
    fn test_message_assistant() {
        let content = "I am an AI assistant.".to_string();
        let message = Message::assistant(content.clone());
        assert_eq!(message.role, "assistant");
        match message.content_type {
            MessageContent::Text { content: c } => assert_eq!(c, Some(content)),
            _ => panic!("Expected Text content"),
        }
    }

    #[test]
    fn test_message_assistant_with_tool_calls() {
        let tool_calls = vec![ToolCall {
            id: "call_123".to_string(),
            call_type: "function".to_string(),
            function: FunctionCall {
                name: "get_weather".to_string(),
                arguments: "{\"location\": \"London\"}".to_string(),
            },
        }];
        let message = Message::assistant_with_tool_calls(tool_calls.clone());
        assert_eq!(message.role, "assistant");
        match message.content_type {
            MessageContent::Text { content } => assert!(content.is_none()),
            _ => panic!("Expected Text content"),
        }
        assert_eq!(message.tool_calls.as_ref().unwrap().len(), 1);
        assert_eq!(message.tool_calls.unwrap()[0].id, "call_123");
    }

    #[test]
    fn test_message_tool_result() {
        let tool_call_id = "call_123".to_string();
        let content = "The weather is sunny.".to_string();
        let message = Message::tool_result(tool_call_id.clone(), content.clone());
        assert_eq!(message.role, "tool");
        assert_eq!(message.tool_call_id, Some(tool_call_id));
        match message.content_type {
            MessageContent::Text { content: c } => assert_eq!(c, Some(content)),
            _ => panic!("Expected Text content"),
        }
    }
}
