// todo
// yubikey::list_yubikeys,
// yubikey::encrypt_with_yubikey,
// yubikey::authenticate_with_yubikey,
// yubikey::generate_yubikey_challenge
use crate::{Error, Result, yubikey, TauriState};

#[tauri::command]
pub fn list_yubikeys() -> Result<Vec<yubikey::YubiKeyInfo>> {
    yubikey::list_yubikeys().map_err(|e| Error::YubiKeyError(e.to_string()))
}
#[tauri::command]
pub fn encrypt_with_yubikey(yubikey_serial: u32, data: String) -> Result<String> {
    println!("encrypt_with_yubikey {:?}", yubikey_serial);
    println!("encrypt_with_yubikey {:?}", data);
    Err(Error::YubiKeyError("Not implemented".to_string()))
    // yubikey::encrypt_with_yubikey(yubikey_serial, &data).map_err(|e| Error::YubiKeyError(e.to_string()))
}

#[tauri::command]
pub fn save_yubikey_settings(state: TauriState, serial: u32, public_key: String) -> Result<()> {
    let app_state = state.lock().map_err(|e| Error::StateLock(e.to_string()))?;
    let list = yubikey::list_yubikeys().map_err(|e| Error::YubiKeyError(e.to_string()))?;
    let found = list.iter().find(move |x| x.serial.unwrap() == serial).ok_or(Error::YubiKeyError("YubiKey not found".to_string()))?;
    let mut selected = found.clone();
    selected.set_pub_key(public_key);
    selected.save(&app_state).map_err(|e| Error::YubiKeyError(e.to_string()))?;
    Ok(())
}