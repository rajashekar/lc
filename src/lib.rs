//! LC (LLM Client) Library
//!
//! This library provides the core functionality for the LC CLI tool,
//! including configuration management, provider handling, and chat functionality.

pub mod chat;
pub mod cli;
pub mod config;
pub mod database;
pub mod dump_metadata;
pub mod error;
pub mod http_client;
pub mod image_utils;
pub mod mcp;
pub mod mcp_daemon;
pub mod model_metadata;
pub mod models_cache;
pub mod provider;
pub mod proxy;
pub mod readers;
pub mod search;
pub mod sync;
pub mod token_utils;
pub mod unified_cache;
pub mod vector_db;
pub mod webchatproxy;

// Re-export commonly used types for easier access in tests
pub use config::{CachedToken, Config, ProviderConfig};
pub use provider::{ChatRequest, Message, OpenAIClient};
