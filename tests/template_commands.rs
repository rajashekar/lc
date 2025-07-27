//! Integration tests for template commands
//! 
//! This module contains comprehensive integration tests for all template-related
//! CLI commands, testing the underlying functionality as the CLI would use it.

mod common;

use lc::config::Config;
use std::collections::HashMap;

#[cfg(test)]
mod template_add_tests {
    use super::*;

    fn create_empty_config() -> Config {
        Config {
            providers: HashMap::new(),
            default_provider: None,
            default_model: None,
            aliases: HashMap::new(),
            system_prompt: None,
            templates: HashMap::new(),
            max_tokens: None,
            temperature: None,
        }
    }

    #[test]
    fn test_template_add_basic() {
        let mut config = create_empty_config();
        
        // Add a basic template
        let result = config.add_template("code_review".to_string(), "You are a senior software engineer reviewing code. Please provide constructive feedback.".to_string());
        assert!(result.is_ok());
        
        // Verify template was added
        let template = config.get_template("code_review");
        assert_eq!(template, Some(&"You are a senior software engineer reviewing code. Please provide constructive feedback.".to_string()));
        
        // Verify it appears in the templates list
        let templates = config.list_templates();
        assert_eq!(templates.len(), 1);
        assert_eq!(templates.get("code_review"), Some(&"You are a senior software engineer reviewing code. Please provide constructive feedback.".to_string()));
    }

    #[test]
    fn test_template_add_multiple() {
        let mut config = create_empty_config();
        
        // Add multiple templates
        let result1 = config.add_template("code_review".to_string(), "You are a senior software engineer reviewing code.".to_string());
        let result2 = config.add_template("documentation".to_string(), "You are a technical writer creating clear documentation.".to_string());
        let result3 = config.add_template("debugging".to_string(), "You are an expert debugger helping to solve issues.".to_string());
        
        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert!(result3.is_ok());
        
        // Verify all templates exist
        assert_eq!(config.get_template("code_review"), Some(&"You are a senior software engineer reviewing code.".to_string()));
        assert_eq!(config.get_template("documentation"), Some(&"You are a technical writer creating clear documentation.".to_string()));
        assert_eq!(config.get_template("debugging"), Some(&"You are an expert debugger helping to solve issues.".to_string()));
        
        // Verify templates list contains all
        let templates = config.list_templates();
        assert_eq!(templates.len(), 3);
    }

    #[test]
    fn test_template_add_overwrite_existing() {
        let mut config = create_empty_config();
        
        // Add initial template
        let result1 = config.add_template("prompt".to_string(), "Original prompt content".to_string());
        assert!(result1.is_ok());
        assert_eq!(config.get_template("prompt"), Some(&"Original prompt content".to_string()));
        
        // Overwrite with new content
        let result2 = config.add_template("prompt".to_string(), "Updated prompt content".to_string());
        assert!(result2.is_ok());
        assert_eq!(config.get_template("prompt"), Some(&"Updated prompt content".to_string()));
        
        // Verify only one template exists
        let templates = config.list_templates();
        assert_eq!(templates.len(), 1);
    }

    #[test]
    fn test_template_add_empty_name() {
        let mut config = create_empty_config();
        
        // Add template with empty name
        let result = config.add_template("".to_string(), "Some prompt content".to_string());
        assert!(result.is_ok()); // Empty names are technically allowed
        
        // Verify template was added
        assert_eq!(config.get_template(""), Some(&"Some prompt content".to_string()));
    }

    #[test]
    fn test_template_add_empty_content() {
        let mut config = create_empty_config();
        
        // Add template with empty content
        let result = config.add_template("empty".to_string(), "".to_string());
        assert!(result.is_ok());
        
        // Verify template was added with empty content
        assert_eq!(config.get_template("empty"), Some(&"".to_string()));
    }

    #[test]
    fn test_template_add_special_characters() {
        let mut config = create_empty_config();
        
        // Add templates with special characters in names
        let result1 = config.add_template("code-review".to_string(), "Code review prompt".to_string());
        let result2 = config.add_template("code_review".to_string(), "Code review with underscore".to_string());
        let result3 = config.add_template("code.review".to_string(), "Code review with dot".to_string());
        let result4 = config.add_template("code123".to_string(), "Code review with numbers".to_string());
        
        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert!(result3.is_ok());
        assert!(result4.is_ok());
        
        // Verify all templates exist
        assert_eq!(config.get_template("code-review"), Some(&"Code review prompt".to_string()));
        assert_eq!(config.get_template("code_review"), Some(&"Code review with underscore".to_string()));
        assert_eq!(config.get_template("code.review"), Some(&"Code review with dot".to_string()));
        assert_eq!(config.get_template("code123"), Some(&"Code review with numbers".to_string()));
    }

    #[test]
    fn test_template_add_multiline_content() {
        let mut config = create_empty_config();
        
        let multiline_prompt = "You are a helpful assistant.\n\nPlease follow these guidelines:\n1. Be concise\n2. Be accurate\n3. Be helpful";
        
        // Add template with multiline content
        let result = config.add_template("assistant".to_string(), multiline_prompt.to_string());
        assert!(result.is_ok());
        
        // Verify template content is preserved exactly
        assert_eq!(config.get_template("assistant"), Some(&multiline_prompt.to_string()));
    }

    #[test]
    fn test_template_add_long_content() {
        let mut config = create_empty_config();
        
        let long_prompt = "A".repeat(1000); // 1000 character prompt
        
        // Add template with long content
        let result = config.add_template("long".to_string(), long_prompt.clone());
        assert!(result.is_ok());
        
        // Verify long content is preserved
        assert_eq!(config.get_template("long"), Some(&long_prompt));
    }
}

#[cfg(test)]
mod template_delete_tests {
    use super::*;

    fn create_config_with_templates() -> Config {
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
        
        // Add test templates
        config.templates.insert("code_review".to_string(), "You are a senior software engineer reviewing code.".to_string());
        config.templates.insert("documentation".to_string(), "You are a technical writer creating clear documentation.".to_string());
        config.templates.insert("debugging".to_string(), "You are an expert debugger helping to solve issues.".to_string());
        
        config
    }

    #[test]
    fn test_template_delete_existing() {
        let mut config = create_config_with_templates();
        
        // Verify template exists before deletion
        assert_eq!(config.get_template("code_review"), Some(&"You are a senior software engineer reviewing code.".to_string()));
        assert_eq!(config.list_templates().len(), 3);
        
        // Delete the template
        let result = config.remove_template("code_review".to_string());
        assert!(result.is_ok());
        
        // Verify template was removed
        assert!(config.get_template("code_review").is_none());
        assert_eq!(config.list_templates().len(), 2);
        
        // Verify other templates still exist
        assert_eq!(config.get_template("documentation"), Some(&"You are a technical writer creating clear documentation.".to_string()));
        assert_eq!(config.get_template("debugging"), Some(&"You are an expert debugger helping to solve issues.".to_string()));
    }

    #[test]
    fn test_template_delete_nonexistent() {
        let mut config = create_config_with_templates();
        
        // Try to delete non-existent template
        let result = config.remove_template("nonexistent".to_string());
        assert!(result.is_err());
        
        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("Template 'nonexistent' not found"));
        
        // Verify no templates were affected
        assert_eq!(config.list_templates().len(), 3);
        assert_eq!(config.get_template("code_review"), Some(&"You are a senior software engineer reviewing code.".to_string()));
    }

    #[test]
    fn test_template_delete_all() {
        let mut config = create_config_with_templates();
        
        // Delete all templates one by one
        let result1 = config.remove_template("code_review".to_string());
        let result2 = config.remove_template("documentation".to_string());
        let result3 = config.remove_template("debugging".to_string());
        
        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert!(result3.is_ok());
        
        // Verify all templates are gone
        assert!(config.list_templates().is_empty());
        assert!(config.get_template("code_review").is_none());
        assert!(config.get_template("documentation").is_none());
        assert!(config.get_template("debugging").is_none());
    }

    #[test]
    fn test_template_delete_empty_name() {
        let mut config = create_config_with_templates();
        
        // Add template with empty name
        config.templates.insert("".to_string(), "Empty name template".to_string());
        assert_eq!(config.list_templates().len(), 4);
        
        // Delete template with empty name
        let result = config.remove_template("".to_string());
        assert!(result.is_ok());
        
        // Verify empty name template was removed
        assert!(config.get_template("").is_none());
        assert_eq!(config.list_templates().len(), 3);
    }

    #[test]
    fn test_template_delete_case_sensitive() {
        let mut config = create_config_with_templates();
        
        // Try to delete with different case
        let result = config.remove_template("CODE_REVIEW".to_string());
        assert!(result.is_err());
        
        // Verify original template still exists
        assert_eq!(config.get_template("code_review"), Some(&"You are a senior software engineer reviewing code.".to_string()));
        assert_eq!(config.list_templates().len(), 3);
    }
}

#[cfg(test)]
mod template_list_tests {
    use super::*;

    #[test]
    fn test_template_list_empty() {
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
        
        let templates = config.list_templates();
        assert!(templates.is_empty());
        assert_eq!(templates.len(), 0);
    }

    #[test]
    fn test_template_list_with_templates() {
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
        
        // Add some templates
        config.templates.insert("code_review".to_string(), "You are a senior software engineer reviewing code.".to_string());
        config.templates.insert("documentation".to_string(), "You are a technical writer creating clear documentation.".to_string());
        config.templates.insert("debugging".to_string(), "You are an expert debugger helping to solve issues.".to_string());
        
        let templates = config.list_templates();
        assert_eq!(templates.len(), 3);
        
        // Verify all templates are present
        assert_eq!(templates.get("code_review"), Some(&"You are a senior software engineer reviewing code.".to_string()));
        assert_eq!(templates.get("documentation"), Some(&"You are a technical writer creating clear documentation.".to_string()));
        assert_eq!(templates.get("debugging"), Some(&"You are an expert debugger helping to solve issues.".to_string()));
    }

    #[test]
    fn test_template_list_ordering() {
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
        
        // Add templates in specific order
        config.templates.insert("zebra".to_string(), "Zebra template".to_string());
        config.templates.insert("alpha".to_string(), "Alpha template".to_string());
        config.templates.insert("beta".to_string(), "Beta template".to_string());
        
        let templates = config.list_templates();
        assert_eq!(templates.len(), 3);
        
        // HashMap doesn't guarantee order, but all should be present
        assert!(templates.contains_key("zebra"));
        assert!(templates.contains_key("alpha"));
        assert!(templates.contains_key("beta"));
    }

    #[test]
    fn test_template_list_immutable() {
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
        config.templates.insert("test".to_string(), "Test template".to_string());
        
        let templates = config.list_templates();
        assert_eq!(templates.len(), 1);
        
        // The returned reference should be immutable
        // (This is enforced by the type system, but we can verify the content)
        assert_eq!(templates.get("test"), Some(&"Test template".to_string()));
    }
}

#[cfg(test)]
mod template_resolution_tests {
    use super::*;

    fn create_config_with_templates() -> Config {
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
        
        // Add test templates
        config.templates.insert("code_review".to_string(), "You are a senior software engineer reviewing code.".to_string());
        config.templates.insert("documentation".to_string(), "You are a technical writer creating clear documentation.".to_string());
        config.templates.insert("short".to_string(), "Brief".to_string());
        config.templates.insert("empty".to_string(), "".to_string());
        
        config
    }

    #[test]
    fn test_template_get_existing() {
        let config = create_config_with_templates();
        
        // Test getting existing templates
        assert_eq!(config.get_template("code_review"), Some(&"You are a senior software engineer reviewing code.".to_string()));
        assert_eq!(config.get_template("documentation"), Some(&"You are a technical writer creating clear documentation.".to_string()));
        assert_eq!(config.get_template("short"), Some(&"Brief".to_string()));
        assert_eq!(config.get_template("empty"), Some(&"".to_string()));
    }

    #[test]
    fn test_template_get_nonexistent() {
        let config = create_config_with_templates();
        
        // Test getting non-existent templates
        assert!(config.get_template("nonexistent").is_none());
        assert!(config.get_template("").is_none());
        assert!(config.get_template("CODE_REVIEW").is_none()); // Case sensitive
    }

    #[test]
    fn test_template_resolve_with_prefix() {
        let config = create_config_with_templates();
        
        // Test resolving templates with t: prefix
        assert_eq!(config.resolve_template_or_prompt("t:code_review"), "You are a senior software engineer reviewing code.");
        assert_eq!(config.resolve_template_or_prompt("t:documentation"), "You are a technical writer creating clear documentation.");
        assert_eq!(config.resolve_template_or_prompt("t:short"), "Brief");
        assert_eq!(config.resolve_template_or_prompt("t:empty"), "");
    }

    #[test]
    fn test_template_resolve_nonexistent() {
        let config = create_config_with_templates();
        
        // Test resolving non-existent templates - should return original input
        assert_eq!(config.resolve_template_or_prompt("t:nonexistent"), "t:nonexistent");
        assert_eq!(config.resolve_template_or_prompt("t:"), "t:");
        assert_eq!(config.resolve_template_or_prompt("t:CODE_REVIEW"), "t:CODE_REVIEW");
    }

    #[test]
    fn test_template_resolve_without_prefix() {
        let config = create_config_with_templates();
        
        // Test resolving without t: prefix - should return original input
        assert_eq!(config.resolve_template_or_prompt("code_review"), "code_review");
        assert_eq!(config.resolve_template_or_prompt("regular prompt"), "regular prompt");
        assert_eq!(config.resolve_template_or_prompt(""), "");
    }

    #[test]
    fn test_template_resolve_case_sensitivity() {
        let config = create_config_with_templates();
        
        // Templates should be case sensitive
        assert_eq!(config.resolve_template_or_prompt("t:code_review"), "You are a senior software engineer reviewing code.");
        assert_eq!(config.resolve_template_or_prompt("t:CODE_REVIEW"), "t:CODE_REVIEW"); // Not found
        assert_eq!(config.resolve_template_or_prompt("t:Code_Review"), "t:Code_Review"); // Not found
    }

    #[test]
    fn test_template_resolve_edge_cases() {
        let config = create_config_with_templates();
        
        // Test edge cases
        assert_eq!(config.resolve_template_or_prompt("t:"), "t:"); // Empty template name
        assert_eq!(config.resolve_template_or_prompt("tt:code_review"), "tt:code_review"); // Wrong prefix
        assert_eq!(config.resolve_template_or_prompt("t::code_review"), "t::code_review"); // Double colon
        assert_eq!(config.resolve_template_or_prompt("prefix t:code_review"), "prefix t:code_review"); // Not at start
    }

    #[test]
    fn test_template_resolve_special_characters() {
        let mut config = create_config_with_templates();
        
        // Add templates with special characters
        config.templates.insert("special-chars".to_string(), "Template with special characters: !@#$%^&*()".to_string());
        config.templates.insert("unicode".to_string(), "Template with unicode: 🚀 ✨ 🎉".to_string());
        config.templates.insert("quotes".to_string(), "Template with \"quotes\" and 'apostrophes'".to_string());
        
        // Test resolving templates with special content
        assert_eq!(config.resolve_template_or_prompt("t:special-chars"), "Template with special characters: !@#$%^&*()");
        assert_eq!(config.resolve_template_or_prompt("t:unicode"), "Template with unicode: 🚀 ✨ 🎉");
        assert_eq!(config.resolve_template_or_prompt("t:quotes"), "Template with \"quotes\" and 'apostrophes'");
    }
}

#[cfg(test)]
mod template_validation_tests {
    use super::*;

    #[test]
    fn test_template_name_validation() {
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
        
        // Various template names should be allowed
        let valid_names = vec![
            "normal-template",
            "template_with_underscores",
            "template.with.dots",
            "template123",
            "123template",
            "a",
            "very-long-template-name-with-many-dashes-and-words",
            "UPPERCASE",
            "MixedCase",
        ];
        
        for name in valid_names {
            let result = config.add_template(name.to_string(), "Test content".to_string());
            assert!(result.is_ok(), "Failed to add template with name: {}", name);
            
            let template = config.get_template(name);
            assert_eq!(template, Some(&"Test content".to_string()));
        }
    }

    #[test]
    fn test_template_content_validation() {
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
        
        // Various content types should be allowed
        let long_content = "Very long content ".repeat(100);
        let test_contents = vec![
            "",
            "Simple content",
            "Content with\nnewlines\nand\ttabs",
            "Content with special chars: !@#$%^&*()",
            "Content with unicode: 🚀 ✨ 🎉",
            "Content with \"quotes\" and 'apostrophes'",
            &long_content,
            "JSON-like: {\"key\": \"value\", \"number\": 123}",
            "Code-like: function test() { return 'hello'; }",
        ];
        
        for (i, content) in test_contents.iter().enumerate() {
            let name = format!("template_{}", i);
            let result = config.add_template(name.clone(), content.to_string());
            assert!(result.is_ok(), "Failed to add template with content: {}", content);
            
            let template = config.get_template(&name);
            assert_eq!(template, Some(&content.to_string()));
        }
    }

    #[test]
    fn test_template_name_uniqueness() {
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
        
        // Add template
        config.add_template("unique".to_string(), "First content".to_string()).unwrap();
        assert_eq!(config.get_template("unique"), Some(&"First content".to_string()));
        
        // Add template with same name - should overwrite
        config.add_template("unique".to_string(), "Second content".to_string()).unwrap();
        assert_eq!(config.get_template("unique"), Some(&"Second content".to_string()));
        
        // Should still have only one template
        assert_eq!(config.list_templates().len(), 1);
    }
}

#[cfg(test)]
mod template_integration_tests {
    use super::*;

    #[test]
    fn test_template_workflow_complete() {
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
        
        // Start with empty templates
        assert!(config.list_templates().is_empty());
        
        // Add first template
        let result = config.add_template("code_review".to_string(), "You are a senior software engineer reviewing code.".to_string());
        assert!(result.is_ok());
        assert_eq!(config.list_templates().len(), 1);
        
        // Add second template
        let result = config.add_template("documentation".to_string(), "You are a technical writer creating clear documentation.".to_string());
        assert!(result.is_ok());
        assert_eq!(config.list_templates().len(), 2);
        
        // Verify both templates exist
        assert_eq!(config.get_template("code_review"), Some(&"You are a senior software engineer reviewing code.".to_string()));
        assert_eq!(config.get_template("documentation"), Some(&"You are a technical writer creating clear documentation.".to_string()));
        
        // Test template resolution
        assert_eq!(config.resolve_template_or_prompt("t:code_review"), "You are a senior software engineer reviewing code.");
        assert_eq!(config.resolve_template_or_prompt("t:documentation"), "You are a technical writer creating clear documentation.");
        
        // Update existing template
        let result = config.add_template("code_review".to_string(), "You are an expert code reviewer with 10+ years experience.".to_string());
        assert!(result.is_ok());
        assert_eq!(config.list_templates().len(), 2); // Still 2 templates
        assert_eq!(config.get_template("code_review"), Some(&"You are an expert code reviewer with 10+ years experience.".to_string()));
        
        // Remove one template
        let result = config.remove_template("documentation".to_string());
        assert!(result.is_ok());
        assert_eq!(config.list_templates().len(), 1);
        assert!(config.get_template("documentation").is_none());
        assert_eq!(config.get_template("code_review"), Some(&"You are an expert code reviewer with 10+ years experience.".to_string()));
        
        // Remove last template
        let result = config.remove_template("code_review".to_string());
        assert!(result.is_ok());
        assert!(config.list_templates().is_empty());
        assert!(config.get_template("code_review").is_none());
    }

    #[test]
    fn test_template_persistence_simulation() {
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
        };
        
        // Add templates
        config1.add_template("code_review".to_string(), "You are a senior software engineer reviewing code.".to_string()).unwrap();
        config1.add_template("documentation".to_string(), "You are a technical writer creating clear documentation.".to_string()).unwrap();
        
        // Simulate serialization/deserialization by cloning the templates
        let mut config2 = Config {
            providers: HashMap::new(),
            default_provider: None,
            default_model: None,
            aliases: HashMap::new(),
            system_prompt: None,
            templates: HashMap::new(),
            max_tokens: None,
            temperature: None,
        };
        config2.templates = config1.templates.clone();
        
        // Verify templates persisted
        assert_eq!(config2.list_templates().len(), 2);
        assert_eq!(config2.get_template("code_review"), Some(&"You are a senior software engineer reviewing code.".to_string()));
        assert_eq!(config2.get_template("documentation"), Some(&"You are a technical writer creating clear documentation.".to_string()));
        
        // Test resolution still works
        assert_eq!(config2.resolve_template_or_prompt("t:code_review"), "You are a senior software engineer reviewing code.");
    }

    #[test]
    fn test_template_with_system_prompt_simulation() {
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
        
        // Add template
        config.add_template("assistant".to_string(), "You are a helpful AI assistant.".to_string()).unwrap();
        
        // Simulate setting system prompt to template reference
        config.system_prompt = Some("t:assistant".to_string());
        
        // Simulate resolving system prompt
        let resolved_system_prompt = if let Some(system_prompt) = &config.system_prompt {
            config.resolve_template_or_prompt(system_prompt)
        } else {
            "".to_string()
        };
        
        assert_eq!(resolved_system_prompt, "You are a helpful AI assistant.");
    }

    #[test]
    fn test_template_complex_scenarios() {
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
        
        // Add templates with various complexities
        config.add_template("simple".to_string(), "Simple template".to_string()).unwrap();
        config.add_template("multiline".to_string(), "Line 1\nLine 2\nLine 3".to_string()).unwrap();
        config.add_template("with_template_ref".to_string(), "This contains t:simple reference".to_string()).unwrap();
        config.add_template("json".to_string(), "{\"role\": \"assistant\", \"content\": \"Hello\"}".to_string()).unwrap();
        
        // Test various resolution scenarios
        assert_eq!(config.resolve_template_or_prompt("t:simple"), "Simple template");
        assert_eq!(config.resolve_template_or_prompt("t:multiline"), "Line 1\nLine 2\nLine 3");
        assert_eq!(config.resolve_template_or_prompt("t:with_template_ref"), "This contains t:simple reference"); // No recursive resolution
        assert_eq!(config.resolve_template_or_prompt("t:json"), "{\"role\": \"assistant\", \"content\": \"Hello\"}");
        
        // Test non-template inputs
        assert_eq!(config.resolve_template_or_prompt("regular input"), "regular input");
        assert_eq!(config.resolve_template_or_prompt("t:nonexistent"), "t:nonexistent");
    }
}
        