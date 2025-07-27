//! Integration tests for configuration commands
//! 
//! This module contains integration tests for configuration management
//! commands (lc config set, get, delete).

mod common;

use common::{create_config_with_providers};
use lc::config::Config;
use std::collections::HashMap;

#[cfg(test)]
mod config_set_tests {
    use super::*;

    #[test]
    fn test_config_set_default_provider() {
        let mut config = create_config_with_providers();

        // Set default provider
        config.default_provider = Some("anthropic".to_string());
        
        assert_eq!(config.default_provider, Some("anthropic".to_string()));
    }

    #[test]
    fn test_config_set_default_model() {
        let mut config = create_config_with_providers();

        // Set default model
        config.default_model = Some("gpt-4".to_string());
        
        assert_eq!(config.default_model, Some("gpt-4".to_string()));
    }

    #[test]
    fn test_config_set_system_prompt() {
        let mut config = create_config_with_providers();

        // Set system prompt
        config.system_prompt = Some("You are a helpful assistant.".to_string());
        
        assert_eq!(config.system_prompt, Some("You are a helpful assistant.".to_string()));
    }

    #[test]
    fn test_config_set_max_tokens() {
        let mut config = create_config_with_providers();

        // Test parsing max tokens
        let parsed_tokens = Config::parse_max_tokens("2k").unwrap();
        assert_eq!(parsed_tokens, 2000);

        config.max_tokens = Some(parsed_tokens);
        assert_eq!(config.max_tokens, Some(2000));
    }

    #[test]
    fn test_config_set_temperature() {
        let mut config = create_config_with_providers();

        // Test parsing temperature
        let parsed_temp = Config::parse_temperature("0.7").unwrap();
        assert_eq!(parsed_temp, 0.7);

        config.temperature = Some(parsed_temp);
        assert_eq!(config.temperature, Some(0.7));
    }
}

#[cfg(test)]
mod config_get_tests {
    use super::*;

    #[test]
    fn test_config_get_existing_values() {
        let mut config = create_config_with_providers();
        
        // Set some values
        config.default_provider = Some("openai".to_string());
        config.default_model = Some("gpt-4".to_string());
        config.system_prompt = Some("Test prompt".to_string());
        config.max_tokens = Some(1000);
        config.temperature = Some(0.5);

        // Verify values can be retrieved
        assert_eq!(config.default_provider, Some("openai".to_string()));
        assert_eq!(config.default_model, Some("gpt-4".to_string()));
        assert_eq!(config.system_prompt, Some("Test prompt".to_string()));
        assert_eq!(config.max_tokens, Some(1000));
        assert_eq!(config.temperature, Some(0.5));
    }

    #[test]
    fn test_config_get_unset_values() {
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

        // Verify all values are None
        assert!(config.default_provider.is_none());
        assert!(config.default_model.is_none());
        assert!(config.system_prompt.is_none());
        assert!(config.max_tokens.is_none());
        assert!(config.temperature.is_none());
    }
}

#[cfg(test)]
mod config_delete_tests {
    use super::*;

    #[test]
    fn test_config_delete_values() {
        let mut config = create_config_with_providers();
        
        // Set values first
        config.default_provider = Some("openai".to_string());
        config.default_model = Some("gpt-4".to_string());
        config.system_prompt = Some("Test prompt".to_string());
        config.max_tokens = Some(1000);
        config.temperature = Some(0.5);

        // Delete values
        config.default_provider = None;
        config.default_model = None;
        config.system_prompt = None;
        config.max_tokens = None;
        config.temperature = None;

        // Verify values are deleted
        assert!(config.default_provider.is_none());
        assert!(config.default_model.is_none());
        assert!(config.system_prompt.is_none());
        assert!(config.max_tokens.is_none());
        assert!(config.temperature.is_none());
    }
}

#[cfg(test)]
mod config_validation_tests {
    use super::*;

    #[test]
    fn test_max_tokens_parsing() {
        // Test valid formats
        assert_eq!(Config::parse_max_tokens("1000").unwrap(), 1000);
        assert_eq!(Config::parse_max_tokens("2k").unwrap(), 2000);
        assert_eq!(Config::parse_max_tokens("1.5k").unwrap(), 1500);

        // Test invalid formats
        assert!(Config::parse_max_tokens("invalid").is_err());
        assert!(Config::parse_max_tokens("").is_err());
    }

    #[test]
    fn test_temperature_parsing() {
        // Test valid formats
        assert_eq!(Config::parse_temperature("0.0").unwrap(), 0.0);
        assert_eq!(Config::parse_temperature("0.7").unwrap(), 0.7);
        assert_eq!(Config::parse_temperature("1.0").unwrap(), 1.0);
        assert_eq!(Config::parse_temperature("2.0").unwrap(), 2.0);

        // Test invalid formats
        assert!(Config::parse_temperature("invalid").is_err());
        assert!(Config::parse_temperature("").is_err());
    }

    #[test]
    fn test_template_resolution() {
        let mut config = create_config_with_providers();
        
        // Add a template
        config.templates.insert("helpful".to_string(), "You are a helpful assistant.".to_string());

        // Test template resolution
        let resolved = config.resolve_template_or_prompt("t:helpful");
        assert_eq!(resolved, "You are a helpful assistant.");

        // Test non-template input
        let resolved = config.resolve_template_or_prompt("Regular prompt");
        assert_eq!(resolved, "Regular prompt");

        // Test non-existent template
        let resolved = config.resolve_template_or_prompt("t:nonexistent");
        assert_eq!(resolved, "t:nonexistent");
    }
}