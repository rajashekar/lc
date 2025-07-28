//! LC (LLM Client) Library
//! 
//! This library provides the core functionality for the LC CLI tool,
//! including configuration management, provider handling, and chat functionality.

pub mod cli;
pub mod config;
pub mod database;
pub mod provider;
pub mod chat;
pub mod error;
pub mod models_cache;
pub mod model_metadata;
pub mod proxy;
pub mod token_utils;
pub mod unified_cache;
pub mod mcp;
pub mod vector_db;

// Re-export commonly used types for easier access in tests
pub use config::{Config, ProviderConfig, CachedToken};
pub use provider::{OpenAIClient, ChatRequest, Message};