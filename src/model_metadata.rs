use serde::{Deserialize, Serialize};

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

pub struct MetadataExtractor;

impl MetadataExtractor {
    pub fn extract_from_provider(provider: &str, raw_json: &str) -> Result<Vec<ModelMetadata>, Box<dyn std::error::Error>> {
        match provider {
            "openai" => Self::extract_openai(raw_json),
            "groq" => Self::extract_groq(raw_json),
            "claude" => Self::extract_claude(raw_json),
            "cohere" => Self::extract_cohere(raw_json),
            "mistral" => Self::extract_mistral(raw_json),
            "fireworks" => Self::extract_fireworks(raw_json),
            "nvidia" => Self::extract_nvidia(raw_json),
            "openrouter" => Self::extract_openrouter(raw_json),
            "kilo" => Self::extract_kilo(raw_json),
            "venice" => Self::extract_venice(raw_json),
            "requesty" => Self::extract_requesty(raw_json),
            "chutes" => Self::extract_chutes(raw_json),
            "github" => Self::extract_github(raw_json),
            "together" => Self::extract_together(raw_json),
            "gemini" => Self::extract_gemini(raw_json),
            "ollama" => Self::extract_ollama(raw_json),
            _ => Self::extract_generic(provider, raw_json),
        }
    }
    
    fn extract_openai(raw_json: &str) -> Result<Vec<ModelMetadata>, Box<dyn std::error::Error>> {
    #[derive(Deserialize)]
    struct OpenAIResponse {
        data: Vec<OpenAIModel>,
    }
    
    #[derive(Deserialize, Serialize)]
    struct OpenAIModel {
        id: String,
        object: String,
        owned_by: String,
        created: i64,
        capabilities: Option<Vec<String>>, // Expect potential capabilities array
    }
    
    let response: OpenAIResponse = serde_json::from_str(raw_json)?;
    let mut models = Vec::new();
    
    for model in response.data {
        let raw_data = serde_json::to_value(&model)?;
        let mut metadata = ModelMetadata {
            id: model.id.clone(),
            provider: "openai".to_string(),
            owned_by: Some(model.owned_by),
            created: Some(model.created),
            raw_data,
            ..Default::default()
        };
        
        // Extract capabilities if present
        if let Some(ref capabilities) = model.capabilities {
            capabilities.iter().for_each(|capability| match capability.as_str() {
                "tools" | "function-calling" => metadata.supports_tools = true,
                "json-mode" => metadata.supports_json_mode = true,
                _ => (),
            });
        }
        
        models.push(metadata);
    }
    
    Ok(models)
}
    
    fn extract_groq(raw_json: &str) -> Result<Vec<ModelMetadata>, Box<dyn std::error::Error>> {
        #[derive(Deserialize)]
        struct GroqResponse {
            data: Vec<GroqModel>,
        }
        
        #[derive(Deserialize, Serialize)]
        struct GroqModel {
            id: String,
            object: String,
            owned_by: String,
            created: i64,
            active: bool,
            context_window: u32,
            max_completion_tokens: u32,
        }
        
        let response: GroqResponse = serde_json::from_str(raw_json)?;
        let mut models = Vec::new();
        
        for model in response.data {
            let raw_data = serde_json::to_value(&model)?;
            let metadata = ModelMetadata {
                id: model.id.clone(),
                provider: "groq".to_string(),
                owned_by: Some(model.owned_by),
                created: Some(model.created),
                context_length: Some(model.context_window),
                max_output_tokens: Some(model.max_completion_tokens),
                is_deprecated: !model.active,
                raw_data,
                ..Default::default()
            };
            
// Self::infer_groq_capabilities(&mut metadata); (removed call due to lack of explicit capability info)
            models.push(metadata);
        }
        
        Ok(models)
    }
    
    fn extract_claude(raw_json: &str) -> Result<Vec<ModelMetadata>, Box<dyn std::error::Error>> {
        #[derive(Deserialize)]
        struct ClaudeResponse {
            data: Vec<ClaudeModel>,
        }
        
        #[derive(Deserialize, Serialize)]
        struct ClaudeModel {
            id: String,
            display_name: String,
            created_at: String,
            #[serde(rename = "type")]
            model_type: String,
        }
        
        let response: ClaudeResponse = serde_json::from_str(raw_json)?;
        let mut models = Vec::new();
        
        for model in response.data {
            let raw_data = serde_json::to_value(&model)?;
            let metadata = ModelMetadata {
                id: model.id.clone(),
                provider: "claude".to_string(),
                display_name: Some(model.display_name),
                raw_data,
                ..Default::default()
            };
            
// Self::infer_claude_capabilities(&mut metadata); (removed call due to lack of explicit capability info)
            models.push(metadata);
        }
        
        Ok(models)
    }
    
    fn extract_cohere(raw_json: &str) -> Result<Vec<ModelMetadata>, Box<dyn std::error::Error>> {
        #[derive(Deserialize)]
        struct CohereResponse {
            models: Vec<CohereModel>,
        }
        
        #[derive(Deserialize, Serialize)]
        struct CohereModel {
            name: String,
            context_length: u32,
            endpoints: Vec<String>,
            features: Option<Vec<String>>,
            supports_vision: bool,
            finetuned: bool,
        }
        
        let response: CohereResponse = serde_json::from_str(raw_json)?;
        let mut models = Vec::new();
        
        for model in response.models {
            let raw_data = serde_json::to_value(&model)?;
            let features = model.features.clone().unwrap_or_default();
            
            let metadata = ModelMetadata {
                id: model.name.clone(),
                provider: "cohere".to_string(),
                context_length: Some(model.context_length),
                supports_vision: model.supports_vision,
                supports_tools: features.contains(&"tools".to_string()),
                supports_function_calling: features.contains(&"strict_tools".to_string()),
                supports_json_mode: features.contains(&"json_mode".to_string()) || features.contains(&"json_schema".to_string()),
                is_fine_tunable: model.finetuned,
                model_type: Self::infer_cohere_model_type(&model.endpoints),
                raw_data,
                ..Default::default()
            };
            
            models.push(metadata);
        }
        
        Ok(models)
    }
    
    fn extract_mistral(raw_json: &str) -> Result<Vec<ModelMetadata>, Box<dyn std::error::Error>> {
        #[derive(Deserialize)]
        struct MistralResponse {
            data: Vec<MistralModel>,
        }
        
        #[derive(Deserialize, Serialize)]
        struct MistralModel {
            id: String,
            name: String,
            description: String,
            max_context_length: u32,
            capabilities: MistralCapabilities,
            created: i64,
            owned_by: String,
        }
        
        #[derive(Deserialize, Serialize)]
        struct MistralCapabilities {
            completion_chat: bool,
            completion_fim: bool,
            function_calling: bool,
            fine_tuning: bool,
            vision: bool,
        }
        
        let response: MistralResponse = serde_json::from_str(raw_json)?;
        let mut models = Vec::new();
        
        for model in response.data {
            let raw_data = serde_json::to_value(&model)?;
            let metadata = ModelMetadata {
                id: model.id.clone(),
                provider: "mistral".to_string(),
                display_name: Some(model.name),
                description: Some(model.description),
                owned_by: Some(model.owned_by),
                created: Some(model.created),
                context_length: Some(model.max_context_length),
                supports_function_calling: model.capabilities.function_calling,
                supports_vision: model.capabilities.vision,
                supports_code: model.capabilities.completion_fim,
                is_fine_tunable: model.capabilities.fine_tuning,
                model_type: if model.capabilities.completion_chat { ModelType::Chat } else { ModelType::Completion },
                raw_data,
                ..Default::default()
            };
            
            models.push(metadata);
        }
        
        Ok(models)
    }
    
    fn extract_fireworks(raw_json: &str) -> Result<Vec<ModelMetadata>, Box<dyn std::error::Error>> {
        #[derive(Deserialize)]
        struct FireworksResponse {
            data: Vec<FireworksModel>,
        }
        
        #[derive(Deserialize, Serialize)]
        struct FireworksModel {
            id: String,
            owned_by: String,
            created: i64,
            context_length: Option<u32>,
            supports_chat: bool,
            supports_image_input: bool,
            supports_tools: bool,
        }
        
        let response: FireworksResponse = serde_json::from_str(raw_json)?;
        let mut models = Vec::new();
        
        for model in response.data {
            let raw_data = serde_json::to_value(&model)?;
            let metadata = ModelMetadata {
                id: model.id.clone(),
                provider: "fireworks".to_string(),
                owned_by: Some(model.owned_by),
                created: Some(model.created),
                context_length: model.context_length,
                supports_tools: model.supports_tools,
                supports_vision: model.supports_image_input,
                model_type: if model.supports_chat { ModelType::Chat } else { ModelType::Other("generation".to_string()) },
                raw_data,
                ..Default::default()
            };
            
            models.push(metadata);
        }
        
        Ok(models)
    }
    
    fn extract_nvidia(raw_json: &str) -> Result<Vec<ModelMetadata>, Box<dyn std::error::Error>> {
        // NVIDIA uses OpenAI-compatible format
        Self::extract_openai(raw_json).map(|mut models| {
            for model in &mut models {
                model.provider = "nvidia".to_string();
                // Reset all inferred capabilities due to lack of explicit NVIDIA capability info
                model.supports_tools = false;
                model.supports_function_calling = false;
                model.supports_json_mode = false;
                model.supports_streaming = false;
                model.supports_vision = false;
                model.supports_audio = false;
                model.supports_reasoning = false;
                model.supports_code = false;
            }
            models
        })
    }
    
    fn extract_openrouter(raw_json: &str) -> Result<Vec<ModelMetadata>, Box<dyn std::error::Error>> {
        #[derive(Deserialize)]
        struct OpenRouterResponse {
            data: Vec<OpenRouterModel>,
        }
        
        #[derive(Deserialize, Serialize)]
        struct OpenRouterModel {
            id: String,
            name: String,
            description: Option<String>,
            context_length: Option<u32>,
            created: Option<i64>,
            supported_parameters: Option<Vec<String>>,
            architecture: Option<OpenRouterArchitecture>,
            pricing: Option<OpenRouterPricing>,
        }
        
        #[derive(Deserialize, Serialize)]
        struct OpenRouterArchitecture {
            input_modalities: Option<Vec<String>>,
            output_modalities: Option<Vec<String>>,
            modality: Option<String>,
        }
        
        #[derive(Deserialize, Serialize)]
        struct OpenRouterPricing {
            prompt: Option<String>,
            completion: Option<String>,
            image: Option<String>,
        }
        
        let response: OpenRouterResponse = serde_json::from_str(raw_json)?;
        let mut models = Vec::new();
        
        for model in response.data {
            let raw_data = serde_json::to_value(&model)?;
            
            // Extract capabilities from supported_parameters
            let supported_params = model.supported_parameters.unwrap_or_default();
            let supports_tools = supported_params.contains(&"tools".to_string());
            let supports_reasoning = supported_params.contains(&"reasoning".to_string());
            
            // Extract vision support from architecture
            let supports_vision = model.architecture
                .as_ref()
                .and_then(|arch| arch.input_modalities.as_ref())
                .map_or(false, |modalities| modalities.contains(&"image".to_string()));
            
            // Only use explicit code support fields from JSON - no inference
            let supports_code = false; // OpenRouter doesn't provide explicit code capability flags
            
            // Extract pricing (convert from per-token to per-million-token)
            let input_price_per_m = model.pricing
                .as_ref()
                .and_then(|p| p.prompt.as_ref())
                .and_then(|price_str| price_str.parse::<f64>().ok())
                .map(|price| price * 1_000_000.0);
                
            let output_price_per_m = model.pricing
                .as_ref()
                .and_then(|p| p.completion.as_ref())
                .and_then(|price_str| price_str.parse::<f64>().ok())
                .map(|price| price * 1_000_000.0);
            
            let metadata = ModelMetadata {
                id: model.id.clone(),
                provider: "openrouter".to_string(),
                display_name: Some(model.name),
                description: model.description,
                created: model.created,
                context_length: model.context_length,
                input_price_per_m,
                output_price_per_m,
                // Capabilities
                supports_tools,
                supports_function_calling: supports_tools, // Same as tools for OpenRouter
                supports_vision,
                supports_reasoning,
                supports_code,
                supports_audio: false, // OpenRouter doesn't seem to have audio models
                supports_json_mode: supported_params.contains(&"response_format".to_string()),
                supports_streaming: true, // Most models support streaming
                model_type: ModelType::Chat,
                is_deprecated: false,
                is_fine_tunable: false,
                raw_data,
                ..Default::default()
            };
            
            models.push(metadata);
        }
        
        Ok(models)
    }
    
    fn extract_kilo(raw_json: &str) -> Result<Vec<ModelMetadata>, Box<dyn std::error::Error>> {
        // Kilo uses the same format as OpenRouter, so we can reuse the extraction logic
        Self::extract_openrouter(raw_json).map(|mut models| {
            for model in &mut models {
                model.provider = "kilo".to_string();
            }
            models
        })
    }
    
    fn extract_venice(raw_json: &str) -> Result<Vec<ModelMetadata>, Box<dyn std::error::Error>> {
        #[derive(Deserialize)]
        struct VeniceResponse {
            data: Vec<VeniceModel>,
        }
        
        #[derive(Deserialize, Serialize)]
        struct VeniceModel {
            id: String,
            model_spec: Option<VeniceModelSpec>,
        }
        
        #[derive(Deserialize, Serialize)]
        struct VeniceModelSpec {
            name: Option<String>,
            #[serde(rename = "availableContextTokens")]
            available_context_tokens: Option<u32>,
            capabilities: Option<VeniceCapabilities>,
            pricing: Option<VenicePricing>,
        }
        
        #[derive(Deserialize, Serialize)]
        struct VeniceCapabilities {
            #[serde(rename = "supportsFunctionCalling")]
            supports_function_calling: Option<bool>,
            #[serde(rename = "supportsVision")]
            supports_vision: Option<bool>,
            #[serde(rename = "optimizedForCode")]
            optimized_for_code: Option<bool>,
            #[serde(rename = "supportsReasoning")]
            supports_reasoning: Option<bool>,
        }
        
        #[derive(Deserialize, Serialize)]
        struct VenicePricing {
            input: Option<VenicePriceInfo>,
            output: Option<VenicePriceInfo>,
        }
        
        #[derive(Deserialize, Serialize)]
        struct VenicePriceInfo {
            usd: Option<f64>,
        }
        
        let response: VeniceResponse = serde_json::from_str(raw_json)?;
        let mut models = Vec::new();
        
        for model in response.data {
            let raw_data = serde_json::to_value(&model)?;
            let mut metadata = ModelMetadata {
                id: model.id.clone(),
                provider: "venice".to_string(),
                raw_data,
                ..Default::default()
            };
            
            // Extract from model_spec if available
            if let Some(ref model_spec) = model.model_spec {
                // Extract capabilities from the capabilities object
                if let Some(ref capabilities) = model_spec.capabilities {
                    metadata.supports_function_calling = capabilities.supports_function_calling.unwrap_or(false);
                    metadata.supports_tools = capabilities.supports_function_calling.unwrap_or(false);
                    metadata.supports_vision = capabilities.supports_vision.unwrap_or(false);
                    metadata.supports_code = capabilities.optimized_for_code.unwrap_or(false);
                    metadata.supports_reasoning = capabilities.supports_reasoning.unwrap_or(false);
                }
                
                // Extract context length
                if let Some(context_tokens) = model_spec.available_context_tokens {
                    metadata.context_length = Some(context_tokens);
                }
                
                // Extract pricing (convert to per million tokens)
                if let Some(ref pricing) = model_spec.pricing {
                    if let Some(ref input) = pricing.input {
                        if let Some(usd) = input.usd {
                            metadata.input_price_per_m = Some(usd);
                        }
                    }
                    if let Some(ref output) = pricing.output {
                        if let Some(usd) = output.usd {
                            metadata.output_price_per_m = Some(usd);
                        }
                    }
                }
                
                // Extract display name
                if let Some(ref name) = model_spec.name {
                    metadata.display_name = Some(name.clone());
                }
            }
            
            // Fallback to model ID if no display name
            if metadata.display_name.is_none() {
                metadata.display_name = Some(model.id.clone());
            }
            
            models.push(metadata);
        }
        
        Ok(models)
    }
    
    fn extract_ollama(raw_json: &str) -> Result<Vec<ModelMetadata>, Box<dyn std::error::Error>> {
        #[derive(Deserialize)]
        struct OllamaResponse {
            data: Vec<OllamaModel>,
        }
        
        #[derive(Deserialize, Serialize)]
        struct OllamaModel {
            id: String,
            object: String,
            owned_by: String,
            created: i64,
        }
        
        let response: OllamaResponse = serde_json::from_str(raw_json)?;
        let mut models = Vec::new();
        
        for model in response.data {
            let raw_data = serde_json::to_value(&model)?;
            let metadata = ModelMetadata {
                id: model.id.clone(),
                provider: "ollama".to_string(),
                owned_by: Some(model.owned_by),
                created: Some(model.created),
                raw_data,
                // Do NOT infer any capabilities - only use explicit JSON data
                // All capabilities remain false unless explicitly stated in JSON
                ..Default::default()
            };
            
            models.push(metadata);
        }
        
        Ok(models)
    }
    
    fn extract_generic(provider: &str, raw_json: &str) -> Result<Vec<ModelMetadata>, Box<dyn std::error::Error>> {
        // Try to parse as standard OpenAI format first
        if let Ok(openai_models) = Self::extract_openai(raw_json) {
            // For non-OpenAI providers, we need to reset the inferred capabilities
            // and only use explicit data from the JSON
            
            // Re-parse the JSON to get access to the original model objects
            let value: serde_json::Value = serde_json::from_str(raw_json)?;
            let mut model_raw_data_map = std::collections::HashMap::new();
            
            // Build a map of model ID to its raw JSON data
            if let Some(data) = value.get("data").and_then(|d| d.as_array()) {
                for model_value in data {
                    if let Some(id) = model_value.get("id").and_then(|i| i.as_str()) {
                        model_raw_data_map.insert(id.to_string(), model_value.clone());
                    }
                }
            }
            
            return Ok(openai_models.into_iter().map(|model| {
                let mut metadata = ModelMetadata {
                    id: model.id.clone(),
                    provider: provider.to_string(),
                    display_name: model.display_name,
                    description: model.description,
                    owned_by: model.owned_by,
                    created: model.created,
                    context_length: model.context_length,
                    max_input_tokens: model.max_input_tokens,
                    max_output_tokens: model.max_output_tokens,
                    input_price_per_m: model.input_price_per_m,
                    output_price_per_m: model.output_price_per_m,
                    model_type: model.model_type,
                    raw_data: model_raw_data_map.get(&model.id).unwrap_or(&model.raw_data).clone(),
                    // Reset all capability flags to false - only set based on explicit JSON data
                    ..Default::default()
                };
                
                // Only extract explicit capabilities from the raw JSON data
                if let Some(raw_obj) = metadata.raw_data.as_object() {
                    // Check for explicit capability fields in the JSON
                    if let Some(tools) = raw_obj.get("supports_tools").and_then(|v| v.as_bool()) {
                        metadata.supports_tools = tools;
                    }
                    if let Some(vision) = raw_obj.get("supports_vision").and_then(|v| v.as_bool()) {
                        metadata.supports_vision = vision;
                    }
                    // Also check for supports_image_input (used by Hyperbolic and others)
                    if let Some(image_input) = raw_obj.get("supports_image_input").and_then(|v| v.as_bool()) {
                        metadata.supports_vision = image_input;
                    }
                    if let Some(audio) = raw_obj.get("supports_audio").and_then(|v| v.as_bool()) {
                        metadata.supports_audio = audio;
                    }
                    if let Some(reasoning) = raw_obj.get("supports_reasoning").and_then(|v| v.as_bool()) {
                        metadata.supports_reasoning = reasoning;
                    }
                    if let Some(code) = raw_obj.get("supports_code").and_then(|v| v.as_bool()) {
                        metadata.supports_code = code;
                    }
                    if let Some(func_calling) = raw_obj.get("supports_function_calling").and_then(|v| v.as_bool()) {
                        metadata.supports_function_calling = func_calling;
                    }
                    if let Some(json_mode) = raw_obj.get("supports_json_mode").and_then(|v| v.as_bool()) {
                        metadata.supports_json_mode = json_mode;
                    }
                    if let Some(streaming) = raw_obj.get("supports_streaming").and_then(|v| v.as_bool()) {
                        metadata.supports_streaming = streaming;
                    }
                    if let Some(deprecated) = raw_obj.get("is_deprecated").and_then(|v| v.as_bool()) {
                        metadata.is_deprecated = deprecated;
                    }
                    if let Some(fine_tunable) = raw_obj.get("is_fine_tunable").and_then(|v| v.as_bool()) {
                        metadata.is_fine_tunable = fine_tunable;
                    }
                }
                
                metadata
            }).collect());
        }
        
        // If that fails, try to extract basic model list
        let value: serde_json::Value = serde_json::from_str(raw_json)?;
let mut models = Vec::new();

    // First, try to get models from "data" field (wrapped format)
    if let Some(data) = value.get("data").and_then(|d| d.as_array()) {
        for model_value in data {
            if let Some(id) = model_value.get("id").and_then(|i| i.as_str()) {
                let mut metadata = ModelMetadata {
                    id: id.to_string(),
                    provider: provider.to_string(),
                    raw_data: model_value.clone(),
                    ..Default::default()
                };
                if let Some(supports_image_input) = model_value.get("supports_image_input").and_then(|v| v.as_bool()) {
                    metadata.supports_vision = supports_image_input;
                }
                models.push(metadata);
            }
        }
    }
    // If no "data" field, try to parse as direct array (providers like together, github)
    else if let Some(array) = value.as_array() {
        for model_value in array {
            if let Some(id) = model_value.get("id").and_then(|i| i.as_str()) {
                let mut metadata = ModelMetadata {
                    id: id.to_string(),
                    provider: provider.to_string(),
                    raw_data: model_value.clone(),
                    ..Default::default()
                };
                if let Some(supports_image_input) = model_value.get("supports_image_input").and_then(|v| v.as_bool()) {
                    metadata.supports_vision = supports_image_input;
                }
                models.push(metadata);
            }
        }
    }
        
        Ok(models)
    }
    
    fn extract_requesty(raw_json: &str) -> Result<Vec<ModelMetadata>, Box<dyn std::error::Error>> {
        #[derive(Deserialize)]
        struct RequestyResponse {
            data: Vec<RequestyModel>,
        }
        
        #[derive(Deserialize, Serialize)]
        struct RequestyModel {
            id: String,
            description: Option<String>,
            context_window: Option<u32>,
            max_output_tokens: Option<u32>,
            input_price: Option<f64>,
            output_price: Option<f64>,
            supports_reasoning: Option<bool>,
            supports_vision: Option<bool>,
            supports_caching: Option<bool>,
            supports_computer_use: Option<bool>,
            created: Option<i64>,
            owned_by: Option<String>,
        }
        
        let response: RequestyResponse = serde_json::from_str(raw_json)?;
        let mut models = Vec::new();
        
        for model in response.data {
            let raw_data = serde_json::to_value(&model)?;
            let metadata = ModelMetadata {
                id: model.id.clone(),
                provider: "requesty".to_string(),
                description: model.description,
                owned_by: model.owned_by,
                created: model.created,
                context_length: model.context_window,
                max_output_tokens: model.max_output_tokens,
                // Convert from per-token to per-million-token pricing
                input_price_per_m: model.input_price.map(|p| p * 1_000_000.0),
                output_price_per_m: model.output_price.map(|p| p * 1_000_000.0),
                // Extract capabilities from explicit fields
                supports_reasoning: model.supports_reasoning.unwrap_or(false),
                supports_vision: model.supports_vision.unwrap_or(false),
                // Requesty doesn't seem to have explicit tools support field
                // Don't assume tools support unless explicitly stated
                supports_tools: false,
                supports_function_calling: false,
                // Other capabilities
                supports_audio: false,
                supports_code: false,
                supports_json_mode: false,
                supports_streaming: true, // Most modern models support streaming
                model_type: ModelType::Chat,
                is_deprecated: false,
                is_fine_tunable: false,
                display_name: Some(model.id.clone()),
                raw_data,
                ..Default::default()
            };
            
            models.push(metadata);
        }
        
        Ok(models)
    }
    
    fn extract_chutes(raw_json: &str) -> Result<Vec<ModelMetadata>, Box<dyn std::error::Error>> {
        #[derive(Deserialize)]
        struct ChutesResponse {
            data: Vec<ChutesModel>,
        }
        
        #[derive(Deserialize, Serialize)]
        struct ChutesModel {
            id: String,
            object: String,
            owned_by: String,
            created: i64,
            max_model_len: u32,
            price: ChutesPrice,
            root: String,
        }
        
        #[derive(Deserialize, Serialize)]
        struct ChutesPrice {
            tao: f64,
            usd: f64,
        }
        
        let response: ChutesResponse = serde_json::from_str(raw_json)?;
        let mut models = Vec::new();
        
        for model in response.data {
            let raw_data = serde_json::to_value(&model)?;
            
            // Convert single USD price to per-million-token pricing
            // Treat the USD price as per 1M input tokens, same for output
            let price_per_m = if model.price.usd > 0.0 {
                Some(model.price.usd)
            } else {
                None
            };
            
            let metadata = ModelMetadata {
                id: model.id.clone(),
                provider: "chutes".to_string(),
                owned_by: Some(model.owned_by),
                created: Some(model.created),
                context_length: Some(model.max_model_len),
                // Use same price for input and output as suggested
                input_price_per_m: price_per_m,
                output_price_per_m: price_per_m,
                // Chutes doesn't specify tools support, so don't assume it
                supports_tools: false,
                supports_function_calling: false,
                supports_vision: false,
                supports_audio: false,
                supports_reasoning: false,
                supports_code: false,
                supports_json_mode: false,
                supports_streaming: true, // Most modern models support streaming
                model_type: ModelType::Chat,
                is_deprecated: false,
                is_fine_tunable: false,
                display_name: Some(model.id.clone()),
                raw_data,
                ..Default::default()
            };
            
            models.push(metadata);
        }
        
        Ok(models)
    }
    
    fn extract_github(raw_json: &str) -> Result<Vec<ModelMetadata>, Box<dyn std::error::Error>> {
        #[derive(Deserialize, Serialize)]
        struct GitHubModel {
            id: String,
            name: String,
            publisher: Option<String>,
            summary: Option<String>,
            capabilities: Option<Vec<String>>,
            supported_input_modalities: Option<Vec<String>>,
            supported_output_modalities: Option<Vec<String>>,
            limits: Option<GitHubLimits>,
            tags: Option<Vec<String>>,
            version: Option<String>,
            rate_limit_tier: Option<String>,
            registry: Option<String>,
            html_url: Option<String>,
        }
        
        #[derive(Deserialize, Serialize)]
        struct GitHubLimits {
            max_input_tokens: Option<u32>,
            max_output_tokens: Option<u32>,
        }
        
        // GitHub returns a direct array of models
        let github_models: Vec<GitHubModel> = serde_json::from_str(raw_json)?;
        let mut models = Vec::new();
        
        for model in github_models {
            let raw_data = serde_json::to_value(&model)?;
            
            // Extract capabilities
            let capabilities = model.capabilities.unwrap_or_default();
            let supports_tools = capabilities.contains(&"tool-calling".to_string());
            let supports_function_calling = supports_tools; // Same as tools for GitHub
            let supports_streaming = capabilities.contains(&"streaming".to_string());
            let supports_reasoning = capabilities.contains(&"reasoning".to_string());
            
            // Extract vision support from input modalities
            let input_modalities = model.supported_input_modalities.unwrap_or_default();
            let supports_vision = input_modalities.contains(&"image".to_string());
            let supports_audio = input_modalities.contains(&"audio".to_string());
            
            // Extract context and output limits
            let context_length = model.limits.as_ref().and_then(|l| l.max_input_tokens);
            let max_output_tokens = model.limits.as_ref().and_then(|l| l.max_output_tokens);
            
            // Infer code support from tags or model name
            let tags = model.tags.unwrap_or_default();
            let model_name_lower = model.name.to_lowercase();
            let supports_code = tags.contains(&"coding".to_string()) ||
                               model_name_lower.contains("code") ||
                               model_name_lower.contains("coder");
            
            // Determine model type based on output modalities and capabilities
            let output_modalities = model.supported_output_modalities.unwrap_or_default();
            let model_type = if output_modalities.contains(&"embeddings".to_string()) {
                ModelType::Embedding
            } else if output_modalities.contains(&"text".to_string()) {
                ModelType::Chat
            } else {
                ModelType::Other("unknown".to_string())
            };
            
            // Check if model is deprecated (GitHub doesn't seem to have this info)
            let is_deprecated = false;
            
            // Check if model supports JSON mode (infer from capabilities or tags)
            let supports_json_mode = tags.contains(&"structured".to_string()) ||
                                    capabilities.contains(&"structured-output".to_string());
            
            let metadata = ModelMetadata {
                id: model.id.clone(),
                provider: "github".to_string(),
                display_name: Some(model.name),
                description: model.summary,
                owned_by: model.publisher,
                created: None, // GitHub doesn't provide creation timestamp
                context_length,
                max_input_tokens: context_length,
                max_output_tokens,
                input_price_per_m: None, // GitHub doesn't provide pricing info
                output_price_per_m: None,
                supports_tools,
                supports_vision,
                supports_audio,
                supports_reasoning,
                supports_code,
                supports_function_calling,
                supports_json_mode,
                supports_streaming,
                model_type,
                is_deprecated,
                is_fine_tunable: false, // GitHub doesn't provide this info
                raw_data,
                ..Default::default()
            };
            
            models.push(metadata);
        }
        
        Ok(models)
    }
    
    fn extract_together(raw_json: &str) -> Result<Vec<ModelMetadata>, Box<dyn std::error::Error>> {
        #[derive(Deserialize, Serialize)]
        struct TogetherModel {
            id: String,
            display_name: Option<String>,
            organization: Option<String>,
            context_length: Option<u32>,
            created: Option<i64>,
            license: Option<String>,
            link: Option<String>,
            object: Option<String>,
            pricing: Option<TogetherPricing>,
            running: Option<bool>,
            #[serde(rename = "type")]
            model_type: Option<String>,
        }
        
        #[derive(Deserialize, Serialize)]
        struct TogetherPricing {
            base: Option<f64>,
            finetune: Option<f64>,
            hourly: Option<f64>,
            input: Option<f64>,
            output: Option<f64>,
        }
        
        // Together returns a direct array of models
        let together_models: Vec<TogetherModel> = serde_json::from_str(raw_json)?;
        let mut models = Vec::new();
        
        for model in together_models {
            let raw_data = serde_json::to_value(&model)?;
            
            // Extract pricing (convert from per-token to per-million-token)
            let input_price_per_m = model.pricing
                .as_ref()
                .and_then(|p| p.input)
                .map(|price| price * 1_000_000.0);
                
            let output_price_per_m = model.pricing
                .as_ref()
                .and_then(|p| p.output)
                .map(|price| price * 1_000_000.0);
            
            // Determine model type based on Together's type field
            let model_type = match model.model_type.as_deref() {
                Some("chat") => ModelType::Chat,
                Some("completion") => ModelType::Completion,
                Some("embedding") => ModelType::Embedding,
                Some("image") => ModelType::ImageGeneration,
                Some("audio") => ModelType::AudioGeneration,
                Some("transcribe") => ModelType::Other("transcription".to_string()),
                Some("moderation") => ModelType::Moderation,
                Some("rerank") => ModelType::Other("rerank".to_string()),
                Some("language") => ModelType::Completion,
                Some(other) => ModelType::Other(other.to_string()),
                None => ModelType::Chat, // Default to chat if not specified
            };
            
            // Only use explicit capability fields from JSON - no inference from names or types
            let supports_vision = false; // Together doesn't provide explicit vision capability flags
            let supports_code = false; // Together doesn't provide explicit code capability flags
            let supports_reasoning = false; // Together doesn't provide explicit reasoning capability flags
            let supports_audio = false; // Together doesn't provide explicit audio capability flags
            let supports_tools = false; // Together doesn't provide explicit tools capability flags
            let supports_function_calling = false; // Together doesn't provide explicit function calling capability flags
            let supports_streaming = false; // Together doesn't provide explicit streaming capability flags
            let supports_json_mode = false; // Together doesn't provide explicit JSON mode capability flags
            
            // Check if model is deprecated (not running)
            let is_deprecated = model.running == Some(false);
            
            let metadata = ModelMetadata {
                id: model.id.clone(),
                provider: "together".to_string(),
                display_name: model.display_name.or_else(|| Some(model.id.clone())),
                description: None, // Together doesn't provide descriptions
                owned_by: model.organization,
                created: model.created,
                context_length: model.context_length,
                max_input_tokens: model.context_length,
                max_output_tokens: None, // Together doesn't specify output limits
                input_price_per_m,
                output_price_per_m,
                supports_tools,
                supports_vision,
                supports_audio,
                supports_reasoning,
                supports_code,
                supports_function_calling,
                supports_json_mode,
                supports_streaming,
                model_type,
                is_deprecated,
                is_fine_tunable: model.pricing.as_ref().map_or(false, |p| p.finetune.unwrap_or(0.0) > 0.0),
                raw_data,
                ..Default::default()
            };
            
            models.push(metadata);
        }
        
        Ok(models)
    }
    
    fn extract_gemini(raw_json: &str) -> Result<Vec<ModelMetadata>, Box<dyn std::error::Error>> {
        #[derive(Deserialize)]
        struct GeminiResponse {
            models: Vec<GeminiModel>,
        }
        
        #[derive(Deserialize, Serialize)]
        struct GeminiModel {
            name: String,
            #[serde(rename = "displayName")]
            display_name: String,
            description: Option<String>,
            #[serde(rename = "inputTokenLimit")]
            input_token_limit: Option<u32>,
            #[serde(rename = "outputTokenLimit")]
            output_token_limit: Option<u32>,
            #[serde(rename = "supportedGenerationMethods")]
            supported_generation_methods: Vec<String>,
            temperature: Option<f32>,
            #[serde(rename = "topP")]
            top_p: Option<f32>,
            #[serde(rename = "topK")]
            top_k: Option<u32>,
            #[serde(rename = "maxTemperature")]
            max_temperature: Option<f32>,
        }
        
        let response: GeminiResponse = serde_json::from_str(raw_json)?;
        let mut models = Vec::new();
        
        for model in response.models {
            let raw_data = serde_json::to_value(&model)?;
            
            // Extract model ID from name (e.g., "models/gemini-1.5-pro-latest" -> "gemini-1.5-pro-latest")
            let id = model.name.strip_prefix("models/").unwrap_or(&model.name).to_string();
            
// Only use explicit capabilities from JSON - no inference from model names
let supports_tools = false; // Gemini doesn't provide explicit tools capability flags
let supports_function_calling = false; // Gemini doesn't provide explicit function calling capability flags
let supports_vision = false; // Gemini doesn't provide explicit vision capability flags
let supports_code = false; // Gemini doesn't provide explicit code capability flags
let supports_reasoning = false; // Gemini doesn't provide explicit reasoning capability flags
let supports_json_mode = false; // Gemini doesn't provide explicit JSON mode capability flags
let supports_streaming = false; // Gemini doesn't provide explicit streaming capability flags
let _supports_audio = false; // Gemini doesn't provide explicit audio capability flags

// Check if model is deprecated based only on explicit fields - no name heuristics
let is_deprecated = false; // Gemini doesn't provide explicit deprecation info
            
            let metadata = ModelMetadata {
                id: id.clone(),
                provider: "gemini".to_string(),
                display_name: Some(model.display_name),
                description: model.description,
                owned_by: Some("Google".to_string()),
                created: None, // Gemini doesn't provide creation timestamp
                context_length: model.input_token_limit,
                max_input_tokens: model.input_token_limit,
                max_output_tokens: model.output_token_limit,
                input_price_per_m: None, // Gemini doesn't provide pricing in models API
                output_price_per_m: None,
                supports_tools,
                supports_vision,
                supports_audio: _supports_audio,
                supports_reasoning,
                supports_code,
                supports_function_calling,
                supports_json_mode,
                supports_streaming,
                model_type: ModelType::Chat, // All Gemini models are chat models
                is_deprecated,
                is_fine_tunable: false, // Gemini doesn't support fine-tuning via API
                raw_data,
                ..Default::default()
            };
            
            models.push(metadata);
        }
        
        Ok(models)
    }
    
    
    fn infer_cohere_model_type(endpoints: &[String]) -> ModelType {
        if endpoints.contains(&"chat".to_string()) {
            ModelType::Chat
        } else if endpoints.contains(&"embed".to_string()) {
            ModelType::Embedding
        } else if endpoints.contains(&"rerank".to_string()) {
            ModelType::Other("rerank".to_string())
        } else {
            ModelType::Other("unknown".to_string())
        }
    }
}