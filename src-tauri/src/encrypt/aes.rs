use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use rand::{rngs::OsRng, RngCore};
use crate::encrypt::{Error, Result};
use crypto::digest::Digest;
use crypto::sha3::Sha3;
use rand::thread_rng;

#[derive(Debug)]
pub struct AES {
    key: [u8; 32],
    salt: [u8; 16],
}

impl AES {
    pub fn new(password: &str) -> Self {
        let salt = Self::generate_salt();
        let key = Self::derive_key(password, &salt);
        Self { key, salt }
    }

    pub fn from_salt(password: &str, salt: &[u8]) -> Self {
        let mut salt_array = [0u8; 16];
        salt_array.copy_from_slice(salt);
        let key = Self::derive_key(password, &salt_array);
        Self {
            key,
            salt: salt_array,
        }
    }

    pub fn from_encrypted(password: &str, encrypted: &str) -> Result<Self> {
        let data = BASE64.decode(encrypted.as_bytes())?;
        if data.len() < 16 {
            return Err(Error::Base64(
                "Invalid encrypted data length".to_string(),
            ));
        }

        // Split into salt and encrypted data
        let (salt, _) = data.split_at(16);
        let mut salt_array = [0u8; 16];
        salt_array.copy_from_slice(salt);
        Ok(Self::from_salt(password, &salt_array))
    }

    fn generate_salt() -> [u8; 16] {
        let mut salt = [0u8; 16];
        thread_rng().fill_bytes(&mut salt);
        salt
    }

    fn derive_key(password: &str, salt: &[u8]) -> [u8; 32] {
        let mut hasher = Sha3::sha3_256();
        hasher.input(password.as_bytes());
        hasher.input(salt);
        let mut key = [0u8; 32];
        hasher.result(&mut key);
        key
    }

    pub fn encrypt(&self, data: &[u8]) -> Result<String> {
        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .map_err(|e| Error::EncryptPassword(e.to_string()))?;

        // Generate a random 96-bit nonce
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt the data
        let ciphertext = cipher
            .encrypt(nonce, data)
            .map_err(|e| Error::EncryptPassword(e.to_string()))?;

        // Combine nonce and ciphertext and encode as base64
        let mut combined = nonce.to_vec();
        combined.extend_from_slice(&ciphertext);
        let encrypted = combined;

        // Combine salt and encrypted data
        let mut combined = Vec::with_capacity(16 + encrypted.len());
        combined.extend_from_slice(&self.salt);
        combined.extend_from_slice(&encrypted);
        Ok(BASE64.encode(combined))
    }

    pub fn decrypt(&self, encoded: &str) -> Result<Vec<u8>> {
        let data = BASE64.decode(encoded.as_bytes())?;
        if data.len() < 16 {
            return Err(Error::DecryptPassword(
                "Invalid encrypted data length".to_string(),
            ));
        }

        // Split into salt and encrypted data
        let (salt, encrypted) = data.split_at(16);
        let mut salt_array = [0u8; 16];
        salt_array.copy_from_slice(salt);
        let key = self.key;

        let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| Error::EncryptPassword(e.to_string()))?;

        if encrypted.len() < 12 {
            return Err(Error::DecryptPassword(
                "Invalid encrypted data length".to_string(),
            ));
        }

        // Split into nonce and ciphertext
        let (nonce_bytes, ciphertext) = encrypted.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        // Decrypt the data
        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| Error::DecryptPassword(e.to_string()))?;
        Ok(plaintext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_cycle() {
        let encryptor = AES::new("test-password");
        let original_data = b"Hello, World!";

        let encrypted = encryptor.encrypt(original_data).unwrap();
        let decrypted = encryptor.decrypt(&encrypted).unwrap();

        assert_eq!(original_data.to_vec(), decrypted);
    }

    #[test]
    fn test_different_passwords_produce_different_results() {
        let encryptor1 = AES::new("password1");
        let encryptor2 = AES::new("password2");
        let data = b"test data";

        let encrypted1 = encryptor1.encrypt(data).unwrap();
        let encrypted2 = encryptor2.encrypt(data).unwrap();

        assert_ne!(encrypted1, encrypted2);
    }

    #[test]
    fn test_wrong_password_fails_decryption() {
        let encryptor1 = AES::new("correct-password");
        let encryptor2 = AES::new("wrong-password");

        let data = b"sensitive information";
        let encrypted = encryptor1.encrypt(data).unwrap();

        assert!(encryptor2.decrypt(&encrypted).is_err());
    }

    #[test]
    fn test_corrupted_data() {
        let encryptor = AES::new("password");
        let data = b"test data";

        let encrypted = encryptor.encrypt(data).unwrap();
        let corrupted = encrypted[..encrypted.len() - 1].to_string() + "X";

        assert!(encryptor.decrypt(&corrupted).is_err());
    }

    #[test]
    fn test_invalid_base64() {
        let encryptor = AES::new("password");
        assert!(matches!(
            encryptor.decrypt("not-base64!@#$"),
            Err(Error::Base64(_))
        ));
    }

    #[test]
    fn test_same_data_different_encryption() {
        let encryptor = AES::new("password");
        let data = b"test data";

        let encrypted1 = encryptor.encrypt(data).unwrap();
        let encrypted2 = encryptor.encrypt(data).unwrap();

        assert_ne!(encrypted1, encrypted2);
    }
    #[test]
    fn test_verify_password() {
        let good_password = "pasword";
        let wrong_password = "wrong-password";
        let good_encryptor = AES::new(good_password);
        let bad_encryptor = AES::new(wrong_password);
        let good_encrypted = good_encryptor.encrypt(b"password").unwrap();

        assert!(bad_encryptor.decrypt(&good_encrypted).is_err())
    }

    #[test]
    fn test_good_password_is_successfuly_decrypted() {
        let good_password = "pasword";
        let good_encryptor = AES::new(good_password);
        let encrypted = good_encryptor.encrypt(good_password.as_bytes()).unwrap();
        let decrypted = good_encryptor.decrypt(&encrypted).unwrap();
        assert_eq!(good_password.as_bytes(), decrypted);
    }

    #[test]
    fn test_good_password_is_decripted_from_different_decryptor() {
        let good_password = "pasword";
        let good_encryptor = AES::new(good_password);
        let encrypted = good_encryptor.encrypt(good_password.as_bytes()).unwrap();
        let good_encryptor2 = AES::from_encrypted(good_password, &encrypted).unwrap();
        let decrypted = good_encryptor2.decrypt(&encrypted).unwrap();
        assert_eq!(good_password.as_bytes(), decrypted);
    }
}
