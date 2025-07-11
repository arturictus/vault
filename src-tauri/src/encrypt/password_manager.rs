use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use pbkdf2::{
    pbkdf2_hmac,
    hmac::Hmac,
};
use sha2::Sha256;
use rand::{RngCore};
use base64::{engine::general_purpose, Engine as _};
use std::error::Error;

pub struct PasswordManager {
    master_password: String,
}

impl PasswordManager {
    pub fn new(master_password: &str) -> Self {
        PasswordManager {
            master_password: master_password.to_string(),
        }
    }

    fn encrypt_data(
        &self,
        data: &str,
        salt: &[u8],
        nonce: &[u8; 12],
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        // Derive a 256-bit key using PBKDF2
        let mut key = [0u8; 32];
        pbkdf2_hmac::<Sha256>(
            self.master_password.as_bytes(),
            salt,
            100_000, // Adjust iterations for security
            &mut key,
        );
        let cipher = Aes256Gcm::new_from_slice(&key)?;

        // Encrypt data using AES-256-GCM
        let ciphertext = cipher
            .encrypt(Nonce::from_slice(nonce), data.as_bytes())
            .map_err(|e| format!("AES-GCM encryption error: {}", e))?;
        Ok(ciphertext)
    }

    fn decrypt_data(
        &self,
        ciphertext: &[u8],
        salt: &[u8],
        nonce: &[u8; 12],
    ) -> Result<String, Box<dyn Error>> {
        // Derive a 256-bit key using PBKDF2
        let mut key = [0u8; 32];
        pbkdf2_hmac::<Sha256>(
            self.master_password.as_bytes(),
            salt,
            100_000, // Adjust iterations for security
            &mut key,
        );
        let cipher = Aes256Gcm::new_from_slice(&key)?;

        // Decrypt data using AES-256-GCM
        let plaintext = cipher
            .decrypt(Nonce::from_slice(nonce), ciphertext)
            .map_err(|e| format!("AES-GCM decryption error: {}", e))?;
        Ok(String::from_utf8(plaintext)?)
    }

    pub fn store_password(
        &self,
        service: &str,
        password: &str,
    ) -> Result<StoredData, Box<dyn Error>> {
        // Generate random salt and nonce
        let mut salt = [0u8; 16];
        let mut nonce = [0u8; 12];
        OsRng.fill_bytes(&mut salt);
        OsRng.fill_bytes(&mut nonce);

        // Encrypt service:password
        let data = format!("{}:{}", service, password);
        let ciphertext = self.encrypt_data(&data, &salt, &nonce)?;

        Ok(StoredData {
            service: service.to_string(),
            salt: general_purpose::STANDARD.encode(salt),
            nonce: general_purpose::STANDARD.encode(nonce),
            ciphertext: general_purpose::STANDARD.encode(ciphertext),
        })
    }

    fn retrieve_password(
        &self,
        stored_data: &StoredData,
    ) -> Result<String, Box<dyn Error>> {
        // Decode salt, nonce, and ciphertext
        let salt = general_purpose::STANDARD.decode(&stored_data.salt)?;
        let nonce = general_purpose::STANDARD.decode(&stored_data.nonce)?;
        let ciphertext = general_purpose::STANDARD.decode(&stored_data.ciphertext)?;

        if nonce.len() != 12 {
            return Err("Invalid nonce length".into());
        }
        let nonce_array: [u8; 12] = nonce.try_into().map_err(|_| "Nonce conversion error")?;

        // Decrypt data
        self.decrypt_data(&ciphertext, &salt, &nonce_array)
    }
}

#[derive(Debug, serde::Serialize)]
pub struct StoredData {
    service: String,
    salt: String,
    nonce: String,
    ciphertext: String,
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_manager() {
        let master_password = "testPassword123";

        let pm = PasswordManager::new(master_password);

        // Store a password
        let service = "TestService";
        let password = "TestPassword!2025";
        let stored_data = pm.store_password(service, password).unwrap();

        // Retrieve and decrypt
        let retrieved = pm.retrieve_password(&stored_data).unwrap();

        assert_eq!(retrieved, format!("{}:{}", service, password));
    }
}

