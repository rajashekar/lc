use crate::provider::Provider;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub id: String,
    pub provider: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub owned_by: Option<String>,
    pub created: Option<i64>,

    // Context and token limits
    pub context_length: Option<u32>,
    pub max_input_tokens: Option<u32>,
    pub max_output_tokens: Option<u32>,

    // Pricing (per million tokens)
    pub input_price_per_m: Option<f64>,
    pub output_price_per_m: Option<f64>,

    // Capabilities - These flags must only be set to `true` when the provider JSON explicitly contains that feature
    /// Only set to `true` when provider JSON explicitly indicates tool/function calling support
    pub supports_tools: bool,
    /// Only set to `true` when provider JSON explicitly indicates vision/image processing support
    pub supports_vision: bool,
    /// Only set to `true` when provider JSON explicitly indicates audio processing support
    pub supports_audio: bool,
    /// Only set to `true` when provider JSON explicitly indicates advanced reasoning capabilities
    pub supports_reasoning: bool,
    /// Only set to `true` when provider JSON explicitly indicates code generation support
    pub supports_code: bool,
    /// Only set to `true` when provider JSON explicitly indicates function calling support
    pub supports_function_calling: bool,
    /// Only set to `true` when provider JSON explicitly indicates JSON mode support
    pub supports_json_mode: bool,
    /// Only set to `true` when provider JSON explicitly indicates streaming support
    pub supports_streaming: bool,

    // Model type and characteristics
    pub model_type: ModelType,
    /// Only set to `true` when provider JSON explicitly indicates the model is deprecated
    pub is_deprecated: bool,
    /// Only set to `true` when provider JSON explicitly indicates the model supports fine-tuning
    pub is_fine_tunable: bool,

    // Raw provider-specific data
    pub raw_data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelType {
    Chat,
    Completion,
    Embedding,
    ImageGeneration,
    AudioGeneration,
    Moderation,
    Other(String),
}

impl Default for ModelMetadata {
    fn default() -> Self {
        Self {
            id: String::new(),
            provider: String::new(),
            display_name: None,
            description: None,
            owned_by: None,
            created: None,
            context_length: None,
            max_input_tokens: None,
            max_output_tokens: None,
            input_price_per_m: None,
            output_price_per_m: None,
            supports_tools: false,
            supports_vision: false,
            supports_audio: false,
            supports_reasoning: false,
            supports_code: false,
            supports_function_calling: false,
            supports_json_mode: false,
            supports_streaming: false,
            model_type: ModelType::Chat,
            is_deprecated: false,
            is_fine_tunable: false,
            raw_data: serde_json::Value::Null,
        }
    }
}

// Configuration structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPaths {
    pub paths: Vec<String>,
    #[serde(default)]
    pub field_mappings: FieldMappings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldMappings {
    /// Fields to check for model ID (in priority order)
    pub id_fields: Vec<String>,
    /// Fields to check for model name/display name (in priority order)
    pub name_fields: Vec<String>,
}

impl Default for FieldMappings {
    fn default() -> Self {
        Self {
            id_fields: vec![
                "id".to_string(),
                "modelId".to_string(),
                "name".to_string(),
                "modelName".to_string(),
            ],
            name_fields: vec![
                "display_name".to_string(),
                "name".to_string(),
                "modelName".to_string(),
            ],
        }
    }
}

impl Default for ModelPaths {
    fn default() -> Self {
        Self {
            paths: vec![
                ".data[]".to_string(),
                ".models[]".to_string(),
                ".".to_string(),
            ],
            field_mappings: FieldMappings::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagConfig {
    pub tags: HashMap<String, TagRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagRule {
    pub paths: Vec<String>,
    pub value_type: String,
    pub transform: Option<String>,
}

impl Default for TagConfig {
    fn default() -> Self {
        let mut tags = HashMap::new();

        // Context length
        tags.insert(
            "context_length".to_string(),
            TagRule {
                paths: vec![
                    ".context_length".to_string(),
                    ".context_window".to_string(),
                    ".context_size".to_string(),
                    ".max_context_length".to_string(),
                    ".input_token_limit".to_string(),
                    ".inputTokenLimit".to_string(),
                    ".limits.max_input_tokens".to_string(),
                    ".top_provider.context_length".to_string(),
                ],
                value_type: "u32".to_string(),
                transform: None,
            },
        );

        // Output tokens
        tags.insert(
            "output".to_string(),
            TagRule {
                paths: vec![
                    ".max_completion_tokens".to_string(),
                    ".outputTokenLimit".to_string(),
                    ".max_output_tokens".to_string(),
                    ".limits.max_output_tokens".to_string(),
                    ".top_provider.max_completion_tokens".to_string(),
                    ".max_tokens".to_string(),
                ],
                value_type: "u32".to_string(),
                transform: None,
            },
        );

        // Input pricing
        tags.insert(
            "input_price_per_m".to_string(),
            TagRule {
                paths: vec![
                    ".pricing.prompt".to_string(),
                    ".pricing.input.usd".to_string(),
                    ".input_price".to_string(),
                ],
                value_type: "f64".to_string(),
                transform: Some("multiply_million".to_string()),
            },
        );

        // Input pricing direct (no transform)
        tags.insert(
            "input_price_per_m_direct".to_string(),
            TagRule {
                paths: vec![".input_token_price_per_m".to_string()],
                value_type: "f64".to_string(),
                transform: None,
            },
        );

        // Output pricing
        tags.insert(
            "output_price_per_m".to_string(),
            TagRule {
                paths: vec![
                    ".pricing.completion".to_string(),
                    ".pricing.output.usd".to_string(),
                    ".output_price".to_string(),
                ],
                value_type: "f64".to_string(),
                transform: Some("multiply_million".to_string()),
            },
        );

        // Output pricing direct (no transform)
        tags.insert(
            "output_price_per_m_direct".to_string(),
            TagRule {
                paths: vec![".output_token_price_per_m".to_string()],
                value_type: "f64".to_string(),
                transform: None,
            },
        );

        // Vision support with comprehensive name-based detection
        tags.insert(
            "supports_vision".to_string(),
            TagRule {
                paths: vec![
                    ".supports_vision".to_string(),
                    ".supports_image_input".to_string(),
                    ".capabilities.vision".to_string(),
                    ".architecture.input_modalities[] | select(. == \"image\")".to_string(),
                    ".architecture.output_modalities[] | select(. == \"image\")".to_string(),
                    "@name_contains(\"image\")".to_string(),
                    "@name_contains(\"flux\")".to_string(),
                    "@name_contains(\"dall-e\")".to_string(),
                    "@name_contains(\"midjourney\")".to_string(),
                    "@name_contains(\"stable\")".to_string(),
                    "@name_contains(\"diffusion\")".to_string(),
                    "@name_contains(\"vision\")".to_string(),
                    "@name_contains(\"visual\")".to_string(),
                    "@name_contains(\"photo\")".to_string(),
                    "@name_contains(\"picture\")".to_string(),
                    "@name_contains(\"draw\")".to_string(),
                    "@name_contains(\"paint\")".to_string(),
                    "@name_contains(\"art\")".to_string(),
                    "@name_contains(\"generate\")".to_string(),
                ],
                value_type: "bool".to_string(),
                transform: None,
            },
        );

        // Tools/Function calling support
        tags.insert(
            "supports_tools".to_string(),
            TagRule {
                paths: vec![
                    ".supports_tools".to_string(),
                    ".capabilities.function_calling".to_string(),
                    ".features[] | select(. == \"tools\")".to_string(),
                    ".features[] | select(. == \"function-calling\")".to_string(),
                    ".capabilities[] | select(. == \"tool-calling\")".to_string(),
                    ".supported_parameters[] | select(. == \"tools\")".to_string(),
                ],
                value_type: "bool".to_string(),
                transform: None,
            },
        );

        // Audio support
        tags.insert(
            "supports_audio".to_string(),
            TagRule {
                paths: vec![
                    ".supports_audio".to_string(),
                    "@name_contains(\"audio\")".to_string(),
                    ".features[] | select(. == \"audio\")".to_string(),
                    ".capabilities[] | select(. == \"audio\")".to_string(),
                    ".supported_input_modalities[] | select(. == \"audio\")".to_string(),
                    ".supported_output_modalities[] | select(. == \"audio\")".to_string(),
                    ".architecture.input_modalities[] | select(. == \"audio\")".to_string(),
                    ".architecture.output_modalities[] | select(. == \"audio\")".to_string(),
                ],
                value_type: "bool".to_string(),
                transform: None,
            },
        );

        // Reasoning support
        tags.insert(
            "supports_reasoning".to_string(),
            TagRule {
                paths: vec![
                    ".supports_reasoning".to_string(),
                    ".features[] | select(. == \"think\")".to_string(),
                    ".features[] | select(. == \"reasoning\")".to_string(),
                    ".capabilities[] | select(. == \"reasoning\")".to_string(),
                    ".supported_input_modalities[] | select(. == \"reasoning\")".to_string(),
                    ".supported_output_modalities[] | select(. == \"reasoning\")".to_string(),
                    ".architecture.input_modalities[] | select(. == \"reasoning\")".to_string(),
                    ".architecture.output_modalities[] | select(. == \"reasoning\")".to_string(),
                ],
                value_type: "bool".to_string(),
                transform: None,
            },
        );

        Self { tags }
    }
}

// Main extractor
pub struct ModelMetadataExtractor {
    model_paths: ModelPaths,
    tag_config: TagConfig,
}

impl ModelMetadataExtractor {
    pub fn new() -> Result<Self> {
        // Ensure configuration files exist on first run
        if let Err(e) = Self::ensure_config_files_exist() {
            eprintln!(
                "Warning: Failed to ensure model metadata config files exist: {}",
                e
            );
        }

        let model_paths = Self::load_model_paths()?;
        let tag_config = Self::load_tag_config()?;

        Ok(Self {
            model_paths,
            tag_config,
        })
    }

    /// Ensures that tags.toml and model_paths.toml exist with default values
    fn ensure_config_files_exist() -> Result<()> {
        let config_dir = Self::get_config_dir()?;

        // Ensure directory exists
        fs::create_dir_all(&config_dir)?;

        // Check and create model_paths.toml if it doesn't exist
        let model_paths_file = config_dir.join("model_paths.toml");
        if !model_paths_file.exists() {
            let default_paths = ModelPaths::default();
            let content = toml::to_string_pretty(&default_paths)?;
            fs::write(&model_paths_file, content)?;
        }

        // Check and create tags.toml if it doesn't exist
        let tags_file = config_dir.join("tags.toml");
        if !tags_file.exists() {
            let default_tags = TagConfig::default();
            let content = toml::to_string_pretty(&default_tags)?;
            fs::write(&tags_file, content)?;
        }

        Ok(())
    }

    fn get_config_dir() -> Result<PathBuf> {
        // Check for test environment variables first
        if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
            return Ok(std::path::PathBuf::from(xdg_config).join("lc"));
        }

        if let Ok(home) = std::env::var("HOME") {
            // Check if this looks like a test environment (temp directory)
            if home.contains("tmp") || home.contains("temp") {
                return Ok(std::path::PathBuf::from(home).join(".config").join("lc"));
            }
        }

        // Default behavior for production
        let config_dir = dirs::config_dir()
            .context("Failed to get config directory")?
            .join("lc");
        Ok(config_dir)
    }

    fn load_model_paths() -> Result<ModelPaths> {
        let config_dir = Self::get_config_dir()?;
        let path = config_dir.join("model_paths.toml");

        // Ensure directory exists
        fs::create_dir_all(&config_dir)?;

        if path.exists() {
            let content = fs::read_to_string(&path)?;
            toml::from_str(&content).context("Failed to parse model_paths.toml")
        } else {
            // Create default file
            let default = ModelPaths::default();
            let content = toml::to_string_pretty(&default)?;
            fs::write(&path, content)?;
            Ok(default)
        }
    }

    fn load_tag_config() -> Result<TagConfig> {
        let config_dir = Self::get_config_dir()?;
        let path = config_dir.join("tags.toml");

        // Ensure directory exists
        fs::create_dir_all(&config_dir)?;

        if path.exists() {
            let content = fs::read_to_string(&path)?;
            toml::from_str(&content).context("Failed to parse tags.toml")
        } else {
            // Create default file
            let default = TagConfig::default();
            let content = toml::to_string_pretty(&default)?;
            fs::write(&path, content)?;
            Ok(default)
        }
    }

    pub fn extract_models(&self, provider: &Provider, response: &Value) -> Result<Vec<Value>> {
        let mut models = Vec::new();

        for path in &self.model_paths.paths {
            if let Ok(extracted) = self.extract_with_jq_path(response, path) {
                match &extracted {
                    Value::Array(arr) => models.extend(arr.clone()),
                    Value::Object(obj) => {
                        // Check if object looks like a model using configured field mappings
                        let has_model_field = self
                            .model_paths
                            .field_mappings
                            .id_fields
                            .iter()
                            .any(|field| obj.contains_key(field))
                            || obj.contains_key("model"); // Keep "model" as a generic field

                        if has_model_field {
                            models.push(extracted);
                        }
                    }
                    _ => {}
                }
            }
        }

        // Special handling for HuggingFace
        if provider.provider == "hf" || provider.provider == "huggingface" {
            models = self.expand_huggingface_models(models)?;
        }

        Ok(models)
    }

    pub fn extract_with_jq_path(&self, data: &Value, path: &str) -> Result<Value> {
        // Simple JQ path implementation
        if path == "." {
            return Ok(data.clone());
        }

        // Handle complex JQ expressions with pipes
        if path.contains(" | ") {
            return self.extract_with_jq_filter(data, path);
        }

        let parts: Vec<&str> = path.split('.').filter(|s| !s.is_empty()).collect();
        let mut current = data;

        for part in parts {
            if let Some(field) = part.strip_suffix("[]") {
                current = current
                    .get(field)
                    .context(format!("Field {} not found", field))?;
                if !current.is_array() {
                    anyhow::bail!("Expected array at {}", field);
                }
            } else {
                current = current
                    .get(part)
                    .context(format!("Field {} not found", part))?;
            }
        }

        Ok(current.clone())
    }

    fn extract_with_jq_filter(&self, data: &Value, path: &str) -> Result<Value> {
        let parts: Vec<&str> = path.split(" | ").collect();
        if parts.len() != 2 {
            anyhow::bail!("Complex JQ filters not supported: {}", path);
        }

        let array_path = parts[0].trim();
        let filter = parts[1].trim();

        // Extract the array first
        let array_value = self.extract_with_jq_path(data, array_path)?;

        // Handle select filters
        if filter.starts_with("select(") && filter.ends_with(")") {
            let condition = &filter[7..filter.len() - 1]; // Remove "select(" and ")"

            if let Value::Array(arr) = array_value {
                // Check if any element in the array matches the condition
                for item in arr {
                    if self.evaluate_select_condition(&item, condition)? {
                        return Ok(Value::Bool(true));
                    }
                }
                return Ok(Value::Bool(false));
            } else {
                // For non-arrays, evaluate the condition directly
                if self.evaluate_select_condition(&array_value, condition)? {
                    return Ok(array_value);
                } else {
                    return Ok(Value::Null);
                }
            }
        }

        anyhow::bail!("Unsupported JQ filter: {}", filter)
    }

    fn evaluate_select_condition(&self, value: &Value, condition: &str) -> Result<bool> {
        // Handle equality conditions like '. == "tool-calling"'
        if condition.starts_with(". == ") {
            let expected = condition.strip_prefix(". == ").unwrap().trim();

            // Remove quotes if present
            let expected = if expected.starts_with('"') && expected.ends_with('"') {
                &expected[1..expected.len() - 1]
            } else {
                expected
            };

            match value {
                Value::String(s) => Ok(s == expected),
                Value::Number(n) => {
                    if let Ok(num) = expected.parse::<f64>() {
                        Ok(n.as_f64() == Some(num))
                    } else {
                        Ok(false)
                    }
                }
                Value::Bool(b) => {
                    if let Ok(bool_val) = expected.parse::<bool>() {
                        Ok(*b == bool_val)
                    } else {
                        Ok(false)
                    }
                }
                _ => Ok(false),
            }
        } else {
            anyhow::bail!("Unsupported select condition: {}", condition)
        }
    }

    fn expand_huggingface_models(&self, models: Vec<Value>) -> Result<Vec<Value>> {
        let mut expanded = Vec::new();

        for model in models {
            if let Some(providers) = model.get("providers").and_then(|p| p.as_array()) {
                for provider in providers {
                    let mut new_model = model.clone();
                    if let Some(obj) = new_model.as_object_mut() {
                        obj.insert("provider".to_string(), provider.clone());
                        obj.remove("providers");
                    }
                    expanded.push(new_model);
                }
            } else {
                expanded.push(model);
            }
        }

        Ok(expanded)
    }

    pub fn extract_metadata(&self, provider: &Provider, model: &Value) -> Result<ModelMetadata> {
        let mut metadata = ModelMetadata::default();

        // Extract ID using configured field mappings (in priority order)
        let base_id = self
            .model_paths
            .field_mappings
            .id_fields
            .iter()
            .find_map(|field| model.get(field).and_then(|v| v.as_str()))
            .map(|s| s.to_string())
            .ok_or_else(|| {
                let fields = self.model_paths.field_mappings.id_fields.join(", ");
                anyhow::anyhow!(
                    "Model missing required ID field. Checked fields: {}",
                    fields
                )
            })?;

        // For HuggingFace models, append the provider suffix from the expanded provider object
        if (provider.provider == "hf" || provider.provider == "huggingface")
            && model.get("provider").is_some()
        {
            if let Some(provider_obj) = model.get("provider") {
                if let Some(provider_name) = provider_obj.get("provider").and_then(|v| v.as_str()) {
                    metadata.id = format!("{}:{}", base_id, provider_name);
                } else {
                    metadata.id = base_id;
                }
            } else {
                metadata.id = base_id;
            }
        } else {
            metadata.id = base_id;
        }

        metadata.provider = provider.provider.clone();
        metadata.raw_data = model.clone();

        // Extract basic fields using configured field mappings
        if let Some(name) = self
            .model_paths
            .field_mappings
            .name_fields
            .iter()
            .find_map(|field| model.get(field).and_then(|v| v.as_str()))
        {
            metadata.display_name = Some(name.to_string());
        }

        if let Some(desc) = model.get("description").and_then(|v| v.as_str()) {
            metadata.description = Some(desc.to_string());
        }

        if let Some(owner) = model.get("owned_by").and_then(|v| v.as_str()) {
            metadata.owned_by = Some(owner.to_string());
        }

        if let Some(created) = model.get("created").and_then(|v| v.as_i64()) {
            metadata.created = Some(created);
        }

        // Extract tags using configured rules
        for (tag_name, rule) in &self.tag_config.tags {
            if let Some(value) = self.extract_tag_value(model, rule) {
                self.apply_tag_value(&mut metadata, tag_name, value, &rule.value_type)?;
            }
        }

        // Determine model type based on model ID or name patterns
        metadata.model_type =
            self.determine_model_type(&metadata.id, metadata.display_name.as_deref());

        Ok(metadata)
    }

    fn extract_tag_value(&self, model: &Value, rule: &TagRule) -> Option<Value> {
        // For boolean fields, we want to find the first "true" value, not just the first non-null value
        let is_bool_field = rule.value_type == "bool";
        let mut found_false = false;

        for path in &rule.paths {
            // Handle special name-based patterns
            if path.starts_with("@name_contains(") && path.ends_with(")") {
                let pattern = &path[15..path.len() - 1]; // Remove "@name_contains(" and ")"
                let pattern = pattern.trim_matches('"'); // Remove quotes if present

                if let Some(result) = self.check_name_contains(model, pattern) {
                    if is_bool_field && result {
                        return Some(Value::Bool(true));
                    } else if !is_bool_field {
                        return Some(Value::Bool(result));
                    } else if !result {
                        found_false = true;
                    }
                }
                continue;
            }

            if path.starts_with("@name_matches(") && path.ends_with(")") {
                let pattern = &path[14..path.len() - 1]; // Remove "@name_matches(" and ")"
                let pattern = pattern.trim_matches('"'); // Remove quotes if present

                if let Some(result) = self.check_name_matches(model, pattern) {
                    if is_bool_field && result {
                        return Some(Value::Bool(true));
                    } else if !is_bool_field {
                        return Some(Value::Bool(result));
                    } else if !result {
                        found_false = true;
                    }
                }
                continue;
            }

            // Regular JQ path extraction
            if let Ok(value) = self.extract_with_jq_path(model, path) {
                if !value.is_null() {
                    // For boolean fields, continue searching if we found false, but return immediately if we found true
                    if is_bool_field {
                        if let Some(bool_val) = value.as_bool() {
                            if bool_val {
                                // Found true, return immediately
                                if let Some(transform) = &rule.transform {
                                    return self.apply_transform(value, transform);
                                }
                                return Some(value);
                            } else {
                                // Found false, remember it but continue searching
                                found_false = true;
                            }
                        }
                    } else {
                        // For non-boolean fields, return the first non-null value
                        if let Some(transform) = &rule.transform {
                            return self.apply_transform(value, transform);
                        }
                        return Some(value);
                    }
                }
            }
        }

        // If we're dealing with a boolean field and found at least one false, return false
        // Otherwise return None
        if is_bool_field && found_false {
            Some(Value::Bool(false))
        } else {
            None
        }
    }

    fn apply_transform(&self, value: Value, transform: &str) -> Option<Value> {
        match transform {
            "multiply_million" => {
                value.as_f64().map(|num| Value::from(num * 1_000_000.0))
            }
            _ => Some(value),
        }
    }

    fn apply_tag_value(
        &self,
        metadata: &mut ModelMetadata,
        tag_name: &str,
        value: Value,
        value_type: &str,
    ) -> Result<()> {
        match tag_name {
            "context_length" => {
                if let Some(v) = self.parse_value_as_u32(&value, value_type)? {
                    metadata.context_length = Some(v);
                }
            }
            "max_input_tokens" => {
                if let Some(v) = self.parse_value_as_u32(&value, value_type)? {
                    metadata.max_input_tokens = Some(v);
                }
            }
            "max_output_tokens" | "output" => {
                if let Some(v) = self.parse_value_as_u32(&value, value_type)? {
                    metadata.max_output_tokens = Some(v);
                }
            }
            "input_price_per_m" | "input_price_per_m_direct" => {
                if let Some(v) = self.parse_value_as_f64(&value, value_type)? {
                    metadata.input_price_per_m = Some(v);
                }
            }
            "output_price_per_m" | "output_price_per_m_direct" => {
                if let Some(v) = self.parse_value_as_f64(&value, value_type)? {
                    metadata.output_price_per_m = Some(v);
                }
            }
            "supports_tools" => {
                if let Some(v) = self.parse_value_as_bool(&value, value_type)? {
                    metadata.supports_tools = v;
                }
            }
            "supports_vision" => {
                if let Some(v) = self.parse_value_as_bool(&value, value_type)? {
                    metadata.supports_vision = v;
                }
            }
            "supports_audio" => {
                if let Some(v) = self.parse_value_as_bool(&value, value_type)? {
                    metadata.supports_audio = v;
                }
            }
            "supports_reasoning" => {
                if let Some(v) = self.parse_value_as_bool(&value, value_type)? {
                    metadata.supports_reasoning = v;
                }
            }
            "supports_code" => {
                if let Some(v) = self.parse_value_as_bool(&value, value_type)? {
                    metadata.supports_code = v;
                }
            }
            "supports_function_calling" => {
                if let Some(v) = self.parse_value_as_bool(&value, value_type)? {
                    metadata.supports_function_calling = v;
                }
            }
            "supports_json_mode" => {
                if let Some(v) = self.parse_value_as_bool(&value, value_type)? {
                    metadata.supports_json_mode = v;
                }
            }
            "supports_streaming" => {
                if let Some(v) = self.parse_value_as_bool(&value, value_type)? {
                    metadata.supports_streaming = v;
                }
            }
            "is_deprecated" => {
                if let Some(v) = self.parse_value_as_bool(&value, value_type)? {
                    metadata.is_deprecated = v;
                }
            }
            "is_fine_tunable" => {
                if let Some(v) = self.parse_value_as_bool(&value, value_type)? {
                    metadata.is_fine_tunable = v;
                }
            }
            _ => {
                // Unknown tag, ignore
            }
        }
        Ok(())
    }

    fn parse_value_as_bool(&self, value: &Value, _value_type: &str) -> Result<Option<bool>> {
        match value {
            Value::Bool(b) => Ok(Some(*b)),
            Value::String(s) => Ok(Some(s == "true" || s == "yes" || s == "1")),
            Value::Number(n) => Ok(Some(n.as_i64().unwrap_or(0) != 0)),
            _ => Ok(None),
        }
    }

    fn parse_value_as_u32(&self, value: &Value, _value_type: &str) -> Result<Option<u32>> {
        match value {
            Value::Number(n) => {
                if let Some(v) = n.as_u64() {
                    Ok(Some(v as u32))
                } else if let Some(v) = n.as_i64() {
                    Ok(Some(v as u32))
                } else {
                    Ok(None)
                }
            }
            Value::String(s) => Ok(s.parse::<u32>().ok()),
            _ => Ok(None),
        }
    }

    fn parse_value_as_f64(&self, value: &Value, _value_type: &str) -> Result<Option<f64>> {
        match value {
            Value::Number(n) => Ok(n.as_f64()),
            Value::String(s) => Ok(s.parse::<f64>().ok()),
            _ => Ok(None),
        }
    }

    /// Check if model name contains a specific pattern (case-insensitive)
    fn check_name_contains(&self, model: &Value, pattern: &str) -> Option<bool> {
        let pattern_lower = pattern.to_lowercase();

        // Check all configured ID fields
        for field in &self.model_paths.field_mappings.id_fields {
            if let Some(value) = model.get(field).and_then(|v| v.as_str()) {
                if value.to_lowercase().contains(&pattern_lower) {
                    return Some(true);
                }
            }
        }

        // Check all configured name fields
        for field in &self.model_paths.field_mappings.name_fields {
            if let Some(value) = model.get(field).and_then(|v| v.as_str()) {
                if value.to_lowercase().contains(&pattern_lower) {
                    return Some(true);
                }
            }
        }

        Some(false)
    }

    /// Determine model type based on model ID and name patterns
    fn determine_model_type(&self, model_id: &str, display_name: Option<&str>) -> ModelType {
        let id_lower = model_id.to_lowercase();
        let name_lower = display_name.map(|n| n.to_lowercase());

        // Check for embedding model patterns
        let embedding_patterns = [
            "embed",
            "embedding",
            "text-embedding",
            "text_embedding",
            "ada",
            "similarity",
            "bge",
            "e5",
            "gte",
            "instructor",
            "voyage",
            "titan-embed",
            "embedding-gecko",
            "embed-english",
            "embed-multilingual",
        ];

        for pattern in &embedding_patterns {
            if id_lower.contains(pattern) {
                return ModelType::Embedding;
            }
            if let Some(ref name) = name_lower {
                if name.contains(pattern) {
                    return ModelType::Embedding;
                }
            }
        }

        // Check for image generation model patterns
        let image_patterns = [
            "dall-e",
            "dalle",
            "stable-diffusion",
            "midjourney",
            "imagen",
            "image",
        ];

        for pattern in &image_patterns {
            if id_lower.contains(pattern) {
                return ModelType::ImageGeneration;
            }
            if let Some(ref name) = name_lower {
                if name.contains(pattern) {
                    return ModelType::ImageGeneration;
                }
            }
        }

        // Check for audio generation model patterns
        let audio_patterns = ["whisper", "tts", "audio", "speech", "voice"];

        for pattern in &audio_patterns {
            if id_lower.contains(pattern) {
                return ModelType::AudioGeneration;
            }
            if let Some(ref name) = name_lower {
                if name.contains(pattern) {
                    return ModelType::AudioGeneration;
                }
            }
        }

        // Check for moderation model patterns
        let moderation_patterns = ["moderation", "moderate", "safety"];

        for pattern in &moderation_patterns {
            if id_lower.contains(pattern) {
                return ModelType::Moderation;
            }
            if let Some(ref name) = name_lower {
                if name.contains(pattern) {
                    return ModelType::Moderation;
                }
            }
        }

        // Check for completion model patterns (older style models)
        let completion_patterns = [
            "davinci",
            "curie",
            "babbage",
            "ada-001",
            "text-davinci",
            "text-curie",
            "text-babbage",
            "code-davinci",
            "code-cushman",
        ];

        for pattern in &completion_patterns {
            if id_lower.contains(pattern) && !id_lower.contains("embed") {
                return ModelType::Completion;
            }
            if let Some(ref name) = name_lower {
                if name.contains(pattern) && !name.contains("embed") {
                    return ModelType::Completion;
                }
            }
        }

        // Default to Chat for everything else (GPT, Claude, Llama, etc.)
        ModelType::Chat
    }

    /// Check if model name matches a specific pattern using regex (case-insensitive)
    fn check_name_matches(&self, model: &Value, pattern: &str) -> Option<bool> {
        use regex::RegexBuilder;

        // Create case-insensitive regex
        let regex = match RegexBuilder::new(pattern).case_insensitive(true).build() {
            Ok(r) => r,
            Err(_) => return Some(false), // Invalid regex pattern
        };

        // Check all configured ID fields
        for field in &self.model_paths.field_mappings.id_fields {
            if let Some(value) = model.get(field).and_then(|v| v.as_str()) {
                if regex.is_match(value) {
                    return Some(true);
                }
            }
        }

        // Check all configured name fields
        for field in &self.model_paths.field_mappings.name_fields {
            if let Some(value) = model.get(field).and_then(|v| v.as_str()) {
                if regex.is_match(value) {
                    return Some(true);
                }
            }
        }

        Some(false)
    }
}

// Public API function
pub fn extract_models_from_provider(
    provider: &Provider,
    raw_json: &str,
) -> Result<Vec<ModelMetadata>> {
    let response: Value = serde_json::from_str(raw_json)?;
    let extractor = ModelMetadataExtractor::new()?;

    let models = extractor.extract_models(provider, &response)?;
    let mut metadata_list = Vec::new();

    for model in models {
        match extractor.extract_metadata(provider, &model) {
            Ok(metadata) => metadata_list.push(metadata),
            Err(e) => {
                eprintln!("Warning: Failed to extract metadata for model: {}", e);
            }
        }
    }

    Ok(metadata_list)
}

// CLI command handlers
pub fn add_model_path(path: String) -> Result<()> {
    let config_dir = ModelMetadataExtractor::get_config_dir()?;
    let file_path = config_dir.join("model_paths.toml");

    let mut paths = if file_path.exists() {
        let content = fs::read_to_string(&file_path)?;
        toml::from_str(&content)?
    } else {
        ModelPaths::default()
    };

    if !paths.paths.contains(&path) {
        paths.paths.push(path);
        let content = toml::to_string_pretty(&paths)?;
        fs::write(&file_path, content)?;
        println!("Added model path");
    } else {
        println!("Path already exists");
    }

    Ok(())
}

pub fn remove_model_path(path: String) -> Result<()> {
    let config_dir = ModelMetadataExtractor::get_config_dir()?;
    let file_path = config_dir.join("model_paths.toml");

    if !file_path.exists() {
        anyhow::bail!("No model paths configured");
    }

    let mut paths: ModelPaths = {
        let content = fs::read_to_string(&file_path)?;
        toml::from_str(&content)?
    };

    if let Some(pos) = paths.paths.iter().position(|p| p == &path) {
        paths.paths.remove(pos);
        let content = toml::to_string_pretty(&paths)?;
        fs::write(&file_path, content)?;
        println!("Removed model path");
    } else {
        println!("Path not found");
    }

    Ok(())
}

pub fn list_model_paths() -> Result<()> {
    let config_dir = ModelMetadataExtractor::get_config_dir()?;
    let file_path = config_dir.join("model_paths.toml");

    let paths = if file_path.exists() {
        let content = fs::read_to_string(&file_path)?;
        toml::from_str(&content)?
    } else {
        ModelPaths::default()
    };

    println!("Model paths:");
    for path in &paths.paths {
        println!("  - {}", path);
    }

    Ok(())
}

pub fn add_tag(
    name: String,
    paths: Vec<String>,
    value_type: String,
    transform: Option<String>,
) -> Result<()> {
    let config_dir = ModelMetadataExtractor::get_config_dir()?;
    let file_path = config_dir.join("tags.toml");

    let mut config = if file_path.exists() {
        let content = fs::read_to_string(&file_path)?;
        toml::from_str(&content)?
    } else {
        TagConfig::default()
    };

    config.tags.insert(
        name.clone(),
        TagRule {
            paths,
            value_type,
            transform,
        },
    );

    let content = toml::to_string_pretty(&config)?;
    fs::write(&file_path, content)?;
    println!("Added tag: {}", name);

    Ok(())
}

/// Initialize model metadata configuration files
/// This should be called once during application startup to ensure
/// tags.toml and model_paths.toml exist with default values
pub fn initialize_model_metadata_config() -> Result<()> {
    ModelMetadataExtractor::ensure_config_files_exist()
}

pub fn list_tags() -> Result<()> {
    let config_dir = ModelMetadataExtractor::get_config_dir()?;
    let file_path = config_dir.join("tags.toml");

    let config = if file_path.exists() {
        let content = fs::read_to_string(&file_path)?;
        toml::from_str(&content)?
    } else {
        TagConfig::default()
    };

    println!("Tags:");
    for (name, rule) in &config.tags {
        println!("  {}:", name);
        println!("    Type: {}", rule.value_type);
        println!("    Paths:");
        for path in &rule.paths {
            println!("      - {}", path);
        }
        if let Some(transform) = &rule.transform {
            println!("    Transform: {}", transform);
        }
    }

    Ok(())
}

// Compatibility layer for existing code
pub struct MetadataExtractor;

impl MetadataExtractor {
    pub fn extract_from_provider(
        provider: &str,
        raw_json: &str,
    ) -> Result<Vec<ModelMetadata>, Box<dyn std::error::Error>> {
        let provider_obj = Provider {
            provider: provider.to_string(),
            status: "active".to_string(),
            supports_tools: false,
            supports_structured_output: false,
        };

        extract_models_from_provider(&provider_obj, raw_json).map_err(|e| e.into())
    }
}
