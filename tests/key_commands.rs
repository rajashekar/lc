//! Integration tests for key management commands
//!
//! This module contains integration tests for API key management
//! commands (lc keys add, list, get, remove).

mod common;

use common::{assertions, create_config_with_providers, get_test_provider_name};
use lc::config::Config;
use std::collections::HashMap;

#[cfg(test)]
mod key_add_tests {
    use super::*;

    #[test]
    fn test_key_add_for_existing_provider() {
        let mut config = create_config_with_providers();
        let openai_name = get_test_provider_name("openai");

        // Test setting API key for existing provider
        let result = config.set_api_key(openai_name.clone(), "sk-test123".to_string());

        assert!(result.is_ok());
        assertions::assert_provider_api_key(&config, &openai_name, "sk-test123");
    }

    #[test]
    fn test_key_add_for_nonexistent_provider() {
        let mut config = create_config_with_providers();

        // Test setting API key for non-existent provider
        let result = config.set_api_key("nonexistent".to_string(), "sk-test123".to_string());

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_key_add_multiple_providers() {
        let mut config = create_config_with_providers();
        let openai_name = get_test_provider_name("openai");
        let anthropic_name = get_test_provider_name("anthropic");

        let test_keys = vec![(&openai_name, "sk-openai-123"), (&anthropic_name, "sk-ant-456")];

        for (provider, key) in test_keys {
            let result = config.set_api_key(provider.to_string(), key.to_string());
            assert!(result.is_ok());
            assertions::assert_provider_api_key(&config, provider, key);
        }
    }
}

#[cfg(test)]
mod key_list_tests {
    use super::*;

    #[test]
    fn test_key_list_empty() {
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
    fn test_key_list_with_keys() {
        let mut config = create_config_with_providers();
        let openai_name = get_test_provider_name("openai");
        let anthropic_name = get_test_provider_name("anthropic");

        // Set API keys for some providers
        config
            .set_api_key(openai_name.clone(), "sk-openai-123".to_string())
            .unwrap();

        // Check which providers have keys
        let openai_provider = config.get_provider(&openai_name).unwrap();
        let anthropic_provider = config.get_provider(&anthropic_name).unwrap();

        assert!(openai_provider.api_key.is_some());
        assert!(anthropic_provider.api_key.is_some()); // From test setup
    }

    #[test]
    fn test_key_list_mixed_providers() {
        let mut config = create_config_with_providers();
        let openai_name = get_test_provider_name("openai");
        let anthropic_name = get_test_provider_name("anthropic");

        // Remove API key from one provider
        if let Some(provider_config) = config.providers.get_mut(&anthropic_name) {
            provider_config.api_key = None;
        }

        // Set API key for another
        config
            .set_api_key(openai_name.clone(), "sk-openai-123".to_string())
            .unwrap();

        // Check status
        let openai_provider = config.get_provider(&openai_name).unwrap();
        let anthropic_provider = config.get_provider(&anthropic_name).unwrap();

        assert!(openai_provider.api_key.is_some());
        assert!(anthropic_provider.api_key.is_none());
    }
}

#[cfg(test)]
mod key_get_tests {
    use super::*;

    #[test]
    fn test_key_get_existing() {
        let mut config = create_config_with_providers();
        let openai_name = get_test_provider_name("openai");
        
        config
            .set_api_key(openai_name.clone(), "sk-test-key".to_string())
            .unwrap();

        let provider = config.get_provider(&openai_name).unwrap();
        assert_eq!(provider.api_key.as_ref().unwrap(), "sk-test-key");
    }

    #[test]
    fn test_key_get_nonexistent_provider() {
        let config = create_config_with_providers();

        let result = config.get_provider("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_key_get_provider_without_key() {
        let mut config = create_config_with_providers();
        let openai_name = get_test_provider_name("openai");

        // Remove API key
        if let Some(provider_config) = config.providers.get_mut(&openai_name) {
            provider_config.api_key = None;
        }

        let provider = config.get_provider(&openai_name).unwrap();
        assert!(provider.api_key.is_none());
    }
}

#[cfg(test)]
mod key_remove_tests {
    use super::*;

    #[test]
    fn test_key_remove_existing() {
        let mut config = create_config_with_providers();
        let openai_name = get_test_provider_name("openai");
        
        config
            .set_api_key(openai_name.clone(), "sk-test-key".to_string())
            .unwrap();

        // Verify key exists
        assertions::assert_provider_has_api_key(&config, &openai_name);

        // Remove key
        if let Some(provider_config) = config.providers.get_mut(&openai_name) {
            provider_config.api_key = None;
        }

        // Verify key is removed
        let provider = config.get_provider(&openai_name).unwrap();
        assert!(provider.api_key.is_none());
    }

    #[test]
    fn test_key_remove_nonexistent_provider() {
        let config = create_config_with_providers();

        let result = config.get_provider("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_key_remove_already_empty() {
        let mut config = create_config_with_providers();
        let openai_name = get_test_provider_name("openai");

        // Remove API key first
        if let Some(provider_config) = config.providers.get_mut(&openai_name) {
            provider_config.api_key = None;
        }

        // Verify it's already None
        let provider = config.get_provider(&openai_name).unwrap();
        assert!(provider.api_key.is_none());

        // "Remove" again (should be idempotent)
        if let Some(provider_config) = config.providers.get_mut(&openai_name) {
            provider_config.api_key = None;
        }

        let provider = config.get_provider(&openai_name).unwrap();
        assert!(provider.api_key.is_none());
    }
}
