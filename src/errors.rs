//! Domain-specific error types for better error handling
//! 
//! This module provides structured error types to replace unsafe unwrap() and expect() calls
//! throughout the codebase, improving reliability and debuggability.

use thiserror::Error;

/// Main error type for CLI operations
#[derive(Debug, Error)]
pub enum CliError {
    #[error("Provider not found: {0}")]
    ProviderNotFound(String),
    
    #[error("Provider '{0}' has no API key configured")]
    MissingApiKey(String),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    #[error("Model not found: {0}")]
    ModelNotFound(String),
    
    #[error("Cache error: {0}")]
    CacheError(String),
    
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("HTTP request failed: {0}")]
    Http(String),
    
    #[error("Session error: {0}")]
    Session(String),
    
    #[error("MCP error: {0}")]
    Mcp(String),
    
    #[error("Authentication error: {0}")]
    Auth(String),
    
    #[error("Template processing error: {0}")]
    Template(String),
}

/// Error type for cache operations
#[derive(Debug, Error)]
pub enum CacheError {
    #[error("Cache not initialized")]
    NotInitialized,
    
    #[error("Failed to load cache: {0}")]
    LoadFailed(String),
    
    #[error("Failed to save cache: {0}")]
    SaveFailed(String),
    
    #[error("Cache data corrupted")]
    Corrupted,
}

/// Error type for provider operations
#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("Provider '{0}' not configured")]
    NotConfigured(String),
    
    #[error("API key missing for provider '{0}'")]
    MissingApiKey(String),
    
    #[error("Invalid endpoint URL: {0}")]
    InvalidEndpoint(String),
    
    #[error("HTTP client creation failed: {0}")]
    ClientCreation(String),
    
    #[error("Request failed: {0}")]
    RequestFailed(String),
}

/// Error type for database operations
#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("Connection pool exhausted")]
    PoolExhausted,
    
    #[error("Connection not available")]
    NoConnection,
    
    #[error("Query failed: {0}")]
    QueryFailed(String),
    
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),
    
    #[error("Database operation failed: {0}")]
    OperationFailed(#[from] rusqlite::Error),
}

/// Result type alias using our main error type
pub type CliResult<T> = Result<T, CliError>;

/// Result type alias for cache operations
pub type CacheResult<T> = Result<T, CacheError>;

/// Result type alias for provider operations
pub type ProviderResult<T> = Result<T, ProviderError>;

/// Result type alias for database operations
pub type DatabaseResult<T> = Result<T, DatabaseError>;

/// Extension trait for Option types to convert to Results with context
pub trait OptionExt<T> {
    /// Convert Option to Result with a context message
    fn ok_or_context<C>(self, context: C) -> CliResult<T>
    where
        C: std::fmt::Display;
}

impl<T> OptionExt<T> for Option<T> {
    fn ok_or_context<C>(self, context: C) -> CliResult<T>
    where
        C: std::fmt::Display,
    {
        self.ok_or_else(|| CliError::InvalidConfig(context.to_string()))
    }
}