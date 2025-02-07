use std::{
    fs,
    path::{Path, PathBuf},
};
use uuid::Uuid;

use crate::encrypt;
use crate::file_system;

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
    name: String,
}

impl Vault {
    pub fn new(name: String) -> Self {
        Self { name }
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
        value: data.value,
    };
    let json: String = serde_json::to_string(&secret).map_err(|e| e.to_string())?;
    let pk_path = vault.pk_path();
    let encrypted =
        encrypt::encrypt_string(&pk_path.as_path(), &json).map_err(|e| e.to_string())?;
    let out_path = vault.secret_path(&secret.id);
    fs::write(out_path, encrypted).map_err(|e| e.to_string())?;
    Ok("Submitted secret".to_string())
}

#[tauri::command]
pub fn get_secret(id: &str) -> Result<Secret, String> {
    let vault = Vault::new("default".to_string());
    let pk_path = vault.pk_path();
    let secret_path = vault.secret_path(id);
    let encrypted = fs::read_to_string(secret_path).map_err(|e| e.to_string())?;
    let decrypted =
        encrypt::decrypt_string(&pk_path.as_path(), &encrypted).map_err(|e| e.to_string())?;
    let secret: Secret = serde_json::from_str(&decrypted).map_err(|e| e.to_string())?;
    Ok(secret)
}

#[tauri::command]
pub fn get_secrets() -> Result<Vec<Secret>, String> {
    let vault = Vault::new("default".to_string());
    let pk_path = vault.pk_path();
    let secret_dir = vault.path();
    println!("Secret dir: {:?}", secret_dir);
    let mut secrets = vec![];
    for entry in fs::read_dir(secret_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if !entry.path().is_dir()
            && entry
                .path()
                .extension()
                .map(|s| s == "enc")
                .unwrap_or(false)
        {
            println!("This is an enc file");
            let path = entry.path();
            println!("Path to file: {:?}", path);
            let encrypted = fs::read_to_string(&path).map_err(|e| e.to_string())?;
            let decrypted = encrypt::decrypt_string(&pk_path.as_path(), &encrypted)
                .map_err(|e| e.to_string())?;
            let secret: Secret = serde_json::from_str(&decrypted).map_err(|e| e.to_string())?;
            secrets.push(secret);
        }
    }
    Ok(secrets)
}
