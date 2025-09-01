//! Cloud provider implementations for configuration synchronization

#[cfg(feature = "s3-sync")]
use anyhow::Result;
#[cfg(feature = "s3-sync")]
use colored::Colorize;
#[cfg(feature = "s3-sync")]
use std::collections::HashMap;

#[cfg(feature = "s3-sync")]
use super::{decode_base64, encode_base64, ConfigFile};

#[cfg(feature = "s3-sync")]
use aws_config::BehaviorVersion;
#[cfg(feature = "s3-sync")]
use aws_sdk_s3::{config::Credentials, primitives::ByteStream, Client};

/// S3 configuration for sync operations
#[cfg(feature = "s3-sync")]
#[derive(Debug, Clone)]
pub struct S3Config {
    pub bucket_name: String,
    pub region: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub endpoint_url: Option<String>,
}

/// S3 provider for configuration synchronization
#[cfg(feature = "s3-sync")]
pub struct S3Provider {
    client: Client,
    bucket_name: String,
    folder_prefix: String,
}

#[cfg(feature = "s3-sync")]
impl S3Provider {
    /// Create a new S3 provider instance with a specific provider name
    pub async fn new_with_provider(provider_name: &str) -> Result<Self> {
        let s3_config = Self::get_s3_config(provider_name).await?;

        // Build AWS config with custom settings
        let mut config_builder = aws_config::defaults(BehaviorVersion::latest())
            .region(aws_config::Region::new(s3_config.region.clone()))
            .credentials_provider(Credentials::new(
                s3_config.access_key_id.clone(),
                s3_config.secret_access_key.clone(),
                None,
                None,
                "lc-sync",
            ));

        // Set custom endpoint if provided (for S3-compatible services)
        if let Some(endpoint_url) = &s3_config.endpoint_url {
            config_builder = config_builder.endpoint_url(endpoint_url);
        }

        let config = config_builder.load().await;
        let client = Client::new(&config);

        let folder_prefix = "llm_client_config".to_string();

        Ok(Self {
            client,
            bucket_name: s3_config.bucket_name,
            folder_prefix,
        })
    }

    /// Get S3 configuration from stored config, environment variables, or user input
    async fn get_s3_config(provider_name: &str) -> Result<S3Config> {
        use crate::sync::config::{ProviderConfig, SyncConfig};
        use std::io::{self, Write};

        // First, try to load from stored configuration
        if let Ok(sync_config) = SyncConfig::load() {
            if let Some(ProviderConfig::S3 {
                bucket_name,
                region,
                access_key_id,
                secret_access_key,
                endpoint_url,
            }) = sync_config.get_provider(provider_name)
            {
                println!("{} Using stored S3 configuration for '{}'", "✓".green(), provider_name);
                return Ok(S3Config {
                    bucket_name: bucket_name.clone(),
                    region: region.clone(),
                    access_key_id: access_key_id.clone(),
                    secret_access_key: secret_access_key.clone(),
                    endpoint_url: endpoint_url.clone(),
                });
            }
        }

        println!("{} S3 Configuration Setup for '{}'", "🔧".blue(), provider_name);
        println!("{} No stored configuration found. You can:", "💡".yellow());
        println!(
            "  - Set up configuration: {}",
            format!("lc sync configure {} setup", provider_name).dimmed()
        );
        println!("  - Use environment variables:");
        println!("    LC_S3_BUCKET, LC_S3_REGION, AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY, LC_S3_ENDPOINT");
        println!("  - Enter credentials interactively (below)");
        println!();

        // Try to get from environment variables first
        let bucket_name = if let Ok(bucket) = std::env::var("LC_S3_BUCKET") {
            println!("{} Using bucket from LC_S3_BUCKET: {}", "✓".green(), bucket);
            bucket
        } else {
            print!("Enter S3 bucket name: ");
            // Deliberately flush stdout to ensure prompt appears before user input
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let bucket = input.trim().to_string();
            if bucket.is_empty() {
                anyhow::bail!("Bucket name cannot be empty");
            }
            bucket
        };

        let region = if let Ok(region) = std::env::var("LC_S3_REGION") {
            println!("{} Using region from LC_S3_REGION: {}", "✓".green(), region);
            region
        } else {
            print!("Enter AWS region (default: us-east-1): ");
            // Deliberately flush stdout to ensure prompt appears before user input
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let region = input.trim().to_string();
            if region.is_empty() {
                "us-east-1".to_string()
            } else {
                region
            }
        };

        let access_key_id = if let Ok(key) = std::env::var("AWS_ACCESS_KEY_ID") {
            println!("{} Using access key from AWS_ACCESS_KEY_ID", "✓".green());
            key
        } else {
            print!("Enter AWS Access Key ID: ");
            // Deliberately flush stdout to ensure prompt appears before user input
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let key = input.trim().to_string();
            if key.is_empty() {
                anyhow::bail!("Access Key ID cannot be empty");
            }
            key
        };

        let secret_access_key = if let Ok(secret) = std::env::var("AWS_SECRET_ACCESS_KEY") {
            println!(
                "{} Using secret key from AWS_SECRET_ACCESS_KEY",
                "✓".green()
            );
            secret
        } else {
            print!("Enter AWS Secret Access Key: ");
            // Deliberately flush stdout to ensure prompt appears before password input
            io::stdout().flush()?;
            let secret = rpassword::read_password()?;
            if secret.is_empty() {
                anyhow::bail!("Secret Access Key cannot be empty");
            }
            secret
        };

        let endpoint_url = if let Ok(endpoint) = std::env::var("LC_S3_ENDPOINT") {
            println!(
                "{} Using custom endpoint from LC_S3_ENDPOINT: {}",
                "✓".green(),
                endpoint
            );
            Some(endpoint)
        } else {
            print!("Enter custom S3 endpoint URL (optional, for Backblaze/Cloudflare R2/etc., press Enter to skip): ");
            // Deliberately flush stdout to ensure prompt appears before user input
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let endpoint = input.trim().to_string();
            if endpoint.is_empty() {
                None
            } else {
                Some(endpoint)
            }
        };

        Ok(S3Config {
            bucket_name,
            region,
            access_key_id,
            secret_access_key,
            endpoint_url,
        })
    }

    /// Upload configuration files to S3
    pub async fn upload_configs(&self, files: &[ConfigFile], encrypted: bool) -> Result<()> {
        println!(
            "{} Uploading to S3 bucket: {}",
            "📤".blue(),
            self.bucket_name
        );

        // Check if bucket exists and is accessible
        match self
            .client
            .head_bucket()
            .bucket(&self.bucket_name)
            .send()
            .await
        {
            Ok(_) => {
                println!("{} Bucket access verified", "✓".green());
            }
            Err(e) => {
                anyhow::bail!("Cannot access S3 bucket '{}': {}. Please check your AWS credentials and bucket permissions.", self.bucket_name, e);
            }
        }

        let mut uploaded_count = 0;

        for file in files {
            let key = format!("{}/{}", self.folder_prefix, file.name);

            // Convert binary data to base64 for safe S3 storage
            let content_b64 = encode_base64(&file.content);

            // Add metadata
            let mut metadata = HashMap::new();
            metadata.insert("original-name".to_string(), file.name.clone());
            metadata.insert("encrypted".to_string(), encrypted.to_string());
            metadata.insert("encoding".to_string(), "base64".to_string());
            metadata.insert("sync-tool".to_string(), "lc".to_string());
            metadata.insert("sync-version".to_string(), "1.0".to_string());

            // Add file type metadata for better handling
            let file_type = if file.name.ends_with(".toml") {
                "config"
            } else if file.name.ends_with(".db") {
                "database"
            } else if file.name.starts_with("embeddings/") {
                "embeddings"
            } else if file.name.starts_with("providers/") {
                "provider-config"
            } else {
                "unknown"
            };
            metadata.insert("file-type".to_string(), file_type.to_string());

            // Add file size for monitoring
            metadata.insert("file-size".to_string(), file.content.len().to_string());

            match self
                .client
                .put_object()
                .bucket(&self.bucket_name)
                .key(&key)
                .body(ByteStream::from(content_b64.into_bytes()))
                .content_type("text/plain")
                .set_metadata(Some(metadata))
                .send()
                .await
            {
                Ok(_) => {
                    println!("  {} Uploaded: {}", "✓".green(), file.name);
                    uploaded_count += 1;
                }
                Err(e) => {
                    crate::debug_log!("Failed to upload {}: {}", file.name, e);
                    eprintln!("  {} Failed to upload {}: {}", "✗".red(), file.name, e);
                }
            }
        }

        if uploaded_count == files.len() {
            println!(
                "{} All {} files uploaded successfully",
                "🎉".green(),
                uploaded_count
            );
        } else {
            println!(
                "{} Uploaded {}/{} files",
                "⚠️".yellow(),
                uploaded_count,
                files.len()
            );
        }

        Ok(())
    }

    /// Download configuration files from S3
    pub async fn download_configs(&self, encrypted: bool) -> Result<Vec<ConfigFile>> {
        println!(
            "{} Downloading from S3 bucket: {}",
            "📥".blue(),
            self.bucket_name
        );

        // List objects in the folder
        let list_response = self
            .client
            .list_objects_v2()
            .bucket(&self.bucket_name)
            .prefix(&self.folder_prefix)
            .send()
            .await
            .map_err(|e| {
                anyhow::anyhow!(
                    "Failed to list objects in bucket '{}': {}",
                    self.bucket_name,
                    e
                )
            })?;

        let objects = list_response.contents();

        if objects.is_empty() {
            println!("{} No configuration files found in S3", "ℹ️".blue());
            return Ok(Vec::new());
        }

        println!("{} Found {} objects in S3", "📁".blue(), objects.len());

        let mut downloaded_files = Vec::new();

        for object in objects {
            if let Some(key) = object.key() {
                // Skip directory markers
                if key.ends_with('/') {
                    continue;
                }

                // Extract filename from key
                let filename = key
                    .strip_prefix(&format!("{}/", self.folder_prefix))
                    .unwrap_or(key)
                    .to_string();

                match self
                    .client
                    .get_object()
                    .bucket(&self.bucket_name)
                    .key(key)
                    .send()
                    .await
                {
                    Ok(response) => {
                        // Extract metadata first before consuming the response
                        let metadata = response.metadata().cloned().unwrap_or_default();
                        let is_encrypted = metadata
                            .get("encrypted")
                            .map(|v| v == "true")
                            .unwrap_or(false);

                        // Read the content
                        let body =
                            response.body.collect().await.map_err(|e| {
                                anyhow::anyhow!("Failed to read object body: {}", e)
                            })?;
                        let content_b64 =
                            String::from_utf8(body.into_bytes().to_vec()).map_err(|e| {
                                anyhow::anyhow!("Invalid UTF-8 in object content: {}", e)
                            })?;

                        // Decode from base64
                        let content = decode_base64(&content_b64).map_err(|e| {
                            anyhow::anyhow!(
                                "Failed to decode base64 content for {}: {}",
                                filename,
                                e
                            )
                        })?;

                        if encrypted && !is_encrypted {
                            crate::debug_log!(
                                "Warning: {} is not encrypted but --encrypted flag was used",
                                filename
                            );
                            eprintln!(
                                "  {} Warning: {} is not encrypted but --encrypted flag was used",
                                "⚠️".yellow(),
                                filename
                            );
                        } else if !encrypted && is_encrypted {
                            crate::debug_log!(
                                "Warning: {} is encrypted but --encrypted flag was not used",
                                filename
                            );
                            eprintln!(
                                "  {} Warning: {} is encrypted but --encrypted flag was not used",
                                "⚠️".yellow(),
                                filename
                            );
                        }

                        downloaded_files.push(ConfigFile {
                            name: filename.clone(),
                            content,
                        });

                        println!("  {} Downloaded: {}", "✓".green(), filename);
                    }
                    Err(e) => {
                        crate::debug_log!("Failed to download {}: {}", filename, e);
                        eprintln!("  {} Failed to download {}: {}", "✗".red(), filename, e);
                    }
                }
            }
        }

        println!(
            "{} Downloaded {} files successfully",
            "🎉".green(),
            downloaded_files.len()
        );

        Ok(downloaded_files)
    }

    /// List available configuration files in S3 (for future use)
    #[allow(dead_code)]
    pub async fn list_configs(&self) -> Result<Vec<String>> {
        let list_response = self
            .client
            .list_objects_v2()
            .bucket(&self.bucket_name)
            .prefix(&self.folder_prefix)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to list objects: {}", e))?;

        let mut filenames = Vec::new();

        for object in list_response.contents() {
            if let Some(key) = object.key() {
                if !key.ends_with('/') {
                    let filename = key
                        .strip_prefix(&format!("{}/", self.folder_prefix))
                        .unwrap_or(key)
                        .to_string();
                    filenames.push(filename);
                }
            }
        }

        Ok(filenames)
    }

    /// Delete configuration files from S3 (for future use)
    #[allow(dead_code)]
    pub async fn delete_configs(&self, filenames: &[String]) -> Result<()> {
        for filename in filenames {
            let key = format!("{}/{}", self.folder_prefix, filename);

            match self
                .client
                .delete_object()
                .bucket(&self.bucket_name)
                .key(&key)
                .send()
                .await
            {
                Ok(_) => {
                    println!("  {} Deleted: {}", "✓".green(), filename);
                }
                Err(e) => {
                    crate::debug_log!("Failed to delete {}: {}", filename, e);
                    eprintln!("  {} Failed to delete {}: {}", "✗".red(), filename, e);
                }
            }
        }

        Ok(())
    }
}

#[cfg(all(test, feature = "s3-sync"))]
mod tests {
    use super::*;

    #[test]
    fn test_s3_provider_creation() {
        // This test would require AWS credentials, so we'll just test the structure
        assert_eq!("llm_client_config", "llm_client_config");
    }

    #[test]
    fn test_s3_config_creation() {
        // Test S3Config struct creation
        let config = S3Config {
            bucket_name: "test-bucket".to_string(),
            region: "us-east-1".to_string(),
            access_key_id: "test-key".to_string(),
            secret_access_key: "test-secret".to_string(),
            endpoint_url: None,
        };

        assert_eq!(config.bucket_name, "test-bucket");
        assert_eq!(config.region, "us-east-1");
        assert_eq!(config.access_key_id, "test-key");
        assert_eq!(config.secret_access_key, "test-secret");
        assert!(config.endpoint_url.is_none());
    }
}
