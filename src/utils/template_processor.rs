use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use tera::{Context as TeraContext, Filter, Tera, Value};

use crate::provider::{ChatRequest, ContentPart, Message, MessageContent};

/// Template processor for handling request/response transformations
#[derive(Clone)]
pub struct TemplateProcessor {
    tera: Tera,
    template_map: HashMap<String, String>, // content -> name
}

/// Endpoint-specific templates with model pattern support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointTemplates {
    /// Default template for all models
    #[serde(default)]
    pub template: Option<TemplateConfig>,

    /// Model-specific templates (exact match)
    #[serde(default)]
    pub model_templates: HashMap<String, TemplateConfig>,

    /// Model pattern templates (regex match)
    #[serde(default)]
    pub model_template_patterns: HashMap<String, TemplateConfig>,
}

/// Template configuration for request/response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateConfig {
    /// Request transformation template
    pub request: Option<String>,
    /// Response parsing template
    pub response: Option<String>,
    /// Streaming response parsing template
    pub stream_response: Option<String>,
}

/// Model-specific endpoint templates (for backward compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEndpointTemplates {
    #[serde(default)]
    pub chat: Option<TemplateConfig>,
    #[serde(default)]
    pub images: Option<TemplateConfig>,
    #[serde(default)]
    pub embeddings: Option<TemplateConfig>,
}

impl EndpointTemplates {
    /// Get template for a specific model, checking patterns and defaults
    #[allow(dead_code)]
    pub fn get_template_for_model(&self, model_name: &str, template_type: &str) -> Option<String> {
        // First check exact match
        if let Some(template) = self.model_templates.get(model_name) {
            return match template_type {
                "request" => template.request.clone(),
                "response" => template.response.clone(),
                "stream_response" => template.stream_response.clone(),
                _ => None,
            };
        }

        // Then check regex patterns
        for (pattern, template) in &self.model_template_patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if re.is_match(model_name) {
                    return match template_type {
                        "request" => template.request.clone(),
                        "response" => template.response.clone(),
                        "stream_response" => template.stream_response.clone(),
                        _ => None,
                    };
                }
            }
        }

        // Finally fall back to default template
        if let Some(template) = &self.template {
            return match template_type {
                "request" => template.request.clone(),
                "response" => template.response.clone(),
                "stream_response" => template.stream_response.clone(),
                _ => None,
            };
        }

        None
    }
}

impl TemplateProcessor {
    /// Create a new template processor
    pub fn new() -> Result<Self> {
        let mut tera = Tera::default();

        // Register custom filters
        tera.register_filter("json", JsonFilter);
        tera.register_filter("gemini_role", GeminiRoleFilter);
        tera.register_filter("system_to_user_role", SystemToUserRoleFilter);
        tera.register_filter("default", DefaultFilter);
        tera.register_filter("select_tool_calls", SelectToolCallsFilter);
        tera.register_filter("from_json", FromJsonFilter);
        tera.register_filter("selectattr", SelectAttrFilter);
        tera.register_filter("base_messages", BaseMessagesFilter);
        tera.register_filter("anthropic_messages", AnthropicMessagesFilter);
        tera.register_filter("gemini_messages", GeminiMessagesFilter);

        Ok(Self {
            tera,
            template_map: HashMap::new(),
        })
    }

    /// Register a template and return its internal name
    pub fn register_template(&mut self, content: &str) -> Result<String> {
        if let Some(name) = self.template_map.get(content) {
            return Ok(name.clone());
        }

        let name = format!("tpl_{}", self.template_map.len());
        self.tera.add_raw_template(&name, content)?;
        self.template_map.insert(content.to_string(), name.clone());
        Ok(name)
    }

    /// Render a template directly
    #[allow(dead_code)]
    pub fn render_template(&self, template: &str, context: &TeraContext) -> Result<String> {
        if let Some(name) = self.template_map.get(template) {
            Ok(self.tera.render(name, context)?)
        } else {
            // Fallback for unregistered templates (should be avoided in critical path)
            let mut tera = self.tera.clone();
            tera.add_raw_template("temp", template)?;
            Ok(tera.render("temp", context)?)
        }
    }

    /// Process a chat request using the provided template
    pub fn process_request(
        &self,
        request: &ChatRequest,
        template: &str,
        provider_vars: &HashMap<String, String>,
    ) -> Result<JsonValue> {
        let template_name = self.template_map.get(template).ok_or_else(|| {
            anyhow::anyhow!("Template not registered. Ensure all templates are registered during initialization.")
        })?;

        // Build context from ChatRequest
        let mut context = TeraContext::new();

        // Add basic fields
        context.insert("model", &request.model);
        context.insert("max_tokens", &request.max_tokens);
        context.insert("temperature", &request.temperature);
        context.insert("stream", &request.stream);
        context.insert("tools", &request.tools);

        // Process messages into a format suitable for templates
        let processed_messages = self.process_messages(&request.messages)?;
        context.insert("messages", &processed_messages);

        // Extract system prompt if present
        if let Some(system_msg) = request.messages.iter().find(|m| m.role == "system") {
            if let Some(content) = system_msg.get_text_content() {
                context.insert("system_prompt", content);
            }
        }

        // Add provider-specific variables
        for (key, value) in provider_vars {
            context.insert(key, value);
        }

        // Render template
        let rendered = self
            .tera
            .render(template_name, &context)
            .context("Failed to render request template")?;

        // Parse as JSON to validate
        let json_value: JsonValue =
            serde_json::from_str(&rendered).context("Template did not produce valid JSON")?;

        Ok(json_value)
    }

    /// Process an image generation request using the provided template
    pub fn process_image_request(
        &self,
        request: &crate::provider::ImageGenerationRequest,
        template: &str,
        provider_vars: &HashMap<String, String>,
    ) -> Result<JsonValue> {
        let template_name = self.template_map.get(template).ok_or_else(|| {
            anyhow::anyhow!("Template not registered. Ensure all templates are registered during initialization.")
        })?;

        // Build context from ImageGenerationRequest
        let mut context = TeraContext::new();

        // Add basic fields
        context.insert("prompt", &request.prompt);
        context.insert("model", &request.model);
        context.insert("n", &request.n);
        context.insert("size", &request.size);
        context.insert("quality", &request.quality);
        context.insert("style", &request.style);
        context.insert("response_format", &request.response_format);

        // Add provider-specific variables
        for (key, value) in provider_vars {
            context.insert(key, value);
        }

        // Render template
        let rendered = self
            .tera
            .render(template_name, &context)
            .context("Failed to render image request template")?;

        // Parse as JSON to validate
        let json_value: JsonValue =
            serde_json::from_str(&rendered).context("Image template did not produce valid JSON")?;

        Ok(json_value)
    }

    /// Process an audio transcription request using the provided template
    #[allow(dead_code)]
    pub fn process_audio_request(
        &self,
        request: &crate::provider::AudioTranscriptionRequest,
        template: &str,
        provider_vars: &HashMap<String, String>,
    ) -> Result<JsonValue> {
        let template_name = self.template_map.get(template).ok_or_else(|| {
            anyhow::anyhow!("Template not registered. Ensure all templates are registered during initialization.")
        })?;

        // Build context from AudioTranscriptionRequest
        let mut context = TeraContext::new();

        // Add basic fields
        context.insert("file", &request.file);
        context.insert("model", &request.model);
        context.insert("language", &request.language);
        context.insert("prompt", &request.prompt);
        context.insert("response_format", &request.response_format);
        context.insert("temperature", &request.temperature);

        // Add provider-specific variables
        for (key, value) in provider_vars {
            context.insert(key, value);
        }

        // Render template
        let rendered = self
            .tera
            .render(template_name, &context)
            .context("Failed to render audio request template")?;

        // Parse as JSON to validate
        let json_value: JsonValue =
            serde_json::from_str(&rendered).context("Audio template did not produce valid JSON")?;

        Ok(json_value)
    }

    /// Process a speech generation request using the provided template
    pub fn process_speech_request(
        &self,
        request: &crate::provider::AudioSpeechRequest,
        template: &str,
        provider_vars: &HashMap<String, String>,
    ) -> Result<JsonValue> {
        let template_name = self.template_map.get(template).ok_or_else(|| {
            anyhow::anyhow!("Template not registered. Ensure all templates are registered during initialization.")
        })?;

        // Build context from AudioSpeechRequest
        let mut context = TeraContext::new();

        // Add basic fields
        context.insert("model", &request.model);
        context.insert("input", &request.input);
        context.insert("voice", &request.voice);
        context.insert("response_format", &request.response_format);
        context.insert("speed", &request.speed);

        // Add provider-specific variables
        for (key, value) in provider_vars {
            context.insert(key, value);
        }

        // Render template
        let rendered = self
            .tera
            .render(template_name, &context)
            .context("Failed to render speech request template")?;

        // Parse as JSON to validate
        let json_value: JsonValue = serde_json::from_str(&rendered)
            .context("Speech template did not produce valid JSON")?;

        Ok(json_value)
    }

    /// Process an embeddings request using the provided template
    pub fn process_embeddings_request(
        &self,
        request: &crate::provider::EmbeddingRequest,
        template: &str,
        provider_vars: &HashMap<String, String>,
    ) -> Result<JsonValue> {
        let template_name = self.template_map.get(template).ok_or_else(|| {
            anyhow::anyhow!("Template not registered. Ensure all templates are registered during initialization.")
        })?;

        // Build context from EmbeddingRequest
        let mut context = TeraContext::new();

        // Add basic fields
        context.insert("model", &request.model);
        context.insert("input", &request.input);
        context.insert("encoding_format", &request.encoding_format);

        // Add provider-specific variables
        for (key, value) in provider_vars {
            context.insert(key, value);
        }

        // Render template
        let rendered = self
            .tera
            .render(template_name, &context)
            .context("Failed to render embeddings request template")?;

        // Parse as JSON to validate
        let json_value: JsonValue = serde_json::from_str(&rendered)
            .context("Embeddings template did not produce valid JSON")?;

        Ok(json_value)
    }

    /// Process a response using the provided template
    pub fn process_response(&self, response: &JsonValue, template: &str) -> Result<JsonValue> {
        let template_name = self.template_map.get(template).ok_or_else(|| {
            anyhow::anyhow!("Template not registered. Ensure all templates are registered during initialization.")
        })?;

        // Build context from response
        let context = TeraContext::from_serialize(response)
            .context("Failed to serialize response to context")?;

        // Render template
        let rendered = self
            .tera
            .render(template_name, &context)
            .context("Failed to render response template")?;

        // Parse as JSON
        let json_value: JsonValue = serde_json::from_str(&rendered)
            .context("Response template did not produce valid JSON")?;

        Ok(json_value)
    }

    /// Process messages into a format suitable for templates
    fn process_messages(&self, messages: &[Message]) -> Result<Vec<ProcessedMessage>> {
        let mut processed = Vec::new();

        for message in messages {
            let mut proc_msg = ProcessedMessage {
                role: message.role.clone(),
                content: None,
                images: Vec::new(),
                tool_calls: message.tool_calls.clone(),
                tool_call_id: message.tool_call_id.clone(),
            };

            match &message.content_type {
                MessageContent::Text { content } => {
                    proc_msg.content = content.clone();
                }
                MessageContent::Multimodal { content } => {
                    for part in content {
                        match part {
                            ContentPart::Text { text } => {
                                proc_msg.content = Some(text.clone());
                            }
                            ContentPart::ImageUrl { image_url } => {
                                // Extract base64 data and mime type from data URL
                                if let Some(data_url) = image_url.url.strip_prefix("data:") {
                                    if let Some(comma_pos) = data_url.find(',') {
                                        let header = &data_url[..comma_pos];
                                        let data = &data_url[comma_pos + 1..];

                                        let mime_type = if let Some(semi_pos) = header.find(';') {
                                            header[..semi_pos].to_string()
                                        } else {
                                            header.to_string()
                                        };

                                        proc_msg.images.push(ProcessedImage {
                                            mime_type,
                                            data: data.to_string(),
                                            url: image_url.url.clone(),
                                        });
                                    }
                                } else {
                                    // Regular URL
                                    proc_msg.images.push(ProcessedImage {
                                        mime_type: "image/jpeg".to_string(), // Default
                                        data: String::new(),
                                        url: image_url.url.clone(),
                                    });
                                }
                            }
                        }
                    }
                }
            }

            processed.push(proc_msg);
        }

        Ok(processed)
    }
}

/// Processed message format for templates
#[derive(Debug, Serialize)]
struct ProcessedMessage {
    role: String,
    content: Option<String>,
    images: Vec<ProcessedImage>,
    tool_calls: Option<Vec<crate::provider::ToolCall>>,
    tool_call_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct ProcessedImage {
    mime_type: String,
    data: String,
    url: String,
}

/// Custom filter to convert values to JSON
struct JsonFilter;

impl Filter for JsonFilter {
    fn filter(&self, value: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
        match serde_json::to_string(&value) {
            Ok(json_str) => Ok(Value::String(json_str)),
            Err(e) => Err(tera::Error::msg(format!(
                "Failed to serialize to JSON: {}",
                e
            ))),
        }
    }
}

/// Filter to convert OpenAI roles to Gemini roles
struct GeminiRoleFilter;

impl Filter for GeminiRoleFilter {
    fn filter(&self, value: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
        match value.as_str() {
            Some("user") => Ok(Value::String("user".to_string())),
            Some("assistant") => Ok(Value::String("model".to_string())),
            Some("system") => Ok(Value::String("user".to_string())), // Gemini handles system as user
            Some(other) => Ok(Value::String(other.to_string())),
            None => Ok(value.clone()),
        }
    }
}

/// Filter to convert system roles to user roles (for providers that don't support system roles)
struct SystemToUserRoleFilter;

impl Filter for SystemToUserRoleFilter {
    fn filter(&self, value: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
        match value.as_str() {
            Some("system") => Ok(Value::String("user".to_string())), // Convert system to user
            Some(other) => Ok(Value::String(other.to_string())),
            None => Ok(value.clone()),
        }
    }
}

/// Filter to provide default values
struct DefaultFilter;

impl Filter for DefaultFilter {
    fn filter(&self, value: &Value, args: &HashMap<String, Value>) -> tera::Result<Value> {
        if value.is_null() || (value.is_string() && value.as_str() == Some("")) {
            if let Some(default_value) = args.get("value") {
                Ok(default_value.clone())
            } else {
                Ok(Value::Null)
            }
        } else {
            Ok(value.clone())
        }
    }
}

/// Filter to select items with tool calls
struct SelectToolCallsFilter;

impl Filter for SelectToolCallsFilter {
    fn filter(&self, value: &Value, args: &HashMap<String, Value>) -> tera::Result<Value> {
        if let Some(array) = value.as_array() {
            let key = args
                .get("key")
                .and_then(|v| v.as_str())
                .unwrap_or("functionCall");

            let filtered: Vec<Value> = array
                .iter()
                .filter(|item| {
                    item.as_object()
                        .map(|obj| obj.contains_key(key))
                        .unwrap_or(false)
                })
                .cloned()
                .collect();

            Ok(Value::Array(filtered))
        } else {
            Ok(Value::Array(vec![]))
        }
    }
}

/// Filter to parse JSON strings
struct FromJsonFilter;

impl Filter for FromJsonFilter {
    fn filter(&self, value: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
        if let Some(json_str) = value.as_str() {
            match serde_json::from_str::<JsonValue>(json_str) {
                Ok(parsed) => {
                    // Convert JsonValue to Tera Value
                    match serde_json::to_value(&parsed) {
                        Ok(tera_value) => Ok(tera_value),
                        Err(e) => Err(tera::Error::msg(format!(
                            "Failed to convert to Tera value: {}",
                            e
                        ))),
                    }
                }
                Err(e) => Err(tera::Error::msg(format!("Failed to parse JSON: {}", e))),
            }
        } else {
            Ok(value.clone())
        }
    }
}

/// Filter to select items by attribute value (simplified version of Jinja2's selectattr)
struct SelectAttrFilter;

impl Filter for SelectAttrFilter {
    fn filter(&self, value: &Value, args: &HashMap<String, Value>) -> tera::Result<Value> {
        if let Some(array) = value.as_array() {
            let attr_name = args
                .get("attr")
                .and_then(|v| v.as_str())
                .ok_or_else(|| tera::Error::msg("selectattr filter requires 'attr' argument"))?;

            let test_value = args
                .get("value")
                .ok_or_else(|| tera::Error::msg("selectattr filter requires 'value' argument"))?;

            let filtered: Vec<Value> = array
                .iter()
                .filter(|item| {
                    if let Some(obj) = item.as_object() {
                        if let Some(attr_value) = obj.get(attr_name) {
                            attr_value == test_value
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                })
                .cloned()
                .collect();

            Ok(Value::Array(filtered))
        } else {
            Ok(Value::Array(vec![]))
        }
    }
}

/// Filter to create base messages with only essential fields (role, content) for simple providers
struct BaseMessagesFilter;

impl Filter for BaseMessagesFilter {
    fn filter(&self, value: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
        if let Some(array) = value.as_array() {
            let cleaned: Vec<Value> = array
                .iter()
                .map(|item| {
                    if let Some(obj) = item.as_object() {
                        let mut cleaned_obj = serde_json::Map::new();

                        // Only include non-null, non-empty fields that are commonly supported
                        for (key, value) in obj {
                            match key.as_str() {
                                "role" | "content" => {
                                    // Always include role and content
                                    cleaned_obj.insert(key.clone(), value.clone());
                                }
                                "tool_calls" => {
                                    // Only include tool_calls if it's not null and not empty
                                    if !value.is_null()
                                        && value.as_array().is_none_or(|arr| !arr.is_empty())
                                    {
                                        cleaned_obj.insert(key.clone(), value.clone());
                                    }
                                }
                                "tool_call_id" => {
                                    // Only include tool_call_id if it's not null and not empty
                                    if !value.is_null()
                                        && value.as_str().is_some_and(|s| !s.is_empty())
                                    {
                                        cleaned_obj.insert(key.clone(), value.clone());
                                    }
                                }
                                // Skip images and any other fields that might cause issues
                                _ => {}
                            }
                        }

                        Value::Object(cleaned_obj)
                    } else {
                        item.clone()
                    }
                })
                .collect();

            Ok(Value::Array(cleaned))
        } else {
            Ok(value.clone())
        }
    }
}

/// Filter to convert messages to Anthropic's specific format with content arrays
struct AnthropicMessagesFilter;

impl Filter for AnthropicMessagesFilter {
    fn filter(&self, value: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
        if let Some(array) = value.as_array() {
            let converted: Vec<Value> = array
                .iter()
                .map(|item| {
                    if let Some(obj) = item.as_object() {
                        let mut anthropic_msg = serde_json::Map::new();

                        // Always include role
                        if let Some(role) = obj.get("role") {
                            anthropic_msg.insert("role".to_string(), role.clone());
                        }

                        // Convert content to Anthropic's format
                        let mut content_parts = Vec::new();

                        // Add text content if present
                        if let Some(text_content) = obj.get("content") {
                            if !text_content.is_null()
                                && text_content.as_str().is_some_and(|s| !s.is_empty())
                            {
                                let text_part = serde_json::json!({
                                    "type": "text",
                                    "text": text_content
                                });
                                content_parts.push(text_part);
                            }
                        }

                        // Add image content if present
                        if let Some(images) = obj.get("images") {
                            if let Some(images_array) = images.as_array() {
                                for image in images_array {
                                    if let Some(image_obj) = image.as_object() {
                                        if let (Some(data), Some(mime_type)) = (
                                            image_obj.get("data").and_then(|v| v.as_str()),
                                            image_obj.get("mime_type").and_then(|v| v.as_str()),
                                        ) {
                                            if !data.is_empty() {
                                                // Base64 image
                                                let image_part = serde_json::json!({
                                                    "type": "image",
                                                    "source": {
                                                        "type": "base64",
                                                        "media_type": mime_type,
                                                        "data": data
                                                    }
                                                });
                                                content_parts.push(image_part);
                                            }
                                        } else if let Some(url) =
                                            image_obj.get("url").and_then(|v| v.as_str())
                                        {
                                            if !url.starts_with("data:") && !url.is_empty() {
                                                // URL image
                                                let image_part = serde_json::json!({
                                                    "type": "image",
                                                    "source": {
                                                        "type": "url",
                                                        "url": url
                                                    }
                                                });
                                                content_parts.push(image_part);
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Set content as array if we have parts, otherwise as string
                        if content_parts.len() > 1
                            || (content_parts.len() == 1
                                && content_parts[0].get("type")
                                    == Some(&serde_json::Value::String("image".to_string())))
                        {
                            anthropic_msg.insert(
                                "content".to_string(),
                                serde_json::Value::Array(content_parts),
                            );
                        } else if let Some(first_part) = content_parts.first() {
                            if let Some(text) = first_part.get("text") {
                                anthropic_msg.insert("content".to_string(), text.clone());
                            }
                        }

                        // Include tool_calls if present and not empty
                        if let Some(tool_calls) = obj.get("tool_calls") {
                            if !tool_calls.is_null()
                                && tool_calls.as_array().is_none_or(|arr| !arr.is_empty())
                            {
                                anthropic_msg.insert("tool_calls".to_string(), tool_calls.clone());
                            }
                        }

                        // Include tool_call_id if present and not empty
                        if let Some(tool_call_id) = obj.get("tool_call_id") {
                            if !tool_call_id.is_null()
                                && tool_call_id.as_str().is_some_and(|s| !s.is_empty())
                            {
                                anthropic_msg
                                    .insert("tool_call_id".to_string(), tool_call_id.clone());
                            }
                        }

                        Value::Object(anthropic_msg)
                    } else {
                        item.clone()
                    }
                })
                .collect();

            Ok(Value::Array(converted))
        } else {
            Ok(value.clone())
        }
    }
}

/// Filter to convert messages to Gemini's specific format with parts arrays
struct GeminiMessagesFilter;

impl Filter for GeminiMessagesFilter {
    fn filter(&self, value: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
        if let Some(array) = value.as_array() {
            let converted: Vec<Value> = array
                .iter()
                .map(|item| {
                    if let Some(obj) = item.as_object() {
                        let mut gemini_msg = serde_json::Map::new();

                        // Convert role to Gemini format
                        if let Some(role) = obj.get("role").and_then(|v| v.as_str()) {
                            let gemini_role = match role {
                                "assistant" => "model",
                                "system" => "user", // Gemini handles system as user
                                other => other,
                            };
                            gemini_msg.insert(
                                "role".to_string(),
                                serde_json::Value::String(gemini_role.to_string()),
                            );
                        }

                        // Convert content to Gemini's parts format
                        let mut parts = Vec::new();

                        // Add text content if present
                        if let Some(text_content) = obj.get("content") {
                            if !text_content.is_null()
                                && text_content.as_str().is_some_and(|s| !s.is_empty())
                            {
                                let text_part = serde_json::json!({
                                    "text": text_content
                                });
                                parts.push(text_part);
                            }
                        }

                        // Add image content if present
                        if let Some(images) = obj.get("images") {
                            if let Some(images_array) = images.as_array() {
                                for image in images_array {
                                    if let Some(image_obj) = image.as_object() {
                                        if let (Some(data), Some(mime_type)) = (
                                            image_obj.get("data").and_then(|v| v.as_str()),
                                            image_obj.get("mime_type").and_then(|v| v.as_str()),
                                        ) {
                                            if !data.is_empty() {
                                                // Base64 image for Gemini
                                                let image_part = serde_json::json!({
                                                    "inlineData": {
                                                        "mimeType": mime_type,
                                                        "data": data
                                                    }
                                                });
                                                parts.push(image_part);
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Set parts array
                        gemini_msg.insert("parts".to_string(), serde_json::Value::Array(parts));

                        // Include tool_calls if present and not empty (for function calling)
                        if let Some(tool_calls) = obj.get("tool_calls") {
                            if !tool_calls.is_null()
                                && tool_calls.as_array().is_none_or(|arr| !arr.is_empty())
                            {
                                gemini_msg.insert("tool_calls".to_string(), tool_calls.clone());
                            }
                        }

                        // Include tool_call_id if present and not empty
                        if let Some(tool_call_id) = obj.get("tool_call_id") {
                            if !tool_call_id.is_null()
                                && tool_call_id.as_str().is_some_and(|s| !s.is_empty())
                            {
                                gemini_msg.insert("tool_call_id".to_string(), tool_call_id.clone());
                            }
                        }

                        Value::Object(gemini_msg)
                    } else {
                        item.clone()
                    }
                })
                .collect();

            Ok(Value::Array(converted))
        } else {
            Ok(value.clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_filter() {
        let filter = JsonFilter;
        let value = Value::String("test".to_string());
        let args = HashMap::new();

        let result = filter.filter(&value, &args).unwrap();
        assert_eq!(result, Value::String("\"test\"".to_string()));
    }

    #[test]
    fn test_gemini_role_filter() {
        let filter = GeminiRoleFilter;
        let args = HashMap::new();

        let value = Value::String("assistant".to_string());
        let result = filter.filter(&value, &args).unwrap();
        assert_eq!(result, Value::String("model".to_string()));

        let value = Value::String("system".to_string());
        let result = filter.filter(&value, &args).unwrap();
        assert_eq!(result, Value::String("user".to_string()));
    }

    #[test]
    fn test_default_filter() {
        let filter = DefaultFilter;
        let mut args = HashMap::new();
        args.insert("value".to_string(), Value::String("default".to_string()));

        let value = Value::Null;
        let result = filter.filter(&value, &args).unwrap();
        assert_eq!(result, Value::String("default".to_string()));

        let value = Value::String("existing".to_string());
        let result = filter.filter(&value, &args).unwrap();
        assert_eq!(result, Value::String("existing".to_string()));
    }

    #[test]
    fn test_template_registration() {
        let mut processor = TemplateProcessor::new().unwrap();
        let template = r#"{"test": "{{ value }}"}"#;

        // Register template
        let name = processor.register_template(template).unwrap();

        // Should get same name for same content
        let name2 = processor.register_template(template).unwrap();
        assert_eq!(name, name2);

        // Render
        let mut context = TeraContext::new();
        context.insert("value", "hello");

        let result = processor.render_template(template, &context).unwrap();
        assert_eq!(result, r#"{"test": "hello"}"#);
    }
}
