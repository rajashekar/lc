use crate::{config::Config, provider::OpenAIClient};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelsCache {
    pub last_updated: u64,                    // Unix timestamp
    pub models: HashMap<String, Vec<String>>, // provider -> models
    // Cache the serialized JSON to avoid repeated serialization
    #[serde(skip)]
    pub cached_json: Option<String>,
}

#[derive(Debug)]
pub struct CachedModel {
    pub provider: String,
    pub model: String,
}

impl ModelsCache {
    pub fn new() -> Self {
        Self {
            last_updated: 0,
            models: HashMap::new(),
            cached_json: None,
        }
    }

    fn invalidate_cache(&mut self) {
        self.cached_json = None;
    }

    fn get_cached_json(&mut self) -> Result<&str> {
        if self.cached_json.is_none() {
            self.cached_json = Some(serde_json::to_string_pretty(self)?);
        }
        Ok(self.cached_json.as_ref().unwrap())
    }

    pub fn load() -> Result<Self> {
        let cache_path = Self::cache_file_path()?;

        if cache_path.exists() {
            let content = fs::read_to_string(&cache_path)?;
            let cache: ModelsCache = serde_json::from_str(&content)?;
            Ok(cache)
        } else {
            Ok(Self::new())
        }
    }

    pub fn save(&mut self) -> Result<()> {
        let cache_path = Self::cache_file_path()?;

        // Ensure cache directory exists
        if let Some(parent) = cache_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Use cached JSON if available to avoid re-serialization
        let content = self.get_cached_json()?;
        fs::write(&cache_path, content)?;
        Ok(())
    }

    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();

        // Cache expires after 24 hours (86400 seconds)
        now - self.last_updated > 86400
    }

    pub fn needs_refresh(&self) -> bool {
        self.models.is_empty() || self.is_expired()
    }

    pub async fn refresh(&mut self) -> Result<()> {
        println!("Refreshing models cache...");

        let config = Config::load()?;
        let mut new_models = HashMap::new();
        let mut successful_providers = 0;
        let mut total_models = 0;

        for (provider_name, provider_config) in &config.providers {
            // Skip providers without API keys
            if provider_config.api_key.is_none() {
                continue;
            }

            print!("Fetching models from {}... ", provider_name);

            let client = OpenAIClient::new_with_headers(
                provider_config.endpoint.clone(),
                provider_config.api_key.clone().unwrap(),
                provider_config.models_path.clone(),
                provider_config.chat_path.clone(),
                provider_config.headers.clone(),
            );

            match client.list_models().await {
                Ok(models) => {
                    let model_names: Vec<String> = models.into_iter().map(|m| m.id).collect();
                    let count = model_names.len();
                    new_models.insert(provider_name.clone(), model_names);
                    successful_providers += 1;
                    total_models += count;
                    println!("✓ ({} models)", count);
                }
                Err(e) => {
                    println!("✗ ({})", e);
                }
            }
        }

        self.models = new_models;
        self.last_updated = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();

        // Invalidate cached JSON since data changed
        self.invalidate_cache();
        self.save()?;

        println!(
            "\nCache updated: {} providers, {} total models",
            successful_providers, total_models
        );
        Ok(())
    }

    pub fn get_all_models(&self) -> Vec<CachedModel> {
        let mut all_models = Vec::new();

        for (provider, models) in &self.models {
            for model in models {
                all_models.push(CachedModel {
                    provider: provider.clone(),
                    model: model.clone(),
                });
            }
        }

        // Sort by provider, then by model
        all_models.sort_by(|a, b| a.provider.cmp(&b.provider).then(a.model.cmp(&b.model)));

        all_models
    }

    fn cache_file_path() -> Result<PathBuf> {
        let config_dir =
            dirs::config_dir().ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;

        Ok(config_dir.join("lc").join("models_cache.json"))
    }
}
