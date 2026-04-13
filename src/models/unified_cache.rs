use crate::{
    config::Config,
    debug_log, // Import debug_log macro
    model_metadata::{extract_models_from_provider, ModelMetadata},
    provider::Provider,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedProviderData {
    pub last_updated: u64,          // Unix timestamp
    pub raw_response: String,       // Raw JSON response from provider
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
        self.cached_json
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("Failed to get cached JSON"))
    }
}

// In-memory cache entry with TTL
#[derive(Debug, Clone)]
struct MemoryCacheEntry {
    data: CachedProviderData,
    expires_at: u64,
}

impl MemoryCacheEntry {
    fn new(data: CachedProviderData, ttl_seconds: u64) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();

        Self {
            data,
            expires_at: now + ttl_seconds,
        }
    }

    fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();

        now >= self.expires_at
    }
}

// Global in-memory cache with efficient invalidation
lazy_static::lazy_static! {
    static ref MEMORY_CACHE: Arc<RwLock<HashMap<String, MemoryCacheEntry>>> =
        Arc::new(RwLock::new(HashMap::new()));
}

pub struct UnifiedCache;

impl UnifiedCache {
    /// Cache TTL in seconds (24 hours)
    const CACHE_TTL: u64 = 86400;

    /// Get the models directory path (cross-platform)
    pub fn models_dir() -> Result<PathBuf> {
        let config_dir =
            dirs::config_dir().ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;

        Ok(config_dir.join("lc").join("models"))
    }

    /// Get the cache file path for a specific provider
    pub fn provider_cache_path(provider: &str) -> Result<PathBuf> {
        let models_dir = Self::models_dir()?;
        Ok(models_dir.join(format!("{}.json", provider)))
    }

    /// Check in-memory cache first, then file cache
    pub async fn is_cache_fresh(provider: &str) -> Result<bool> {
        debug_log!("Checking cache freshness for provider '{}'", provider);

        // Check in-memory cache first
        if let Ok(cache) = MEMORY_CACHE.read() {
            if let Some(entry) = cache.get(provider) {
                if !entry.is_expired() {
                    debug_log!("Found fresh in-memory cache for provider '{}'", provider);
                    return Ok(true);
                } else {
                    debug_log!("In-memory cache expired for provider '{}'", provider);
                }
            }
        }

        // Fall back to file cache
        let cache_path = Self::provider_cache_path(provider)?;

        if !cache_path.exists() {
            debug_log!("Cache file does not exist for provider '{}'", provider);
            return Ok(false);
        }

        // Use async file I/O to avoid blocking
        let content = fs::read_to_string(&cache_path).await?;
        let cached_data: CachedProviderData = serde_json::from_str(&content)?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();

        let age_seconds = now - cached_data.last_updated;
        let is_fresh = age_seconds < Self::CACHE_TTL;

        debug_log!(
            "File cache for provider '{}' is {} seconds old, fresh: {}",
            provider,
            age_seconds,
            is_fresh
        );

        // If file cache is fresh, populate in-memory cache
        if is_fresh {
            Self::populate_memory_cache(provider, cached_data);
        }

        Ok(is_fresh)
    }

    /// Populate in-memory cache with data
    fn populate_memory_cache(provider: &str, data: CachedProviderData) {
        if let Ok(mut cache) = MEMORY_CACHE.write() {
            let entry = MemoryCacheEntry::new(data, Self::CACHE_TTL);
            cache.insert(provider.to_string(), entry);
            debug_log!("Populated in-memory cache for provider '{}'", provider);
        }
    }

    /// Invalidate cache for a specific provider
    pub fn invalidate_provider_cache(provider: &str) {
        if let Ok(mut cache) = MEMORY_CACHE.write() {
            cache.remove(provider);
            debug_log!("Invalidated in-memory cache for provider '{}'", provider);
        }
    }

    /// Clear all in-memory cache
    #[allow(dead_code)]
    pub fn clear_memory_cache() {
        if let Ok(mut cache) = MEMORY_CACHE.write() {
            cache.clear();
            debug_log!("Cleared all in-memory cache");
        }
    }

    /// Get cache age in human-readable format (e.g., "5 mins ago", "2 hrs ago")
    pub async fn get_cache_age_display(provider: &str) -> Result<String> {
        // Check in-memory cache first
        if let Ok(cache) = MEMORY_CACHE.read() {
            if let Some(entry) = cache.get(provider) {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or(Duration::from_secs(0))
                    .as_secs();

                let age_seconds = now - entry.data.last_updated;
                return Ok(Self::format_age(age_seconds));
            }
        }

        // Fall back to file cache
        let cache_path = Self::provider_cache_path(provider)?;

        if !cache_path.exists() {
            return Ok("No cache".to_string());
        }

        let content = fs::read_to_string(&cache_path).await?;
        let cached_data: CachedProviderData = serde_json::from_str(&content)?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();

        let age_seconds = now - cached_data.last_updated;
        Ok(Self::format_age(age_seconds))
    }

    /// Format age in seconds to human-readable string
    fn format_age(age_seconds: u64) -> String {
        if age_seconds < 60 {
            format!("{} secs ago", age_seconds)
        } else if age_seconds < 3600 {
            let minutes = age_seconds / 60;
            format!("{} min{} ago", minutes, if minutes == 1 { "" } else { "s" })
        } else if age_seconds < 86400 {
            let hours = age_seconds / 3600;
            format!("{} hr{} ago", hours, if hours == 1 { "" } else { "s" })
        } else {
            let days = age_seconds / 86400;
            format!("{} day{} ago", days, if days == 1 { "" } else { "s" })
        }
    }

    /// Load cached models for a provider (async with in-memory cache)
    pub async fn load_provider_models(provider: &str) -> Result<Vec<ModelMetadata>> {
        debug_log!("Loading cached models for provider '{}'", provider);

        // Check in-memory cache first
        if let Ok(cache) = MEMORY_CACHE.read() {
            if let Some(entry) = cache.get(provider) {
                if !entry.is_expired() {
                    debug_log!(
                        "Loaded {} models from in-memory cache for provider '{}'",
                        entry.data.models.len(),
                        provider
                    );
                    return Ok(entry.data.models.clone());
                } else {
                    debug_log!("In-memory cache expired for provider '{}'", provider);
                }
            }
        }

        // Fall back to file cache
        let cache_path = Self::provider_cache_path(provider)?;

        if !cache_path.exists() {
            debug_log!("No cache file found for provider '{}'", provider);
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&cache_path).await?;
        let cached_data: CachedProviderData = serde_json::from_str(&content)?;

        debug_log!(
            "Loaded {} models from file cache for provider '{}'",
            cached_data.models.len(),
            provider
        );

        // Populate in-memory cache if data is fresh
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();

        if now - cached_data.last_updated < Self::CACHE_TTL {
            Self::populate_memory_cache(provider, cached_data.clone());
        }

        Ok(cached_data.models)
    }

    /// Fetch and cache models for a provider
    pub async fn fetch_and_cache_provider_models(
        provider: &str,
        force_refresh: bool,
    ) -> Result<Vec<ModelMetadata>> {
        debug_log!(
            "Fetching models for provider '{}', force_refresh: {}",
            provider,
            force_refresh
        );

        // Check if we need to refresh
        if !force_refresh && Self::is_cache_fresh(provider).await? {
            debug_log!(
                "Using cached models for provider '{}' (cache is fresh)",
                provider
            );
            return Self::load_provider_models(provider).await;
        }

        debug_log!(
            "Cache is stale or refresh forced, fetching fresh models for provider '{}'",
            provider
        );
        println!("Fetching models from provider '{}'...", provider);

        // Invalidate existing cache
        Self::invalidate_provider_cache(provider);

        // Load config and create client
        let config = Config::load()?;
        // Load provider with authentication (API key, headers, tokens) from centralized keys
        let provider_config = config.get_provider_with_auth(provider)?;

        debug_log!(
            "Creating authenticated client for provider '{}' with endpoint: {}",
            provider,
            provider_config.endpoint
        );

        let mut config_mut = config.clone();
        let client = crate::chat::create_authenticated_client(&mut config_mut, provider).await?;

        // Save config if tokens were updated
        if config_mut.get_cached_token(provider) != config.get_cached_token(provider) {
            debug_log!(
                "Tokens were updated for provider '{}', saving config",
                provider
            );
            config_mut.save()?;
        }

        // Build the models URL
        let models_url = format!(
            "{}{}",
            provider_config.endpoint, provider_config.models_path
        );
        debug_log!("Fetching models from URL: {}", models_url);

        // Fetch raw response using the client's list_models method
        debug_log!(
            "Making API request to fetch models from provider '{}'",
            provider
        );

        // Make the actual API request to fetch models
        let models_list = client.list_models().await?;

        // Create a JSON response that matches the OpenAI models format
        // This is what we'll cache as the "raw response"
        let models_json = serde_json::json!({
            "object": "list",
            "data": models_list.iter().map(|m| {
                serde_json::json!({
                    "id": m.id,
                    "object": m.object,
                    "providers": m.providers.iter().map(|p| {
                        serde_json::json!({
                            "provider": p.provider,
                            "status": p.status,
                            "supports_tools": p.supports_tools,
                            "supports_structured_output": p.supports_structured_output
                        })
                    }).collect::<Vec<_>>()
                })
            }).collect::<Vec<_>>()
        });

        let raw_response = serde_json::to_string_pretty(&models_json)?;

        debug_log!(
            "Received raw response from provider '{}' ({} bytes)",
            provider,
            raw_response.len()
        );

        // Debug log the full response when -d flag is used
        debug_log!(
            "Full response from provider '{}': {}",
            provider,
            raw_response
        );

        // Extract metadata using the new generic approach
        debug_log!(
            "Extracting metadata from response for provider '{}'",
            provider
        );

        // Offload the CPU-intensive extraction to a blocking thread
        let provider_clone = provider.to_string();
        let raw_response_clone = raw_response.clone();

        let models = tokio::task::spawn_blocking(move || {
            // Create a Provider object for the extractor
            let provider_obj = Provider {
                provider: provider_clone.clone(),
                status: "active".to_string(),
                supports_tools: false,
                supports_structured_output: false,
            };

            extract_models_from_provider(&provider_obj, &raw_response_clone)
        })
        .await??;

        debug_log!(
            "Extracted {} models from provider '{}'",
            models.len(),
            provider
        );

        // Cache the data (both in-memory and file)
        debug_log!("Saving cache data for provider '{}'", provider);
        Self::save_provider_cache(provider, &raw_response, &models).await?;

        Ok(models)
    }

    /// Save provider data to cache (async with in-memory caching)
    async fn save_provider_cache(
        provider: &str,
        raw_response: &str,
        models: &[ModelMetadata],
    ) -> Result<()> {
        let cache_path = Self::provider_cache_path(provider)?;

        debug_log!(
            "Saving cache for provider '{}' to: {}",
            provider,
            cache_path.display()
        );

        // Create cached data
        let cached_data = CachedProviderData::new(raw_response.to_string(), models.to_vec());

        // Update in-memory cache first (fastest access)
        Self::populate_memory_cache(provider, cached_data.clone());

        // Ensure cache directory exists
        if let Some(parent) = cache_path.parent() {
            debug_log!("Creating cache directory: {}", parent.display());
            fs::create_dir_all(parent).await?;
        }

        // Use async file I/O to avoid blocking
        let mut cached_data_mut = cached_data;
        let content = cached_data_mut.get_cached_json()?;
        debug_log!(
            "Writing {} bytes to cache file for provider '{}'",
            content.len(),
            provider
        );
        fs::write(&cache_path, content).await?;

        debug_log!(
            "Successfully saved cache for provider '{}' with {} models",
            provider,
            models.len()
        );

        Ok(())
    }

    /// Load all cached models from all providers (async with in-memory cache)
    pub async fn load_all_cached_models() -> Result<Vec<ModelMetadata>> {
        let models_dir = Self::models_dir()?;
        let mut all_models = Vec::new();

        if !models_dir.exists() {
            return Ok(all_models);
        }

        let mut entries = fs::read_dir(&models_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            if let Some(extension) = path.extension() {
                if extension == "json" {
                    if let Some(provider_name) = path.file_stem().and_then(|s| s.to_str()) {
                        match Self::load_provider_models(provider_name).await {
                            Ok(mut models) => {
                                all_models.append(&mut models);
                            }
                            Err(e) => {
                                eprintln!(
                                    "Warning: Failed to load cached models for {}: {}",
                                    provider_name, e
                                );
                            }
                        }
                    }
                }
            }
        }

        // Sort by provider, then by model name
        all_models.sort_by(|a, b| a.provider.cmp(&b.provider).then(a.id.cmp(&b.id)));

        Ok(all_models)
    }

    /// Refresh all providers' caches
    pub async fn refresh_all_providers() -> Result<()> {
        let config = Config::load()?;
        let mut successful_providers = 0;
        let mut total_models = 0;

        println!("Refreshing models cache for all providers...");

        for provider_name in config.providers.keys() {
            // Skip providers that have neither API key nor custom headers (after loading centralized auth)
            let pc_auth = match config.get_provider_with_auth(provider_name) {
                Ok(cfg) => cfg,
                Err(_) => continue,
            };
            if pc_auth.api_key.is_none() && pc_auth.headers.is_empty() {
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

        println!(
            "\nCache updated: {} providers, {} total models",
            successful_providers, total_models
        );
        Ok(())
    }
}
