use std::fs;
use lc::model_metadata::MetadataExtractor;
use serde_json::Value;

/// Helper function to get the capability value for a specific model from the JSON
fn get_model_capability_from_json(json_data: &str, model_id: &str, field: &str) -> Option<bool> {
    let parsed: Value = serde_json::from_str(json_data).ok()?;
    
    // Handle different JSON structures
    let models = if let Some(models) = parsed.get("models").and_then(|m| m.as_array()) {
        models
    } else if let Some(data) = parsed.get("data").and_then(|d| d.as_array()) {
        data
    } else if let Some(models_array) = parsed.as_array() {
        // Handle direct array format (like GitHub)
        models_array
    } else {
        return None;
    };
    
    for model in models {
        let name = model.get("id").and_then(|n| n.as_str())
            .or_else(|| model.get("name").and_then(|n| n.as_str()));
        
        if let Some(name) = name {
            if name == model_id {
                // Check direct field
                if let Some(value) = model.get(field).and_then(|v| v.as_bool()) {
                    return Some(value);
                }
                
                // Check capabilities object for Mistral
                if let Some(capabilities) = model.get("capabilities").and_then(|c| c.as_object()) {
                    match field {
                        "supports_function_calling" => {
                            return capabilities.get("function_calling").and_then(|v| v.as_bool());
                        }
                        "supports_vision" => {
                            return capabilities.get("vision").and_then(|v| v.as_bool());
                        }
                        "supports_code" => {
                            return capabilities.get("completion_fim").and_then(|v| v.as_bool());
                        }
                        _ => {}
                    }
                }
                
                // Check Fireworks field mappings
                if field == "supports_vision" {
                    if let Some(value) = model.get("supports_image_input").and_then(|v| v.as_bool()) {
                        return Some(value);
                    }
                }
                
                // Check OpenRouter/Kilo supported_parameters array
                if let Some(supported_params) = model.get("supported_parameters").and_then(|p| p.as_array()) {
                    let params: Vec<String> = supported_params.iter()
                        .filter_map(|p| p.as_str().map(|s| s.to_string()))
                        .collect();
                    
                    match field {
                        "supports_tools" => {
                            if params.contains(&"tools".to_string()) {
                                return Some(true);
                            }
                        }
                        "supports_function_calling" => {
                            if params.contains(&"tools".to_string()) {
                                return Some(true);
                            }
                        }
                        "supports_reasoning" => {
                            if params.contains(&"reasoning".to_string()) {
                                return Some(true);
                            }
                        }
                        "supports_json_mode" => {
                            if params.contains(&"response_format".to_string()) {
                                return Some(true);
                            }
                        }
                        _ => {}
                    }
                }
                
                // Check OpenRouter architecture for vision support
                if field == "supports_vision" {
                    if let Some(architecture) = model.get("architecture").and_then(|a| a.as_object()) {
                        if let Some(input_modalities) = architecture.get("input_modalities").and_then(|m| m.as_array()) {
                            let modalities: Vec<String> = input_modalities.iter()
                                .filter_map(|m| m.as_str().map(|s| s.to_string()))
                                .collect();
                            if modalities.contains(&"image".to_string()) {
                                return Some(true);
                            }
                        }
                    }
                }
                
                // Check GitHub capabilities array
                if let Some(capabilities) = model.get("capabilities").and_then(|c| c.as_array()) {
                    let caps: Vec<String> = capabilities.iter()
                        .filter_map(|c| c.as_str().map(|s| s.to_string()))
                        .collect();
                    
                    match field {
                        "supports_tools" => {
                            if caps.contains(&"tool-calling".to_string()) {
                                return Some(true);
                            }
                        }
                        "supports_function_calling" => {
                            if caps.contains(&"tool-calling".to_string()) {
                                return Some(true);
                            }
                        }
                        "supports_reasoning" => {
                            if caps.contains(&"reasoning".to_string()) {
                                return Some(true);
                            }
                        }
                        "supports_streaming" => {
                            if caps.contains(&"streaming".to_string()) {
                                return Some(true);
                            }
                        }
                        _ => {}
                    }
                }
                
                // Check GitHub supported_input_modalities for vision and audio
                if field == "supports_vision" {
                    if let Some(input_modalities) = model.get("supported_input_modalities").and_then(|m| m.as_array()) {
                        let modalities: Vec<String> = input_modalities.iter()
                            .filter_map(|m| m.as_str().map(|s| s.to_string()))
                            .collect();
                        if modalities.contains(&"image".to_string()) {
                            return Some(true);
                        }
                    }
                }
                
                if field == "supports_audio" {
                    if let Some(input_modalities) = model.get("supported_input_modalities").and_then(|m| m.as_array()) {
                        let modalities: Vec<String> = input_modalities.iter()
                            .filter_map(|m| m.as_str().map(|s| s.to_string()))
                            .collect();
                        if modalities.contains(&"audio".to_string()) {
                            return Some(true);
                        }
                    }
                }
                
                // Check GitHub tags for code support
                if field == "supports_code" {
                    if let Some(tags) = model.get("tags").and_then(|t| t.as_array()) {
                        let tag_list: Vec<String> = tags.iter()
                            .filter_map(|t| t.as_str().map(|s| s.to_string()))
                            .collect();
                        if tag_list.contains(&"coding".to_string()) {
                            return Some(true);
                        }
                    }
                }
                
                return Some(false);
            }
        }
    }
    None
}

/// Get the features array for a specific model from the JSON
fn get_model_features_from_json(json_data: &str, model_id: &str) -> Vec<String> {
    let parsed: Value = serde_json::from_str(json_data).ok().unwrap_or(Value::Null);
    let empty_vec = vec![];
    let models = parsed.get("models").and_then(|m| m.as_array()).unwrap_or(&empty_vec);
    
    for model in models {
        if let Some(name) = model.get("name").and_then(|n| n.as_str()) {
            if name == model_id {
                if let Some(features) = model.get("features").and_then(|f| f.as_array()) {
                    return features.iter()
                        .filter_map(|f| f.as_str().map(|s| s.to_string()))
                        .collect();
                }
                return vec![];
            }
        }
    }
    vec![]
}


/// Test that OpenAI models don't have capability flags set unless explicitly present in JSON
#[test]
fn test_openai_models_no_inferred_capabilities() {
    let json_data = fs::read_to_string("models/openai.json")
        .expect("Failed to read OpenAI models fixture");
    
    let models = MetadataExtractor::extract_from_provider("openai", &json_data)
        .expect("Failed to extract OpenAI models");
    
    for model in models {
        // Check that capability flags match exactly what's in the JSON for this model
        let json_supports_tools = get_model_capability_from_json(&json_data, &model.id, "supports_tools").unwrap_or(false);
        assert_eq!(model.supports_tools, json_supports_tools,
                   "Model {} has supports_tools={} but JSON has {}", 
                   model.id, model.supports_tools, json_supports_tools);
        
        let json_supports_vision = get_model_capability_from_json(&json_data, &model.id, "supports_vision").unwrap_or(false);
        assert_eq!(model.supports_vision, json_supports_vision,
                   "Model {} has supports_vision={} but JSON has {}", 
                   model.id, model.supports_vision, json_supports_vision);
        
        let json_supports_audio = get_model_capability_from_json(&json_data, &model.id, "supports_audio").unwrap_or(false);
        assert_eq!(model.supports_audio, json_supports_audio,
                   "Model {} has supports_audio={} but JSON has {}", 
                   model.id, model.supports_audio, json_supports_audio);
        
        let json_supports_reasoning = get_model_capability_from_json(&json_data, &model.id, "supports_reasoning").unwrap_or(false);
        assert_eq!(model.supports_reasoning, json_supports_reasoning,
                   "Model {} has supports_reasoning={} but JSON has {}", 
                   model.id, model.supports_reasoning, json_supports_reasoning);
        
        let json_supports_code = get_model_capability_from_json(&json_data, &model.id, "supports_code").unwrap_or(false);
        assert_eq!(model.supports_code, json_supports_code,
                   "Model {} has supports_code={} but JSON has {}", 
                   model.id, model.supports_code, json_supports_code);
        
        let json_supports_function_calling = get_model_capability_from_json(&json_data, &model.id, "supports_function_calling").unwrap_or(false);
        assert_eq!(model.supports_function_calling, json_supports_function_calling,
                   "Model {} has supports_function_calling={} but JSON has {}", 
                   model.id, model.supports_function_calling, json_supports_function_calling);
    }
}

/// Test that Claude models don't have capability flags set unless explicitly present in JSON
#[test]
fn test_claude_models_no_inferred_capabilities() {
    let json_data = fs::read_to_string("models/claude.json")
        .expect("Failed to read Claude models fixture");
    
    let models = MetadataExtractor::extract_from_provider("claude", &json_data)
        .expect("Failed to extract Claude models");
    
    for model in models {
        // Check that capability flags match exactly what's in the JSON for this model
        let json_supports_tools = get_model_capability_from_json(&json_data, &model.id, "supports_tools").unwrap_or(false);
        assert_eq!(model.supports_tools, json_supports_tools,
                   "Model {} has supports_tools={} but JSON has {}", 
                   model.id, model.supports_tools, json_supports_tools);
        
        let json_supports_vision = get_model_capability_from_json(&json_data, &model.id, "supports_vision").unwrap_or(false);
        assert_eq!(model.supports_vision, json_supports_vision,
                   "Model {} has supports_vision={} but JSON has {}", 
                   model.id, model.supports_vision, json_supports_vision);
        
        let json_supports_audio = get_model_capability_from_json(&json_data, &model.id, "supports_audio").unwrap_or(false);
        assert_eq!(model.supports_audio, json_supports_audio,
                   "Model {} has supports_audio={} but JSON has {}", 
                   model.id, model.supports_audio, json_supports_audio);
        
        let json_supports_reasoning = get_model_capability_from_json(&json_data, &model.id, "supports_reasoning").unwrap_or(false);
        assert_eq!(model.supports_reasoning, json_supports_reasoning,
                   "Model {} has supports_reasoning={} but JSON has {}", 
                   model.id, model.supports_reasoning, json_supports_reasoning);
        
        let json_supports_code = get_model_capability_from_json(&json_data, &model.id, "supports_code").unwrap_or(false);
        assert_eq!(model.supports_code, json_supports_code,
                   "Model {} has supports_code={} but JSON has {}", 
                   model.id, model.supports_code, json_supports_code);
    }
}

/// Test that models across various providers don't have capability flags set unless explicitly present
fn test_provider_models_no_inferred_capabilities(provider: &str) {
    let json_data = fs::read_to_string(format!("models/{}.json", provider))
        .expect(&format!("Failed to read {} models fixture", provider));
    
    let models = MetadataExtractor::extract_from_provider(provider, &json_data)
        .expect(&format!("Failed to extract {} models", provider));
    
    for model in models {
        // Check that capability flags match exactly what's extracted from the JSON using the common logic
        let json_supports_tools = get_model_capability_from_json(&json_data, &model.id, "supports_tools").unwrap_or(false);
        assert_eq!(model.supports_tools, json_supports_tools,
                   "Model {} has supports_tools={} but JSON has {}", 
                   model.id, model.supports_tools, json_supports_tools);
        
        let json_supports_vision = get_model_capability_from_json(&json_data, &model.id, "supports_vision").unwrap_or(false);
        assert_eq!(model.supports_vision, json_supports_vision,
                   "Model {} has supports_vision={} but JSON has {}", 
                   model.id, model.supports_vision, json_supports_vision);
        
        let json_supports_audio = get_model_capability_from_json(&json_data, &model.id, "supports_audio").unwrap_or(false);
        assert_eq!(model.supports_audio, json_supports_audio,
                   "Model {} has supports_audio={} but JSON has {}", 
                   model.id, model.supports_audio, json_supports_audio);
        
        let json_supports_reasoning = get_model_capability_from_json(&json_data, &model.id, "supports_reasoning").unwrap_or(false);
        assert_eq!(model.supports_reasoning, json_supports_reasoning,
                   "Model {} has supports_reasoning={} but JSON has {}", 
                   model.id, model.supports_reasoning, json_supports_reasoning);
        
        let json_supports_code = get_model_capability_from_json(&json_data, &model.id, "supports_code").unwrap_or(false);
        assert_eq!(model.supports_code, json_supports_code,
                   "Model {} has supports_code={} but JSON has {}", 
                   model.id, model.supports_code, json_supports_code);
    }
}


#[test]
fn test_all_providers_no_inferred_capabilities() {
    let providers = vec!["ai21", "cerebras", "chutes", "cohere", "deepinfra", "digitalocean", "fireworks", "github", "github-copilot", "grok", "groq", "hyperbolic", "kilo", "litellm", "meta", "mistral", "nebius", "novita", "nscale", "nvidia", "ollama", "openrouter", "requesty", "sambanova", "together", "venice", "vercel"];
    
    for provider in providers {
        println!("Testing provider: {}", provider);
        test_provider_models_no_inferred_capabilities(provider);
    }
}
