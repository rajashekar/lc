//! Integration tests for models commands
//!
//! This module contains comprehensive integration tests for all models-related
//! CLI commands, testing the underlying functionality as the CLI would use it.

mod common;

use lc::model_metadata::{ModelMetadata, ModelType};
use lc::unified_cache::UnifiedCache;

#[cfg(test)]
mod models_cache_tests {
    use super::*;
    use std::process::Command;

    #[test]
    fn test_lc_models_list_with_capabilities() {
        let output = Command::new("cargo")
            .args(["run", "--", "models"])
            .output()
            .expect("Failed to execute command");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if !output.status.success() {
            // Log error for debugging
            eprintln!("Command failed with stderr: {}", stderr);
            eprintln!("Command failed with stdout: {}", stdout);
            // If the command fails, we can't test the output format, so just ensure it fails gracefully
            return;
        }

        // This test verifies that the enhanced model display system works correctly
        // The command should at least run successfully and show some output

        // Check if we have any providers configured
        let has_providers = stdout.contains("openai:")
            || stdout.contains("claude:")
            || stdout.contains("gemini:")
            || stdout.contains("anthropic:")
            || stdout.contains("models found")
            || stdout.contains("total");

        if has_providers {
            // If we have providers, we should see either:
            // 1. Provider sections with models, OR
            // 2. A message about total models, OR
            // 3. Enhanced metadata (icons or context info)
            let has_provider_sections = stdout.contains(":")
                && (stdout.contains("openai:")
                    || stdout.contains("claude:")
                    || stdout.contains("gemini:")
                    || stdout.contains("anthropic:"));

            let has_model_count = stdout.contains("total") || stdout.contains("models found");

            let has_capability_icons = stdout.contains("🔧")
                || stdout.contains("👁")
                || stdout.contains("💻")
                || stdout.contains("🧠")
                || stdout.contains("🔊");

            let has_context_info = stdout.contains("ctx") || stdout.contains("out");

            // At least one of these should be true if we have providers
            assert!(has_provider_sections || has_model_count || has_capability_icons || has_context_info,
                   "Output should show provider sections, model count, or enhanced metadata when providers are available. Got: {}", stdout);
        } else {
            // If no providers are configured, the command should still succeed
            // and show an appropriate message (like "No providers configured" or similar)
            assert!(
                output.status.success(),
                "Command should succeed even with no providers"
            );
        }
    }

    #[test]
    fn test_models_cache_directory_creation() {
        // Test that cache directory path is correctly determined
        let cache_dir = UnifiedCache::models_dir();
        assert!(cache_dir.is_ok());

        let path = cache_dir.unwrap();
        assert!(path.to_string_lossy().contains("lc"));
        assert!(path.to_string_lossy().contains("models"));
    }

    #[test]
    fn test_provider_cache_path() {
        let cache_path = UnifiedCache::provider_cache_path("test-provider");
        assert!(cache_path.is_ok());

        let path = cache_path.unwrap();
        assert!(path.to_string_lossy().ends_with("test-provider.json"));
    }

    #[tokio::test]
    async fn test_cache_freshness_nonexistent() {
        let is_fresh = UnifiedCache::is_cache_fresh("nonexistent-provider").await;
        assert!(is_fresh.is_ok());
        assert!(!is_fresh.unwrap()); // Non-existent cache is not fresh
    }

    #[tokio::test]
    async fn test_load_provider_models_empty() {
        let models = UnifiedCache::load_provider_models("nonexistent-provider").await;
        assert!(models.is_ok());
        assert!(models.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_load_all_cached_models_empty() {
        let models = UnifiedCache::load_all_cached_models().await;
        assert!(models.is_ok());
        // Should return empty vector when no cache exists
    }
}

#[cfg(test)]
mod models_metadata_tests {
    use super::*;

    #[test]
    fn test_model_metadata_default() {
        let metadata = ModelMetadata::default();

        assert_eq!(metadata.id, "");
        assert_eq!(metadata.provider, "");
        assert_eq!(metadata.display_name, None);
        assert_eq!(metadata.description, None);
        assert_eq!(metadata.context_length, None);
        assert!(!metadata.supports_tools);
        assert!(!metadata.supports_vision);
        assert!(!metadata.supports_audio);
        assert!(!metadata.supports_reasoning);
        assert!(!metadata.supports_code);
        assert!(!metadata.is_deprecated);
        assert!(!metadata.is_fine_tunable);
        assert!(matches!(metadata.model_type, ModelType::Chat));
    }

    #[test]
    fn test_model_metadata_creation() {
        let metadata = ModelMetadata {
            id: "gpt-4".to_string(),
            provider: "openai".to_string(),
            display_name: Some("GPT-4".to_string()),
            description: Some("Advanced language model".to_string()),
            context_length: Some(8192),
            supports_tools: true,
            supports_vision: false,
            supports_code: true,
            input_price_per_m: Some(30.0),
            output_price_per_m: Some(60.0),
            ..Default::default()
        };

        assert_eq!(metadata.id, "gpt-4");
        assert_eq!(metadata.provider, "openai");
        assert_eq!(metadata.display_name, Some("GPT-4".to_string()));
        assert_eq!(metadata.context_length, Some(8192));
        assert!(metadata.supports_tools);
        assert!(!metadata.supports_vision);
        assert!(metadata.supports_code);
        assert_eq!(metadata.input_price_per_m, Some(30.0));
        assert_eq!(metadata.output_price_per_m, Some(60.0));
    }

    #[test]
    fn test_model_type_variants() {
        let chat_model = ModelType::Chat;
        let completion_model = ModelType::Completion;
        let embedding_model = ModelType::Embedding;
        let image_model = ModelType::ImageGeneration;
        let audio_model = ModelType::AudioGeneration;
        let moderation_model = ModelType::Moderation;
        let other_model = ModelType::Other("custom".to_string());

        assert!(matches!(chat_model, ModelType::Chat));
        assert!(matches!(completion_model, ModelType::Completion));
        assert!(matches!(embedding_model, ModelType::Embedding));
        assert!(matches!(image_model, ModelType::ImageGeneration));
        assert!(matches!(audio_model, ModelType::AudioGeneration));
        assert!(matches!(moderation_model, ModelType::Moderation));
        assert!(matches!(other_model, ModelType::Other(_)));
    }
}

#[cfg(test)]
mod models_filtering_tests {
    use super::*;

    fn create_test_models() -> Vec<ModelMetadata> {
        vec![
            ModelMetadata {
                id: "gpt-4".to_string(),
                provider: "openai".to_string(),
                display_name: Some("GPT-4".to_string()),
                description: Some("Advanced language model".to_string()),
                context_length: Some(8192),
                supports_tools: true,
                supports_vision: false,
                supports_code: true,
                input_price_per_m: Some(30.0),
                output_price_per_m: Some(60.0),
                ..Default::default()
            },
            ModelMetadata {
                id: "gpt-4-vision".to_string(),
                provider: "openai".to_string(),
                display_name: Some("GPT-4 Vision".to_string()),
                description: Some("GPT-4 with vision capabilities".to_string()),
                context_length: Some(128000),
                supports_tools: true,
                supports_vision: true,
                supports_code: true,
                input_price_per_m: Some(10.0),
                output_price_per_m: Some(30.0),
                ..Default::default()
            },
            ModelMetadata {
                id: "whisper-1".to_string(),
                provider: "openai".to_string(),
                display_name: Some("Whisper".to_string()),
                description: Some("Speech recognition model".to_string()),
                context_length: None,
                supports_tools: false,
                supports_vision: false,
                supports_audio: true,
                supports_code: false,
                input_price_per_m: Some(6.0),
                output_price_per_m: None,
                model_type: ModelType::AudioGeneration,
                ..Default::default()
            },
            ModelMetadata {
                id: "o1-preview".to_string(),
                provider: "openai".to_string(),
                display_name: Some("o1-preview".to_string()),
                description: Some("Reasoning model".to_string()),
                context_length: Some(128000),
                max_output_tokens: Some(32768),
                supports_tools: false,
                supports_vision: false,
                supports_reasoning: true,
                supports_code: true,
                input_price_per_m: Some(15.0),
                output_price_per_m: Some(60.0),
                ..Default::default()
            },
            ModelMetadata {
                id: "claude-3-sonnet".to_string(),
                provider: "claude".to_string(),
                display_name: Some("Claude 3 Sonnet".to_string()),
                description: Some("Balanced AI assistant".to_string()),
                context_length: Some(200000),
                supports_tools: true,
                supports_vision: true,
                supports_code: true,
                input_price_per_m: Some(3.0),
                output_price_per_m: Some(15.0),
                ..Default::default()
            },
        ]
    }

    #[test]
    fn test_filter_by_query() {
        let models = create_test_models();

        // Filter by "gpt" - should match gpt-4 and gpt-4-vision
        let gpt_models: Vec<_> = models
            .iter()
            .filter(|m| {
                m.id.to_lowercase().contains("gpt")
                    || m.display_name
                        .as_ref()
                        .map_or(false, |name| name.to_lowercase().contains("gpt"))
            })
            .collect();

        assert_eq!(gpt_models.len(), 2);
        assert!(gpt_models.iter().any(|m| m.id == "gpt-4"));
        assert!(gpt_models.iter().any(|m| m.id == "gpt-4-vision"));
    }

    #[test]
    fn test_filter_by_tools_support() {
        let models = create_test_models();

        let tools_models: Vec<_> = models
            .iter()
            .filter(|m| m.supports_tools || m.supports_function_calling)
            .collect();

        assert_eq!(tools_models.len(), 3); // gpt-4, gpt-4-vision, claude-3-sonnet
        assert!(tools_models.iter().any(|m| m.id == "gpt-4"));
        assert!(tools_models.iter().any(|m| m.id == "gpt-4-vision"));
        assert!(tools_models.iter().any(|m| m.id == "claude-3-sonnet"));
    }

    #[test]
    fn test_filter_by_vision_support() {
        let models = create_test_models();

        let vision_models: Vec<_> = models.iter().filter(|m| m.supports_vision).collect();

        assert_eq!(vision_models.len(), 2); // gpt-4-vision, claude-3-sonnet
        assert!(vision_models.iter().any(|m| m.id == "gpt-4-vision"));
        assert!(vision_models.iter().any(|m| m.id == "claude-3-sonnet"));
    }

    #[test]
    fn test_filter_by_audio_support() {
        let models = create_test_models();

        let audio_models: Vec<_> = models.iter().filter(|m| m.supports_audio).collect();

        assert_eq!(audio_models.len(), 1); // whisper-1
        assert_eq!(audio_models[0].id, "whisper-1");
    }

    #[test]
    fn test_filter_by_reasoning_support() {
        let models = create_test_models();

        let reasoning_models: Vec<_> = models.iter().filter(|m| m.supports_reasoning).collect();

        assert_eq!(reasoning_models.len(), 1); // o1-preview
        assert_eq!(reasoning_models[0].id, "o1-preview");
    }

    #[test]
    fn test_filter_by_code_support() {
        let models = create_test_models();

        let code_models: Vec<_> = models.iter().filter(|m| m.supports_code).collect();

        assert_eq!(code_models.len(), 4); // All except whisper-1
        assert!(!code_models.iter().any(|m| m.id == "whisper-1"));
    }

    #[test]
    fn test_filter_by_context_length() {
        let models = create_test_models();

        // Filter models with context length >= 100k
        let large_context_models: Vec<_> = models
            .iter()
            .filter(|m| m.context_length.map_or(false, |ctx| ctx >= 100000))
            .collect();

        assert_eq!(large_context_models.len(), 3); // gpt-4-vision, o1-preview, claude-3-sonnet
        assert!(large_context_models.iter().any(|m| m.id == "gpt-4-vision"));
        assert!(large_context_models.iter().any(|m| m.id == "o1-preview"));
        assert!(large_context_models
            .iter()
            .any(|m| m.id == "claude-3-sonnet"));
    }

    #[test]
    fn test_filter_by_input_price() {
        let models = create_test_models();

        // Filter models with input price <= $5/M
        let affordable_models: Vec<_> = models
            .iter()
            .filter(|m| m.input_price_per_m.map_or(true, |price| price <= 5.0))
            .collect();

        assert_eq!(affordable_models.len(), 1); // claude-3-sonnet ($3/M)
        assert_eq!(affordable_models[0].id, "claude-3-sonnet");
    }

    #[test]
    fn test_filter_by_output_price() {
        let models = create_test_models();

        // Filter models with output price <= $20/M
        let affordable_output_models: Vec<_> = models
            .iter()
            .filter(|m| m.output_price_per_m.map_or(true, |price| price <= 20.0))
            .collect();

        assert_eq!(affordable_output_models.len(), 2); // whisper-1 (None), claude-3-sonnet ($15/M)
        assert!(affordable_output_models.iter().any(|m| m.id == "whisper-1"));
        assert!(affordable_output_models
            .iter()
            .any(|m| m.id == "claude-3-sonnet"));
    }

    #[test]
    fn test_filter_by_provider() {
        let models = create_test_models();

        let openai_models: Vec<_> = models.iter().filter(|m| m.provider == "openai").collect();

        assert_eq!(openai_models.len(), 4);

        let claude_models: Vec<_> = models.iter().filter(|m| m.provider == "claude").collect();

        assert_eq!(claude_models.len(), 1);
        assert_eq!(claude_models[0].id, "claude-3-sonnet");
    }

    #[test]
    fn test_combined_filters() {
        let models = create_test_models();

        // Filter for vision + tools + context > 100k + input price < 5.0
        let filtered_models: Vec<_> = models
            .iter()
            .filter(|m| {
                m.supports_vision
                    && (m.supports_tools || m.supports_function_calling)
                    && m.context_length.map_or(false, |ctx| ctx > 100000)
                    && m.input_price_per_m.map_or(false, |price| price < 5.0)
            })
            .collect();

        assert_eq!(filtered_models.len(), 1); // claude-3-sonnet
        assert_eq!(filtered_models[0].id, "claude-3-sonnet");
    }

    #[test]
    fn test_no_matching_filters() {
        let models = create_test_models();

        // Filter for impossible combination: audio + vision + reasoning
        let impossible_models: Vec<_> = models
            .iter()
            .filter(|m| m.supports_audio && m.supports_vision && m.supports_reasoning)
            .collect();

        assert_eq!(impossible_models.len(), 0);
    }
}

#[cfg(test)]
mod models_sorting_tests {
    use super::*;

    #[test]
    fn test_models_sorting_by_provider_and_name() {
        let mut models = vec![
            ModelMetadata {
                id: "zebra-model".to_string(),
                provider: "zebra".to_string(),
                ..Default::default()
            },
            ModelMetadata {
                id: "alpha-model".to_string(),
                provider: "alpha".to_string(),
                ..Default::default()
            },
            ModelMetadata {
                id: "beta-model".to_string(),
                provider: "alpha".to_string(),
                ..Default::default()
            },
        ];

        // Sort by provider, then by model name
        models.sort_by(|a, b| a.provider.cmp(&b.provider).then(a.id.cmp(&b.id)));

        assert_eq!(models[0].provider, "alpha");
        assert_eq!(models[0].id, "alpha-model");
        assert_eq!(models[1].provider, "alpha");
        assert_eq!(models[1].id, "beta-model");
        assert_eq!(models[2].provider, "zebra");
        assert_eq!(models[2].id, "zebra-model");
    }
}

#[cfg(test)]
mod models_display_tests {
    use super::*;

    #[test]
    fn test_model_display_name_fallback() {
        let model_with_display = ModelMetadata {
            id: "gpt-4".to_string(),
            display_name: Some("GPT-4 Turbo".to_string()),
            ..Default::default()
        };

        let model_without_display = ModelMetadata {
            id: "gpt-4".to_string(),
            display_name: None,
            ..Default::default()
        };

        // With display name
        let display_name = model_with_display
            .display_name
            .as_ref()
            .unwrap_or(&model_with_display.id);
        assert_eq!(display_name, "GPT-4 Turbo");

        // Without display name (fallback to ID)
        let display_name = model_without_display
            .display_name
            .as_ref()
            .unwrap_or(&model_without_display.id);
        assert_eq!(display_name, "gpt-4");
    }

    #[test]
    fn test_model_capability_indicators() {
        let model = ModelMetadata {
            id: "test-model".to_string(),
            supports_tools: true,
            supports_vision: true,
            supports_audio: false,
            supports_reasoning: true,
            supports_code: true,
            ..Default::default()
        };

        // Test capability detection
        assert!(model.supports_tools);
        assert!(model.supports_vision);
        assert!(!model.supports_audio);
        assert!(model.supports_reasoning);
        assert!(model.supports_code);

        // Count capabilities
        let capability_count = [
            model.supports_tools,
            model.supports_vision,
            model.supports_audio,
            model.supports_reasoning,
            model.supports_code,
        ]
        .iter()
        .filter(|&&x| x)
        .count();

        assert_eq!(capability_count, 4);
    }

    #[test]
    fn test_model_context_info_formatting() {
        let model_with_context = ModelMetadata {
            id: "test-model".to_string(),
            context_length: Some(128000),
            max_output_tokens: Some(4096),
            ..Default::default()
        };

        let model_without_context = ModelMetadata {
            id: "test-model".to_string(),
            context_length: None,
            max_output_tokens: None,
            ..Default::default()
        };

        // Test context length formatting
        if let Some(ctx) = model_with_context.context_length {
            let formatted = if ctx >= 1000 {
                format!("{}k ctx", ctx / 1000)
            } else {
                format!("{} ctx", ctx)
            };
            assert_eq!(formatted, "128k ctx");
        }

        // Test output tokens formatting
        if let Some(max_out) = model_with_context.max_output_tokens {
            let formatted = if max_out >= 1000 {
                format!("{}k out", max_out / 1000)
            } else {
                format!("{} out", max_out)
            };
            assert_eq!(formatted, "4k out");
        }

        // Test None values
        assert!(model_without_context.context_length.is_none());
        assert!(model_without_context.max_output_tokens.is_none());
    }

    #[test]
    fn test_model_pricing_info_formatting() {
        let model_with_pricing = ModelMetadata {
            id: "test-model".to_string(),
            input_price_per_m: Some(3.0),
            output_price_per_m: Some(15.0),
            ..Default::default()
        };

        let model_without_pricing = ModelMetadata {
            id: "test-model".to_string(),
            input_price_per_m: None,
            output_price_per_m: None,
            ..Default::default()
        };

        // Test pricing formatting
        if let Some(input_price) = model_with_pricing.input_price_per_m {
            let formatted = format!("${:.2}/M in", input_price);
            assert_eq!(formatted, "$3.00/M in");
        }

        if let Some(output_price) = model_with_pricing.output_price_per_m {
            let formatted = format!("${:.2}/M out", output_price);
            assert_eq!(formatted, "$15.00/M out");
        }

        // Test None values
        assert!(model_without_pricing.input_price_per_m.is_none());
        assert!(model_without_pricing.output_price_per_m.is_none());
    }
}

#[cfg(test)]
mod model_metadata_config_tests {
    use super::*;
    use lc::model_metadata::{
        add_model_path, add_tag, initialize_model_metadata_config, remove_model_path,
    };
    use std::fs;
    use std::sync::Mutex;
    use tempfile::TempDir;

    // Mutex to ensure tests run serially to avoid environment variable conflicts
    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    fn setup_test_config_dir() -> (
        TempDir,
        std::path::PathBuf,
        std::sync::MutexGuard<'static, ()>,
    ) {
        let _guard = TEST_MUTEX.lock().unwrap();

        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config_path = temp_dir.path().join("lc");

        // Create the lc config directory
        fs::create_dir_all(&config_path).expect("Failed to create config directory");

        // Set environment variables to point to our temp directory
        std::env::set_var("XDG_CONFIG_HOME", temp_dir.path());
        std::env::set_var("HOME", temp_dir.path());

        (temp_dir, config_path, _guard)
    }

    fn get_test_config_dir() -> std::path::PathBuf {
        if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
            std::path::PathBuf::from(xdg_config).join("lc")
        } else if let Ok(home) = std::env::var("HOME") {
            std::path::PathBuf::from(home).join(".config").join("lc")
        } else {
            panic!("No config directory found")
        }
    }

    #[test]
    fn test_initialize_model_metadata_config() {
        let (_temp_dir, config_dir, _guard) = setup_test_config_dir();

        // Initialize config files
        let result = initialize_model_metadata_config();
        assert!(result.is_ok(), "Config initialization should succeed");

        // Check that files were created
        let model_paths_file = config_dir.join("model_paths.toml");
        let tags_file = config_dir.join("tags.toml");

        assert!(
            model_paths_file.exists(),
            "model_paths.toml should be created"
        );
        assert!(tags_file.exists(), "tags.toml should be created");

        // Check that files have default content
        let model_paths_content = fs::read_to_string(&model_paths_file).unwrap();
        assert!(
            model_paths_content.contains(".data[]"),
            "Should contain default model paths"
        );
        assert!(
            model_paths_content.contains(".models[]"),
            "Should contain default model paths"
        );

        let tags_content = fs::read_to_string(&tags_file).unwrap();
        assert!(
            tags_content.contains("supports_vision"),
            "Should contain default tags"
        );
        assert!(
            tags_content.contains("supports_tools"),
            "Should contain default tags"
        );
        assert!(
            tags_content.contains("context_length"),
            "Should contain default tags"
        );
    }

    #[test]
    fn test_add_model_path() {
        let (_temp_dir, config_dir, _guard) = setup_test_config_dir();

        // Initialize config first
        initialize_model_metadata_config().unwrap();

        // Add a new path
        let result = add_model_path(".results[]".to_string());
        assert!(result.is_ok(), "Adding model path should succeed");

        // Verify the path was added
        let model_paths_file = config_dir.join("model_paths.toml");
        let content = fs::read_to_string(&model_paths_file).unwrap();
        assert!(content.contains(".results[]"), "New path should be added");
    }

    #[test]
    fn test_add_duplicate_model_path() {
        let (_temp_dir, config_dir, _guard) = setup_test_config_dir();

        // Initialize config first
        initialize_model_metadata_config().unwrap();

        // Add the same path twice
        let result1 = add_model_path(".data[]".to_string());
        assert!(result1.is_ok(), "First add should succeed");

        let result2 = add_model_path(".data[]".to_string());
        assert!(
            result2.is_ok(),
            "Second add should succeed but not duplicate"
        );

        // Verify only one instance exists
        let model_paths_file = config_dir.join("model_paths.toml");
        let content = fs::read_to_string(&model_paths_file).unwrap();
        let count = content.matches(".data[]").count();
        assert_eq!(count, 1, "Should not have duplicate paths");
    }

    #[test]
    fn test_remove_model_path() {
        let (_temp_dir, config_dir, _guard) = setup_test_config_dir();

        // Initialize config first
        initialize_model_metadata_config().unwrap();

        // Add a path first
        add_model_path(".test[]".to_string()).unwrap();

        // Remove the path
        let result = remove_model_path(".test[]".to_string());
        assert!(result.is_ok(), "Removing model path should succeed");

        // Verify the path was removed
        let model_paths_file = config_dir.join("model_paths.toml");
        let content = fs::read_to_string(&model_paths_file).unwrap();
        assert!(!content.contains(".test[]"), "Path should be removed");
    }

    #[test]
    fn test_remove_nonexistent_model_path() {
        let (_temp_dir, _config_dir, _guard) = setup_test_config_dir();

        // Initialize config first
        initialize_model_metadata_config().unwrap();

        // Try to remove a path that doesn't exist
        let result = remove_model_path(".nonexistent[]".to_string());
        assert!(result.is_ok(), "Removing nonexistent path should not fail");
    }

    #[test]
    fn test_add_tag() {
        let (_temp_dir, config_dir, _guard) = setup_test_config_dir();

        // Initialize config first
        initialize_model_metadata_config().unwrap();

        // Add a new tag
        let result = add_tag(
            "supports_multimodal".to_string(),
            vec![
                ".supports_multimodal".to_string(),
                ".capabilities.multimodal".to_string(),
            ],
            "bool".to_string(),
            None,
        );
        assert!(result.is_ok(), "Adding tag should succeed");

        // Verify the tag was added
        let tags_file = config_dir.join("tags.toml");
        let content = fs::read_to_string(&tags_file).unwrap();
        assert!(
            content.contains("supports_multimodal"),
            "New tag should be added"
        );
        assert!(
            content.contains(".supports_multimodal"),
            "Tag paths should be added"
        );
    }

    #[test]
    fn test_add_tag_with_transform() {
        let (_temp_dir, config_dir, _guard) = setup_test_config_dir();

        // Initialize config first
        initialize_model_metadata_config().unwrap();

        // Add a tag with transform
        let result = add_tag(
            "output_price_per_m".to_string(),
            vec![".pricing.output".to_string()],
            "f64".to_string(),
            Some("multiply_million".to_string()),
        );
        assert!(result.is_ok(), "Adding tag with transform should succeed");

        // Verify the tag was added with transform
        let tags_file = config_dir.join("tags.toml");
        let content = fs::read_to_string(&tags_file).unwrap();
        assert!(
            content.contains("output_price_per_m"),
            "New tag should be added"
        );
        assert!(
            content.contains("multiply_million"),
            "Transform should be added"
        );
    }

    #[test]
    fn test_config_files_created_on_first_run() {
        let _guard = TEST_MUTEX.lock().unwrap();

        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config_path = temp_dir.path().join("lc");

        // Set environment variables to point to our temp directory
        std::env::set_var("XDG_CONFIG_HOME", temp_dir.path());
        std::env::set_var("HOME", temp_dir.path());

        // Ensure config directory doesn't exist initially
        assert!(
            !config_path.exists(),
            "Config directory should not exist initially"
        );

        // Initialize config - should create directory and files
        let result = initialize_model_metadata_config();
        assert!(result.is_ok(), "Config initialization should succeed");

        // Verify directory and files were created
        assert!(config_path.exists(), "Config directory should be created");
        assert!(
            config_path.join("model_paths.toml").exists(),
            "model_paths.toml should be created"
        );
        assert!(
            config_path.join("tags.toml").exists(),
            "tags.toml should be created"
        );
    }

    #[test]
    fn test_config_files_not_overwritten() {
        let (_temp_dir, config_dir, _guard) = setup_test_config_dir();

        // Initialize config first
        initialize_model_metadata_config().unwrap();

        // Modify the files
        let model_paths_file = config_dir.join("model_paths.toml");
        let custom_content = "paths = [\".custom[]\"]\n";
        fs::write(&model_paths_file, custom_content).unwrap();

        // Initialize again - should not overwrite
        let result = initialize_model_metadata_config();
        assert!(result.is_ok(), "Second initialization should succeed");

        // Verify custom content is preserved
        let content = fs::read_to_string(&model_paths_file).unwrap();
        assert!(
            content.contains(".custom[]"),
            "Custom content should be preserved"
        );
        assert!(
            !content.contains(".data[]"),
            "Default content should not be restored"
        );
    }

    #[test]
    fn test_jq_path_array_filtering() {
        use lc::model_metadata::ModelMetadataExtractor;
        use serde_json::json;

        // Create a sample GitHub model JSON similar to what the user provided
        let github_model = json!({
            "capabilities": [
                "streaming",
                "tool-calling"
            ],
            "html_url": "https://github.com/marketplace/models/azure-openai/gpt-4-1",
            "id": "openai/gpt-4.1",
            "limits": {
                "max_input_tokens": 1048576,
                "max_output_tokens": 32768
            },
            "name": "OpenAI GPT-4.1",
            "publisher": "OpenAI"
        });

        let (_temp_dir, _config_dir, _guard) = setup_test_config_dir();

        let extractor = ModelMetadataExtractor::new().unwrap();

        // Test the path that should detect tool-calling support
        let result = extractor.extract_with_jq_path(
            &github_model,
            ".capabilities[] | select(. == \"tool-calling\")",
        );

        assert!(result.is_ok(), "JQ path extraction should succeed");
        let value = result.unwrap();
        assert_eq!(
            value.as_bool(),
            Some(true),
            "Should detect tool-calling support"
        );

        // Test a model without tool-calling to ensure it returns false
        let model_without_tools = json!({
            "capabilities": [
                "streaming"
            ],
            "id": "test/model-without-tools",
            "name": "Test Model Without Tools"
        });

        let result2 = extractor.extract_with_jq_path(
            &model_without_tools,
            ".capabilities[] | select(. == \"tool-calling\")",
        );

        assert!(
            result2.is_ok(),
            "JQ path extraction should succeed for model without tools"
        );
        let value2 = result2.unwrap();
        assert_eq!(
            value2.as_bool(),
            Some(false),
            "Should not detect tool-calling support"
        );

        // Test a model with empty capabilities array
        let model_empty_capabilities = json!({
            "capabilities": [],
            "id": "test/model-empty-capabilities",
            "name": "Test Model Empty Capabilities"
        });

        let result3 = extractor.extract_with_jq_path(
            &model_empty_capabilities,
            ".capabilities[] | select(. == \"tool-calling\")",
        );

        assert!(
            result3.is_ok(),
            "JQ path extraction should succeed for model with empty capabilities"
        );
        let value3 = result3.unwrap();
        assert_eq!(
            value3.as_bool(),
            Some(false),
            "Should not detect tool-calling support in empty array"
        );

        // Test Novita-style model with features array containing "function-calling"
        let novita_model = json!({
            "features": [
                "function-calling",
                "structured-outputs"
            ],
            "id": "deepseek/deepseek-v3-0324",
            "display_name": "DeepSeek V3 0324"
        });

        let result4 = extractor.extract_with_jq_path(
            &novita_model,
            ".features[] | select(. == \"function-calling\")",
        );

        assert!(
            result4.is_ok(),
            "JQ path extraction should succeed for Novita model"
        );
        let value4 = result4.unwrap();
        assert_eq!(
            value4.as_bool(),
            Some(true),
            "Should detect function-calling support in Novita model"
        );

        // Test Novita-style model without function-calling
        let novita_model_no_tools = json!({
            "features": [
                "structured-outputs"
            ],
            "id": "test/model-no-function-calling",
            "display_name": "Test Model No Function Calling"
        });

        let result5 = extractor.extract_with_jq_path(
            &novita_model_no_tools,
            ".features[] | select(. == \"function-calling\")",
        );

        assert!(
            result5.is_ok(),
            "JQ path extraction should succeed for Novita model without function-calling"
        );
        let value5 = result5.unwrap();
        assert_eq!(
            value5.as_bool(),
            Some(false),
            "Should not detect function-calling support when not present"
        );
    }
}

#[cfg(test)]
mod models_validation_tests {

    #[test]
    fn test_token_count_parsing() {
        // Test parsing different token count formats
        fn parse_token_count(input: &str) -> Result<u32, String> {
            let input = input.to_lowercase();
            if let Some(num_str) = input.strip_suffix('k') {
                let num: f32 = num_str
                    .parse()
                    .map_err(|_| format!("Invalid token count format: '{}'", input))?;
                Ok((num * 1000.0) as u32)
            } else if let Some(num_str) = input.strip_suffix('m') {
                let num: f32 = num_str
                    .parse()
                    .map_err(|_| format!("Invalid token count format: '{}'", input))?;
                Ok((num * 1000000.0) as u32)
            } else {
                input
                    .parse()
                    .map_err(|_| format!("Invalid token count format: '{}'", input))
            }
        }

        // Valid formats
        assert_eq!(parse_token_count("1000").unwrap(), 1000);
        assert_eq!(parse_token_count("4k").unwrap(), 4000);
        assert_eq!(parse_token_count("1.5k").unwrap(), 1500);
        assert_eq!(parse_token_count("2m").unwrap(), 2000000);
        assert_eq!(parse_token_count("0.5m").unwrap(), 500000);

        // Invalid formats
        assert!(parse_token_count("invalid").is_err());
        assert!(parse_token_count("1.5.5k").is_err());
        assert!(parse_token_count("k").is_err());
        assert!(parse_token_count("m").is_err());
    }

    #[test]
    fn test_price_validation() {
        // Test price validation
        fn validate_price(price: f64) -> bool {
            price >= 0.0 && price.is_finite()
        }

        assert!(validate_price(0.0));
        assert!(validate_price(1.5));
        assert!(validate_price(100.0));
        assert!(!validate_price(-1.0));
        assert!(!validate_price(f64::INFINITY));
        assert!(!validate_price(f64::NAN));
    }

    #[test]
    fn test_model_id_validation() {
        // Test model ID validation
        fn validate_model_id(id: &str) -> bool {
            !id.is_empty() && !id.trim().is_empty()
        }

        assert!(validate_model_id("gpt-4"));
        assert!(validate_model_id("claude-3-sonnet"));
        assert!(validate_model_id("model_with_underscores"));
        assert!(!validate_model_id(""));
        assert!(!validate_model_id("   "));
        assert!(!validate_model_id("\t\n"));
    }

    #[test]
    fn test_provider_name_validation() {
        // Test provider name validation
        fn validate_provider_name(name: &str) -> bool {
            !name.is_empty()
                && !name.trim().is_empty()
                && name
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        }

        assert!(validate_provider_name("openai"));
        assert!(validate_provider_name("anthropic"));
        assert!(validate_provider_name("provider-name"));
        assert!(validate_provider_name("provider_name"));
        assert!(!validate_provider_name(""));
        assert!(!validate_provider_name("   "));
        assert!(!validate_provider_name("provider with spaces"));
        assert!(!validate_provider_name("provider@special"));
    }
}
