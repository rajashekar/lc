//! Centralized API key management for providers
//! 
//! This module handles storing and retrieving API keys separately from provider configurations,
//! allowing provider configs to be shared and version-controlled without exposing secrets.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Structure for storing API keys and secrets
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct KeysConfig {
    /// Provider API keys - provider_name -> api_key
    #[serde(default)]
    pub api_keys: HashMap<String, String>,
    
    /// Additional authentication tokens (e.g., for search providers)
    #[serde(default)]
    pub tokens: HashMap<String, String>,
    
    /// Service account JSON strings for providers like Google Vertex AI
    #[serde(default)]
    pub service_accounts: HashMap<String, String>,
    
    /// OAuth tokens for providers that use OAuth
    #[serde(default)]
    pub oauth_tokens: HashMap<String, String>,
    
    /// Custom headers that contain sensitive values (renamed from auth_headers)
    #[serde(default, alias = "auth_headers")]
    pub custom_headers: HashMap<String, HashMap<String, String>>,
}

impl KeysConfig {
    /// Create a new empty KeysConfig
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Load keys configuration from file
    pub fn load() -> Result<Self> {
        let keys_path = Self::keys_file_path()?;
        
        if keys_path.exists() {
            let content = fs::read_to_string(&keys_path)?;
            let config: KeysConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            // Create default empty keys config
            let config = KeysConfig::default();
            // Ensure directory exists
            if let Some(parent) = keys_path.parent() {
                fs::create_dir_all(parent)?;
            }
            config.save()?;
            Ok(config)
        }
    }
    
    /// Save keys configuration to file
    pub fn save(&self) -> Result<()> {
        let keys_path = Self::keys_file_path()?;
        
        // Ensure directory exists
        if let Some(parent) = keys_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let content = toml::to_string_pretty(self)?;
        fs::write(&keys_path, content)?;
        
        // Set restrictive permissions on keys file (Unix-like systems)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(&keys_path)?;
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o600); // Read/write for owner only
            fs::set_permissions(&keys_path, permissions)?;
        }
        
        Ok(())
    }
    
    /// Get the path to the keys.toml file
    fn keys_file_path() -> Result<PathBuf> {
        let config_dir = crate::config::Config::config_dir()?;
        Ok(config_dir.join("keys.toml"))
    }
    
    /// Set an API key for a provider
    pub fn set_api_key(&mut self, provider: String, api_key: String) -> Result<()> {
        self.api_keys.insert(provider, api_key);
        self.save()
    }
    
    /// Get an API key for a provider
    pub fn get_api_key(&self, provider: &str) -> Option<&String> {
        self.api_keys.get(provider)
    }
    
    /// Remove an API key for a provider
    pub fn remove_api_key(&mut self, provider: &str) -> Result<bool> {
        let removed = self.api_keys.remove(provider).is_some();
        if removed {
            self.save()?;
        }
        Ok(removed)
    }
    
    /// Set a service account JSON for a provider
    pub fn set_service_account(&mut self, provider: String, sa_json: String) -> Result<()> {
        self.service_accounts.insert(provider, sa_json);
        self.save()
    }
    
    /// Get a service account JSON for a provider
    pub fn get_service_account(&self, provider: &str) -> Option<&String> {
        self.service_accounts.get(provider)
    }
    
    /// Set an authentication token
    pub fn set_token(&mut self, name: String, token: String) -> Result<()> {
        self.tokens.insert(name, token);
        self.save()
    }
    
    /// Get an authentication token
    pub fn get_token(&self, name: &str) -> Option<&String> {
        self.tokens.get(name)
    }
    
    /// Set authentication headers for a provider
    pub fn set_auth_headers(&mut self, provider: String, headers: HashMap<String, String>) -> Result<()> {
        self.custom_headers.insert(provider, headers);
        self.save()
    }
    
    /// Get authentication headers for a provider (returns custom headers)
    pub fn get_auth_headers(&self, provider: &str) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        
        // Check for API key
        if let Some(api_key) = self.api_keys.get(provider) {
            headers.insert("Authorization".to_string(), format!("Bearer {}", api_key));
        }
        
        // Check for custom headers
        if let Some(custom) = self.custom_headers.get(provider) {
            headers.extend(custom.clone());
        }
        
        headers
    }
    
    /// Get authentication for a provider (returns the appropriate auth type)
    pub fn get_auth(&self, provider: &str) -> Option<ProviderAuth> {
        // Check different auth types in order
        if let Some(api_key) = self.api_keys.get(provider) {
            return Some(ProviderAuth::ApiKey(api_key.clone()));
        }
        
        if let Some(sa) = self.service_accounts.get(provider) {
            return Some(ProviderAuth::ServiceAccount(sa.clone()));
        }
        
        if let Some(oauth) = self.oauth_tokens.get(provider) {
            return Some(ProviderAuth::OAuthToken(oauth.clone()));
        }
        
        if let Some(token) = self.tokens.get(provider) {
            return Some(ProviderAuth::Token(token.clone()));
        }
        
        if let Some(headers) = self.custom_headers.get(provider) {
            return Some(ProviderAuth::Headers(headers.clone()));
        }
        
        None
    }
    
    /// List all providers with configured keys
    pub fn list_providers_with_keys(&self) -> Vec<String> {
        let mut providers = Vec::new();
        
        for key in self.api_keys.keys() {
            if !providers.contains(key) {
                providers.push(key.clone());
            }
        }
        
        for key in self.service_accounts.keys() {
            if !providers.contains(key) {
                providers.push(key.clone());
            }
        }
        
        for key in self.custom_headers.keys() {
            if !providers.contains(key) {
                providers.push(key.clone());
            }
        }
        
        providers.sort();
        providers
    }
    
    /// Check if a provider has any authentication configured
    pub fn has_auth(&self, provider: &str) -> bool {
        self.api_keys.contains_key(provider)
            || self.service_accounts.contains_key(provider)
            || self.custom_headers.contains_key(provider)
            || self.oauth_tokens.contains_key(provider)
            || self.tokens.contains_key(provider)
    }
    
    /// Migrate keys from old provider configs to centralized keys.toml
    pub fn migrate_from_provider_configs(config: &crate::config::Config) -> Result<Self> {
        let mut keys_config = Self::load()?;
        let mut migrated_count = 0;
        
        for (provider_name, provider_config) in &config.providers {
            // Migrate API keys
            if let Some(api_key) = &provider_config.api_key {
                if !api_key.is_empty() && !keys_config.api_keys.contains_key(provider_name) {
                    keys_config.api_keys.insert(provider_name.clone(), api_key.clone());
                    migrated_count += 1;
                    crate::debug_log!("Migrated API key for provider '{}'", provider_name);
                }
            }
            
            // Migrate auth headers
            let mut auth_headers = HashMap::new();
            for (header_name, header_value) in &provider_config.headers {
                let header_lower = header_name.to_lowercase();
                if header_lower.contains("key") || header_lower.contains("token") 
                    || header_lower.contains("auth") || header_lower.contains("secret") {
                    auth_headers.insert(header_name.clone(), header_value.clone());
                }
            }
            
            if !auth_headers.is_empty() && !keys_config.custom_headers.contains_key(provider_name) {
                keys_config.custom_headers.insert(provider_name.clone(), auth_headers);
                migrated_count += 1;
                crate::debug_log!("Migrated auth headers for provider '{}'", provider_name);
            }
        }
        
        if migrated_count > 0 {
            keys_config.save()?;
            println!("✓ Migrated {} authentication configurations to keys.toml", migrated_count);
        }
        
        Ok(keys_config)
    }
}

/// Helper function to get authentication for a provider from centralized keys
pub fn get_provider_auth(provider: &str) -> Result<Option<ProviderAuth>> {
    let keys = KeysConfig::load()?;
    Ok(keys.get_auth(provider))
}

/// Types of authentication that can be stored
#[derive(Debug, Clone)]
pub enum ProviderAuth {
    ApiKey(String),
    ServiceAccount(String),
    OAuthToken(String),
    Token(String),
    Headers(HashMap<String, String>),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::TempDir;
    
    #[test]
    fn test_keys_config_save_load() {
        // Create a temporary directory for testing
        let temp_dir = TempDir::new().unwrap();
        env::set_var("LC_TEST_CONFIG_DIR", temp_dir.path());
        
        let mut keys = KeysConfig::default();
        keys.set_api_key("openai".to_string(), "test-key".to_string()).unwrap();
        
        let loaded_keys = KeysConfig::load().unwrap();
        assert_eq!(loaded_keys.get_api_key("openai"), Some(&"test-key".to_string()));
    }
    
    #[test]
    fn test_provider_auth_types() {
        let temp_dir = TempDir::new().unwrap();
        env::set_var("LC_TEST_CONFIG_DIR", temp_dir.path());
        
        let mut keys = KeysConfig::default();
        
        // Test API key
        keys.set_api_key("openai".to_string(), "sk-test".to_string()).unwrap();
        assert!(keys.has_auth("openai"));
        
        // Test service account
        keys.set_service_account("vertex".to_string(), "{\"type\":\"service_account\"}".to_string()).unwrap();
        assert!(keys.has_auth("vertex"));
        
        // Test auth headers
        let mut headers = HashMap::new();
        headers.insert("X-API-Key".to_string(), "test-key".to_string());
        keys.set_auth_headers("custom".to_string(), headers).unwrap();
        assert!(keys.has_auth("custom"));
        
        // Test non-existent provider
        assert!(!keys.has_auth("nonexistent"));
    }
}