//! Common test utilities for LC CLI integration tests
//! 
//! This module provides shared functionality for testing CLI commands,
//! including test data creation, temporary environments, and assertion helpers.

use lc::config::{Config, ProviderConfig};
use std::collections::HashMap;
use tempfile::TempDir;
use chrono::Utc;

/// Helper function to create a temporary config for testing
pub fn create_test_config() -> (Config, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    
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
pub fn create_test_provider_config(endpoint: &str) -> ProviderConfig {
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
pub fn create_config_with_providers() -> Config {
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

/// Test data constants
pub mod test_data {
    pub const TEST_PROVIDERS: &[(&str, &str)] = &[
        ("openai", "https://api.openai.com"),
        ("anthropic", "https://api.anthropic.com"),
        ("cohere", "https://api.cohere.ai"),
        ("huggingface", "https://api-inference.huggingface.co"),
    ];
    
    pub const TEST_MODELS: &[&str] = &[
        "gpt-4",
        "gpt-3.5-turbo",
        "claude-3-opus",
        "claude-3-sonnet",
    ];
    
    pub const TEST_HEADERS: &[(&str, &str)] = &[
        ("X-API-Version", "2023-12-01"),
        ("X-Client", "lc-cli"),
        ("User-Agent", "LC/1.0"),
    ];
}

/// Assertion helpers for common test patterns
pub mod assertions {
    use lc::config::Config;
    
    pub fn assert_provider_exists(config: &Config, name: &str) {
        assert!(config.has_provider(name), "Provider '{}' should exist", name);
    }
    
    pub fn assert_provider_not_exists(config: &Config, name: &str) {
        assert!(!config.has_provider(name), "Provider '{}' should not exist", name);
    }
    
    pub fn assert_provider_endpoint(config: &Config, name: &str, expected_endpoint: &str) {
        let provider = config.get_provider(name).expect("Provider should exist");
        assert_eq!(provider.endpoint, expected_endpoint, 
                   "Provider '{}' endpoint should be '{}'", name, expected_endpoint);
    }
    
    pub fn assert_provider_has_api_key(config: &Config, name: &str) {
        let provider = config.get_provider(name).expect("Provider should exist");
        assert!(provider.api_key.is_some(), "Provider '{}' should have an API key", name);
    }
    
    pub fn assert_provider_api_key(config: &Config, name: &str, expected_key: &str) {
        let provider = config.get_provider(name).expect("Provider should exist");
        assert_eq!(provider.api_key.as_ref().unwrap(), expected_key,
                   "Provider '{}' API key should be '{}'", name, expected_key);
    }
    
    pub fn assert_default_provider(config: &Config, expected_name: &str) {
        assert_eq!(config.default_provider.as_ref().unwrap(), expected_name,
                   "Default provider should be '{}'", expected_name);
    }
    
    pub fn assert_header_exists(config: &Config, provider: &str, header_name: &str, expected_value: &str) {
        let headers = config.list_headers(provider).expect("Provider should exist");
        assert_eq!(headers.get(header_name).unwrap(), expected_value,
                   "Header '{}' should have value '{}'", header_name, expected_value);
    }
    
    pub fn assert_header_count(config: &Config, provider: &str, expected_count: usize) {
        let headers = config.list_headers(provider).expect("Provider should exist");
        assert_eq!(headers.len(), expected_count,
                   "Provider '{}' should have {} headers", provider, expected_count);
    }
}