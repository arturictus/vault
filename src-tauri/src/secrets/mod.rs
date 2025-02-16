mod error;
mod vault;
pub use error::{Result, Error};
use vault::Vault;

use crate::{State, Encryptor};

use std::fs;
use uuid::Uuid;


#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct SecretForm {
    kind: String,
    name: String,
    value: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Secret {
    id: String,
    kind: String,
    name: String,
    value: String,
}

#[tauri::command]
pub fn create_secret(state: State, data: SecretForm) -> Result<String> {
    println!("Received secret: {:?}", data);
    let state = state.lock().map_err(|e| Error::AppStateLock(e.to_string()))?;
    let fs = state.file_system();
    let vault = Vault::new("default".to_string(), fs.clone());
    let secret = Secret {
        id: Uuid::new_v4().to_string(),
        kind: data.kind,
        name: data.name,
        value: data.value,
    };
    store_secret(&vault, &secret)?;
    Ok("Submitted secret".to_string())
}

// TODO: This is just temporary. We need to use a proper encrypt::Encryptor
fn store_secret(vault: &Vault, secret: &Secret) -> Result<()> {
    let json = serde_json::to_string(secret)?;
    let encryptor = Encryptor::from_file(vault.pk_path().as_path())?;
    let encrypted = encryptor.encrypt_string(&json)?;
    let out_path = vault.secret_path(&secret.id);
    fs::write(out_path, encrypted)?;
    Ok(())
}

#[tauri::command]
pub fn get_secret(state: State, id: &str) -> Result<Secret> {
    let state = state.lock().map_err(|e| Error::AppStateLock(e.to_string()))?;
    let fs = state.file_system();
    let vault = Vault::new("default".to_string(), fs.clone());
    let secret = read_secret(&vault, id)?;
    Ok(secret)
}


fn read_secret(vault: &Vault, id: &str) -> Result<Secret> {
    let encryptor = Encryptor::from_file(vault.pk_path().as_path())?;

    let secret_path = vault.secret_path(id);
    let encrypted = fs::read_to_string(secret_path)?;
    let decrypted =
        encryptor.decrypt_string(&encrypted)?;
    let secret: Secret = serde_json::from_str(&decrypted)?;
    Ok(secret)
}

#[tauri::command]
pub fn get_secrets(state: State) -> Result<Vec<Secret>> {
    let state = state.lock().map_err(|e| Error::AppStateLock(e.to_string()))?;
    let fs = state.file_system();
    let vault = Vault::new("default".to_string(), fs.clone());
    let secrets = do_get_secrets(vault)?;
    Ok(secrets)
}

fn do_get_secrets(vault: Vault) -> Result<Vec<Secret>> {
    let encryptor = Encryptor::from_file(vault.pk_path().as_path())?;
    let secret_dir = vault.path();
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
    
    

    use crate::AppState;

    fn setup() -> AppState {
        AppState::new_test("secret")
    }
    #[test]
    fn test_store_master_password() {}
}