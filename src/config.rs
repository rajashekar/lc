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
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub token_url: Option<String>,
    #[serde(default)]
    pub cached_token: Option<CachedToken>,
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
    
    /// Get the chat completions URL, replacing {model_name} if it's a full URL
    pub fn get_chat_url(&self, model_name: &str) -> String {
        if self.is_chat_path_full_url() {
            // Replace {model_name} placeholder in the full URL
            self.chat_path.replace("{model_name}", model_name)
        } else {
            // Traditional path-based approach
            format!("{}{}", self.endpoint.trim_end_matches('/'), self.chat_path)
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
        let provider_config = ProviderConfig {
            endpoint,
            api_key: None,
            models: Vec::new(),
            models_path: models_path.unwrap_or_else(default_models_path),
            chat_path: chat_path.unwrap_or_else(default_chat_path),
            headers: HashMap::new(),
            token_url: None,
            cached_token: None,
        };
        
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