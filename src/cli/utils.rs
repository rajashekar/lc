//! Utility functions for CLI operations

use anyhow::Result;
use crate::models::dump_metadata::MetadataDumper;

/// Handle metadata dump command
pub async fn handle_dump_metadata(provider: Option<String>, list: bool) -> Result<()> {
    if list {
        // List available cached metadata files
        MetadataDumper::list_cached_metadata().await?
    } else if let Some(provider_name) = provider {
        // Dump metadata for specific provider
        MetadataDumper::dump_provider_by_name(&provider_name).await?
    } else {
        // Dump metadata for all providers
        MetadataDumper::dump_all_metadata().await?
    }

    Ok(())
}
