//! CLI utility functions used throughout the application and tests

use anyhow::{anyhow, Result};
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::config::Config;

/// Global debug mode flag
static DEBUG_MODE: AtomicBool = AtomicBool::new(false);

/// Set the global debug mode
pub fn set_debug_mode(enabled: bool) {
    DEBUG_MODE.store(enabled, Ordering::Relaxed);
}

/// Check if debug mode is enabled
pub fn is_debug_mode() -> bool {
    DEBUG_MODE.load(Ordering::Relaxed)
}

/// Determine if a file extension represents a code file
pub fn is_code_file(ext: &str) -> bool {
    let code_extensions: HashSet<&str> = [
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
    ]
    .iter()
    .cloned()
    .collect();

    code_extensions.contains(&ext.to_lowercase().as_str())
}

/// Read and format attachment files for inclusion in prompts
pub fn read_and_format_attachments(attachments: &[String]) -> Result<String> {
    if attachments.is_empty() {
        return Ok(String::new());
    }

    let mut result = String::new();

    for attachment_path in attachments {
        let path = Path::new(attachment_path);
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        // Read file content
        let content = fs::read_to_string(path)
            .map_err(|e| anyhow!("Failed to read file '{}': {}", attachment_path, e))?;

        // Add file header
        result.push_str(&format!("=== File: {} ===\n", filename));

        // Check if this is a code file based on extension
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if is_code_file(ext) {
                result.push_str(&format!("```{}\n{}\n```\n", ext.to_lowercase(), content));
            } else {
                result.push_str(&content);
            }
        } else {
            result.push_str(&content);
        }

        result.push('\n');
    }

    Ok(result)
}

/// Resolve model and provider from configuration and CLI overrides
pub fn resolve_model_and_provider(
    config: &Config,
    provider_override: Option<String>,
    model_override: Option<String>,
) -> Result<(String, String)> {
    // Store whether we have explicit provider override to avoid borrow issues
    let has_provider_override = provider_override.is_some();

    let provider = match provider_override {
        Some(p) => {
            if !config.providers.contains_key(&p) {
                return Err(anyhow!("Provider '{}' not found in configuration", p));
            }
            p
        }
        None => config
            .default_provider
            .clone()
            .ok_or_else(|| anyhow!("No default provider configured and none specified"))?,
    };

    let model = match model_override {
        Some(m) => {
            // Check if model is in format "provider:model"
            if m.contains(':') && !has_provider_override {
                let parts: Vec<&str> = m.splitn(2, ':').collect();
                if parts.len() == 2 {
                    let alias_provider = parts[0].to_string();
                    let alias_model = parts[1].to_string();

                    // Verify provider exists
                    if !config.providers.contains_key(&alias_provider) {
                        return Err(anyhow!(
                            "Provider '{}' not found in configuration",
                            alias_provider
                        ));
                    }

                    return Ok((alias_provider, alias_model));
                }
            }

            // Check if it's an alias (only if provider is not explicitly set)
            if !has_provider_override {
                if let Some(alias_target) = config.aliases.get(&m) {
                    let parts: Vec<&str> = alias_target.splitn(2, ':').collect();
                    if parts.len() == 2 {
                        let alias_provider = parts[0].to_string();
                        let alias_model = parts[1].to_string();

                        // Verify provider exists
                        if !config.providers.contains_key(&alias_provider) {
                            return Err(anyhow!(
                                "Provider '{}' from alias not found in configuration",
                                alias_provider
                            ));
                        }

                        return Ok((alias_provider, alias_model));
                    } else {
                        return Err(anyhow!(
                            "Invalid alias target format: '{}'. Expected 'provider:model'",
                            alias_target
                        ));
                    }
                }
            }

            m
        }
        None => config
            .default_model
            .clone()
            .ok_or_else(|| anyhow!("No default model configured and none specified"))?,
    };

    Ok((provider, model))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ProviderConfig;
    use std::collections::HashMap;

    #[test]
    fn test_is_code_file() {
        assert!(is_code_file("rs"));
        assert!(is_code_file("py"));
        assert!(is_code_file("js"));
        assert!(is_code_file("json"));

        assert!(!is_code_file("txt"));
        assert!(!is_code_file("md"));
        assert!(!is_code_file("pdf"));
    }

    #[test]
    fn test_debug_mode() {
        set_debug_mode(true);
        assert!(is_debug_mode());

        set_debug_mode(false);
        assert!(!is_debug_mode());
    }

    #[test]
    fn test_resolve_model_basic() {
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

        config.providers.insert(
            "openai".to_string(),
            ProviderConfig {
                endpoint: "https://api.openai.com".to_string(),
                models_path: "/v1/models".to_string(),
                chat_path: "/v1/chat/completions".to_string(),
                images_path: Some("/images/generations".to_string()),
                embeddings_path: Some("/embeddings".to_string()),
                api_key: Some("key".to_string()),
                models: Vec::new(),
                headers: HashMap::new(),
                token_url: None,
                cached_token: None,
                auth_type: None,
                vars: HashMap::new(),
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

        let (provider, model) = resolve_model_and_provider(&config, None, None).unwrap();
        assert_eq!(provider, "openai");
        assert_eq!(model, "gpt-4");
    }
}
