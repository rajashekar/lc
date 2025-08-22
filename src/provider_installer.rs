//! Provider configuration installer and manager
//! 
//! This module handles downloading, installing, and updating provider configurations
//! from a central repository, keeping API keys separate from the configurations.

use anyhow::Result;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Provider registry that lists available providers and their metadata
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProviderRegistry {
    /// Version of the registry format
    pub version: String,
    
    /// List of available providers
    pub providers: HashMap<String, ProviderMetadata>,
    
    /// Base URL for downloading provider configs
    pub base_url: String,
}

/// Metadata about a provider
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProviderMetadata {
    /// Display name of the provider
    pub name: String,
    
    /// Description of the provider
    pub description: String,
    
    /// Provider configuration file name
    pub config_file: String,
    
    /// Version of the provider config
    pub version: String,
    
    /// Required authentication type
    pub auth_type: AuthType,
    
    /// Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,
    
    /// Whether this provider is officially supported
    #[serde(default)]
    pub official: bool,
    
    /// Documentation URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs_url: Option<String>,
    
    /// Minimum lc version required
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_version: Option<String>,
}

/// Types of authentication required by providers
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum AuthType {
    ApiKey,
    ServiceAccount,
    OAuth,
    Token,
    Headers,
    None,
}

/// Provider installer that manages downloading and installing provider configs
pub struct ProviderInstaller {
    /// Registry URL or local path
    registry_source: String,
    
    /// Cache directory for downloaded configs
    cache_dir: PathBuf,
    
    /// Target directory for installed providers
    providers_dir: PathBuf,
}

impl ProviderInstaller {
    /// Create a new provider installer
    pub fn new() -> Result<Self> {
        let config_dir = crate::config::Config::config_dir()?;
        let cache_dir = config_dir.join(".provider_cache");
        let providers_dir = config_dir.join("providers");
        
        // Default to GitHub repository
        let registry_source = std::env::var("LC_PROVIDER_REGISTRY")
            .unwrap_or_else(|_| "https://raw.githubusercontent.com/your-org/lc-providers/main".to_string());
        
        Ok(Self {
            registry_source,
            cache_dir,
            providers_dir,
        })
    }
    
    /// Fetch the provider registry
    pub async fn fetch_registry(&self) -> Result<ProviderRegistry> {
        let registry_url = format!("{}/registry.json", self.registry_source);
        
        crate::debug_log!("Fetching provider registry from: {}", registry_url);
        
        // Handle local file paths
        if registry_url.starts_with("file://") {
            let path = registry_url.strip_prefix("file://").unwrap();
            let content = fs::read_to_string(path)
                .map_err(|e| anyhow::anyhow!("Failed to read local registry: {}", e))?;
            let registry: ProviderRegistry = serde_json::from_str(&content)
                .map_err(|e| anyhow::anyhow!("Failed to parse registry: {}", e))?;
            
            // Cache the registry locally
            self.cache_registry(&registry)?;
            
            return Ok(registry);
        }
        
        // Create HTTP client for remote URLs
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;
        
        let response = client
            .get(&registry_url)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to fetch registry: {}", e))?;
        
        if !response.status().is_success() {
            anyhow::bail!("Failed to fetch registry: HTTP {}", response.status());
        }
        
        let registry: ProviderRegistry = response
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to parse registry: {}", e))?;
        
        // Cache the registry locally
        self.cache_registry(&registry)?;
        
        Ok(registry)
    }
    
    /// Get cached registry if available
    pub fn get_cached_registry(&self) -> Result<Option<ProviderRegistry>> {
        let cache_file = self.cache_dir.join("registry.json");
        
        if !cache_file.exists() {
            return Ok(None);
        }
        
        // Check if cache is fresh (less than 24 hours old)
        let metadata = fs::metadata(&cache_file)?;
        if let Ok(modified) = metadata.modified() {
            let age = std::time::SystemTime::now()
                .duration_since(modified)
                .unwrap_or(std::time::Duration::MAX);
            
            if age > std::time::Duration::from_secs(24 * 60 * 60) {
                crate::debug_log!("Registry cache is stale (>24 hours old)");
                return Ok(None);
            }
        }
        
        let content = fs::read_to_string(&cache_file)?;
        let registry: ProviderRegistry = serde_json::from_str(&content)?;
        
        Ok(Some(registry))
    }
    
    /// Cache the registry locally
    fn cache_registry(&self, registry: &ProviderRegistry) -> Result<()> {
        fs::create_dir_all(&self.cache_dir)?;
        
        let cache_file = self.cache_dir.join("registry.json");
        let content = serde_json::to_string_pretty(registry)?;
        fs::write(&cache_file, content)?;
        
        Ok(())
    }
    
    /// List available providers
    pub async fn list_available(&self) -> Result<Vec<(String, ProviderMetadata)>> {
        // Try cached registry first
        let registry = if let Some(cached) = self.get_cached_registry()? {
            cached
        } else {
            self.fetch_registry().await?
        };
        
        let mut providers: Vec<_> = registry.providers.into_iter().collect();
        providers.sort_by(|a, b| a.0.cmp(&b.0));
        
        Ok(providers)
    }
    
    /// Install a provider configuration
    pub async fn install_provider(&self, provider_id: &str, force: bool) -> Result<()> {
        println!("{} Installing provider '{}'...", "📦".blue(), provider_id);
        
        // Fetch registry
        let registry = if let Some(cached) = self.get_cached_registry()? {
            cached
        } else {
            println!("{} Fetching provider registry...", "🔄".blue());
            self.fetch_registry().await?
        };
        
        // Find provider metadata
        let metadata = registry.providers.get(provider_id)
            .ok_or_else(|| anyhow::anyhow!("Provider '{}' not found in registry", provider_id))?;
        
        // Check if already installed
        let target_file = self.providers_dir.join(&metadata.config_file);
        if target_file.exists() && !force {
            // Check version
            if let Ok(existing_config) = fs::read_to_string(&target_file) {
                if let Ok(existing_toml) = toml::from_str::<toml::Value>(&existing_config) {
                    if let Some(existing_version) = existing_toml.get("version")
                        .and_then(|v| v.as_str()) {
                        if existing_version == metadata.version {
                            println!("{} Provider '{}' is already up to date (v{})", 
                                "✓".green(), provider_id, metadata.version);
                            return Ok(());
                        }
                    }
                }
            }
            
            println!("{} Provider '{}' already exists. Updating to v{}...", 
                "🔄".yellow(), provider_id, metadata.version);
        }
        
        // Download provider config
        let config_url = format!("{}/providers/{}", registry.base_url, metadata.config_file);
        
        crate::debug_log!("Downloading provider config from: {}", config_url);
        
        let config_content = if config_url.starts_with("file://") {
            // Handle local file
            let path = config_url.strip_prefix("file://").unwrap();
            fs::read_to_string(path)
                .map_err(|e| anyhow::anyhow!("Failed to read local provider config: {}", e))?
        } else {
            // Handle remote URL
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()?;
            
            let response = client
                .get(&config_url)
                .send()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to download provider config: {}", e))?;
            
            if !response.status().is_success() {
                anyhow::bail!("Failed to download provider config: HTTP {}", response.status());
            }
            
            response.text().await?
        };
        
        // Validate the downloaded config
        self.validate_provider_config(&config_content)?;
        
        // Ensure providers directory exists
        fs::create_dir_all(&self.providers_dir)?;
        
        // Write the provider config
        fs::write(&target_file, &config_content)?;
        
        println!("{} Provider '{}' installed successfully (v{})", 
            "✅".green(), provider_id, metadata.version);
        
        // Show authentication instructions
        self.show_auth_instructions(provider_id, metadata)?;
        
        Ok(())
    }
    
    /// Update a provider configuration
    pub async fn update_provider(&self, provider_id: &str) -> Result<()> {
        self.install_provider(provider_id, true).await
    }
    
    /// Update all installed providers
    pub async fn update_all_providers(&self) -> Result<()> {
        println!("{} Updating all installed providers...", "🔄".blue());
        
        // Get list of installed providers
        let installed = self.list_installed_providers()?;
        
        if installed.is_empty() {
            println!("{} No providers installed", "ℹ️".blue());
            return Ok(());
        }
        
        let mut updated_count = 0;
        let mut failed_count = 0;
        
        for provider_id in installed {
            match self.update_provider(&provider_id).await {
                Ok(_) => updated_count += 1,
                Err(e) => {
                    eprintln!("{} Failed to update '{}': {}", "❌".red(), provider_id, e);
                    failed_count += 1;
                }
            }
        }
        
        if failed_count == 0 {
            println!("{} All {} providers updated successfully", "✅".green(), updated_count);
        } else {
            println!("{} Updated {} providers, {} failed", 
                "⚠️".yellow(), updated_count, failed_count);
        }
        
        Ok(())
    }
    
    /// List installed providers
    pub fn list_installed_providers(&self) -> Result<Vec<String>> {
        if !self.providers_dir.exists() {
            return Ok(Vec::new());
        }
        
        let mut providers = Vec::new();
        
        for entry in fs::read_dir(&self.providers_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    providers.push(name.to_string());
                }
            }
        }
        
        providers.sort();
        Ok(providers)
    }
    
    /// Remove an installed provider
    pub fn uninstall_provider(&self, provider_id: &str) -> Result<()> {
        let provider_file = self.providers_dir.join(format!("{}.toml", provider_id));
        
        if !provider_file.exists() {
            anyhow::bail!("Provider '{}' is not installed", provider_id);
        }
        
        fs::remove_file(&provider_file)?;
        
        println!("{} Provider '{}' uninstalled successfully", "✅".green(), provider_id);
        
        // Check if there are any API keys to clean up
        let keys = crate::keys::KeysConfig::load()?;
        if keys.has_auth(provider_id) {
            println!("{} Note: API keys for '{}' are still stored in keys.toml", 
                "ℹ️".blue(), provider_id);
            println!("  To remove them, use: lc keys remove {}", provider_id);
        }
        
        Ok(())
    }
    
    /// Validate a provider configuration
    fn validate_provider_config(&self, config_content: &str) -> Result<()> {
        // Try to parse as TOML
        let config: toml::Value = toml::from_str(config_content)
            .map_err(|e| anyhow::anyhow!("Invalid TOML format: {}", e))?;
        
        // Check required fields
        let required_fields = ["endpoint", "models_path", "chat_path"];
        
        for field in &required_fields {
            if !config.get(field).is_some() {
                anyhow::bail!("Provider config missing required field: {}", field);
            }
        }
        
        Ok(())
    }
    
    /// Show authentication instructions for a provider
    fn show_auth_instructions(&self, provider_id: &str, metadata: &ProviderMetadata) -> Result<()> {
        println!("\n{} Authentication Setup", "🔑".yellow());
        
        match metadata.auth_type {
            AuthType::ApiKey => {
                println!("This provider requires an API key.");
                println!("To set it up, run:");
                println!("  {}", format!("lc keys add {}", provider_id).bold());
            }
            AuthType::ServiceAccount => {
                println!("This provider requires a service account JSON.");
                println!("To set it up, run:");
                println!("  {}", format!("lc keys add {}", provider_id).bold());
            }
            AuthType::OAuth => {
                println!("This provider uses OAuth authentication.");
                println!("Follow the provider's documentation to set up OAuth.");
                if let Some(docs_url) = &metadata.docs_url {
                    println!("Documentation: {}", docs_url.blue());
                }
            }
            AuthType::Token => {
                println!("This provider requires an authentication token.");
                println!("To set it up, run:");
                println!("  {}", format!("lc keys add {}", provider_id).bold());
            }
            AuthType::Headers => {
                println!("This provider requires custom authentication headers.");
                println!("To set them up, run:");
                println!("  {}", format!("lc providers headers {} add <header-name> <header-value>", provider_id).bold());
            }
            AuthType::None => {
                println!("This provider does not require authentication.");
            }
        }
        
        Ok(())
    }
}

/// Create a sample provider registry for testing
pub fn create_sample_registry() -> ProviderRegistry {
    let mut providers = HashMap::new();
    
    // Add sample providers
    providers.insert("openai".to_string(), ProviderMetadata {
        name: "OpenAI".to_string(),
        description: "OpenAI GPT models including GPT-4 and GPT-3.5".to_string(),
        config_file: "openai.toml".to_string(),
        version: "1.0.0".to_string(),
        auth_type: AuthType::ApiKey,
        tags: vec!["official".to_string(), "chat".to_string(), "embeddings".to_string()],
        official: true,
        docs_url: Some("https://platform.openai.com/docs".to_string()),
        min_version: None,
    });
    
    providers.insert("gemini".to_string(), ProviderMetadata {
        name: "Google Gemini".to_string(),
        description: "Google's Gemini models".to_string(),
        config_file: "gemini.toml".to_string(),
        version: "1.0.0".to_string(),
        auth_type: AuthType::ApiKey,
        tags: vec!["official".to_string(), "chat".to_string(), "vision".to_string()],
        official: true,
        docs_url: Some("https://ai.google.dev/docs".to_string()),
        min_version: None,
    });
    
    providers.insert("anthropic".to_string(), ProviderMetadata {
        name: "Anthropic Claude".to_string(),
        description: "Anthropic's Claude models".to_string(),
        config_file: "anthropic.toml".to_string(),
        version: "1.0.0".to_string(),
        auth_type: AuthType::ApiKey,
        tags: vec!["official".to_string(), "chat".to_string()],
        official: true,
        docs_url: Some("https://docs.anthropic.com".to_string()),
        min_version: None,
    });
    
    ProviderRegistry {
        version: "1.0.0".to_string(),
        providers,
        base_url: "https://raw.githubusercontent.com/your-org/lc-providers/main".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_provider_metadata_serialization() {
        let metadata = ProviderMetadata {
            name: "Test Provider".to_string(),
            description: "A test provider".to_string(),
            config_file: "test.toml".to_string(),
            version: "1.0.0".to_string(),
            auth_type: AuthType::ApiKey,
            tags: vec!["test".to_string()],
            official: false,
            docs_url: None,
            min_version: None,
        };
        
        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: ProviderMetadata = serde_json::from_str(&json).unwrap();
        
        assert_eq!(metadata.name, deserialized.name);
        assert_eq!(metadata.version, deserialized.version);
    }
    
    #[test]
    fn test_registry_creation() {
        let registry = create_sample_registry();
        
        assert!(registry.providers.contains_key("openai"));
        assert!(registry.providers.contains_key("gemini"));
        assert!(registry.providers.contains_key("anthropic"));
        
        let openai = &registry.providers["openai"];
        assert_eq!(openai.name, "OpenAI");
        assert!(openai.official);
    }
}