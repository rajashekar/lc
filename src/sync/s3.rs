//! S3 synchronization module (requires s3-sync feature)

#[cfg(feature = "s3-sync")]
use super::ConfigFile;
#[cfg(feature = "s3-sync")]
use anyhow::Result;

/// Upload configuration files to S3 using specified provider
#[cfg(feature = "s3-sync")]
pub async fn upload_to_s3_provider(files: &[ConfigFile], provider: &str, encrypted: bool) -> Result<()> {
    use super::providers::S3Provider;
    
    // Create S3 provider with the specified provider name
    let s3_provider = S3Provider::new_with_provider(provider).await?;
    
    // Upload configs with correct encryption status
    s3_provider.upload_configs(files, encrypted).await
}

/// Download configuration files from S3 using specified provider
#[cfg(feature = "s3-sync")]
pub async fn download_from_s3_provider(provider: &str, encrypted: bool) -> Result<Vec<ConfigFile>> {
    use super::providers::S3Provider;
    
    // Create S3 provider with the specified provider name
    let s3_provider = S3Provider::new_with_provider(provider).await?;
    
    // Download configs with correct encryption status
    s3_provider.download_configs(encrypted).await
}

// Keep the old functions for backward compatibility (deprecated)
#[cfg(feature = "s3-sync")]
#[deprecated(note = "Use upload_to_s3_provider instead")]
pub async fn upload_to_s3(files: &[ConfigFile]) -> Result<()> {
    upload_to_s3_provider(files, "s3", false).await
}

#[cfg(feature = "s3-sync")]
#[deprecated(note = "Use download_from_s3_provider instead")]
pub async fn download_from_s3() -> Result<Vec<ConfigFile>> {
    download_from_s3_provider("s3", false).await
}
