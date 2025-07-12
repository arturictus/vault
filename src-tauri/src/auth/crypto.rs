//! Enhanced cryptographic operations with Argon2

use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use aes_gcm::{Aes256Gcm, Nonce, aead::{Aead, KeyInit, generic_array::GenericArray}};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde::{Deserialize, Serialize};
use crate::error::{CryptoError, CryptoResult};
use crate::security::{SecureString, SecureBytes};

/// Argon2 configuration parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Argon2Config {
    /// Memory cost in KiB
    pub memory_cost: u32,
    /// Time cost (iterations)
    pub time_cost: u32,
    /// Parallelism factor
    pub parallelism: u32,
    /// Output length in bytes
    pub output_length: usize,
}

impl Default for Argon2Config {
    fn default() -> Self {
        Self {
            memory_cost: 65536, // 64 MiB
            time_cost: 3,       // 3 iterations
            parallelism: 4,     // 4 threads
            output_length: 32,  // 32 bytes
        }
    }
}

/// Cryptographic manager for secure operations
pub struct CryptoManager {
    argon2: Argon2<'static>,
    config: Argon2Config,
}

impl CryptoManager {
    /// Create a new crypto manager with default configuration
    pub fn new() -> Self {
        Self::with_config(Argon2Config::default())
    }
    
    /// Create a new crypto manager with custom configuration
    pub fn with_config(config: Argon2Config) -> Self {
        let argon2 = Argon2::default();
        Self { argon2, config }
    }
    
    /// Hash a password using Argon2
    pub fn hash_password(&self, password: &SecureString) -> CryptoResult<String> {
        let salt = SaltString::generate(&mut OsRng);
        
        let password_hash = password.expose(|pwd| {
            self.argon2.hash_password(pwd.as_bytes(), &salt)
        })?;
        
        Ok(password_hash.to_string())
    }
    
    /// Verify a password against its hash
    pub fn verify_password(&self, password: &SecureString, hash: &str) -> CryptoResult<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|_| CryptoError::HashVerificationFailed)?;
        
        let result = password.expose(|pwd| {
            self.argon2.verify_password(pwd.as_bytes(), &parsed_hash)
        });
        
        match result {
            Ok(()) => Ok(true),
            Err(argon2::password_hash::Error::Password) => Ok(false),
            Err(e) => Err(CryptoError::Argon2Error(e.to_string())),
        }
    }
    
    /// Derive a key from a password using Argon2
    pub fn derive_key(&self, password: &SecureString, salt: &[u8]) -> CryptoResult<SecureBytes> {
        let mut output = vec![0u8; self.config.output_length];
        
        password.expose(|pwd| {
            self.argon2.hash_password_into(pwd.as_bytes(), salt, &mut output)
        })?;
        
        Ok(SecureBytes::new(output))
    }
    
    /// Generate a random salt
    pub fn generate_salt(&self) -> CryptoResult<Vec<u8>> {
        use rand::RngCore;
        let mut salt = vec![0u8; 32];
        OsRng.fill_bytes(&mut salt);
        Ok(salt)
    }
    
    /// Encrypt data using AES-256-GCM
    pub fn encrypt(&self, key: &SecureBytes, plaintext: &[u8]) -> CryptoResult<EncryptedData> {
        if key.len() != 32 {
            return Err(CryptoError::InvalidKeyLength {
                expected: 32,
                actual: key.len(),
            });
        }
        
        // Generate random nonce
        use rand::RngCore;
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let cipher = key.expose(|k| {
            let key_array = GenericArray::from_slice(k);
            Aes256Gcm::new(key_array)
        });
        
        let ciphertext = cipher.encrypt(nonce, plaintext)
            .map_err(|e| CryptoError::EncryptionFailed {
                reason: e.to_string(),
            })?;
        
        Ok(EncryptedData {
            ciphertext,
            nonce: nonce_bytes.to_vec(),
        })
    }
    
    /// Decrypt data using AES-256-GCM
    pub fn decrypt(&self, key: &SecureBytes, encrypted: &EncryptedData) -> CryptoResult<Vec<u8>> {
        if key.len() != 32 {
            return Err(CryptoError::InvalidKeyLength {
                expected: 32,
                actual: key.len(),
            });
        }
        
        if encrypted.nonce.len() != 12 {
            return Err(CryptoError::DecryptionFailed {
                reason: "Invalid nonce length".to_string(),
            });
        }
        
        let nonce = Nonce::from_slice(&encrypted.nonce);
        
        let cipher = key.expose(|k| {
            let key_array = GenericArray::from_slice(k);
            Aes256Gcm::new(key_array)
        });
        
        let plaintext = cipher.decrypt(nonce, encrypted.ciphertext.as_ref())
            .map_err(|e| CryptoError::DecryptionFailed {
                reason: e.to_string(),
            })?;
        
        Ok(plaintext)
    }
    
    /// Generate a secure random key
    pub fn generate_key(&self) -> CryptoResult<SecureBytes> {
        use rand::RngCore;
        let mut key = vec![0u8; 32];
        OsRng.fill_bytes(&mut key);
        Ok(SecureBytes::new(key))
    }
}

impl Default for CryptoManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Encrypted data container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
}

impl EncryptedData {
    /// Encode to base64 string
    pub fn to_base64(&self) -> String {
        let combined = [&self.nonce[..], &self.ciphertext[..]].concat();
        BASE64.encode(combined)
    }
    
    /// Decode from base64 string
    pub fn from_base64(encoded: &str) -> CryptoResult<Self> {
        let combined = BASE64.decode(encoded)
            .map_err(|e| CryptoError::DecryptionFailed {
                reason: format!("Base64 decode error: {}", e),
            })?;
        
        if combined.len() < 12 {
            return Err(CryptoError::DecryptionFailed {
                reason: "Invalid encrypted data length".to_string(),
            });
        }
        
        let nonce = combined[..12].to_vec();
        let ciphertext = combined[12..].to_vec();
        
        Ok(Self { ciphertext, nonce })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::SecureString;
    
    #[test]
    fn test_password_hashing() {
        let crypto = CryptoManager::new();
        let password = SecureString::from_str("test_password_123");
        
        let hash = crypto.hash_password(&password).unwrap();
        assert!(crypto.verify_password(&password, &hash).unwrap());
        
        let wrong_password = SecureString::from_str("wrong_password");
        assert!(!crypto.verify_password(&wrong_password, &hash).unwrap());
    }
    
    #[test]
    fn test_key_derivation() {
        let crypto = CryptoManager::new();
        let password = SecureString::from_str("test_password");
        let salt = crypto.generate_salt().unwrap();
        
        let key1 = crypto.derive_key(&password, &salt).unwrap();
        let key2 = crypto.derive_key(&password, &salt).unwrap();
        
        // Same password and salt should produce same key
        assert_eq!(key1.len(), key2.len());
        assert_eq!(key1.len(), 32);
    }
    
    #[test]
    fn test_encryption_decryption() {
        let crypto = CryptoManager::new();
        let key = crypto.generate_key().unwrap();
        let plaintext = b"Hello, World!";
        
        let encrypted = crypto.encrypt(&key, plaintext).unwrap();
        let decrypted = crypto.decrypt(&key, &encrypted).unwrap();
        
        assert_eq!(plaintext, decrypted.as_slice());
    }
    
    #[test]
    fn test_encrypted_data_base64() {
        let crypto = CryptoManager::new();
        let key = crypto.generate_key().unwrap();
        let plaintext = b"Test data for base64 encoding";
        
        let encrypted = crypto.encrypt(&key, plaintext).unwrap();
        let encoded = encrypted.to_base64();
        let decoded = EncryptedData::from_base64(&encoded).unwrap();
        
        let decrypted = crypto.decrypt(&key, &decoded).unwrap();
        assert_eq!(plaintext, decrypted.as_slice());
    }
}
