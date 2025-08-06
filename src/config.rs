//! Tests for ProviderConfig URL templating with provider vars and model placeholders

#[cfg(test)]
mod provider_config_tests {
    use super::*;

    #[test]
    fn test_get_chat_url_full_url_with_model_and_vars() {
        let mut pc = ProviderConfig {
            endpoint: "https://aiplatform.googleapis.com".to_string(),
            api_key: None,
            models: vec![],
            models_path: "/v1beta1/{project}/locations/{location}/models".to_string(),
            chat_path: "https://aiplatform.googleapis.com/v1/projects/{project}/locations/{location}/publishers/google/models/{model}:streamGenerateContent".to_string(),
            images_path: None,
            embeddings_path: None,
            headers: HashMap::new(),
            token_url: Some("https://oauth2.googleapis.com/token".to_string()),
            cached_token: None,
            auth_type: Some("google_sa_jwt".to_string()),
            vars: HashMap::new(),
        };

        pc.vars.insert("project".to_string(), "my-proj".to_string());
        pc.vars.insert("location".to_string(), "us-central1".to_string());

        // Should replace {model} and interpolate vars
        let url = pc.get_chat_url("gemini-1.5-pro");
        assert_eq!(
            url,
            "https://aiplatform.googleapis.com/v1/projects/my-proj/locations/us-central1/publishers/google/models/gemini-1.5-pro:streamGenerateContent"
        );

        // Legacy placeholder {model_name} should also work
        pc.chat_path = "https://aiplatform.googleapis.com/v1/projects/{project}/locations/{location}/models/{model_name}:generateContent".to_string();
        let url2 = pc.get_chat_url("gemini-1.5-flash");
        assert_eq!(
            url2,
            "https://aiplatform.googleapis.com/v1/projects/my-proj/locations/us-central1/models/gemini-1.5-flash:generateContent"
        );
    }

    #[test]
    fn test_get_chat_url_non_full_path_concatenation() {
        let pc = ProviderConfig {
            endpoint: "https://api.openai.com".to_string(),
            api_key: None,
            models: vec![],
            models_path: "/v1/models".to_string(),
            chat_path: "/v1/chat/completions".to_string(),
            images_path: None,
            embeddings_path: None,
            headers: HashMap::new(),
            token_url: None,
            cached_token: None,
            auth_type: None,
            vars: HashMap::new(),
        };

        // For non-full URLs, no interpolation or model replacement occurs here
        let url = pc.get_chat_url("gpt-4o");
        assert_eq!(url, "https://api.openai.com/v1/chat/completions");
    }
}
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub providers: HashMap<String, ProviderConfig>,
    pub default_provider: Option<String>,
    pub default_model: Option<String>,
    #[serde(default)]
    pub aliases: HashMap<String, String>, // alias_name -> provider:model
    #[serde(default)]
    pub system_prompt: Option<String>,
    #[serde(default)]
    pub templates: HashMap<String, String>, // template_name -> prompt_content
    #[serde(default)]
    pub max_tokens: Option<u32>,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub stream: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProviderConfig {
    pub endpoint: String,
    pub api_key: Option<String>,
    pub models: Vec<String>,
    #[serde(default = "default_models_path")]
    pub models_path: String,
    #[serde(default = "default_chat_path")]
    pub chat_path: String,
    #[serde(default)]
    pub images_path: Option<String>,
    #[serde(default)]
    pub embeddings_path: Option<String>,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub token_url: Option<String>,
    #[serde(default)]
    pub cached_token: Option<CachedToken>,
    #[serde(default)]
    pub auth_type: Option<String>, // e.g., "google_sa_jwt"
    #[serde(default)]
    pub vars: HashMap<String, String>, // arbitrary provider vars like project, location
}

impl ProviderConfig {
    /// Check if the chat_path is a full URL (starts with https://)
    pub fn is_chat_path_full_url(&self) -> bool {
        self.chat_path.starts_with("https://")
    }
    
    /// Get the models endpoint URL
    pub fn get_models_url(&self) -> String {
        format!("{}{}", self.endpoint.trim_end_matches('/'), self.models_path)
    }
    
    /// Get the chat completions URL, replacing {model_name} and template variables
    pub fn get_chat_url(&self, model_name: &str) -> String {
        crate::debug_log!("ProviderConfig::get_chat_url called with model: {}", model_name);
        crate::debug_log!("  chat_path: {}", self.chat_path);
        crate::debug_log!("  is_full_url: {}", self.is_chat_path_full_url());
        crate::debug_log!("  vars: {:?}", self.vars);
        
        if self.is_chat_path_full_url() {
            // Full URL path - process template variables directly
            let mut url = self.chat_path.replace("{model}", model_name)
                                        .replace("{model_name}", model_name);
            crate::debug_log!("  after model replacement: {}", url);
            
            // Interpolate known vars if present
            for (k, v) in &self.vars {
                let old_url = url.clone();
                url = url.replace(&format!("{{{}}}", k), v);
                crate::debug_log!("  replaced {{{}}} with '{}': {} -> {}", k, v, old_url, url);
            }
            crate::debug_log!("  final URL: {}", url);
            url
        } else {
            // Relative path - first process template variables in the path, then combine with endpoint
            let mut processed_path = self.chat_path.replace("{model}", model_name)
                                                   .replace("{model_name}", model_name);
            crate::debug_log!("  after model replacement in path: {}", processed_path);
            
            // Interpolate known vars in the path
            for (k, v) in &self.vars {
                let old_path = processed_path.clone();
                processed_path = processed_path.replace(&format!("{{{}}}", k), v);
                crate::debug_log!("  replaced {{{}}} with '{}' in path: {} -> {}", k, v, old_path, processed_path);
            }
            
            let url = format!("{}{}", self.endpoint.trim_end_matches('/'), processed_path);
            crate::debug_log!("  final URL: {}", url);
            url
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct CachedToken {
    pub token: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

fn default_models_path() -> String {
    "/models".to_string()
}

fn default_chat_path() -> String {
    "/chat/completions".to_string()
}

#[derive(Debug, Clone)]
pub struct ProviderPaths {
    pub models_path: String,
    pub chat_path: String,
    pub images_path: Option<String>,
    pub embeddings_path: Option<String>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_file_path()?;
        
        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            let config: Config = toml::from_str(&content)?;
            Ok(config)
        } else {
            // Create default config
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
            
            // Ensure config directory exists
            if let Some(parent) = config_path.parent() {
                fs::create_dir_all(parent)?;
            }
            
            config.save()?;
            Ok(config)
        }
    }
    
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_file_path()?;
        let content = toml::to_string_pretty(self)?;
        fs::write(&config_path, content)?;
        Ok(())
    }
    
    pub fn add_provider(&mut self, name: String, endpoint: String) -> Result<()> {
        self.add_provider_with_paths(name, endpoint, None, None)
    }
    
    pub fn add_provider_with_paths(&mut self, name: String, endpoint: String, models_path: Option<String>, chat_path: Option<String>) -> Result<()> {
        let mut provider_config = ProviderConfig {
            endpoint: endpoint.clone(),
            api_key: None,
            models: Vec::new(),
            models_path: models_path.unwrap_or_else(default_models_path),
            chat_path: chat_path.unwrap_or_else(default_chat_path),
            images_path: None,
            embeddings_path: None,
            headers: HashMap::new(),
            token_url: None,
            cached_token: None,
            auth_type: None,
            vars: HashMap::new(),
        };

        // Auto-detect Vertex AI host to mark google_sa_jwt
        if provider_config.endpoint.contains("aiplatform.googleapis.com") {
            provider_config.auth_type = Some("google_sa_jwt".to_string());
            // Default token URL for SA JWT exchange if user later runs lc p t
            if provider_config.token_url.is_none() {
                provider_config.token_url = Some("https://oauth2.googleapis.com/token".to_string());
            }
        }
        
        self.providers.insert(name.clone(), provider_config);
        
        // Set as default if it's the first provider
        if self.default_provider.is_none() {
            self.default_provider = Some(name);
        }
        
        Ok(())
    }
    
    pub fn set_api_key(&mut self, provider: String, api_key: String) -> Result<()> {
        if let Some(provider_config) = self.providers.get_mut(&provider) {
            provider_config.api_key = Some(api_key);
            Ok(())
        } else {
            anyhow::bail!("Provider '{}' not found", provider);
        }
    }
    
    pub fn has_provider(&self, name: &str) -> bool {
        self.providers.contains_key(name)
    }
    
    pub fn get_provider(&self, name: &str) -> Result<&ProviderConfig> {
        self.providers.get(name)
            .ok_or_else(|| anyhow::anyhow!("Provider '{}' not found", name))
    }
    
    
    pub fn add_header(&mut self, provider: String, header_name: String, header_value: String) -> Result<()> {
        if let Some(provider_config) = self.providers.get_mut(&provider) {
            provider_config.headers.insert(header_name, header_value);
            Ok(())
        } else {
            anyhow::bail!("Provider '{}' not found", provider);
        }
    }
    
    pub fn remove_header(&mut self, provider: String, header_name: String) -> Result<()> {
        if let Some(provider_config) = self.providers.get_mut(&provider) {
            if provider_config.headers.remove(&header_name).is_some() {
                Ok(())
            } else {
                anyhow::bail!("Header '{}' not found for provider '{}'", header_name, provider);
            }
        } else {
            anyhow::bail!("Provider '{}' not found", provider);
        }
    }
    
    pub fn list_headers(&self, provider: &str) -> Result<&HashMap<String, String>> {
        if let Some(provider_config) = self.providers.get(provider) {
            Ok(&provider_config.headers)
        } else {
            anyhow::bail!("Provider '{}' not found", provider);
        }
    }
    
    pub fn add_alias(&mut self, alias_name: String, provider_model: String) -> Result<()> {
        // Validate that the provider_model contains a colon
        if !provider_model.contains(':') {
            anyhow::bail!("Alias target must be in format 'provider:model', got '{}'", provider_model);
        }
        
        // Extract provider and validate it exists
        let parts: Vec<&str> = provider_model.splitn(2, ':').collect();
        let provider_name = parts[0];
        
        if !self.has_provider(provider_name) {
            anyhow::bail!("Provider '{}' not found. Add it first with 'lc providers add'", provider_name);
        }
        
        self.aliases.insert(alias_name, provider_model);
        Ok(())
    }
    
    pub fn remove_alias(&mut self, alias_name: String) -> Result<()> {
        if self.aliases.remove(&alias_name).is_some() {
            Ok(())
        } else {
            anyhow::bail!("Alias '{}' not found", alias_name);
        }
    }
    
    pub fn get_alias(&self, alias_name: &str) -> Option<&String> {
        self.aliases.get(alias_name)
    }
    
    pub fn list_aliases(&self) -> &HashMap<String, String> {
        &self.aliases
    }
    
    pub fn add_template(&mut self, template_name: String, prompt_content: String) -> Result<()> {
        self.templates.insert(template_name, prompt_content);
        Ok(())
    }
    
    pub fn remove_template(&mut self, template_name: String) -> Result<()> {
        if self.templates.remove(&template_name).is_some() {
            Ok(())
        } else {
            anyhow::bail!("Template '{}' not found", template_name);
        }
    }
    
    pub fn get_template(&self, template_name: &str) -> Option<&String> {
        self.templates.get(template_name)
    }
    
    pub fn list_templates(&self) -> &HashMap<String, String> {
        &self.templates
    }
    
    pub fn resolve_template_or_prompt(&self, input: &str) -> String {
        if let Some(template_name) = input.strip_prefix("t:") {
            if let Some(template_content) = self.get_template(template_name) {
                template_content.clone()
            } else {
                // If template not found, return the original input
                input.to_string()
            }
        } else {
            input.to_string()
        }
    }
    
    pub fn parse_max_tokens(input: &str) -> Result<u32> {
        let input = input.to_lowercase();
        if let Some(num_str) = input.strip_suffix('k') {
            let num: f32 = num_str.parse()
                .map_err(|_| anyhow::anyhow!("Invalid max_tokens format: '{}'", input))?;
            Ok((num * 1000.0) as u32)
        } else {
            input.parse()
                .map_err(|_| anyhow::anyhow!("Invalid max_tokens format: '{}'", input))
        }
    }
    
    pub fn parse_temperature(input: &str) -> Result<f32> {
        input.parse()
            .map_err(|_| anyhow::anyhow!("Invalid temperature format: '{}'", input))
    }
    
    fn config_file_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
        
        Ok(config_dir.join("lc").join("config.toml"))
    }
    
    pub fn config_dir() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
        
        let lc_dir = config_dir.join("lc");
        fs::create_dir_all(&lc_dir)?;
        Ok(lc_dir)
    }
    
    pub fn set_token_url(&mut self, provider: String, token_url: String) -> Result<()> {
        if let Some(provider_config) = self.providers.get_mut(&provider) {
            provider_config.token_url = Some(token_url);
            // Clear cached token when token_url changes
            provider_config.cached_token = None;
            Ok(())
        } else {
            anyhow::bail!("Provider '{}' not found", provider);
        }
    }

    // Provider vars helpers
    pub fn set_provider_var(&mut self, provider: &str, key: &str, value: &str) -> Result<()> {
        if let Some(pc) = self.providers.get_mut(provider) {
            pc.vars.insert(key.to_string(), value.to_string());
            Ok(())
        } else {
            anyhow::bail!("Provider '{}' not found", provider);
        }
    }

    pub fn get_provider_var(&self, provider: &str, key: &str) -> Option<&String> {
        self.providers.get(provider).and_then(|pc| pc.vars.get(key))
    }

    pub fn list_provider_vars(&self, provider: &str) -> Result<&HashMap<String, String>> {
        if let Some(pc) = self.providers.get(provider) {
            Ok(&pc.vars)
        } else {
            anyhow::bail!("Provider '{}' not found", provider);
        }
    }

    // Provider path management methods
    pub fn set_provider_models_path(&mut self, provider: &str, path: &str) -> Result<()> {
        if let Some(pc) = self.providers.get_mut(provider) {
            pc.models_path = path.to_string();
            Ok(())
        } else {
            anyhow::bail!("Provider '{}' not found", provider);
        }
    }
    
    pub fn set_provider_chat_path(&mut self, provider: &str, path: &str) -> Result<()> {
        if let Some(pc) = self.providers.get_mut(provider) {
            pc.chat_path = path.to_string();
            Ok(())
        } else {
            anyhow::bail!("Provider '{}' not found", provider);
        }
    }
    
    pub fn set_provider_images_path(&mut self, provider: &str, path: &str) -> Result<()> {
        if let Some(pc) = self.providers.get_mut(provider) {
            pc.images_path = Some(path.to_string());
            Ok(())
        } else {
            anyhow::bail!("Provider '{}' not found", provider);
        }
    }
    
    pub fn set_provider_embeddings_path(&mut self, provider: &str, path: &str) -> Result<()> {
        if let Some(pc) = self.providers.get_mut(provider) {
            pc.embeddings_path = Some(path.to_string());
            Ok(())
        } else {
            anyhow::bail!("Provider '{}' not found", provider);
        }
    }
    
    pub fn reset_provider_models_path(&mut self, provider: &str) -> Result<()> {
        if let Some(pc) = self.providers.get_mut(provider) {
            pc.models_path = default_models_path();
            Ok(())
        } else {
            anyhow::bail!("Provider '{}' not found", provider);
        }
    }
    
    pub fn reset_provider_chat_path(&mut self, provider: &str) -> Result<()> {
        if let Some(pc) = self.providers.get_mut(provider) {
            pc.chat_path = default_chat_path();
            Ok(())
        } else {
            anyhow::bail!("Provider '{}' not found", provider);
        }
    }
    
    pub fn reset_provider_images_path(&mut self, provider: &str) -> Result<()> {
        if let Some(pc) = self.providers.get_mut(provider) {
            pc.images_path = None;
            Ok(())
        } else {
            anyhow::bail!("Provider '{}' not found", provider);
        }
    }
    
    pub fn reset_provider_embeddings_path(&mut self, provider: &str) -> Result<()> {
        if let Some(pc) = self.providers.get_mut(provider) {
            pc.embeddings_path = None;
            Ok(())
        } else {
            anyhow::bail!("Provider '{}' not found", provider);
        }
    }
    
    pub fn list_provider_paths(&self, provider: &str) -> Result<ProviderPaths> {
        if let Some(pc) = self.providers.get(provider) {
            Ok(ProviderPaths {
                models_path: pc.models_path.clone(),
                chat_path: pc.chat_path.clone(),
                images_path: pc.images_path.clone(),
                embeddings_path: pc.embeddings_path.clone(),
            })
        } else {
            anyhow::bail!("Provider '{}' not found", provider);
        }
    }
    
    pub fn get_token_url(&self, provider: &str) -> Option<&String> {
        self.providers.get(provider)?.token_url.as_ref()
    }
    
    pub fn set_cached_token(&mut self, provider: String, token: String, expires_at: DateTime<Utc>) -> Result<()> {
        if let Some(provider_config) = self.providers.get_mut(&provider) {
            provider_config.cached_token = Some(CachedToken {
                token,
                expires_at,
            });
            Ok(())
        } else {
            anyhow::bail!("Provider '{}' not found", provider);
        }
    }
    
    pub fn get_cached_token(&self, provider: &str) -> Option<&CachedToken> {
        self.providers.get(provider)?.cached_token.as_ref()
    }
    
}