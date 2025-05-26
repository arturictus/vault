use p256::ecdsa::{SigningKey, Signature, VerifyingKey};
use p256::elliptic_curve::sec1::ToEncodedPoint;
use p256::ecdh::EphemeralSecret as P256EphemeralSecret;
use p256::PublicKey as P256PublicKey;
use p256::SecretKey as P256SecretKey;
use p384::ecdsa::{SigningKey as P384SigningKey, Signature as P384Signature, VerifyingKey as P384VerifyingKey};
use p384::ecdh::EphemeralSecret as P384EphemeralSecret;
use p384::PublicKey as P384PublicKey;
use p384::SecretKey as P384SecretKey;
use yubikey::piv as piv_card;
use rand::rngs::OsRng;
use sha2::{Sha256, Sha384, Digest};
use hkdf::Hkdf;
use aes_gcm::{Aes256Gcm, Key, Nonce, aead::{Aead, KeyInit}};
use signature::{Signer, Verifier};

pub enum EccAlgorithm {
    P256,
    P384,
}

pub fn generate_key_pair(algorithm: EccAlgorithm) -> (Vec<u8>, Vec<u8>) {
    match algorithm {
        EccAlgorithm::P256 => {
            let signing_key = SigningKey::random(&mut OsRng);
            let verifying_key = VerifyingKey::from(&signing_key);
            (signing_key.to_bytes().to_vec(), verifying_key.to_encoded_point(false).as_bytes().to_vec())
        }
        EccAlgorithm::P384 => {
            let signing_key = P384SigningKey::random(&mut OsRng);
            let verifying_key = P384VerifyingKey::from(&signing_key);
            (signing_key.to_bytes().to_vec(), verifying_key.to_encoded_point(false).as_bytes().to_vec())
        }
    }
}

pub fn sign(data: &[u8], private_key_bytes: &[u8], algorithm: EccAlgorithm) -> Result<Vec<u8>, &'static str> {
    match algorithm {
        EccAlgorithm::P256 => {
            let secret_key = P256SecretKey::from_slice(private_key_bytes)
                .map_err(|_| "Invalid P256 private key slice for signing")?;
            let signing_key = SigningKey::from(&secret_key);
            // Use Digest::digest() and Signer trait
            let digest = Sha256::digest(data);
            let signature: Signature = signing_key.sign(&digest);
            Ok(signature.to_bytes().to_vec())
        }
        EccAlgorithm::P384 => {
            let secret_key = P384SecretKey::from_slice(private_key_bytes)
                .map_err(|_| "Invalid P384 private key slice for signing")?;
            let signing_key = P384SigningKey::from(&secret_key);
            // Use Digest::digest() and Signer trait
            let digest = Sha384::digest(data);
            let signature: P384Signature = signing_key.sign(&digest);
            Ok(signature.to_bytes().to_vec())
        }
    }
}

pub fn verify(data: &[u8], signature_bytes: &[u8], public_key_bytes: &[u8], algorithm: EccAlgorithm) -> Result<bool, &'static str> {
    match algorithm {
        EccAlgorithm::P256 => {
            let verifying_key = VerifyingKey::from_sec1_bytes(public_key_bytes)
                .map_err(|_| "Invalid P256 public key for verification")?;
            let signature = Signature::try_from(signature_bytes)
                .map_err(|_| "Invalid P256 signature format")?;
            // Use Digest::digest() and Verifier trait
            let digest = Sha256::digest(data);
            Ok(verifying_key.verify(&digest, &signature).is_ok())
        }
        EccAlgorithm::P384 => {
            let verifying_key = P384VerifyingKey::from_sec1_bytes(public_key_bytes)
                .map_err(|_| "Invalid P384 public key for verification")?;
            let signature = P384Signature::try_from(signature_bytes)
                .map_err(|_| "Invalid P384 signature format")?;
            // Use Digest::digest() and Verifier trait
            let digest = Sha384::digest(data);
            Ok(verifying_key.verify(&digest, &signature).is_ok())
        }
    }
}

// Encryption and decryption using ECIES with AES-GCM and HKDF

pub fn encrypt(data: &[u8], public_key_bytes: &[u8], algorithm: EccAlgorithm) -> Result<Vec<u8>, &'static str> {
    match algorithm {
        EccAlgorithm::P256 => {
            let recipient_public_key = P256PublicKey::from_sec1_bytes(public_key_bytes)
                .map_err(|_| "Invalid P256 public key for encryption")?;
            let ephemeral_secret = P256EphemeralSecret::random(&mut OsRng);
            let ephemeral_public_key_point = ephemeral_secret.public_key();
            let shared_secret = ephemeral_secret.diffie_hellman(&recipient_public_key);

            let hk = Hkdf::<Sha256>::new(None, shared_secret.raw_secret_bytes().as_ref());
            let mut okm = [0u8; 44]; 
            hk.expand(b"aes-256-gcm-key-nonce", &mut okm).map_err(|_| "HKDF expansion failed")?;
            let (key_bytes, nonce_bytes) = okm.split_at(32);
            
            let key = Key::<Aes256Gcm>::from_slice(key_bytes);
            let nonce = Nonce::from_slice(nonce_bytes);

            let cipher = Aes256Gcm::new(&key); 
            let ciphertext = cipher.encrypt(nonce, data)
                .map_err(|_| "AES-GCM encryption failed")?;

            let mut result = Vec::new();
            result.extend_from_slice(ephemeral_public_key_point.to_encoded_point(false).as_bytes());
            result.extend_from_slice(&ciphertext);
            Ok(result)
        }
        EccAlgorithm::P384 => {
            let recipient_public_key = P384PublicKey::from_sec1_bytes(public_key_bytes)
                .map_err(|_| "Invalid P384 public key for encryption")?;
            let ephemeral_secret = P384EphemeralSecret::random(&mut OsRng);
            let ephemeral_public_key_point = ephemeral_secret.public_key();
            let shared_secret = ephemeral_secret.diffie_hellman(&recipient_public_key);

            let hk = Hkdf::<Sha384>::new(None, shared_secret.raw_secret_bytes().as_ref());
            let mut okm = [0u8; 44]; 
            hk.expand(b"aes-256-gcm-key-nonce", &mut okm).map_err(|_| "HKDF expansion failed")?;
            let (key_bytes, nonce_bytes) = okm.split_at(32);

            let key = Key::<Aes256Gcm>::from_slice(key_bytes);
            let nonce = Nonce::from_slice(nonce_bytes);

            let cipher = Aes256Gcm::new(&key);
            let ciphertext = cipher.encrypt(nonce, data)
                .map_err(|_| "AES-GCM encryption failed")?;

            let mut result = Vec::new();
            result.extend_from_slice(ephemeral_public_key_point.to_encoded_point(false).as_bytes());
            result.extend_from_slice(&ciphertext);
            Ok(result)
        }
    }
}

pub fn decrypt(encrypted_data_with_key: &[u8], private_key_bytes: &[u8], algorithm: EccAlgorithm) -> Result<Vec<u8>, &'static str> {
    match algorithm {
        EccAlgorithm::P256 => {
            let local_secret_key = P256SecretKey::from_slice(private_key_bytes)
                .map_err(|_| "Invalid P256 private key for decryption")?;

            let ephemeral_pk_size = 1 + 32 + 32; 
            if encrypted_data_with_key.len() < ephemeral_pk_size {
                return Err("Encrypted data too short to contain P256 ephemeral public key");
            }
            let (ephemeral_public_key_bytes, ciphertext) = encrypted_data_with_key.split_at(ephemeral_pk_size);
            
            let ephemeral_public_key = P256PublicKey::from_sec1_bytes(ephemeral_public_key_bytes)
                .map_err(|_| "Invalid P256 ephemeral public key")?;

            let shared_secret = p256::ecdh::diffie_hellman(local_secret_key.to_nonzero_scalar(), ephemeral_public_key.as_affine());

            let hk = Hkdf::<Sha256>::new(None, shared_secret.raw_secret_bytes().as_ref());
            let mut okm = [0u8; 44]; 
            hk.expand(b"aes-256-gcm-key-nonce", &mut okm).map_err(|_| "HKDF expansion failed for P256 decryption")?;
            let (key_bytes, nonce_bytes) = okm.split_at(32);

            let key = Key::<Aes256Gcm>::from_slice(key_bytes);
            let nonce = Nonce::from_slice(nonce_bytes);
            let cipher = Aes256Gcm::new(&key);

            cipher.decrypt(nonce, ciphertext)
                .map_err(|_| "AES-GCM P256 decryption failed")
        }
        EccAlgorithm::P384 => {
             let local_secret_key = P384SecretKey::from_slice(private_key_bytes)
                .map_err(|_| "Invalid P384 private key for decryption")?;

            let ephemeral_pk_size = 1 + 48 + 48; 
             if encrypted_data_with_key.len() < ephemeral_pk_size {
                return Err("Encrypted data too short to contain P384 ephemeral public key");
            }
            let (ephemeral_public_key_bytes, ciphertext) = encrypted_data_with_key.split_at(ephemeral_pk_size);

            let ephemeral_public_key = P384PublicKey::from_sec1_bytes(ephemeral_public_key_bytes)
                .map_err(|_| "Invalid P384 ephemeral public key")?;
            
            let shared_secret = p384::ecdh::diffie_hellman(local_secret_key.to_nonzero_scalar(), ephemeral_public_key.as_affine());

            let hk = Hkdf::<Sha384>::new(None, shared_secret.raw_secret_bytes().as_ref());
            let mut okm = [0u8; 44]; 
            hk.expand(b"aes-256-gcm-key-nonce", &mut okm).map_err(|_| "HKDF expansion failed for P384 decryption")?;
            let (key_bytes, nonce_bytes) = okm.split_at(32);

            let key = Key::<Aes256Gcm>::from_slice(key_bytes);
            let nonce = Nonce::from_slice(nonce_bytes);
            let cipher = Aes256Gcm::new(&key);

            cipher.decrypt(nonce, ciphertext)
                .map_err(|_| "AES-GCM P384 decryption failed")
        }
    }
}

// Helper to map piv::AlgorithmId to EccAlgorithm
pub fn from_piv_algorithm_id(piv_alg_id: piv_card::AlgorithmId) -> Result<EccAlgorithm, &'static str> {
    match piv_alg_id {
        piv_card::AlgorithmId::EccP256 => Ok(EccAlgorithm::P256),
        piv_card::AlgorithmId::EccP384 => Ok(EccAlgorithm::P384),
        _ => Err("Unsupported PIV AlgorithmId for ECC"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use yubikey::piv::AlgorithmId as PivAlgorithmId; // Reverted to original yubikey::piv import for tests

    #[test]
    fn test_generate_key_pair_p256() {
        let (private_key, public_key) = generate_key_pair(EccAlgorithm::P256);
        assert!(!private_key.is_empty());
        assert!(!public_key.is_empty());
        // P256 private key is 32 bytes
        assert_eq!(private_key.len(), 32);
        // P256 uncompressed public key is 1 (prefix 0x04) + 32 (x) + 32 (y) = 65 bytes
        assert_eq!(public_key.len(), 65);
    }

    #[test]
    fn test_generate_key_pair_p384() {
        let (private_key, public_key) = generate_key_pair(EccAlgorithm::P384);
        assert!(!private_key.is_empty());
        assert!(!public_key.is_empty());
        // P384 private key is 48 bytes
        assert_eq!(private_key.len(), 48);
        // P384 uncompressed public key is 1 (prefix 0x04) + 48 (x) + 48 (y) = 97 bytes
        assert_eq!(public_key.len(), 97);
    }

    #[test]
    fn test_sign_verify_p256() {
        let (private_key, public_key) = generate_key_pair(EccAlgorithm::P256);
        let data = b"hello world";

        let signature = sign(data, &private_key, EccAlgorithm::P256).expect("P256 signing failed");
        assert!(!signature.is_empty());

        let is_valid = verify(data, &signature, &public_key, EccAlgorithm::P256).expect("P256 verification failed");
        assert!(is_valid);
    }

    #[test]
    fn test_sign_verify_p384() {
        let (private_key, public_key) = generate_key_pair(EccAlgorithm::P384);
        let data = b"hello world";

        let signature = sign(data, &private_key, EccAlgorithm::P384).expect("P384 signing failed");
        assert!(!signature.is_empty());

        let is_valid = verify(data, &signature, &public_key, EccAlgorithm::P384).expect("P384 verification failed");
        assert!(is_valid);
    }

    #[test]
    fn test_sign_verify_p256_invalid_signature() {
        let (private_key, public_key) = generate_key_pair(EccAlgorithm::P256);
        let data = b"hello world";
        let wrong_data = b"hello mars";

        let signature = sign(data, &private_key, EccAlgorithm::P256).expect("P256 signing failed");
        
        let is_valid = verify(wrong_data, &signature, &public_key, EccAlgorithm::P256).expect("P256 verification with wrong data failed");
        assert!(!is_valid);

        let mut tampered_signature = signature.clone();
        tampered_signature[0] ^= 0xff; // Tamper with the signature
        let is_valid_tampered = verify(data, &tampered_signature, &public_key, EccAlgorithm::P256).expect("P256 verification with tampered signature failed");
        assert!(!is_valid_tampered);
    }
    
    #[test]
    fn test_encrypt_decrypt_p256() {
        let (private_key_bytes, public_key_bytes) = generate_key_pair(EccAlgorithm::P256);
        let original_data = b"super secret message for P256";

        let encrypted_data = encrypt(original_data, &public_key_bytes, EccAlgorithm::P256)
            .expect("P256 encryption failed");
        assert!(!encrypted_data.is_empty());

        let decrypted_data = decrypt(&encrypted_data, &private_key_bytes, EccAlgorithm::P256)
            .expect("P256 decryption failed");
        
        assert_eq!(original_data.to_vec(), decrypted_data);
    }

    #[test]
    fn test_encrypt_decrypt_p384() {
        let (private_key_bytes, public_key_bytes) = generate_key_pair(EccAlgorithm::P384);
        let original_data = b"super secret message for P384";

        let encrypted_data = encrypt(original_data, &public_key_bytes, EccAlgorithm::P384)
            .expect("P384 encryption failed");
        assert!(!encrypted_data.is_empty());

        let decrypted_data = decrypt(&encrypted_data, &private_key_bytes, EccAlgorithm::P384)
            .expect("P384 decryption failed");
        
        assert_eq!(original_data.to_vec(), decrypted_data);
    }

    #[test]
    fn test_encrypt_decrypt_p256_wrong_key() {
        let (_private_key_bytes, public_key_bytes) = generate_key_pair(EccAlgorithm::P256);
        let (wrong_private_key_bytes, _wrong_public_key_bytes) = generate_key_pair(EccAlgorithm::P256);
        let original_data = b"another secret message";

        let encrypted_data = encrypt(original_data, &public_key_bytes, EccAlgorithm::P256)
            .expect("P256 encryption failed");

        let decryption_result = decrypt(&encrypted_data, &wrong_private_key_bytes, EccAlgorithm::P256);
        assert!(decryption_result.is_err(), "Decryption with wrong P256 key should fail");
        assert_eq!(decryption_result.unwrap_err(), "AES-GCM P256 decryption failed");
    }
    
    #[test]
    fn test_encrypt_decrypt_p384_wrong_key() {
        let (_private_key_bytes, public_key_bytes) = generate_key_pair(EccAlgorithm::P384);
        let (wrong_private_key_bytes, _wrong_public_key_bytes) = generate_key_pair(EccAlgorithm::P384);
        let original_data = b"yet another secret for P384";

        let encrypted_data = encrypt(original_data, &public_key_bytes, EccAlgorithm::P384)
            .expect("P384 encryption failed");

        let decryption_result = decrypt(&encrypted_data, &wrong_private_key_bytes, EccAlgorithm::P384);
        assert!(decryption_result.is_err(), "Decryption with wrong P384 key should fail");
        assert_eq!(decryption_result.unwrap_err(), "AES-GCM P384 decryption failed");
    }

    #[test]
    fn test_from_piv_algorithm_id() {
        assert!(matches!(from_piv_algorithm_id(PivAlgorithmId::EccP256), Ok(EccAlgorithm::P256)));
        assert!(matches!(from_piv_algorithm_id(PivAlgorithmId::EccP384), Ok(EccAlgorithm::P384)));
        
        // Test an unsupported algorithm
        assert!(matches!(from_piv_algorithm_id(PivAlgorithmId::Rsa2048), Err(_)));
        // Example of another unsupported one, if it exists in piv_card::AlgorithmId
        // assert!(matches!(from_piv_algorithm_id(PivAlgorithmId::Des3), Err(_))); 
    }
}
