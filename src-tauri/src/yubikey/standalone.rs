// yubikey.rs
// Rust module for YubiKey cryptographic operations

use pkcs11::Ctx;
use pkcs11::types::{
    CKA_CLASS, CKA_ID, CKA_LABEL, CKF_RW_SESSION, CKF_SERIAL_SESSION,
    CKO_PRIVATE_KEY, CKU_USER, CKM_RSA_PKCS, CK_OBJECT_HANDLE, CK_SESSION_HANDLE,
    CK_ATTRIBUTE, CK_ULONG, CK_BYTE, CK_BBOOL, CK_MECHANISM, CK_VOID_PTR,
};
use std::path::Path;
use std::error::Error;
use std::fmt;
use std::fs;

/// Custom error type for YubiKey operations
#[derive(Debug)]
pub enum YubiKeyError {
    NotFound,
    Pkcs11Error(String),
    IoError(std::io::Error),
    Other(String),
}

impl fmt::Display for YubiKeyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            YubiKeyError::NotFound => write!(f, "YubiKey or required key not found"),
            YubiKeyError::Pkcs11Error(msg) => write!(f, "PKCS#11 error: {}", msg),
            YubiKeyError::IoError(e) => write!(f, "I/O error: {}", e),
            YubiKeyError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl Error for YubiKeyError {}

impl From<pkcs11::errors::Error> for YubiKeyError {
    fn from(err: pkcs11::errors::Error) -> Self {
        YubiKeyError::Pkcs11Error(err.to_string())
    }
}

impl From<std::io::Error> for YubiKeyError {
    fn from(err: std::io::Error) -> Self {
        YubiKeyError::IoError(err)
    }
}

/// YubiKey context that manages the connection and session
pub struct YubiKey {
    ctx: Ctx,
    session: Option<CK_SESSION_HANDLE>,
    slot_id: Option<pkcs11::types::CK_SLOT_ID>,
}

impl YubiKey {
    /// Create a new YubiKey context by loading the PKCS#11 library
    pub fn new(pkcs11_lib_path: &str) -> Result<Self, YubiKeyError> {
        let ctx = Ctx::new_and_initialize(pkcs11_lib_path)
            .map_err(|e| YubiKeyError::Pkcs11Error(e.to_string()))?;
        
        Ok(Self {
            ctx,
            session: None,
            slot_id: None,
        })
    }
    
    /// Find the PKCS#11 library by searching in common locations
    pub fn find_pkcs11_path() -> Result<String, YubiKeyError> {
        // First check if we have an environment variable from the build script
        if let Ok(path) = std::env::var("OPENSC_PATH") {
            if Path::new(&path).exists() {
                return Ok(path);
            }
        }
        
        // Common paths for different operating systems
        let paths = match std::env::consts::OS {
            "macos" => vec![
                // Common macOS locations
                "/Library/OpenSC/lib/opensc-pkcs11.so",
                "/usr/local/lib/opensc-pkcs11.so",
                "/opt/homebrew/lib/opensc-pkcs11.so",
                "/usr/local/opt/opensc/lib/pkcs11/opensc-pkcs11.so",
                "/opt/local/lib/opensc-pkcs11.so",
            ],
            "linux" => vec![
                "/usr/lib/x86_64-linux-gnu/opensc-pkcs11.so",
                "/usr/lib/opensc-pkcs11.so",
                "/usr/local/lib/opensc-pkcs11.so",
            ],
            "windows" => vec![
                "C:\\Windows\\System32\\opensc-pkcs11.dll",
                "C:\\Program Files\\OpenSC Project\\OpenSC\\pkcs11\\opensc-pkcs11.dll",
            ],
            _ => vec![],
        };

        // Try each path
        for path in paths {
            if Path::new(path).exists() {
                return Ok(path.to_string());
            }
        }

        Err(YubiKeyError::Other(format!(
            "Could not find PKCS#11 library. Please install OpenSC or specify the path manually."
        )))
    }
    
    /// Get the default PKCS#11 library path based on the operating system
    pub fn default_pkcs11_path() -> &'static str {
        #[cfg(target_os = "linux")]
        return "/usr/lib/x86_64-linux-gnu/opensc-pkcs11.so";
        
        #[cfg(target_os = "macos")]
        return "/Library/OpenSC/lib/opensc-pkcs11.so";
        
        #[cfg(target_os = "windows")]
        return "C:\\Windows\\System32\\opensc-pkcs11.dll";
        
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        return "";
    }
    
    /// Create a new YubiKey context using the default PKCS#11 library path
    pub fn default() -> Result<Self, YubiKeyError> {
        // Try to find the library path automatically
        match Self::find_pkcs11_path() {
            Ok(path) => Self::new(&path),
            Err(_) => {
                // Fall back to the default path if automatic detection fails
                let default_path = Self::default_pkcs11_path();
                Self::new(default_path).map_err(|e| {
                    YubiKeyError::Other(format!(
                        "Failed to load PKCS#11 library. Please ensure OpenSC is installed. Error: {}", e
                    ))
                })
            }
        }
    }
    
    /// Connect to the YubiKey device
    pub fn connect(&mut self) -> Result<(), YubiKeyError> {
        // Get available slots with tokens
        let slots = self.ctx.get_slot_list(true)?;
        if slots.is_empty() {
            return Err(YubiKeyError::NotFound);
        }
        
        // Store the slot ID for later use
        self.slot_id = Some(slots[0]);
        
        // Open a session with the first available slot
        let session = self.ctx.open_session(
            slots[0], 
            CKF_SERIAL_SESSION | CKF_RW_SESSION, 
            None, 
            None
        )?;
        
        self.session = Some(session);
        Ok(())
    }
    
    /// Log in to the YubiKey with the provided PIN
    pub fn login(&self, pin: &str) -> Result<(), YubiKeyError> {
        if let Some(session) = self.session {
            self.ctx.login(session, CKU_USER, Some(pin))?;
            Ok(())
        } else {
            Err(YubiKeyError::Other("Not connected to YubiKey".to_string()))
        }
    }
    
    /// Find a private key on the YubiKey
    pub fn find_private_key(&self, key_label: Option<&str>, key_id: Option<&[u8]>) 
        -> Result<CK_OBJECT_HANDLE, YubiKeyError> {
        
        if let Some(session) = self.session {
            // Create a template to search for private keys
            let mut template = vec![
                CK_ATTRIBUTE {
                    attrType: CKA_CLASS,
                    pValue: &CKO_PRIVATE_KEY as *const CK_ULONG as CK_VOID_PTR,
                    ulValueLen: std::mem::size_of::<CK_ULONG>() as CK_ULONG,
                },
            ];
            
            // Add key label if provided
            if let Some(label) = key_label {
                template.push(CK_ATTRIBUTE {
                    attrType: CKA_LABEL,
                    pValue: label.as_ptr() as CK_VOID_PTR,
                    ulValueLen: label.len() as CK_ULONG,
                });
            }
            
            // Add key ID if provided
            if let Some(id) = key_id {
                template.push(CK_ATTRIBUTE {
                    attrType: CKA_ID,
                    pValue: id.as_ptr() as CK_VOID_PTR,
                    ulValueLen: id.len() as CK_ULONG,
                });
            }
            
            // Find private keys matching the template
            self.ctx.find_objects_init(session, &template)?;
            let objects = self.ctx.find_objects(session, 1)?;
            self.ctx.find_objects_final(session)?;
            
            if objects.is_empty() {
                return Err(YubiKeyError::NotFound);
            }
            
            Ok(objects[0])
        } else {
            Err(YubiKeyError::Other("Not connected to YubiKey".to_string()))
        }
    }
    
    /// Decrypt data using a private key on the YubiKey
    pub fn decrypt(&self, key: CK_OBJECT_HANDLE, encrypted_data: &[u8]) 
        -> Result<Vec<u8>, YubiKeyError> {
        
        if let Some(session) = self.session {
            // For RSA-based decryption (common for YubiKeys)
            let mechanism = CK_MECHANISM {
                mechanism: CKM_RSA_PKCS,
                pParameter: std::ptr::null_mut(),
                ulParameterLen: 0,
            };
            
            // Initialize decryption operation
            self.ctx.decrypt_init(session, &mechanism, key)?;
            
            // Perform decryption
            let decrypted_data = self.ctx.decrypt(session, encrypted_data)?;
            
            Ok(decrypted_data)
        } else {
            Err(YubiKeyError::Other("Not connected to YubiKey".to_string()))
        }
    }
    
    /// Log out from the YubiKey
    pub fn logout(&self) -> Result<(), YubiKeyError> {
        if let Some(session) = self.session {
            self.ctx.logout(session)?;
            Ok(())
        } else {
            Err(YubiKeyError::Other("Not connected to YubiKey".to_string()))
        }
    }
}

impl Drop for YubiKey {
    fn drop(&mut self) {
        if let Some(session) = self.session {
            // Try to log out and close the session
            let _ = self.ctx.logout(session);
            let _ = self.ctx.close_session(session);
        }
        
        // Finalize the PKCS#11 context
        self.ctx.finalize();
    }
}

// Example usage code - you can remove this section if not needed
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    
    fn decrypt_file_example() -> Result<(), Box<dyn Error>> {
        // Initialize YubiKey context
        let mut yubikey = YubiKey::default()?;
        
        // Connect to YubiKey
        yubikey.connect()?;
        
        // Prompt for PIN
        println!("Enter YubiKey PIN:");
        let mut pin = String::new();
        std::io::stdin().read_line(&mut pin)?;
        let pin = pin.trim();
        
        // Login with PIN
        yubikey.login(pin)?;
        
        // Find the decryption key (without specifying label or ID)
        let private_key = yubikey.find_private_key(None, None)?;
        
        // Load encrypted data from file
        let encrypted_data = fs::read("encrypted_data.bin")?;
        
        // Decrypt the data
        let decrypted_data = yubikey.decrypt(private_key, &encrypted_data)?;
        
        // Save decrypted data to file
        fs::write("decrypted_data.txt", decrypted_data)?;
        
        // Logout is handled automatically when yubikey goes out of scope
        
        Ok(())
    }

    #[test]
    fn test_decrypt_file_example() {
        decrypt_file_example() 
            .expect("Failed to decrypt file");
    }
}