//! Integration tests for embed commands
//! 
//! This module contains comprehensive integration tests for all embedding-related
//! CLI commands, testing the underlying functionality as the CLI would use it.

mod common;

use lc::vector_db::VectorDatabase;
use lc::provider::{EmbeddingRequest, EmbeddingResponse, EmbeddingData, EmbeddingUsage};
use lc::config::Config;
use std::collections::HashMap;
use tempfile::TempDir;

#[cfg(test)]
mod embed_request_tests {
    use super::*;

    #[test]
    fn test_embedding_request_creation() {
        let request = EmbeddingRequest {
            model: "text-embedding-3-small".to_string(),
            input: "Test text for embedding".to_string(),
            encoding_format: Some("float".to_string()),
        };

        assert_eq!(request.model, "text-embedding-3-small");
        assert_eq!(request.input, "Test text for embedding");
        assert_eq!(request.encoding_format, Some("float".to_string()));
    }

    #[test]
    fn test_embedding_request_without_encoding_format() {
        let request = EmbeddingRequest {
            model: "text-embedding-ada-002".to_string(),
            input: "Another test text".to_string(),
            encoding_format: None,
        };

        assert_eq!(request.model, "text-embedding-ada-002");
        assert_eq!(request.input, "Another test text");
        assert_eq!(request.encoding_format, None);
    }

    #[test]
    fn test_embedding_request_with_long_text() {
        let long_text = "This is a very long text that might be used for embedding. ".repeat(100);
        let request = EmbeddingRequest {
            model: "text-embedding-3-large".to_string(),
            input: long_text.clone(),
            encoding_format: Some("float".to_string()),
        };

        assert_eq!(request.model, "text-embedding-3-large");
        assert_eq!(request.input, long_text);
        assert!(request.input.len() > 1000);
    }
}

#[cfg(test)]
mod embed_response_tests {
    use super::*;

    fn create_test_embedding_data() -> EmbeddingData {
        EmbeddingData {
            embedding: vec![0.1, 0.2, 0.3, 0.4, 0.5],
            index: 0,
            object: "embedding".to_string(),
        }
    }

    fn create_test_embedding_usage() -> EmbeddingUsage {
        EmbeddingUsage {
            prompt_tokens: 10,
            total_tokens: 10,
        }
    }

    #[test]
    fn test_embedding_response_creation() {
        let data = vec![create_test_embedding_data()];
        let usage = create_test_embedding_usage();

        let response = EmbeddingResponse {
            data: data.clone(),
            model: "text-embedding-3-small".to_string(),
            usage,
        };

        assert_eq!(response.data.len(), 1);
        assert_eq!(response.model, "text-embedding-3-small");
        assert_eq!(response.usage.prompt_tokens, 10);
        assert_eq!(response.usage.total_tokens, 10);
    }

    #[test]
    fn test_embedding_data_structure() {
        let embedding_data = create_test_embedding_data();

        assert_eq!(embedding_data.embedding.len(), 5);
        assert_eq!(embedding_data.embedding[0], 0.1);
        assert_eq!(embedding_data.embedding[4], 0.5);
        assert_eq!(embedding_data.index, 0);
        assert_eq!(embedding_data.object, "embedding");
    }

    #[test]
    fn test_embedding_usage_structure() {
        let usage = create_test_embedding_usage();

        assert_eq!(usage.prompt_tokens, 10);
        assert_eq!(usage.total_tokens, 10);
    }

    #[test]
    fn test_multiple_embeddings_response() {
        let data = vec![
            EmbeddingData {
                embedding: vec![0.1, 0.2, 0.3],
                index: 0,
                object: "embedding".to_string(),
            },
            EmbeddingData {
                embedding: vec![0.4, 0.5, 0.6],
                index: 1,
                object: "embedding".to_string(),
            },
        ];

        let response = EmbeddingResponse {
            data: data.clone(),
            model: "text-embedding-3-small".to_string(),
            usage: EmbeddingUsage {
                prompt_tokens: 20,
                total_tokens: 20,
            },
        };

        assert_eq!(response.data.len(), 2);
        assert_eq!(response.data[0].index, 0);
        assert_eq!(response.data[1].index, 1);
        assert_eq!(response.data[0].embedding[0], 0.1);
        assert_eq!(response.data[1].embedding[0], 0.4);
    }
}

#[cfg(test)]
mod embed_model_resolution_tests {
    use super::*;

    fn create_test_config_with_embedding_providers() -> Config {
        let mut config = Config {
            providers: HashMap::new(),
            default_provider: Some("openai".to_string()),
            default_model: Some("text-embedding-3-small".to_string()),
            aliases: HashMap::new(),
            system_prompt: None,
            templates: HashMap::new(),
            max_tokens: None,
            temperature: None,
        };

        // Add OpenAI provider with embedding models
        config.providers.insert("openai".to_string(), lc::config::ProviderConfig {
            endpoint: "https://api.openai.com/v1".to_string(),
            api_key: Some("sk-test123".to_string()),
            models: vec![
                "text-embedding-3-small".to_string(),
                "text-embedding-3-large".to_string(),
                "text-embedding-ada-002".to_string(),
            ],
            models_path: "/v1/models".to_string(),
            chat_path: "/v1/chat/completions".to_string(),
            headers: HashMap::new(),
            token_url: None,
            cached_token: None,
        });

        // Add Cohere provider with embedding models
        config.providers.insert("cohere".to_string(), lc::config::ProviderConfig {
            endpoint: "https://api.cohere.ai/v1".to_string(),
            api_key: Some("cohere-test-key".to_string()),
            models: vec![
                "embed-english-v3.0".to_string(),
                "embed-multilingual-v3.0".to_string(),
            ],
            models_path: "/v1/models".to_string(),
            chat_path: "/v1/chat".to_string(),
            headers: HashMap::new(),
            token_url: None,
            cached_token: None,
        });

        // Add embedding model aliases
        config.aliases.insert("small".to_string(), "openai:text-embedding-3-small".to_string());
        config.aliases.insert("large".to_string(), "openai:text-embedding-3-large".to_string());
        config.aliases.insert("cohere-en".to_string(), "cohere:embed-english-v3.0".to_string());

        config
    }

    #[test]
    fn test_embed_model_resolution_with_defaults() {
        let config = create_test_config_with_embedding_providers();

        // Test using defaults
        let result = lc::cli::resolve_model_and_provider(&config, None, None);
        assert!(result.is_ok());
        let (provider, model) = result.unwrap();
        assert_eq!(provider, "openai");
        assert_eq!(model, "text-embedding-3-small");
    }

    #[test]
    fn test_embed_model_resolution_with_provider_model_format() {
        let config = create_test_config_with_embedding_providers();

        // Test provider:model format
        let result = lc::cli::resolve_model_and_provider(&config, None, Some("cohere:embed-english-v3.0".to_string()));
        assert!(result.is_ok());
        let (provider, model) = result.unwrap();
        assert_eq!(provider, "cohere");
        assert_eq!(model, "embed-english-v3.0");
    }

    #[test]
    fn test_embed_model_resolution_with_aliases() {
        let config = create_test_config_with_embedding_providers();

        // Test alias resolution
        let result = lc::cli::resolve_model_and_provider(&config, None, Some("large".to_string()));
        assert!(result.is_ok());
        let (provider, model) = result.unwrap();
        assert_eq!(provider, "openai");
        assert_eq!(model, "text-embedding-3-large");

        let result = lc::cli::resolve_model_and_provider(&config, None, Some("cohere-en".to_string()));
        assert!(result.is_ok());
        let (provider, model) = result.unwrap();
        assert_eq!(provider, "cohere");
        assert_eq!(model, "embed-english-v3.0");
    }

    #[test]
    fn test_embed_model_resolution_with_explicit_provider() {
        let config = create_test_config_with_embedding_providers();

        // Test with explicit provider override
        let result = lc::cli::resolve_model_and_provider(&config, Some("cohere".to_string()), Some("embed-multilingual-v3.0".to_string()));
        assert!(result.is_ok());
        let (provider, model) = result.unwrap();
        assert_eq!(provider, "cohere");
        assert_eq!(model, "embed-multilingual-v3.0");
    }
}

#[cfg(test)]
mod embed_validation_tests {
    use super::*;

    #[test]
    fn test_embedding_text_validation() {
        // Test empty text
        let empty_text = "";
        assert!(empty_text.is_empty());

        // Test normal text
        let normal_text = "This is a normal text for embedding";
        assert!(!normal_text.is_empty());
        assert!(normal_text.len() > 0);

        // Test very long text
        let long_text = "word ".repeat(20000);
        assert!(long_text.len() > 50000);
        // In real usage, this might need chunking or truncation
    }

    #[test]
    fn test_embedding_model_name_validation() {
        let valid_models = vec![
            "text-embedding-3-small",
            "text-embedding-3-large",
            "text-embedding-ada-002",
            "embed-english-v3.0",
            "embed-multilingual-v3.0",
        ];

        for model in valid_models {
            assert!(!model.is_empty());
            assert!(!model.trim().is_empty());
            // Model names should not contain spaces
            assert!(!model.contains(' '));
        }
    }

    #[test]
    fn test_embedding_vector_validation() {
        // Test valid embedding vector
        let valid_vector = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        assert!(!valid_vector.is_empty());
        assert!(valid_vector.iter().all(|&x: &f64| x.is_finite()));

        // Test vector with invalid values
        let invalid_vector = vec![0.1, f64::NAN, 0.3, f64::INFINITY, 0.5];
        assert!(!invalid_vector.iter().all(|&x| x.is_finite()));

        // Test empty vector
        let empty_vector: Vec<f64> = vec![];
        assert!(empty_vector.is_empty());
    }

    #[test]
    fn test_embedding_dimensions_consistency() {
        // Test that embeddings from the same model should have consistent dimensions
        let embedding1 = vec![0.1; 1536]; // OpenAI text-embedding-3-small dimension
        let embedding2 = vec![0.2; 1536];
        let embedding3 = vec![0.3; 1024]; // Different dimension

        assert_eq!(embedding1.len(), embedding2.len());
        assert_ne!(embedding1.len(), embedding3.len());

        // In real usage, we should validate dimension consistency
        fn validate_dimension_consistency(embeddings: &[Vec<f64>]) -> bool {
            if embeddings.is_empty() {
                return true;
            }
            let expected_dim = embeddings[0].len();
            embeddings.iter().all(|emb| emb.len() == expected_dim)
        }

        assert!(validate_dimension_consistency(&[embedding1.clone(), embedding2.clone()]));
        assert!(!validate_dimension_consistency(&[embedding1, embedding3]));
    }
}

#[cfg(test)]
mod embed_error_handling_tests {
    use super::*;

    #[test]
    fn test_embed_with_invalid_provider() {
        let config = Config {
            providers: HashMap::new(),
            default_provider: None,
            default_model: None,
            aliases: HashMap::new(),
            system_prompt: None,
            templates: HashMap::new(),
            max_tokens: None,
            temperature: None,
        };

        // Test with non-existent provider
        let result = lc::cli::resolve_model_and_provider(&config, Some("nonexistent".to_string()), Some("some-model".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_embed_with_missing_api_key() {
        let mut config = Config {
            providers: HashMap::new(),
            default_provider: Some("openai".to_string()),
            default_model: Some("text-embedding-3-small".to_string()),
            aliases: HashMap::new(),
            system_prompt: None,
            templates: HashMap::new(),
            max_tokens: None,
            temperature: None,
        };

        // Add provider without API key
        config.providers.insert("openai".to_string(), lc::config::ProviderConfig {
            endpoint: "https://api.openai.com/v1".to_string(),
            api_key: None, // No API key
            models: vec!["text-embedding-3-small".to_string()],
            models_path: "/v1/models".to_string(),
            chat_path: "/v1/chat/completions".to_string(),
            headers: HashMap::new(),
            token_url: None,
            cached_token: None,
        });

        // This would fail in actual usage when trying to create authenticated client
        let provider_config = config.get_provider("openai").unwrap();
        assert!(provider_config.api_key.is_none());
    }

    #[test]
    fn test_embed_with_invalid_alias() {
        let mut config = Config {
            providers: HashMap::new(),
            default_provider: Some("openai".to_string()),
            default_model: None,
            aliases: HashMap::new(),
            system_prompt: None,
            templates: HashMap::new(),
            max_tokens: None,
            temperature: None,
        };

        // Add provider
        config.providers.insert("openai".to_string(), lc::config::ProviderConfig {
            endpoint: "https://api.openai.com/v1".to_string(),
            api_key: Some("sk-test123".to_string()),
            models: vec!["text-embedding-3-small".to_string()],
            models_path: "/v1/models".to_string(),
            chat_path: "/v1/chat/completions".to_string(),
            headers: HashMap::new(),
            token_url: None,
            cached_token: None,
        });

        // Add invalid alias (missing provider:model format)
        config.aliases.insert("invalid_alias".to_string(), "just-a-model".to_string());

        let result = lc::cli::resolve_model_and_provider(&config, None, Some("invalid_alias".to_string()));
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod embed_integration_tests {
    use super::*;

    #[test]
    fn test_complete_embed_workflow_simulation() {
        let config = Config {
            providers: HashMap::new(),
            default_provider: Some("openai".to_string()),
            default_model: Some("text-embedding-3-small".to_string()),
            aliases: HashMap::new(),
            system_prompt: None,
            templates: HashMap::new(),
            max_tokens: None,
            temperature: None,
        };

        let text = "Machine learning is a subset of artificial intelligence";

        // Test model resolution
        let result = lc::cli::resolve_model_and_provider(&config, None, None);
        assert!(result.is_ok());
        let (provider, model) = result.unwrap();
        assert_eq!(provider, "openai");
        assert_eq!(model, "text-embedding-3-small");

        // Test embedding request creation
        let request = EmbeddingRequest {
            model: model.clone(),
            input: text.to_string(),
            encoding_format: Some("float".to_string()),
        };

        assert_eq!(request.model, "text-embedding-3-small");
        assert_eq!(request.input, text);
        assert_eq!(request.encoding_format, Some("float".to_string()));

        // Simulate successful response
        let mock_response = EmbeddingResponse {
            data: vec![EmbeddingData {
                embedding: vec![0.1; 1536], // OpenAI text-embedding-3-small dimension
                index: 0,
                object: "embedding".to_string(),
            }],
            model: model.clone(),
            usage: EmbeddingUsage {
                prompt_tokens: 12,
                total_tokens: 12,
            },
        };

        assert_eq!(mock_response.data.len(), 1);
        assert_eq!(mock_response.data[0].embedding.len(), 1536);
        assert_eq!(mock_response.usage.prompt_tokens, 12);
    }

    #[test]
    fn test_embed_with_different_providers() {
        let mut config = Config {
            providers: HashMap::new(),
            default_provider: None,
            default_model: None,
            aliases: HashMap::new(),
            system_prompt: None,
            templates: HashMap::new(),
            max_tokens: None,
            temperature: None,
        };

        // Add multiple providers
        config.providers.insert("openai".to_string(), lc::config::ProviderConfig {
            endpoint: "https://api.openai.com/v1".to_string(),
            api_key: Some("sk-openai-test".to_string()),
            models: vec!["text-embedding-3-small".to_string()],
            models_path: "/v1/models".to_string(),
            chat_path: "/v1/chat/completions".to_string(),
            headers: HashMap::new(),
            token_url: None,
            cached_token: None,
        });

        config.providers.insert("cohere".to_string(), lc::config::ProviderConfig {
            endpoint: "https://api.cohere.ai/v1".to_string(),
            api_key: Some("cohere-test-key".to_string()),
            models: vec!["embed-english-v3.0".to_string()],
            models_path: "/v1/models".to_string(),
            chat_path: "/v1/chat".to_string(),
            headers: HashMap::new(),
            token_url: None,
            cached_token: None,
        });

        // Test OpenAI
        let result = lc::cli::resolve_model_and_provider(&config, Some("openai".to_string()), Some("text-embedding-3-small".to_string()));
        assert!(result.is_ok());
        let (provider, model) = result.unwrap();
        assert_eq!(provider, "openai");
        assert_eq!(model, "text-embedding-3-small");

        // Test Cohere
        let result = lc::cli::resolve_model_and_provider(&config, Some("cohere".to_string()), Some("embed-english-v3.0".to_string()));
        assert!(result.is_ok());
        let (provider, model) = result.unwrap();
        assert_eq!(provider, "cohere");
        assert_eq!(model, "embed-english-v3.0");
    }

    #[test]
    fn test_embed_parameter_validation_workflow() {
        // Test various text inputs
        let test_cases = vec![
            ("Short text", true),
            ("", false), // Empty text should be invalid
            ("A", true), // Short text should be valid
            ("Text with special characters: !@#$%^&*()", true),
            ("Multi-line\ntext\nwith\nbreaks", true),
            ("Text with unicode: 你好世界 🌍", true),
        ];

        for (text, should_be_valid) in test_cases {
            let is_valid = !text.is_empty() && !text.trim().is_empty();
            assert_eq!(is_valid, should_be_valid, "Text validation failed for: '{}'", text);

            if is_valid {
                let request = EmbeddingRequest {
                    model: "text-embedding-3-small".to_string(),
                    input: text.to_string(),
                    encoding_format: Some("float".to_string()),
                };
                assert_eq!(request.input, text);
            }
        }
    }
}