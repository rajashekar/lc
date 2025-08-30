//! Integration tests for provider commands
//!
//! This module contains comprehensive integration tests for all provider-related
//! CLI commands, testing them as a user would interact with the CLI.

mod common;

use chrono::Utc;
use common::{assertions, create_config_with_providers, test_data};
use lc::config::Config;
use lc::test_utils::get_test_provider_name;
use serial_test::serial;
use std::collections::HashMap;

// Import test utilities
use lc::test_with_cleanup;

#[cfg(test)]
mod provider_add_tests {
    use super::*;

    #[test]
    #[serial]
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
            stream: None,
        };

        // Test adding a basic provider
        let result = config.add_provider(
            "test-provider".to_string(),
            "https://api.test.com".to_string(),
        );

        assert!(result.is_ok());
        assertions::assert_provider_exists(&config, "test-provider");
        assertions::assert_provider_endpoint(&config, "test-provider", "https://api.test.com");
        assertions::assert_default_provider(&config, "test-provider");

        let provider = config.get_provider("test-provider").unwrap();
        assert_eq!(provider.models_path, "/models");
        assert_eq!(provider.chat_path, "/chat/completions");
        assert!(provider.api_key.is_none());
        assert!(provider.headers.is_empty());
    }

    #[test]
    #[serial]
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
            stream: None,
        };

        // Test adding a provider with custom paths
        let result = config.add_provider_with_paths(
            "test-custom-provider".to_string(),
            "https://api.custom.com".to_string(),
            Some("/v1/models".to_string()),
            Some("/v1/completions".to_string()),
        );

        assert!(result.is_ok());
        assertions::assert_provider_exists(&config, "test-custom-provider");

        let provider = config.get_provider("test-custom-provider").unwrap();
        assert_eq!(provider.endpoint, "https://api.custom.com");
        assert_eq!(provider.models_path, "/v1/models");
        assert_eq!(provider.chat_path, "/v1/completions");
    }

    #[test]
    #[serial]
    fn test_provider_add_multiple_providers() {
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

        // Add multiple providers from test data
        for (name, url) in test_data::TEST_PROVIDERS {
            let result = config.add_provider(name.to_string(), url.to_string());
            assert!(result.is_ok(), "Failed to add provider {}", name);
            assertions::assert_provider_exists(&config, name);
            assertions::assert_provider_endpoint(&config, name, url);
        }

        // First provider should be default
        assertions::assert_default_provider(&config, test_data::TEST_PROVIDERS[0].0);

        // Verify total count
        assert_eq!(config.providers.len(), test_data::TEST_PROVIDERS.len());
    }

    #[test]
    #[serial]
    fn test_provider_add_second_provider_doesnt_change_default() {
        let mut config = create_config_with_providers();
        let original_default = config.default_provider.clone();

        // Add another provider
        let result = config.add_provider(
            "test-new-provider".to_string(),
            "https://api.new.com".to_string(),
        );

        assert!(result.is_ok());
        assertions::assert_provider_exists(&config, "test-new-provider");
        // Default should remain unchanged
        assert_eq!(config.default_provider, original_default);
    }
}

#[cfg(test)]
mod provider_update_tests {
    use super::*;

    #[test]
    #[serial]
    fn test_provider_update_existing() {
        let mut config = create_config_with_providers();

        // Update existing provider
        let openai_name = get_test_provider_name("openai");
        let result =
            config.add_provider(openai_name.clone(), "https://api.openai.com/v2".to_string());

        assert!(result.is_ok());
        assertions::assert_provider_endpoint(&config, &openai_name, "https://api.openai.com/v2");
    }

    #[test]
    #[serial]
    fn test_provider_update_preserves_provider_count() {
        test_with_cleanup!({
            let mut config = create_config_with_providers();
            let original_count = config.providers.len();
            let openai_name = get_test_provider_name("openai");

            // Update existing provider
            let result = config.add_provider(openai_name, "https://api.openai.com/v2".to_string());

            assert!(result.is_ok());
            assert_eq!(config.providers.len(), original_count);
        });
    }
}

#[cfg(test)]
mod provider_remove_tests {
    use super::*;

    #[test]
    #[serial]
    fn test_provider_remove_existing() {
        test_with_cleanup!({
            let mut config = create_config_with_providers();
            let anthropic_name = get_test_provider_name("anthropic");

            // Remove existing provider
            assertions::assert_provider_exists(&config, &anthropic_name);
            config.providers.remove(&anthropic_name);
            assertions::assert_provider_not_exists(&config, &anthropic_name);
        });
    }

    #[test]
    #[serial]
    fn test_provider_remove_nonexistent() {
        let config = create_config_with_providers();

        // Try to get non-existent provider
        let result = config.get_provider("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    #[serial]
    fn test_provider_remove_all() {
        let mut config = create_config_with_providers();
        let provider_names: Vec<String> = config.providers.keys().cloned().collect();

        // Remove all providers
        for name in &provider_names {
            config.providers.remove(name);
            assertions::assert_provider_not_exists(&config, name);
        }

        assert!(config.providers.is_empty());
    }
}

#[cfg(test)]
mod provider_list_tests {
    use super::*;

    #[test]
    #[serial]
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
            stream: None,
        };

        assert!(config.providers.is_empty());
    }

    #[test]
    #[serial]
    fn test_provider_list_with_providers() {
        test_with_cleanup!({
            let config = create_config_with_providers();
            let openai_name = get_test_provider_name("openai");
            let anthropic_name = get_test_provider_name("anthropic");

            assert_eq!(config.providers.len(), 2);
            assertions::assert_provider_exists(&config, &openai_name);
            assertions::assert_provider_exists(&config, &anthropic_name);
        });
    }

    #[test]
    #[serial]
    fn test_provider_list_ordering() {
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

        // Add providers in specific order
        let providers = vec![
            ("test-zebra", "https://api.zebra.com"),
            ("test-alpha", "https://api.alpha.com"),
            ("test-beta", "https://api.beta.com"),
        ];

        for (name, url) in providers {
            config
                .add_provider(name.to_string(), url.to_string())
                .unwrap();
        }

        // Verify all providers exist
        assertions::assert_provider_exists(&config, "test-zebra");
        assertions::assert_provider_exists(&config, "test-alpha");
        assertions::assert_provider_exists(&config, "test-beta");

        // Get sorted list for display
        let mut sorted_providers: Vec<_> = config.providers.iter().collect();
        sorted_providers.sort_by(|a, b| a.0.cmp(b.0));

        let sorted_names: Vec<&String> = sorted_providers.iter().map(|(name, _)| *name).collect();
        assert_eq!(sorted_names, vec!["test-alpha", "test-beta", "test-zebra"]);
    }
}

#[cfg(test)]
mod provider_api_key_tests {
    use super::*;

    #[test]
    #[serial]
    fn test_provider_api_key_management() {
        test_with_cleanup!({
            let mut config = create_config_with_providers();
            let openai_name = get_test_provider_name("openai");

            // Test setting API key
            let result = config.set_api_key(openai_name.clone(), "new-api-key".to_string());
            assert!(result.is_ok());
            assertions::assert_provider_api_key(&config, &openai_name, "new-api-key");

            // Test setting API key for non-existent provider
            let result = config.set_api_key("nonexistent".to_string(), "key".to_string());
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("not found"));
        });
    }

    #[test]
    #[serial]
    #[serial]
    fn test_provider_api_key_update() {
        test_with_cleanup!({
            let mut config = create_config_with_providers();
            let openai_name = get_test_provider_name("openai");

            // Set initial API key
            config
                .set_api_key(openai_name.clone(), "key1".to_string())
                .unwrap();
            assertions::assert_provider_api_key(&config, &openai_name, "key1");

            // Update API key
            config
                .set_api_key(openai_name.clone(), "key2".to_string())
                .unwrap();
            assertions::assert_provider_api_key(&config, &openai_name, "key2");
        });
    }

    #[test]
    #[serial]
    fn test_provider_api_key_removal() {
        test_with_cleanup!({
            let mut config = create_config_with_providers();
            let openai_name = get_test_provider_name("openai");

            // Set API key
            config
                .set_api_key(openai_name.clone(), "test-key".to_string())
                .unwrap();
            assertions::assert_provider_has_api_key(&config, &openai_name);

            // Remove API key by setting to None
            if let Some(provider_config) = config.providers.get_mut(&openai_name) {
                provider_config.api_key = None;
            }

            let provider = config.get_provider(&openai_name).unwrap();
            assert!(provider.api_key.is_none());
        });
    }
}

#[cfg(test)]
mod provider_headers_tests {
    use super::*;

    #[test]
    #[serial]
    fn test_provider_headers_management() {
        test_with_cleanup!({
            let mut config = create_config_with_providers();
            let openai_name = get_test_provider_name("openai");

            // Test adding header
            let result = config.add_header(
                openai_name.clone(),
                "X-Custom-Header".to_string(),
                "custom-value".to_string(),
            );
            assert!(result.is_ok());
            assertions::assert_header_exists(
                &config,
                &openai_name,
                "X-Custom-Header",
                "custom-value",
            );

            // Test removing header
            let result = config.remove_header(openai_name.clone(), "X-Custom-Header".to_string());
            assert!(result.is_ok());

            let headers = config.list_headers(&openai_name).unwrap();
            assert!(!headers.contains_key("X-Custom-Header"));

            // Test removing non-existent header
            let result = config.remove_header(openai_name.clone(), "Non-Existent".to_string());
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("not found"));

            // Test headers for non-existent provider
            let result = config.add_header(
                "nonexistent".to_string(),
                "header".to_string(),
                "value".to_string(),
            );
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("not found"));
        });
    }

    #[test]
    #[serial]
    fn test_provider_multiple_headers() {
        test_with_cleanup!({
            let mut config = create_config_with_providers();
            let openai_name = get_test_provider_name("openai");

            // Add multiple headers from test data
            for (header_name, header_value) in test_data::TEST_HEADERS {
                let result = config.add_header(
                    openai_name.clone(),
                    header_name.to_string(),
                    header_value.to_string(),
                );
                assert!(result.is_ok());
                assertions::assert_header_exists(&config, &openai_name, header_name, header_value);
            }

            assertions::assert_header_count(&config, &openai_name, test_data::TEST_HEADERS.len());
        });
    }

    #[test]
    #[serial]
    fn test_provider_headers_isolation() {
        test_with_cleanup!({
            let mut config = create_config_with_providers();
            let openai_name = get_test_provider_name("openai");
            let anthropic_name = get_test_provider_name("anthropic");

            // Add headers to different providers
            config
                .add_header(
                    openai_name.clone(),
                    "X-OpenAI-Version".to_string(),
                    "2023-12-01".to_string(),
                )
                .unwrap();
            config
                .add_header(
                    anthropic_name.clone(),
                    "X-Anthropic-Version".to_string(),
                    "2023-06-01".to_string(),
                )
                .unwrap();

            // Verify headers are isolated per provider
            let openai_headers = config.list_headers(&openai_name).unwrap();
            let anthropic_headers = config.list_headers(&anthropic_name).unwrap();

            assert!(openai_headers.contains_key("X-OpenAI-Version"));
            assert!(!openai_headers.contains_key("X-Anthropic-Version"));

            assert!(anthropic_headers.contains_key("X-Anthropic-Version"));
            assert!(!anthropic_headers.contains_key("X-OpenAI-Version"));
        });
    }
}

#[cfg(test)]
mod provider_token_url_tests {
    use super::*;

    #[test]
    #[serial]
    fn test_provider_token_url_management() {
        test_with_cleanup!({
            let mut config = create_config_with_providers();
            let openai_name = get_test_provider_name("openai");

            // Test setting token URL
            let result = config.set_token_url(
                openai_name.clone(),
                "https://auth.openai.com/token".to_string(),
            );
            assert!(result.is_ok());

            let token_url = config.get_token_url(&openai_name);
            assert_eq!(
                token_url,
                Some(&"https://auth.openai.com/token".to_string())
            );

            // Test setting token URL for non-existent provider
            let result =
                config.set_token_url("nonexistent".to_string(), "https://example.com".to_string());
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("not found"));
        });
    }

    #[test]
    #[serial]
    fn test_provider_cached_token_management() {
        test_with_cleanup!({
            let mut config = create_config_with_providers();
            let openai_name = get_test_provider_name("openai");
            let expires_at = Utc::now() + chrono::Duration::hours(1);

            // Test setting cached token
            let result = config.set_cached_token(
                openai_name.clone(),
                "cached-token-123".to_string(),
                expires_at,
            );
            assert!(result.is_ok());

            let cached_token = config.get_cached_token(&openai_name);
            assert!(cached_token.is_some());
            assert_eq!(cached_token.unwrap().token, "cached-token-123");
            assert_eq!(cached_token.unwrap().expires_at, expires_at);

            // Test setting cached token for non-existent provider
            let result =
                config.set_cached_token("nonexistent".to_string(), "token".to_string(), expires_at);
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("not found"));
        });
    }

    #[test]
    #[serial]
    fn test_provider_token_url_clears_cached_token() {
        test_with_cleanup!({
            let mut config = create_config_with_providers();
            let openai_name = get_test_provider_name("openai");
            let expires_at = Utc::now() + chrono::Duration::hours(1);

            // Set a cached token first
            config
                .set_cached_token(openai_name.clone(), "cached-token".to_string(), expires_at)
                .unwrap();

            assert!(config.get_cached_token(&openai_name).is_some());

            // Setting token URL should clear cached token
            config
                .set_token_url(
                    openai_name.clone(),
                    "https://auth.openai.com/token".to_string(),
                )
                .unwrap();

            assert!(config.get_cached_token(&openai_name).is_none());
        });
    }
}
