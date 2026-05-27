//! AES256-GCM encryption/decryption functionality for configuration files

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};

/// Derive a 256-bit key from a password using PBKDF2
pub fn derive_key_from_password(password: &str, salt: &[u8]) -> Result<[u8; 32]> {
    use pbkdf2::pbkdf2_hmac;
    use sha2::Sha256;

    let mut key = [0u8; 32];
    let iterations = 100_000;

    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, iterations, &mut key);

    Ok(key)
}

/// Encrypt data using AES256-GCM with a random salt
pub fn encrypt_data(data: &[u8], password: &str) -> Result<Vec<u8>> {
    use rand::RngCore;

    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut salt);

    let key = derive_key_from_password(password, &salt)?;
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    let ciphertext = cipher
        .encrypt(&nonce, data)
        .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

    // Prepend salt (16 bytes) and nonce (12 bytes) to ciphertext for storage
    let mut result = Vec::with_capacity(salt.len() + nonce.len() + ciphertext.len());
    result.extend_from_slice(&salt);
    result.extend_from_slice(&nonce);
    result.extend_from_slice(&ciphertext);

    Ok(result)
}

/// Decrypt data using AES256-GCM
#[allow(dead_code)]
pub fn decrypt_data(encrypted_data: &[u8], password: &str) -> Result<Vec<u8>> {
    // Try new format: [SALT(16)][NONCE(12)][CIPHERTEXT]
    if encrypted_data.len() >= 28 {
        let (salt, rest) = encrypted_data.split_at(16);
        let (nonce_bytes, ciphertext) = rest.split_at(12);

        if let Ok(key) = derive_key_from_password(password, salt) {
            let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
            let nonce = Nonce::from_slice(nonce_bytes);

            if let Ok(plaintext) = cipher.decrypt(nonce, ciphertext) {
                return Ok(plaintext);
            }
        }
    }

    // Fallback to old format: [NONCE(12)][CIPHERTEXT] with constant salt
    if encrypted_data.len() >= 12 {
        let old_salt = b"lc_sync_encryption_salt_v1";
        if let Ok(key) = derive_key_from_password(password, old_salt) {
            let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));

            let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
            let nonce = Nonce::from_slice(nonce_bytes);

            if let Ok(plaintext) = cipher.decrypt(nonce, ciphertext) {
                return Ok(plaintext);
            }
        }
    }

    anyhow::bail!("Invalid encrypted data: too short or incorrect password");
}

/// Encode binary data to base64 for safe transport/storage
#[allow(dead_code)]
pub fn encode_base64(data: &[u8]) -> String {
    general_purpose::STANDARD.encode(data)
}

/// Decode base64 data back to binary
#[allow(dead_code)]
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
        let salt = b"test_salt_123456";
        let key1 = derive_key_from_password(password, salt).unwrap();
        let key2 = derive_key_from_password(password, salt).unwrap();

        // Same password and salt should produce same key
        assert_eq!(key1, key2);

        // Different password should produce different key
        let key3 = derive_key_from_password("different_password", salt).unwrap();
        assert_ne!(key1, key3);

        // Different salt should produce different key
        let salt2 = b"different_salt_1";
        let key4 = derive_key_from_password(password, salt2).unwrap();
        assert_ne!(key1, key4);
    }

    #[test]
    fn test_encryption_decryption() {
        let data = b"Hello, World! This is test data for encryption.";
        let password = "test_password_123";

        // Encrypt
        let encrypted = encrypt_data(data, password).unwrap();
        assert_ne!(encrypted.as_slice(), data);
        assert!(encrypted.len() > data.len()); // Should be larger due to salt + nonce + auth tag

        // Decrypt
        let decrypted = decrypt_data(&encrypted, password).unwrap();
        assert_eq!(decrypted.as_slice(), data);
    }

    #[test]
    fn test_encryption_with_wrong_key() {
        let data = b"Hello, World!";
        let password = "password1";

        let encrypted = encrypt_data(data, password).unwrap();

        // Decryption with wrong key should fail
        assert!(decrypt_data(&encrypted, "password_wrong").is_err());
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
        let password = "test";

        // Too short data should fail
        assert!(decrypt_data(b"short", password).is_err());

        // Invalid data should fail
        let invalid_data = vec![0u8; 30]; // 16 salt + 12 nonce + 2 garbage
        assert!(decrypt_data(&invalid_data, password).is_err());
    }
}
