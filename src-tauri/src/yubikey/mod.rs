mod experimental_setup;

use crate::error::{Error, Result};
use crate::encrypt; 
use crate::AppState; // Changed: split from previous line
use base64::Engine;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use yubikey::{YubiKey, piv};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct YubiKeyInfo {
    pub serial: Option<u32>,
    pub name: String,
    pub version: Option<String>,
    pub is_fips: bool,
    pub form_factor: String,
    pub pub_key: Option<String>
}

#[cfg(test)]
impl Default for YubiKeyInfo {
    fn default() -> Self {
        Self {
            serial: None,
            name: "YubiKey".to_string(),
            version: None,
            is_fips: false,
            form_factor: "YubiKey".to_string(),
            pub_key: None
        }
    }
}

impl YubiKeyInfo {
    fn from_yubikey(key: &mut YubiKey) -> Self { // Takes &mut YubiKey
        // Get serial number
        let serial_u32 = u32::from(key.serial());

        // Format the version as a string
        let version = Some(key.version().to_string());

        // YubiKey 0.8.0 doesn't expose form_factor directly
        let form_factor = "YubiKey".to_string();

        // Try to get the public key using the new helper
        let pub_key = YubiKeyInfo::extract_public_key_pem(key).ok();

        Self {
            serial: Some(serial_u32),
            name: format!("YubiKey {}", serial_u32),
            version,
            is_fips: false, // Not directly accessible in 0.8.0
            form_factor,
            pub_key,
        }
    }

    // Helper function to extract PEM-encoded public key from a YubiKey instance
    fn extract_public_key_pem(yubikey: &mut YubiKey) -> Result<String> {
        use rsa::pkcs1::der::EncodePem;
        let slot = piv::SlotId::KeyManagement;
        let cert = yubikey::certificate::Certificate::read(yubikey, slot)
            .map_err(|e| Error::YubiKeyError(format!("Failed to get certificate from slot {:?} for pubkey extraction: {}", slot, e)))?;
        cert.subject_pki().to_pem(rsa::pkcs1::der::pem::LineEnding::LF)
            .map_err(|e| Error::YubiKeyError(format!("Failed to encode public key to PEM for pubkey extraction: {}", e)))
    }

    pub fn set_pub_key(&mut self, pub_key: String) {
        self.pub_key = Some(pub_key);
    }

    pub fn save(&self, app_state: &AppState) -> Result<()> {
        let fs = app_state.file_system();
        let data = serde_json::to_string(self).map_err(|e| Error::YubiKeyError(e.to_string()))?;
        // TODO: encrypt the data before saving with master password
        std::fs::write(fs.yubikey_settings(), data).map_err(|e| Error::YubiKeyError(e.to_string()))?;
        Ok(())
    }

    pub fn get(app_state: &AppState) -> Result<Self> {
        let fs = app_state.file_system();
        let data = std::fs::read_to_string(fs.yubikey_settings()).map_err(|e| Error::YubiKeyError(e.to_string()))?;
        let yubikey_info: YubiKeyInfo = serde_json::from_str(&data).map_err(|e| Error::YubiKeyError(e.to_string()))?;
        Ok(yubikey_info)
    }
}

impl From<AppState> for YubiKeyInfo {
    fn from(app_state: AppState) -> Self {
        YubiKeyInfo::get(&app_state).unwrap()
    }
}
// Error handler for YubiKey operations
impl From<yubikey::Error> for Error {
    fn from(err: yubikey::Error) -> Self {
        Error::YubiKeyError(format!("{}", err))
    }
}

/// List all connected YubiKeys
pub fn list_yubikeys() -> Result<Vec<YubiKeyInfo>> {
    let mut keys = Vec::new();
    let mut context = yubikey::reader::Context::open()
        .map_err(|e| Error::YubiKeyError(format!("Failed to open YubiKey context: {}", e)))?;

    // Find all YubiKey readers
    match context.iter() {
        Ok(readers) => {
            for reader in readers {
                // Try to open the YubiKey from this reader
                if let Ok(mut yubikey_instance) = reader.open() { // yubikey_instance is now mutable
                    keys.push(YubiKeyInfo::from_yubikey(&mut yubikey_instance)); // Pass &mut
                }
            }
            Ok(keys)
        }
        Err(e) => Err(Error::YubiKeyError(format!(
            "Failed to iterate readers: {}",
            e
        ))),
    }
}

pub fn encrypt_with_yubikey(app_state: &AppState, data: &str) -> Result<String> {
    let info = YubiKeyInfo::get(app_state)?;
    let pub_key = info.pub_key.ok_or(Error::YubiKeyError("Public key not found".to_string()))?;
    let encryptor = crate::encrypt::PublicKey::from_pem(&pub_key)?;
    encryptor.encrypt(data.as_bytes())
        .map_err(|e| Error::YubiKeyError(e.to_string()))
        .map(|encrypted| base64::engine::general_purpose::STANDARD.encode(&encrypted))
}





// New struct to wrap a YubiKey instance
pub struct YubiKeyDevice {
    yk: YubiKey,
    authentication: Option<piv::AlgorithmId>,
    key_management: Option<piv::AlgorithmId>,
}

impl YubiKeyDevice {
    /// Opens a YubiKey by its serial number and wraps it.
    pub fn open(serial_u32: u32) -> Result<Self> {
        let serial = yubikey::Serial::from(serial_u32);
        let mut yk = YubiKey::open_by_serial(serial)
            .map_err(|e| Error::YubiKeyError(format!("Failed to open YubiKey {}: {}", serial_u32, e)))?;

        // Attempt to get authentication algorithm
        let authentication_algorithm = piv::metadata(&mut yk, piv::SlotId::Authentication)
            .ok()
            .and_then(|meta| match meta.algorithm {
                piv::ManagementAlgorithmId::Asymmetric(alg_id) => Some(alg_id),
                _ => None,
            });

        // Attempt to get key management algorithm
        let key_management_algorithm = piv::metadata(&mut yk, piv::SlotId::KeyManagement)
            .ok()
            .and_then(|meta| match meta.algorithm {
                piv::ManagementAlgorithmId::Asymmetric(alg_id) => Some(alg_id),
                _ => None,
            });

        Ok(Self {
            yk,
            authentication: authentication_algorithm,
            key_management: key_management_algorithm,
        })
    }

    /// Retrieves the public key from the YubiKey's Key Management slot and its algorithm ID.
    pub fn get_public_key(&mut self) -> Result<String> {
        use rsa::pkcs1::der::EncodePem;
        let slot = piv::SlotId::KeyManagement;

        // Use the stored key_management algorithm if available
        self.key_management.ok_or_else(|| {
            Error::YubiKeyError(format!(
                "Key management algorithm not found for slot {:?} for decryption. Device might not have been properly initialized or slot is not configured.",
                slot
            ))
        })?;

        // Ensure PIN is verified if required by YubiKey policy for reading certificate/pubkey
        // This depends on YubiKey\'s PIV configuration. For simplicity, assuming it might not always be needed
        // or that prior operations (like decrypt/sign which verify PIN) are sufficient.
        // If PIN is strictly required for cert reading, it should be passed here.
        // However, typically reading public certs doesn\'t require PIN.

        let cert = yubikey::certificate::Certificate::read(&mut self.yk, slot)
            .map_err(|e| Error::YubiKeyError(format!("Failed to get certificate from slot {:?}: {}", slot, e)))?;
        
        let pem = cert.subject_pki().to_pem(rsa::pkcs1::der::pem::LineEnding::LF)
            .map_err(|e| Error::YubiKeyError(format!("Failed to encode public key to PEM: {}", e)))?;
        Ok(pem)
    }

    /// Encrypts data using the YubiKey\\\'s public key (retrieved from the device).
    pub fn encrypt_data(&mut self, data: Vec<u8>) -> Result<String> {
        // Retrieve the algorithm ID from the stored key_management metadata.
        let algorithm_id = self.key_management.ok_or_else(|| {
            Error::YubiKeyError(
                "Key management algorithm not found. Device might not have been properly initialized or slot is not configured.".to_string(),
            )
        })?;

        match algorithm_id {
            piv::AlgorithmId::Rsa1024 | piv::AlgorithmId::Rsa2048 => {
                // The key is RSA, proceed with RSA encryption.
                let pub_key_pem = self.get_public_key()?; // Fetches the PEM-encoded public key.
                
                let encryptor = crate::encrypt::PublicKey::from_pem(&pub_key_pem)?;
                let encrypted_bytes = encryptor.encrypt(&data)?;
                Ok(base64::engine::general_purpose::STANDARD.encode(&encrypted_bytes))
            }
            piv::AlgorithmId::EccP256 | piv::AlgorithmId::EccP384 => {
                // The key is ECC.
                Err(Error::YubiKeyError(format!(
                    "Encryption with ECC algorithm ({:?}) is not currently supported by this function. Only RSA encryption is available.",
                    algorithm_id
                )))
            }
            // All variants of piv::AlgorithmId relevant here (Rsa1024, Rsa2048, EccP256, EccP384)
            // are explicitly handled above. No other Asymmetric AlgorithmId types are defined
            // in the provided enum that would be stored in self.key_management.
        }
    }

    /// Decrypts data using the YubiKey.
    pub fn decrypt_data(
        &mut self,
        pin: String,
        encrypted_data_base64_bytes: Vec<u8>,
    ) -> Result<Vec<u8>> {
        self.yk.verify_pin(pin.as_bytes())?;

        let slot = piv::SlotId::KeyManagement;

        // Use the stored key_management algorithm if available
        let algorithm = self.key_management.ok_or_else(|| {
            Error::YubiKeyError(format!(
                "Key management algorithm not found for slot {:?} for decryption. Device might not have been properly initialized or slot is not configured.",
                slot
            ))
        })?;

        let raw_ciphertext = base64::engine::general_purpose::STANDARD.decode(&encrypted_data_base64_bytes)
            .map_err(|e| Error::YubiKeyError(format!("Failed to decode base64 encrypted data: {}", e)))?;

        let decrypted_zeroizing_vec = piv::decrypt_data(&mut self.yk, &raw_ciphertext, algorithm, slot)
            .map_err(|e| Error::from(e))?;
        
        let padded_data = decrypted_zeroizing_vec.to_vec();
        let block_len = padded_data.len();

        // PKCS#1 v1.5 unpadding (EME-PKCS1-v1_5 type 2)
        if block_len < 11 {
            return Err(Error::YubiKeyError(format!(
                "Decryption error: Padded data too short ({} bytes). Expected at least 11 for PKCS#1 v1.5.",
                block_len
            )));
        }
        if padded_data[0] != 0x00 || padded_data[1] != 0x02 {
            return Err(Error::YubiKeyError(format!(
                "Decryption error: Invalid PKCS#1 v1.5 padding header. Expected 00 02, got: {:02X} {:02X}",
                padded_data.get(0).cloned().unwrap_or(0xFF),
                padded_data.get(1).cloned().unwrap_or(0xFF)
            )));
        }
        let mut separator_index: Option<usize> = None;
        for i in 2..block_len {
            if padded_data[i] == 0x00 {
                separator_index = Some(i);
                break;
            }
        }
        match separator_index {
            Some(idx) => {
                if (idx - 2) < 8 {
                    return Err(Error::YubiKeyError(format!(
                        "Decryption error: PKCS#1 v1.5 padding string (PS) too short ({} bytes). Minimum 8 bytes required. Separator at index {}.",
                        idx - 2, idx
                    )));
                }
                if (idx + 1) >= block_len {
                     return Err(Error::YubiKeyError(
                        "Decryption error: PKCS#1 v1.5 unpadding resulted in an empty message.".to_string(),
                    ));
                }
                Ok(padded_data[idx + 1..].to_vec())
            }
            None => Err(Error::YubiKeyError(
                "Decryption error: PKCS#1 v1.5 padding separator 0x00 not found after PS.".to_string(),
            )),
        }
    }

    pub fn generate_authentication_challenge(&mut self) -> Result<String> {
        let slot = piv::SlotId::Authentication;
        
        // Use the stored authentication algorithm if available
        let alg_id = self.authentication.ok_or_else(|| {
            Error::YubiKeyError(format!(
                "Authentication algorithm not found for slot {:?} for challenge generation. Device might not have been properly initialized or slot is not configured.",
                slot
            ))
        })?;

        let challenge_len = match alg_id {
            piv::AlgorithmId::EccP384 => 48,
            piv::AlgorithmId::Rsa1024 | piv::AlgorithmId::Rsa2048 | piv::AlgorithmId::EccP256 => 32,
            _ => return Err(Error::YubiKeyError(format!(
                "Unsupported asymmetric algorithm {:?} in authentication slot {:?} for challenge generation.",
                alg_id, slot
            ))),
        };

        let mut rng = rand::thread_rng();
        let mut challenge_bytes = vec![0u8; challenge_len];
        rng.fill_bytes(&mut challenge_bytes);

        Ok(base64::engine::general_purpose::STANDARD.encode(&challenge_bytes))
    }
    /// Authenticates with the YubiKey by signing a challenge.
    pub fn authenticate(
        &mut self,
        pin: String,
        challenge_base64: &str,
    ) -> Result<String> {
        self.yk.verify_pin(pin.as_bytes())
            .map_err(|e| Error::YubiKeyError(format!("PIN verification failed: {}", e)))?;

        let slot = piv::SlotId::Authentication; // Slot 9A

        let alg_id = self.authentication.ok_or_else(|| {
            Error::YubiKeyError(format!(
                "Authentication algorithm not found for slot {:?}. Device might not have been properly initialized or slot is not configured.",
                slot
            ))
        })?;

        let challenge_bytes = base64::engine::general_purpose::STANDARD
            .decode(challenge_base64)
            .map_err(|e| Error::YubiKeyError(format!("Invalid challenge format (base64 decode failed): {}", e)))?;
        
        match alg_id { 
            piv::AlgorithmId::Rsa1024 | piv::AlgorithmId::Rsa2048 | piv::AlgorithmId::EccP256 => {
                if challenge_bytes.len() != 32 {
                    return Err(Error::YubiKeyError(format!(
                        "Challenge size mismatch for {:?}: expected 32 bytes, got {}. This indicates an issue with challenge generation or decoding.",
                        alg_id, challenge_bytes.len()
                    )));
                }
            }
            piv::AlgorithmId::EccP384 => {
                if challenge_bytes.len() == 32 {
                    return Err(Error::YubiKeyError(format!(
                        "The key in slot {:?} is {:?}, which typically requires a 48-byte hash (e.g., SHA-384) for signing. The provided challenge is 32 bytes. This combination is problematic and may lead to smart card errors. The challenge must be a 48-byte hash for use with this {:?} key.",
                        slot, alg_id, alg_id
                    )));
                } else if challenge_bytes.len() != 48 {
                    return Err(Error::YubiKeyError(format!(
                        "Challenge size mismatch for {:?}: expected 48 bytes (SHA-384 hash), got {} bytes.",
                        alg_id, challenge_bytes.len()
                    )));
                }
            }
        }

        let signature_bytes = piv::sign_data(&mut self.yk, &challenge_bytes, alg_id, slot)
            .map_err(|e| Error::YubiKeyError(format!("Failed to sign challenge with algorithm {:?} in slot {:?}: {}", alg_id, slot, e)))?;

        if signature_bytes.is_empty() {
            return Err(Error::YubiKeyError(format!("Authentication failed: produced an empty signature with algorithm {:?} in slot {:?}", alg_id, slot)));
        }
        Ok(base64::engine::general_purpose::STANDARD.encode(&signature_bytes))
    }
}



#[cfg(test)]
mod test {

    use super::*;
    use serial_test::serial;

    fn get_device() -> YubiKeyDevice {
        let yubikey_serial = 32233649; // Standard test serial
        YubiKeyDevice::open(yubikey_serial).unwrap()
    }

    #[test]
    #[serial]
    fn test_encrypt_with_yubikey() { // This test was actually for do_encrypt
        let mut device = get_device();
        // Using the free function which now wraps YubiKeyDevice
        let result = device.encrypt_data("data".as_bytes().to_vec());
        assert!(result.is_ok(), "do_encrypt failed: {:?}", result.err());
        result.unwrap(); // Check it unwraps
    }

    #[test]
    #[serial]
    fn test_authenticate_with_yubikey() {
        let yubikey_serial = 32233649;
        let pin = "123456".to_string();
        let mut device = YubiKeyDevice::open(yubikey_serial).unwrap(); // Ensure the YubiKey is open
        let challenge = device.generate_authentication_challenge().unwrap();
        
        // Using the free function
        let signature_result = device.authenticate(pin, &challenge);
        
        assert!(signature_result.is_ok(), "Authentication failed: {:?}", signature_result.err());
        let signature_base64 = signature_result.unwrap();
        assert!(!signature_base64.is_empty(), "Signature should not be empty");
    }

    #[test]
    #[serial]
    fn test_decrypt_data() {
        let mut device = get_device();
        let pin = "123456".to_string();
        let original_data_str = "This is a secret test message for YubiKey!";
        let original_data_bytes = original_data_str.as_bytes().to_vec();

        println!("Original data (str): {}", original_data_str);
        println!("Original data bytes (len {}): {:?}", original_data_bytes.len(), original_data_bytes);
        println!("Original data bytes (hex): {}", original_data_bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>());

        // 1. Encrypt the data using the YubiKey's public key (via free function)
        let encrypted_data_base64_result = device.encrypt_data(original_data_bytes.clone());
        assert!(encrypted_data_base64_result.is_ok(), "Encryption failed: {:?}", encrypted_data_base64_result.err());
        let encrypted_data_base64_string = encrypted_data_base64_result.unwrap();
        println!("Encrypted data (base64 string): {}", encrypted_data_base64_string);

        // 2. Decrypt the data using the YubiKey (via free function)
        let decrypted_bytes_result = device.decrypt_data(pin, encrypted_data_base64_string.into_bytes());
        
        if let Err(e) = &decrypted_bytes_result {
            eprintln!("Raw decryption function returned error: {:?}", e);
        }
        assert!(decrypted_bytes_result.is_ok(), "Raw decryption failed: {:?}", decrypted_bytes_result.err());
        let decrypted_bytes = decrypted_bytes_result.unwrap();

        println!("Decrypted data bytes (len {}): {:?}", decrypted_bytes.len(), decrypted_bytes);
        println!("Decrypted data bytes (hex): {}", decrypted_bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>());

        // 3. Verify that the decrypted raw bytes match the original raw bytes
        // First check length, then content for more specific error messages if they differ.
        if original_data_bytes.len() != decrypted_bytes.len() {
            assert_eq!(decrypted_bytes.len(), original_data_bytes.len(), "Decrypted data length mismatch.");
        }
        assert_eq!(decrypted_bytes, original_data_bytes, "Decrypted raw byte content does not match original raw byte content.");

        // 4. Convert decrypted bytes to string and verify
        let decrypted_string_result = String::from_utf8(decrypted_bytes);
        assert!(decrypted_string_result.is_ok(), "UTF-8 conversion of decrypted bytes failed: {:?}", decrypted_string_result.err());
        let decrypted_data_final_str = decrypted_string_result.unwrap();

        assert_eq!(decrypted_data_final_str, original_data_str, "Decrypted string does not match original string.");
    }

    #[test]
    #[serial]
    fn test_get_public_key_success() {
        let mut device = get_device();

        let result = device.get_public_key();
        assert!(result.is_ok(), "get_public_key failed: {:?}", result.err());

        let pem = result.unwrap(); 
        println!("Public key PEM:\n{}", pem);
        assert!(!pem.is_empty(), "PEM string should not be empty");
        assert!(pem.starts_with("-----BEGIN PUBLIC KEY-----"), "PEM string should start with -----BEGIN PUBLIC KEY-----");
        assert!(pem.ends_with("-----END PUBLIC KEY-----\n"), "PEM string should end with -----END PUBLIC KEY-----");

        if let Some(alg_id) = device.key_management {
            match alg_id {
                piv::AlgorithmId::Rsa1024 |
                piv::AlgorithmId::Rsa2048 |
                piv::AlgorithmId::EccP256 |
                piv::AlgorithmId::EccP384 => {
                    println!("Device key management slot configured with algorithm: {:?}", alg_id);
                }
                // All variants of piv::AlgorithmId relevant here are explicitly handled.
            }
        } else {
            panic!("Key management algorithm not found on device after get_public_key success. This indicates an issue with device initialization or test setup.");
        }
    }

    #[test]
    #[serial]
    fn test_yubikey_device_authenticate_success() {
        let mut device = get_device();
        let pin = "123456".to_string();    // Standard test PIN
        let challenge = device.generate_authentication_challenge().unwrap();

        let signature_result = device.authenticate(pin, &challenge);
        
        assert!(signature_result.is_ok(), "YubiKeyDevice.authenticate failed: {:?}", signature_result.err());
        let signature_base64 = signature_result.unwrap();
        assert!(!signature_base64.is_empty(), "Signature from YubiKeyDevice.authenticate should not be empty");
    }

    #[test]
    #[serial]
    fn test_yubikey_device_authenticate_incorrect_pin() {
        let mut device = get_device();
        let incorrect_pin = "000000".to_string(); // An incorrect PIN
        let challenge = device.generate_authentication_challenge().unwrap();

        let signature_result = device.authenticate(incorrect_pin, &challenge);
        
        assert!(signature_result.is_err(), "YubiKeyDevice.authenticate should fail with an incorrect PIN");
        if let Err(Error::YubiKeyError(msg)) = signature_result {
            assert!(msg.starts_with("PIN verification failed:"), "Error message mismatch for incorrect PIN. Got: {}", msg);
        } else {
            panic!("Expected YubiKeyError for incorrect PIN, got {:?}", signature_result);
        }
    }

    #[test]
    #[serial]
    fn test_yubikey_device_authenticate_malformed_challenge() {
        let yubikey_serial = 32233649;
        let pin = "123456".to_string();
        let malformed_challenge = "this is not valid base64 characters !!!"; // Malformed challenge

        let device_result = YubiKeyDevice::open(yubikey_serial);
        assert!(device_result.is_ok(), "Failed to open YubiKeyDevice for testing: {:?}", device_result.err());
        let mut device = device_result.unwrap();

        let signature_result = device.authenticate(pin, malformed_challenge);
        
        assert!(signature_result.is_err(), "YubiKeyDevice.authenticate should fail with a malformed challenge");
        if let Err(Error::YubiKeyError(msg)) = signature_result {
            assert!(msg.contains("Invalid challenge format (base64 decode failed):"), "Error message mismatch for malformed challenge. Got: {}", msg);
        } else {
            panic!("Expected YubiKeyError for malformed challenge, got {:?}", signature_result);
        }
    }
}
