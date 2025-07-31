//! Test module for CLI functionality
//! 
//! This module contains comprehensive tests for all CLI commands,
//! with a focus on the providers command and its various options.

use crate::config::{Config, ProviderConfig};
use std::collections::HashMap;
use tempfile::TempDir;
use chrono::Utc;

/// Helper function to create a temporary config for testing
fn create_test_config() -> (Config, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let _config_path = temp_dir.path().join("config.toml");
    
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
    
    (config, temp_dir)
}

/// Helper function to create a test provider config
fn create_test_provider_config(endpoint: &str) -> ProviderConfig {
    ProviderConfig {
        endpoint: endpoint.to_string(),
        api_key: Some("test-api-key".to_string()),
        models: vec!["test-model-1".to_string(), "test-model-2".to_string()],
        models_path: "/models".to_string(),
        chat_path: "/chat/completions".to_string(),
        headers: HashMap::new(),
        token_url: None,
        cached_token: None,
    }
}

/// Helper function to create a config with test providers
fn create_config_with_providers() -> Config {
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
    
    // Add test providers
    config.providers.insert(
        "openai".to_string(),
        create_test_provider_config("https://api.openai.com")
    );
    
    config.providers.insert(
        "anthropic".to_string(),
        create_test_provider_config("https://api.anthropic.com")
    );
    
    config.default_provider = Some("openai".to_string());
    
    config
}

#[cfg(test)]
mod provider_tests {
    use super::*;

    #[test]
    fn test_provider_add_basic() {
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

        // Test adding a basic provider
        let result = config.add_provider(
            "test-provider".to_string(),
            "https://api.test.com".to_string()
        );

        assert!(result.is_ok());
        assert!(config.has_provider("test-provider"));
        
        let provider = config.get_provider("test-provider").unwrap();
        assert_eq!(provider.endpoint, "https://api.test.com");
        assert_eq!(provider.models_path, "/models");
        assert_eq!(provider.chat_path, "/chat/completions");
        assert!(provider.api_key.is_none());
        assert!(provider.headers.is_empty());
        
        // Should be set as default since it's the first provider
        assert_eq!(config.default_provider, Some("test-provider".to_string()));
    }

    #[test]
    fn test_provider_add_with_custom_paths() {
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

        // Test adding a provider with custom paths
        let result = config.add_provider_with_paths(
            "custom-provider".to_string(),
            "https://api.custom.com".to_string(),
            Some("/v1/models".to_string()),
            Some("/v1/completions".to_string())
        );

        assert!(result.is_ok());
        assert!(config.has_provider("custom-provider"));
        
        let provider = config.get_provider("custom-provider").unwrap();
        assert_eq!(provider.endpoint, "https://api.custom.com");
        assert_eq!(provider.models_path, "/v1/models");
        assert_eq!(provider.chat_path, "/v1/completions");
    }

    #[test]
    fn test_provider_add_second_provider_doesnt_change_default() {
        let mut config = create_config_with_providers();
        let original_default = config.default_provider.clone();

        // Add another provider
        let result = config.add_provider(
            "new-provider".to_string(),
            "https://api.new.com".to_string()
        );

        assert!(result.is_ok());
        assert!(config.has_provider("new-provider"));
        // Default should remain unchanged
        assert_eq!(config.default_provider, original_default);
    }

    #[test]
    fn test_provider_update_existing() {
        let mut config = create_config_with_providers();

        // Update existing provider
        let result = config.add_provider(
            "openai".to_string(),
            "https://api.openai.com/v2".to_string()
        );

        assert!(result.is_ok());
        let provider = config.get_provider("openai").unwrap();
        assert_eq!(provider.endpoint, "https://api.openai.com/v2");
    }

    #[test]
    fn test_provider_remove_existing() {
        let mut config = create_config_with_providers();

        // Remove existing provider
        assert!(config.has_provider("anthropic"));
        config.providers.remove("anthropic");
        assert!(!config.has_provider("anthropic"));
    }

    #[test]
    fn test_provider_remove_nonexistent() {
        let config = create_config_with_providers();

        // Try to get non-existent provider
        let result = config.get_provider("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_provider_list_empty() {
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

        assert!(config.providers.is_empty());
    }

    #[test]
    fn test_provider_list_with_providers() {
        let config = create_config_with_providers();

        assert_eq!(config.providers.len(), 2);
        assert!(config.has_provider("openai"));
        assert!(config.has_provider("anthropic"));
    }

    #[test]
    fn test_provider_api_key_management() {
        let mut config = create_config_with_providers();

        // Test setting API key
        let result = config.set_api_key(
            "openai".to_string(),
            "new-api-key".to_string()
        );
        assert!(result.is_ok());

        let provider = config.get_provider("openai").unwrap();
        assert_eq!(provider.api_key, Some("new-api-key".to_string()));

        // Test setting API key for non-existent provider
        let result = config.set_api_key(
            "nonexistent".to_string(),
            "key".to_string()
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_provider_headers_management() {
        let mut config = create_config_with_providers();

        // Test adding header
        let result = config.add_header(
            "openai".to_string(),
            "X-Custom-Header".to_string(),
            "custom-value".to_string()
        );
        assert!(result.is_ok());

        let headers = config.list_headers("openai").unwrap();
        assert_eq!(headers.get("X-Custom-Header"), Some(&"custom-value".to_string()));

        // Test removing header
        let result = config.remove_header(
            "openai".to_string(),
            "X-Custom-Header".to_string()
        );
        assert!(result.is_ok());

        let headers = config.list_headers("openai").unwrap();
        assert!(!headers.contains_key("X-Custom-Header"));

        // Test removing non-existent header
        let result = config.remove_header(
            "openai".to_string(),
            "Non-Existent".to_string()
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));

        // Test headers for non-existent provider
        let result = config.add_header(
            "nonexistent".to_string(),
            "header".to_string(),
            "value".to_string()
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_provider_token_url_management() {
        let mut config = create_config_with_providers();

        // Test setting token URL
        let result = config.set_token_url(
            "openai".to_string(),
            "https://auth.openai.com/token".to_string()
        );
        assert!(result.is_ok());

        let token_url = config.get_token_url("openai");
        assert_eq!(token_url, Some(&"https://auth.openai.com/token".to_string()));

        // Test setting token URL for non-existent provider
        let result = config.set_token_url(
            "nonexistent".to_string(),
            "https://example.com".to_string()
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_provider_cached_token_management() {
        let mut config = create_config_with_providers();
        let expires_at = Utc::now() + chrono::Duration::hours(1);

        // Test setting cached token
        let result = config.set_cached_token(
            "openai".to_string(),
            "cached-token-123".to_string(),
            expires_at
        );
        assert!(result.is_ok());

        let cached_token = config.get_cached_token("openai");
        assert!(cached_token.is_some());
        assert_eq!(cached_token.unwrap().token, "cached-token-123");
        assert_eq!(cached_token.unwrap().expires_at, expires_at);

        // Test setting cached token for non-existent provider
        let result = config.set_cached_token(
            "nonexistent".to_string(),
            "token".to_string(),
            expires_at
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_provider_token_url_clears_cached_token() {
        let mut config = create_config_with_providers();
        let expires_at = Utc::now() + chrono::Duration::hours(1);

        // Set a cached token first
        config.set_cached_token(
            "openai".to_string(),
            "cached-token".to_string(),
            expires_at
        ).unwrap();

        assert!(config.get_cached_token("openai").is_some());

        // Setting token URL should clear cached token
        config.set_token_url(
            "openai".to_string(),
            "https://auth.openai.com/token".to_string()
        ).unwrap();

        assert!(config.get_cached_token("openai").is_none());
    }
}

#[cfg(test)]
mod provider_command_tests {
    use crate::cli::{ProviderCommands, HeaderCommands};

    /// Mock test for provider add command
    /// Note: This would require refactoring the actual command handlers to accept
    /// a config parameter instead of loading from file system
    #[test]
    fn test_provider_add_command_structure() {
        // Test the command structure itself
        let command = ProviderCommands::Add {
            name: "test-provider".to_string(),
            url: "https://api.test.com".to_string(),
            models_path: Some("/v1/models".to_string()),
            chat_path: Some("/v1/chat".to_string()),
        };

        match command {
            ProviderCommands::Add { name, url, models_path, chat_path } => {
                assert_eq!(name, "test-provider");
                assert_eq!(url, "https://api.test.com");
                assert_eq!(models_path, Some("/v1/models".to_string()));
                assert_eq!(chat_path, Some("/v1/chat".to_string()));
            }
            _ => panic!("Expected Add command"),
        }
    }

    #[test]
    fn test_provider_update_command_structure() {
        let command = ProviderCommands::Update {
            name: "existing-provider".to_string(),
            url: "https://api.updated.com".to_string(),
        };

        match command {
            ProviderCommands::Update { name, url } => {
                assert_eq!(name, "existing-provider");
                assert_eq!(url, "https://api.updated.com");
            }
            _ => panic!("Expected Update command"),
        }
    }

    #[test]
    fn test_provider_remove_command_structure() {
        let command = ProviderCommands::Remove {
            name: "provider-to-remove".to_string(),
        };

        match command {
            ProviderCommands::Remove { name } => {
                assert_eq!(name, "provider-to-remove");
            }
            _ => panic!("Expected Remove command"),
        }
    }

    #[test]
    fn test_provider_list_command_structure() {
        let command = ProviderCommands::List;

        match command {
            ProviderCommands::List => {
                // Command structure is correct
            }
            _ => panic!("Expected List command"),
        }
    }

    #[test]
    fn test_provider_models_command_structure() {
        let command = ProviderCommands::Models {
            name: "test-provider".to_string(),
            refresh: true,
        };

        match command {
            ProviderCommands::Models { name, refresh } => {
                assert_eq!(name, "test-provider");
                assert_eq!(refresh, true);
            }
            _ => panic!("Expected Models command"),
        }
    }

    #[test]
    fn test_provider_headers_command_structure() {
        let add_command = ProviderCommands::Headers {
            provider: "test-provider".to_string(),
            command: HeaderCommands::Add {
                name: "X-API-Version".to_string(),
                value: "v1".to_string(),
            },
        };

        match add_command {
            ProviderCommands::Headers { provider, command } => {
                assert_eq!(provider, "test-provider");
                match command {
                    HeaderCommands::Add { name, value } => {
                        assert_eq!(name, "X-API-Version");
                        assert_eq!(value, "v1");
                    }
                    _ => panic!("Expected Add header command"),
                }
            }
            _ => panic!("Expected Headers command"),
        }

        let delete_command = ProviderCommands::Headers {
            provider: "test-provider".to_string(),
            command: HeaderCommands::Delete {
                name: "X-API-Version".to_string(),
            },
        };

        match delete_command {
            ProviderCommands::Headers { provider, command } => {
                assert_eq!(provider, "test-provider");
                match command {
                    HeaderCommands::Delete { name } => {
                        assert_eq!(name, "X-API-Version");
                    }
                    _ => panic!("Expected Delete header command"),
                }
            }
            _ => panic!("Expected Headers command"),
        }

        let list_command = ProviderCommands::Headers {
            provider: "test-provider".to_string(),
            command: HeaderCommands::List,
        };

        match list_command {
            ProviderCommands::Headers { provider, command } => {
                assert_eq!(provider, "test-provider");
                match command {
                    HeaderCommands::List => {
                        // Command structure is correct
                    }
                    _ => panic!("Expected List header command"),
                }
            }
            _ => panic!("Expected Headers command"),
        }
    }

    #[test]
    fn test_provider_token_url_command_structure() {
        let command = ProviderCommands::TokenUrl {
            provider: "test-provider".to_string(),
            url: "https://auth.test.com/token".to_string(),
        };

        match command {
            ProviderCommands::TokenUrl { provider, url } => {
                assert_eq!(provider, "test-provider");
                assert_eq!(url, "https://auth.test.com/token");
            }
            _ => panic!("Expected TokenUrl command"),
        }
    }
}

#[cfg(test)]
mod provider_edge_cases {
    use super::*;

    #[test]
    fn test_provider_name_validation() {
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

        // Test empty provider name
        let result = config.add_provider("".to_string(), "https://api.test.com".to_string());
        assert!(result.is_ok()); // Config doesn't validate empty names, but CLI should

        // Test provider name with special characters
        let result = config.add_provider("test-provider_123".to_string(), "https://api.test.com".to_string());
        assert!(result.is_ok());

        // Test provider name with spaces
        let result = config.add_provider("test provider".to_string(), "https://api.test.com".to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn test_provider_url_validation() {
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

        // Test various URL formats
        let urls = vec![
            "https://api.test.com",
            "http://localhost:8080",
            "https://api.test.com/",
            "https://api.test.com/v1",
            "invalid-url",
            "",
        ];

        for url in urls {
            let result = config.add_provider(format!("provider-{}", url.len()), url.to_string());
            assert!(result.is_ok()); // Config doesn't validate URLs, but should be validated elsewhere
        }
    }

    #[test]
    fn test_provider_paths_validation() {
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

        // Test various path formats
        let test_cases = vec![
            (Some("/models".to_string()), Some("/chat/completions".to_string())),
            (Some("/v1/models".to_string()), Some("/v1/chat/completions".to_string())),
            (Some("models".to_string()), Some("chat".to_string())), // Without leading slash
            (Some("".to_string()), Some("".to_string())), // Empty paths
            (None, None), // Default paths
        ];

        for (i, (models_path, chat_path)) in test_cases.into_iter().enumerate() {
            let result = config.add_provider_with_paths(
                format!("provider-{}", i),
                "https://api.test.com".to_string(),
                models_path.clone(),
                chat_path.clone(),
            );
            assert!(result.is_ok());

            let provider = config.get_provider(&format!("provider-{}", i)).unwrap();
            assert_eq!(provider.models_path, models_path.unwrap_or_else(|| "/models".to_string()));
            assert_eq!(provider.chat_path, chat_path.unwrap_or_else(|| "/chat/completions".to_string()));
        }
    }

    #[test]
    fn test_provider_duplicate_names() {
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

        // Add provider
        let result = config.add_provider("duplicate".to_string(), "https://api1.test.com".to_string());
        assert!(result.is_ok());

        let provider1 = config.get_provider("duplicate").unwrap();
        assert_eq!(provider1.endpoint, "https://api1.test.com");

        // Add provider with same name (should overwrite)
        let result = config.add_provider("duplicate".to_string(), "https://api2.test.com".to_string());
        assert!(result.is_ok());

        let provider2 = config.get_provider("duplicate").unwrap();
        assert_eq!(provider2.endpoint, "https://api2.test.com");
    }

    #[test]
    fn test_provider_case_sensitivity() {
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

        // Add providers with different cases
        config.add_provider("OpenAI".to_string(), "https://api.openai.com".to_string()).unwrap();
        config.add_provider("openai".to_string(), "https://api.openai.com/v2".to_string()).unwrap();
        config.add_provider("OPENAI".to_string(), "https://api.openai.com/v3".to_string()).unwrap();

        // All should be treated as different providers
        assert!(config.has_provider("OpenAI"));
        assert!(config.has_provider("openai"));
        assert!(config.has_provider("OPENAI"));
        assert_eq!(config.providers.len(), 3);
    }

    #[test]
    fn test_provider_header_edge_cases() {
        let mut config = create_config_with_providers();

        // Test header with empty name
        let result = config.add_header("openai".to_string(), "".to_string(), "value".to_string());
        assert!(result.is_ok()); // Config allows empty header names

        // Test header with empty value
        let result = config.add_header("openai".to_string(), "X-Empty".to_string(), "".to_string());
        assert!(result.is_ok());

        // Test header with special characters
        let result = config.add_header("openai".to_string(), "X-Special-Chars!@#".to_string(), "value!@#$%".to_string());
        assert!(result.is_ok());

        // Test overwriting existing header
        config.add_header("openai".to_string(), "X-Test".to_string(), "original".to_string()).unwrap();
        config.add_header("openai".to_string(), "X-Test".to_string(), "updated".to_string()).unwrap();
        
        let headers = config.list_headers("openai").unwrap();
        assert_eq!(headers.get("X-Test"), Some(&"updated".to_string()));
    }
}

#[cfg(test)]
mod provider_integration_tests {
    use super::*;

    #[test]
    fn test_provider_workflow_complete() {
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

        // 1. Add provider
        config.add_provider_with_paths(
            "test-provider".to_string(),
            "https://api.test.com".to_string(),
            Some("/v1/models".to_string()),
            Some("/v1/chat".to_string()),
        ).unwrap();

        assert!(config.has_provider("test-provider"));
        assert_eq!(config.default_provider, Some("test-provider".to_string()));

        // 2. Set API key
        config.set_api_key("test-provider".to_string(), "secret-key".to_string()).unwrap();
        let provider = config.get_provider("test-provider").unwrap();
        assert_eq!(provider.api_key, Some("secret-key".to_string()));

        // 3. Add headers
        config.add_header("test-provider".to_string(), "X-API-Version".to_string(), "v1".to_string()).unwrap();
        config.add_header("test-provider".to_string(), "X-Client".to_string(), "lc-cli".to_string()).unwrap();
        
        let headers = config.list_headers("test-provider").unwrap();
        assert_eq!(headers.len(), 2);
        assert_eq!(headers.get("X-API-Version"), Some(&"v1".to_string()));
        assert_eq!(headers.get("X-Client"), Some(&"lc-cli".to_string()));

        // 4. Set token URL
        config.set_token_url("test-provider".to_string(), "https://auth.test.com/token".to_string()).unwrap();
        assert_eq!(config.get_token_url("test-provider"), Some(&"https://auth.test.com/token".to_string()));

        // 5. Set cached token
        let expires_at = Utc::now() + chrono::Duration::hours(1);
        config.set_cached_token("test-provider".to_string(), "cached-token".to_string(), expires_at).unwrap();
        
        let cached_token = config.get_cached_token("test-provider").unwrap();
        assert_eq!(cached_token.token, "cached-token");

        // 6. Update provider URL (note: add_provider creates a new config, so API key is lost)
        config.add_provider("test-provider".to_string(), "https://api.test.com/v2".to_string()).unwrap();
        let updated_provider = config.get_provider("test-provider").unwrap();
        assert_eq!(updated_provider.endpoint, "https://api.test.com/v2");
        // API key is lost when updating via add_provider (this is expected behavior)
        assert_eq!(updated_provider.api_key, None);
        
        // Re-set the API key after update
        config.set_api_key("test-provider".to_string(), "secret-key-updated".to_string()).unwrap();
        let provider_with_key = config.get_provider("test-provider").unwrap();
        assert_eq!(provider_with_key.api_key, Some("secret-key-updated".to_string()));

        // 7. Re-add headers after provider update (since they were lost)
        config.add_header("test-provider".to_string(), "X-API-Version".to_string(), "v1".to_string()).unwrap();
        config.add_header("test-provider".to_string(), "X-Client".to_string(), "lc-cli".to_string()).unwrap();
        
        // Now remove one header
        config.remove_header("test-provider".to_string(), "X-API-Version".to_string()).unwrap();
        let headers = config.list_headers("test-provider").unwrap();
        assert_eq!(headers.len(), 1);
        assert!(!headers.contains_key("X-API-Version"));
        assert!(headers.contains_key("X-Client"));

        // 8. Remove provider
        config.providers.remove("test-provider");
        assert!(!config.has_provider("test-provider"));
    }

    #[test]
    fn test_multiple_providers_workflow() {
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
        let providers = vec![
            ("openai", "https://api.openai.com"),
            ("anthropic", "https://api.anthropic.com"),
            ("cohere", "https://api.cohere.ai"),
        ];

        for (name, url) in providers {
            config.add_provider(name.to_string(), url.to_string()).unwrap();
            config.set_api_key(name.to_string(), format!("{}-api-key", name)).unwrap();
        }

        // Verify all providers exist
        assert_eq!(config.providers.len(), 3);
        assert!(config.has_provider("openai"));
        assert!(config.has_provider("anthropic"));
        assert!(config.has_provider("cohere"));

        // First provider should be default
        assert_eq!(config.default_provider, Some("openai".to_string()));

        // Each provider should have its API key
        for (name, _) in &[("openai", ""), ("anthropic", ""), ("cohere", "")] {
            let provider = config.get_provider(name).unwrap();
            assert_eq!(provider.api_key, Some(format!("{}-api-key", name)));
        }

        // Add different headers to each provider
        config.add_header("openai".to_string(), "X-OpenAI-Version".to_string(), "2023-12-01".to_string()).unwrap();
        config.add_header("anthropic".to_string(), "X-Anthropic-Version".to_string(), "2023-06-01".to_string()).unwrap();
        config.add_header("cohere".to_string(), "X-Cohere-Version".to_string(), "2023-08-01".to_string()).unwrap();

        // Verify headers are isolated per provider
        let openai_headers = config.list_headers("openai").unwrap();
        let anthropic_headers = config.list_headers("anthropic").unwrap();
        let cohere_headers = config.list_headers("cohere").unwrap();

        assert!(openai_headers.contains_key("X-OpenAI-Version"));
        assert!(!openai_headers.contains_key("X-Anthropic-Version"));
        assert!(!openai_headers.contains_key("X-Cohere-Version"));

        assert!(anthropic_headers.contains_key("X-Anthropic-Version"));
        assert!(!anthropic_headers.contains_key("X-OpenAI-Version"));
        assert!(!anthropic_headers.contains_key("X-Cohere-Version"));

        assert!(cohere_headers.contains_key("X-Cohere-Version"));
        assert!(!cohere_headers.contains_key("X-OpenAI-Version"));
        assert!(!cohere_headers.contains_key("X-Anthropic-Version"));
    }

    #[test]
    fn test_provider_error_scenarios() {
        let mut config = create_config_with_providers();

        // Test operations on non-existent provider
        let nonexistent_provider = "nonexistent".to_string();
        
        // Test setting API key for non-existent provider
        let result = config.set_api_key(nonexistent_provider.clone(), "key".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));

        // Test adding header for non-existent provider
        let result = config.add_header(nonexistent_provider.clone(), "header".to_string(), "value".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));

        // Test removing header for non-existent provider
        let result = config.remove_header(nonexistent_provider.clone(), "header".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));

        // Test listing headers for non-existent provider
        let result = config.list_headers(&nonexistent_provider);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));

        // Test setting token URL for non-existent provider
        let result = config.set_token_url(nonexistent_provider.clone(), "https://example.com".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));

        // Test setting cached token for non-existent provider
        let expires_at = Utc::now() + chrono::Duration::hours(1);
        let result = config.set_cached_token(nonexistent_provider.clone(), "token".to_string(), expires_at);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_provider_concurrent_operations() {
        let mut config = create_config_with_providers();

        // Test multiple operations on the same provider
        let provider_name = "openai".to_string();

        // Set API key
        config.set_api_key(provider_name.clone(), "api-key-1".to_string()).unwrap();
        
        // Add multiple headers
        config.add_header(provider_name.clone(), "X-Header-1".to_string(), "value-1".to_string()).unwrap();
        config.add_header(provider_name.clone(), "X-Header-2".to_string(), "value-2".to_string()).unwrap();
        config.add_header(provider_name.clone(), "X-Header-3".to_string(), "value-3".to_string()).unwrap();

        // Set token URL
        config.set_token_url(provider_name.clone(), "https://auth.openai.com/token".to_string()).unwrap();

        // Verify all operations succeeded
        let provider = config.get_provider(&provider_name).unwrap();
        assert_eq!(provider.api_key, Some("api-key-1".to_string()));
        assert_eq!(provider.token_url, Some("https://auth.openai.com/token".to_string()));

        let headers = config.list_headers(&provider_name).unwrap();
        assert_eq!(headers.len(), 3);
        assert!(headers.contains_key("X-Header-1"));
        assert!(headers.contains_key("X-Header-2"));
        assert!(headers.contains_key("X-Header-3"));

        // Update API key
        config.set_api_key(provider_name.clone(), "api-key-2".to_string()).unwrap();
        let provider = config.get_provider(&provider_name).unwrap();
        assert_eq!(provider.api_key, Some("api-key-2".to_string()));

        // Remove some headers
        config.remove_header(provider_name.clone(), "X-Header-2".to_string()).unwrap();
        let headers = config.list_headers(&provider_name).unwrap();
        assert_eq!(headers.len(), 2);
        assert!(!headers.contains_key("X-Header-2"));
    }
}

#[cfg(test)]
mod provider_alias_integration_tests {
    use super::*;

    #[test]
    fn test_provider_alias_workflow() {
        let mut config = create_config_with_providers();

        // Test adding alias that references existing provider
        let result = config.add_alias("gpt4".to_string(), "openai:gpt-4".to_string());
        assert!(result.is_ok());

        let alias = config.get_alias("gpt4");
        assert_eq!(alias, Some(&"openai:gpt-4".to_string()));

        // Test adding alias that references non-existent provider
        let result = config.add_alias("invalid".to_string(), "nonexistent:model".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));

        // Test adding alias with invalid format
        let result = config.add_alias("invalid-format".to_string(), "just-a-model".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("format"));

        // Test removing alias
        let result = config.remove_alias("gpt4".to_string());
        assert!(result.is_ok());
        assert!(config.get_alias("gpt4").is_none());

        // Test removing non-existent alias
        let result = config.remove_alias("nonexistent".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }
}

#[cfg(test)]
mod provider_config_persistence_tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_config_save_and_load() {
        let (mut config, temp_dir) = create_test_config();
        let config_path = temp_dir.path().join("config.toml");

        // Add a provider
        config.add_provider_with_paths(
            "test-provider".to_string(),
            "https://api.test.com".to_string(),
            Some("/v1/models".to_string()),
            Some("/v1/chat".to_string()),
        ).unwrap();

        // Set API key and headers
        config.set_api_key("test-provider".to_string(), "secret-key".to_string()).unwrap();
        config.add_header("test-provider".to_string(), "X-Custom".to_string(), "value".to_string()).unwrap();

        // Save config to file
        let toml_content = toml::to_string_pretty(&config).unwrap();
        fs::write(&config_path, &toml_content).unwrap();

        // Load config from file
        let loaded_content = fs::read_to_string(&config_path).unwrap();
        let loaded_config: Config = toml::from_str(&loaded_content).unwrap();

        // Verify loaded config matches original
        assert!(loaded_config.has_provider("test-provider"));
        let provider = loaded_config.get_provider("test-provider").unwrap();
        assert_eq!(provider.endpoint, "https://api.test.com");
        assert_eq!(provider.models_path, "/v1/models");
        assert_eq!(provider.chat_path, "/v1/chat");
        assert_eq!(provider.api_key, Some("secret-key".to_string()));
        assert_eq!(provider.headers.get("X-Custom"), Some(&"value".to_string()));
    }

    #[test]
    fn test_config_with_cached_token_serialization() {
        let (mut config, temp_dir) = create_test_config();
        let config_path = temp_dir.path().join("config.toml");

        // Add provider with cached token
        config.add_provider("test-provider".to_string(), "https://api.test.com".to_string()).unwrap();
        
        let expires_at = Utc::now() + chrono::Duration::hours(1);
        config.set_cached_token("test-provider".to_string(), "cached-token-123".to_string(), expires_at).unwrap();

        // Save and reload
        let toml_content = toml::to_string_pretty(&config).unwrap();
        fs::write(&config_path, &toml_content).unwrap();

        let loaded_content = fs::read_to_string(&config_path).unwrap();
        let loaded_config: Config = toml::from_str(&loaded_content).unwrap();

        // Verify cached token is preserved
        let cached_token = loaded_config.get_cached_token("test-provider");
        assert!(cached_token.is_some());
        assert_eq!(cached_token.unwrap().token, "cached-token-123");
        assert_eq!(cached_token.unwrap().expires_at, expires_at);
    }
}