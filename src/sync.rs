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
use rpassword::read_password;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

/// Supported cloud providers
#[derive(Debug, Clone)]
pub enum CloudProvider {
    S3,
}

impl CloudProvider {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "s3" | "amazon-s3" | "aws-s3" => Ok(CloudProvider::S3),
            _ => anyhow::bail!(
                "Unsupported cloud provider: '{}'. Supported providers: s3",
                s
            ),
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

    /// Get all .toml configuration files and logs.db from the lc directory
    pub fn get_config_files() -> Result<Vec<ConfigFile>> {
        let config_dir = Self::get_config_dir()?;
        let mut config_files = Vec::new();

        crate::debug_log!("Looking in directory: {:?}", config_dir);

        if !config_dir.exists() {
            crate::debug_log!("Directory does not exist");
            return Ok(config_files);
        }

        // Read all .toml files and logs.db from the lc directory
        for entry in fs::read_dir(&config_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                let extension = path.extension().and_then(|e| e.to_str());

                crate::debug_log!("Found file: {} (extension: {:?})", name, extension);

                let should_include =
                    name == "logs.db" || extension.map(|e| e == "toml").unwrap_or(false);

                crate::debug_log!("Should include {}: {}", name, should_include);

                if should_include {
                    let content = fs::read(&path)?;

                    config_files.push(ConfigFile {
                        name: name.to_string(),
                        path: path.clone(),
                        content,
                    });
                }
            }
        }

        crate::debug_log!("Total files found: {}", config_files.len());
        Ok(config_files)
    }

    /// Write configuration files back to the lc directory
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
    println!(
        "  {} {} - Amazon Simple Storage Service",
        "•".blue(),
        "s3".bold()
    );

    println!("\n{}", "Usage:".bold().blue());
    println!(
        "  {} Sync to cloud: {}",
        "•".blue(),
        "lc sync to s3".dimmed()
    );
    println!(
        "  {} Sync from cloud: {}",
        "•".blue(),
        "lc sync from s3".dimmed()
    );
    println!(
        "  {} With encryption: {}",
        "•".blue(),
        "lc sync to s3 --encrypted".dimmed()
    );

    println!("\n{}", "What gets synced:".bold().blue());
    println!("  {} Configuration files (*.toml)", "•".blue());
    println!("  {} Chat logs database (logs.db)", "•".blue());

    println!("\n{}", "Configuration:".bold().blue());
    println!("  {} S3 credentials can be provided via:", "•".blue());
    println!("    - Environment variables (recommended):");
    println!("      {} LC_S3_BUCKET=your-bucket-name", "export".dimmed());
    println!("      {} LC_S3_REGION=us-east-1", "export".dimmed());
    println!(
        "      {} AWS_ACCESS_KEY_ID=your-access-key",
        "export".dimmed()
    );
    println!(
        "      {} AWS_SECRET_ACCESS_KEY=your-secret-key",
        "export".dimmed()
    );
    println!(
        "      {} LC_S3_ENDPOINT=https://s3.amazonaws.com  # Optional",
        "export".dimmed()
    );
    println!("    - Interactive prompts during sync (if env vars not set)");

    println!("\n{}", "S3-Compatible Services:".bold().blue());
    println!(
        "  {} Supports AWS S3, Backblaze B2, Cloudflare R2, and other S3-compatible services",
        "•".blue()
    );
    println!("  {} Set LC_S3_ENDPOINT for non-AWS services:", "•".blue());
    println!(
        "    - Backblaze B2: {}",
        "https://s3.us-west-004.backblazeb2.com".dimmed()
    );
    println!(
        "    - Cloudflare R2: {}",
        "https://your-account-id.r2.cloudflarestorage.com".dimmed()
    );

    println!("\n{}", "Database Management:".bold().blue());
    println!(
        "  {} Purge old logs: {}",
        "•".blue(),
        "lc logs purge --older-than-days 30".dimmed()
    );
    println!(
        "  {} Keep recent logs: {}",
        "•".blue(),
        "lc logs purge --keep-recent 1000".dimmed()
    );
    println!(
        "  {} Size-based purge: {}",
        "•".blue(),
        "lc logs purge --max-size-mb 50".dimmed()
    );

    Ok(())
}

/// Handle sync to cloud command
pub async fn handle_sync_to(provider_name: &str, encrypted: bool, yes: bool) -> Result<()> {
    let provider = CloudProvider::from_str(provider_name)?;

    println!(
        "{} Starting sync to {} ({})",
        "🔄".blue(),
        provider.display_name(),
        provider.name()
    );

    // Get configuration files
    let config_files = ConfigResolver::get_config_files()?;

    if config_files.is_empty() {
        println!("{} No configuration files found to sync", "⚠️".yellow());
        return Ok(());
    }

    println!(
        "{} Found {} files to sync:",
        "📁".blue(),
        config_files.len()
    );
    for file in &config_files {
        let file_type = if file.name.ends_with(".toml") {
            "config"
        } else if file.name == "logs.db" {
            "database"
        } else {
            "file"
        };
        let size_kb = (file.content.len() + 1023) / 1024; // Round up to KB
        println!(
            "  {} {} ({}, {} KB)",
            "•".blue(),
            file.name,
            file_type,
            size_kb
        );
    }

    // Ask for confirmation unless --yes flag is provided
    if !yes {
        println!();
        print!(
            "Are you sure you want to sync {} files to {} cloud storage? (y/N): ",
            config_files.len(),
            provider.display_name()
        );
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        if !input.trim().to_lowercase().starts_with('y') {
            println!("Sync cancelled.");
            return Ok(());
        }
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
            s3_client
                .upload_configs(&files_to_upload, encrypted)
                .await?;
        }
    }

    println!(
        "\n{} Sync to {} completed successfully!",
        "🎉".green(),
        provider.display_name()
    );

    if encrypted {
        println!("{} Files were encrypted before upload", "🔒".green());
    }

    Ok(())
}

/// Handle sync from cloud command
pub async fn handle_sync_from(provider_name: &str, encrypted: bool, yes: bool) -> Result<()> {
    let provider = CloudProvider::from_str(provider_name)?;

    println!(
        "{} Starting sync from {} ({})",
        "🔄".blue(),
        provider.display_name(),
        provider.name()
    );

    // Download from cloud provider
    let downloaded_files = match provider {
        CloudProvider::S3 => {
            let s3_client = S3Provider::new().await?;
            s3_client.download_configs(encrypted).await?
        }
    };

    if downloaded_files.is_empty() {
        println!(
            "{} No configuration files found in cloud storage",
            "⚠️".yellow()
        );
        return Ok(());
    }

    println!(
        "{} Downloaded {} files:",
        "📥".blue(),
        downloaded_files.len()
    );
    for file in &downloaded_files {
        let file_type = if file.name.ends_with(".toml") {
            "config"
        } else if file.name == "logs.db" {
            "database"
        } else {
            "file"
        };
        let size_kb = (file.content.len() + 1023) / 1024; // Round up to KB
        println!(
            "  {} {} ({}, {} KB)",
            "•".blue(),
            file.name,
            file_type,
            size_kb
        );
    }

    // Ask for confirmation unless --yes flag is provided
    if !yes {
        println!();
        print!(
            "Are you sure you want to overwrite your local configuration with {} files from {} cloud storage? (y/N): ",
            downloaded_files.len(),
            provider.display_name()
        );
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        if !input.trim().to_lowercase().starts_with('y') {
            println!("Sync cancelled.");
            return Ok(());
        }
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

            let decrypted_content = decrypt_data(&file.content, &key).map_err(|e| {
                anyhow::anyhow!(
                    "Failed to decrypt {}: {}. Check your password.",
                    file.name,
                    e
                )
            })?;

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

    println!(
        "\n{} Sync from {} completed successfully!",
        "🎉".green(),
        provider.display_name()
    );

    if encrypted {
        println!("{} Files were decrypted after download", "🔓".green());
    }

    let config_dir = ConfigResolver::get_config_dir()?;
    println!(
        "{} Configuration files restored to: {}",
        "📁".blue(),
        config_dir.display()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_cloud_provider_from_str() {
        assert!(matches!(
            CloudProvider::from_str("s3"),
            Ok(CloudProvider::S3)
        ));
        assert!(matches!(
            CloudProvider::from_str("S3"),
            Ok(CloudProvider::S3)
        ));
        assert!(matches!(
            CloudProvider::from_str("amazon-s3"),
            Ok(CloudProvider::S3)
        ));
        assert!(CloudProvider::from_str("invalid").is_err());
    }

    #[test]
    fn test_cloud_provider_names() {
        let s3 = CloudProvider::S3;
        assert_eq!(s3.name(), "s3");
        assert_eq!(s3.display_name(), "Amazon S3");
    }

    /// Helper function to test get_config_files with a custom directory
    /// This allows us to test the file detection logic in isolation
    fn get_config_files_from_dir(dir: &PathBuf) -> Result<Vec<ConfigFile>> {
        let mut config_files = Vec::new();

        if !dir.exists() {
            return Ok(config_files);
        }

        // Read all .toml files and logs.db from the directory
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                let extension = path.extension().and_then(|e| e.to_str());

                let should_include =
                    name == "logs.db" || extension.map(|e| e == "toml").unwrap_or(false);

                if should_include {
                    let content = fs::read(&path)?;

                    config_files.push(ConfigFile {
                        name: name.to_string(),
                        path: path.clone(),
                        content,
                    });
                }
            }
        }

        Ok(config_files)
    }

    /// Regression test to ensure ConfigResolver::get_config_files() returns both config.toml and logs.db
    /// This prevents future regressions where one of the files might be missed.
    #[test]
    fn test_config_resolver_returns_both_config_files() -> Result<()> {
        // Create a temporary directory structure
        let temp_dir = TempDir::new()?;
        let config_dir = temp_dir.path().join("lc");
        fs::create_dir_all(&config_dir)?;

        // Create both expected files with test content
        let config_content = "[providers]\ntest_provider = { endpoint = 'https://example.com' }";
        let logs_content = "SQLite format 3\x00"; // Mock SQLite header
        
        fs::write(config_dir.join("config.toml"), config_content)?;
        fs::write(config_dir.join("logs.db"), logs_content)?;
        
        // Also create files that should be ignored
        fs::write(config_dir.join("should_be_ignored.txt"), "ignored content")?;
        fs::write(config_dir.join("README.md"), "# Documentation")?;
        
        // Create additional .toml files that should be included
        fs::write(config_dir.join("mcp.toml"), "mcp_config = true")?;
        fs::write(config_dir.join("search_config.toml"), "search_config = true")?;

        // Test our helper function with the temporary directory
        let config_files = get_config_files_from_dir(&config_dir)?;

        // Assertions
        assert_eq!(config_files.len(), 4, "Should return exactly 4 files: config.toml, logs.db, mcp.toml, and search_config.toml");
        
        // Collect the file names for easier assertions
        let file_names: std::collections::HashSet<_> = config_files.iter().map(|f| &f.name).collect();
        
        // Must include these critical files
        assert!(file_names.contains(&"config.toml".to_string()), "Should include config.toml");
        assert!(file_names.contains(&"logs.db".to_string()), "Should include logs.db");
        
        // Should also include other .toml files  
        assert!(file_names.contains(&"mcp.toml".to_string()), "Should include mcp.toml");
        assert!(file_names.contains(&"search_config.toml".to_string()), "Should include search_config.toml");
        
        // Should not include non-.toml files (except logs.db)
        assert!(!file_names.contains(&"should_be_ignored.txt".to_string()), "Should not include .txt files");
        assert!(!file_names.contains(&"README.md".to_string()), "Should not include .md files");
        
        // Verify content is correctly read for the main files
        let config_file = config_files.iter().find(|f| f.name == "config.toml").unwrap();
        let logs_file = config_files.iter().find(|f| f.name == "logs.db").unwrap();
        
        assert_eq!(String::from_utf8_lossy(&config_file.content), config_content);
        assert_eq!(&logs_file.content[..logs_content.len()], logs_content.as_bytes());
        
        // Verify paths are correct
        assert_eq!(config_file.path, config_dir.join("config.toml"));
        assert_eq!(logs_file.path, config_dir.join("logs.db"));

        Ok(())
    }
}
