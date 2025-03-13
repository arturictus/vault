use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use rand::rngs::OsRng;
use rsa::{
    pkcs8::{DecodePrivateKey, DecodePublicKey, EncodePrivateKey, EncodePublicKey, LineEnding},
    Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey,
};

use crate::encrypt::{Error, Result};
use crate::{AppState, MasterPassword, W};
use std::fs;
#[cfg(test)]
use std::path::Path;

pub type PublicKey = W<RsaPublicKey>;

#[derive(Debug)]
pub struct RSA {
    pub private_key: RsaPrivateKey,
    pub public_key: RsaPublicKey,
}

impl PublicKey {
    pub fn encrypt(&self, data: &[u8]) -> Result<String> {
        let mut rng = OsRng;
        let encrypted = self.0.encrypt(&mut rng, Pkcs1v15Encrypt, data)?;
        Ok(BASE64.encode(encrypted))
    }
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        let pem = fs::read_to_string(path)?;
        Self::from_pem(&pem)
    }

    pub fn from_pem(pem: &str) -> Result<Self> {
        let public_key = RsaPublicKey::from_public_key_pem(pem)?;
        Ok(W(public_key))
    }
}


impl RSA {
    pub fn new() -> Result<Self> {
        let mut rng = OsRng;
        let private_key = RsaPrivateKey::new(&mut rng, 2048)?;
        let public_key = RsaPublicKey::from(&private_key);

        Ok(Self {
            private_key,
            public_key,
        })
    }

    #[cfg(test)]
    pub fn from_file(path: &Path) -> Result<Self> {
        let pem = fs::read_to_string(path)?;
        Self::from_string(&pem)
    }

    pub fn from_string(pem: &str) -> Result<Self> {
        let private_key = RsaPrivateKey::from_pkcs8_pem(pem)?;
        let public_key = RsaPublicKey::from(&private_key);

        Ok(Self {
            private_key,
            public_key,
        })
    }

    #[cfg(test)]
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        let pem = self.private_key.to_pkcs8_pem(LineEnding::LF)?;
        fs::write(path, pem.as_bytes())?;
        Ok(())
    }

    pub fn private_key_pem(&self) -> Result<String> {
        let pem = self.private_key.to_pkcs8_pem(LineEnding::LF)?;
        Ok(pem.to_string())
    }

    pub fn public_key_pem(&self) -> Result<String> {
        let pem = self.public_key.to_public_key_pem(LineEnding::LF)?;
        Ok(pem)
    }

    #[allow(dead_code)]
    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut rng = OsRng;
        let encrypted = self.public_key.encrypt(&mut rng, Pkcs1v15Encrypt, data)?;
        Ok(encrypted)
    }
 
    #[allow(dead_code)]
    pub fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>> {
        let decrypted = self.private_key.decrypt(Pkcs1v15Encrypt, encrypted_data)?;
        Ok(decrypted)
    }
    
    #[allow(dead_code)]
    pub fn encrypt_string(&self, input: &str) -> Result<String> {
        let encrypted = self.encrypt(input.as_bytes())?;
        Ok(BASE64.encode(encrypted))
    }

    #[allow(dead_code)]
    pub fn decrypt_string(&self, input: &str) -> Result<String> {
        let decoded = BASE64.decode(input)?;
        let decrypted = self.decrypt(&decoded)?;
        let result = String::from_utf8(decrypted)?;
        Ok(result)
    }
}

impl TryFrom<&AppState> for RSA {
    type Error = crate::encrypt::Error;

    fn try_from(state: &AppState) -> Result<Self> {
        let fs = state.file_system();
        let password_encryptor = MasterPassword::get_encryptor(state)?;
        let encrypted_pk = fs::read(fs.master_pk())
            .map_err(|_e| Error::Io("Unable to read file with master privatekey".to_string()))?;
        let encrypted_str = String::from_utf8_lossy(&encrypted_pk);
        let raw_pk = password_encryptor.decrypt(&encrypted_str)?;
        let pk = String::from_utf8_lossy(&raw_pk);
        RSA::from_string(&pk)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    use crate::AppState;

    fn state() -> AppState {
        let password = "password";
        let state = AppState::new_test(&password);
        state
    }

    #[test]
    fn test_try_from_trait() {
        let state = state();
        assert!((RSA::try_from(&state).is_ok()));
    }

    #[test]
    fn test_encryption_decryption() {
        let encryptor = RSA::new().unwrap();
        let original = "test secret";
        let encrypted = encryptor.encrypt_string(original).unwrap();
        let decrypted = encryptor.decrypt_string(&encrypted).unwrap();
        assert_eq!(original, decrypted);
    }

    #[test]
    fn test_key_persistence() {
        let dir = tempdir().unwrap();
        let key_path = dir.path().join("private_key.pem");

        let encryptor = RSA::new().unwrap();
        encryptor.save_to_file(&key_path).unwrap();

        let loaded_encryptor = RSA::from_file(&key_path).unwrap();
        let original = "test secret";
        let encrypted = encryptor.encrypt_string(original).unwrap();
        let decrypted = loaded_encryptor.decrypt_string(&encrypted).unwrap();
        assert_eq!(original, decrypted);
    }

    #[test]
    fn test_public_key_to_pem() {
        let encryptor = RSA::new().unwrap();
        let public_key_pem = encryptor.public_key_pem().unwrap();
        assert!(public_key_pem.starts_with("-----BEGIN PUBLIC KEY-----"));
        assert!(public_key_pem.ends_with("-----END PUBLIC KEY-----\n"));
    }

    #[test]
    fn test_public_key_from_pem() {
        let msg = "hello";
        let public_key = W::<RsaPublicKey>::from_file("tests/fixtures/pubkey.pem").unwrap();
        let encrypted = public_key.encrypt(msg.as_bytes()).unwrap();
        assert_ne!(msg, encrypted);
    }
}
