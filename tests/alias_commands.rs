//! Integration tests for alias commands
//! 
//! This module contains comprehensive integration tests for all alias-related
//! CLI commands, testing the underlying functionality as the CLI would use it.

mod common;

use lc::config::Config;
use std::collections::HashMap;

#[cfg(test)]
mod alias_add_tests {
    use super::*;

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
            stream: None,
        };
        
        // Add some test providers
        config.providers.insert("openai".to_string(), lc::config::ProviderConfig {
            endpoint: "https://api.openai.com/v1/chat/completions".to_string(),
            api_key: None,
            models: Vec::new(),
            models_path: "/models".to_string(),
            chat_path: "/chat/completions".to_string(),
            headers: HashMap::new(),
            token_url: None,
            cached_token: None,
            auth_type: None,
            vars: std::collections::HashMap::new(),
            images_path: Some("/images/generations".to_string()),
            embeddings_path: Some("/embeddings".to_string()),
        });
        
        config.providers.insert("anthropic".to_string(), lc::config::ProviderConfig {
            endpoint: "https://api.anthropic.com/v1/messages".to_string(),
            api_key: None,
            models: Vec::new(),
            models_path: "/models".to_string(),
            chat_path: "/chat/completions".to_string(),
            headers: HashMap::new(),
            token_url: None,
            cached_token: None,
            auth_type: None,
            vars: std::collections::HashMap::new(),
            images_path: Some("/images/generations".to_string()),
            embeddings_path: Some("/embeddings".to_string()),
        });
        
        config
    }

    #[test]
    fn test_alias_add_basic() {
        let mut config = create_config_with_providers();
        
        // Add a basic alias
        let result = config.add_alias("gpt4".to_string(), "openai:gpt-4".to_string());
        assert!(result.is_ok());
        
        // Verify alias was added
        let alias = config.get_alias("gpt4");
        assert_eq!(alias, Some(&"openai:gpt-4".to_string()));
        
        // Verify it appears in the aliases list
        let aliases = config.list_aliases();
        assert_eq!(aliases.len(), 1);
        assert_eq!(aliases.get("gpt4"), Some(&"openai:gpt-4".to_string()));
    }

    #[test]
    fn test_alias_add_multiple() {
        let mut config = create_config_with_providers();
        
        // Add multiple aliases
        let result1 = config.add_alias("gpt4".to_string(), "openai:gpt-4".to_string());
        let result2 = config.add_alias("claude".to_string(), "anthropic:claude-3-sonnet".to_string());
        let result3 = config.add_alias("gpt35".to_string(), "openai:gpt-3.5-turbo".to_string());
        
        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert!(result3.is_ok());
        
        // Verify all aliases exist
        assert_eq!(config.get_alias("gpt4"), Some(&"openai:gpt-4".to_string()));
        assert_eq!(config.get_alias("claude"), Some(&"anthropic:claude-3-sonnet".to_string()));
        assert_eq!(config.get_alias("gpt35"), Some(&"openai:gpt-3.5-turbo".to_string()));
        
        // Verify aliases list contains all
        let aliases = config.list_aliases();
        assert_eq!(aliases.len(), 3);
    }

    #[test]
    fn test_alias_add_overwrite_existing() {
        let mut config = create_config_with_providers();
        
        // Add initial alias
        let result1 = config.add_alias("gpt".to_string(), "openai:gpt-4".to_string());
        assert!(result1.is_ok());
        assert_eq!(config.get_alias("gpt"), Some(&"openai:gpt-4".to_string()));
        
        // Overwrite with new target
        let result2 = config.add_alias("gpt".to_string(), "openai:gpt-3.5-turbo".to_string());
        assert!(result2.is_ok());
        assert_eq!(config.get_alias("gpt"), Some(&"openai:gpt-3.5-turbo".to_string()));
        
        // Verify only one alias exists
        let aliases = config.list_aliases();
        assert_eq!(aliases.len(), 1);
    }

    #[test]
    fn test_alias_add_nonexistent_provider() {
        let mut config = create_config_with_providers();
        
        // Try to add alias with non-existent provider
        let result = config.add_alias("invalid".to_string(), "nonexistent:model".to_string());
        assert!(result.is_err());
        
        // Verify alias was not added
        assert!(config.get_alias("invalid").is_none());
        assert!(config.list_aliases().is_empty());
    }

    #[test]
    fn test_alias_add_invalid_format() {
        let mut config = create_config_with_providers();
        
        // Try to add alias without colon separator
        let result = config.add_alias("invalid".to_string(), "just-a-model".to_string());
        assert!(result.is_err());
        
        // Verify alias was not added
        assert!(config.get_alias("invalid").is_none());
        assert!(config.list_aliases().is_empty());
    }

    #[test]
    fn test_alias_add_empty_name() {
        let mut config = create_config_with_providers();
        
        // Try to add alias with empty name
        let result = config.add_alias("".to_string(), "openai:gpt-4".to_string());
        assert!(result.is_ok()); // Empty names are technically allowed
        
        // Verify alias was added
        assert_eq!(config.get_alias(""), Some(&"openai:gpt-4".to_string()));
    }

    #[test]
    fn test_alias_add_special_characters() {
        let mut config = create_config_with_providers();
        
        // Add aliases with special characters
        let result1 = config.add_alias("gpt-4".to_string(), "openai:gpt-4".to_string());
        let result2 = config.add_alias("claude_3".to_string(), "anthropic:claude-3-sonnet".to_string());
        let result3 = config.add_alias("gpt.4".to_string(), "openai:gpt-4".to_string());
        
        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert!(result3.is_ok());
        
        // Verify all aliases exist
        assert_eq!(config.get_alias("gpt-4"), Some(&"openai:gpt-4".to_string()));
        assert_eq!(config.get_alias("claude_3"), Some(&"anthropic:claude-3-sonnet".to_string()));
        assert_eq!(config.get_alias("gpt.4"), Some(&"openai:gpt-4".to_string()));
    }
}

#[cfg(test)]
mod alias_delete_tests {
    use super::*;

    fn create_config_with_aliases() -> Config {
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
        config.providers.insert("openai".to_string(), lc::config::ProviderConfig {
            endpoint: "https://api.openai.com/v1/chat/completions".to_string(),
            api_key: None,
            models: Vec::new(),
            models_path: "/models".to_string(),
            chat_path: "/chat/completions".to_string(),
            headers: HashMap::new(),
            token_url: None,
            cached_token: None,
            auth_type: None,
            vars: std::collections::HashMap::new(),
            images_path: Some("/images/generations".to_string()),
            embeddings_path: Some("/embeddings".to_string()),
        });
        
        // Add test aliases
        config.aliases.insert("gpt4".to_string(), "openai:gpt-4".to_string());
        config.aliases.insert("gpt35".to_string(), "openai:gpt-3.5-turbo".to_string());
        config.aliases.insert("claude".to_string(), "anthropic:claude-3-sonnet".to_string());
        
        config
    }

    #[test]
    fn test_alias_delete_existing() {
        let mut config = create_config_with_aliases();
        
        // Verify alias exists before deletion
        assert_eq!(config.get_alias("gpt4"), Some(&"openai:gpt-4".to_string()));
        assert_eq!(config.list_aliases().len(), 3);
        
        // Delete the alias
        let result = config.remove_alias("gpt4".to_string());
        assert!(result.is_ok());
        
        // Verify alias was removed
        assert!(config.get_alias("gpt4").is_none());
        assert_eq!(config.list_aliases().len(), 2);
        
        // Verify other aliases still exist
        assert_eq!(config.get_alias("gpt35"), Some(&"openai:gpt-3.5-turbo".to_string()));
        assert_eq!(config.get_alias("claude"), Some(&"anthropic:claude-3-sonnet".to_string()));
    }

    #[test]
    fn test_alias_delete_nonexistent() {
        let mut config = create_config_with_aliases();
        
        // Try to delete non-existent alias
        let result = config.remove_alias("nonexistent".to_string());
        assert!(result.is_err());
        
        // Verify no aliases were affected
        assert_eq!(config.list_aliases().len(), 3);
        assert_eq!(config.get_alias("gpt4"), Some(&"openai:gpt-4".to_string()));
    }

    #[test]
    fn test_alias_delete_all() {
        let mut config = create_config_with_aliases();
        
        // Delete all aliases one by one
        let result1 = config.remove_alias("gpt4".to_string());
        let result2 = config.remove_alias("gpt35".to_string());
        let result3 = config.remove_alias("claude".to_string());
        
        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert!(result3.is_ok());
        
        // Verify all aliases are gone
        assert!(config.list_aliases().is_empty());
        assert!(config.get_alias("gpt4").is_none());
        assert!(config.get_alias("gpt35").is_none());
        assert!(config.get_alias("claude").is_none());
    }

    #[test]
    fn test_alias_delete_empty_name() {
        let mut config = create_config_with_aliases();
        
        // Add alias with empty name
        config.aliases.insert("".to_string(), "openai:gpt-4".to_string());
        assert_eq!(config.list_aliases().len(), 4);
        
        // Delete alias with empty name
        let result = config.remove_alias("".to_string());
        assert!(result.is_ok());
        
        // Verify empty name alias was removed
        assert!(config.get_alias("").is_none());
        assert_eq!(config.list_aliases().len(), 3);
    }

    #[test]
    fn test_alias_delete_case_sensitive() {
        let mut config = create_config_with_aliases();
        
        // Try to delete with different case
        let result = config.remove_alias("GPT4".to_string());
        assert!(result.is_err());
        
        // Verify original alias still exists
        assert_eq!(config.get_alias("gpt4"), Some(&"openai:gpt-4".to_string()));
        assert_eq!(config.list_aliases().len(), 3);
    }
}

#[cfg(test)]
mod alias_list_tests {
    use super::*;

    #[test]
    fn test_alias_list_empty() {
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
        
        let aliases = config.list_aliases();
        assert!(aliases.is_empty());
        assert_eq!(aliases.len(), 0);
    }

    #[test]
    fn test_alias_list_with_aliases() {
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
        
        // Add some aliases
        config.aliases.insert("gpt4".to_string(), "openai:gpt-4".to_string());
        config.aliases.insert("claude".to_string(), "anthropic:claude-3-sonnet".to_string());
        config.aliases.insert("gpt35".to_string(), "openai:gpt-3.5-turbo".to_string());
        
        let aliases = config.list_aliases();
        assert_eq!(aliases.len(), 3);
        
        // Verify all aliases are present
        assert_eq!(aliases.get("gpt4"), Some(&"openai:gpt-4".to_string()));
        assert_eq!(aliases.get("claude"), Some(&"anthropic:claude-3-sonnet".to_string()));
        assert_eq!(aliases.get("gpt35"), Some(&"openai:gpt-3.5-turbo".to_string()));
    }

    #[test]
    fn test_alias_list_ordering() {
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
        
        // Add aliases in specific order
        config.aliases.insert("zebra".to_string(), "provider:zebra-model".to_string());
        config.aliases.insert("alpha".to_string(), "provider:alpha-model".to_string());
        config.aliases.insert("beta".to_string(), "provider:beta-model".to_string());
        
        let aliases = config.list_aliases();
        assert_eq!(aliases.len(), 3);
        
        // HashMap doesn't guarantee order, but all should be present
        assert!(aliases.contains_key("zebra"));
        assert!(aliases.contains_key("alpha"));
        assert!(aliases.contains_key("beta"));
    }

    #[test]
    fn test_alias_list_immutable() {
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
        config.aliases.insert("test".to_string(), "provider:model".to_string());
        
        let aliases = config.list_aliases();
        assert_eq!(aliases.len(), 1);
        
        // The returned reference should be immutable
        // (This is enforced by the type system, but we can verify the content)
        assert_eq!(aliases.get("test"), Some(&"provider:model".to_string()));
    }
}

#[cfg(test)]
mod alias_validation_tests {
    use super::*;

    #[test]
    fn test_alias_target_format_validation() {
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
        
        // Valid formats
        let valid_targets = vec![
            "openai:gpt-4",
            "anthropic:claude-3-sonnet",
            "provider:model-name",
            "a:b",
            "long-provider-name:very-long-model-name-with-dashes",
        ];
        
        for target in valid_targets {
            let result = config.add_alias("test".to_string(), target.to_string());
            // Will fail due to provider validation, but format validation should pass
            // The error should be about provider not existing, not format
            if result.is_err() {
                let error_msg = format!("{}", result.unwrap_err());
                assert!(!error_msg.contains("format"));
            }
        }
    }

    #[test]
    fn test_alias_target_invalid_formats() {
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
        
        // Invalid formats (no colon)
        let invalid_targets = vec![
            "just-a-model",
            "no-separator-here",
            "model",
            "",
            "provider-model", // dash instead of colon
            "provider model", // space instead of colon
        ];
        
        for target in invalid_targets {
            let result = config.add_alias("test".to_string(), target.to_string());
            assert!(result.is_err());
            
            let error_msg = format!("{}", result.unwrap_err());
            assert!(error_msg.contains("format") || error_msg.contains("provider:model"));
        }
    }

    #[test]
    fn test_alias_target_multiple_colons() {
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
        
        // Add a provider first
        config.providers.insert("provider".to_string(), lc::config::ProviderConfig {
            endpoint: "https://api.example.com".to_string(),
            api_key: None,
            models: Vec::new(),
            models_path: "/models".to_string(),
            chat_path: "/chat/completions".to_string(),
            headers: HashMap::new(),
            token_url: None,
            cached_token: None,
            auth_type: None,
            vars: std::collections::HashMap::new(),
            images_path: Some("/images/generations".to_string()),
            embeddings_path: Some("/embeddings".to_string()),
        });
        
        // Target with multiple colons - should be valid (only first colon is separator)
        let result = config.add_alias("test".to_string(), "provider:model:with:colons".to_string());
        assert!(result.is_ok());
        
        let alias = config.get_alias("test");
        assert_eq!(alias, Some(&"provider:model:with:colons".to_string()));
    }

    #[test]
    fn test_alias_name_edge_cases() {
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
        
        // Add a provider first
        config.providers.insert("provider".to_string(), lc::config::ProviderConfig {
            endpoint: "https://api.example.com".to_string(),
            api_key: None,
            models: Vec::new(),
            models_path: "/models".to_string(),
            chat_path: "/chat/completions".to_string(),
            headers: HashMap::new(),
            token_url: None,
            cached_token: None,
            auth_type: None,
            vars: std::collections::HashMap::new(),
            images_path: Some("/images/generations".to_string()),
            embeddings_path: Some("/embeddings".to_string()),
        });
        
        // Various alias names
        let alias_names = vec![
            "normal-alias",
            "alias_with_underscores",
            "alias.with.dots",
            "alias123",
            "123alias",
            "a",
            "very-long-alias-name-with-many-dashes-and-words",
        ];
        
        for name in alias_names {
            let result = config.add_alias(name.to_string(), "provider:model".to_string());
            assert!(result.is_ok(), "Failed to add alias with name: {}", name);
            
            let alias = config.get_alias(name);
            assert_eq!(alias, Some(&"provider:model".to_string()));
        }
    }
}

#[cfg(test)]
mod alias_resolution_tests {
    use super::*;

    fn create_config_with_aliases_and_providers() -> Config {
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
        
        // Add providers
        config.providers.insert("openai".to_string(), lc::config::ProviderConfig {
            endpoint: "https://api.openai.com/v1/chat/completions".to_string(),
            models: Vec::new(),
            models_path: "/models".to_string(),
            chat_path: "/chat/completions".to_string(),
            api_key: None,
            headers: HashMap::new(),
            token_url: None,
            cached_token: None,
            auth_type: None,
            vars: std::collections::HashMap::new(),
            images_path: Some("/images/generations".to_string()),
            embeddings_path: Some("/embeddings".to_string()),
        });
        
        config.providers.insert("anthropic".to_string(), lc::config::ProviderConfig {
            endpoint: "https://api.anthropic.com/v1/messages".to_string(),
            models: Vec::new(),
            models_path: "/models".to_string(),
            chat_path: "/chat/completions".to_string(),
            api_key: None,
            headers: HashMap::new(),
            token_url: None,
            cached_token: None,
            auth_type: None,
            vars: std::collections::HashMap::new(),
            images_path: Some("/images/generations".to_string()),
            embeddings_path: Some("/embeddings".to_string()),
        });
        
        // Add aliases
        config.aliases.insert("gpt4".to_string(), "openai:gpt-4".to_string());
        config.aliases.insert("claude".to_string(), "anthropic:claude-3-sonnet".to_string());
        config.aliases.insert("fast".to_string(), "openai:gpt-3.5-turbo".to_string());
        
        config
    }

    #[test]
    fn test_alias_get_existing() {
        let config = create_config_with_aliases_and_providers();
        
        // Test getting existing aliases
        assert_eq!(config.get_alias("gpt4"), Some(&"openai:gpt-4".to_string()));
        assert_eq!(config.get_alias("claude"), Some(&"anthropic:claude-3-sonnet".to_string()));
        assert_eq!(config.get_alias("fast"), Some(&"openai:gpt-3.5-turbo".to_string()));
    }

    #[test]
    fn test_alias_get_nonexistent() {
        let config = create_config_with_aliases_and_providers();
        
        // Test getting non-existent aliases
        assert!(config.get_alias("nonexistent").is_none());
        assert!(config.get_alias("").is_none());
        assert!(config.get_alias("GPT4").is_none()); // Case sensitive
    }

    #[test]
    fn test_alias_case_sensitivity() {
        let config = create_config_with_aliases_and_providers();
        
        // Aliases should be case sensitive
        assert_eq!(config.get_alias("gpt4"), Some(&"openai:gpt-4".to_string()));
        assert!(config.get_alias("GPT4").is_none());
        assert!(config.get_alias("Gpt4").is_none());
        assert!(config.get_alias("GpT4").is_none());
    }

    #[test]
    fn test_alias_resolution_chain() {
        let mut config = create_config_with_aliases_and_providers();
        
        // Create an alias that points to another alias (should not be resolved recursively)
        config.aliases.insert("best".to_string(), "gpt4".to_string());
        
        // The alias should return the literal value, not resolve recursively
        assert_eq!(config.get_alias("best"), Some(&"gpt4".to_string()));
        assert_ne!(config.get_alias("best"), Some(&"openai:gpt-4".to_string()));
    }

    #[test]
    fn test_alias_with_special_model_names() {
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
        
        // Add provider
        config.providers.insert("provider".to_string(), lc::config::ProviderConfig {
            endpoint: "https://api.example.com".to_string(),
            api_key: None,
            models: Vec::new(),
            models_path: "/models".to_string(),
            chat_path: "/chat/completions".to_string(),
            headers: HashMap::new(),
            token_url: None,
            cached_token: None,
            auth_type: None,
            vars: std::collections::HashMap::new(),
            images_path: Some("/images/generations".to_string()),
            embeddings_path: Some("/embeddings".to_string()),
        });
        
        // Add aliases with special model names
        let special_models = vec![
            "model-with-dashes",
            "model_with_underscores",
            "model.with.dots",
            "model123",
            "123model",
            "model@version",
            "model:with:colons",
        ];
        
        for model in special_models {
            let target = format!("provider:{}", model);
            let alias_name = format!("alias_{}", model.replace([':', '@', '.'], "_"));
            
            let result = config.add_alias(alias_name.clone(), target.clone());
            assert!(result.is_ok());
            
            let resolved = config.get_alias(&alias_name);
            assert_eq!(resolved, Some(&target));
        }
    }
}

#[cfg(test)]
mod alias_integration_tests {
    use super::*;

    #[test]
    fn test_alias_workflow_complete() {
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
        
        // Add provider
        config.providers.insert("openai".to_string(), lc::config::ProviderConfig {
            endpoint: "https://api.openai.com/v1/chat/completions".to_string(),
            models: Vec::new(),
            models_path: "/models".to_string(),
            chat_path: "/chat/completions".to_string(),
            api_key: None,
            headers: HashMap::new(),
            token_url: None,
            cached_token: None,
            auth_type: None,
            vars: std::collections::HashMap::new(),
            images_path: Some("/images/generations".to_string()),
            embeddings_path: Some("/embeddings".to_string()),
        });
        
        // Start with empty aliases
        assert!(config.list_aliases().is_empty());
        
        // Add first alias
        let result = config.add_alias("gpt4".to_string(), "openai:gpt-4".to_string());
        assert!(result.is_ok());
        assert_eq!(config.list_aliases().len(), 1);
        
        // Add second alias
        let result = config.add_alias("fast".to_string(), "openai:gpt-3.5-turbo".to_string());
        assert!(result.is_ok());
        assert_eq!(config.list_aliases().len(), 2);
        
        // Verify both aliases exist
        assert_eq!(config.get_alias("gpt4"), Some(&"openai:gpt-4".to_string()));
        assert_eq!(config.get_alias("fast"), Some(&"openai:gpt-3.5-turbo".to_string()));
        
        // Update existing alias
        let result = config.add_alias("gpt4".to_string(), "openai:gpt-4-turbo".to_string());
        assert!(result.is_ok());
        assert_eq!(config.list_aliases().len(), 2); // Still 2 aliases
        assert_eq!(config.get_alias("gpt4"), Some(&"openai:gpt-4-turbo".to_string()));
        
        // Remove one alias
        let result = config.remove_alias("fast".to_string());
        assert!(result.is_ok());
        assert_eq!(config.list_aliases().len(), 1);
        assert!(config.get_alias("fast").is_none());
        assert_eq!(config.get_alias("gpt4"), Some(&"openai:gpt-4-turbo".to_string()));
        
        // Remove last alias
        let result = config.remove_alias("gpt4".to_string());
        assert!(result.is_ok());
        assert!(config.list_aliases().is_empty());
        assert!(config.get_alias("gpt4").is_none());
    }

    #[test]
    fn test_alias_persistence_simulation() {
        // Simulate config save/load cycle
        let mut config1 = Config {
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
        
        // Add provider
        config1.providers.insert("openai".to_string(), lc::config::ProviderConfig {
            endpoint: "https://api.openai.com/v1/chat/completions".to_string(),
            models: Vec::new(),
            models_path: "/models".to_string(),
            chat_path: "/chat/completions".to_string(),
            api_key: None,
            headers: HashMap::new(),
            token_url: None,
            cached_token: None,
            auth_type: None,
            vars: std::collections::HashMap::new(),
            images_path: Some("/images/generations".to_string()),
            embeddings_path: Some("/embeddings".to_string()),
        });
        
        // Add aliases
        config1.add_alias("gpt4".to_string(), "openai:gpt-4".to_string()).unwrap();
        config1.add_alias("fast".to_string(), "openai:gpt-3.5-turbo".to_string()).unwrap();
        
        // Simulate serialization/deserialization by cloning the aliases
        let mut config2 = Config {
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
        config2.providers = config1.providers.clone();
        config2.aliases = config1.aliases.clone();
        
        // Verify aliases persisted
        assert_eq!(config2.list_aliases().len(), 2);
        assert_eq!(config2.get_alias("gpt4"), Some(&"openai:gpt-4".to_string()));
        assert_eq!(config2.get_alias("fast"), Some(&"openai:gpt-3.5-turbo".to_string()));
    }

    #[test]
    fn test_alias_with_provider_removal() {
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
        
        // Add provider and alias
        config.providers.insert("openai".to_string(), lc::config::ProviderConfig {
            endpoint: "https://api.openai.com/v1/chat/completions".to_string(),
            models: Vec::new(),
            models_path: "/models".to_string(),
            chat_path: "/chat/completions".to_string(),
            api_key: None,
            headers: HashMap::new(),
            token_url: None,
            cached_token: None,
            auth_type: None,
            vars: std::collections::HashMap::new(),
            images_path: Some("/images/generations".to_string()),
            embeddings_path: Some("/embeddings".to_string()),
        });
        
        config.add_alias("gpt4".to_string(), "openai:gpt-4".to_string()).unwrap();
        assert_eq!(config.get_alias("gpt4"), Some(&"openai:gpt-4".to_string()));
        
        // Remove the provider
        config.providers.remove("openai");
        
        // Alias should still exist but point to non-existent provider
        assert_eq!(config.get_alias("gpt4"), Some(&"openai:gpt-4".to_string()));
        
        // Adding new alias with same provider should fail
        let result = config.add_alias("new".to_string(), "openai:gpt-3.5".to_string());
        assert!(result.is_err());
    }
}