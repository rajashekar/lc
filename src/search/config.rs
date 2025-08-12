use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use super::providers::{SearchProviderConfig, SearchProviderType};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchConfig {
    pub providers: HashMap<String, SearchProviderConfig>,
    pub default_provider: Option<String>,
}

impl SearchConfig {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            default_provider: None,
        }
    }

    pub fn load() -> Result<Self> {
        let config_path = Self::config_file_path()?;
        
        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            let config: SearchConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            let config = Self::new();
            // Only try to save if we can create the parent directory
            if Self::ensure_config_dir().is_ok() {
                let _ = config.save(); // Ignore save errors during initial load
            }
            Ok(config)
        }
    }
    
    /// Ensure the config directory exists
    fn ensure_config_dir() -> Result<()> {
        let config_path = Self::config_file_path()?;
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        Ok(())
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_file_path()?;

        // Ensure parent directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        fs::write(&config_path, content)?;
        Ok(())
    }

    pub fn add_provider(
        &mut self,
        name: String,
        url: String,
        provider_type: SearchProviderType,
    ) -> Result<()> {
        let provider_config = SearchProviderConfig {
            url,
            provider_type,
            headers: HashMap::new(),
        };

        self.providers.insert(name.clone(), provider_config);

        // Set as default if it's the first provider
        if self.default_provider.is_none() {
            self.default_provider = Some(name);
        }

        Ok(())
    }

    /// Add provider with auto-detected type from URL
    pub fn add_provider_auto(&mut self, name: String, url: String) -> Result<()> {
        let provider_type = SearchProviderType::detect_from_url(&url)?;
        self.add_provider(name, url, provider_type)
    }

    pub fn delete_provider(&mut self, name: &str) -> Result<()> {
        if self.providers.remove(name).is_none() {
            anyhow::bail!("Search provider '{}' not found", name);
        }

        // Clear default if it was the deleted provider
        if self.default_provider.as_ref() == Some(&name.to_string()) {
            self.default_provider = None;
        }

        Ok(())
    }

    pub fn set_header(
        &mut self,
        provider: &str,
        header_name: String,
        header_value: String,
    ) -> Result<()> {
        if let Some(provider_config) = self.providers.get_mut(provider) {
            provider_config.headers.insert(header_name, header_value);
            Ok(())
        } else {
            anyhow::bail!("Search provider '{}' not found", provider);
        }
    }

    pub fn get_provider(&self, name: &str) -> Result<&SearchProviderConfig> {
        self.providers
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Search provider '{}' not found", name))
    }

    pub fn has_provider(&self, name: &str) -> bool {
        self.providers.contains_key(name)
    }

    pub fn list_providers(&self) -> &HashMap<String, SearchProviderConfig> {
        &self.providers
    }

    pub fn set_default_provider(&mut self, name: String) -> Result<()> {
        if name.is_empty() {
            self.default_provider = None;
        } else if self.has_provider(&name) {
            self.default_provider = Some(name);
        } else {
            anyhow::bail!("Search provider '{}' not found", name);
        }
        Ok(())
    }

    pub fn get_default_provider(&self) -> Option<&String> {
        self.default_provider.as_ref()
    }

    fn config_file_path() -> Result<PathBuf> {
        let config_dir = crate::config::Config::config_dir()?;
        Ok(config_dir.join("search_config.toml"))
    }
}
