//! Integration tests for chat commands
//!
//! This module contains comprehensive integration tests for all chat-related
//! functionality, testing the underlying components as the CLI would use them.

mod common;

use chrono::Utc;
use lc::config::Config;
use lc::database::ChatEntry;
use lc::provider::{ChatRequest, Function, Message, Tool};
use std::collections::HashMap;
use tempfile::TempDir;

#[cfg(test)]
mod chat_model_resolution_tests {
    use super::*;

    fn create_test_config_with_providers() -> Config {
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

        // Add test providers
        config.providers.insert(
            "openai".to_string(),
            lc::config::ProviderConfig {
                endpoint: "https://api.openai.com/v1/chat/completions".to_string(),
                api_key: Some("sk-test123".to_string()),
                models: vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()],
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
            "anthropic".to_string(),
            lc::config::ProviderConfig {
                endpoint: "https://api.anthropic.com/v1/messages".to_string(),
                api_key: Some("sk-ant-test123".to_string()),
                models: vec!["claude-3-sonnet".to_string()],
                models_path: "/v1/models".to_string(),
                chat_path: "/v1/messages".to_string(),
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
    fn test_resolve_model_and_provider_with_defaults() {
        let config = create_test_config_with_providers();

        // Test using defaults
        let result = lc::utils::resolve_model_and_provider(&config, None, None);
        assert!(result.is_ok());
        let (provider, model) = result.unwrap();
        assert_eq!(provider, "openai");
        assert_eq!(model, "gpt-4");
    }

    #[test]
    fn test_resolve_model_and_provider_with_overrides() {
        let config = create_test_config_with_providers();

        // Test with provider override
        let result =
            lc::utils::resolve_model_and_provider(&config, Some("anthropic".to_string()), None);
        assert!(result.is_ok());
        let (provider, model) = result.unwrap();
        assert_eq!(provider, "anthropic");
        assert_eq!(model, "gpt-4"); // Uses default model

        // Test with model override
        let result =
            lc::utils::resolve_model_and_provider(&config, None, Some("gpt-3.5-turbo".to_string()));
        assert!(result.is_ok());
        let (provider, model) = result.unwrap();
        assert_eq!(provider, "openai"); // Uses default provider
        assert_eq!(model, "gpt-3.5-turbo");

        // Test with both overrides
        let result = lc::utils::resolve_model_and_provider(
            &config,
            Some("anthropic".to_string()),
            Some("claude-3-sonnet".to_string()),
        );
        assert!(result.is_ok());
        let (provider, model) = result.unwrap();
        assert_eq!(provider, "anthropic");
        assert_eq!(model, "claude-3-sonnet");
    }

    #[test]
    fn test_resolve_model_and_provider_with_provider_model_format() {
        let config = create_test_config_with_providers();

        // Test provider:model format
        let result = lc::utils::resolve_model_and_provider(
            &config,
            None,
            Some("anthropic:claude-3-sonnet".to_string()),
        );
        assert!(result.is_ok());
        let (provider, model) = result.unwrap();
        assert_eq!(provider, "anthropic");
        assert_eq!(model, "claude-3-sonnet");

        // Test with explicit provider override (should ignore provider in model string)
        let result = lc::utils::resolve_model_and_provider(
            &config,
            Some("openai".to_string()),
            Some("anthropic:claude-3-sonnet".to_string()),
        );
        assert!(result.is_ok());
        let (provider, model) = result.unwrap();
        assert_eq!(provider, "openai");
        assert_eq!(model, "anthropic:claude-3-sonnet"); // Treated as literal model name
    }

    #[test]
    fn test_resolve_model_and_provider_with_aliases() {
        let config = create_test_config_with_providers();

        // Test alias resolution
        let result = lc::utils::resolve_model_and_provider(&config, None, Some("gpt4".to_string()));
        assert!(result.is_ok());
        let (provider, model) = result.unwrap();
        assert_eq!(provider, "openai");
        assert_eq!(model, "gpt-4");

        let result =
            lc::utils::resolve_model_and_provider(&config, None, Some("claude".to_string()));
        assert!(result.is_ok());
        let (provider, model) = result.unwrap();
        assert_eq!(provider, "anthropic");
        assert_eq!(model, "claude-3-sonnet");
    }

    #[test]
    fn test_resolve_model_and_provider_error_cases() {
        let config = create_test_config_with_providers();

        // Test non-existent provider
        let result =
            lc::utils::resolve_model_and_provider(&config, Some("nonexistent".to_string()), None);
        assert!(result.is_err());

        // Test non-existent provider in model string
        let result = lc::utils::resolve_model_and_provider(
            &config,
            None,
            Some("nonexistent:model".to_string()),
        );
        assert!(result.is_err());

        // Test config without defaults
        let mut config_no_defaults = config.clone();
        config_no_defaults.default_provider = None;
        config_no_defaults.default_model = None;

        let result = lc::utils::resolve_model_and_provider(&config_no_defaults, None, None);
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod chat_message_handling_tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        // Test user message
        let user_msg = Message::user("Hello, world!".to_string());
        assert_eq!(user_msg.role, "user");
        if let lc::provider::MessageContent::Text { content } = &user_msg.content_type {
            assert_eq!(content, &Some("Hello, world!".to_string()));
        } else {
            panic!("Expected Text content");
        }
        assert!(user_msg.tool_calls.is_none());
        assert!(user_msg.tool_call_id.is_none());

        // Test assistant message
        let assistant_msg = Message::assistant("Hi there!".to_string());
        assert_eq!(assistant_msg.role, "assistant");
        if let lc::provider::MessageContent::Text { content } = &assistant_msg.content_type {
            assert_eq!(content, &Some("Hi there!".to_string()));
        } else {
            panic!("Expected Text content");
        }
        assert!(assistant_msg.tool_calls.is_none());
        assert!(assistant_msg.tool_call_id.is_none());

        // Test system message
        let system_msg = Message {
            role: "system".to_string(),
            content_type: lc::provider::MessageContent::Text {
                content: Some("You are a helpful assistant.".to_string()),
            },
            tool_calls: None,
            tool_call_id: None,
        };
        assert_eq!(system_msg.role, "system");
        if let lc::provider::MessageContent::Text { content } = &system_msg.content_type {
            assert_eq!(content, &Some("You are a helpful assistant.".to_string()));
        } else {
            panic!("Expected Text content");
        }
    }

    #[test]
    fn test_chat_request_creation() {
        let messages = vec![Message::user("What is 2+2?".to_string())];

        let request = ChatRequest {
            model: "gpt-4".to_string(),
            messages: messages.clone(),
            max_tokens: Some(100),
            temperature: Some(0.7),
            tools: None,
            stream: None,
        };

        assert_eq!(request.model, "gpt-4");
        assert_eq!(request.messages.len(), 1);
        assert_eq!(request.max_tokens, Some(100));
        assert_eq!(request.temperature, Some(0.7));
        assert!(request.tools.is_none());
    }

    #[test]
    fn test_chat_request_with_tools() {
        let tool = Tool {
            tool_type: "function".to_string(),
            function: Function {
                name: "get_weather".to_string(),
                description: "Get current weather".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "location": {
                            "type": "string",
                            "description": "The city name"
                        }
                    },
                    "required": ["location"]
                }),
            },
        };

        let request = ChatRequest {
            model: "gpt-4".to_string(),
            messages: vec![Message::user("What's the weather?".to_string())],
            max_tokens: Some(100),
            temperature: Some(0.7),
            tools: Some(vec![tool.clone()]),
            stream: None,
        };

        assert!(request.tools.is_some());
        let tools = request.tools.unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].function.name, "get_weather");
    }

    #[test]
    fn test_conversation_history_building() {
        let history = vec![
            ChatEntry {
                chat_id: "test-session".to_string(),
                model: "gpt-4".to_string(),
                question: "Hello".to_string(),
                response: "Hi there!".to_string(),
                timestamp: Utc::now(),
                input_tokens: Some(10),
                output_tokens: Some(5),
            },
            ChatEntry {
                chat_id: "test-session".to_string(),
                model: "gpt-4".to_string(),
                question: "How are you?".to_string(),
                response: "I'm doing well, thank you!".to_string(),
                timestamp: Utc::now(),
                input_tokens: Some(15),
                output_tokens: Some(8),
            },
        ];

        let mut messages = Vec::new();

        // Add system prompt
        messages.push(Message {
            role: "system".to_string(),
            content_type: lc::provider::MessageContent::Text {
                content: Some("You are a helpful assistant.".to_string()),
            },
            tool_calls: None,
            tool_call_id: None,
        });

        // Add conversation history
        for entry in &history {
            messages.push(Message::user(entry.question.clone()));
            messages.push(Message::assistant(entry.response.clone()));
        }

        // Add current prompt
        messages.push(Message::user("What's your name?".to_string()));

        assert_eq!(messages.len(), 6); // 1 system + 2*2 history + 1 current
        assert_eq!(messages[0].role, "system");
        assert_eq!(messages[1].role, "user");
        if let lc::provider::MessageContent::Text { content } = &messages[1].content_type {
            assert_eq!(content, &Some("Hello".to_string()));
        } else {
            panic!("Expected Text content");
        }
        assert_eq!(messages[2].role, "assistant");
        if let lc::provider::MessageContent::Text { content } = &messages[2].content_type {
            assert_eq!(content, &Some("Hi there!".to_string()));
        } else {
            panic!("Expected Text content");
        }
        assert_eq!(messages[5].role, "user");
        if let lc::provider::MessageContent::Text { content } = &messages[5].content_type {
            assert_eq!(content, &Some("What's your name?".to_string()));
        } else {
            panic!("Expected Text content");
        }
    }
}

#[cfg(test)]
mod chat_parameter_validation_tests {
    use super::*;

    #[test]
    fn test_max_tokens_parsing() {
        // Test valid max tokens values
        let valid_values = vec![
            ("100", 100),
            ("1k", 1000),
            ("2K", 2000),
            ("1.5k", 1500),
            ("4096", 4096),
        ];

        for (input, expected) in valid_values {
            let result = lc::config::Config::parse_max_tokens(input);
            assert!(result.is_ok(), "Failed to parse: {}", input);
            assert_eq!(result.unwrap(), expected);
        }

        // Test invalid max tokens values
        let invalid_values = vec!["-100", "abc", "1.5.5k", ""];

        for input in invalid_values {
            let result = lc::config::Config::parse_max_tokens(input);
            assert!(result.is_err(), "Should fail for: {}", input);
        }

        // Test edge case: 0 is actually valid (parsed successfully)
        let result = lc::config::Config::parse_max_tokens("0");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_temperature_parsing() {
        // Test valid temperature values
        let valid_values = vec![
            ("0.0", 0.0),
            ("0.5", 0.5),
            ("1.0", 1.0),
            ("1.5", 1.5),
            ("2.0", 2.0),
            ("0", 0.0),
            ("1", 1.0),
        ];

        for (input, expected) in valid_values {
            let result = lc::config::Config::parse_temperature(input);
            assert!(result.is_ok(), "Failed to parse: {}", input);
            assert!((result.unwrap() - expected).abs() < f32::EPSILON);
        }

        // Test invalid temperature values
        let invalid_values = vec!["abc", "", "1.0.0"];

        for input in invalid_values {
            let result = lc::config::Config::parse_temperature(input);
            assert!(result.is_err(), "Should fail for: {}", input);
        }

        // Test edge cases: negative and > 2.0 are actually valid (parsed successfully)
        let result = lc::config::Config::parse_temperature("-0.1");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), -0.1);

        let result = lc::config::Config::parse_temperature("2.1");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2.1);
    }

    #[test]
    fn test_parameter_precedence() {
        let config = Config {
            providers: HashMap::new(),
            default_provider: None,
            default_model: None,
            aliases: HashMap::new(),
            system_prompt: Some("Default system prompt".to_string()),
            templates: HashMap::new(),
            max_tokens: Some(1000),
            temperature: Some(0.5),
            stream: None,
        };

        // Test that CLI overrides take precedence over config
        let cli_max_tokens = Some("2000".to_string());
        let cli_temperature = Some("0.8".to_string());

        let final_max_tokens = if let Some(override_tokens) = &cli_max_tokens {
            Some(lc::config::Config::parse_max_tokens(override_tokens).unwrap())
        } else {
            config.max_tokens
        };

        let final_temperature = if let Some(override_temp) = &cli_temperature {
            Some(lc::config::Config::parse_temperature(override_temp).unwrap())
        } else {
            config.temperature
        };

        assert_eq!(final_max_tokens, Some(2000));
        assert_eq!(final_temperature, Some(0.8));

        // Test that config values are used when no CLI override
        let final_max_tokens_no_override = config.max_tokens;
        let final_temperature_no_override = config.temperature;

        assert_eq!(final_max_tokens_no_override, Some(1000));
        assert_eq!(final_temperature_no_override, Some(0.5));
    }
}

#[cfg(test)]
mod chat_template_resolution_tests {
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
            stream: None,
        };

        // Add templates
        config.templates.insert(
            "greeting".to_string(),
            "Hello! How can I help you today?".to_string(),
        );
        config.templates.insert(
            "code_review".to_string(),
            "Please review this code for best practices and potential issues.".to_string(),
        );
        config.templates.insert(
            "explain".to_string(),
            "Please explain the following concept in simple terms.".to_string(),
        );

        config
    }

    #[test]
    fn test_template_resolution() {
        let config = create_config_with_templates();

        // Test template resolution with t: prefix
        let resolved = config.resolve_template_or_prompt("t:greeting");
        assert_eq!(resolved, "Hello! How can I help you today?");

        let resolved = config.resolve_template_or_prompt("t:code_review");
        assert_eq!(
            resolved,
            "Please review this code for best practices and potential issues."
        );

        // Test non-existent template
        let resolved = config.resolve_template_or_prompt("t:nonexistent");
        assert_eq!(resolved, "t:nonexistent"); // Returns original if template not found

        // Test without t: prefix (should return as-is)
        let resolved = config.resolve_template_or_prompt("Just a regular prompt");
        assert_eq!(resolved, "Just a regular prompt");
    }

    #[test]
    fn test_system_prompt_template_resolution() {
        let mut config = create_config_with_templates();
        config.system_prompt = Some("t:greeting".to_string());

        // Test that system prompt resolves templates
        let resolved_system_prompt = config.system_prompt.as_ref().map(|system_prompt| config.resolve_template_or_prompt(system_prompt));

        assert_eq!(
            resolved_system_prompt,
            Some("Hello! How can I help you today?".to_string())
        );

        // Test with non-template system prompt
        config.system_prompt = Some("You are a helpful assistant.".to_string());
        let resolved_system_prompt = config.system_prompt.as_ref().map(|system_prompt| config.resolve_template_or_prompt(system_prompt));

        assert_eq!(
            resolved_system_prompt,
            Some("You are a helpful assistant.".to_string())
        );
    }

    #[test]
    fn test_template_management() {
        let mut config = create_config_with_templates();

        // Test adding template
        let result = config.add_template(
            "new_template".to_string(),
            "New template content".to_string(),
        );
        assert!(result.is_ok());
        assert_eq!(
            config.get_template("new_template"),
            Some(&"New template content".to_string())
        );

        // Test overwriting existing template
        let result = config.add_template("greeting".to_string(), "Updated greeting".to_string());
        assert!(result.is_ok());
        assert_eq!(
            config.get_template("greeting"),
            Some(&"Updated greeting".to_string())
        );

        // Test removing template
        let result = config.remove_template("greeting".to_string());
        assert!(result.is_ok());
        assert_eq!(config.get_template("greeting"), None);

        // Test removing non-existent template
        let result = config.remove_template("nonexistent".to_string());
        assert!(result.is_err());

        // Test listing templates
        let templates = config.list_templates();
        assert!(templates.len() >= 2); // At least code_review and explain should remain
        assert!(templates.iter().any(|(name, _)| name == "code_review"));
        assert!(templates.iter().any(|(name, _)| name == "explain"));
    }
}

#[cfg(test)]
mod chat_attachment_handling_tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_file_attachment_reading() {
        let temp_dir = TempDir::new().unwrap();

        // Create test files
        let file1_path = temp_dir.path().join("test1.txt");
        let file2_path = temp_dir.path().join("test2.py");

        fs::write(&file1_path, "This is a test file.").unwrap();
        fs::write(&file2_path, "def hello():\n    print('Hello, world!')").unwrap();

        let attachments = vec![
            file1_path.to_string_lossy().to_string(),
            file2_path.to_string_lossy().to_string(),
        ];

        let result = lc::utils::read_and_format_attachments(&attachments);
        assert!(result.is_ok());

        let formatted = result.unwrap();
        assert!(formatted.contains("=== File:"));
        assert!(formatted.contains("This is a test file."));
        assert!(formatted.contains("def hello():"));
        assert!(formatted.contains("```py")); // Should detect Python code
    }

    #[test]
    fn test_code_file_detection() {
        let code_extensions = vec![
            "rs", "py", "js", "ts", "java", "cpp", "c", "h", "go", "rb", "php", "swift", "kt",
            "html", "css", "json", "yaml", "toml",
        ];

        for ext in code_extensions {
            assert!(
                lc::utils::is_code_file(ext),
                "Extension '{}' should be detected as code",
                ext
            );
        }

        let non_code_extensions = vec!["txt", "md", "doc", "pdf", "png", "jpg"];

        for ext in non_code_extensions {
            assert!(
                !lc::utils::is_code_file(ext),
                "Extension '{}' should not be detected as code",
                ext
            );
        }
    }

    #[test]
    fn test_empty_attachments() {
        let result = lc::utils::read_and_format_attachments(&[]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
    }

    #[test]
    fn test_nonexistent_file_attachment() {
        let attachments = vec!["nonexistent_file.txt".to_string()];
        let result = lc::utils::read_and_format_attachments(&attachments);
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod chat_session_management_tests {
    use super::*;

    #[test]
    fn test_session_id_generation() {
        // Test that session IDs are unique
        let session1 = uuid::Uuid::new_v4().to_string();
        let session2 = uuid::Uuid::new_v4().to_string();

        assert_ne!(session1, session2);
        assert_eq!(session1.len(), 36); // Standard UUID length
        assert_eq!(session2.len(), 36);
    }

    #[test]
    fn test_chat_entry_creation() {
        let entry = ChatEntry {
            chat_id: "test-session".to_string(),
            model: "gpt-4".to_string(),
            question: "What is AI?".to_string(),
            response: "AI stands for Artificial Intelligence...".to_string(),
            timestamp: Utc::now(),
            input_tokens: Some(10),
            output_tokens: Some(25),
        };

        assert_eq!(entry.chat_id, "test-session");
        assert_eq!(entry.model, "gpt-4");
        assert_eq!(entry.question, "What is AI?");
        assert!(entry.response.starts_with("AI stands for"));
        assert_eq!(entry.input_tokens, Some(10));
        assert_eq!(entry.output_tokens, Some(25));
    }

    #[test]
    fn test_conversation_continuity() {
        // Test that conversation history maintains order
        let mut history = Vec::new();

        // Add entries in chronological order
        for i in 1..=3 {
            history.push(ChatEntry {
                chat_id: "test-session".to_string(),
                model: "gpt-4".to_string(),
                question: format!("Question {}", i),
                response: format!("Response {}", i),
                timestamp: Utc::now(),
                input_tokens: Some(10),
                output_tokens: Some(15),
            });
        }

        assert_eq!(history.len(), 3);
        assert_eq!(history[0].question, "Question 1");
        assert_eq!(history[1].question, "Question 2");
        assert_eq!(history[2].question, "Question 3");

        // Test that we can build conversation messages correctly
        let mut messages = Vec::new();
        for entry in &history {
            messages.push(Message::user(entry.question.clone()));
            messages.push(Message::assistant(entry.response.clone()));
        }

        assert_eq!(messages.len(), 6); // 3 pairs of user/assistant messages
        assert_eq!(messages[0].role, "user");
        assert_eq!(messages[1].role, "assistant");
        assert_eq!(messages[4].role, "user");
        assert_eq!(messages[5].role, "assistant");
    }
}

#[cfg(test)]
mod chat_error_handling_tests {
    use super::*;

    #[test]
    fn test_invalid_model_resolution() {
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

        // Test with no providers configured
        let result = lc::utils::resolve_model_and_provider(&config, None, None);
        assert!(result.is_err());

        // Test with invalid provider
        let result =
            lc::utils::resolve_model_and_provider(&config, Some("invalid".to_string()), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_alias_format() {
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
                endpoint: "https://api.openai.com/v1/chat/completions".to_string(),
                api_key: Some("sk-test123".to_string()),
                models: vec!["gpt-4".to_string()],
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
            lc::utils::resolve_model_and_provider(&config, None, Some("invalid_alias".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_api_key_error() {
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

        // Add provider without API key
        config.providers.insert(
            "openai".to_string(),
            lc::config::ProviderConfig {
                endpoint: "https://api.openai.com/v1/chat/completions".to_string(),
                api_key: None, // No API key
                models: vec!["gpt-4".to_string()],
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
    fn test_parameter_validation_errors() {
        // Test invalid max_tokens
        let invalid_max_tokens = vec!["-100", "abc", ""];
        for invalid in invalid_max_tokens {
            let result = lc::config::Config::parse_max_tokens(invalid);
            assert!(result.is_err(), "Should fail for max_tokens: {}", invalid);
        }

        // Test invalid temperature
        let invalid_temperatures = vec!["abc", ""];
        for invalid in invalid_temperatures {
            let result = lc::config::Config::parse_temperature(invalid);
            assert!(result.is_err(), "Should fail for temperature: {}", invalid);
        }
    }
}

#[cfg(test)]
mod chat_integration_tests {
    use super::*;

    #[test]
    fn test_complete_chat_workflow_simulation() {
        let config = Config {
            providers: HashMap::new(),
            default_provider: Some("openai".to_string()),
            default_model: Some("gpt-4".to_string()),
            aliases: HashMap::new(),
            system_prompt: Some("You are a helpful assistant.".to_string()),
            templates: HashMap::new(),
            max_tokens: Some(1000),
            temperature: Some(0.7),
            stream: None,
        };

        // Simulate chat workflow
        let _session_id = uuid::Uuid::new_v4().to_string();
        let prompt = "Hello, how are you?";

        // Test model resolution
        let result = lc::utils::resolve_model_and_provider(&config, None, None);
        assert!(result.is_ok());
        let (provider, model) = result.unwrap();
        assert_eq!(provider, "openai");
        assert_eq!(model, "gpt-4");

        // Test message building
        let mut messages = Vec::new();

        // Add system prompt
        if let Some(system_prompt) = &config.system_prompt {
            messages.push(Message {
                role: "system".to_string(),
                content_type: lc::provider::MessageContent::Text {
                    content: Some(system_prompt.clone()),
                },
                tool_calls: None,
                tool_call_id: None,
            });
        }

        // Add user prompt
        messages.push(Message::user(prompt.to_string()));

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, "system");
        assert_eq!(messages[1].role, "user");
        if let lc::provider::MessageContent::Text { content } = &messages[1].content_type {
            assert_eq!(content, &Some("Hello, how are you?".to_string()));
        } else {
            panic!("Expected Text content");
        }

        // Test chat request creation
        let request = ChatRequest {
            model: model.clone(),
            messages: messages.clone(),
            max_tokens: config.max_tokens,
            temperature: config.temperature,
            tools: None,
            stream: None,
        };

        assert_eq!(request.model, "gpt-4");
        assert_eq!(request.max_tokens, Some(1000));
        assert_eq!(request.temperature, Some(0.7));
    }

    #[test]
    fn test_chat_with_template_and_alias() {
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

        // Add provider
        config.providers.insert(
            "openai".to_string(),
            lc::config::ProviderConfig {
                endpoint: "https://api.openai.com/v1/chat/completions".to_string(),
                api_key: Some("sk-test123".to_string()),
                models: vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()],
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

        // Add alias and template
        config
            .aliases
            .insert("gpt4".to_string(), "openai:gpt-4".to_string());
        config.templates.insert(
            "code_review".to_string(),
            "Please review this code for best practices.".to_string(),
        );

        // Test using alias for model resolution
        let result = lc::utils::resolve_model_and_provider(&config, None, Some("gpt4".to_string()));
        assert!(result.is_ok());
        let (provider, model) = result.unwrap();
        assert_eq!(provider, "openai");
        assert_eq!(model, "gpt-4");

        // Test template resolution
        let prompt = "t:code_review";
        let resolved_prompt = config.resolve_template_or_prompt(prompt);
        assert_eq!(
            resolved_prompt,
            "Please review this code for best practices."
        );
    }

    #[test]
    fn test_chat_with_attachments_simulation() {
        let temp_dir = TempDir::new().unwrap();

        // Create test file
        let file_path = temp_dir.path().join("example.rs");
        std::fs::write(
            &file_path,
            "fn main() {\n    println!(\"Hello, world!\");\n}",
        )
        .unwrap();

        let attachments = vec![file_path.to_string_lossy().to_string()];

        // Test attachment reading
        let result = lc::utils::read_and_format_attachments(&attachments);
        assert!(result.is_ok());

        let formatted = result.unwrap();
        assert!(formatted.contains("=== File:"));
        assert!(formatted.contains("fn main()"));
        assert!(formatted.contains("```rs")); // Should detect Rust code

        // Test combining prompt with attachments
        let base_prompt = "Please review this code";
        let full_prompt = format!("{}\n\n{}", base_prompt, formatted);

        assert!(full_prompt.contains("Please review this code"));
        assert!(full_prompt.contains("fn main()"));
    }

    #[test]
    fn test_chat_parameter_override_workflow() {
        let config = Config {
            providers: HashMap::new(),
            default_provider: Some("openai".to_string()),
            default_model: Some("gpt-4".to_string()),
            aliases: HashMap::new(),
            system_prompt: Some("Default system prompt".to_string()),
            templates: HashMap::new(),
            max_tokens: Some(1000),
            temperature: Some(0.5),
            stream: None,
        };

        // Test CLI parameter overrides
        let cli_max_tokens = Some("2000".to_string());
        let cli_temperature = Some("0.8".to_string());
        let cli_system_prompt = Some("Custom system prompt".to_string());

        // Simulate parameter resolution
        let final_max_tokens = if let Some(override_tokens) = &cli_max_tokens {
            Some(lc::config::Config::parse_max_tokens(override_tokens).unwrap())
        } else {
            config.max_tokens
        };

        let final_temperature = if let Some(override_temp) = &cli_temperature {
            Some(lc::config::Config::parse_temperature(override_temp).unwrap())
        } else {
            config.temperature
        };

        let final_system_prompt = cli_system_prompt.as_ref().or(config.system_prompt.as_ref());

        assert_eq!(final_max_tokens, Some(2000));
        assert_eq!(final_temperature, Some(0.8));
        assert_eq!(
            final_system_prompt,
            Some(&"Custom system prompt".to_string())
        );
    }

    #[test]
    fn test_chat_error_recovery_workflow() {
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

        // Test error when no providers configured
        let result = lc::utils::resolve_model_and_provider(&config, None, None);
        assert!(result.is_err());

        // Add provider and test recovery
        config.providers.insert(
            "openai".to_string(),
            lc::config::ProviderConfig {
                endpoint: "https://api.openai.com/v1/chat/completions".to_string(),
                api_key: Some("sk-test123".to_string()),
                models: vec!["gpt-4".to_string()],
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

        config.default_provider = Some("openai".to_string());
        config.default_model = Some("gpt-4".to_string());

        // Now it should work
        let result = lc::utils::resolve_model_and_provider(&config, None, None);
        assert!(result.is_ok());
        let (provider, model) = result.unwrap();
        assert_eq!(provider, "openai");
        assert_eq!(model, "gpt-4");
    }
}

#[cfg(test)]
mod chat_session_continuation_tests {
    use super::*;

    #[test]
    fn test_continue_session_flag_parsing() {
        // Test that --continue flag is properly parsed
        use clap::Parser;

        // Test with --continue flag
        let args = vec!["lc", "--continue", "Hello", "world"];
        let cli = lc::cli::Cli::try_parse_from(args);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        assert!(cli.continue_session);
        assert_eq!(cli.prompt, vec!["Hello", "world"]);

        // Test with -c short flag
        let args = vec!["lc", "-c", "Hello", "world"];
        let cli = lc::cli::Cli::try_parse_from(args);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        assert!(cli.continue_session);
        assert_eq!(cli.prompt, vec!["Hello", "world"]);

        // Test without continue flag
        let args = vec!["lc", "Hello", "world"];
        let cli = lc::cli::Cli::try_parse_from(args);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        assert!(!cli.continue_session);
        assert_eq!(cli.prompt, vec!["Hello", "world"]);
    }

    #[test]
    fn test_chat_id_option_parsing() {
        use clap::Parser;

        // Test with --cid option
        let args = vec!["lc", "--cid", "test-session-123", "Hello", "world"];
        let cli = lc::cli::Cli::try_parse_from(args);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        assert_eq!(cli.chat_id, Some("test-session-123".to_string()));
        assert_eq!(cli.prompt, vec!["Hello", "world"]);

        // Test without --cid option
        let args = vec!["lc", "Hello", "world"];
        let cli = lc::cli::Cli::try_parse_from(args);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        assert_eq!(cli.chat_id, None);
        assert_eq!(cli.prompt, vec!["Hello", "world"]);
    }

    #[test]
    fn test_continue_and_cid_together() {
        use clap::Parser;

        // Test that both --continue and --cid can be used together
        let args = vec!["lc", "--continue", "--cid", "session-456", "Hello", "world"];
        let cli = lc::cli::Cli::try_parse_from(args);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        assert!(cli.continue_session);
        assert_eq!(cli.chat_id, Some("session-456".to_string()));
        assert_eq!(cli.prompt, vec!["Hello", "world"]);
    }

    #[test]
    fn test_chat_command_cid_option() {
        use clap::Parser;

        // Test chat subcommand with --cid option
        let args = vec!["lc", "chat", "-m", "gpt-4", "--cid", "chat-session-789"];
        let cli = lc::cli::Cli::try_parse_from(args);
        assert!(cli.is_ok());
        let cli = cli.unwrap();

        if let Some(lc::cli::Commands::Chat { model, cid, .. }) = cli.command {
            assert_eq!(model, Some("gpt-4".to_string()));
            assert_eq!(cid, Some("chat-session-789".to_string()));
        } else {
            panic!("Expected Chat command");
        }
    }

    #[test]
    fn test_session_id_validation() {
        // Test various session ID formats
        let valid_session_ids = vec![
            "simple-session",
            "session_with_underscores",
            "session-123-456",
            "a1b2c3d4-e5f6-7890-abcd-ef1234567890", // UUID format
            "very-long-session-id-with-many-characters-123456789",
        ];

        for session_id in valid_session_ids {
            use clap::Parser;
            let args = vec!["lc", "--cid", session_id, "test prompt"];
            let cli = lc::cli::Cli::try_parse_from(args);
            assert!(cli.is_ok(), "Session ID '{}' should be valid", session_id);
            let cli = cli.unwrap();
            assert_eq!(cli.chat_id, Some(session_id.to_string()));
        }
    }

    #[test]
    fn test_session_continuation_logic() {
        // Test the logic for determining session ID in chat scenarios

        // Case 1: --cid provided, should use that
        let provided_cid = Some("user-provided-session".to_string());
        let continue_flag = false;

        let session_id = provided_cid.clone().unwrap_or_else(|| {
            if continue_flag {
                // In real implementation, this would get from database
                "existing-session-from-db".to_string()
            } else {
                uuid::Uuid::new_v4().to_string()
            }
        });

        assert_eq!(session_id, "user-provided-session");

        // Case 2: --continue flag set, no --cid (would use existing session)
        let continue_flag = true;

        let session_id = if continue_flag {
            "existing-session-from-db".to_string()
        } else {
            uuid::Uuid::new_v4().to_string()
        };

        assert_eq!(session_id, "existing-session-from-db");

        // Case 3: Neither flag set (should generate new session)
        let continue_flag = false;

        let session_id = if continue_flag {
            "existing-session-from-db".to_string()
        } else {
            uuid::Uuid::new_v4().to_string()
        };

        // Should be a new UUID (36 characters)
        assert_eq!(session_id.len(), 36);
    }

    #[test]
    fn test_session_precedence_rules() {
        // Test precedence: --cid takes precedence over --continue

        struct SessionConfig {
            cid: Option<String>,
            continue_session: bool,
        }

        let test_cases = vec![
            // (cid, continue, expected_behavior)
            (
                Some("explicit-session".to_string()),
                false,
                "explicit-session",
            ),
            (
                Some("explicit-session".to_string()),
                true,
                "explicit-session",
            ), // cid wins
            (None, true, "continue-existing"),
            (None, false, "new-session"),
        ];

        for (cid, continue_flag, expected_behavior) in test_cases {
            let config = SessionConfig {
                cid: cid.clone(),
                continue_session: continue_flag,
            };

            let result_behavior = if let Some(session_id) = config.cid {
                session_id
            } else if config.continue_session {
                "continue-existing".to_string()
            } else {
                "new-session".to_string()
            };

            assert_eq!(
                result_behavior, expected_behavior,
                "Failed for cid={:?}, continue={}",
                cid, continue_flag
            );
        }
    }

    #[test]
    fn test_direct_prompt_with_session_options() {
        // Test that direct prompts work with session options
        use clap::Parser;

        // Test direct prompt with --continue
        let args = vec!["lc", "--continue", "-m", "gpt-4", "What", "is", "AI?"];
        let cli = lc::cli::Cli::try_parse_from(args);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        assert!(cli.continue_session);
        assert_eq!(cli.model, Some("gpt-4".to_string()));
        assert_eq!(cli.prompt, vec!["What", "is", "AI?"]);

        // Test direct prompt with --cid
        let args = vec![
            "lc",
            "--cid",
            "my-session",
            "-m",
            "gpt-4",
            "What",
            "is",
            "AI?",
        ];
        let cli = lc::cli::Cli::try_parse_from(args);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        assert_eq!(cli.chat_id, Some("my-session".to_string()));
        assert_eq!(cli.model, Some("gpt-4".to_string()));
        assert_eq!(cli.prompt, vec!["What", "is", "AI?"]);
    }

    #[test]
    fn test_session_options_with_other_flags() {
        use clap::Parser;

        // Test session options combined with other common flags
        let args = vec![
            "lc",
            "--continue",
            "--cid",
            "test-session",
            "-p",
            "openai",
            "-m",
            "gpt-4",
            "-s",
            "You are helpful",
            "--max-tokens",
            "1000",
            "--temperature",
            "0.7",
            "-a",
            "file.txt",
            "-t",
            "mcp-server",
            "-v",
            "vector-db",
            "Analyze",
            "this",
        ];

        let cli = lc::cli::Cli::try_parse_from(args);
        assert!(cli.is_ok());
        let cli = cli.unwrap();

        // Verify session options
        assert!(cli.continue_session);
        assert_eq!(cli.chat_id, Some("test-session".to_string()));

        // Verify other options are preserved
        assert_eq!(cli.provider, Some("openai".to_string()));
        assert_eq!(cli.model, Some("gpt-4".to_string()));
        assert_eq!(cli.system_prompt, Some("You are helpful".to_string()));
        assert_eq!(cli.max_tokens, Some("1000".to_string()));
        assert_eq!(cli.temperature, Some("0.7".to_string()));
        assert_eq!(cli.attachments, vec!["file.txt"]);
        assert_eq!(cli.tools, Some("mcp-server".to_string()));
        assert_eq!(cli.vectordb, Some("vector-db".to_string()));
        assert_eq!(cli.prompt, vec!["Analyze", "this"]);
    }

    #[test]
    fn test_session_options_error_cases() {
        use clap::Parser;

        // Test missing value for --cid (should fail)
        let args = vec!["lc", "--cid"];
        let cli = lc::cli::Cli::try_parse_from(args);
        assert!(cli.is_err(), "--cid without value should fail");

        // Test --continue with no additional arguments (should be ok)
        let args = vec!["lc", "--continue"];
        let cli = lc::cli::Cli::try_parse_from(args);
        assert!(cli.is_ok(), "--continue alone should be valid");
        let cli = cli.unwrap();
        assert!(cli.continue_session);
        assert!(cli.prompt.is_empty());
    }

    #[test]
    fn test_uuid_session_id_format() {
        // Test that generated session IDs follow UUID format
        let session_id = uuid::Uuid::new_v4().to_string();

        // UUID format: 8-4-4-4-12 characters separated by hyphens
        assert_eq!(session_id.len(), 36);
        assert_eq!(session_id.chars().filter(|&c| c == '-').count(), 4);

        // Test that multiple generated UUIDs are unique
        let mut session_ids = std::collections::HashSet::new();
        for _ in 0..100 {
            let id = uuid::Uuid::new_v4().to_string();
            assert!(session_ids.insert(id), "Generated duplicate session ID");
        }
    }

    #[test]
    fn test_session_workflow_simulation() {
        // Simulate a complete session workflow

        // Step 1: Start new session (no flags)
        let session_id_1 = uuid::Uuid::new_v4().to_string();
        assert_eq!(session_id_1.len(), 36);

        // Step 2: Continue session with --continue
        // In real implementation, this would retrieve the last session ID from database
        let last_session_id = session_id_1.clone();
        let continue_session_id = last_session_id; // Would come from DB
        assert_eq!(continue_session_id, session_id_1);

        // Step 3: Use specific session with --cid
        let specific_session_id = "user-chosen-session-123".to_string();
        assert_eq!(specific_session_id, "user-chosen-session-123");

        // Step 4: Verify session continuity
        let conversation_history =
            vec![("Hello", "Hi there!"), ("How are you?", "I'm doing well!")];

        // Each entry would be associated with the same session ID
        for (question, response) in conversation_history {
            let entry = ChatEntry {
                chat_id: session_id_1.clone(),
                model: "gpt-4".to_string(),
                question: question.to_string(),
                response: response.to_string(),
                timestamp: Utc::now(),
                input_tokens: Some(10),
                output_tokens: Some(15),
            };

            assert_eq!(entry.chat_id, session_id_1);
        }
    }
}
