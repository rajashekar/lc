//! CLI module - organized by domain

use anyhow::Result;
use std::sync::atomic::AtomicBool;

// CLI definitions (structs and enums)
pub mod definitions;

// Submodules - to be implemented separately
pub mod aliases;
pub mod audio;
pub mod chat;
pub mod config;
pub mod completion;
pub mod image;
pub mod keys;
pub mod logging;
pub mod mcp;
pub mod models;
pub mod prompts;
pub mod providers;
pub mod proxy;
pub mod search;
pub mod sync;
pub mod templates;
pub mod usage;
pub mod utils;
pub mod vectors;
pub mod webchatproxy;

// Re-export all CLI types for easy access
pub use definitions::*;

// Global debug flag
pub static DEBUG_MODE: AtomicBool = AtomicBool::new(false);

// Set debug mode
pub fn set_debug_mode(enabled: bool) {
    DEBUG_MODE.store(enabled, std::sync::atomic::Ordering::Relaxed);
}

// Helper function for parsing environment variables
#[allow(dead_code)]
pub fn parse_env_var(s: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = s.splitn(2, '=').collect();
    if parts.len() != 2 {
        return Err(format!(
            "Invalid environment variable format: '{}'. Expected 'KEY=VALUE'",
            s
        ));
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}