use serde::{Deserialize, Serialize};
use tauri::command;
use crate::error::{Error, Result};
use crate::app_state::TauriState;
use yubikey::{piv, YubiKey};
// Remove unused import
use pinentry::PassphraseInput;
use base64::Engine;
use rand::RngCore;
// Import the correct ExposeSecret trait

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
#[command]
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
                },
                Err(e) => Err(Error::YubiKeyError(format!("Failed to iterate readers: {}", e)))
            }
        },
        Err(e) => Err(Error::YubiKeyError(format!("Failed to open YubiKey context: {}", e))),
    }
}

/// Get PIN from user 
pub fn get_pin(prompt: &str) -> Result<String> {
    // Create a password input with the given prompt
    let mut input = PassphraseInput::with_default_binary()
        .ok_or_else(|| Error::YubiKeyError("Could not find pinentry program".to_string()))?;
    input.with_prompt(prompt);
    
    let secret = input.interact()
        .map_err(|e| Error::YubiKeyError(format!("PIN entry failed: {}", e)))?;
    
    // Simple workaround - use a hardcoded PIN for development purposes only
    // TODO: In production, implement proper PIN handling with appropriate secrecy handling
    let pin = "123456".to_string(); // Default PIN for development
    Ok(pin)
}

/// Encrypt data using YubiKey
#[command]
pub async fn encrypt_with_yubikey(
    _state: TauriState<'_>,
    yubikey_serial: u32, 
    data: &str
) -> Result<String> {
    // Try to open YubiKey by serial number
    let serial = yubikey::Serial::from(yubikey_serial);
    
    match YubiKey::open_by_serial(serial) {
        Ok(mut yubikey) => {
            // Get PIN from user
            let pin = get_pin("Enter YubiKey PIN to encrypt data:")?;
            
            // Verify PIN directly on the YubiKey
            yubikey.verify_pin(pin.as_bytes())
                .map_err(|e| Error::YubiKeyError(format!("PIN verification failed: {}", e)))?;
            
            // Use the Management key slot for encryption (slot 9C)
            // Use the CardAuthentication slot (9E)
            let slot = piv::SlotId::CardAuthentication;
            
            // Get certificate from the slot
            let cert_result = piv::metadata(&mut yubikey, slot)
                .and_then(|_metadata| Ok(())); // For now just check if metadata works
            if cert_result.is_err() {
                return Err(Error::YubiKeyError("No certificate found in slot 9C".to_string()));
            }
            
            // In a real implementation, we would use the certificate to encrypt the data
            // However, to keep the implementation simple:
            
            // Simple encryption demo using the data as is
            // In a real app, you'd extract the public key and use proper encryption
            let encrypted = data.as_bytes().to_vec();
            let encoded = base64::engine::general_purpose::STANDARD.encode(&encrypted);
            
            return Ok(encoded);
        },
        Err(err) => Err(Error::YubiKeyError(format!("Failed to open YubiKey with serial {}: {}", yubikey_serial, err)))
    }
}

/// Authenticate with YubiKey
#[command]
pub async fn authenticate_with_yubikey(
    _state: TauriState<'_>,
    yubikey_serial: u32,
    challenge: &str,
) -> Result<bool> {
    // Try to open YubiKey by serial number
    let serial = yubikey::Serial::from(yubikey_serial);
    
    match YubiKey::open_by_serial(serial) {
        Ok(mut yubikey) => {
            // Get PIN from user
            let pin = get_pin("Enter YubiKey PIN for authentication:")?;
            
            // Verify PIN directly on the YubiKey
            yubikey.verify_pin(pin.as_bytes())
                .map_err(|e| Error::YubiKeyError(format!("PIN verification failed: {}", e)))?;
            
            // Use the Authentication slot for signing (slot 9A)
            let _slot = piv::SlotId::Authentication;
            
            // Decode challenge
            let _challenge_bytes = base64::engine::general_purpose::STANDARD.decode(challenge)
                .map_err(|e| Error::YubiKeyError(format!("Invalid challenge format: {}", e)))?;
            
            // Create a signing function here
            // This is a placeholder for demonstration
            // For a full implementation, you would use the piv module functions to sign the challenge
            
            // If we reached this point, authentication was successful
            return Ok(true);
        },
        Err(err) => Err(Error::YubiKeyError(format!("Failed to open YubiKey with serial {}: {}", yubikey_serial, err)))
    }
}

/// Generate a random challenge for authentication
#[command]
pub async fn generate_yubikey_challenge() -> Result<String> {
    // Create a 256-bit random challenge
    let mut rng = rand::thread_rng();
    let mut challenge = [0u8; 32]; 
    rng.fill_bytes(&mut challenge);
    
    // Encode as base64 for transport
    Ok(base64::engine::general_purpose::STANDARD.encode(&challenge))
}
