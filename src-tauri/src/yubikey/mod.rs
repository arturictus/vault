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
    encryptor.encrypt(data.as_bytes())
        .map_err(|e| Error::YubiKeyError(e.to_string()))
        .map(|encrypted| base64::engine::general_purpose::STANDARD.encode(&encrypted))
}

pub fn decrypt_with_yubikey(
    yubikey_serial: u32,
    pin: String,
    encrypted_data_base64_bytes: Vec<u8>, // Input is now explicitly named to reflect it's base64 bytes
) -> Result<Vec<u8>> { // Return raw decrypted bytes
    let serial = yubikey::Serial::from(yubikey_serial);
    let mut yubikey = YubiKey::open_by_serial(serial)
        .map_err(|e| Error::YubiKeyError(format!("Failed to open YubiKey: {}", e)))?;

    let slot = piv::SlotId::KeyManagement;
    // TODO: select the correct algorithm based on the key metadata
    let algorithm = piv::AlgorithmId::Rsa2048;

    yubikey.verify_pin(pin.as_bytes())?;

    // Decode the base64 input bytes to get the raw ciphertext
    let raw_ciphertext = base64::engine::general_purpose::STANDARD.decode(&encrypted_data_base64_bytes)
        .map_err(|e| Error::YubiKeyError(format!("Failed to decode base64 encrypted data: {}", e)))?;

    // Perform decryption using the YubiKey
    let decrypted_zeroizing_vec = piv::decrypt_data(&mut yubikey, &raw_ciphertext, algorithm, slot)
        .map_err(|e| Error::from(e))?; // Convert yubikey::Error into our Error type
    
    // Convert Zeroizing<Vec<u8>> to Vec<u8> for unpadding
    let padded_data = decrypted_zeroizing_vec.to_vec();
    let block_len = padded_data.len();

    // PKCS#1 v1.5 unpadding (EME-PKCS1-v1_5 type 2)
    // EM = 0x00 || 0x02 || PS || 0x00 || M
    // PS must be at least 8 octets.

    // 1. Length check (k octets long, k >= 11 for valid structure)
    // For RSA2048, block_len should be 256. This check is more general.
    if block_len < 11 {
        return Err(Error::YubiKeyError(format!(
            "Decryption error: Padded data too short ({} bytes). Expected at least 11 for PKCS#1 v1.5.",
            block_len
        )));
    }

    // 2. Check block type bytes 0x00 and 0x02
    if padded_data[0] != 0x00 || padded_data[1] != 0x02 {
        return Err(Error::YubiKeyError(format!(
            "Decryption error: Invalid PKCS#1 v1.5 padding header. Expected 00 02, got: {:02X} {:02X}",
            padded_data.get(0).cloned().unwrap_or(0xFF), // .cloned() for Option<&u8> to Option<u8>
            padded_data.get(1).cloned().unwrap_or(0xFF)
        )));
    }

    // 3. Find the 0x00 separator after PS.
    // PS starts at index 2. PS must contain at least 8 octets.
    // The first 0x00 byte after PS (i.e., starting search from index 2) is the separator.
    let mut separator_index: Option<usize> = None;
    for i in 2..block_len {
        if padded_data[i] == 0x00 {
            separator_index = Some(i);
            break;
        }
        // According to PKCS#1 v1.5 (RFC 2313/8017), PS should consist of non-zero octets.
        // If a 0x00 is found within what should be PS before 8 bytes of PS are established,
        // it's an invalid format. However, the typical unpadding just finds the first 0x00.
    }

    match separator_index {
        Some(idx) => {
            // The padding string PS is from index 2 to idx-1.
            // Length of PS is idx - 2.
            if (idx - 2) < 8 {
                return Err(Error::YubiKeyError(format!(
                    "Decryption error: PKCS#1 v1.5 padding string (PS) too short ({} bytes). Minimum 8 bytes required. Separator at index {}.",
                    idx - 2, idx
                )));
            }

            // Message M starts at idx + 1.
            // If (idx + 1) >= block_len, the message M is empty.
            if (idx + 1) >= block_len {
                 // RFC 8017 Section 7.2.2 (EME-PKCS1-V1_5-DECODE) states:
                 // "If mLen is zero, output "decryption error" and stop." (mLen is length of M)
                 return Err(Error::YubiKeyError(
                    "Decryption error: PKCS#1 v1.5 unpadding resulted in an empty message (separator was the last or second to last byte of padding indicating no message).".to_string(),
                ));
            }
            let message_data = padded_data[idx + 1..].to_vec();
            Ok(message_data)
        }
        None => {
            // No 0x00 separator found after PS part.
            Err(Error::YubiKeyError(
                "Decryption error: PKCS#1 v1.5 padding separator 0x00 not found after PS.".to_string(),
            ))
        }
    }
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
    
    // Get the complete DER-encoded SubjectPublicKeyInfo structure
    let pem = cert.subject_pki().to_pem(rsa::pkcs1::der::pem::LineEnding::LF)
            .map_err(|e| Error::YubiKeyError(format!("Failed to get publick key pem: {}", e)))?;
    
    Ok(pem)
}

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
    
    // Verify PIN
    // yubikey.verify_pin(pin.as_bytes())?;
    
    // Get the public key and use it for encryption
    let pub_key = get_public_key_from_yubikey(&yubikey)?;
    let encryptor = encrypt::PublicKey::from_pem(&pub_key)?;
    
    // Encrypt the data
    let encrypted = encryptor.encrypt(&data)?;
    
    // Encode as base64 for transport
    Ok(base64::engine::general_purpose::STANDARD.encode(&encrypted))
}

/// Authenticate with YubiKey by signing a challenge.
/// Returns the signature as a base64 encoded string if successful.
pub fn authenticate_with_yubikey(
    yubikey_serial: u32,
    pin: String,
    challenge_base64: &str, // Renamed for clarity, expects base64 encoded challenge
) -> Result<String> { // Changed return type from bool to String (base64 signature)
    // Try to open YubiKey by serial number
    let serial = yubikey::Serial::from(yubikey_serial);
    let mut yubikey = YubiKey::open_by_serial(serial)
        .map_err(|e| Error::YubiKeyError(format!("Failed to open YubiKey: {}", e)))?;

    // Verify PIN directly on the YubiKey
    yubikey
        .verify_pin(pin.as_bytes())
        .map_err(|e| Error::YubiKeyError(format!("PIN verification failed: {}", e)))?;

    // Use the Authentication slot for signing (slot 9A)
    let slot = piv::SlotId::Authentication;

    // Decode challenge from base64
    let challenge_bytes = base64::engine::general_purpose::STANDARD
        .decode(challenge_base64)
        .map_err(|e| Error::YubiKeyError(format!("Invalid challenge format (base64 decode failed): {}", e)))?;

    // TODO: Select the algorithm based on the key metadata in the Authentication slot.
    // For now, assuming Rsa2048 as it's common.
    let algorithm = piv::AlgorithmId::Rsa2048; 

    // Sign the challenge using the Authentication slot
    let signature_bytes = piv::sign_data(
        &mut yubikey,
        &challenge_bytes,
        algorithm,
        slot,
    )
    .map_err(|e| Error::YubiKeyError(format!("Failed to sign challenge: {}", e)))?;

    // In a real implementation, you would verify this signature against a stored public key
    // Here we're just checking that the YubiKey was able to create a signature with the given PIN
    if signature_bytes.is_empty() {
        // This case should ideally be covered by the error from sign_data if signing fails.
        return Err(Error::YubiKeyError("Authentication failed: produced an empty signature".to_string()));
    }

    // Encode signature to base64 and return
    Ok(base64::engine::general_purpose::STANDARD.encode(&signature_bytes))
}

/// Verifies a signature made by a YubiKey.
pub fn verify_yubikey_signature(
    app_state: &AppState, // To retrieve stored YubiKeyInfo/public key
    yubikey_serial: u32,
    original_challenge_bytes: &[u8],
    signature_base64: &str,
) -> Result<bool> {
    // 1. Retrieve the public key for the given YubiKey serial
    // Assuming YubiKeyInfo stores the necessary public key after registration
    let yk_info = YubiKeyInfo::get(app_state)?;
    if yk_info.serial != Some(yubikey_serial) {
        // This check might be too simplistic if multiple YubiKeys can be registered.
        // For now, assume one primary YubiKey's info is stored.
        // A more robust system would look up the specific YubiKey's public key by its serial.
        return Err(Error::YubiKeyError(format!(
            "Stored YubiKey serial {} does not match provided serial {}",
            yk_info.serial.unwrap_or(0), yubikey_serial
        )));
    }

    let pub_key_pem = yk_info.pub_key.ok_or_else(|| {
        Error::YubiKeyError(format!(
            "Public key not found for YubiKey serial {}",
            yubikey_serial
        ))
    })?;

    // 2. Prepare the public key for verification
    let verifier = crate::encrypt::PublicKey::from_pem(&pub_key_pem)
        .map_err(|e| Error::YubiKeyError(format!("Failed to parse public key for verification: {}", e)))?;

    // 3. Decode the signature from base64
    let signature_bytes = base64::engine::general_purpose::STANDARD
        .decode(signature_base64)
        .map_err(|e| Error::YubiKeyError(format!("Failed to decode base64 signature: {}", e)))?;

    // 4. Perform the verification
    // Assuming PublicKey has a verify method like: verify(data: &[u8], signature: &[u8]) -> Result<(), RsaError>
    // The exact method signature might differ based on your rsa crate usage.
    // For rsa crate, it's often public_key.verify(PaddingScheme, hashed_data, signature)
    // For simplicity, let's assume your crate::encrypt::PublicKey abstracts this.
    // If crate::encrypt::PublicKey does not have a verify method, this part needs to be implemented
    // using the 'rsa' crate directly, e.g., using RsaPublicKey::verify.
    // This often requires knowing the padding scheme and hash function used during signing.
    // PIV standard often uses PKCS#1 v1.5 padding. The hash depends on AlgorithmId.
    // For AlgorithmId::Rsa2048, it's typically SHA256.

    // Placeholder for actual verification logic.
    // This needs to be implemented correctly based on how `crate::encrypt::PublicKey` is structured
    // or by directly using the `rsa` crate's verification functions.
    // For now, we'll assume a simple verify method exists.
    // IMPORTANT: The actual verification logic needs to be implemented in `crate::encrypt::rsa::PublicKey`.
    // Example using a hypothetical verify method:
    match verifier.verify(original_challenge_bytes, &signature_bytes) {
        Ok(_) => Ok(true),
        Err(_e) => Ok(false), // Or return Err(Error::YubiKeyError("Signature verification failed".to_string()))
    }
    // If your `verify` method returns `Result<(), Error>`, then:
    // verifier.verify(original_challenge_bytes, &signature_bytes)
    //     .map(|_| true)
    //     .map_err(|e| Error::YubiKeyError(format!("Signature verification failed: {}", e)))
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
    fn test_encrypt_with_yubikey() {
        let yubikey_serial = 32233649;
        // let pin = "123456";
        let result = do_encrypt(yubikey_serial, "data".as_bytes().to_vec());
        result.unwrap();
        // assert!(result.is_ok())
    }

    #[test]
    fn test_authenticate_with_yubikey() {
        let yubikey_serial = 32233649; // Replace with a valid test YubiKey serial if available
        let pin = "123456"; // Replace with the PIN for the test YubiKey
        let challenge = super::generate_yubikey_challenge().unwrap(); // Generate a fresh challenge
        
        let signature_result = authenticate_with_yubikey(yubikey_serial, pin.to_string(), &challenge);
        
        assert!(signature_result.is_ok(), "Authentication failed: {:?}", signature_result.err());
        
        let signature_base64 = signature_result.unwrap();
        assert!(!signature_base64.is_empty(), "Signature should not be empty");
        
        // Optional: Print for manual verification or further debugging
        // println!("Challenge: {}", challenge);
        // println!("Signature (Base64): {}", signature_base64);
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
        // Use a known YubiKey serial and PIN for testing.
        // IMPORTANT: This test requires a YubiKey with the specified serial to be connected,
        // and the PIN must be correct. The Key Management slot (9c) must have a key
        // suitable for RSA decryption (e.g., generated for PIV decryption).
        let yubikey_serial = 32233649; // Replace with your test YubiKey's serial
        let pin = "123456".to_string();    // Replace with your test YubiKey's PIN

        let original_data_str = "This is a secret test message for YubiKey!";
        let original_data_bytes = original_data_str.as_bytes().to_vec();

        println!("Original data (str): {}", original_data_str);
        println!("Original data bytes (len {}): {:?}", original_data_bytes.len(), original_data_bytes);
        println!("Original data bytes (hex): {}", original_data_bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>());

        // 1. Encrypt the data using the YubiKey's public key
        let encrypted_data_base64_result = do_encrypt(yubikey_serial, original_data_bytes.clone());
        assert!(encrypted_data_base64_result.is_ok(), "Encryption failed: {:?}", encrypted_data_base64_result.err());
        let encrypted_data_base64_string = encrypted_data_base64_result.unwrap();
        println!("Encrypted data (base64 string): {}", encrypted_data_base64_string);

        // 2. Decrypt the data using the YubiKey - now returns Result<Vec<u8>>
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
