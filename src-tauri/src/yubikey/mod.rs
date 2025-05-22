mod experimental_setup;
mod standalone;
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
    fn from_yubikey(key: &YubiKey) -> Self {
        // Get serial number
        let serial_u32 = u32::from(key.serial());

        // Format the version as a string (version is already a Version struct, not a Result)
        let version = Some(key.version().to_string());

        // YubiKey 0.8.0 doesn't expose form_factor directly
        let form_factor = "YubiKey".to_string();

        // Try to get the public key
        let pub_key = get_public_key_from_yubikey(key).ok();

        Self {
            serial: Some(serial_u32),
            name: format!("YubiKey {}", serial_u32),
            version,
            is_fips: false, // Not directly accessible in 0.8.0
            form_factor,
            pub_key
        }
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
                if let Ok(yubikey) = reader.open() {
                    keys.push(YubiKeyInfo::from_yubikey(&yubikey));
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
    encryptor.encrypt(data.as_bytes()).map_err(|e| Error::YubiKeyError(e.to_string()))
}

pub fn decrypt_with_yubikey(
    yubikey_serial: u32,
    pin: String,
    encrypted_data: Vec<u8>,
) -> Result<String> {
    let serial = yubikey::Serial::from(yubikey_serial);
    let mut yubikey = YubiKey::open_by_serial(serial)
        .map_err(|e| Error::YubiKeyError(format!("Failed to open YubiKey: {}", e)))?;

    let slot = piv::SlotId::KeyManagement;
    // TODO: select the correct algorithm based on the key metadata
    let algorithm = piv::AlgorithmId::Rsa2048;

    yubikey.verify_pin(pin.as_bytes())?;

    let encrypted_data = base64::engine::general_purpose::STANDARD.decode(&encrypted_data).map_err(|e| Error::YubiKeyError(e.to_string()))?;

    let decrypted = piv::decrypt_data(&mut yubikey, &encrypted_data, algorithm, slot)?;
    // Convert the decrypted data to a string
    String::from_utf8(decrypted.to_vec())
        .map_err(|e| Error::YubiKeyError(format!("Invalid UTF-8 in decrypted data: {}", e)))
}

pub fn get_public_key_from_yubikey(yubikey: &YubiKey) -> Result<String> {
    use rsa::pkcs1::der::EncodePem;
    // Create a new instance of YubiKey since we need a mutable reference
    let mut yubikey_mut = YubiKey::open_by_serial(yubikey.serial())
        .map_err(|e| Error::YubiKeyError(format!("Failed to open YubiKey: {}", e)))?;
    
    // Use the Key Management slot for public key
    let slot = piv::SlotId::KeyManagement;
    
    // Get the certificate from the slot
    let cert = yubikey::certificate::Certificate::read(&mut yubikey_mut, slot)
        .map_err(|e| Error::YubiKeyError(format!("Failed to get certificate: {}", e)))?;
    
    // Get the PEM from the certificate
    let pem = cert.subject_pki().to_pem(rsa::pkcs1::der::pem::LineEnding::LF)
            .map_err(|e| Error::YubiKeyError(format!("Failed to get publick key pem: {}", e)))?;
    
    Ok(pem)
}

// TODO: use this https://github.com/iqlusioninc/yubikey.rs/blob/main/tests/integration.rs#L332
// pub fn get_algorithm_from_yubikey(yubikey: &YubiKey) -> Result<piv::AlgorithmId> {
//     // Create a new instance of YubiKey since we need a mutable reference
//     let mut yubikey_mut = YubiKey::open_by_serial(yubikey.serial())
//         .map_err(|e| Error::YubiKeyError(format!("Failed to open YubiKey: {}", e)))?;
    
//     // Use the Key Management slot for public key
//     let slot = piv::SlotId::KeyManagement;
    
//     // Get the certificate from the slot
//     let cert = yubikey::certificate::Certificate::read(&mut yubikey_mut, slot)
//         .map_err(|e| Error::YubiKeyError(format!("Failed to get certificate: {}", e)))?;
    
//     // Get the algorithm from the certificate's public key
//     let algorithm = match cert.subject_pki() {
//         yubikey::piv::key::PublicKey::Rsa(rsa) => {
//             let key_size = rsa.bits() as usize;
//             match key_size {
//                 1024 => piv::AlgorithmId::Rsa1024,
//                 2048 => piv::AlgorithmId::Rsa2048,
//                 _ => return Err(Error::YubiKeyError(format!("Unsupported RSA key size: {}", key_size)))
//             }
//         },
//         yubikey::piv::key::PublicKey::EccP256(_) => piv::AlgorithmId::EccP256,
//         yubikey::piv::key::PublicKey::EccP384(_) => piv::AlgorithmId::EccP384,
//         _ => return Err(Error::YubiKeyError("Unsupported key type".to_string())),
//     };
    
//     Ok(algorithm)
// }

/// Get the public key from a YubiKey by serial number
pub fn get_public_key(yubikey_serial: u32) -> Result<String> {
    let serial = yubikey::Serial::from(yubikey_serial);
    let yubikey = YubiKey::open_by_serial(serial)
        .map_err(|e| Error::YubiKeyError(format!("Failed to open YubiKey: {}", e)))?;
    
    get_public_key_from_yubikey(&yubikey)
}

pub fn do_encrypt(yubikey_serial: u32,
    data: Vec<u8>) -> Result<String> {
    let serial = yubikey::Serial::from(yubikey_serial);
    let yubikey = YubiKey::open_by_serial(serial)
        .map_err(|e| Error::YubiKeyError(format!("Failed to open YubiKey: {}", e)))?;
        
    // Get the public key and use it for encryption
    let pub_key = get_public_key_from_yubikey(&yubikey)?;
    let encryptor = encrypt::PublicKey::from_pem(&pub_key)?;
    
    // Encrypt the data
    let encrypted = encryptor.encrypt(&data)?;
    
    // Encode as base64 for transport
    Ok(base64::engine::general_purpose::STANDARD.encode(&encrypted))
}

/// Authenticate with YubiKey
pub fn authenticate_with_yubikey(
    yubikey_serial: u32,
    pin: String,
    challenge: &str,
) -> Result<bool> {
    // Try to open YubiKey by serial number
    let serial = yubikey::Serial::from(yubikey_serial);
    let mut yubikey = YubiKey::open_by_serial(serial)
        .map_err(|e| Error::YubiKeyError(format!("Failed to open YubiKey: {}", e)))?;

    // Verify PIN directly on the YubiKey
    yubikey
        .verify_pin(pin.as_bytes())
        .map_err(|e| Error::YubiKeyError(format!("PIN verification failed: {}", e)))?;

    // Use the Authentication slot for signing (slot 9A)
    let _slot = piv::SlotId::Authentication;

    // Decode challenge
    let _challenge_bytes = base64::engine::general_purpose::STANDARD
        .decode(challenge)
        .map_err(|e| Error::YubiKeyError(format!("Invalid challenge format: {}", e)))?;

    // Sign the challenge using the Authentication slot
    let signature = piv::sign_data(
        &mut yubikey,
        &_challenge_bytes,
        piv::AlgorithmId::Rsa2048, // Use appropriate algorithm based on your key
        _slot
    )
    .map_err(|e| Error::YubiKeyError(format!("Failed to sign challenge: {}", e)))?;

    // In a real implementation, you would verify this signature against a stored public key
    // Here we're just checking that the YubiKey was able to create a signature with the given PIN
    if signature.is_empty() {
        return Err(Error::YubiKeyError("Authentication failed: empty signature".to_string()));
    }

    // If we reached this point, authentication was successful
    return Ok(true);
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
    use std::fs;
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
    fn test_encrypt_with_yubikey() {
        let yubikey_serial = 32233649;
        let result = do_encrypt(yubikey_serial, "data".as_bytes().to_vec());
        assert!(result.is_ok());
        fs::write("tests/fixtures/encrypted.bin", result.unwrap()).unwrap();
    }
    // #[test]
    // fn test_get_algorithm_from_yubikey() {
    //     let yubikey_serial = 32233649;
    //     let result = get_algorithm_from_yubikey(yubikey_serial);
    //     assert!(result.is_ok())
    // }

    #[test]
    fn test_authenticate_with_yubikey() {
        let yubikey_serial = 32233649;
        let pin = "123456";
        let challenge = "SGVsbG8sIFl1YmlLZXkh";
        authenticate_with_yubikey(yubikey_serial, pin.to_string(), challenge).unwrap();
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
        let encrypted_data =
            std::fs::read("tests/fixtures/encrypted.bin").expect("Failed to read encrypted data");
        let decrypted = decrypt_with_yubikey(13062801, "123456".to_string(), encrypted_data);
        assert!(decrypted.is_ok());
        // assert_eq!(decrypted, "This is a test message");
    }

    // #[test]
    // fn test_yubikey_encrypt_decrypt() {
    //     let data = "Hello, YubiKey!";
    //     let yubikey_serial = 13062801;

    //     let encrypted = super::encrypt_with_yubikey(yubikey_serial, data).unwrap();
    //     assert_ne!(encrypted, data);

    //     // Decrypt the data
    //     // let decrypted = super::decrypt_with_yubikey(yubikey_serial, &encrypted).unwrap();
    //     // assert_eq!(decrypted, data);
    // }
}
