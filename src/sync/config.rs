//! Sync configuration management for storing cloud provider settings

use anyhow::Result;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Sync configuration for all providers
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SyncConfig {
    pub providers: HashMap<String, ProviderConfig>,
}

/// Configuration for a specific cloud provider
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum ProviderConfig {
    #[serde(rename = "s3")]
    S3 {
        bucket_name: String,
        region: String,
        access_key_id: String,
        secret_access_key: String,
        endpoint_url: Option<String>,
    },
}

impl SyncConfig {
    /// Load sync configuration from file
    pub fn load() -> Result<Self> {
        let config_path = Self::config_file_path()?;

        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            let config: SyncConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(SyncConfig::default())
        }
    }

    /// Save sync configuration to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_file_path()?;

        // Ensure config directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        fs::write(&config_path, content)?;
        Ok(())
    }

    /// Get the path to the sync configuration file
    fn config_file_path() -> Result<PathBuf> {
        let config_dir = crate::config::Config::config_dir()?;
        Ok(config_dir.join("sync.toml"))
    }

    /// Add or update a provider configuration
    pub fn set_provider(&mut self, name: String, config: ProviderConfig) {
        self.providers.insert(name, config);
    }

    /// Get a provider configuration
    pub fn get_provider(&self, name: &str) -> Option<&ProviderConfig> {
        self.providers.get(name)
    }

    /// Remove a provider configuration
    pub fn remove_provider(&mut self, name: &str) -> bool {
        self.providers.remove(name).is_some()
    }
}

impl ProviderConfig {
    /// Create a new S3 provider configuration
    pub fn new_s3(
        bucket_name: String,
        region: String,
        access_key_id: String,
        secret_access_key: String,
        endpoint_url: Option<String>,
    ) -> Self {
        ProviderConfig::S3 {
            bucket_name,
            region,
            access_key_id,
            secret_access_key,
            endpoint_url,
        }
    }

    /// Display provider configuration (hiding sensitive data)
    pub fn display(&self) -> String {
        match self {
            ProviderConfig::S3 {
                bucket_name,
                region,
                access_key_id,
                endpoint_url,
                ..
            } => {
                let mut info = format!(
                    "S3 Configuration:\n  Bucket: {}\n  Region: {}\n  Access Key: {}***",
                    bucket_name,
                    region,
                    &access_key_id[..access_key_id.len().min(8)]
                );

                if let Some(endpoint) = endpoint_url {
                    info.push_str(&format!("\n  Endpoint: {}", endpoint));
                }

                info
            }
        }
    }
}

/// Handle sync configure command
pub async fn handle_sync_configure(
    provider_name: &str,
    command: Option<crate::cli::ConfigureCommands>,
) -> Result<()> {
    use crate::cli::ConfigureCommands;

    match command {
        Some(ConfigureCommands::Setup) | None => {
            // Setup provider configuration
            match provider_name.to_lowercase().as_str() {
                "s3" | "amazon-s3" | "aws-s3" | "cloudflare" | "backblaze" => {
                    setup_s3_config(provider_name).await?;
                }
                _ => {
                    anyhow::bail!(
                        "Unsupported provider '{}'. Supported providers: s3, cloudflare, backblaze",
                        provider_name
                    );
                }
            }
        }
        Some(ConfigureCommands::Show) => {
            // Show provider configuration
            let config = SyncConfig::load()?;

            if let Some(provider_config) = config.get_provider(provider_name) {
                println!(
                    "\n{}",
                    format!("Configuration for '{}':", provider_name)
                        .bold()
                        .blue()
                );
                println!("{}", provider_config.display());
            } else {
                println!(
                    "{} No configuration found for provider '{}'",
                    "ℹ️".blue(),
                    provider_name
                );
                println!(
                    "Run {} to set up configuration",
                    format!("lc sync configure {} setup", provider_name).dimmed()
                );
            }
        }
        Some(ConfigureCommands::Remove) => {
            // Remove provider configuration
            let mut config = SyncConfig::load()?;

            if config.remove_provider(provider_name) {
                config.save()?;
                println!(
                    "{} Configuration for '{}' removed successfully",
                    "✓".green(),
                    provider_name
                );
            } else {
                println!(
                    "{} No configuration found for provider '{}'",
                    "ℹ️".blue(),
                    provider_name
                );
            }
        }
        Some(ConfigureCommands::Provider { provider }) => {
            println!("Setting cloud provider to: {}", provider);
            // TODO: Implement provider configuration
        }
        Some(ConfigureCommands::S3 {
            bucket,
            region,
            endpoint,
        }) => {
            println!("Setting S3 bucket to: {}", bucket);
            println!("Setting S3 region to: {}", region);
            if let Some(endpoint_url) = endpoint {
                println!("Setting S3 endpoint to: {}", endpoint_url);
            }
            // TODO: Implement S3 configuration
        }
        Some(ConfigureCommands::Gcs { bucket, key_file }) => {
            println!("Setting GCS bucket to: {}", bucket);
            if let Some(key_path) = key_file {
                println!("Setting GCS key file to: {}", key_path);
            }
            // TODO: Implement GCS configuration
        }
    }

    Ok(())
}

/// Setup S3 configuration interactively
async fn setup_s3_config(provider_name: &str) -> Result<()> {
    use std::io::{self, Write};

    println!(
        "{} Setting up S3 configuration for '{}'",
        "🔧".blue(),
        provider_name
    );
    println!(
        "{} This will be stored in your lc config directory",
        "ℹ️".blue()
    );
    println!();

    // Get bucket name
    print!("Enter S3 bucket name: ");
    // Deliberately flush stdout to ensure prompt appears before user input
    io::stdout().flush()?;
    let mut bucket_name = String::new();
    io::stdin().read_line(&mut bucket_name)?;
    let bucket_name = bucket_name.trim().to_string();
    if bucket_name.is_empty() {
        anyhow::bail!("Bucket name cannot be empty");
    }

    // Get region
    print!("Enter AWS region (default: us-east-1): ");
    // Deliberately flush stdout to ensure prompt appears before user input
    io::stdout().flush()?;
    let mut region = String::new();
    io::stdin().read_line(&mut region)?;
    let region = region.trim().to_string();
    let region = if region.is_empty() {
        "us-east-1".to_string()
    } else {
        region
    };

    // Get access key ID
    print!("Enter AWS Access Key ID: ");
    // Deliberately flush stdout to ensure prompt appears before user input
    io::stdout().flush()?;
    let mut access_key_id = String::new();
    io::stdin().read_line(&mut access_key_id)?;
    let access_key_id = access_key_id.trim().to_string();
    if access_key_id.is_empty() {
        anyhow::bail!("Access Key ID cannot be empty");
    }

    // Get secret access key (hidden input)
    print!("Enter AWS Secret Access Key: ");
    // Deliberately flush stdout to ensure prompt appears before password input
    io::stdout().flush()?;
    let secret_access_key = rpassword::read_password()?;
    if secret_access_key.is_empty() {
        anyhow::bail!("Secret Access Key cannot be empty");
    }

    // Get optional endpoint URL
    print!("Enter custom S3 endpoint URL (optional, for Backblaze/Cloudflare R2/etc., press Enter to skip): ");
    // Deliberately flush stdout to ensure prompt appears before user input
    io::stdout().flush()?;
    let mut endpoint_url = String::new();
    io::stdin().read_line(&mut endpoint_url)?;
    let endpoint_url = endpoint_url.trim().to_string();
    let endpoint_url = if endpoint_url.is_empty() {
        None
    } else {
        Some(endpoint_url)
    };

    // Create and save configuration
    let provider_config = ProviderConfig::new_s3(
        bucket_name.clone(),
        region.clone(),
        access_key_id.clone(),
        secret_access_key,
        endpoint_url.clone(),
    );

    let mut config = SyncConfig::load()?;
    config.set_provider(provider_name.to_string(), provider_config);
    config.save()?;

    println!(
        "\n{} S3 configuration for '{}' saved successfully!",
        "✓".green(),
        provider_name
    );
    println!("{} Configuration details:", "📋".blue());
    println!("  Bucket: {}", bucket_name);
    println!("  Region: {}", region);
    println!(
        "  Access Key: {}***",
        &access_key_id[..access_key_id.len().min(8)]
    );
    if let Some(endpoint) = endpoint_url {
        println!("  Endpoint: {}", endpoint);
    }

    println!("\n{} You can now use:", "💡".yellow());
    println!(
        "  {} - Sync to {}",
        format!("lc sync to {}", provider_name).dimmed(),
        provider_name
    );
    println!(
        "  {} - Sync from {}",
        format!("lc sync from {}", provider_name).dimmed(),
        provider_name
    );
    println!(
        "  {} - View configuration",
        format!("lc sync configure {} show", provider_name).dimmed()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_config_creation() {
        let config = ProviderConfig::new_s3(
            "test-bucket".to_string(),
            "us-east-1".to_string(),
            "test-key".to_string(),
            "test-secret".to_string(),
            None,
        );

        // Test that the config was created successfully
        assert!(matches!(config, ProviderConfig::S3 { .. }));
        assert!(config.display().contains("test-bucket"));
        assert!(config.display().contains("us-east-1"));
    }

    #[test]
    fn test_sync_config_operations() {
        let mut config = SyncConfig::default();

        let provider_config = ProviderConfig::new_s3(
            "test-bucket".to_string(),
            "us-east-1".to_string(),
            "test-key".to_string(),
            "test-secret".to_string(),
            None,
        );

        // Test adding provider
        config.set_provider("s3".to_string(), provider_config);
        assert!(config.get_provider("s3").is_some());
        assert_eq!(config.providers.len(), 1);

        // Test getting provider
        let retrieved = config.get_provider("s3");
        assert!(retrieved.is_some());

        // Test removing provider
        assert!(config.remove_provider("s3"));
        assert!(config.get_provider("s3").is_none());
        assert_eq!(config.providers.len(), 0);
    }
}
