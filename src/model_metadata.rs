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
    
    // Capabilities
    pub supports_tools: bool,
    pub supports_vision: bool,
    pub supports_audio: bool,
    pub supports_reasoning: bool,
    pub supports_code: bool,
    pub supports_function_calling: bool,
    pub supports_json_mode: bool,
    pub supports_streaming: bool,
    
    // Model type and characteristics
    pub model_type: ModelType,
    pub is_deprecated: bool,
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
            
            // Infer capabilities from model name
            Self::infer_openai_capabilities(&mut metadata);
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
            let mut metadata = ModelMetadata {
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
            
            Self::infer_groq_capabilities(&mut metadata);
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
            let mut metadata = ModelMetadata {
                id: model.id.clone(),
                provider: "claude".to_string(),
                display_name: Some(model.display_name),
                raw_data,
                ..Default::default()
            };
            
            Self::infer_claude_capabilities(&mut metadata);
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
            
            // Extract code support from model name/description
            let model_name_lower = model.name.to_lowercase();
            let supports_code = model_name_lower.contains("code") ||
                               model_name_lower.contains("coder") ||
                               model.description.as_ref().map_or(false, |desc| {
                                   let desc_lower = desc.to_lowercase();
                                   desc_lower.contains("code") || desc_lower.contains("programming")
                               });
            
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
    
    fn extract_generic(provider: &str, raw_json: &str) -> Result<Vec<ModelMetadata>, Box<dyn std::error::Error>> {
        // Try to parse as standard OpenAI format first
        if let Ok(models) = Self::extract_openai(raw_json) {
            return Ok(models.into_iter().map(|mut m| {
                m.provider = provider.to_string();
                m
            }).collect());
        }
        
        // If that fails, try to extract basic model list
        let value: serde_json::Value = serde_json::from_str(raw_json)?;
        let mut models = Vec::new();
        
        // First, try to get models from "data" field (wrapped format)
        if let Some(data) = value.get("data").and_then(|d| d.as_array()) {
            for model_value in data {
                if let Some(id) = model_value.get("id").and_then(|i| i.as_str()) {
                    let metadata = ModelMetadata {
                        id: id.to_string(),
                        provider: provider.to_string(),
                        raw_data: model_value.clone(),
                        ..Default::default()
                    };
                    models.push(metadata);
                }
            }
        }
        // If no "data" field, try to parse as direct array (providers like together, github)
        else if let Some(array) = value.as_array() {
            for model_value in array {
                if let Some(id) = model_value.get("id").and_then(|i| i.as_str()) {
                    let metadata = ModelMetadata {
                        id: id.to_string(),
                        provider: provider.to_string(),
                        raw_data: model_value.clone(),
                        ..Default::default()
                    };
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
            
            // Infer capabilities from model name and type
            let model_name_lower = model.display_name.as_ref()
                .unwrap_or(&model.id)
                .to_lowercase();
            
            let supports_vision = model_name_lower.contains("vision") ||
                                 model_name_lower.contains("vl") ||
                                 model_name_lower.contains("multimodal");
            
            let supports_code = model_name_lower.contains("code") ||
                               model_name_lower.contains("coder") ||
                               model_name_lower.contains("starcoder") ||
                               model_name_lower.contains("codestral");
            
            let supports_reasoning = model_name_lower.contains("reasoning") ||
                                    model_name_lower.contains("qwq") ||
                                    model_name_lower.contains("r1");
            
            let supports_audio = matches!(model_type, ModelType::AudioGeneration) ||
                                model_name_lower.contains("whisper") ||
                                model_name_lower.contains("sonic");
            
            // Most Together chat models support tools and streaming
            let supports_tools = matches!(model_type, ModelType::Chat) &&
                                 !model_name_lower.contains("guard") &&
                                 !model_name_lower.contains("embed");
            
            let supports_function_calling = supports_tools;
            let supports_streaming = matches!(model_type, ModelType::Chat);
            let supports_json_mode = supports_tools; // Most tool-capable models support JSON mode
            
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
            
            // Determine capabilities based on supported generation methods and model name
            let supports_tools = model.supported_generation_methods.contains(&"generateContent".to_string());
            let supports_function_calling = supports_tools; // Same as tools for Gemini
            
            // Infer vision support from model name (Gemini Pro Vision, etc.)
            let model_name_lower = model.display_name.to_lowercase();
            let supports_vision = model_name_lower.contains("vision") ||
                                 model_name_lower.contains("pro") || // Most Gemini Pro models support vision
                                 model_name_lower.contains("1.5"); // Gemini 1.5 models support vision
            
            // Infer code support from model name
            let supports_code = model_name_lower.contains("code") ||
                               model_name_lower.contains("pro") || // Pro models generally good at code
                               model_name_lower.contains("1.5"); // 1.5 models are good at code
            
            // Infer reasoning support from model name
            let supports_reasoning = model_name_lower.contains("pro") ||
                                    model_name_lower.contains("1.5") ||
                                    model_name_lower.contains("ultra");
            
            // Most Gemini models support JSON mode and streaming
            let supports_json_mode = supports_tools; // Tool-capable models typically support JSON
            let supports_streaming = true; // Gemini supports streaming
            
            // Determine if model is deprecated (basic heuristic)
            let is_deprecated = model_name_lower.contains("deprecated") ||
                               model_name_lower.contains("legacy");
            
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
                supports_audio: false, // Gemini doesn't support audio in text models
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
    
    // Capability inference helpers
    fn infer_openai_capabilities(metadata: &mut ModelMetadata) {
        let id = &metadata.id;
        
        // Vision models
        if id.contains("vision") || id.contains("gpt-4o") || id.contains("gpt-4-turbo") {
            metadata.supports_vision = true;
        }
        
        // Audio models
        if id.contains("audio") || id.contains("tts") || id.contains("whisper") {
            metadata.supports_audio = true;
            metadata.model_type = ModelType::AudioGeneration;
        }
        
        // Reasoning models
        if id.contains("o1") || id.contains("o3") || id.contains("reasoning") {
            metadata.supports_reasoning = true;
        }
        
        // Code models
        if id.contains("code") || id.contains("davinci-002") || id.contains("babbage-002") {
            metadata.supports_code = true;
        }
        
        // Image generation
        if id.contains("dall-e") {
            metadata.model_type = ModelType::ImageGeneration;
        }
        
        // Embeddings
        if id.contains("embedding") {
            metadata.model_type = ModelType::Embedding;
        }
        
        // Moderation
        if id.contains("moderation") {
            metadata.model_type = ModelType::Moderation;
        }
        
        // Most OpenAI models support tools and function calling
        if matches!(metadata.model_type, ModelType::Chat) {
            metadata.supports_tools = true;
            metadata.supports_function_calling = true;
            metadata.supports_json_mode = true;
            metadata.supports_streaming = true;
        }
    }
    
    fn infer_groq_capabilities(metadata: &mut ModelMetadata) {
        let id = &metadata.id;
        
        // Audio models
        if id.contains("whisper") {
            metadata.supports_audio = true;
            metadata.model_type = ModelType::AudioGeneration;
        }
        
        // Code models
        if id.contains("code") || id.contains("starcoder") {
            metadata.supports_code = true;
        }
        
        // Most Groq models support tools
        if matches!(metadata.model_type, ModelType::Chat) {
            metadata.supports_tools = true;
            metadata.supports_function_calling = true;
            metadata.supports_streaming = true;
        }
    }
    
    fn infer_claude_capabilities(metadata: &mut ModelMetadata) {
        // All Claude models support tools, streaming, and JSON mode
        metadata.supports_tools = true;
        metadata.supports_function_calling = true;
        metadata.supports_json_mode = true;
        metadata.supports_streaming = true;
        
        // Claude 3+ models support vision
        if metadata.id.contains("claude-3") || metadata.id.contains("claude-4") {
            metadata.supports_vision = true;
        }
        
        // Set context lengths based on model
        if metadata.id.contains("haiku") {
            metadata.context_length = Some(200000);
        } else if metadata.id.contains("sonnet") {
            metadata.context_length = Some(200000);
        } else if metadata.id.contains("opus") {
            metadata.context_length = Some(200000);
        }
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