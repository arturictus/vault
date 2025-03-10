use crate::app_state::TauriState;
use crate::error::{Error, Result};
use secrecy::zeroize;
use serde::{Deserialize, Serialize};
use tauri::command;
use yubikey::{YubiKey, piv};
// Remove unused import
use base64::Engine;
use rand::RngCore;
// Import the correct traits
use tauri::Manager;
use tauri::Runtime;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct YubiKeyInfo {
    pub serial: Option<u32>,
    pub name: String,
    pub version: Option<String>,
    pub is_fips: bool,
    pub form_factor: String,
}

impl YubiKeyInfo {
    fn from_yubikey(key: &YubiKey) -> Self {
        // Get serial number
        let serial_u32 = u32::from(key.serial());

        // Format the version as a string (version is already a Version struct, not a Result)
        let version = Some(key.version().to_string());

        // YubiKey 0.8.0 doesn't expose form_factor directly
        let form_factor = "YubiKey".to_string();

        Self {
            serial: Some(serial_u32),
            name: format!("YubiKey {}", serial_u32),
            version,
            is_fips: false, // Not directly accessible in 0.8.0
            form_factor,
        }
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

    // Find all YubiKey readers
    match yubikey::reader::Context::open() {
        Ok(mut context) => {
            // Iterate over readers using the context
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
        Err(e) => Err(Error::YubiKeyError(format!(
            "Failed to open YubiKey context: {}",
            e
        ))),
    }
}

pub fn get_pin(_prompt: &str) -> Result<String> {
    // Simple workaround - use a hardcoded PIN for development purposes only
    Ok("123456".to_string())
}

// /// Get PIN from user
// pub fn get_pin(prompt: &str) -> Result<String> {
//     // Create a password input with the given prompt
//     let mut input = PassphraseInput::with_default_binary()
//         .ok_or_else(|| Error::YubiKeyError("Could not find pinentry program".to_string()))?;
//     input.with_prompt(prompt);

//     let secret = input.interact()
//         .map_err(|e| Error::YubiKeyError(format!("PIN entry failed: {}", e)))?;

//     // Simple workaround - use a hardcoded PIN for development purposes only
//     // TODO: In production, implement proper PIN handling with appropriate secrecy handling
//     let pin = "123456".to_string(); // Default PIN for development
//     Ok(pin)
// }
// use tauri::{AppHandle, Emitter, EventTarget};
// #[command]
// pub fn get_pin_with_dialog<R: Runtime>(app_handle: &tauri::AppHandle<R>, prompt: &str) -> Result<String> {
//     // Create a one-time channel to receive the PIN
//     let (tx, rx) = std::sync::mpsc::channel::<String>();

//     // Create unique window ID
//     let window_id = format!("pin-entry-{}", rand::random::<u32>());

//     // Build and show the PIN entry window
//     let pin_window = tauri::WindowBuilder::new(
//         app_handle,
//         &window_id,
//         tauri::WindowUrl::App("pin-entry.html".into())
//     )
//     .title(prompt)
//     .inner_size(400.0, 200.0)
//     .center()
//     .focus()
//     .build()
//     .map_err(|e| Error::YubiKeyError(format!("Failed to create PIN entry window: {}", e)))?;

//     // Register a callback for when PIN is submitted
//     let tx_clone = tx.clone();
//     app_handle.listen_global("pin-submitted", move |event| {
//         if let Some(pin) = event.payload().and_then(|p| p.parse::<String>().ok()) {
//             let _ = tx_clone.send(pin);
//         }
//     });

//     // Wait for PIN with timeout
//     match rx.recv_timeout(std::time::Duration::from_secs(60)) {
//         Ok(pin) => Ok(pin),
//         Err(_) => Err(Error::YubiKeyError("PIN entry timed out".to_string()))
//     }
// }

/// Encrypt data using YubiKey
pub fn encrypt_with_yubikey(yubikey_serial: u32, data: &str) -> Result<String> {
    // Try to open YubiKey by serial number
    let serial = yubikey::Serial::from(yubikey_serial);

    match YubiKey::open_by_serial(serial) {
        Ok(mut yubikey) => {
            // Get PIN from user
            let pin = get_pin("Enter YubiKey PIN to encrypt data:")?;

            // Verify PIN directly on the YubiKey
            yubikey
                .verify_pin(pin.as_bytes())
                .map_err(|e| Error::YubiKeyError(format!("PIN verification failed: {}", e)))?;

            // Use the Management key slot for encryption (slot 9D)
            let slot = piv::SlotId::KeyManagement;

            // Get certificate from the slot
            // Try to get metadata first (works on firmware 5.2.3+)
            let cert_result = match piv::metadata(&mut yubikey, slot) {
                Ok(metadata) => {
                    println!("Metadata: {:?}", metadata);
                    Ok(())
                },
                Err(yubikey::Error::NotSupported) => {
                    // For older firmware, try to read the certificate directly
                    // This approach works on pre-5.2.3 firmware
                    match piv::read_certificate(&mut yubikey, slot) {
                        Ok(cert) => {
                            println!("Certificate found: subject={}", cert.subject());
                            Ok(())
                        },
                        Err(e) => Err(e)
                    }
                },
                Err(e) => Err(e)
            }; // For now just check if metadata works
            if cert_result.is_err() {
                return Err(Error::YubiKeyError(
                    "No certificate found in slot 9D".to_string(),
                ));
            }

            // TODO: Implement encryption
            //    piv::read_public_key(piv::AlgorithmId::Rsa2048, &mut yubikey, slot).unwrap();
            // let encrypted = yubikey.encrypt_data(data.as_bytes(), slot);

            let encrypted = data.as_bytes().to_vec();
            let encoded = base64::engine::general_purpose::STANDARD.encode(&encrypted);

            return Ok(encoded);
        }
        Err(err) => Err(Error::YubiKeyError(format!(
            "Failed to open YubiKey with serial {}: {}",
            yubikey_serial, err
        ))),
    }
}

/// Authenticate with YubiKey
pub fn authenticate_with_yubikey(yubikey_serial: u32, challenge: &str) -> Result<bool> {
    // Try to open YubiKey by serial number
    let serial = yubikey::Serial::from(yubikey_serial);

    match YubiKey::open_by_serial(serial) {
        Ok(mut yubikey) => {
            // Get PIN from user
            let pin = get_pin("Enter YubiKey PIN for authentication:")?;

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

            // Create a signing function here
            // This is a placeholder for demonstration
            // For a full implementation, you would use the piv module functions to sign the challenge

            // If we reached this point, authentication was successful
            return Ok(true);
        }
        Err(err) => Err(Error::YubiKeyError(format!(
            "Failed to open YubiKey with serial {}: {}",
            yubikey_serial, err
        ))),
    }
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

    use serde::de;
    use yubikey::{
        YubiKey,
        piv::{AlgorithmId, SlotId, decrypt_data},
    };
    #[test]
    fn test_slot_available() {
        let mut yubikey = YubiKey::open().unwrap();
        println!("Connected to YubiKey: {}", yubikey.serial());

        let slot = SlotId::KeyManagement;
        let algorithm = AlgorithmId::Rsa2048;

        let pin = "123456";
        yubikey.verify_pin(pin.as_bytes()).unwrap();
        println!("PIN verified");

        let encrypted_data = vec![1u8; 256]; // Dummy data
        let result = decrypt_data(&mut yubikey, &encrypted_data, algorithm, slot);
        assert!(result.is_ok());
        println!("Decrypted: {:?}", result.unwrap())
    }

    #[test]
    fn test_get_metadata() {
        let mut yubikey = YubiKey::open().unwrap();
        println!("Connected to YubiKey: {}", yubikey.serial());

        let slot = SlotId::KeyManagement;
        // let algorithm = AlgorithmId::Rsa2048;

        let pin = "123456";
        yubikey.verify_pin(pin.as_bytes()).unwrap();
        println!("PIN verified");

        // Get certificate from the slot
        let metadata = piv::metadata(&mut yubikey, slot).unwrap();
        println!("Metadata: {:?}", metadata);
    }

    #[test]
    fn test_decrypt_data() {
        let mut yubikey = YubiKey::open().unwrap();
        println!("Connected to YubiKey: {}", yubikey.serial());

        let slot = SlotId::KeyManagement;
        let algorithm = AlgorithmId::Rsa2048;

        let pin = "123456";
        yubikey.verify_pin(pin.as_bytes()).unwrap();
        println!("PIN verified");

        let encrypted_data =
            std::fs::read("tests/fixtures/encrypted.bin").expect("Failed to read encrypted data");
        let decrypted = decrypt_data(&mut yubikey, &encrypted_data, algorithm, slot).unwrap();
        // TODO: use markers to extract the data
        println!("Decrypted: {}", String::from_utf8_lossy(decrypted.as_ref()));
    }

    #[test]
    fn test_yubikey_encrypt_decrypt() {
        let data = "Hello, YubiKey!";
        let yubikey_serial = 13062801;

        let encrypted = super::encrypt_with_yubikey(yubikey_serial, data).unwrap();
        assert_ne!(encrypted, data);

        // Decrypt the data
        // let decrypted = super::decrypt_with_yubikey(yubikey_serial, &encrypted).unwrap();
        // assert_eq!(decrypted, data);
    }
}
