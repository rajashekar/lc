use clap::Parser;
use lc::cli::Cli;
use lc::config::{Config, ProviderConfig};
use lc::utils::{is_code_file, read_and_format_attachments, resolve_model_and_provider};
use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;

// Helper function to create a comprehensive test config
fn create_comprehensive_config() -> Config {
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
        ProviderConfig {
            endpoint: "https://api.openai.com".to_string(),
            models_path: "/v1/models".to_string(),
            chat_path: "/v1/chat/completions".to_string(),
            images_path: Some("/images/generations".to_string()),
            embeddings_path: Some("/embeddings".to_string()),
            api_key: Some("openai-key".to_string()),
            models: Vec::new(),
            headers: HashMap::new(),
            token_url: None,
            cached_token: None,
            auth_type: None,
            vars: std::collections::HashMap::new(),

            chat_templates: None,
            images_templates: None,
            embeddings_templates: None,
            models_templates: None,
            audio_path: None,
            speech_path: None,
            audio_templates: None,
            speech_templates: None,
        },
    );

    config.providers.insert(
        "anthropic".to_string(),
        ProviderConfig {
            endpoint: "https://api.anthropic.com".to_string(),
            models_path: "/v1/models".to_string(),
            chat_path: "/v1/messages".to_string(),
            images_path: Some("/images/generations".to_string()),
            embeddings_path: Some("/embeddings".to_string()),
            api_key: Some("anthropic-key".to_string()),
            models: Vec::new(),
            headers: HashMap::new(),
            token_url: None,
            cached_token: None,
            auth_type: None,
            vars: std::collections::HashMap::new(),

            chat_templates: None,
            images_templates: None,
            embeddings_templates: None,
            models_templates: None,
            audio_path: None,
            speech_path: None,
            audio_templates: None,
            speech_templates: None,
        },
    );

    // Set configuration defaults
    config.default_provider = Some("openai".to_string());
    config.default_model = Some("gpt-4".to_string());
    config.system_prompt = Some("You are a helpful assistant".to_string());
    config.max_tokens = Some(1000);
    config.temperature = Some(0.7);

    // Add aliases
    config
        .aliases
        .insert("fast".to_string(), "openai:gpt-3.5-turbo".to_string());
    config
        .aliases
        .insert("smart".to_string(), "anthropic:claude-3".to_string());

    // Add templates
    config
        .templates
        .insert("code".to_string(), "Explain this code: ".to_string());
    config.templates.insert(
        "review".to_string(),
        "Review this code for bugs: ".to_string(),
    );

    config
}

// Test modules for different aspects of prompt functionality
mod prompt_argument_parsing_tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_prompt_basic_parsing() {
        let args = vec!["lc", "Hello", "world"];
        let cli = Cli::try_parse_from(args).unwrap();

        assert_eq!(cli.prompt, vec!["Hello", "world"]);
        assert!(cli.provider.is_none());
        assert!(cli.model.is_none());
        assert!(cli.system_prompt.is_none());
        assert!(cli.max_tokens.is_none());
        assert!(cli.temperature.is_none());
        assert!(cli.attachments.is_empty());
        assert!(cli.tools.is_none());
        assert!(!cli.debug);
        assert!(cli.command.is_none());
    }

    #[test]
    fn test_prompt_with_provider_override() {
        let args = vec!["lc", "-p", "openai", "Hello", "world"];
        let cli = Cli::try_parse_from(args).unwrap();

        assert_eq!(cli.prompt, vec!["Hello", "world"]);
        assert_eq!(cli.provider, Some("openai".to_string()));
        assert!(cli.model.is_none());
    }

    #[test]
    fn test_prompt_with_model_override() {
        let args = vec!["lc", "-m", "gpt-4", "Hello", "world"];
        let cli = Cli::try_parse_from(args).unwrap();

        assert_eq!(cli.prompt, vec!["Hello", "world"]);
        assert!(cli.provider.is_none());
        assert_eq!(cli.model, Some("gpt-4".to_string()));
    }

    #[test]
    fn test_prompt_with_system_prompt_override() {
        let args = vec!["lc", "-s", "You are a helpful assistant", "Hello"];
        let cli = Cli::try_parse_from(args).unwrap();

        assert_eq!(cli.prompt, vec!["Hello"]);
        assert_eq!(
            cli.system_prompt,
            Some("You are a helpful assistant".to_string())
        );
    }

    #[test]
    fn test_prompt_with_max_tokens_override() {
        let args = vec!["lc", "--max-tokens", "2000", "Hello"];
        let cli = Cli::try_parse_from(args).unwrap();

        assert_eq!(cli.prompt, vec!["Hello"]);
        assert_eq!(cli.max_tokens, Some("2000".to_string()));
    }

    #[test]
    fn test_prompt_with_temperature_override() {
        let args = vec!["lc", "--temperature", "0.7", "Hello"];
        let cli = Cli::try_parse_from(args).unwrap();

        assert_eq!(cli.prompt, vec!["Hello"]);
        assert_eq!(cli.temperature, Some("0.7".to_string()));
    }

    #[test]
    fn test_prompt_with_attachments() {
        let args = vec!["lc", "-a", "file1.txt", "-a", "file2.py", "Hello"];
        let cli = Cli::try_parse_from(args).unwrap();

        assert_eq!(cli.prompt, vec!["Hello"]);
        assert_eq!(cli.attachments, vec!["file1.txt", "file2.py"]);
    }

    #[test]
    fn test_prompt_with_tools() {
        let args = vec!["lc", "-t", "server1,server2", "Hello"];
        let cli = Cli::try_parse_from(args).unwrap();

        assert_eq!(cli.prompt, vec!["Hello"]);
        assert_eq!(cli.tools, Some("server1,server2".to_string()));
    }

    #[test]
    fn test_prompt_with_debug_flag() {
        let args = vec!["lc", "-d", "Hello"];
        let cli = Cli::try_parse_from(args).unwrap();

        assert_eq!(cli.prompt, vec!["Hello"]);
        assert!(cli.debug);
    }

    #[test]
    fn test_prompt_with_all_options() {
        let args = vec![
            "lc",
            "-p",
            "openai",
            "-m",
            "gpt-4",
            "-s",
            "You are helpful",
            "--max-tokens",
            "1000",
            "--temperature",
            "0.8",
            "-a",
            "file.txt",
            "-t",
            "server1",
            "-d",
            "Hello",
            "world",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        assert_eq!(cli.prompt, vec!["Hello", "world"]);
        assert_eq!(cli.provider, Some("openai".to_string()));
        assert_eq!(cli.model, Some("gpt-4".to_string()));
        assert_eq!(cli.system_prompt, Some("You are helpful".to_string()));
        assert_eq!(cli.max_tokens, Some("1000".to_string()));
        assert_eq!(cli.temperature, Some("0.8".to_string()));
        assert_eq!(cli.attachments, vec!["file.txt"]);
        assert_eq!(cli.tools, Some("server1".to_string()));
        assert!(cli.debug);
    }

    #[test]
    fn test_prompt_empty_arguments() {
        let args = vec!["lc"];
        let cli = Cli::try_parse_from(args).unwrap();

        assert!(cli.prompt.is_empty());
        assert!(cli.command.is_none());
    }

    #[test]
    fn test_prompt_with_long_flags() {
        let args = vec![
            "lc",
            "--provider",
            "anthropic",
            "--model",
            "claude-3",
            "--system",
            "Be concise",
            "--attach",
            "code.rs",
            "--tools",
            "mcp-server",
            "--debug",
            "Explain",
            "this",
            "code",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        assert_eq!(cli.prompt, vec!["Explain", "this", "code"]);
        assert_eq!(cli.provider, Some("anthropic".to_string()));
        assert_eq!(cli.model, Some("claude-3".to_string()));
        assert_eq!(cli.system_prompt, Some("Be concise".to_string()));
        assert_eq!(cli.attachments, vec!["code.rs"]);
        assert_eq!(cli.tools, Some("mcp-server".to_string()));
        assert!(cli.debug);
    }
}

mod prompt_model_resolution_tests {
    use super::*;

    fn create_test_config() -> Config {
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
        config.providers.insert(
            "openai".to_string(),
            ProviderConfig {
                endpoint: "https://api.openai.com".to_string(),
                models_path: "/v1/models".to_string(),
                chat_path: "/v1/chat/completions".to_string(),
                images_path: Some("/images/generations".to_string()),
                embeddings_path: Some("/embeddings".to_string()),
                api_key: Some("test-key".to_string()),
                models: Vec::new(),
                headers: HashMap::new(),
                token_url: None,
                cached_token: None,
                auth_type: None,
                vars: std::collections::HashMap::new(),

                chat_templates: None,
                images_templates: None,
                embeddings_templates: None,
                models_templates: None,
                audio_path: None,
                speech_path: None,
                audio_templates: None,
                speech_templates: None,
            },
        );

        config.providers.insert(
            "anthropic".to_string(),
            ProviderConfig {
                endpoint: "https://api.anthropic.com".to_string(),
                models_path: "/v1/models".to_string(),
                chat_path: "/v1/messages".to_string(),
                images_path: Some("/images/generations".to_string()),
                embeddings_path: Some("/embeddings".to_string()),
                api_key: Some("test-key".to_string()),
                models: Vec::new(),
                headers: HashMap::new(),
                token_url: None,
                cached_token: None,
                auth_type: None,
                vars: std::collections::HashMap::new(),

                chat_templates: None,
                images_templates: None,
                embeddings_templates: None,
                models_templates: None,
                audio_path: None,
                speech_path: None,
                audio_templates: None,
                speech_templates: None,
            },
        );

        // Set defaults
        config.default_provider = Some("openai".to_string());
        config.default_model = Some("gpt-4".to_string());

        // Add aliases
        config
            .aliases
            .insert("fast".to_string(), "openai:gpt-3.5-turbo".to_string());
        config
            .aliases
            .insert("smart".to_string(), "anthropic:claude-3".to_string());

        config
    }

    #[test]
    fn test_resolve_model_with_defaults() {
        let config = create_test_config();

        let (provider, model) = resolve_model_and_provider(&config, None, None).unwrap();

        assert_eq!(provider, "openai");
        assert_eq!(model, "gpt-4");
    }

    #[test]
    fn test_resolve_model_with_provider_override() {
        let config = create_test_config();

        let (provider, model) =
            resolve_model_and_provider(&config, Some("anthropic".to_string()), None).unwrap();

        assert_eq!(provider, "anthropic");
        assert_eq!(model, "gpt-4"); // Uses default model
    }

    #[test]
    fn test_resolve_model_with_model_override() {
        let config = create_test_config();

        let (provider, model) =
            resolve_model_and_provider(&config, None, Some("claude-3".to_string())).unwrap();

        assert_eq!(provider, "openai"); // Uses default provider
        assert_eq!(model, "claude-3");
    }

    #[test]
    fn test_resolve_model_with_provider_model_format() {
        let config = create_test_config();

        let (provider, model) =
            resolve_model_and_provider(&config, None, Some("anthropic:claude-3".to_string()))
                .unwrap();

        assert_eq!(provider, "anthropic");
        assert_eq!(model, "claude-3");
    }

    #[test]
    fn test_resolve_model_with_alias() {
        let config = create_test_config();

        let (provider, model) =
            resolve_model_and_provider(&config, None, Some("fast".to_string())).unwrap();

        assert_eq!(provider, "openai");
        assert_eq!(model, "gpt-3.5-turbo");
    }

    #[test]
    fn test_resolve_model_provider_override_with_alias() {
        let config = create_test_config();

        // When provider is explicitly provided, model should be treated as literal
        let (provider, model) = resolve_model_and_provider(
            &config,
            Some("anthropic".to_string()),
            Some("fast".to_string()),
        )
        .unwrap();

        assert_eq!(provider, "anthropic");
        assert_eq!(model, "fast"); // Treated as literal, not resolved as alias
    }

    #[test]
    fn test_resolve_model_nonexistent_provider() {
        let config = create_test_config();

        let result = resolve_model_and_provider(&config, Some("nonexistent".to_string()), None);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Provider 'nonexistent' not found"));
    }

    #[test]
    fn test_resolve_model_no_default_provider() {
        let mut config = create_test_config();
        config.default_provider = None;

        let result = resolve_model_and_provider(&config, None, None);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No default provider configured"));
    }

    #[test]
    fn test_resolve_model_no_default_model() {
        let mut config = create_test_config();
        config.default_model = None;

        let result = resolve_model_and_provider(&config, None, None);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No default model configured"));
    }

    #[test]
    fn test_resolve_model_invalid_alias_format() {
        let mut config = create_test_config();
        config
            .aliases
            .insert("invalid".to_string(), "just-model-name".to_string());

        let result = resolve_model_and_provider(&config, None, Some("invalid".to_string()));

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid alias target format"));
    }
}

mod prompt_attachment_handling_tests {
    use super::*;

    #[test]
    fn test_read_attachments_empty() {
        let attachments: Vec<String> = vec![];
        let result = read_and_format_attachments(&attachments).unwrap();

        assert!(result.is_empty());
    }

    #[test]
    fn test_read_attachments_single_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "Hello, world!").unwrap();

        let attachments = vec![file_path.to_string_lossy().to_string()];
        let result = read_and_format_attachments(&attachments).unwrap();

        assert!(result.contains("=== File:"));
        assert!(result.contains("test.txt"));
        assert!(result.contains("Hello, world!"));
    }

    #[test]
    fn test_read_attachments_multiple_files() {
        let temp_dir = TempDir::new().unwrap();

        let file1_path = temp_dir.path().join("file1.txt");
        fs::write(&file1_path, "Content 1").unwrap();

        let file2_path = temp_dir.path().join("file2.txt");
        fs::write(&file2_path, "Content 2").unwrap();

        let attachments = vec![
            file1_path.to_string_lossy().to_string(),
            file2_path.to_string_lossy().to_string(),
        ];
        let result = read_and_format_attachments(&attachments).unwrap();

        assert!(result.contains("file1.txt"));
        assert!(result.contains("Content 1"));
        assert!(result.contains("file2.txt"));
        assert!(result.contains("Content 2"));
        assert!(result.matches("=== File:").count() == 2);
    }

    #[test]
    fn test_read_attachments_code_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        fs::write(&file_path, "fn main() {\n    println!(\"Hello\");\n}").unwrap();

        let attachments = vec![file_path.to_string_lossy().to_string()];
        let result = read_and_format_attachments(&attachments).unwrap();

        assert!(result.contains("```rs"));
        assert!(result.contains("fn main()"));
        assert!(result.contains("```"));
    }

    #[test]
    fn test_read_attachments_nonexistent_file() {
        let attachments = vec!["nonexistent.txt".to_string()];
        let result = read_and_format_attachments(&attachments);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to read file"));
    }

    #[test]
    fn test_is_code_file_detection() {
        assert!(is_code_file("rs"));
        assert!(is_code_file("py"));
        assert!(is_code_file("js"));
        assert!(is_code_file("ts"));
        assert!(is_code_file("java"));
        assert!(is_code_file("cpp"));
        assert!(is_code_file("json"));
        assert!(is_code_file("yaml"));
        assert!(is_code_file("toml"));

        assert!(!is_code_file("txt"));
        assert!(!is_code_file("md"));
        assert!(!is_code_file("pdf"));
        assert!(!is_code_file(""));
    }

    #[test]
    fn test_is_code_file_case_insensitive() {
        assert!(is_code_file("RS"));
        assert!(is_code_file("PY"));
        assert!(is_code_file("JS"));
        assert!(is_code_file("JSON"));
    }

    #[test]
    fn test_read_attachments_mixed_file_types() {
        let temp_dir = TempDir::new().unwrap();

        let code_file = temp_dir.path().join("script.py");
        fs::write(&code_file, "print('hello')").unwrap();

        let text_file = temp_dir.path().join("readme.txt");
        fs::write(&text_file, "This is a readme").unwrap();

        let attachments = vec![
            code_file.to_string_lossy().to_string(),
            text_file.to_string_lossy().to_string(),
        ];
        let result = read_and_format_attachments(&attachments).unwrap();

        // Code file should be wrapped in code blocks
        assert!(result.contains("```py"));
        assert!(result.contains("print('hello')"));

        // Text file should not be wrapped in code blocks
        assert!(result.contains("This is a readme"));
        assert!(!result.contains("```txt"));
    }
}

mod prompt_parameter_validation_tests {
    use super::*;

    #[test]
    fn test_max_tokens_parsing_valid() {
        // Test regular numbers
        assert_eq!(Config::parse_max_tokens("1000").unwrap(), 1000);
        assert_eq!(Config::parse_max_tokens("500").unwrap(), 500);

        // Test with 'k' suffix
        assert_eq!(Config::parse_max_tokens("2k").unwrap(), 2000);
        assert_eq!(Config::parse_max_tokens("1.5k").unwrap(), 1500);
        assert_eq!(Config::parse_max_tokens("0.5k").unwrap(), 500);
    }

    #[test]
    fn test_max_tokens_parsing_invalid() {
        assert!(Config::parse_max_tokens("invalid").is_err());
        assert!(Config::parse_max_tokens("").is_err());
        assert!(Config::parse_max_tokens("-100").is_err());
        assert!(Config::parse_max_tokens("1.5.5k").is_err());
    }

    #[test]
    fn test_temperature_parsing_valid() {
        assert_eq!(Config::parse_temperature("0.0").unwrap(), 0.0);
        assert_eq!(Config::parse_temperature("0.7").unwrap(), 0.7);
        assert_eq!(Config::parse_temperature("1.0").unwrap(), 1.0);
        assert_eq!(Config::parse_temperature("2.0").unwrap(), 2.0);
        assert_eq!(Config::parse_temperature("1").unwrap(), 1.0);
    }

    #[test]
    fn test_temperature_parsing_invalid() {
        assert!(Config::parse_temperature("invalid").is_err());
        assert!(Config::parse_temperature("").is_err());
        // Note: The current implementation doesn't validate range, only parsing
        // These tests check what the current implementation actually does
    }

    #[test]
    fn test_temperature_parsing_edge_cases() {
        // Test boundary values
        assert_eq!(Config::parse_temperature("0").unwrap(), 0.0);
        assert_eq!(Config::parse_temperature("2").unwrap(), 2.0);

        // Test precision
        assert_eq!(Config::parse_temperature("0.123").unwrap(), 0.123);
        assert_eq!(Config::parse_temperature("1.999").unwrap(), 1.999);

        // Test values outside typical range (current implementation allows these)
        assert_eq!(Config::parse_temperature("2.1").unwrap(), 2.1);
        assert_eq!(Config::parse_temperature("3.0").unwrap(), 3.0);
    }
}

mod prompt_tools_integration_tests {
    use super::*;

    #[test]
    fn test_tools_parsing_single_server() {
        let args = vec!["lc", "-t", "server1", "Hello"];
        let cli = Cli::try_parse_from(args).unwrap();

        assert_eq!(cli.tools, Some("server1".to_string()));
    }

    #[test]
    fn test_tools_parsing_multiple_servers() {
        let args = vec!["lc", "-t", "server1,server2,server3", "Hello"];
        let cli = Cli::try_parse_from(args).unwrap();

        assert_eq!(cli.tools, Some("server1,server2,server3".to_string()));
    }

    #[test]
    fn test_tools_parsing_with_spaces() {
        let args = vec!["lc", "-t", "server1, server2, server3", "Hello"];
        let cli = Cli::try_parse_from(args).unwrap();

        assert_eq!(cli.tools, Some("server1, server2, server3".to_string()));
    }

    #[test]
    fn test_tools_empty_string() {
        let args = vec!["lc", "-t", "", "Hello"];
        let cli = Cli::try_parse_from(args).unwrap();

        assert_eq!(cli.tools, Some("".to_string()));
    }
}

mod prompt_error_handling_tests {
    use super::*;

    #[test]
    fn test_resolve_model_with_invalid_provider_model_format() {
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

        let result =
            resolve_model_and_provider(&config, None, Some("provider:model:extra".to_string()));

        // Should treat as literal model name when format is invalid
        assert!(result.is_err()); // Will fail due to no default provider
    }

    #[test]
    fn test_resolve_model_with_colon_in_model_name() {
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
        config.providers.insert(
            "test".to_string(),
            ProviderConfig {
                endpoint: "https://test.com".to_string(),
                models_path: "/models".to_string(),
                chat_path: "/chat".to_string(),
                images_path: Some("/images/generations".to_string()),
                embeddings_path: Some("/embeddings".to_string()),
                api_key: Some("key".to_string()),
                models: Vec::new(),
                headers: HashMap::new(),
                token_url: None,
                cached_token: None,
                auth_type: None,
                vars: std::collections::HashMap::new(),

                chat_templates: None,
                images_templates: None,
                embeddings_templates: None,
                models_templates: None,
                audio_path: None,
                speech_path: None,
                audio_templates: None,
                speech_templates: None,
            },
        );
        config.default_provider = Some("test".to_string());

        // When provider is explicitly provided, model with colon should be treated literally
        let (provider, model) = resolve_model_and_provider(
            &config,
            Some("test".to_string()),
            Some("model:with:colons".to_string()),
        )
        .unwrap();

        assert_eq!(provider, "test");
        assert_eq!(model, "model:with:colons");
    }

    #[test]
    fn test_read_attachments_permission_error() {
        // This test would require creating a file with restricted permissions
        // For now, we'll test with a directory instead of a file
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().join("subdir");
        fs::create_dir(&dir_path).unwrap();

        let attachments = vec![dir_path.to_string_lossy().to_string()];
        let result = read_and_format_attachments(&attachments);

        // Should fail when trying to read a directory as a file
        assert!(result.is_err());
    }
}

mod prompt_integration_tests {
    use super::*;

    #[test]
    fn test_prompt_workflow_with_defaults() {
        let config = create_comprehensive_config();

        // Test that defaults are properly resolved
        let (provider, model) = resolve_model_and_provider(&config, None, None).unwrap();
        assert_eq!(provider, "openai");
        assert_eq!(model, "gpt-4");

        // Test that config values are accessible
        assert_eq!(
            config.system_prompt,
            Some("You are a helpful assistant".to_string())
        );
        assert_eq!(config.max_tokens, Some(1000));
        assert_eq!(config.temperature, Some(0.7));
    }

    #[test]
    fn test_prompt_workflow_with_overrides() {
        let config = create_comprehensive_config();

        // Test provider and model override
        let (provider, model) = resolve_model_and_provider(
            &config,
            Some("anthropic".to_string()),
            Some("claude-3".to_string()),
        )
        .unwrap();
        assert_eq!(provider, "anthropic");
        assert_eq!(model, "claude-3");
    }

    #[test]
    fn test_prompt_workflow_with_alias_resolution() {
        let config = create_comprehensive_config();

        // Test alias resolution
        let (provider, model) =
            resolve_model_and_provider(&config, None, Some("fast".to_string())).unwrap();
        assert_eq!(provider, "openai");
        assert_eq!(model, "gpt-3.5-turbo");
    }

    #[test]
    fn test_prompt_workflow_with_template_resolution() {
        let config = create_comprehensive_config();

        // Test template resolution
        let resolved = config.resolve_template_or_prompt("t:code");
        assert_eq!(resolved, "Explain this code: ");

        let resolved = config.resolve_template_or_prompt("Regular prompt");
        assert_eq!(resolved, "Regular prompt");
    }

    #[test]
    fn test_prompt_workflow_parameter_precedence() {
        let config = create_comprehensive_config();

        // Test that CLI overrides take precedence over config defaults
        let cli_max_tokens = Some("2000".to_string());
        let cli_temperature = Some("0.9".to_string());

        // Parse CLI overrides
        let parsed_max_tokens = cli_max_tokens
            .as_ref()
            .map(|s| Config::parse_max_tokens(s).unwrap());
        let parsed_temperature = cli_temperature
            .as_ref()
            .map(|s| Config::parse_temperature(s).unwrap());

        // CLI overrides should take precedence
        assert_eq!(parsed_max_tokens, Some(2000));
        assert_eq!(parsed_temperature, Some(0.9));

        // Config defaults should be different
        assert_eq!(config.max_tokens, Some(1000));
        assert_eq!(config.temperature, Some(0.7));
    }

    #[test]
    fn test_prompt_workflow_with_attachments_and_tools() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.py");
        fs::write(&file_path, "def hello():\n    print('world')").unwrap();

        let attachments = vec![file_path.to_string_lossy().to_string()];
        let result = read_and_format_attachments(&attachments).unwrap();

        // Should format code file properly
        assert!(result.contains("```py"));
        assert!(result.contains("def hello()"));
        assert!(result.contains("print('world')"));
    }

    #[test]
    fn test_prompt_complex_scenario() {
        let config = create_comprehensive_config();
        let temp_dir = TempDir::new().unwrap();

        // Create test files
        let rust_file = temp_dir.path().join("main.rs");
        fs::write(
            &rust_file,
            "fn main() {\n    println!(\"Hello, Rust!\");\n}",
        )
        .unwrap();

        let json_file = temp_dir.path().join("config.json");
        fs::write(
            &json_file,
            "{\n  \"name\": \"test\",\n  \"version\": \"1.0\"\n}",
        )
        .unwrap();

        // Test complex model resolution with alias
        let (provider, model) =
            resolve_model_and_provider(&config, None, Some("smart".to_string())).unwrap();
        assert_eq!(provider, "anthropic");
        assert_eq!(model, "claude-3");

        // Test attachment handling
        let attachments = vec![
            rust_file.to_string_lossy().to_string(),
            json_file.to_string_lossy().to_string(),
        ];
        let attachment_content = read_and_format_attachments(&attachments).unwrap();

        assert!(attachment_content.contains("```rs"));
        assert!(attachment_content.contains("fn main()"));
        assert!(attachment_content.contains("```json"));
        assert!(attachment_content.contains("\"name\": \"test\""));

        // Test template resolution
        let system_prompt = config.resolve_template_or_prompt("t:review");
        assert_eq!(system_prompt, "Review this code for bugs: ");
    }

    #[test]
    fn test_prompt_edge_cases() {
        let _config = create_comprehensive_config();

        // Test empty prompt
        let args = vec!["lc"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert!(cli.prompt.is_empty());

        // Test single word prompt
        let args = vec!["lc", "Hello"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert_eq!(cli.prompt, vec!["Hello"]);

        // Test prompt with special characters
        let args = vec!["lc", "Hello,", "world!", "@#$%"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert_eq!(cli.prompt, vec!["Hello,", "world!", "@#$%"]);
    }

    #[test]
    fn test_prompt_with_subcommand_conflict() {
        // Test that prompt arguments don't conflict with subcommands
        let args = vec!["lc", "providers", "list"];
        let cli = Cli::try_parse_from(args).unwrap();

        // Should parse as subcommand, not prompt
        assert!(cli.prompt.is_empty());
        assert!(cli.command.is_some());
    }
}

mod prompt_debug_mode_tests {
    use super::*;
    use lc::utils::set_debug_mode;

    #[test]
    fn test_debug_flag_parsing() {
        let args = vec!["lc", "-d", "Hello"];
        let cli = Cli::try_parse_from(args).unwrap();

        assert!(cli.debug);
        assert_eq!(cli.prompt, vec!["Hello"]);
    }

    #[test]
    fn test_debug_flag_long_form() {
        let args = vec!["lc", "--debug", "Hello"];
        let cli = Cli::try_parse_from(args).unwrap();

        assert!(cli.debug);
        assert_eq!(cli.prompt, vec!["Hello"]);
    }

    #[test]
    fn test_debug_mode_setting() {
        // Test debug mode can be set and retrieved
        set_debug_mode(true);
        // Note: We can't easily test the actual debug output without capturing stderr
        // but we can test that the function doesn't panic

        set_debug_mode(false);
        // Reset to false for other tests
    }
}

mod prompt_validation_edge_cases_tests {
    use super::*;

    #[test]
    fn test_max_tokens_k_suffix_variations() {
        // Test various 'k' suffix formats
        assert_eq!(Config::parse_max_tokens("1k").unwrap(), 1000);
        assert_eq!(Config::parse_max_tokens("2K").unwrap(), 2000);
        assert_eq!(Config::parse_max_tokens("0.1k").unwrap(), 100);
        assert_eq!(Config::parse_max_tokens("10.5k").unwrap(), 10500);
    }

    #[test]
    fn test_max_tokens_boundary_values() {
        assert_eq!(Config::parse_max_tokens("1").unwrap(), 1);
        assert_eq!(Config::parse_max_tokens("0").unwrap(), 0);
        assert_eq!(Config::parse_max_tokens("999999").unwrap(), 999999);
    }

    #[test]
    fn test_temperature_precision() {
        assert_eq!(Config::parse_temperature("0.001").unwrap(), 0.001);
        assert_eq!(Config::parse_temperature("1.999").unwrap(), 1.999);
        assert_eq!(Config::parse_temperature("0.5").unwrap(), 0.5);
    }

    #[test]
    fn test_temperature_scientific_notation() {
        // The current implementation actually accepts scientific notation
        // because it uses f32::parse which supports it
        assert_eq!(Config::parse_temperature("1e-3").unwrap(), 0.001);
        assert_eq!(Config::parse_temperature("1.5e2").unwrap(), 150.0);
    }
}

mod prompt_attachment_edge_cases_tests {
    use super::*;

    #[test]
    fn test_attachment_empty_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("empty.txt");
        fs::write(&file_path, "").unwrap();

        let attachments = vec![file_path.to_string_lossy().to_string()];
        let result = read_and_format_attachments(&attachments).unwrap();

        assert!(result.contains("=== File:"));
        assert!(result.contains("empty.txt"));
    }

    #[test]
    fn test_attachment_large_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("large.txt");
        let large_content = "x".repeat(10000);
        fs::write(&file_path, &large_content).unwrap();

        let attachments = vec![file_path.to_string_lossy().to_string()];
        let result = read_and_format_attachments(&attachments).unwrap();

        assert!(result.contains("=== File:"));
        assert!(result.contains(&large_content));
    }

    #[test]
    fn test_attachment_binary_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("binary.bin");
        let binary_data = vec![0u8, 1u8, 2u8, 255u8];
        fs::write(&file_path, &binary_data).unwrap();

        let attachments = vec![file_path.to_string_lossy().to_string()];
        let result = read_and_format_attachments(&attachments);

        // Should handle binary files (might contain invalid UTF-8)
        // The result depends on how Rust handles invalid UTF-8 in read_to_string
        // It might succeed with replacement characters or fail
        match result {
            Ok(content) => {
                assert!(content.contains("=== File:"));
            }
            Err(_) => {
                // Binary files might fail to read as UTF-8, which is acceptable
            }
        }
    }

    #[test]
    fn test_attachment_unicode_content() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("unicode.txt");
        let unicode_content = "Hello 世界 🌍 café naïve résumé";
        fs::write(&file_path, unicode_content).unwrap();

        let attachments = vec![file_path.to_string_lossy().to_string()];
        let result = read_and_format_attachments(&attachments).unwrap();

        assert!(result.contains("=== File:"));
        assert!(result.contains(unicode_content));
    }

    #[test]
    fn test_code_file_extensions_comprehensive() {
        // Test all supported code file extensions
        let code_extensions = vec![
            "rs",
            "py",
            "js",
            "ts",
            "java",
            "cpp",
            "c",
            "h",
            "hpp",
            "go",
            "rb",
            "php",
            "swift",
            "kt",
            "scala",
            "sh",
            "bash",
            "zsh",
            "fish",
            "ps1",
            "bat",
            "cmd",
            "html",
            "css",
            "scss",
            "sass",
            "less",
            "xml",
            "json",
            "yaml",
            "yml",
            "toml",
            "ini",
            "cfg",
            "conf",
            "sql",
            "r",
            "m",
            "mm",
            "pl",
            "pm",
            "lua",
            "vim",
            "dockerfile",
            "makefile",
            "cmake",
            "gradle",
            "maven",
        ];

        for ext in code_extensions {
            assert!(
                is_code_file(ext),
                "Extension '{}' should be recognized as code",
                ext
            );
        }
    }

    #[test]
    fn test_non_code_file_extensions() {
        let non_code_extensions = vec![
            "txt", "md", "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "png", "jpg", "jpeg",
            "gif", "bmp", "svg", "mp3", "mp4", "avi", "zip", "tar", "gz", "rar", "exe", "dll",
            "so", "dylib",
        ];

        for ext in non_code_extensions {
            assert!(
                !is_code_file(ext),
                "Extension '{}' should not be recognized as code",
                ext
            );
        }
    }
}

mod prompt_comprehensive_integration_tests {
    use super::*;

    #[test]
    fn test_full_prompt_workflow_simulation() {
        let config = create_comprehensive_config();
        let temp_dir = TempDir::new().unwrap();

        // Create multiple test files
        let py_file = temp_dir.path().join("script.py");
        fs::write(
            &py_file,
            "#!/usr/bin/env python3\ndef main():\n    print('Hello from Python')",
        )
        .unwrap();

        let rs_file = temp_dir.path().join("main.rs");
        fs::write(
            &rs_file,
            "fn main() {\n    println!(\"Hello from Rust\");\n}",
        )
        .unwrap();

        let txt_file = temp_dir.path().join("readme.txt");
        fs::write(&txt_file, "This is a readme file\nwith multiple lines").unwrap();

        // Test CLI parsing with all options
        let py_file_str = py_file.to_string_lossy().to_string();
        let rs_file_str = rs_file.to_string_lossy().to_string();
        let txt_file_str = txt_file.to_string_lossy().to_string();

        let args = vec![
            "lc",
            "-p",
            "anthropic",
            "-m",
            "claude-3",
            "-s",
            "t:code",
            "--max-tokens",
            "2k",
            "--temperature",
            "0.8",
            "-a",
            &py_file_str,
            "-a",
            &rs_file_str,
            "-a",
            &txt_file_str,
            "-t",
            "server1,server2",
            "-d",
            "Analyze",
            "these",
            "files",
        ];

        let cli = Cli::try_parse_from(args).unwrap();

        // Verify CLI parsing
        assert_eq!(cli.prompt, vec!["Analyze", "these", "files"]);
        assert_eq!(cli.provider, Some("anthropic".to_string()));
        assert_eq!(cli.model, Some("claude-3".to_string()));
        assert_eq!(cli.system_prompt, Some("t:code".to_string()));
        assert_eq!(cli.max_tokens, Some("2k".to_string()));
        assert_eq!(cli.temperature, Some("0.8".to_string()));
        assert_eq!(cli.attachments.len(), 3);
        assert_eq!(cli.tools, Some("server1,server2".to_string()));
        assert!(cli.debug);

        // Test model resolution
        let (provider, model) =
            resolve_model_and_provider(&config, cli.provider, cli.model).unwrap();
        assert_eq!(provider, "anthropic");
        assert_eq!(model, "claude-3");

        // Test parameter parsing
        let max_tokens = Config::parse_max_tokens(&cli.max_tokens.unwrap()).unwrap();
        let temperature = Config::parse_temperature(&cli.temperature.unwrap()).unwrap();
        assert_eq!(max_tokens, 2000);
        assert_eq!(temperature, 0.8);

        // Test system prompt resolution
        let system_prompt = config.resolve_template_or_prompt(&cli.system_prompt.unwrap());
        assert_eq!(system_prompt, "Explain this code: ");

        // Test attachment processing
        let attachment_content = read_and_format_attachments(&cli.attachments).unwrap();

        // Verify all files are included with proper formatting
        assert!(attachment_content.contains("```py"));
        assert!(attachment_content.contains("def main():"));
        assert!(attachment_content.contains("```rs"));
        assert!(attachment_content.contains("fn main()"));
        assert!(attachment_content.contains("This is a readme file"));
        assert!(!attachment_content.contains("```txt")); // txt files shouldn't be code-formatted

        // Verify file separators
        assert_eq!(attachment_content.matches("=== File:").count(), 3);
    }

    #[test]
    fn test_prompt_error_recovery_scenarios() {
        let config = create_comprehensive_config();

        // Test recovery from missing provider
        let result = resolve_model_and_provider(
            &config,
            Some("nonexistent".to_string()),
            Some("gpt-4".to_string()),
        );
        assert!(result.is_err());

        // Test recovery from invalid alias
        let mut config_with_bad_alias = config.clone();
        config_with_bad_alias
            .aliases
            .insert("bad".to_string(), "invalid-format".to_string());

        let result =
            resolve_model_and_provider(&config_with_bad_alias, None, Some("bad".to_string()));
        assert!(result.is_err());

        // Test recovery from missing files
        let bad_attachments = vec![
            "nonexistent1.txt".to_string(),
            "nonexistent2.py".to_string(),
        ];
        let result = read_and_format_attachments(&bad_attachments);
        assert!(result.is_err());
    }

    #[test]
    fn test_prompt_performance_edge_cases() {
        let config = create_comprehensive_config();

        // Test with many aliases
        let mut config_many_aliases = config.clone();
        for i in 0..100 {
            config_many_aliases
                .aliases
                .insert(format!("alias{}", i), format!("openai:model{}", i));
        }

        // Should still resolve quickly
        let (provider, model) =
            resolve_model_and_provider(&config_many_aliases, None, Some("alias50".to_string()))
                .unwrap();
        assert_eq!(provider, "openai");
        assert_eq!(model, "model50");

        // Test with many templates
        let mut config_many_templates = config.clone();
        for i in 0..100 {
            config_many_templates
                .templates
                .insert(format!("template{}", i), format!("Template content {}", i));
        }

        let resolved = config_many_templates.resolve_template_or_prompt("t:template50");
        assert_eq!(resolved, "Template content 50");
    }
}
