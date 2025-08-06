//! AES256-GCM encryption/decryption functionality for configuration files

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};

/// Derive a 256-bit key from a password using a simple approach
/// In production, you might want to use PBKDF2, scrypt, or Argon2
pub fn derive_key_from_password(password: &str) -> Result<[u8; 32]> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    // Simple key derivation - in production, use proper KDF like PBKDF2
    let mut hasher = DefaultHasher::new();
    password.hash(&mut hasher);
    let hash1 = hasher.finish();

    // Create a second hash for more entropy
    let mut hasher2 = DefaultHasher::new();
    format!("{}{}", password, hash1).hash(&mut hasher2);
    let hash2 = hasher2.finish();

    // Combine hashes to create 32-byte key
    let mut key = [0u8; 32];
    key[0..8].copy_from_slice(&hash1.to_le_bytes());
    key[8..16].copy_from_slice(&hash2.to_le_bytes());
    key[16..24].copy_from_slice(&hash1.to_be_bytes());
    key[24..32].copy_from_slice(&hash2.to_be_bytes());

    Ok(key)
}

/// Encrypt data using AES256-GCM
pub fn encrypt_data(data: &[u8], key: &[u8; 32]) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    let ciphertext = cipher
        .encrypt(&nonce, data)
        .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

    // Prepend nonce to ciphertext for storage
    let mut result = Vec::with_capacity(nonce.len() + ciphertext.len());
    result.extend_from_slice(&nonce);
    result.extend_from_slice(&ciphertext);

    Ok(result)
}

/// Decrypt data using AES256-GCM
pub fn decrypt_data(encrypted_data: &[u8], key: &[u8; 32]) -> Result<Vec<u8>> {
    if encrypted_data.len() < 12 {
        anyhow::bail!("Invalid encrypted data: too short");
    }

    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));

    // Extract nonce and ciphertext
    let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;

    Ok(plaintext)
}

/// Encode binary data to base64 for safe transport/storage
pub fn encode_base64(data: &[u8]) -> String {
    general_purpose::STANDARD.encode(data)
}

/// Decode base64 data back to binary
pub fn decode_base64(data: &str) -> Result<Vec<u8>> {
    general_purpose::STANDARD
        .decode(data)
        .map_err(|e| anyhow::anyhow!("Base64 decode failed: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_derivation() {
        let password = "test_password_123";
        let key1 = derive_key_from_password(password).unwrap();
        let key2 = derive_key_from_password(password).unwrap();

        // Same password should produce same key
        assert_eq!(key1, key2);

        // Different password should produce different key
        let key3 = derive_key_from_password("different_password").unwrap();
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_encryption_decryption() {
        let data = b"Hello, World! This is test data for encryption.";
        let password = "test_password_123";
        let key = derive_key_from_password(password).unwrap();

        // Encrypt
        let encrypted = encrypt_data(data, &key).unwrap();
        assert_ne!(encrypted.as_slice(), data);
        assert!(encrypted.len() > data.len()); // Should be larger due to nonce + auth tag

        // Decrypt
        let decrypted = decrypt_data(&encrypted, &key).unwrap();
        assert_eq!(decrypted.as_slice(), data);
    }

    #[test]
    fn test_encryption_with_wrong_key() {
        let data = b"Hello, World!";
        let key1 = derive_key_from_password("password1").unwrap();
        let key2 = derive_key_from_password("password2").unwrap();

        let encrypted = encrypt_data(data, &key1).unwrap();

        // Decryption with wrong key should fail
        assert!(decrypt_data(&encrypted, &key2).is_err());
    }

    #[test]
    fn test_base64_encoding() {
        let data = b"Hello, World!";
        let encoded = encode_base64(data);
        let decoded = decode_base64(&encoded).unwrap();

        assert_eq!(decoded.as_slice(), data);
    }

    #[test]
    fn test_invalid_encrypted_data() {
        let key = derive_key_from_password("test").unwrap();

        // Too short data should fail
        assert!(decrypt_data(b"short", &key).is_err());

        // Invalid data should fail
        let invalid_data = vec![0u8; 20];
        assert!(decrypt_data(&invalid_data, &key).is_err());
    }
}
