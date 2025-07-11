use crate::{TauriState, Error, Result, MasterPassword, PasswordManager};

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
pub fn setup(state: TauriState, password: &str) -> Result<()> {
    let mut state = state.lock().map_err(|e| Error::StateLock(e.to_string()))?;
    let manager = PasswordManager::new(password);
    let data = manager.store_password("Application password", "mocked password for confirmation").map_err(|e| Error::MasterPassword(e.to_string()))?;
    Ok(())
   
}

#[tauri::command]
pub fn verify_master_password(state: TauriState, password: &str) -> Result<String> {
    let mut state = state.lock().map_err(|e| Error::StateLock(e.to_string()))?;
    MasterPassword::verify(&mut state, password).map_err(|e| Error::MasterPassword(e.to_string()))
}