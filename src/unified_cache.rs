use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use crate::{config::Config, model_metadata::{MetadataExtractor, ModelMetadata}};

#[derive(Debug, Serialize, Deserialize)]
pub struct CachedProviderData {
    pub last_updated: u64, // Unix timestamp
    pub raw_response: String, // Raw JSON response from provider
    pub models: Vec<ModelMetadata>, // Extracted metadata
    // Cache the serialized JSON to avoid repeated serialization
    #[serde(skip)]
    pub cached_json: Option<String>,
}

impl CachedProviderData {
    fn new(raw_response: String, models: Vec<ModelMetadata>) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or(std::time::Duration::from_secs(0))
            .as_secs();
            
        Self {
            last_updated: now,
            raw_response,
            models,
            cached_json: None,
        }
    }
    
    fn get_cached_json(&mut self) -> Result<&str> {
        if self.cached_json.is_none() {
            self.cached_json = Some(serde_json::to_string_pretty(self)?);
        }
        Ok(self.cached_json.as_ref().unwrap())
    }
}

pub struct UnifiedCache;

impl UnifiedCache {
    /// Get the models directory path (cross-platform)
    pub fn models_dir() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
        
        Ok(config_dir.join("lc").join("models"))
    }

    /// Get the cache file path for a specific provider
    pub fn provider_cache_path(provider: &str) -> Result<PathBuf> {
        let models_dir = Self::models_dir()?;
        Ok(models_dir.join(format!("{}.json", provider)))
    }

    /// Check if a provider's cache file exists and is fresh (< 24 hours old)
    pub fn is_cache_fresh(provider: &str) -> Result<bool> {
        let cache_path = Self::provider_cache_path(provider)?;
        
        crate::debug_log!("Checking cache freshness for provider '{}' at path: {}", provider, cache_path.display());
        
        if !cache_path.exists() {
            crate::debug_log!("Cache file does not exist for provider '{}'", provider);
            return Ok(false);
        }

        let content = fs::read_to_string(&cache_path)?;
        let cached_data: CachedProviderData = serde_json::from_str(&content)?;
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        
        let age_seconds = now - cached_data.last_updated;
        let is_fresh = age_seconds < 86400;
        
        crate::debug_log!("Cache for provider '{}' is {} seconds old, fresh: {}", provider, age_seconds, is_fresh);
        
        // Cache is fresh if less than 24 hours old (86400 seconds)
        Ok(is_fresh)
    }

    /// Get cache age in human-readable format (e.g., "5 mins ago", "2 hrs ago")
    pub fn get_cache_age_display(provider: &str) -> Result<String> {
        let cache_path = Self::provider_cache_path(provider)?;
        
        if !cache_path.exists() {
            return Ok("No cache".to_string());
        }

        let content = fs::read_to_string(&cache_path)?;
        let cached_data: CachedProviderData = serde_json::from_str(&content)?;
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        
        let age_seconds = now - cached_data.last_updated;
        
        // Convert to human-readable format
        if age_seconds < 60 {
            Ok(format!("{} secs ago", age_seconds))
        } else if age_seconds < 3600 {
            let minutes = age_seconds / 60;
            Ok(format!("{} min{} ago", minutes, if minutes == 1 { "" } else { "s" }))
        } else if age_seconds < 86400 {
            let hours = age_seconds / 3600;
            Ok(format!("{} hr{} ago", hours, if hours == 1 { "" } else { "s" }))
        } else {
            let days = age_seconds / 86400;
            Ok(format!("{} day{} ago", days, if days == 1 { "" } else { "s" }))
        }
    }

    /// Load cached models for a provider
    pub fn load_provider_models(provider: &str) -> Result<Vec<ModelMetadata>> {
        let cache_path = Self::provider_cache_path(provider)?;
        
        crate::debug_log!("Loading cached models for provider '{}' from: {}", provider, cache_path.display());
        
        if !cache_path.exists() {
            crate::debug_log!("No cache file found for provider '{}'", provider);
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&cache_path)?;
        let cached_data: CachedProviderData = serde_json::from_str(&content)?;
        
        crate::debug_log!("Loaded {} models from cache for provider '{}'", cached_data.models.len(), provider);
        
        Ok(cached_data.models)
    }

    /// Fetch and cache models for a provider
    pub async fn fetch_and_cache_provider_models(provider: &str, force_refresh: bool) -> Result<Vec<ModelMetadata>> {
        crate::debug_log!("Fetching models for provider '{}', force_refresh: {}", provider, force_refresh);
        
        // Check if we need to refresh
        if !force_refresh && Self::is_cache_fresh(provider)? {
            crate::debug_log!("Using cached models for provider '{}' (cache is fresh)", provider);
            return Self::load_provider_models(provider);
        }

        crate::debug_log!("Cache is stale or refresh forced, fetching fresh models for provider '{}'", provider);
        println!("Fetching models from provider '{}'...", provider);

        // Load config and create client
        let config = Config::load()?;
        let provider_config = config.get_provider(provider)?;
        
        crate::debug_log!("Creating authenticated client for provider '{}' with endpoint: {}", provider, provider_config.endpoint);
        
        let mut config_mut = config.clone();
        let client = crate::chat::create_authenticated_client(&mut config_mut, provider).await?;
        
        // Save config if tokens were updated
        if config_mut.get_cached_token(provider) != config.get_cached_token(provider) {
            crate::debug_log!("Tokens were updated for provider '{}', saving config", provider);
            config_mut.save()?;
        }

        // Fetch raw response
        crate::debug_log!("Making API request to fetch models from provider '{}'", provider);
        let raw_response = crate::cli::fetch_raw_models_response(&client, provider_config).await?;
        
        crate::debug_log!("Received raw response from provider '{}' ({} bytes)", provider, raw_response.len());
        
        // Extract metadata
        crate::debug_log!("Extracting metadata from response for provider '{}'", provider);
        let models = MetadataExtractor::extract_from_provider(provider, &raw_response)
            .map_err(|e| anyhow::anyhow!("Failed to extract metadata: {}", e))?;
        
        crate::debug_log!("Extracted {} models from provider '{}'", models.len(), provider);
        
        // Cache the data
        crate::debug_log!("Saving cache data for provider '{}'", provider);
        Self::save_provider_cache(provider, &raw_response, &models)?;
        
        Ok(models)
    }

    /// Save provider data to cache
    fn save_provider_cache(provider: &str, raw_response: &str, models: &[ModelMetadata]) -> Result<()> {
        let cache_path = Self::provider_cache_path(provider)?;
        
        crate::debug_log!("Saving cache for provider '{}' to: {}", provider, cache_path.display());
        
        // Ensure cache directory exists
        if let Some(parent) = cache_path.parent() {
            crate::debug_log!("Creating cache directory: {}", parent.display());
            fs::create_dir_all(parent)?;
        }
        
        // Use the optimized constructor and cached serialization
        let mut cached_data = CachedProviderData::new(
            raw_response.to_string(),
            models.to_vec()
        );
        
        let content = cached_data.get_cached_json()?;
        crate::debug_log!("Writing {} bytes to cache file for provider '{}'", content.len(), provider);
        fs::write(&cache_path, content)?;
        
        crate::debug_log!("Successfully saved cache for provider '{}' with {} models", provider, models.len());
        
        Ok(())
    }

    /// Load all cached models from all providers
    pub fn load_all_cached_models() -> Result<Vec<ModelMetadata>> {
        let models_dir = Self::models_dir()?;
        let mut all_models = Vec::new();
        
        if !models_dir.exists() {
            return Ok(all_models);
        }
        
        let entries = fs::read_dir(&models_dir)?;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if let Some(extension) = path.extension() {
                if extension == "json" {
                    if let Some(provider_name) = path.file_stem().and_then(|s| s.to_str()) {
                        match Self::load_provider_models(provider_name) {
                            Ok(mut models) => {
                                all_models.append(&mut models);
                            }
                            Err(e) => {
                                eprintln!("Warning: Failed to load cached models for {}: {}", provider_name, e);
                            }
                        }
                    }
                }
            }
        }
        
        // Sort by provider, then by model name
        all_models.sort_by(|a, b| {
            a.provider.cmp(&b.provider).then(a.id.cmp(&b.id))
        });
        
        Ok(all_models)
    }

    /// Refresh all providers' caches
    pub async fn refresh_all_providers() -> Result<()> {
        let config = Config::load()?;
        let mut successful_providers = 0;
        let mut total_models = 0;
        
        println!("Refreshing models cache for all providers...");
        
        for (provider_name, provider_config) in &config.providers {
            // Skip providers without API keys
            if provider_config.api_key.is_none() {
                continue;
            }
            
            match Self::fetch_and_cache_provider_models(provider_name, true).await {
                Ok(models) => {
                    let count = models.len();
                    successful_providers += 1;
                    total_models += count;
                    println!("✓ {} ({} models)", provider_name, count);
                }
                Err(e) => {
                    println!("✗ {} ({})", provider_name, e);
                }
            }
        }
        
        println!("\nCache updated: {} providers, {} total models", successful_providers, total_models);
        Ok(())
    }

}