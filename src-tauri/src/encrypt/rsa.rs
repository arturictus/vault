use crate::encrypt::{Error, Result};
use crate::{AppState, MasterPassword};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use rand::rngs::OsRng;
use rsa::{
    Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey,
    pkcs1v15::Signature as RsaSignature,
    pkcs8::{DecodePrivateKey, DecodePublicKey, EncodePrivateKey, EncodePublicKey, LineEnding},
    sha2::Sha256,
    signature::Verifier as RsaVerifier,
};
use std::fs;
#[cfg(test)]
use std::path::Path;

// This struct is for operations involving only the public key.
pub struct PublicKey {
    key: RsaPublicKey,
}

impl PublicKey {
    pub fn from_pem(pem: &str) -> Result<Self> {
        let rsa_public_key = RsaPublicKey::from_public_key_pem(pem)
            .map_err(|e| Error::Rsa(format!("Failed to parse public key from PEM: {}", e)))?;
        Ok(Self {
            key: rsa_public_key,
        })
    }

    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        // Changed return type to Result<Vec<u8>>
        let mut rng = OsRng;
        let encrypted_data = self
            .key
            .encrypt(&mut rng, Pkcs1v15Encrypt, data)
            .map_err(|e| Error::Rsa(format!("RSA encryption failed: {}", e)))?;
        Ok(encrypted_data) // Return raw encrypted bytes
    }

    pub fn verify(&self, data: &[u8], signature_bytes: &[u8]) -> Result<()> {
        let signature = RsaSignature::try_from(signature_bytes)
            .map_err(|e| Error::Rsa(format!("Failed to create signature from bytes: {}", e)))?;

        let verifying_key = rsa::pkcs1v15::VerifyingKey::<Sha256>::new(self.key.clone());

        verifying_key
            .verify(data, &signature)
            .map_err(|e| Error::Rsa(format!("Signature verification failed: {}", e)))
    }
}

// Renamed RSA to RsaKeyPair to better reflect its purpose (holding a key pair)
#[derive(Debug)]
pub struct RsaKeyPair {
    pub private_key: RsaPrivateKey,
    pub public_key: RsaPublicKey,
}

impl RsaKeyPair {
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
        // Use the public_key field of RsaKeyPair
        let pem = self
            .public_key
            .to_public_key_pem(LineEnding::LF)
            .map_err(|e| Error::Rsa(format!("Failed to encode public key to PEM: {}", e)))?;
        Ok(pem)
    }

    #[allow(dead_code)]
    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut rng = OsRng;
        // Use the public_key field of RsaKeyPair
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

impl TryFrom<&AppState> for RsaKeyPair {
    type Error = crate::encrypt::Error;

    fn try_from(state: &AppState) -> Result<Self> {
        let fs = state.file_system();
        let password_encryptor = MasterPassword::get_encryptor(state)?;
        let encrypted_pk = fs::read(fs.master_pk())
            .map_err(|_e| Error::Io("Unable to read file with master privatekey".to_string()))?;
        let encrypted_str = String::from_utf8_lossy(&encrypted_pk);
        let raw_pk = password_encryptor.decrypt(&encrypted_str)?;
        let pk = String::from_utf8_lossy(&raw_pk);
        RsaKeyPair::from_string(&pk)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AppState;
    use rsa::{signature::SignatureEncoding, signature::Signer};
    use tempfile::tempdir;

    fn state() -> AppState {
        let password = "password";

        AppState::new_test(password)
    }

    #[test]
    fn test_try_from_trait() {
        let state = state();
        assert!((RsaKeyPair::try_from(&state).is_ok()));
    }

    #[test]
    fn test_encryption_decryption() {
        let encryptor = RsaKeyPair::new().unwrap();
        let original = "test secret";
        let encrypted = encryptor.encrypt_string(original).unwrap();
        let decrypted = encryptor.decrypt_string(&encrypted).unwrap();
        assert_eq!(original, decrypted);
    }

    #[test]
    fn test_key_persistence() {
        let dir = tempdir().unwrap();
        let key_path = dir.path().join("private_key.pem");

        let encryptor = RsaKeyPair::new().unwrap();
        encryptor.save_to_file(&key_path).unwrap();

        let loaded_encryptor = RsaKeyPair::from_file(&key_path).unwrap();
        let original = "test secret";
        let encrypted = encryptor.encrypt_string(original).unwrap();
        let decrypted = loaded_encryptor.decrypt_string(&encrypted).unwrap();
        assert_eq!(original, decrypted);
    }

    #[test]
    fn test_public_key_to_pem() {
        let encryptor = RsaKeyPair::new().unwrap();
        let public_key_pem = encryptor.public_key_pem().unwrap();
        assert!(public_key_pem.starts_with("-----BEGIN PUBLIC KEY-----"));
        assert!(public_key_pem.ends_with("-----END PUBLIC KEY-----\n"));
    }

    #[test]
    fn test_public_key_verify() {
        // TODO: implement sign and verify in the main module
        // 1. Generate a key pair
        let key_pair = RsaKeyPair::new().unwrap();
        let public_key_pem = key_pair.public_key_pem().unwrap();
        let private_key_pem = key_pair.private_key_pem().unwrap();

        // 2. Create PublicKey instance for verification
        let verifier_pk = PublicKey::from_pem(&public_key_pem).unwrap();

        // 3. Sign data using the private key (simulating YubiKey signing for test purposes)
        let data_to_sign = b"hello world";
        let private_key_for_signing = RsaPrivateKey::from_pkcs8_pem(&private_key_pem).unwrap();
        let signing_key = rsa::pkcs1v15::SigningKey::<Sha256>::new(private_key_for_signing);
        let signature: RsaSignature = signing_key.sign(data_to_sign);

        // 4. Verify with PublicKey
        let verification_result = verifier_pk.verify(data_to_sign, signature.to_bytes().as_ref());
        assert!(
            verification_result.is_ok(),
            "Verification failed: {:?}",
            verification_result.err()
        );

        // 5. Test with wrong data (should fail)
        let wrong_data = b"hello mars";
        let verification_should_fail =
            verifier_pk.verify(wrong_data, signature.to_bytes().as_ref());
        assert!(
            verification_should_fail.is_err(),
            "Verification should have failed for wrong data"
        );
    }
}
