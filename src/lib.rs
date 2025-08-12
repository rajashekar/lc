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

pub mod chat;
pub mod cli;
pub mod completion;
pub mod config;
pub mod database;
pub mod dump_metadata;
pub mod error;
pub mod http_client;
pub mod image_utils;
pub mod input;
pub mod mcp;
// MCP daemon module - Unix implementation with Windows stubs
// On Windows, all daemon functions return appropriate "unsupported" errors
pub mod mcp_daemon;
pub mod model_metadata;
pub mod models_cache;
pub mod provider;
pub mod proxy;
pub mod readers;
pub mod search;
pub mod sync;
pub mod template_processor;
pub mod test_utils;
pub mod token_utils;
pub mod unified_cache;
pub mod usage_stats;
pub mod vector_db;
pub mod webchatproxy;

// Re-export commonly used types for easier access in tests
pub use config::{CachedToken, Config, ProviderConfig};
pub use provider::{ChatRequest, Message, OpenAIClient};
