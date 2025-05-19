// #![allow(dead_code)]
// use crate::error::{Error, Result};

// use rsa::pkcs8::der::oid::db::rfc4519::O;
// use secrecy::zeroize;
// use serde::{Deserialize, Serialize};
// use yubikey::{YubiKey, piv};
// // Remove unused import
// use base64::Engine;
// use rand::RngCore;
// // Import the correct traits
// use yubikey::{
//     MgmKey, PinPolicy, Serial, TouchPolicy,
//     certificate::{Certificate, yubikey_signer},
//     piv::{AlgorithmId, Key, ManagementSlotId, RetiredSlotId, SlotId},
// };

// pub fn generate_key(serial: u32, pin: String) -> Result<()> {
//     let serial = yubikey::Serial::from(serial);
//     let mut yubikey = YubiKey::open_by_serial(serial)
//         .map_err(|e| Error::YubiKeyError(format!("Failed to open YubiKey: {}", e)))?;
//     let slot = piv::SlotId::KeyManagement;
//     let new_key = piv::generate(
//         &mut yubikey,
//         slot,
//         AlgorithmId::Rsa2048,
//         yubikey::PinPolicy::Once,
//         yubikey::TouchPolicy::Never,
//     )?;
//     println!("Generated new key: {:?}", new_key);
//     Ok(())
// }

// /// Sets up a YubiKey with PIV functionality
// /// This function:
// /// 1. Opens the YubiKey
// /// 2. Verifies the PIN
// /// 3. Generates a new key in the Management slot
// /// 4. Sets up proper PIN and touch policies
// pub fn setup_yubikey(serial: u32, pin: String) -> Result<()> {
//     // Open YubiKey by serial number
//     let serial = yubikey::Serial::from(serial);
//     let mut yubikey = YubiKey::open_by_serial(serial)
//         .map_err(|e| Error::YubiKeyError(format!("Failed to open YubiKey: {}", e)))?;

//     // Verify PIN
//     yubikey.verify_pin(pin.as_bytes())
//         .map_err(|e| Error::YubiKeyError(format!("PIN verification failed: {}", e)))?;

//     // Use the Management key slot (9D)
//     let slot = piv::SlotId::KeyManagement;

//     // First, try to get metadata to check if key exists
//     match yubikey::certificate::read_certificate(&mut yubikey, slot) {
//         Ok(cert) => {
//             println!("Existing certificate found: {:?}", cert);
//         }
//         Err(yubikey::Error::NotSupported) => {
//             // For older firmware, try to read the certificate directly
//             match piv::read_certificate(&mut yubikey, slot) {
//                 Ok(cert) => {
//                     println!("Existing certificate found: subject={}", cert.subject());
//                 }
//                 Err(_) => {
//                     // No existing key, generate a new one
//                     println!("No existing key found, generating new key...");
//                     let new_key = piv::generate(
//                         &mut yubikey,
//                         slot,
//                         AlgorithmId::Rsa2048,
//                         PinPolicy::Once,  // Require PIN once per session
//                         TouchPolicy::Never,  // Don't require touch
//                     )?;
//                     println!("Generated new key: {:?}", new_key);
//                 }
//             }
//         }
//         Err(e) => {
//             return Err(Error::YubiKeyError(format!("Failed to check key status: {}", e)));
//         }
//     }

//     // Set up PIN policy for the slot
//     piv::set_pin_policy(
//         &mut yubikey,
//         slot,
//         PinPolicy::Once,  // Require PIN once per session
//     )?;

//     // Set up touch policy for the slot
//     piv::set_touch_policy(
//         &mut yubikey,
//         slot,
//         TouchPolicy::Never,  // Don't require touch
//     )?;

//     // Verify the setup by trying to read the public key
//     match piv::read_public_key(AlgorithmId::Rsa2048, &mut yubikey, slot) {
//         Ok(pub_key) => {
//             println!("Successfully read public key from slot");
//             println!("Public key: {:?}", pub_key);
//         }
//         Err(e) => {
//             return Err(Error::YubiKeyError(format!("Failed to verify setup: {}", e)));
//         }
//     }

//     Ok(())
// }

// /// Tests the YubiKey setup by performing a simple encryption/decryption operation
// pub fn test_yubikey_setup(serial: u32, pin: String) -> Result<()> {
//     let serial = yubikey::Serial::from(serial);
//     let mut yubikey = YubiKey::open_by_serial(serial)
//         .map_err(|e| Error::YubiKeyError(format!("Failed to open YubiKey: {}", e)))?;

//     // Verify PIN
//     yubikey.verify_pin(pin.as_bytes())
//         .map_err(|e| Error::YubiKeyError(format!("PIN verification failed: {}", e)))?;

//     let slot = piv::SlotId::KeyManagement;
//     let test_data = b"Hello, YubiKey!";

//     // Try to encrypt some test data
//     let encrypted = piv::encrypt_data(
//         &mut yubikey,
//         test_data,
//         AlgorithmId::Rsa2048,
//         slot,
//     )?;

//     println!("Successfully encrypted test data");

//     // Try to decrypt the data
//     let decrypted = piv::decrypt_data(
//         &mut yubikey,
//         &encrypted,
//         AlgorithmId::Rsa2048,
//         slot,
//     )?;

//     println!("Successfully decrypted test data");
//     println!("Decrypted text: {}", String::from_utf8_lossy(&decrypted));

//     Ok(())
// }

// /// Encrypt data using YubiKey
// /// TODO: Unable to get the public key from the YubiKey
// /// public key should be provided by user or generated by the YubiKey
// pub fn encrypt_with_yubikey(yubikey_serial: u32, pin: String, data: &str) -> Result<String> {
//     // Try to open YubiKey by serial number
//     let serial = yubikey::Serial::from(yubikey_serial);
//     let mut yubikey = YubiKey::open_by_serial(serial)
//         .map_err(|e| Error::YubiKeyError(format!("Failed to open YubiKey: {}", e)))?;

//     // Verify PIN directly on the YubiKey
//     yubikey
//         .verify_pin(pin.as_bytes())
//         .map_err(|e| Error::YubiKeyError(format!("PIN verification failed: {}", e)))?;

//     // Use the Management key slot for encryption (slot 9D)
//     let slot = piv::SlotId::KeyManagement;

//     // Get certificate from the slot
//     // Try to get metadata first (works on firmware 5.2.3+)
//     let cert_result = match yubikey.read_cert(slot) {
//         Ok(cert) => {
//             println!("Metadata: {:?}", cert);
//             Ok(())
//         }
//         Err(yubikey::Error::NotSupported) => {
//             // For older firmware, try to read the certificate directly
//             // This approach works on pre-5.2.3 firmware
//             // match piv::read_certificate(&mut yubikey, slot) {
//             //     Ok(cert) => {
//             //         println!("Certificate found: subject={}", cert.subject());
//             //         Ok(())
//             //     },
//             //     Err(e) => Err(e)
//             // }
//             Err(yubikey::Error::NotSupported)
//         }
//         Err(e) => Err(e),
//     }; // For now just check if metadata works
//     if cert_result.is_err() {
//         return Err(Error::YubiKeyError(
//             "No certificate found in slot 9D".to_string(),
//         ));
//     }

//     // TODO: Implement encryption
//     //    piv::read_public_key(piv::AlgorithmId::Rsa2048, &mut yubikey, slot).unwrap();
//     // let encrypted = yubikey.encrypt_data(data.as_bytes(), slot);

//     let encrypted = data.as_bytes().to_vec();
//     let encoded = base64::engine::general_purpose::STANDARD.encode(&encrypted);

//     return Ok(encoded);
// }

// #[cfg(test)]
// mod test {
//     use super::*;
//     use yubikey::{
//         YubiKey,
//         piv::{AlgorithmId, SlotId, decrypt_data},
//     };

//     #[test]
//     fn test_setup_yubikey() {
//         let serial = 32233649;
//         let pin = "123456".to_string();
//         let result = setup_yubikey(serial, pin);
//         println!("{:?}", result);
//         assert!(result.is_ok());
//     }

//     #[test]
//     fn test_get_metadata() {
//         let mut yubikey = YubiKey::open().unwrap();

//         let slot = SlotId::KeyManagement;
//         // let algorithm = AlgorithmId::Rsa2048;

//         let pin = "123456";
//         yubikey.verify_pin(pin.as_bytes()).unwrap();

//         // Get certificate from the slot
//         yubikey.read_cert(slot).unwrap();
//     }

//     #[test]
//     fn test_yubikey_encryption() {
//         let serial = 32233649;  // Replace with your YubiKey's serial number
//         let pin = "123456";     // Replace with your YubiKey's PIN
//         let result = test_yubikey_setup(serial, pin.to_string());
//         assert!(result.is_ok());
//     }
// }
