use thiserror::Error;

#[derive(Error, Debug)]
pub enum RllmError {
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("Network error: {0}")]
    Network(String),
}

impl From<rusqlite::Error> for RllmError {
    fn from(err: rusqlite::Error) -> Self {
        RllmError::Database(err.to_string())
    }
}

impl From<reqwest::Error> for RllmError {
    fn from(err: reqwest::Error) -> Self {
        RllmError::Network(err.to_string())
    }
}

impl From<std::io::Error> for RllmError {
    fn from(err: std::io::Error) -> Self {
        RllmError::Config(err.to_string())
    }
}