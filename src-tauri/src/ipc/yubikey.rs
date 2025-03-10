// todo
// yubikey::list_yubikeys,
// yubikey::encrypt_with_yubikey,
// yubikey::authenticate_with_yubikey,
// yubikey::generate_yubikey_challenge
use crate::{Error, Result, yubikey};

#[tauri::command]
pub fn list_yubikeys() -> Result<Vec<yubikey::YubiKeyInfo>> {
    yubikey::list_yubikeys().map_err(|e| Error::YubiKeyError(e.to_string()))
}
#[tauri::command]
pub fn encrypt_with_yubikey(yubikey_serial: u32, data: String) -> Result<String> {
    yubikey::encrypt_with_yubikey(yubikey_serial, &data).map_err(|e| Error::YubiKeyError(e.to_string()))
}