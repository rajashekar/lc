//! Synchronization functionality for lc configurations

pub mod config;
pub mod encryption;
pub mod providers;

#[cfg(feature = "s3-sync")]
pub mod s3;

pub mod sync;

// Re-export main sync functions from sync module
pub use sync::{
    decrypt_files, encrypt_files, handle_sync_from, handle_sync_providers, handle_sync_to,
    ConfigFile,
};

// Re-export config handler from config module
pub use config::handle_sync_configure;

// Re-export encryption utilities from encryption module
pub use encryption::{
    decode_base64, decrypt_data, derive_key_from_password, encode_base64, encrypt_data,
};
