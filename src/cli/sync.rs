//! Sync functionality commands

use crate::cli::SyncCommands;
use anyhow::Result;
use colored::*;

/// Handle sync-related commands
pub async fn handle(command: SyncCommands) -> Result<()> {
    match command {
        SyncCommands::Providers => {
            // List supported cloud providers
            crate::sync::handle_sync_providers().await?
        }
        SyncCommands::Configure { provider, command } => {
            // Handle provider-specific configuration
            crate::sync::handle_sync_configure(&provider, command).await?
        }
        SyncCommands::To { provider, encrypted, yes } => {
            // Sync configuration to cloud provider
            println!("{} Syncing configuration to {}...", "📤".cyan(), provider);
            if encrypted {
                println!("  {} Encryption enabled", "🔒".yellow());
            }
            crate::sync::handle_sync_to(&provider, encrypted, yes).await?
        }
        SyncCommands::From { provider, encrypted, yes } => {
            // Sync configuration from cloud provider
            println!("{} Syncing configuration from {}...", "📥".cyan(), provider);
            if encrypted {
                println!("  {} Decryption enabled", "🔓".yellow());
            }
            crate::sync::handle_sync_from(&provider, encrypted, yes).await?
        }
    }
    Ok(())
}
