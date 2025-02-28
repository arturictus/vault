use crate::{TauriState, Error, Result};
use crate::secrets::{NewSecretForm, Secret};

#[tauri::command]
pub fn create_secret(state: TauriState, data: NewSecretForm) -> Result<String> {
    println!("Received secret: {:?}", data);
    let state = state.lock().map_err(|e| Error::StateLock(e.to_string()))?;
    let secret: Secret = data.into();
    secret.save(&state)?;
    Ok("Submitted secret".to_string())
}

#[tauri::command]
pub fn get_secrets(state: TauriState) -> Result<Vec<Secret>> {
    let state = state.lock().map_err(|e| Error::StateLock(e.to_string()))?;
    let secrets = Secret::all(&state)?;
    Ok(secrets)
}

#[tauri::command]
pub fn get_secret(state: TauriState, id: &str) -> Result<Secret> {
    let state = state.lock().map_err(|e| Error::StateLock(e.to_string()))?;
    let secret = Secret::find(&state, id)?;
    Ok(secret)
}