use std::{fs, path::{Path, PathBuf}};
use uuid::Uuid;

use crate::file_system;
use crate::encrypt;

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

pub struct Vault {
    name: String
}

impl Vault {
    pub fn new(name: String) -> Self {
        Self {
            name
        }
    }
    pub fn path(&self) -> PathBuf {
        Path::new(&file_system::vault_folder(&self.name)).to_path_buf()
    }
    pub fn secret_path(&self, id: &str) -> PathBuf {
        self.path().join(format!("{}.enc", id)).to_path_buf()
    }

    pub fn pk_path(&self) -> PathBuf {
        Path::new(&file_system::pk_for_vault(&self.name)).to_path_buf()
    }
}

#[tauri::command]
pub fn create_secret(data: SecretForm) -> Result<String, String> {
    println!("Received secret: {:?}", data);
    let vault = Vault::new("default".to_string());
    let secret = Secret {
        id: Uuid::new_v4().to_string(),
        kind: data.kind,
        name: data.name,
        value: data.value
    };
    let json: String = serde_json::to_string(&secret).map_err(|e| e.to_string())?;
    let pk_path = vault.pk_path();
    let encrypted = encrypt::encrypt_string(&pk_path.as_path(), &json).map_err(|e| e.to_string())?;
    let out_path = vault.secret_path(&secret.id);
    fs::write(out_path, encrypted).map_err(|e| e.to_string())?;
    Ok("Submitted secret".to_string())
}

#[tauri::command]
pub fn get_secrets() -> Result<Vec<Secret>, String> {
    Ok(vec![
        Secret {
            id: "1".to_string(),
            kind: "login".to_string(),
            name: "secret1".to_string(),
            value: "password".to_string(),
        },
        Secret {
            id: "2".to_string(),
            kind: "login".to_string(),
            name: "secret2".to_string(),
            value: "password".to_string(),
        },
        Secret {
            id: "3".to_string(),
            kind: "login".to_string(),
            name: "secret3".to_string(),
            value: "password".to_string(),
        },
    ])
}

#[tauri::command]
pub fn get_secret(id: String) -> Result<Secret, String> {
    get_secrets()
        .unwrap()
        .into_iter()
        .find(|secret| secret.id == id)
        .ok_or_else(|| "Secret not found".to_string())
}
