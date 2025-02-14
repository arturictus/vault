mod error;
mod vault;
pub use error::Result;
use vault::{Vault, VaultFs};

use std::fs;
use uuid::Uuid;

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

#[tauri::command]
pub fn create_secret(data: SecretForm) -> Result<String> {
    println!("Received secret: {:?}", data);
    let vault = Vault::new("default".to_string());
    let secret = Secret {
        id: Uuid::new_v4().to_string(),
        kind: data.kind,
        name: data.name,
        value: data.value,
    };
    store_secret(&vault, &secret)?;
    Ok("Submitted secret".to_string())
}


fn store_secret(vault: &Vault, secret: &Secret) -> Result<()> {
    let json = serde_json::to_string(secret)?;
    let encrypted = encrypt::encrypt_string(vault.pk_path().as_path(), &json)?;
    let out_path = vault.secret_path(&secret.id);
    fs::write(out_path, encrypted)?;
    Ok(())
}

#[tauri::command]
pub fn get_secret(id: &str) -> Result<Secret> {
    let vault = Vault::new("default".to_string());
    let secret = read_secret(&vault, id)?;
    Ok(secret)
}

fn read_secret(vault: &Vault, id: &str) -> Result<Secret> {
    let pk_path = vault.pk_path();
    let secret_path = vault.secret_path(id);
    let encrypted = fs::read_to_string(secret_path)?;
    let decrypted =
        encrypt::decrypt_string(pk_path.as_path(), &encrypted)?;
    let secret: Secret = serde_json::from_str(&decrypted)?;
    Ok(secret)
}

#[tauri::command]
pub fn get_secrets() -> Result<Vec<Secret>> {
    let vault = Vault::new("default".to_string());
    let secrets = do_get_secrets(vault)?;
    Ok(secrets)
}

fn do_get_secrets(vault: Vault) -> Result<Vec<Secret>> {
    let pk_path = vault.pk_path();
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
            let decrypted = encrypt::decrypt_string(pk_path.as_path(), &encrypted)?;
            let secret: Secret = serde_json::from_str(&decrypted)?;
            secrets.push(secret);
        }
    }
    Ok(secrets)
}

#[cfg(test)]
mod tests {
    
    use crate::file_system::{TestFileSystem, FileSystem};

    fn setup() -> TestFileSystem {
        let fs = TestFileSystem::default();
        fs.init().unwrap();
        fs
    }
    #[test]
    fn test_store_master_password() {}
}