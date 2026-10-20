//! AES256-GCM encryption/decryption functionality for configuration files

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};

/// Derive a 256-bit key from a password using a simple approach
/// In production, you might want to use PBKDF2, scrypt, or Argon2
pub fn derive_key_from_password(password: &str, salt: &[u8]) -> Result<[u8; 32]> {
    use pbkdf2::pbkdf2_hmac;
    use sha2::Sha256;

    // Use PBKDF2 with HMAC-SHA256 for secure key derivation
    let mut key = [0u8; 32];
    let iterations = 100_000;

    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, iterations, &mut key);

    Ok(key)
}

/// Encrypt data using AES256-GCM
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

    // Prepend salt and nonce to ciphertext for storage
    let mut result = Vec::with_capacity(salt.len() + nonce.len() + ciphertext.len());
    result.extend_from_slice(&salt);
    result.extend_from_slice(&nonce);
    result.extend_from_slice(&ciphertext);

    Ok(result)
}

/// Decrypt data using AES256-GCM
#[allow(dead_code)]
pub fn decrypt_data(encrypted_data: &[u8], password: &str) -> Result<Vec<u8>> {
    // Try new format first: [SALT(16)][NONCE(12)][CIPHERTEXT]
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

    // Fallback to legacy format: [NONCE(12)][CIPHERTEXT]
    if encrypted_data.len() >= 12 {
        let legacy_salt = b"lc_sync_encryption_salt_v1";
        let key = derive_key_from_password(password, legacy_salt)?;

        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        if let Ok(plaintext) = cipher.decrypt(nonce, ciphertext) {
            return Ok(plaintext);
        }
    }

    anyhow::bail!("Decryption failed: invalid data or wrong password");
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
        let key4 = derive_key_from_password(password, b"different_salt12").unwrap();
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
    fn test_encryption_with_wrong_password() {
        let data = b"Hello, World!";
        let password = "test_password_123";
        let wrong_password = "wrong_password";

        let encrypted = encrypt_data(data, password).unwrap();

        // Decryption with wrong password should fail
        assert!(decrypt_data(&encrypted, wrong_password).is_err());
    }

    #[test]
    fn test_legacy_decryption() {
        // Manually create a legacy encrypted block
        let data = b"Legacy data format test.";
        let password = "test_password_123";

        // Use the old hardcoded salt
        let legacy_salt = b"lc_sync_encryption_salt_v1";
        let key = derive_key_from_password(password, legacy_salt).unwrap();

        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

        let ciphertext = cipher.encrypt(&nonce, data.as_ref()).unwrap();

        // Prepend only nonce (legacy format)
        let mut legacy_encrypted = Vec::new();
        legacy_encrypted.extend_from_slice(&nonce);
        legacy_encrypted.extend_from_slice(&ciphertext);

        // Verify we can decrypt it using the new function
        let decrypted = decrypt_data(&legacy_encrypted, password).unwrap();
        assert_eq!(decrypted.as_slice(), data);
    }

    #[test]
    fn security_salt_is_random_and_ciphertext_is_authenticated() {
        let password = "review-test-password";
        let first = encrypt_data(b"same plaintext", password).unwrap();
        let second = encrypt_data(b"same plaintext", password).unwrap();
        assert_ne!(&first[..16], &second[..16]);
        assert_eq!(decrypt_data(&first, password).unwrap(), b"same plaintext");
        for index in [0, 16, first.len() - 1] {
            let mut tampered = first.clone();
            tampered[index] ^= 1;
            assert!(decrypt_data(&tampered, password).is_err());
        }
    }

    #[test]
    fn security_legacy_empty_and_short_payloads_remain_readable() {
        let password = "review-test-password";
        let key = derive_key_from_password(password, b"lc_sync_encryption_salt_v1").unwrap();
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
        for data in [
            b"".as_slice(),
            b"x".as_slice(),
            b"longer legacy configuration data".as_slice(),
        ] {
            let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
            let mut legacy = nonce.to_vec();
            legacy.extend_from_slice(&cipher.encrypt(&nonce, data).unwrap());
            assert_eq!(decrypt_data(&legacy, password).unwrap(), data);
            assert!(decrypt_data(&legacy, "wrong-password").is_err());
        }
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
        let password = "test_password_123";

        // Too short data should fail
        assert!(decrypt_data(b"short", password).is_err());

        // Invalid data should fail
        let invalid_data = vec![0u8; 30]; // Valid length, invalid content
        assert!(decrypt_data(&invalid_data, password).is_err());
    }
}
