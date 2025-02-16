use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use rand::rngs::OsRng;
use rsa::{
    pkcs8::{DecodePrivateKey, EncodePrivateKey, LineEnding, EncodePublicKey},
    Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey,
};
use std::fs;
use std::path::Path;
use crate::encrypt::Result;

#[derive(Debug)]
pub struct Encryptor {
    pub private_key: RsaPrivateKey,
    pub public_key: RsaPublicKey,
}

impl Encryptor {
    pub fn new() -> Result<Self> {
        let mut rng = OsRng;
        let private_key = RsaPrivateKey::new(&mut rng, 2048)?;
        let public_key = RsaPublicKey::from(&private_key);

        Ok(Self {
            private_key,
            public_key,
        })
    }

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

    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut rng = OsRng;
        let encrypted = self.public_key.encrypt(&mut rng, Pkcs1v15Encrypt, data)?;
        Ok(encrypted)
    }

    pub fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>> {
        let decrypted = self.private_key.decrypt(Pkcs1v15Encrypt, encrypted_data)?;
        Ok(decrypted)
    }

    pub fn encrypt_string(&self, input: &str) -> Result<String> {
        let encrypted = self.encrypt(input.as_bytes())?;
        Ok(BASE64.encode(encrypted))
    }

    pub fn decrypt_string(&self, input: &str) -> Result<String> {
        let decoded = BASE64.decode(input)?;
        let decrypted = self.decrypt(&decoded)?;
        let result = String::from_utf8(decrypted)?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_encryption_decryption() {
        let encryptor = Encryptor::new().unwrap();
        let original = "test secret";
        let encrypted = encryptor.encrypt_string(original).unwrap();
        let decrypted = encryptor.decrypt_string(&encrypted).unwrap();
        assert_eq!(original, decrypted);
    }

    #[test]
    fn test_key_persistence() {
        let dir = tempdir().unwrap();
        let key_path = dir.path().join("private_key.pem");

        let encryptor = Encryptor::new().unwrap();
        encryptor.save_to_file(&key_path).unwrap();

        let loaded_encryptor = Encryptor::from_file(&key_path).unwrap();
        let original = "test secret";
        let encrypted = encryptor.encrypt_string(original).unwrap();
        let decrypted = loaded_encryptor.decrypt_string(&encrypted).unwrap();
        assert_eq!(original, decrypted);
    }

    #[test]
    fn test_public_key_to_pem() {
        let encryptor = Encryptor::new().unwrap();
        let public_key_pem = encryptor.public_key_pem().unwrap();
        assert!(public_key_pem.starts_with("-----BEGIN PUBLIC KEY-----"));
        assert!(public_key_pem.ends_with("-----END PUBLIC KEY-----\n"));
    }
}
