mod error;
pub use error::{Result, Error};
use crate::{AppState, MasterPassword};

use std::fs;
use uuid::Uuid;

static VAULT: &str = "default";


#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct NewSecretForm {
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

impl From<NewSecretForm> for Secret {
    fn from(data: NewSecretForm) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            kind: data.kind,
            name: data.name,
            value: data.value,
        }
    }
}

impl Secret {
    pub fn save(&self, state: &AppState) -> Result<()> {
        let fs = state.file_system();
        let json = serde_json::to_string(&self)?;
        let encryptor = MasterPassword::from_state(state)?;
        let encrypted = encryptor.encrypt_string(&json)?;
        let out_path = fs.secret_path(VAULT, &self.id);
        fs::write(out_path, encrypted)?;
        Ok(())
    }
    
    pub fn find(state: &AppState, id: &str) -> Result<Secret> {
        let fs = state.file_system();
        let encryptor = MasterPassword::from_state(state)?;
        let secret_path = fs.secret_path(VAULT, id);
        let encrypted = fs::read_to_string(secret_path)?;
        let decrypted =
            encryptor.decrypt_string(&encrypted)?;
        let secret: Secret = serde_json::from_str(&decrypted)?;
        Ok(secret)
    }
    
    pub fn all(state: &AppState) -> Result<Vec<Secret>> {
        let fs = state.file_system();
        let encryptor = MasterPassword::from_state(state)?;
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
        secret.save(&state).unwrap();
        let read_secret = Secret::find(&state, id).unwrap();
        assert_eq!(secret, read_secret);
        let all = Secret::all(&state).unwrap();
        assert_eq!([secret], all.as_slice());
    }
}