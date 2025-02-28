use crate::{TauriState, Error, Result, MasterPassword};

#[tauri::command]
pub fn save_master_password(
    state: TauriState,
    password: &str,
    private_key: Option<&str>,
) -> Result<String> {
    let mut state = state.lock().map_err(|e| Error::StateLock(e.to_string()))?;
    MasterPassword::save(&mut state, password, private_key).map_err(|e| Error::MasterPassword(e.to_string()))
}

#[tauri::command]
pub fn verify_master_password(state: TauriState, password: &str) -> Result<String> {
    let mut state = state.lock().map_err(|e| Error::StateLock(e.to_string()))?;
    MasterPassword::verify(&mut state, password).map_err(|e| Error::MasterPassword(e.to_string()))
}