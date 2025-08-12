//! Configuration management for the lc CLI tool
//!
//! This module handles loading, saving, and managing configuration for providers,
//! aliases, templates, and other settings.
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use super::template_processor::TemplateConfig;

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
    #[serde(default)]
    pub chat_templates: Option<HashMap<String, TemplateConfig>>, // Chat endpoint templates
    #[serde(default)]
    pub images_templates: Option<HashMap<String, TemplateConfig>>, // Images endpoint templates
    #[serde(default)]
    pub embeddings_templates: Option<HashMap<String, TemplateConfig>>, // Embeddings endpoint templates
    #[serde(default)]
    pub models_templates: Option<HashMap<String, TemplateConfig>>, // Models endpoint templates
}

impl ProviderConfig {
    /// Check if the chat_path is a full URL (starts with https://)
    pub fn is_chat_path_full_url(&self) -> bool {
        self.chat_path.starts_with("https://")
    }

    /// Get the models endpoint URL
    pub fn get_models_url(&self) -> String {
        format!(
            "{}{}",
            self.endpoint.trim_end_matches('/'),
            self.models_path
        )
    }

    /// Get the chat completions URL, replacing {model_name} and template variables
    pub fn get_chat_url(&self, model_name: &str) -> String {
        crate::debug_log!(
            "ProviderConfig::get_chat_url called with model: {}",
            model_name
        );
        crate::debug_log!("  chat_path: {}", self.chat_path);
        crate::debug_log!("  is_full_url: {}", self.is_chat_path_full_url());
        crate::debug_log!("  vars: {:?}", self.vars);

        if self.is_chat_path_full_url() {
            // Full URL path - process template variables directly
            let mut url = self
                .chat_path
                .replace("{model}", model_name)
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
            let mut processed_path = self
                .chat_path
                .replace("{model}", model_name)
                .replace("{model_name}", model_name);
            crate::debug_log!("  after model replacement in path: {}", processed_path);

            // Interpolate known vars in the path
            for (k, v) in &self.vars {
                let old_path = processed_path.clone();
                processed_path = processed_path.replace(&format!("{{{}}}", k), v);
                crate::debug_log!(
                    "  replaced {{{}}} with '{}' in path: {} -> {}",
                    k,
                    v,
                    old_path,
                    processed_path
                );
            }

            let url = format!("{}{}", self.endpoint.trim_end_matches('/'), processed_path);
            crate::debug_log!("  final URL: {}", url);
            url
        }
    }

    /// Get template for a specific endpoint and model
    pub fn get_endpoint_template(&self, endpoint: &str, model_name: &str) -> Option<String> {
        let endpoint_templates = match endpoint {
            "chat" => self.chat_templates.as_ref()?,
            "images" => self.images_templates.as_ref()?,
            "embeddings" => self.embeddings_templates.as_ref()?,
            "models" => self.models_templates.as_ref()?,
            _ => return None,
        };

        self.get_template_for_model(endpoint_templates, model_name, "request")
    }

    /// Get response template for a specific endpoint and model
    pub fn get_endpoint_response_template(&self, endpoint: &str, model_name: &str) -> Option<String> {
        let endpoint_templates = match endpoint {
            "chat" => self.chat_templates.as_ref()?,
            "images" => self.images_templates.as_ref()?,
            "embeddings" => self.embeddings_templates.as_ref()?,
            "models" => self.models_templates.as_ref()?,
            _ => return None,
        };

        self.get_template_for_model(endpoint_templates, model_name, "response")
    }

    /// Get template for a specific model from endpoint templates
    fn get_template_for_model(&self, templates: &HashMap<String, TemplateConfig>, model_name: &str, template_type: &str) -> Option<String> {
        // First check exact match
        if let Some(template) = templates.get(model_name) {
            return match template_type {
                "request" => template.request.clone(),
                "response" => template.response.clone(),
                "stream_response" => template.stream_response.clone(),
                _ => None,
            };
        }
        
        // Then check regex patterns (skip empty string which is the default)
        for (pattern, template) in templates {
            if !pattern.is_empty() {
                if let Ok(re) = regex::Regex::new(pattern) {
                    if re.is_match(model_name) {
                        return match template_type {
                            "request" => template.request.clone(),
                            "response" => template.response.clone(),
                            "stream_response" => template.stream_response.clone(),
                            _ => None,
                        };
                    }
                }
            }
        }
        
        // Finally check for default template (empty key)
        if let Some(template) = templates.get("") {
            return match template_type {
                "request" => template.request.clone(),
                "response" => template.response.clone(),
                "stream_response" => template.stream_response.clone(),
                _ => None,
            };
        }
        
        None
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
        let providers_dir = Self::providers_dir()?;

        let mut config = if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            let mut config: Config = toml::from_str(&content)?;
            
            // If providers exist in main config, migrate them to separate files
            if !config.providers.is_empty() {
                Self::migrate_providers_to_separate_files(&mut config)?;
            }
            
            config
        } else {
            // Create default config
            Config {
                providers: HashMap::new(),
                default_provider: None,
                default_model: None,
                aliases: HashMap::new(),
                system_prompt: None,
                templates: HashMap::new(),
                max_tokens: None,
                temperature: None,
                stream: None,
            }
        };
        // Load providers from separate files
        config.providers = Self::load_providers_from_files(&providers_dir)?;

        // Ensure config directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Ensure providers directory exists
        fs::create_dir_all(&providers_dir)?;

        // Save the main config (without providers)
        config.save_main_config()?;
        
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        // Save main config without providers
        self.save_main_config()?;
        
        // Save each provider to its own file
        self.save_providers_to_files()?;
        
        Ok(())
    }

    fn save_main_config(&self) -> Result<()> {
        let config_path = Self::config_file_path()?;
        
        // Create a config without providers for the main file
        let main_config = Config {
            providers: HashMap::new(), // Empty - providers are in separate files
            default_provider: self.default_provider.clone(),
            default_model: self.default_model.clone(),
            aliases: self.aliases.clone(),
            system_prompt: self.system_prompt.clone(),
            templates: self.templates.clone(),
            max_tokens: self.max_tokens,
            temperature: self.temperature,
            stream: self.stream,
        };
        
        let content = toml::to_string_pretty(&main_config)?;
        fs::write(&config_path, content)?;
        Ok(())
    }

    fn save_providers_to_files(&self) -> Result<()> {
        let providers_dir = Self::providers_dir()?;
        fs::create_dir_all(&providers_dir)?;

        for (provider_name, provider_config) in &self.providers {
            self.save_single_provider_flat(provider_name, provider_config)?;
        }
        
        Ok(())
    }

    fn load_providers_from_files(providers_dir: &PathBuf) -> Result<HashMap<String, ProviderConfig>> {
        let mut providers = HashMap::new();
        
        if !providers_dir.exists() {
            return Ok(providers);
        }

        for entry in fs::read_dir(providers_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                if let Some(provider_name) = path.file_stem().and_then(|s| s.to_str()) {
                    let content = fs::read_to_string(&path)?;
                    
                    // Try to parse as new flatter format first
                    match Self::parse_flat_provider_config(&content) {
                        Ok(config) => {
                            providers.insert(provider_name.to_string(), config);
                        }
                        Err(_flat_error) => {
                            // Fall back to old nested format for backward compatibility
                            let provider_data: HashMap<String, HashMap<String, ProviderConfig>> = toml::from_str(&content)?;
                            
                            if let Some(providers_section) = provider_data.get("providers") {
                                for (name, config) in providers_section {
                                    providers.insert(name.clone(), config.clone());
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(providers)
    }

    fn parse_flat_provider_config(content: &str) -> Result<ProviderConfig> {
        #[derive(Deserialize)]
        struct FlatProviderConfig {
            #[serde(flatten)]
            config: ProviderConfig,
        }

        let flat_config: FlatProviderConfig = toml::from_str(content)?;
        Ok(flat_config.config)
    }

    fn migrate_providers_to_separate_files(config: &mut Config) -> Result<()> {
        let providers_dir = Self::providers_dir()?;
        fs::create_dir_all(&providers_dir)?;

        // Save each provider to its own file using the new flat format
        for (provider_name, provider_config) in &config.providers {
            Self::save_single_provider_flat_static(&providers_dir, provider_name, provider_config)?;
        }

        // Clear providers from main config since they're now in separate files
        config.providers.clear();
        
        Ok(())
    }

    pub fn add_provider(&mut self, name: String, endpoint: String) -> Result<()> {
        self.add_provider_with_paths(name, endpoint, None, None)
    }

    pub fn add_provider_with_paths(
        &mut self,
        name: String,
        endpoint: String,
        models_path: Option<String>,
        chat_path: Option<String>,
    ) -> Result<()> {
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
            chat_templates: None,
            images_templates: None,
            embeddings_templates: None,
            models_templates: None,
        };

        // Auto-detect Vertex AI host to mark google_sa_jwt
        if provider_config
            .endpoint
            .contains("aiplatform.googleapis.com")
        {
            provider_config.auth_type = Some("google_sa_jwt".to_string());
            // Default token URL for SA JWT exchange if user later runs lc p t
            if provider_config.token_url.is_none() {
                provider_config.token_url = Some("https://oauth2.googleapis.com/token".to_string());
            }
        }

        self.providers.insert(name.clone(), provider_config.clone());

        // Set as default if it's the first provider
        if self.default_provider.is_none() {
            self.default_provider = Some(name.clone());
        }

        // Save the provider to its own file
        self.save_single_provider(&name, &provider_config)?;

        Ok(())
    }

    pub fn set_api_key(&mut self, provider: String, api_key: String) -> Result<()> {
        if let Some(provider_config) = self.providers.get_mut(&provider) {
            provider_config.api_key = Some(api_key);
            let config_clone = provider_config.clone();
            self.save_single_provider(&provider, &config_clone)?;
            Ok(())
        } else {
            anyhow::bail!("Provider '{}' not found", provider);
        }
    }

    pub fn has_provider(&self, name: &str) -> bool {
        self.providers.contains_key(name)
    }

    pub fn get_provider(&self, name: &str) -> Result<&ProviderConfig> {
        self.providers
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Provider '{}' not found", name))
    }

    pub fn add_header(
        &mut self,
        provider: String,
        header_name: String,
        header_value: String,
    ) -> Result<()> {
        if let Some(provider_config) = self.providers.get_mut(&provider) {
            provider_config.headers.insert(header_name, header_value);
            let config_clone = provider_config.clone();
            self.save_single_provider(&provider, &config_clone)?;
            Ok(())
        } else {
            anyhow::bail!("Provider '{}' not found", provider);
        }
    }

    pub fn remove_header(&mut self, provider: String, header_name: String) -> Result<()> {
        if let Some(provider_config) = self.providers.get_mut(&provider) {
            if provider_config.headers.remove(&header_name).is_some() {
                let config_clone = provider_config.clone();
                self.save_single_provider(&provider, &config_clone)?;
                Ok(())
            } else {
                anyhow::bail!(
                    "Header '{}' not found for provider '{}'",
                    header_name,
                    provider
                );
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
            anyhow::bail!(
                "Alias target must be in format 'provider:model', got '{}'",
                provider_model
            );
        }

        // Extract provider and validate it exists
        let parts: Vec<&str> = provider_model.splitn(2, ':').collect();
        let provider_name = parts[0];

        if !self.has_provider(provider_name) {
            anyhow::bail!(
                "Provider '{}' not found. Add it first with 'lc providers add'",
                provider_name
            );
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
            let num: f32 = num_str
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid max_tokens format: '{}'", input))?;
            Ok((num * 1000.0) as u32)
        } else {
            input
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid max_tokens format: '{}'", input))
        }
    }

    pub fn parse_temperature(input: &str) -> Result<f32> {
        input
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid temperature format: '{}'", input))
    }

    fn config_file_path() -> Result<PathBuf> {
        let config_dir = Self::config_dir()?;
        Ok(config_dir.join("config.toml"))
    }

    fn providers_dir() -> Result<PathBuf> {
        let config_dir = Self::config_dir()?;
        Ok(config_dir.join("providers"))
    }

    pub fn config_dir() -> Result<PathBuf> {
        // Use data_local_dir for cross-platform data storage to match database location
        // On macOS: ~/Library/Application Support/lc
        // On Linux: ~/.local/share/lc
        // On Windows: %LOCALAPPDATA%/lc
        let data_dir = dirs::data_local_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find data directory"))?
            .join("lc");
        
        // Only create directory if it doesn't exist to prevent potential recursion
        if !data_dir.exists() {
            fs::create_dir_all(&data_dir)?;
        }
        Ok(data_dir)
    }

    fn save_single_provider(&self, provider_name: &str, provider_config: &ProviderConfig) -> Result<()> {
        self.save_single_provider_flat(provider_name, provider_config)
    }

    fn save_single_provider_flat(&self, provider_name: &str, provider_config: &ProviderConfig) -> Result<()> {
        let providers_dir = Self::providers_dir()?;
        Self::save_single_provider_flat_static(&providers_dir, provider_name, provider_config)
    }

    fn save_single_provider_flat_static(providers_dir: &PathBuf, provider_name: &str, provider_config: &ProviderConfig) -> Result<()> {
        fs::create_dir_all(providers_dir)?;

        let provider_file = providers_dir.join(format!("{}.toml", provider_name));
        
        // Use the new flat format - serialize the ProviderConfig directly
        let content = toml::to_string_pretty(provider_config)?;
        fs::write(&provider_file, content)?;
        
        Ok(())
    }

    pub fn set_token_url(&mut self, provider: String, token_url: String) -> Result<()> {
        if let Some(provider_config) = self.providers.get_mut(&provider) {
            provider_config.token_url = Some(token_url);
            // Clear cached token when token_url changes
            provider_config.cached_token = None;
            let config_clone = provider_config.clone();
            self.save_single_provider(&provider, &config_clone)?;
            Ok(())
        } else {
            anyhow::bail!("Provider '{}' not found", provider);
        }
    }

    // Provider vars helpers
    pub fn set_provider_var(&mut self, provider: &str, key: &str, value: &str) -> Result<()> {
        if let Some(pc) = self.providers.get_mut(provider) {
            pc.vars.insert(key.to_string(), value.to_string());
            let config_clone = pc.clone();
            self.save_single_provider(provider, &config_clone)?;
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
            let config_clone = pc.clone();
            self.save_single_provider(provider, &config_clone)?;
            Ok(())
        } else {
            anyhow::bail!("Provider '{}' not found", provider);
        }
    }

    pub fn set_provider_chat_path(&mut self, provider: &str, path: &str) -> Result<()> {
        if let Some(pc) = self.providers.get_mut(provider) {
            pc.chat_path = path.to_string();
            let config_clone = pc.clone();
            self.save_single_provider(provider, &config_clone)?;
            Ok(())
        } else {
            anyhow::bail!("Provider '{}' not found", provider);
        }
    }

    pub fn set_provider_images_path(&mut self, provider: &str, path: &str) -> Result<()> {
        if let Some(pc) = self.providers.get_mut(provider) {
            pc.images_path = Some(path.to_string());
            let config_clone = pc.clone();
            self.save_single_provider(provider, &config_clone)?;
            Ok(())
        } else {
            anyhow::bail!("Provider '{}' not found", provider);
        }
    }

    pub fn set_provider_embeddings_path(&mut self, provider: &str, path: &str) -> Result<()> {
        if let Some(pc) = self.providers.get_mut(provider) {
            pc.embeddings_path = Some(path.to_string());
            let config_clone = pc.clone();
            self.save_single_provider(provider, &config_clone)?;
            Ok(())
        } else {
            anyhow::bail!("Provider '{}' not found", provider);
        }
    }

    pub fn reset_provider_models_path(&mut self, provider: &str) -> Result<()> {
        if let Some(pc) = self.providers.get_mut(provider) {
            pc.models_path = default_models_path();
            let config_clone = pc.clone();
            self.save_single_provider(provider, &config_clone)?;
            Ok(())
        } else {
            anyhow::bail!("Provider '{}' not found", provider);
        }
    }

    pub fn reset_provider_chat_path(&mut self, provider: &str) -> Result<()> {
        if let Some(pc) = self.providers.get_mut(provider) {
            pc.chat_path = default_chat_path();
            let config_clone = pc.clone();
            self.save_single_provider(provider, &config_clone)?;
            Ok(())
        } else {
            anyhow::bail!("Provider '{}' not found", provider);
        }
    }

    pub fn reset_provider_images_path(&mut self, provider: &str) -> Result<()> {
        if let Some(pc) = self.providers.get_mut(provider) {
            pc.images_path = None;
            let config_clone = pc.clone();
            self.save_single_provider(provider, &config_clone)?;
            Ok(())
        } else {
            anyhow::bail!("Provider '{}' not found", provider);
        }
    }

    pub fn reset_provider_embeddings_path(&mut self, provider: &str) -> Result<()> {
        if let Some(pc) = self.providers.get_mut(provider) {
            pc.embeddings_path = None;
            let config_clone = pc.clone();
            self.save_single_provider(provider, &config_clone)?;
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

    pub fn set_cached_token(
        &mut self,
        provider: String,
        token: String,
        expires_at: DateTime<Utc>,
    ) -> Result<()> {
        if let Some(provider_config) = self.providers.get_mut(&provider) {
            provider_config.cached_token = Some(CachedToken { token, expires_at });
            let config_clone = provider_config.clone();
            self.save_single_provider(&provider, &config_clone)?;
            Ok(())
        } else {
            anyhow::bail!("Provider '{}' not found", provider);
        }
    }

    pub fn get_cached_token(&self, provider: &str) -> Option<&CachedToken> {
        self.providers.get(provider)?.cached_token.as_ref()
    }
}
