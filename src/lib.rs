//! LC (LLM Client) Library
//!
//! This library provides the core functionality for the LC CLI tool,
//! including configuration management, provider handling, and chat functionality.
//!
//! # Platform Differences
//!
//! This crate includes platform-specific features that are only available on certain operating systems:
//!
//! ## Unix Socket Support
//!
//! Unix socket functionality is only available on Unix-like systems (Linux, macOS, BSD, WSL2) and requires
//! the `unix-sockets` feature to be enabled.
//!
//! ### MCP Daemon
//!
//! The MCP (Model Context Protocol) daemon functionality uses Unix domain sockets for inter-process
//! communication and is therefore only available on Unix systems:
//!
//! - **Unix systems** (Linux, macOS, WSL2): Full MCP daemon support with persistent connections
//! - **Windows**: MCP daemon is not supported; direct MCP connections work on all platforms
//!
//! ## Usage Statistics
//!
//! Usage statistics functionality works on all platforms (Windows, macOS, Linux, WSL2).
//! It uses SQLite database which has full cross-platform support.
//!
//! ### Feature Flags
//!
//! - `unix-sockets`: Enables Unix socket functionality (default on Unix systems)
//! - `pdf`: Enables PDF processing support (default)
//!
//! To build without Unix socket support:
//! ```bash
//! cargo build --no-default-features --features pdf
//! ```
//!
//! To build with all features:
//! ```bash
//! cargo build --features "unix-sockets,pdf"
//! ```

// CLI modules
pub mod cli;

// Core modules
pub mod core;
// Re-export core modules at the top level for compatibility
pub use core::chat;
pub use core::provider;
pub use core::provider_installer;
pub use core::http_client;
pub use core::completion;

// Data modules
pub mod data;
// Re-export data modules at the top level for compatibility
pub use data::database;
pub use data::config;
pub use data::keys;
pub use data::vector_db;

// Model-related modules
pub mod models;
// Re-export models modules at the top level for compatibility
pub use models::metadata as model_metadata;
pub use models::cache as models_cache;
pub use models::unified_cache;
pub use models::dump_metadata;

// Service modules
pub mod services;
// Re-export service modules at the top level for compatibility
pub use services::proxy;
pub use services::mcp;
// MCP daemon module - Unix implementation with Windows stubs
// On Windows, all daemon functions return appropriate "unsupported" errors
pub use services::mcp_daemon;
pub use services::webchatproxy;

// Utility modules
pub mod utils;
// Re-export utility modules at the top level for compatibility
pub use utils::audio as audio_utils;
pub use utils::image as image_utils;
pub use utils::token as token_utils;
pub use utils::input;
pub use utils::test as test_utils;
pub use utils::template_processor;

// Analytics modules
pub mod analytics;
// Re-export analytics modules at the top level for compatibility
pub use analytics::usage_stats;

// Standalone modules (not yet categorized)
pub mod error;
pub mod readers;
pub mod search;
pub mod sync;

// Global debug flag
use std::sync::atomic::AtomicBool;
pub static DEBUG_MODE: AtomicBool = AtomicBool::new(false);

// Debug logging macro
#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        if $crate::DEBUG_MODE.load(std::sync::atomic::Ordering::Relaxed) {
            use colored::Colorize;
            eprintln!("{} {}", "[DEBUG]".dimmed(), format!($($arg)*));
        }
    };
}

// Re-export commonly used types for easier access in tests
pub use config::{CachedToken, Config, ProviderConfig};
pub use provider::{ChatRequest, Message, OpenAIClient};
