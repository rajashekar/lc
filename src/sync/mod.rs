//! Synchronization functionality for lc configurations

pub mod config;
pub mod encryption;

#[cfg(feature = "s3-sync")]
pub mod s3;

pub mod sync;

// Re-export main sync functions from sync module
pub use sync::{
    handle_sync_providers,
    handle_sync_to,
    handle_sync_from,
    ConfigFile,
    encrypt_files,
    decrypt_files,
};

// Re-export config handler from config module
pub use config::handle_sync_configure;

// Re-export encryption utilities from encryption module
pub use encryption::{
    derive_key_from_password,
    encrypt_data,
    decrypt_data,
    encode_base64,
    decode_base64,
};