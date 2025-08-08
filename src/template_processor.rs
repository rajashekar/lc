use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use tera::{Context as TeraContext, Filter, Tera, Value};

use crate::provider::{ChatRequest, Message, MessageContent, ContentPart};

/// Template processor for handling request/response transformations
#[derive(Clone)]
pub struct TemplateProcessor {
    tera: Tera,
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
        tera.register_filter("bedrock_role", BedrockRoleFilter);
        tera.register_filter("default", DefaultFilter);
        tera.register_filter("select_tool_calls", SelectToolCallsFilter);
        
        Ok(Self { tera })
    }

    /// Render a template directly
    #[allow(dead_code)]
    pub fn render_template(&mut self, name: &str, template: &str, context: &TeraContext) -> Result<String> {
        self.tera.add_raw_template(name, template)?;
        Ok(self.tera.render(name, context)?)
    }

    /// Process a chat request using the provided template
    pub fn process_request(
        &mut self,
        request: &ChatRequest,
        template: &str,
        provider_vars: &HashMap<String, String>,
    ) -> Result<JsonValue> {
        // Add template to Tera
        self.tera
            .add_raw_template("request", template)
            .context("Failed to parse request template")?;

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
        let rendered = self.tera
            .render("request", &context)
            .context("Failed to render request template")?;

        // Parse as JSON to validate
        let json_value: JsonValue = serde_json::from_str(&rendered)
            .context("Template did not produce valid JSON")?;

        Ok(json_value)
    }

    /// Process a response using the provided template
    pub fn process_response(
        &mut self,
        response: &JsonValue,
        template: &str,
    ) -> Result<JsonValue> {
        // Add template to Tera
        self.tera
            .add_raw_template("response", template)
            .context("Failed to parse response template")?;

        // Build context from response
        let context = TeraContext::from_serialize(response)
            .context("Failed to serialize response to context")?;

        // Render template
        let rendered = self.tera
            .render("response", &context)
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
            Err(e) => Err(tera::Error::msg(format!("Failed to serialize to JSON: {}", e))),
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

/// Filter to convert OpenAI roles to Bedrock roles
struct BedrockRoleFilter;

impl Filter for BedrockRoleFilter {
    fn filter(&self, value: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
        match value.as_str() {
            Some("system") => Ok(Value::String("user".to_string())), // Bedrock handles system as user
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
            let key = args.get("key")
                .and_then(|v| v.as_str())
                .unwrap_or("functionCall");
                
            let filtered: Vec<Value> = array.iter()
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
}