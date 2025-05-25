mod experimental_setup;

use crate::error::{Error, Result};
use crate::{encrypt, AppState};
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

pub fn decrypt_with_yubikey(
    yubikey_serial: u32,
    pin: String,
    encrypted_data_base64_bytes: Vec<u8>, // Input is now explicitly named to reflect it's base64 bytes
) -> Result<Vec<u8>> { // Return raw decrypted bytes
    let mut device = YubiKeyDevice::open(yubikey_serial)?;
    device.decrypt_data(pin, encrypted_data_base64_bytes)
}

/// Authenticate with YubiKey by signing a challenge.
/// Returns the signature as a base64 encoded string if successful.
pub fn authenticate_with_yubikey(
    yubikey_serial: u32,
    pin: String,
    challenge_base64: &str, // Renamed for clarity, expects base64 encoded challenge
) -> Result<String> { // Changed return type from bool to String (base64 signature)
    let mut device = YubiKeyDevice::open(yubikey_serial)?;
    device.authenticate(pin, challenge_base64)
}

// New struct to wrap a YubiKey instance
pub struct YubiKeyDevice {
    yk: YubiKey,
}

impl YubiKeyDevice {
    /// Opens a YubiKey by its serial number and wraps it.
    pub fn open(serial_u32: u32) -> Result<Self> {
        let serial = yubikey::Serial::from(serial_u32);
        let yk = YubiKey::open_by_serial(serial)
            .map_err(|e| Error::YubiKeyError(format!("Failed to open YubiKey {}: {}", serial_u32, e)))?;
        Ok(Self { yk })
    }

    /// Retrieves the public key from the YubiKey's Key Management slot and its algorithm ID.
    pub fn get_public_key(&mut self) -> Result<(String, piv::AlgorithmId)> {
        use rsa::pkcs1::der::EncodePem;
        let slot = piv::SlotId::KeyManagement;

        // Get metadata to find out the algorithm
        let metadata = piv::metadata(&mut self.yk, slot)
            .map_err(|e| Error::YubiKeyError(format!("Failed to get metadata from slot {:?}: {}", slot, e)))?;
        
        let algorithm_id = match metadata.algorithm {
            piv::ManagementAlgorithmId::Asymmetric(alg_id) => alg_id,
            _ => return Err(Error::YubiKeyError(format!(
                "Slot {:?} does not contain an asymmetric key, found algorithm: {:?}",
                slot, metadata.algorithm
            ))),
        };

        // Ensure PIN is verified if required by YubiKey policy for reading certificate/pubkey
        // This depends on YubiKey's PIV configuration. For simplicity, assuming it might not always be needed
        // or that prior operations (like decrypt/sign which verify PIN) are sufficient.
        // If PIN is strictly required for cert reading, it should be passed here.
        // However, typically reading public certs doesn't require PIN.

        let cert = yubikey::certificate::Certificate::read(&mut self.yk, slot)
            .map_err(|e| Error::YubiKeyError(format!("Failed to get certificate from slot {:?}: {}", slot, e)))?;
        
        let pem = cert.subject_pki().to_pem(rsa::pkcs1::der::pem::LineEnding::LF)
            .map_err(|e| Error::YubiKeyError(format!("Failed to encode public key to PEM: {}", e)))?;
        Ok((pem, algorithm_id))
    }

    /// Encrypts data using the YubiKey's public key (retrieved from the device).
    pub fn encrypt_data(&mut self, data: Vec<u8>) -> Result<String> {
        let (pub_key_pem, _algorithm_id) = self.get_public_key()?; // algorithm_id is not used here but fetched
        let encryptor = encrypt::PublicKey::from_pem(&pub_key_pem)?;
        let encrypted_bytes = encryptor.encrypt(&data)?;
        Ok(base64::engine::general_purpose::STANDARD.encode(&encrypted_bytes))
    }

    /// Decrypts data using the YubiKey.
    pub fn decrypt_data(
        &mut self,
        pin: String,
        encrypted_data_base64_bytes: Vec<u8>,
    ) -> Result<Vec<u8>> {
        self.yk.verify_pin(pin.as_bytes())?;

        let slot = piv::SlotId::KeyManagement;
        // TODO: select the correct algorithm based on the key metadata
        let algorithm = piv::AlgorithmId::Rsa2048;

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

    /// Authenticates with the YubiKey by signing a challenge.
    pub fn authenticate(
        &mut self,
        pin: String,
        challenge_base64: &str,
    ) -> Result<String> {
        self.yk.verify_pin(pin.as_bytes())
            .map_err(|e| Error::YubiKeyError(format!("PIN verification failed: {}", e)))?;

        let slot = piv::SlotId::Authentication; // Slot 9A

        let key_metadata = piv::metadata(&mut self.yk, slot)
            .map_err(|e| Error::YubiKeyError(format!("Failed to read PIV metadata for slot {:?}: {}", slot, e)))?;
        
        let algorithm_from_metadata = key_metadata.algorithm;

        let challenge_bytes = base64::engine::general_purpose::STANDARD
            .decode(challenge_base64)
            .map_err(|e| Error::YubiKeyError(format!("Invalid challenge format (base64 decode failed): {}", e)))?;
        
        match algorithm_from_metadata {
            piv::ManagementAlgorithmId::Asymmetric(alg_id) => {
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
                        if challenge_bytes.len() != 32 {
                            return Err(Error::YubiKeyError(format!(
                                "Challenge size mismatch for {:?}: expected 32 bytes (current setup), got {}. Note: {:?} typically uses a 48-byte hash (SHA-384).",
                                alg_id, challenge_bytes.len(), alg_id
                            )));
                        }
                        // println!("Warning: Using a 32-byte challenge for {:?}. This algorithm usually expects a 48-byte (SHA-384) hash input for piv::sign_data.", alg_id);
                    }
                }
                // Proceed to sign_data with alg_id
                let signature_bytes = piv::sign_data(&mut self.yk, &challenge_bytes, alg_id, slot)
                    .map_err(|e| Error::YubiKeyError(format!("Failed to sign challenge with algorithm {:?} in slot {:?}: {}", alg_id, slot, e)))?;

                if signature_bytes.is_empty() {
                    return Err(Error::YubiKeyError(format!("Authentication failed: produced an empty signature with algorithm {:?} in slot {:?}", alg_id, slot)));
                }
                return Ok(base64::engine::general_purpose::STANDARD.encode(&signature_bytes));
            }
            piv::ManagementAlgorithmId::ThreeDes => { // Corrected casing
                return Err(Error::YubiKeyError(format!(
                    "Algorithm {:?} in slot {:?} is symmetric and cannot be used for signing.",
                    algorithm_from_metadata, slot
                )));
            }
            piv::ManagementAlgorithmId::PinPuk => {
                 return Err(Error::YubiKeyError(format!(
                    "Algorithm {:?} in slot {:?} is for PIN/PUK management and cannot be used for signing.",
                    algorithm_from_metadata, slot
                )));
            }
            // The compiler will ensure this match is exhaustive for ManagementAlgorithmId variants.
            // If all variants return, any code after this match would be unreachable.
        }
        // If the match is exhaustive and all arms return, this line will be flagged as unreachable by the compiler, which is expected.
        // However, to satisfy the function signature that expects a Result,
        // and in case ManagementAlgorithmId gains new variants not handled above,
        // we can leave a general error here. Or, if the match is truly exhaustive,
        // this line might not be strictly necessary if the compiler understands all paths return.
        // For robustness against future enum changes if not recompiled, or if a variant was missed:
        // Err(Error::YubiKeyError(format!(
        //     "Unhandled or unsuitable algorithm type {:?} for signing in slot {:?}.",
        //     algorithm_from_metadata, slot
        // )))
    }
}

/// Get the public key from a YubiKey by serial number
pub fn get_public_key(yubikey_serial: u32) -> Result<(String, piv::AlgorithmId)> {
    let mut device = YubiKeyDevice::open(yubikey_serial)?;
    device.get_public_key()
}

/// Encrypts data using the public key of the specified YubiKey.
pub fn do_encrypt(yubikey_serial: u32, data: Vec<u8>) -> Result<String> {
    let mut device = YubiKeyDevice::open(yubikey_serial)?;
    device.encrypt_data(data)
}

/// Generate a random challenge for authentication
pub fn generate_yubikey_challenge() -> Result<String> {
    // Create a 256-bit random challenge
    let mut rng = rand::thread_rng();
    let mut challenge = [0u8; 32];
    rng.fill_bytes(&mut challenge);

    // Encode as base64 for transport
    Ok(base64::engine::general_purpose::STANDARD.encode(&challenge))
}

#[cfg(test)]
mod test {

    use super::*;
    #[test]
    fn test_yubikey_challenge() {
        let challenge = super::generate_yubikey_challenge().unwrap();
        assert_eq!(challenge.len(), 44);
    }

    use yubikey::{
        YubiKey,
        piv::{AlgorithmId, SlotId, decrypt_data},
    };

    #[test]
    fn test_encrypt_with_yubikey() { // This test was actually for do_encrypt
        let yubikey_serial = 32233649;
        // Using the free function which now wraps YubiKeyDevice
        let result = do_encrypt(yubikey_serial, "data".as_bytes().to_vec());
        assert!(result.is_ok(), "do_encrypt failed: {:?}", result.err());
        result.unwrap(); // Check it unwraps
    }

    #[test]
    fn test_authenticate_with_yubikey() {
        let yubikey_serial = 32233649;
        let pin = "123456".to_string();
        let challenge = generate_yubikey_challenge().unwrap();
        
        // Using the free function
        let signature_result = authenticate_with_yubikey(yubikey_serial, pin, &challenge);
        
        assert!(signature_result.is_ok(), "Authentication failed: {:?}", signature_result.err());
        let signature_base64 = signature_result.unwrap();
        assert!(!signature_base64.is_empty(), "Signature should not be empty");
    }

    #[test]
    fn test_slot_available() {
        let mut yubikey = YubiKey::open().unwrap();

        let slot = SlotId::KeyManagement;
        let algorithm = AlgorithmId::Rsa2048;

        let pin = "123456";
        yubikey.verify_pin(pin.as_bytes()).unwrap();

        let encrypted_data = vec![1u8; 256]; // Dummy data
        let result = decrypt_data(&mut yubikey, &encrypted_data, algorithm, slot);
        assert!(result.is_ok());
    }

    #[test]
    fn test_decrypt_data() {
        let yubikey_serial = 32233649;
        let pin = "123456".to_string();
        let original_data_str = "This is a secret test message for YubiKey!";
        let original_data_bytes = original_data_str.as_bytes().to_vec();

        println!("Original data (str): {}", original_data_str);
        println!("Original data bytes (len {}): {:?}", original_data_bytes.len(), original_data_bytes);
        println!("Original data bytes (hex): {}", original_data_bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>());

        // 1. Encrypt the data using the YubiKey's public key (via free function)
        let encrypted_data_base64_result = do_encrypt(yubikey_serial, original_data_bytes.clone());
        assert!(encrypted_data_base64_result.is_ok(), "Encryption failed: {:?}", encrypted_data_base64_result.err());
        let encrypted_data_base64_string = encrypted_data_base64_result.unwrap();
        println!("Encrypted data (base64 string): {}", encrypted_data_base64_string);

        // 2. Decrypt the data using the YubiKey (via free function)
        let decrypted_bytes_result = decrypt_with_yubikey(yubikey_serial, pin, encrypted_data_base64_string.into_bytes());
        
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
    fn test_get_public_key_success() {
        let yubikey_serial = 32233649; // Standard test serial, ensure this YubiKey is available and configured

        let result = get_public_key(yubikey_serial);
        assert!(result.is_ok(), "get_public_key failed: {:?}", result.err());

        let (pem, algorithm_id) = result.unwrap();
        assert!(!pem.is_empty(), "PEM string should not be empty");
        assert!(pem.starts_with("-----BEGIN PUBLIC KEY-----"), "PEM string should start with -----BEGIN PUBLIC KEY-----");
        assert!(pem.ends_with("-----END PUBLIC KEY-----\n"), "PEM string should end with -----END PUBLIC KEY-----\n");

        // Check if the algorithm ID is one of the expected asymmetric types
        match algorithm_id {
            piv::AlgorithmId::Rsa1024 |
            piv::AlgorithmId::Rsa2048 |
            piv::AlgorithmId::EccP256 |
            piv::AlgorithmId::EccP384 => {
                // This is an expected asymmetric algorithm
                println!("Successfully retrieved public key with algorithm: {:?}", algorithm_id);
            },
            _ => panic!("Unexpected or unsupported algorithm ID for KeyManagement slot: {:?}", algorithm_id),
        }
    }

    #[test]
    fn test_yubikey_device_authenticate_success() {
        let yubikey_serial = 32233649; // Standard test serial
        let pin = "123456".to_string();    // Standard test PIN
        let challenge = generate_yubikey_challenge().unwrap();

        let device_result = YubiKeyDevice::open(yubikey_serial);
        assert!(device_result.is_ok(), "Failed to open YubiKeyDevice for testing: {:?}", device_result.err());
        let mut device = device_result.unwrap();

        let signature_result = device.authenticate(pin, &challenge);
        
        assert!(signature_result.is_ok(), "YubiKeyDevice.authenticate failed: {:?}", signature_result.err());
        let signature_base64 = signature_result.unwrap();
        assert!(!signature_base64.is_empty(), "Signature from YubiKeyDevice.authenticate should not be empty");
    }

    #[test]
    fn test_yubikey_device_authenticate_incorrect_pin() {
        let yubikey_serial = 32233649;
        let incorrect_pin = "000000".to_string(); // An incorrect PIN
        let challenge = generate_yubikey_challenge().unwrap();

        let device_result = YubiKeyDevice::open(yubikey_serial);
        assert!(device_result.is_ok(), "Failed to open YubiKeyDevice for testing: {:?}", device_result.err());
        let mut device = device_result.unwrap();

        let signature_result = device.authenticate(incorrect_pin, &challenge);
        
        assert!(signature_result.is_err(), "YubiKeyDevice.authenticate should fail with an incorrect PIN");
        if let Err(Error::YubiKeyError(msg)) = signature_result {
            assert!(msg.starts_with("PIN verification failed:"), "Error message mismatch for incorrect PIN. Got: {}", msg);
        } else {
            panic!("Expected YubiKeyError for incorrect PIN, got {:?}", signature_result);
        }
    }

    #[test]
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
