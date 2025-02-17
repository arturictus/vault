mod error;
pub use error::{Result, Error};
use crate::{AppState, Encryptor, State};

use std::fs;
use uuid::Uuid;

static VAULT: &str = "default";


#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct SecretForm {
    kind: String,
    name: String,
    value: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq)]
pub struct Secret {
    id: String,
    kind: String,
    name: String,
    value: String,
}

impl Secret {
    pub fn new(data: &SecretForm) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            kind: data.kind.clone(),
            name: data.name.clone(),
            value: data.value.clone()
        }
    }
    
}

#[tauri::command]
pub fn create_secret(state: State, data: SecretForm) -> Result<String> {
    println!("Received secret: {:?}", data);
    let state = state.lock().map_err(|e| Error::AppStateLock(e.to_string()))?;
    let secret = Secret::new(&data);
    store_secret(&state, &secret)?;
    Ok("Submitted secret".to_string())
}

#[tauri::command]
pub fn get_secrets(state: State) -> Result<Vec<Secret>> {
    let state = state.lock().map_err(|e| Error::AppStateLock(e.to_string()))?;
    let secrets = do_get_secrets(&state)?;
    Ok(secrets)
}

#[tauri::command]
pub fn get_secret(state: State, id: &str) -> Result<Secret> {
    let state = state.lock().map_err(|e| Error::AppStateLock(e.to_string()))?;
    let secret = read_secret(&state, id)?;
    Ok(secret)
}

// Internal functions
fn store_secret(state: &AppState, secret: &Secret) -> Result<()> {
    let fs = state.file_system();
    let json = serde_json::to_string(&secret)?;
    let encryptor = Encryptor::from_state(state)?;
    let encrypted = encryptor.encrypt_string(&json)?;
    let out_path = fs.secret_path(VAULT, &secret.id);
    fs::write(out_path, encrypted)?;
    Ok(())
}

fn read_secret(state: &AppState, id: &str) -> Result<Secret> {
    let fs = state.file_system();
    let encryptor = Encryptor::from_state(state)?;
    let secret_path = fs.secret_path(VAULT, id);
    let encrypted = fs::read_to_string(secret_path)?;
    let decrypted =
        encryptor.decrypt_string(&encrypted)?;
    let secret: Secret = serde_json::from_str(&decrypted)?;
    Ok(secret)
}

fn do_get_secrets(state: &AppState) -> Result<Vec<Secret>> {
    let fs = state.file_system();
    let encryptor = Encryptor::from_state(state)?;
    let secret_dir = fs.vault_folder(VAULT);
    let mut secrets = vec![];
    for entry in fs::read_dir(secret_dir)? {
        let entry = entry?;
        if !entry.path().is_dir()
            && entry
                .path()
                .extension()
                .map(|s| s == "enc")
                .unwrap_or(false)
        {
            let path = entry.path();
            let encrypted = fs::read_to_string(&path)?;
            let decrypted = encryptor.decrypt_string(&encrypted)?;
            let secret: Secret = serde_json::from_str(&decrypted)?;
            secrets.push(secret);
        }
    }
    Ok(secrets)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AppState;

    fn setup() -> AppState {
        AppState::new_test("secret")
    }

    #[test]
    fn test_store_and_read_secret() {
        let state = setup();
        let id = "test-id";
        let secret = Secret {
            id: id.to_string(),
            kind: "test".to_string(),
            name: "test".to_string(),
            value: "test".to_string(),
        };
        store_secret(&state, &secret).unwrap();
        let read_secret = read_secret(&state, id).unwrap();
        assert_eq!(secret, read_secret);
    }
}