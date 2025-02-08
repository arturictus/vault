use std::num::NonZeroU32;
use std::fmt;
use std::error::Error as StdError;
use ring::{digest, pbkdf2};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce
};
use rand::{rngs::OsRng, RngCore};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

static PBKDF2_ALG: pbkdf2::Algorithm = pbkdf2::PBKDF2_HMAC_SHA256;
const CREDENTIAL_LEN: usize = digest::SHA256_OUTPUT_LEN;
const PBKDF2_ITERATIONS: u32 = 100_000;

#[derive(Debug)]
pub enum EncryptionError {
    Encryption(String),
    Decryption(String),
    Base64(String),
}

impl fmt::Display for EncryptionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EncryptionError::Encryption(msg) => write!(f, "Encryption error: {}", msg),
            EncryptionError::Decryption(msg) => write!(f, "Decryption error: {}", msg),
            EncryptionError::Base64(msg) => write!(f, "Base64 error: {}", msg),
        }
    }
}

impl StdError for EncryptionError {}

#[derive(Debug)]
pub struct PasswordEncryptor {
    key: Vec<u8>,
}

impl PasswordEncryptor {
    /// Creates a new PasswordEncryptor from a password
    pub fn new(password: &str) -> Self {
        let salt = Self::generate_salt();
        let mut key = vec![0u8; CREDENTIAL_LEN];
        
        pbkdf2::derive(
            PBKDF2_ALG,
            NonZeroU32::new(PBKDF2_ITERATIONS).unwrap(),
            &salt,
            password.as_bytes(),
            &mut key,
        );

        Self { key }
    }

    /// Encrypts data using the derived key
    pub fn encrypt(&self, data: &[u8]) -> Result<String, EncryptionError> {
        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .map_err(|e| EncryptionError::Encryption(e.to_string()))?;
        
        // Generate a random 96-bit nonce
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt the data
        let ciphertext = cipher.encrypt(nonce, data)
            .map_err(|e| EncryptionError::Encryption(e.to_string()))?;

        // Combine nonce and ciphertext and encode as base64
        let mut combined = nonce.to_vec();
        combined.extend_from_slice(&ciphertext);
        Ok(BASE64.encode(combined))
    }

    /// Decrypts data using the derived key
    pub fn decrypt(&self, encrypted_data: &str) -> Result<Vec<u8>, EncryptionError> {
        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .map_err(|e| EncryptionError::Decryption(e.to_string()))?;
        
        // Decode the base64 data
        let encrypted_bytes = BASE64.decode(encrypted_data)
            .map_err(|e| EncryptionError::Base64(e.to_string()))?;
        
        if encrypted_bytes.len() < 12 {
            return Err(EncryptionError::Decryption("Invalid encrypted data length".to_string()));
        }
        
        // Split into nonce and ciphertext
        let (nonce_bytes, ciphertext) = encrypted_bytes.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        // Decrypt the data
        let plaintext = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| EncryptionError::Decryption(e.to_string()))?;
        Ok(plaintext)
    }

    /// Generates a random salt for key derivation
    fn generate_salt() -> Vec<u8> {
        let mut salt = vec![0u8; 16];
        OsRng.fill_bytes(&mut salt);
        salt
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_cycle() {
        let encryptor = PasswordEncryptor::new("test-password");
        let original_data = b"Hello, World!";
        
        let encrypted = encryptor.encrypt(original_data).unwrap();
        let decrypted = encryptor.decrypt(&encrypted).unwrap();
        
        assert_eq!(original_data.to_vec(), decrypted);
    }

    #[test]
    fn test_different_passwords_produce_different_results() {
        let encryptor1 = PasswordEncryptor::new("password1");
        let encryptor2 = PasswordEncryptor::new("password2");
        let data = b"test data";
        
        let encrypted1 = encryptor1.encrypt(data).unwrap();
        let encrypted2 = encryptor2.encrypt(data).unwrap();
        
        assert_ne!(encrypted1, encrypted2);
    }

    #[test]
    fn test_wrong_password_fails_decryption() {
        let encryptor1 = PasswordEncryptor::new("correct-password");
        let encryptor2 = PasswordEncryptor::new("wrong-password");
        
        let data = b"sensitive information";
        let encrypted = encryptor1.encrypt(data).unwrap();
        
        assert!(encryptor2.decrypt(&encrypted).is_err());
    }

    #[test]
    fn test_corrupted_data() {
        let encryptor = PasswordEncryptor::new("password");
        let data = b"test data";
        
        let encrypted = encryptor.encrypt(data).unwrap();
        let corrupted = encrypted[..encrypted.len()-1].to_string() + "X";
        
        assert!(encryptor.decrypt(&corrupted).is_err());
    }

    #[test]
    fn test_invalid_base64() {
        let encryptor = PasswordEncryptor::new("password");
        assert!(matches!(
            encryptor.decrypt("not-base64!@#$"),
            Err(EncryptionError::Base64(_))
        ));
    }

    #[test]
    fn test_same_data_different_encryption() {
        let encryptor = PasswordEncryptor::new("password");
        let data = b"test data";
        
        let encrypted1 = encryptor.encrypt(data).unwrap();
        let encrypted2 = encryptor.encrypt(data).unwrap();
        
        assert_ne!(encrypted1, encrypted2);
    }
    #[test]
    fn test_verify_password() {
        let good_password = "pasword";
        let wrong_password = "wrong-password";
        let good_encryptor = PasswordEncryptor::new(&good_password);
        let bad_encryptor  = PasswordEncryptor::new(&wrong_password);
        let good_encrypted = good_encryptor.encrypt(b"password").unwrap();

        assert!(bad_encryptor.decrypt(&good_encrypted).is_err())
    }
}