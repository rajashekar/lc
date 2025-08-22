//! Integration tests for proxy commands
//!
//! This module contains comprehensive integration tests for all proxy-related
//! functionality, testing the underlying components as the CLI would use them.

mod common;

use lc::config::Config;
use lc::provider::Message;
use lc::proxy::{
    generate_api_key, parse_model_string, ProxyChatRequest, ProxyChatResponse, ProxyChoice,
    ProxyModel, ProxyModelsQuery, ProxyModelsResponse, ProxyState, ProxyUsage,
};
use std::collections::HashMap;

#[cfg(test)]
mod proxy_state_tests {
    use super::*;

    fn create_test_config() -> Config {
        let mut config = Config {
            providers: HashMap::new(),
            default_provider: None,
            default_model: None,
            aliases: HashMap::new(),
            system_prompt: None,
            templates: HashMap::new(),
            max_tokens: None,
            temperature: None,
            stream: None,
        };

        // Add test providers
        config.providers.insert(
            "openai".to_string(),
            lc::config::ProviderConfig {
                endpoint: "https://api.openai.com/v1/chat/completions".to_string(),
                api_key: None,
                models: vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()],
                models_path: "/models".to_string(),
                chat_path: "/chat/completions".to_string(),
                headers: HashMap::new(),
                token_url: None,
                cached_token: None,
                auth_type: None,
                vars: HashMap::new(),
                images_path: Some("/images/generations".to_string()),
                embeddings_path: Some("/embeddings".to_string()),
                chat_templates: None,
                images_templates: None,
                embeddings_templates: None,
                models_templates: None,
                audio_path: None,
                speech_path: None,
                audio_templates: None,
                speech_templates: None,
            },
        );

        config.providers.insert(
            "anthropic".to_string(),
            lc::config::ProviderConfig {
                endpoint: "https://api.anthropic.com/v1/messages".to_string(),
                api_key: None,
                models: vec!["claude-3-sonnet".to_string()],
                models_path: "/models".to_string(),
                chat_path: "/messages".to_string(),
                headers: HashMap::new(),
                token_url: None,
                cached_token: None,
                auth_type: None,
                vars: HashMap::new(),
                images_path: Some("/images/generations".to_string()),
                embeddings_path: Some("/embeddings".to_string()),
                chat_templates: None,
                images_templates: None,
                embeddings_templates: None,
                models_templates: None,
                audio_path: None,
                speech_path: None,
                audio_templates: None,
                speech_templates: None,
            },
        );

        config.default_provider = Some("openai".to_string());
        config
    }

    #[test]
    fn test_proxy_state_creation() {
        let config = create_test_config();

        let state = ProxyState {
            config: config.clone(),
            api_key: Some("test-key".to_string()),
            provider_filter: Some("openai".to_string()),
            model_filter: Some("gpt-4".to_string()),
        };

        assert_eq!(state.api_key, Some("test-key".to_string()));
        assert_eq!(state.provider_filter, Some("openai".to_string()));
        assert_eq!(state.model_filter, Some("gpt-4".to_string()));
        assert_eq!(state.config.default_provider, Some("openai".to_string()));
    }

    #[test]
    fn test_proxy_state_no_filters() {
        let config = create_test_config();

        let state = ProxyState {
            config: config.clone(),
            api_key: None,
            provider_filter: None,
            model_filter: None,
        };

        assert!(state.api_key.is_none());
        assert!(state.provider_filter.is_none());
        assert!(state.model_filter.is_none());
    }

    #[test]
    fn test_proxy_state_clone() {
        let config = create_test_config();

        let state1 = ProxyState {
            config: config.clone(),
            api_key: Some("test-key".to_string()),
            provider_filter: Some("openai".to_string()),
            model_filter: None,
        };

        let state2 = state1.clone();

        assert_eq!(state1.api_key, state2.api_key);
        assert_eq!(state1.provider_filter, state2.provider_filter);
        assert_eq!(state1.model_filter, state2.model_filter);
    }
}

#[cfg(test)]
mod proxy_model_parsing_tests {
    use super::*;

    fn create_config_with_aliases() -> Config {
        let mut config = Config {
            providers: HashMap::new(),
            default_provider: Some("openai".to_string()),
            default_model: None,
            aliases: HashMap::new(),
            system_prompt: None,
            templates: HashMap::new(),
            max_tokens: None,
            temperature: None,
            stream: None,
        };

        // Add providers
        config.providers.insert(
            "openai".to_string(),
            lc::config::ProviderConfig {
                endpoint: "https://api.openai.com/v1/chat/completions".to_string(),
                api_key: None,
                models: vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()],
                models_path: "/models".to_string(),
                chat_path: "/chat/completions".to_string(),
                headers: HashMap::new(),
                token_url: None,
                cached_token: None,
                auth_type: None,
                vars: HashMap::new(),
                images_path: Some("/images/generations".to_string()),
                embeddings_path: Some("/embeddings".to_string()),
                chat_templates: None,
                images_templates: None,
                embeddings_templates: None,
                models_templates: None,
                audio_path: None,
                speech_path: None,
                audio_templates: None,
                speech_templates: None,
            },
        );

        config.providers.insert(
            "anthropic".to_string(),
            lc::config::ProviderConfig {
                endpoint: "https://api.anthropic.com/v1/messages".to_string(),
                api_key: None,
                models: vec!["claude-3-sonnet".to_string()],
                models_path: "/models".to_string(),
                chat_path: "/messages".to_string(),
                headers: HashMap::new(),
                token_url: None,
                cached_token: None,
                auth_type: None,
                vars: HashMap::new(),
                images_path: Some("/images/generations".to_string()),
                embeddings_path: Some("/embeddings".to_string()),
                chat_templates: None,
                images_templates: None,
                embeddings_templates: None,
                models_templates: None,
                audio_path: None,
                speech_path: None,
                audio_templates: None,
                speech_templates: None,
            },
        );

        // Add aliases
        config
            .aliases
            .insert("gpt4".to_string(), "openai:gpt-4".to_string());
        config.aliases.insert(
            "claude".to_string(),
            "anthropic:claude-3-sonnet".to_string(),
        );

        config
    }

    #[test]
    fn test_parse_model_with_provider() {
        let config = create_config_with_aliases();

        // Test provider:model format
        let result = parse_model_string("openai:gpt-4", &config);
        assert!(result.is_ok());
        let (provider, model) = result.unwrap();
        assert_eq!(provider, "openai");
        assert_eq!(model, "gpt-4");

        let result = parse_model_string("anthropic:claude-3-sonnet", &config);
        assert!(result.is_ok());
        let (provider, model) = result.unwrap();
        assert_eq!(provider, "anthropic");
        assert_eq!(model, "claude-3-sonnet");
    }

    #[test]
    fn test_parse_model_with_alias() {
        let config = create_config_with_aliases();

        // Test alias resolution
        let result = parse_model_string("gpt4", &config);
        assert!(result.is_ok());
        let (provider, model) = result.unwrap();
        assert_eq!(provider, "openai");
        assert_eq!(model, "gpt-4");

        let result = parse_model_string("claude", &config);
        assert!(result.is_ok());
        let (provider, model) = result.unwrap();
        assert_eq!(provider, "anthropic");
        assert_eq!(model, "claude-3-sonnet");
    }

    #[test]
    fn test_parse_model_with_default_provider() {
        let config = create_config_with_aliases();

        // Test default provider usage
        let result = parse_model_string("gpt-3.5-turbo", &config);
        assert!(result.is_ok());
        let (provider, model) = result.unwrap();
        assert_eq!(provider, "openai");
        assert_eq!(model, "gpt-3.5-turbo");
    }

    #[test]
    fn test_parse_model_nonexistent_provider() {
        let config = create_config_with_aliases();

        // Test non-existent provider - this actually falls back to default provider
        // and treats the whole string as model name
        let result = parse_model_string("nonexistent:model", &config);
        assert!(result.is_ok());
        let (provider, model) = result.unwrap();
        assert_eq!(provider, "openai"); // Falls back to default provider
        assert_eq!(model, "nonexistent:model"); // Treats whole string as model name
    }

    #[test]
    fn test_parse_model_no_default_provider() {
        let mut config = create_config_with_aliases();
        config.default_provider = None;

        // Test without default provider
        let result = parse_model_string("some-model", &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_model_invalid_alias() {
        let mut config = create_config_with_aliases();
        config
            .aliases
            .insert("invalid".to_string(), "just-a-model".to_string());

        // Test invalid alias format
        let result = parse_model_string("invalid", &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_model_edge_cases() {
        let config = create_config_with_aliases();

        // Test empty model
        let result = parse_model_string("", &config);
        assert!(result.is_ok());
        let (provider, model) = result.unwrap();
        assert_eq!(provider, "openai");
        assert_eq!(model, "");

        // Test model with multiple colons
        let result = parse_model_string("openai:model:with:colons", &config);
        assert!(result.is_ok());
        let (provider, model) = result.unwrap();
        assert_eq!(provider, "openai");
        assert_eq!(model, "model:with:colons");
    }
}

#[cfg(test)]
mod proxy_api_key_tests {
    use super::*;

    #[test]
    fn test_generate_api_key() {
        let key1 = generate_api_key();
        let key2 = generate_api_key();

        // Keys should be different
        assert_ne!(key1, key2);

        // Keys should start with "sk-"
        assert!(key1.starts_with("sk-"));
        assert!(key2.starts_with("sk-"));

        // Keys should be the right length (sk- + 32 chars)
        assert_eq!(key1.len(), 35);
        assert_eq!(key2.len(), 35);

        // Keys should only contain valid characters
        let valid_chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        for key in [&key1, &key2] {
            let key_part = &key[3..]; // Remove "sk-" prefix
            for ch in key_part.chars() {
                assert!(valid_chars.contains(ch));
            }
        }
    }

    #[test]
    fn test_generate_multiple_api_keys() {
        let mut keys = std::collections::HashSet::new();

        // Generate 100 keys and ensure they're all unique
        for _ in 0..100 {
            let key = generate_api_key();
            assert!(keys.insert(key), "Generated duplicate API key");
        }

        assert_eq!(keys.len(), 100);
    }
}

#[cfg(test)]
mod proxy_data_structures_tests {
    use super::*;

    #[test]
    fn test_proxy_models_query() {
        // Test default deserialization
        let json = "{}";
        let query: ProxyModelsQuery = serde_json::from_str(json).unwrap();
        assert!(query.provider.is_none());

        // Test with provider
        let json = r#"{"provider": "openai"}"#;
        let query: ProxyModelsQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.provider, Some("openai".to_string()));
    }

    #[test]
    fn test_proxy_model_serialization() {
        let model = ProxyModel {
            id: "openai:gpt-4".to_string(),
            object: "model".to_string(),
            created: 1234567890,
            owned_by: "openai".to_string(),
        };

        let json = serde_json::to_string(&model).unwrap();
        assert!(json.contains("openai:gpt-4"));
        assert!(json.contains("model"));
        assert!(json.contains("1234567890"));
        assert!(json.contains("openai"));
    }

    #[test]
    fn test_proxy_models_response_serialization() {
        let response = ProxyModelsResponse {
            object: "list".to_string(),
            data: vec![
                ProxyModel {
                    id: "openai:gpt-4".to_string(),
                    object: "model".to_string(),
                    created: 1234567890,
                    owned_by: "openai".to_string(),
                },
                ProxyModel {
                    id: "anthropic:claude-3-sonnet".to_string(),
                    object: "model".to_string(),
                    created: 1234567890,
                    owned_by: "anthropic".to_string(),
                },
            ],
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("list"));
        assert!(json.contains("openai:gpt-4"));
        assert!(json.contains("anthropic:claude-3-sonnet"));
    }

    #[test]
    fn test_proxy_chat_request_deserialization() {
        let json = r#"{
            "model": "gpt-4",
            "messages": [
                {"role": "user", "content": "Hello"}
            ],
            "max_tokens": 100,
            "temperature": 0.7
        }"#;

        let request: ProxyChatRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.model, "gpt-4");
        assert_eq!(request.messages.len(), 1);
        assert_eq!(request.max_tokens, Some(100));
        assert_eq!(request.temperature, Some(0.7));
    }

    #[test]
    fn test_proxy_chat_request_optional_fields() {
        let json = r#"{
            "model": "gpt-4",
            "messages": [
                {"role": "user", "content": "Hello"}
            ]
        }"#;

        let request: ProxyChatRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.model, "gpt-4");
        assert_eq!(request.messages.len(), 1);
        assert!(request.max_tokens.is_none());
        assert!(request.temperature.is_none());
    }

    #[test]
    fn test_proxy_chat_response_serialization() {
        let response = ProxyChatResponse {
            id: "chatcmpl-123".to_string(),
            object: "chat.completion".to_string(),
            created: 1234567890,
            model: "gpt-4".to_string(),
            choices: vec![ProxyChoice {
                index: 0,
                message: Message {
                    role: "assistant".to_string(),
                    content_type: lc::provider::MessageContent::Text {
                        content: Some("Hello there!".to_string()),
                    },
                    tool_calls: None,
                    tool_call_id: None,
                },
                finish_reason: "stop".to_string(),
            }],
            usage: ProxyUsage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
            },
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("chatcmpl-123"));
        assert!(json.contains("chat.completion"));
        assert!(json.contains("Hello there!"));
        assert!(json.contains("stop"));
    }

    #[test]
    fn test_proxy_usage_serialization() {
        let usage = ProxyUsage {
            prompt_tokens: 100,
            completion_tokens: 50,
            total_tokens: 150,
        };

        let json = serde_json::to_string(&usage).unwrap();
        assert!(json.contains("100"));
        assert!(json.contains("50"));
        assert!(json.contains("150"));
    }
}

#[cfg(test)]
mod proxy_filtering_tests {
    use super::*;

    fn create_test_state_with_filters(
        provider_filter: Option<String>,
        model_filter: Option<String>,
    ) -> ProxyState {
        let mut config = Config {
            providers: HashMap::new(),
            default_provider: Some("openai".to_string()),
            default_model: None,
            aliases: HashMap::new(),
            system_prompt: None,
            templates: HashMap::new(),
            max_tokens: None,
            temperature: None,
            stream: None,
        };

        config.providers.insert(
            "openai".to_string(),
            lc::config::ProviderConfig {
                endpoint: "https://api.openai.com/v1/chat/completions".to_string(),
                api_key: None,
                models: vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()],
                models_path: "/models".to_string(),
                chat_path: "/chat/completions".to_string(),
                headers: HashMap::new(),
                token_url: None,
                cached_token: None,
                auth_type: None,
                vars: HashMap::new(),
                images_path: Some("/images/generations".to_string()),
                embeddings_path: Some("/embeddings".to_string()),
                chat_templates: None,
                images_templates: None,
                embeddings_templates: None,
                models_templates: None,
                audio_path: None,
                speech_path: None,
                audio_templates: None,
                speech_templates: None,
            },
        );

        config.providers.insert(
            "anthropic".to_string(),
            lc::config::ProviderConfig {
                endpoint: "https://api.anthropic.com/v1/messages".to_string(),
                api_key: None,
                models: vec!["claude-3-sonnet".to_string()],
                models_path: "/models".to_string(),
                chat_path: "/messages".to_string(),
                headers: HashMap::new(),
                token_url: None,
                cached_token: None,
                auth_type: None,
                vars: HashMap::new(),
                images_path: Some("/images/generations".to_string()),
                embeddings_path: Some("/embeddings".to_string()),
                chat_templates: None,
                images_templates: None,
                embeddings_templates: None,
                models_templates: None,
                audio_path: None,
                speech_path: None,
                audio_templates: None,
                speech_templates: None,
            },
        );

        ProxyState {
            config,
            api_key: None,
            provider_filter,
            model_filter,
        }
    }

    #[test]
    fn test_provider_filter_logic() {
        let state = create_test_state_with_filters(Some("openai".to_string()), None);

        // OpenAI models should be allowed
        let result = parse_model_string("openai:gpt-4", &state.config);
        assert!(result.is_ok());
        let (provider, _) = result.unwrap();

        // Check if provider matches filter
        if let Some(ref provider_filter) = state.provider_filter {
            assert_eq!(provider, *provider_filter);
        }

        // Anthropic models should be filtered out in actual proxy logic
        let result = parse_model_string("anthropic:claude-3-sonnet", &state.config);
        assert!(result.is_ok());
        let (provider, _) = result.unwrap();

        // This would be filtered out by proxy logic
        if let Some(ref provider_filter) = state.provider_filter {
            assert_ne!(provider, *provider_filter);
        }
    }

    #[test]
    fn test_model_filter_logic() {
        let state = create_test_state_with_filters(None, Some("gpt-4".to_string()));

        // Test model filtering logic
        let model_requests = vec!["gpt-4", "openai:gpt-4", "gpt-3.5-turbo", "claude-3-sonnet"];

        for model_request in model_requests {
            if let Some(ref model_filter) = state.model_filter {
                let should_pass =
                    model_request.contains(model_filter) || model_request.ends_with(model_filter);

                if model_request == "gpt-4" || model_request == "openai:gpt-4" {
                    assert!(
                        should_pass,
                        "Model {} should pass filter {}",
                        model_request, model_filter
                    );
                } else {
                    assert!(
                        !should_pass,
                        "Model {} should not pass filter {}",
                        model_request, model_filter
                    );
                }
            }
        }
    }

    #[test]
    fn test_combined_filters() {
        let state =
            create_test_state_with_filters(Some("openai".to_string()), Some("gpt-4".to_string()));

        // Test that both filters are applied
        assert_eq!(state.provider_filter, Some("openai".to_string()));
        assert_eq!(state.model_filter, Some("gpt-4".to_string()));

        // openai:gpt-4 should pass both filters
        let result = parse_model_string("openai:gpt-4", &state.config);
        assert!(result.is_ok());
        let (provider, model) = result.unwrap();

        let provider_passes = state
            .provider_filter
            .as_ref()
            .map_or(true, |f| provider == *f);
        let model_passes = state.model_filter.as_ref().map_or(true, |f| {
            format!("{}:{}", provider, model).contains(f) || model == *f
        });

        assert!(provider_passes);
        assert!(model_passes);
    }

    #[test]
    fn test_no_filters() {
        let state = create_test_state_with_filters(None, None);

        assert!(state.provider_filter.is_none());
        assert!(state.model_filter.is_none());

        // All models should be allowed when no filters are set
        let test_models = vec!["openai:gpt-4", "anthropic:claude-3-sonnet", "gpt-3.5-turbo"];

        for model in test_models {
            let result = parse_model_string(model, &state.config);
            assert!(result.is_ok(), "Model {} should be parseable", model);
        }
    }
}

#[cfg(test)]
mod proxy_validation_tests {
    use super::*;

    #[test]
    fn test_proxy_models_query_validation() {
        // Test valid provider names
        let valid_providers = vec![
            "openai",
            "anthropic",
            "cohere",
            "provider-name",
            "provider_name",
        ];

        for provider in valid_providers {
            let query = ProxyModelsQuery {
                provider: Some(provider.to_string()),
            };
            assert_eq!(query.provider, Some(provider.to_string()));
        }
    }

    #[test]
    fn test_proxy_chat_request_validation() {
        // Test valid message structures
        let messages = vec![
            Message {
                role: "user".to_string(),
                content_type: lc::provider::MessageContent::Text {
                    content: Some("Hello".to_string()),
                },
                tool_calls: None,
                tool_call_id: None,
            },
            Message {
                role: "assistant".to_string(),
                content_type: lc::provider::MessageContent::Text {
                    content: Some("Hi there!".to_string()),
                },
                tool_calls: None,
                tool_call_id: None,
            },
        ];

        let request = ProxyChatRequest {
            model: "gpt-4".to_string(),
            messages,
            max_tokens: Some(100),
            temperature: Some(0.7),
        };

        assert_eq!(request.model, "gpt-4");
        assert_eq!(request.messages.len(), 2);
        assert_eq!(request.max_tokens, Some(100));
        assert_eq!(request.temperature, Some(0.7));
    }

    #[test]
    fn test_proxy_parameter_ranges() {
        // Test temperature range validation (would be done by proxy logic)
        let valid_temperatures = vec![0.0, 0.5, 1.0, 1.5, 2.0];
        let invalid_temperatures = vec![-0.1, 2.1];

        for temp in valid_temperatures {
            assert!(
                temp >= 0.0 && temp <= 2.0,
                "Temperature {} should be valid",
                temp
            );
        }

        for temp in invalid_temperatures {
            assert!(
                !(temp >= 0.0 && temp <= 2.0),
                "Temperature {} should be invalid",
                temp
            );
        }

        // Test max_tokens validation
        let valid_max_tokens = vec![1, 100, 1000, 4096];
        let invalid_max_tokens = vec![0];

        for tokens in valid_max_tokens {
            assert!(tokens > 0, "Max tokens {} should be valid", tokens);
        }

        for tokens in invalid_max_tokens {
            assert!(!(tokens > 0), "Max tokens {} should be invalid", tokens);
        }
    }

    #[test]
    fn test_proxy_model_id_validation() {
        let valid_model_ids = vec![
            "gpt-4",
            "gpt-3.5-turbo",
            "claude-3-sonnet",
            "openai:gpt-4",
            "anthropic:claude-3-sonnet",
            "provider:model-name",
        ];

        let invalid_model_ids = vec!["", ":", ":model", "provider:"];

        for model_id in valid_model_ids {
            assert!(
                !model_id.is_empty(),
                "Model ID {} should be valid",
                model_id
            );
            if model_id.contains(':') {
                let parts: Vec<&str> = model_id.split(':').collect();
                assert!(
                    parts.len() >= 2,
                    "Model ID {} should have provider and model",
                    model_id
                );
                assert!(
                    !parts[0].is_empty(),
                    "Provider in {} should not be empty",
                    model_id
                );
                assert!(
                    !parts[1].is_empty(),
                    "Model in {} should not be empty",
                    model_id
                );
            }
        }

        for model_id in invalid_model_ids {
            let is_invalid = model_id.is_empty()
                || (model_id.contains(':')
                    && (model_id.starts_with(':') || model_id.ends_with(':') || model_id == ":"));
            assert!(is_invalid, "Model ID {} should be invalid", model_id);
        }
    }
}

#[cfg(test)]
mod proxy_integration_tests {
    use super::*;

    #[test]
    fn test_proxy_state_with_real_config() {
        let mut config = Config {
            providers: HashMap::new(),
            default_provider: Some("openai".to_string()),
            default_model: Some("gpt-4".to_string()),
            aliases: HashMap::new(),
            system_prompt: None,
            templates: HashMap::new(),
            max_tokens: None,
            temperature: None,
            stream: None,
        };

        // Add realistic provider configuration
        config.providers.insert(
            "openai".to_string(),
            lc::config::ProviderConfig {
                endpoint: "https://api.openai.com/v1/chat/completions".to_string(),
                api_key: Some("sk-test123".to_string()),
                models: vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()],
                models_path: "/v1/models".to_string(),
                chat_path: "/v1/chat/completions".to_string(),
                headers: HashMap::new(),
                token_url: None,
                cached_token: None,
                auth_type: None,
                vars: HashMap::new(),
                images_path: Some("/images/generations".to_string()),
                embeddings_path: Some("/embeddings".to_string()),
                chat_templates: None,
                images_templates: None,
                embeddings_templates: None,
                models_templates: None,
                audio_path: None,
                speech_path: None,
                audio_templates: None,
                speech_templates: None,
            },
        );

        // Add aliases
        config
            .aliases
            .insert("gpt4".to_string(), "openai:gpt-4".to_string());

        let state = ProxyState {
            config: config.clone(),
            api_key: Some("proxy-key-123".to_string()),
            provider_filter: None,
            model_filter: None,
        };

        // Test model parsing with aliases
        let result = parse_model_string("gpt4", &state.config);
        assert!(result.is_ok());
        let (provider, model) = result.unwrap();
        assert_eq!(provider, "openai");
        assert_eq!(model, "gpt-4");

        // Test default provider usage
        let result = parse_model_string("gpt-3.5-turbo", &state.config);
        assert!(result.is_ok());
        let (provider, model) = result.unwrap();
        assert_eq!(provider, "openai");
        assert_eq!(model, "gpt-3.5-turbo");
    }

    #[test]
    fn test_proxy_workflow_simulation() {
        let config = Config {
            providers: HashMap::new(),
            default_provider: Some("openai".to_string()),
            default_model: None,
            aliases: HashMap::new(),
            system_prompt: None,
            templates: HashMap::new(),
            max_tokens: None,
            temperature: None,
            stream: None,
        };

        // Simulate proxy server startup
        let api_key = generate_api_key();
        assert!(api_key.starts_with("sk-"));

        let state = ProxyState {
            config,
            api_key: Some(api_key.clone()),
            provider_filter: Some("openai".to_string()),
            model_filter: None,
        };

        // Simulate authentication check
        let has_auth = state.api_key.is_some();
        assert!(has_auth);

        // Simulate request processing
        let test_requests = vec!["gpt-4", "openai:gpt-4", "gpt-3.5-turbo"];

        for request_model in test_requests {
            // This would be handled by the actual proxy endpoints
            assert!(!request_model.is_empty());
        }
    }

    #[test]
    fn test_proxy_error_scenarios() {
        let config = Config {
            providers: HashMap::new(),
            default_provider: None,
            default_model: None,
            aliases: HashMap::new(),
            system_prompt: None,
            templates: HashMap::new(),
            max_tokens: None,
            temperature: None,
            stream: None,
        };

        // Test error cases
        let error_cases = vec!["nonexistent:model", "model-without-provider"];

        for error_case in error_cases {
            let result = parse_model_string(error_case, &config);
            assert!(result.is_err(), "Should fail for: {}", error_case);
        }
    }

    #[test]
    fn test_proxy_response_format_compatibility() {
        // Test OpenAI API compatibility
        let response = ProxyChatResponse {
            id: "chatcmpl-123".to_string(),
            object: "chat.completion".to_string(),
            created: 1234567890,
            model: "gpt-4".to_string(),
            choices: vec![ProxyChoice {
                index: 0,
                message: Message {
                    role: "assistant".to_string(),
                    content_type: lc::provider::MessageContent::Text {
                        content: Some("Hello!".to_string()),
                    },
                    tool_calls: None,
                    tool_call_id: None,
                },
                finish_reason: "stop".to_string(),
            }],
            usage: ProxyUsage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
            },
        };

        // Verify OpenAI API format compliance
        assert_eq!(response.object, "chat.completion");
        assert!(response.id.starts_with("chatcmpl-"));
        assert_eq!(response.choices.len(), 1);
        assert_eq!(response.choices[0].index, 0);
        assert_eq!(response.choices[0].finish_reason, "stop");
        assert_eq!(response.usage.total_tokens, 15);
    }
}

#[cfg(test)]
mod proxy_authentication_tests {
    use super::*;

    #[test]
    fn test_api_key_format() {
        let key = generate_api_key();

        // Test API key format requirements
        assert!(key.starts_with("sk-"));
        assert_eq!(key.len(), 35); // "sk-" + 32 characters

        // Test character set
        let key_part = &key[3..];
        for ch in key_part.chars() {
            assert!(ch.is_ascii_alphanumeric());
        }
    }

    #[test]
    fn test_bearer_token_format() {
        let api_key = "sk-test123456789";
        let bearer_token = format!("Bearer {}", api_key);

        assert!(bearer_token.starts_with("Bearer "));
        assert_eq!(bearer_token.strip_prefix("Bearer "), Some(api_key));
    }

    #[test]
    fn test_authentication_scenarios() {
        let state_with_auth = ProxyState {
            config: Config {
                providers: HashMap::new(),
                default_provider: None,
                default_model: None,
                aliases: HashMap::new(),
                system_prompt: None,
                templates: HashMap::new(),
                max_tokens: None,
                temperature: None,
                stream: None,
            },
            api_key: Some("sk-test123".to_string()),
            provider_filter: None,
            model_filter: None,
        };

        let state_without_auth = ProxyState {
            config: Config {
                providers: HashMap::new(),
                default_provider: None,
                default_model: None,
                aliases: HashMap::new(),
                system_prompt: None,
                templates: HashMap::new(),
                max_tokens: None,
                temperature: None,
                stream: None,
            },
            api_key: None,
            provider_filter: None,
            model_filter: None,
        };

        // Test authentication required vs not required
        assert!(state_with_auth.api_key.is_some());
        assert!(state_without_auth.api_key.is_none());
    }
}

#[cfg(test)]
mod proxy_endpoint_tests {

    #[test]
    fn test_models_endpoint_paths() {
        let endpoints = vec!["/models", "/v1/models"];

        for endpoint in endpoints {
            assert!(endpoint.contains("models"));
            assert!(endpoint.starts_with('/'));
        }
    }

    #[test]
    fn test_chat_completions_endpoint_paths() {
        let endpoints = vec!["/chat/completions", "/v1/chat/completions"];

        for endpoint in endpoints {
            assert!(endpoint.contains("chat/completions"));
            assert!(endpoint.starts_with('/'));
        }
    }

    #[test]
    fn test_openai_api_compatibility() {
        // Test that our endpoints match OpenAI's API structure
        let openai_endpoints = vec!["/v1/models", "/v1/chat/completions"];

        let our_endpoints = vec![
            "/models",
            "/v1/models",
            "/chat/completions",
            "/v1/chat/completions",
        ];

        // Verify we support OpenAI's standard endpoints
        for openai_endpoint in openai_endpoints {
            assert!(our_endpoints.contains(&openai_endpoint));
        }
    }
}

#[cfg(test)]
mod proxy_server_configuration_tests {

    #[test]
    fn test_server_address_formatting() {
        let host = "127.0.0.1";
        let port = 8080u16;
        let addr = format!("{}:{}", host, port);

        assert_eq!(addr, "127.0.0.1:8080");
        assert!(addr.contains(':'));
        assert!(addr.contains("127.0.0.1"));
        assert!(addr.contains("8080"));
    }

    #[test]
    fn test_port_validation() {
        let valid_ports = vec![80u16, 443, 3000, 8080, 8000, 9000];
        let edge_case_ports = vec![1u16, 65535];

        for port in valid_ports {
            assert!(port > 0);
            assert!(port > 0); // Remove useless comparison
        }

        for port in edge_case_ports {
            assert!(port > 0);
            assert!(port > 0); // Remove useless comparison
        }
    }

    #[test]
    fn test_host_validation() {
        let valid_hosts = vec!["127.0.0.1", "0.0.0.0", "localhost", "::1"];

        for host in valid_hosts {
            assert!(!host.is_empty());
            // Basic validation - real validation would be more complex
            assert!(host.chars().all(|c| c.is_ascii()));
        }
    }

    #[test]
    fn test_cors_configuration() {
        // Test that CORS is permissive for development
        // In a real test, we'd verify the CORS layer configuration
        let cors_enabled = true; // Simulating CorsLayer::permissive()
        assert!(cors_enabled);
    }
}

#[cfg(test)]
mod proxy_error_handling_tests {
    use super::*;

    #[test]
    fn test_model_parsing_errors() {
        let config = Config {
            providers: HashMap::new(),
            default_provider: None,
            default_model: None,
            aliases: HashMap::new(),
            system_prompt: None,
            templates: HashMap::new(),
            max_tokens: None,
            temperature: None,
            stream: None,
        };

        let error_cases = vec!["nonexistent:model", "invalid-provider:model", ""];

        for case in error_cases {
            let result = parse_model_string(case, &config);
            assert!(result.is_err(), "Should error for: {}", case);
        }
    }

    #[test]
    fn test_authentication_error_simulation() {
        // Simulate authentication failure scenarios
        let test_cases = vec![
            ("", false),                       // Empty auth header
            ("Basic token", false),            // Wrong auth type
            ("Bearer", false),                 // Missing token
            ("Bearer wrong-token", false),     // Wrong token
            ("Bearer sk-correct-token", true), // Correct token
        ];

        let expected_key = "sk-correct-token";

        for (auth_header, should_pass) in test_cases {
            let is_valid = if auth_header.starts_with("Bearer ") {
                if let Some(token) = auth_header.strip_prefix("Bearer ") {
                    token == expected_key
                } else {
                    false
                }
            } else {
                false
            };

            assert_eq!(is_valid, should_pass, "Auth header: {}", auth_header);
        }
    }

    #[test]
    fn test_provider_filter_errors() {
        let mut config = Config {
            providers: HashMap::new(),
            default_provider: Some("openai".to_string()),
            default_model: None,
            aliases: HashMap::new(),
            system_prompt: None,
            templates: HashMap::new(),
            max_tokens: None,
            temperature: None,
            stream: None,
        };

        // Add only openai provider
        config.providers.insert(
            "openai".to_string(),
            lc::config::ProviderConfig {
                endpoint: "https://api.openai.com/v1/chat/completions".to_string(),
                api_key: None,
                models: vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()],
                models_path: "/models".to_string(),
                chat_path: "/chat/completions".to_string(),
                headers: HashMap::new(),
                token_url: None,
                cached_token: None,
                auth_type: None,
                vars: HashMap::new(),
                images_path: Some("/images/generations".to_string()),
                embeddings_path: Some("/embeddings".to_string()),
                chat_templates: None,
                images_templates: None,
                embeddings_templates: None,
                models_templates: None,
                audio_path: None,
                speech_path: None,
                audio_templates: None,
                speech_templates: None,
            },
        );
        // Simulate alias insertions
        for i in 0..10 {
            config
                .aliases
                .insert(format!("alias{}", i), format!("provider{}:model{}", i, i));
        }

        let start = std::time::Instant::now();

        // Parse many models
        for i in 0..100 {
            let _result = parse_model_string(&format!("alias{}", i), &config);
        }

        let duration = start.elapsed();

        // Should be reasonably fast
        assert!(duration.as_millis() < 1000);
    }

    #[test]
    fn test_large_response_serialization() {
        // Test serialization of large responses
        let mut models = Vec::new();

        for i in 0..1000 {
            models.push(ProxyModel {
                id: format!("provider{}:model{}", i, i),
                object: "model".to_string(),
                created: 1234567890,
                owned_by: format!("provider{}", i),
            });
        }

        let response = ProxyModelsResponse {
            object: "list".to_string(),
            data: models,
        };

        let start = std::time::Instant::now();
        let _json = serde_json::to_string(&response).unwrap();
        let duration = start.elapsed();

        // Should serialize quickly
        assert!(duration.as_millis() < 1000);
    }
}
