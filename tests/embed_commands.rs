//! Integration tests for embed commands
//!
//! This module contains comprehensive integration tests for all embedding-related
//! CLI commands, testing the underlying functionality as the CLI would use it.

mod common;

use lc::config::Config;
use lc::provider::{EmbeddingData, EmbeddingRequest, EmbeddingResponse, EmbeddingUsage};
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
        }
    }

    fn create_test_embedding_usage() -> EmbeddingUsage {
        EmbeddingUsage { total_tokens: 10 }
    }

    #[test]
    fn test_embedding_response_creation() {
        let data = vec![create_test_embedding_data()];
        let usage = create_test_embedding_usage();

        let response = EmbeddingResponse {
            data: data.clone(),
            usage,
        };

        assert_eq!(response.data.len(), 1);
        assert_eq!(response.usage.total_tokens, 10);
    }

    #[test]
    fn test_embedding_data_structure() {
        let embedding_data = create_test_embedding_data();

        assert_eq!(embedding_data.embedding.len(), 5);
        assert_eq!(embedding_data.embedding[0], 0.1);
        assert_eq!(embedding_data.embedding[4], 0.5);
    }

    #[test]
    fn test_embedding_usage_structure() {
        let usage = create_test_embedding_usage();

        assert_eq!(usage.total_tokens, 10);
    }

    #[test]
    fn test_multiple_embeddings_response() {
        let data = vec![
            EmbeddingData {
                embedding: vec![0.1, 0.2, 0.3],
            },
            EmbeddingData {
                embedding: vec![0.4, 0.5, 0.6],
            },
        ];

        let response = EmbeddingResponse {
            data: data.clone(),
            usage: EmbeddingUsage { total_tokens: 20 },
        };

        assert_eq!(response.data.len(), 2);
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
            stream: None,
        };

        // Add OpenAI provider with embedding models
        config.providers.insert(
            "openai".to_string(),
            lc::config::ProviderConfig {
                endpoint: "https://api.openai.com/v1".to_string(),
                api_key: Some("sk-test123".to_string()),
                models: vec![
                    "text-embedding-3-small".to_string(),
                    "text-embedding-3-large".to_string(),
                    "text-embedding-ada-002".to_string(),
                ],
                models_path: "/v1/models".to_string(),
                chat_path: "/v1/chat/completions".to_string(),
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
                headers: HashMap::new(),
                token_url: None,
                cached_token: None,
                auth_type: None,
                vars: HashMap::new(),
            },
        );

        // Add Cohere provider with embedding models
        config.providers.insert(
            "cohere".to_string(),
            lc::config::ProviderConfig {
                endpoint: "https://api.cohere.ai/v1".to_string(),
                api_key: Some("cohere-test-key".to_string()),
                models: vec![
                    "embed-english-v3.0".to_string(),
                    "embed-multilingual-v3.0".to_string(),
                ],
                models_path: "/v1/models".to_string(),
                chat_path: "/v1/chat".to_string(),
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
                headers: HashMap::new(),
                token_url: None,
                cached_token: None,
                auth_type: None,
                vars: HashMap::new(),
            },
        );

        // Add embedding model aliases
        config.aliases.insert(
            "small".to_string(),
            "openai:text-embedding-3-small".to_string(),
        );
        config.aliases.insert(
            "large".to_string(),
            "openai:text-embedding-3-large".to_string(),
        );
        config.aliases.insert(
            "cohere-en".to_string(),
            "cohere:embed-english-v3.0".to_string(),
        );

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
        let result = lc::cli::resolve_model_and_provider(
            &config,
            None,
            Some("cohere:embed-english-v3.0".to_string()),
        );
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

        let result =
            lc::cli::resolve_model_and_provider(&config, None, Some("cohere-en".to_string()));
        assert!(result.is_ok());
        let (provider, model) = result.unwrap();
        assert_eq!(provider, "cohere");
        assert_eq!(model, "embed-english-v3.0");
    }

    #[test]
    fn test_embed_model_resolution_with_explicit_provider() {
        let config = create_test_config_with_embedding_providers();

        // Test with explicit provider override
        let result = lc::cli::resolve_model_and_provider(
            &config,
            Some("cohere".to_string()),
            Some("embed-multilingual-v3.0".to_string()),
        );
        assert!(result.is_ok());
        let (provider, model) = result.unwrap();
        assert_eq!(provider, "cohere");
        assert_eq!(model, "embed-multilingual-v3.0");
    }
}

#[cfg(test)]
mod embed_validation_tests {

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

        assert!(validate_dimension_consistency(&[
            embedding1.clone(),
            embedding2.clone()
        ]));
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
            stream: None,
        };

        // Test with non-existent provider
        let result = lc::cli::resolve_model_and_provider(
            &config,
            Some("nonexistent".to_string()),
            Some("some-model".to_string()),
        );
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
            stream: None,
        };

        // Add provider without API key
        config.providers.insert(
            "openai".to_string(),
            lc::config::ProviderConfig {
                endpoint: "https://api.openai.com/v1".to_string(),
                api_key: None, // No API key
                models: vec!["text-embedding-3-small".to_string()],
                models_path: "/v1/models".to_string(),
                chat_path: "/v1/chat/completions".to_string(),
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
                headers: HashMap::new(),
                token_url: None,
                cached_token: None,
                auth_type: None,
                vars: HashMap::new(),
            },
        );

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
            stream: None,
        };

        // Add provider
        config.providers.insert(
            "openai".to_string(),
            lc::config::ProviderConfig {
                endpoint: "https://api.openai.com/v1".to_string(),
                api_key: Some("sk-test123".to_string()),
                models: vec!["text-embedding-3-small".to_string()],
                models_path: "/v1/models".to_string(),
                chat_path: "/v1/chat/completions".to_string(),
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
                headers: HashMap::new(),
                token_url: None,
                cached_token: None,
                auth_type: None,
                vars: HashMap::new(),
            },
        );

        // Add invalid alias (missing provider:model format)
        config
            .aliases
            .insert("invalid_alias".to_string(), "just-a-model".to_string());

        let result =
            lc::cli::resolve_model_and_provider(&config, None, Some("invalid_alias".to_string()));
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
            stream: None,
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
            }],
            usage: EmbeddingUsage { total_tokens: 12 },
        };

        assert_eq!(mock_response.data.len(), 1);
        assert_eq!(mock_response.data[0].embedding.len(), 1536);
        assert_eq!(mock_response.usage.total_tokens, 12);
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
            stream: None,
        };

        // Add multiple providers
        config.providers.insert(
            "openai".to_string(),
            lc::config::ProviderConfig {
                endpoint: "https://api.openai.com/v1".to_string(),
                api_key: Some("sk-openai-test".to_string()),
                models: vec!["text-embedding-3-small".to_string()],
                models_path: "/v1/models".to_string(),
                chat_path: "/v1/chat/completions".to_string(),
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
                headers: HashMap::new(),
                token_url: None,
                cached_token: None,
                auth_type: None,
                vars: HashMap::new(),
            },
        );

        config.providers.insert(
            "cohere".to_string(),
            lc::config::ProviderConfig {
                endpoint: "https://api.cohere.ai/v1".to_string(),
                api_key: Some("cohere-test-key".to_string()),
                models: vec!["embed-english-v3.0".to_string()],
                models_path: "/v1/models".to_string(),
                chat_path: "/v1/chat".to_string(),
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
                headers: HashMap::new(),
                token_url: None,
                cached_token: None,
                auth_type: None,
                vars: HashMap::new(),
            },
        );

        // Test OpenAI
        let result = lc::cli::resolve_model_and_provider(
            &config,
            Some("openai".to_string()),
            Some("text-embedding-3-small".to_string()),
        );
        assert!(result.is_ok());
        let (provider, model) = result.unwrap();
        assert_eq!(provider, "openai");
        assert_eq!(model, "text-embedding-3-small");

        // Test Cohere
        let result = lc::cli::resolve_model_and_provider(
            &config,
            Some("cohere".to_string()),
            Some("embed-english-v3.0".to_string()),
        );
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
            assert_eq!(
                is_valid, should_be_valid,
                "Text validation failed for: '{}'",
                text
            );

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

#[cfg(test)]
mod embed_file_tests {
    use super::*;
    use lc::vector_db::FileProcessor;
    use std::fs;
    use std::path::Path;

    fn create_test_files(temp_dir: &TempDir) -> Vec<String> {
        let test_files = vec![
            ("test1.txt", "This is the content of test file 1.\nIt has multiple lines.\nAnd some more content."),
            ("test2.md", "# Test Markdown\n\nThis is a markdown file with **bold** text.\n\n## Section\n\nMore content here."),
            ("test3.py", "#!/usr/bin/env python3\n\ndef hello_world():\n    print('Hello, World!')\n\nif __name__ == '__main__':\n    hello_world()"),
            ("test4.rs", "fn main() {\n    println!(\"Hello, Rust!\");\n}\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn test_example() {\n        assert_eq!(2 + 2, 4);\n    }\n}"),
            ("config.json", "{\n  \"name\": \"test\",\n  \"version\": \"1.0.0\",\n  \"dependencies\": {\n    \"example\": \"^1.0.0\"\n  }\n}"),
        ];

        let mut file_paths = Vec::new();
        for (filename, content) in test_files {
            let file_path = temp_dir.path().join(filename);
            fs::write(&file_path, content).expect("Failed to write test file");
            file_paths.push(file_path.to_string_lossy().to_string());
        }

        // Create binary file separately
        let binary_file = temp_dir.path().join("binary.bin");
        fs::write(&binary_file, &[0u8, 1u8, 2u8, 3u8, 4u8, 5u8])
            .expect("Failed to write binary file");
        file_paths.push(binary_file.to_string_lossy().to_string());

        file_paths
    }

    fn create_nested_test_files(temp_dir: &TempDir) -> Vec<String> {
        // Create nested directory structure
        let docs_dir = temp_dir.path().join("docs");
        let src_dir = temp_dir.path().join("src");
        fs::create_dir_all(&docs_dir).expect("Failed to create docs directory");
        fs::create_dir_all(&src_dir).expect("Failed to create src directory");

        let nested_files = vec![
            ("docs/readme.md", "# Project Documentation\n\nThis is the main documentation file.\n\n## Features\n\n- Feature 1\n- Feature 2"),
            ("docs/api.md", "# API Documentation\n\n## Endpoints\n\n### GET /api/users\n\nReturns a list of users."),
            ("src/main.rs", "use std::io;\n\nfn main() {\n    println!(\"Enter your name:\");\n    let mut input = String::new();\n    io::stdin().read_line(&mut input).expect(\"Failed to read line\");\n    println!(\"Hello, {}!\", input.trim());\n}"),
            ("src/lib.rs", "//! Library documentation\n\npub mod utils;\n\npub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n\n    #[test]\n    fn test_add() {\n        assert_eq!(add(2, 3), 5);\n    }\n}"),
        ];

        let mut file_paths = Vec::new();
        for (relative_path, content) in nested_files {
            let file_path = temp_dir.path().join(relative_path);
            fs::write(&file_path, content).expect("Failed to write nested test file");
            file_paths.push(file_path.to_string_lossy().to_string());
        }

        file_paths
    }

    #[test]
    fn test_file_processor_text_file_detection() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        create_test_files(&temp_dir);

        let text_files = vec![
            temp_dir.path().join("test1.txt"),
            temp_dir.path().join("test2.md"),
            temp_dir.path().join("test3.py"),
            temp_dir.path().join("test4.rs"),
            temp_dir.path().join("config.json"),
        ];

        let binary_file = temp_dir.path().join("binary.bin");

        for file in text_files {
            assert!(
                FileProcessor::is_text_file(&file),
                "Should detect {} as text file",
                file.display()
            );
        }

        assert!(
            !FileProcessor::is_text_file(&binary_file),
            "Should detect binary.bin as binary file"
        );
    }

    #[test]
    fn test_file_processor_glob_expansion() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        create_test_files(&temp_dir);

        // Test glob pattern for markdown files
        let md_pattern = format!("{}/*.md", temp_dir.path().display());
        let md_files = FileProcessor::expand_file_patterns(&[md_pattern])
            .expect("Failed to expand glob pattern");
        assert_eq!(md_files.len(), 1);
        assert!(md_files[0].to_string_lossy().ends_with("test2.md"));

        // Test glob pattern for all text files
        let txt_pattern = format!("{}/*.txt", temp_dir.path().display());
        let txt_files = FileProcessor::expand_file_patterns(&[txt_pattern])
            .expect("Failed to expand glob pattern");
        assert_eq!(txt_files.len(), 1);
        assert!(txt_files[0].to_string_lossy().ends_with("test1.txt"));
    }

    #[test]
    fn test_file_processor_multiple_file_patterns() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        create_test_files(&temp_dir);

        // Test multiple file patterns
        let file_patterns = vec![
            temp_dir
                .path()
                .join("test1.txt")
                .to_string_lossy()
                .to_string(),
            temp_dir
                .path()
                .join("test2.md")
                .to_string_lossy()
                .to_string(),
            temp_dir
                .path()
                .join("config.json")
                .to_string_lossy()
                .to_string(),
        ];

        let files = FileProcessor::expand_file_patterns(&file_patterns)
            .expect("Failed to expand file patterns");
        assert_eq!(files.len(), 3);
    }

    #[test]
    fn test_file_processor_nested_glob_patterns() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        create_nested_test_files(&temp_dir);

        // Test recursive glob pattern for markdown files
        let md_pattern = format!("{}/**/*.md", temp_dir.path().display());
        let md_files = FileProcessor::expand_file_patterns(&[md_pattern])
            .expect("Failed to expand recursive glob pattern");
        assert_eq!(md_files.len(), 2);
        assert!(md_files
            .iter()
            .any(|f| f.to_string_lossy().ends_with("readme.md")));
        assert!(md_files
            .iter()
            .any(|f| f.to_string_lossy().ends_with("api.md")));

        // Test recursive glob pattern for Rust files
        let rs_pattern = format!("{}/**/*.rs", temp_dir.path().display());
        let rs_files = FileProcessor::expand_file_patterns(&[rs_pattern])
            .expect("Failed to expand recursive glob pattern");
        assert_eq!(rs_files.len(), 2);
        assert!(rs_files
            .iter()
            .any(|f| f.to_string_lossy().ends_with("main.rs")));
        assert!(rs_files
            .iter()
            .any(|f| f.to_string_lossy().ends_with("lib.rs")));
    }

    #[test]
    fn test_text_chunking_algorithm() {
        let test_text = "This is a test document with multiple sentences. Each sentence should be properly handled by the chunking algorithm. The algorithm should split text at appropriate boundaries like sentence endings. It should also handle paragraph breaks properly.\n\nThis is a new paragraph that should be considered for chunking boundaries. The chunking algorithm needs to be smart about where it splits the text to maintain readability and context.";

        let chunks = FileProcessor::chunk_text(test_text, 100, 20);

        // Verify we got multiple chunks
        assert!(
            chunks.len() > 1,
            "Should produce multiple chunks for long text"
        );

        // Verify no chunk is empty
        for chunk in &chunks {
            assert!(!chunk.trim().is_empty(), "No chunk should be empty");
        }

        // Verify chunks cover the original text
        let total_chunk_content: String = chunks.join("");
        assert!(
            total_chunk_content.len() >= (test_text.len() * 8) / 10,
            "Chunks should cover most of the original text"
        );

        // Verify chunks are reasonably sized
        for chunk in &chunks {
            assert!(chunk.len() <= 150, "Chunks should not be excessively long");
        }

        // Verify that chunks maintain some context (overlap or boundary detection)
        if chunks.len() > 1 {
            // Check that consecutive chunks either have overlap or break at natural boundaries
            for i in 0..chunks.len() - 1 {
                let current_chunk = &chunks[i];
                let next_chunk = &chunks[i + 1];

                // Either there's overlap or the current chunk ends at a natural boundary
                let has_overlap =
                    next_chunk.contains(&current_chunk[current_chunk.len().saturating_sub(10)..]);
                let ends_at_boundary = current_chunk.trim_end().ends_with('.')
                    || current_chunk.trim_end().ends_with('\n')
                    || current_chunk.trim_end().ends_with('!')
                    || current_chunk.trim_end().ends_with('?');

                assert!(
                    has_overlap || ends_at_boundary,
                    "Chunks should either overlap or break at natural boundaries"
                );
            }
        }
    }

    #[test]
    fn test_text_chunking_boundary_detection() {
        let test_text = "First sentence. Second sentence.\n\nNew paragraph with more content. Another sentence in the same paragraph.";

        let chunks = FileProcessor::chunk_text(test_text, 50, 10);

        // Verify chunks respect sentence boundaries when possible
        for chunk in &chunks {
            let trimmed = chunk.trim();
            if !trimmed.is_empty() {
                // Most chunks should end with sentence-ending punctuation or be at natural boundaries
                let ends_with_punctuation =
                    trimmed.ends_with('.') || trimmed.ends_with('!') || trimmed.ends_with('?');
                let ends_with_newline = trimmed.ends_with('\n');
                let is_last_chunk = chunk == chunks.last().unwrap();

                // Allow some flexibility for boundary detection
                assert!(
                    ends_with_punctuation
                        || ends_with_newline
                        || is_last_chunk
                        || trimmed.len() < 60,
                    "Chunk should end at natural boundary: '{}'",
                    trimmed
                );
            }
        }
    }

    #[test]
    fn test_text_chunking_infinite_loop_prevention() {
        // Test with very small chunk size to ensure no infinite loops
        let test_text = "A";
        let chunks = FileProcessor::chunk_text(test_text, 1, 0);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], "A");

        // Test with empty text
        let empty_chunks = FileProcessor::chunk_text("", 100, 20);
        assert_eq!(empty_chunks.len(), 1);
        assert_eq!(empty_chunks[0], "");

        // Test with text smaller than chunk size
        let small_text = "Small text";
        let small_chunks = FileProcessor::chunk_text(small_text, 100, 20);
        assert_eq!(small_chunks.len(), 1);
        assert_eq!(small_chunks[0], small_text);
    }

    #[test]
    fn test_file_processor_binary_filtering() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");

        // Create mixed files (text and binary)
        let text_files = vec![
            ("text.txt", "This is text content"),
            ("code.rs", "fn main() { println!(\"Hello\"); }"),
        ];

        let binary_files = vec![
            ("image.jpg", vec![0xFFu8, 0xD8u8, 0xFFu8, 0xE0u8]), // JPEG header
            ("document.pdf", b"%PDF-1.4".to_vec()),              // PDF header
            ("data.bin", vec![0x00u8, 0x01u8, 0x02u8, 0x03u8]),
        ];

        // Write text files
        for (filename, content) in text_files {
            let file_path = temp_dir.path().join(filename);
            fs::write(&file_path, content).expect("Failed to write test file");
        }

        // Write binary files
        for (filename, content) in binary_files {
            let file_path = temp_dir.path().join(filename);
            fs::write(&file_path, content).expect("Failed to write test file");
        }

        // Test that only text files are processed
        let all_pattern = format!("{}/*", temp_dir.path().display());
        let all_files = FileProcessor::expand_file_patterns(&[all_pattern])
            .expect("Failed to expand glob pattern");

        let text_files: Vec<_> = all_files
            .into_iter()
            .filter(|f| FileProcessor::is_text_file(f))
            .collect();

        assert_eq!(text_files.len(), 2); // text.txt and code.rs
        assert!(text_files
            .iter()
            .any(|f| f.to_string_lossy().ends_with("text.txt")));
        assert!(text_files
            .iter()
            .any(|f| f.to_string_lossy().ends_with("code.rs")));
    }

    #[test]
    fn test_file_processor_supported_extensions() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");

        // Test various supported file extensions
        let supported_files = vec![
            ("readme.md", "# Markdown"),
            ("script.py", "print('Python')"),
            ("app.js", "console.log('JavaScript');"),
            ("style.css", "body { margin: 0; }"),
            ("config.json", "{}"),
            ("data.yaml", "key: value"),
            ("source.cpp", "#include <iostream>"),
            ("header.h", "#ifndef HEADER_H"),
            ("build.sh", "#!/bin/bash"),
            ("Dockerfile", "FROM ubuntu:20.04"),
        ];

        for (filename, content) in supported_files {
            let file_path = temp_dir.path().join(filename);
            fs::write(&file_path, content).expect("Failed to write test file");

            assert!(
                FileProcessor::is_text_file(&file_path),
                "Should detect {} as text file",
                filename
            );
        }
    }

    #[test]
    fn test_file_processor_error_handling() {
        // Test with invalid glob pattern
        let invalid_glob = "invalid[glob[pattern";
        let result = FileProcessor::expand_file_patterns(&[invalid_glob.to_string()]);
        // This should not error but return empty results due to the implementation
        assert!(
            result.is_ok(),
            "Should handle invalid glob pattern gracefully"
        );
    }

    #[test]
    fn test_file_metadata_structure() {
        // Test that file metadata is properly structured for database storage
        let file_path = "/path/to/test/file.txt";
        let chunk_index = 2;
        let total_chunks = 5;
        let content = "This is chunk content";

        // Simulate how file metadata would be stored
        struct FileChunkMetadata {
            file_path: String,
            chunk_index: usize,
            total_chunks: usize,
            content: String,
        }

        let metadata = FileChunkMetadata {
            file_path: file_path.to_string(),
            chunk_index,
            total_chunks,
            content: content.to_string(),
        };

        assert_eq!(metadata.file_path, file_path);
        assert_eq!(metadata.chunk_index, chunk_index);
        assert_eq!(metadata.total_chunks, total_chunks);
        assert_eq!(metadata.content, content);

        // Test metadata display format
        let display_format = format!(
            "[{}:{}/{}]",
            Path::new(&metadata.file_path)
                .file_name()
                .unwrap()
                .to_string_lossy(),
            metadata.chunk_index + 1, // 1-based for display
            metadata.total_chunks
        );
        assert_eq!(display_format, "[file.txt:3/5]");
    }

    #[test]
    fn test_file_embedding_workflow_simulation() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let file_paths = create_test_files(&temp_dir);

        // Simulate the complete file embedding workflow
        let file_patterns: Vec<String> = file_paths.iter().cloned().collect();
        let files = FileProcessor::expand_file_patterns(&file_patterns)
            .expect("Failed to expand file patterns");

        // Step 1: Filter text files (already done by expand_file_patterns)
        assert!(files.len() > 0, "Should have text files to process");

        // Step 2: Process each file
        for file_path in files {
            if file_path.exists() {
                let content = fs::read_to_string(&file_path).expect("Failed to read file");

                // Step 3: Chunk the content
                let chunks = FileProcessor::chunk_text(&content, 1200, 200);

                // Step 4: Simulate embedding each chunk with metadata
                for (chunk_index, chunk_content) in chunks.iter().enumerate() {
                    let metadata = (
                        file_path.to_string_lossy().to_string(),
                        chunk_index,
                        chunks.len(),
                        chunk_content.clone(),
                    );

                    // Verify metadata structure
                    assert_eq!(metadata.0, file_path.to_string_lossy().to_string());
                    assert!(metadata.1 < metadata.2);
                    assert!(!metadata.3.is_empty() || chunk_content.trim().is_empty());
                }
            }
        }
    }

    #[test]
    fn test_large_file_chunking_performance() {
        // Test chunking performance with a large file
        let large_content =
            "This is a sentence that will be repeated many times to create a large file. "
                .repeat(1000);

        let start_time = std::time::Instant::now();
        let chunks = FileProcessor::chunk_text(&large_content, 1200, 200);
        let duration = start_time.elapsed();

        // Verify chunking completed in reasonable time (should be very fast)
        assert!(
            duration.as_millis() < 1000,
            "Chunking should complete quickly"
        );

        // Verify chunks are reasonable
        assert!(
            chunks.len() > 1,
            "Large content should produce multiple chunks"
        );

        // Verify no infinite loops occurred
        let total_chunk_length: usize = chunks.iter().map(|c| c.len()).sum();
        assert!(
            total_chunk_length >= large_content.len(),
            "Total chunk content should cover original content"
        );
    }

    #[test]
    fn test_file_processing_integration() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");

        // Create a test file
        let test_file = temp_dir.path().join("test.txt");
        let test_content =
            "This is a test file with some content that should be processed correctly.";
        fs::write(&test_file, test_content).expect("Failed to write test file");

        // Test file processing
        let chunks = FileProcessor::process_file(&test_file).expect("Failed to process file");

        assert_eq!(chunks.len(), 1); // Small content should be single chunk
        assert_eq!(chunks[0], test_content);
    }

    #[test]
    fn test_file_extension_detection() {
        // Test various file extensions
        let test_cases = vec![
            ("test.txt", true),
            ("test.md", true),
            ("test.rs", true),
            ("test.py", true),
            ("test.js", true),
            ("test.json", true),
            ("test.yaml", true),
            ("test.exe", false),
            ("test.jpg", false),
            ("test.pdf", false),
            ("test.zip", false),
        ];

        for (filename, expected) in test_cases {
            let path = Path::new(filename);
            assert_eq!(
                FileProcessor::is_text_file(path),
                expected,
                "Extension detection failed for {}",
                filename
            );
        }
    }
}
