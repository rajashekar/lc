//! Sync command handlers for configuration synchronization

use anyhow::Result;
use colored::*;

/// Configuration file structure for sync operations
#[derive(Debug, Clone)]
pub struct ConfigFile {
    pub name: String,
    pub content: Vec<u8>,
}

/// Encrypt multiple configuration files
pub fn encrypt_files(config_files: &[ConfigFile]) -> Result<Vec<ConfigFile>> {
    use super::encryption::{derive_key_from_password, encrypt_data};

    // Get encryption password from environment or prompt
    let password = std::env::var("LC_SYNC_PASSWORD").unwrap_or_else(|_| {
        rpassword::prompt_password("Enter sync encryption password: ")
            .expect("Failed to read password")
    });

    let key = derive_key_from_password(&password)?;

    let mut encrypted_files = Vec::new();
    for file in config_files {
        let encrypted_content = encrypt_data(&file.content, &key)?;
        encrypted_files.push(ConfigFile {
            name: file.name.clone(),
            content: encrypted_content,
        });
    }

    Ok(encrypted_files)
}

/// Decrypt multiple configuration files
pub fn decrypt_files(encrypted_files: &[ConfigFile]) -> Result<Vec<ConfigFile>> {
    use super::encryption::{decrypt_data, derive_key_from_password};

    // Get encryption password from environment or prompt
    let password = std::env::var("LC_SYNC_PASSWORD").unwrap_or_else(|_| {
        rpassword::prompt_password("Enter sync decryption password: ")
            .expect("Failed to read password")
    });

    let key = derive_key_from_password(&password)?;

    let mut decrypted_files = Vec::new();
    for file in encrypted_files {
        let decrypted_content = decrypt_data(&file.content, &key)?;
        decrypted_files.push(ConfigFile {
            name: file.name.clone(),
            content: decrypted_content,
        });
    }

    Ok(decrypted_files)
}

/// List available sync providers
pub async fn handle_sync_providers() -> Result<()> {
    println!("{}", "Available sync providers:".bold());
    println!("  • {} - Amazon S3 and S3-compatible storage", "s3".cyan());
    println!("  • {} - Amazon S3", "amazon-s3".cyan());
    println!("  • {} - AWS S3", "aws-s3".cyan());
    println!("  • {} - Cloudflare R2", "cloudflare".cyan());
    println!("  • {} - Backblaze B2", "backblaze".cyan());
    println!(
        "\n{}",
        "Configure a provider with: lc sync configure <provider>".italic()
    );
    Ok(())
}

/// Sync configuration files to cloud storage

/// Validate sync provider name
fn validate_sync_provider(provider: &str) -> Result<()> {
    match provider.to_lowercase().as_str() {
        "s3" | "amazon-s3" | "aws-s3" | "cloudflare" | "backblaze" => Ok(()),
        _ => {
            anyhow::bail!("Unsupported sync provider: {}", provider);
        }
    }
}

/// Sync configuration files to cloud storage
pub async fn handle_sync_to(provider: &str, encrypted: bool, yes: bool) -> Result<()> {
    use std::fs;
    use std::io::{self, Write};

    println!(
        "📤 {} configuration to {}...",
        "Syncing".cyan(),
        provider.bold()
    );

    // Validate provider early
    validate_sync_provider(provider)?;

    // Get lc config directory
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
        .join("lc");

    if !config_dir.exists() {
        anyhow::bail!("Configuration directory does not exist: {:?}", config_dir);
    }

    // Collect all configuration files
    let mut config_files = Vec::new();

    // First, collect all .toml and .db files from the main config directory
    for entry in fs::read_dir(&config_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");
            let extension = path.extension().and_then(|e| e.to_str());

            // Include all .toml files and .db files (logs.db, embeddings.db, etc.)
            let should_include = extension.map(|e| e == "toml" || e == "db").unwrap_or(false);

            if should_include {
                let content = fs::read(&path)?;
                config_files.push(ConfigFile {
                    name: file_name.to_string(),
                    content,
                });
            }
        }
    }

    // Collect provider configs from providers/ subdirectory
    let providers_dir = config_dir.join("providers");
    if providers_dir.exists() {
        for entry in fs::read_dir(&providers_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("toml") {
                let content = fs::read(&path)?;
                let name = format!("providers/{}", path.file_name().unwrap().to_string_lossy());
                config_files.push(ConfigFile { name, content });
            }
        }
    }

    // Check for embeddings directory and include any database files there
    let embeddings_dir = config_dir.join("embeddings");
    if embeddings_dir.exists() {
        for entry in fs::read_dir(&embeddings_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("db") {
                let content = fs::read(&path)?;
                let name = format!("embeddings/{}", path.file_name().unwrap().to_string_lossy());
                config_files.push(ConfigFile { name, content });
            }
        }
    }

    if config_files.is_empty() {
        println!("{} No configuration files found to sync", "ℹ️".blue());
        return Ok(());
    }

    println!("Found {} configuration files", config_files.len());

    // Show files to be synced and confirm
    if !yes {
        println!("\nFiles to sync:");
        for file in &config_files {
            println!("  • {}", file.name);
        }

        print!("\nContinue with sync? [y/N]: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Sync cancelled.");
            return Ok(());
        }
    }

    // Encrypt files if requested
    let _files_to_upload = if encrypted {
        println!("🔐 Encrypting configuration files...");
        encrypt_files(&config_files)?
    } else {
        config_files
    };

    #[cfg(feature = "s3-sync")]
    {
        use super::s3::upload_to_s3_provider;
        upload_to_s3_provider(&_files_to_upload, provider, encrypted).await?;
        println!("{} Configuration synced successfully!", "✅".green());
        return Ok(());
    }

    #[cfg(not(feature = "s3-sync"))]
    {
        anyhow::bail!("S3 sync feature not enabled. Build with --features s3-sync");
    }
}

/// Sync configuration files from cloud storage
pub async fn handle_sync_from(provider: &str, _encrypted: bool, yes: bool) -> Result<()> {
    use std::fs;
    use std::io::{self, Write};

    println!(
        "📥 {} configuration from {}...",
        "Syncing".cyan(),
        provider.bold()
    );

    // Validate provider early
    validate_sync_provider(provider)?;

    // Get lc config directory
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
        .join("lc");

    // Create config directory if it doesn't exist
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
    }

    // Confirm before syncing
    if !yes {
        println!(
            "\n⚠️  {} This will overwrite local configuration files!",
            "Warning:".yellow()
        );
        print!("Continue with sync? [y/N]: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Sync cancelled.");
            return Ok(());
        }
    }

    #[cfg(feature = "s3-sync")]
    {
        use super::s3::download_from_s3_provider;
        let _downloaded_files: Vec<ConfigFile> =
            download_from_s3_provider(provider, _encrypted).await?;

        println!("Downloaded {} configuration files", _downloaded_files.len());

        // Decrypt files if they were encrypted
        let files_to_save = if _encrypted {
            println!("🔓 Decrypting configuration files...");
            decrypt_files(&_downloaded_files)?
        } else {
            _downloaded_files
        };

        // Save files to config directory
        for file in files_to_save {
            let file_path = config_dir.join(&file.name);

            // Ensure parent directory exists
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::write(&file_path, &file.content)?;
            println!("  ✓ Saved {}", file.name);
        }

        println!("{} Configuration synced successfully!", "✅".green());
        return Ok(());
    }

    #[cfg(not(feature = "s3-sync"))]
    {
        anyhow::bail!("S3 sync feature not enabled. Build with --features s3-sync");
    }
}
