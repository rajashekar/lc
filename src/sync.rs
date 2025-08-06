//! Configuration synchronization module
//! 
//! This module provides functionality to sync configuration files to and from cloud providers.
//! Currently supports Amazon S3 with optional AES256 encryption.

pub mod config;
pub mod encryption;
pub mod providers;

// Re-export all public items from submodules
pub use config::*;
pub use encryption::*;
pub use providers::*;

use anyhow::Result;
use colored::Colorize;
use std::fs;
use std::path::PathBuf;
use std::io::{self, Write};
use rpassword::read_password;

/// Supported cloud providers
#[derive(Debug, Clone)]
pub enum CloudProvider {
    S3,
}

impl CloudProvider {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "s3" | "amazon-s3" | "aws-s3" => Ok(CloudProvider::S3),
            _ => anyhow::bail!("Unsupported cloud provider: '{}'. Supported providers: s3", s),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            CloudProvider::S3 => "s3",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            CloudProvider::S3 => "Amazon S3",
        }
    }
}

/// Configuration file information
#[derive(Debug, Clone)]
pub struct ConfigFile {
    pub name: String,
    pub path: PathBuf,
    pub content: Vec<u8>,
}

/// Cross-platform configuration directory resolver
pub struct ConfigResolver;

impl ConfigResolver {
    /// Get the configuration directory for the current platform
    pub fn get_config_dir() -> Result<PathBuf> {
        crate::config::Config::config_dir()
    }

    /// Get all .toml configuration files and logs.db in the lc config directory
    pub fn get_config_files() -> Result<Vec<ConfigFile>> {
        let config_dir = Self::get_config_dir()?;
        let mut config_files = Vec::new();

        if !config_dir.exists() {
            return Ok(config_files);
        }

        // Read all .toml files and logs.db in the config directory
        for entry in fs::read_dir(&config_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                let should_include = if let Some(extension) = path.extension() {
                    extension == "toml"
                } else {
                    // Include files without extension if they are logs.db
                    path.file_name()
                        .and_then(|n| n.to_str())
                        .map(|name| name == "logs.db")
                        .unwrap_or(false)
                };

                if should_include {
                    let content = fs::read(&path)?;
                    let name = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string();
                    
                    config_files.push(ConfigFile {
                        name,
                        path: path.clone(),
                        content,
                    });
                }
            }
        }

        Ok(config_files)
    }

    /// Write configuration files back to the config directory
    pub fn write_config_files(files: &[ConfigFile]) -> Result<()> {
        let config_dir = Self::get_config_dir()?;
        fs::create_dir_all(&config_dir)?;

        for file in files {
            let target_path = config_dir.join(&file.name);
            fs::write(&target_path, &file.content)?;
            println!("{} Restored: {}", "✓".green(), file.name);
        }

        Ok(())
    }
}

/// Handle sync providers command
pub async fn handle_sync_providers() -> Result<()> {
    println!("\n{}", "Supported Cloud Providers:".bold().blue());
    println!("  {} {} - Amazon Simple Storage Service", "•".blue(), "s3".bold());
    
    println!("\n{}", "Usage:".bold().blue());
    println!("  {} Sync to cloud: {}", "•".blue(), "lc sync to s3".dimmed());
    println!("  {} Sync from cloud: {}", "•".blue(), "lc sync from s3".dimmed());
    println!("  {} With encryption: {}", "•".blue(), "lc sync to s3 --encrypted".dimmed());
    
    println!("\n{}", "What gets synced:".bold().blue());
    println!("  {} Configuration files (*.toml)", "•".blue());
    println!("  {} Chat logs database (logs.db)", "•".blue());
    
    println!("\n{}", "Configuration:".bold().blue());
    println!("  {} S3 credentials can be provided via:", "•".blue());
    println!("    - Environment variables (recommended):");
    println!("      {} LC_S3_BUCKET=your-bucket-name", "export".dimmed());
    println!("      {} LC_S3_REGION=us-east-1", "export".dimmed());
    println!("      {} AWS_ACCESS_KEY_ID=your-access-key", "export".dimmed());
    println!("      {} AWS_SECRET_ACCESS_KEY=your-secret-key", "export".dimmed());
    println!("      {} LC_S3_ENDPOINT=https://s3.amazonaws.com  # Optional", "export".dimmed());
    println!("    - Interactive prompts during sync (if env vars not set)");
    
    println!("\n{}", "S3-Compatible Services:".bold().blue());
    println!("  {} Supports AWS S3, Backblaze B2, Cloudflare R2, and other S3-compatible services", "•".blue());
    println!("  {} Set LC_S3_ENDPOINT for non-AWS services:", "•".blue());
    println!("    - Backblaze B2: {}", "https://s3.us-west-004.backblazeb2.com".dimmed());
    println!("    - Cloudflare R2: {}", "https://your-account-id.r2.cloudflarestorage.com".dimmed());
    
    println!("\n{}", "Database Management:".bold().blue());
    println!("  {} Purge old logs: {}", "•".blue(), "lc logs purge --older-than-days 30".dimmed());
    println!("  {} Keep recent logs: {}", "•".blue(), "lc logs purge --keep-recent 1000".dimmed());
    println!("  {} Size-based purge: {}", "•".blue(), "lc logs purge --max-size-mb 50".dimmed());
    
    Ok(())
}

/// Handle sync to cloud command
pub async fn handle_sync_to(provider_name: &str, encrypted: bool) -> Result<()> {
    let provider = CloudProvider::from_str(provider_name)?;
    
    println!("{} Starting sync to {} ({})", "🔄".blue(), provider.display_name(), provider.name());
    
    // Get configuration files
    let config_files = ConfigResolver::get_config_files()?;
    
    if config_files.is_empty() {
        println!("{} No configuration files found to sync", "⚠️".yellow());
        return Ok(());
    }
    
    println!("{} Found {} files to sync:", "📁".blue(), config_files.len());
    for file in &config_files {
        let file_type = if file.name.ends_with(".toml") {
            "config"
        } else if file.name == "logs.db" {
            "database"
        } else {
            "file"
        };
        let size_kb = (file.content.len() + 1023) / 1024; // Round up to KB
        println!("  {} {} ({}, {} KB)", "•".blue(), file.name, file_type, size_kb);
    }
    
    // Handle encryption if requested
    let files_to_upload = if encrypted {
        println!("\n{} Encryption enabled", "🔒".yellow());
        print!("Enter encryption password: ");
        io::stdout().flush()?;
        let password = read_password()?;
        
        if password.is_empty() {
            anyhow::bail!("Password cannot be empty");
        }
        
        let key = derive_key_from_password(&password)?;
        let mut encrypted_files = Vec::new();
        
        for file in &config_files {
            let encrypted_content = encrypt_data(&file.content, &key)?;
            let encrypted_file = ConfigFile {
                name: format!("{}.enc", file.name),
                path: file.path.clone(),
                content: encrypted_content,
            };
            encrypted_files.push(encrypted_file);
        }
        
        encrypted_files
    } else {
        config_files
    };
    
    // Upload to cloud provider
    match provider {
        CloudProvider::S3 => {
            let s3_client = S3Provider::new().await?;
            s3_client.upload_configs(&files_to_upload, encrypted).await?;
        }
    }
    
    println!("\n{} Sync to {} completed successfully!", "🎉".green(), provider.display_name());
    
    if encrypted {
        println!("{} Files were encrypted before upload", "🔒".green());
    }
    
    Ok(())
}

/// Handle sync from cloud command
pub async fn handle_sync_from(provider_name: &str, encrypted: bool) -> Result<()> {
    let provider = CloudProvider::from_str(provider_name)?;
    
    println!("{} Starting sync from {} ({})", "🔄".blue(), provider.display_name(), provider.name());
    
    // Download from cloud provider
    let downloaded_files = match provider {
        CloudProvider::S3 => {
            let s3_client = S3Provider::new().await?;
            s3_client.download_configs(encrypted).await?
        }
    };
    
    if downloaded_files.is_empty() {
        println!("{} No configuration files found in cloud storage", "⚠️".yellow());
        return Ok(());
    }
    
    println!("{} Downloaded {} files:", "📥".blue(), downloaded_files.len());
    for file in &downloaded_files {
        let file_type = if file.name.ends_with(".toml") {
            "config"
        } else if file.name == "logs.db" {
            "database"
        } else {
            "file"
        };
        let size_kb = (file.content.len() + 1023) / 1024; // Round up to KB
        println!("  {} {} ({}, {} KB)", "•".blue(), file.name, file_type, size_kb);
    }
    
    // Handle decryption if needed
    let final_files = if encrypted {
        println!("\n{} Decryption enabled", "🔓".yellow());
        print!("Enter decryption password: ");
        io::stdout().flush()?;
        let password = read_password()?;
        
        if password.is_empty() {
            anyhow::bail!("Password cannot be empty");
        }
        
        let key = derive_key_from_password(&password)?;
        let mut decrypted_files = Vec::new();
        
        for file in &downloaded_files {
            // Remove .enc extension if present
            let original_name = if file.name.ends_with(".enc") {
                file.name.strip_suffix(".enc").unwrap().to_string()
            } else {
                file.name.clone()
            };
            
            let decrypted_content = decrypt_data(&file.content, &key)
                .map_err(|e| anyhow::anyhow!("Failed to decrypt {}: {}. Check your password.", file.name, e))?;
            
            let decrypted_file = ConfigFile {
                name: original_name,
                path: file.path.clone(),
                content: decrypted_content,
            };
            decrypted_files.push(decrypted_file);
        }
        
        decrypted_files
    } else {
        downloaded_files
    };
    
    // Write files to local config directory
    ConfigResolver::write_config_files(&final_files)?;
    
    println!("\n{} Sync from {} completed successfully!", "🎉".green(), provider.display_name());
    
    if encrypted {
        println!("{} Files were decrypted after download", "🔓".green());
    }
    
    let config_dir = ConfigResolver::get_config_dir()?;
    println!("{} Configuration files restored to: {}", "📁".blue(), config_dir.display());
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cloud_provider_from_str() {
        assert!(matches!(CloudProvider::from_str("s3"), Ok(CloudProvider::S3)));
        assert!(matches!(CloudProvider::from_str("S3"), Ok(CloudProvider::S3)));
        assert!(matches!(CloudProvider::from_str("amazon-s3"), Ok(CloudProvider::S3)));
        assert!(CloudProvider::from_str("invalid").is_err());
    }

    #[test]
    fn test_cloud_provider_names() {
        let s3 = CloudProvider::S3;
        assert_eq!(s3.name(), "s3");
        assert_eq!(s3.display_name(), "Amazon S3");
    }
}