mod password_encryptor;
use crate::{file_system::DefaultFileSystem, file_system::FileSystem, AppState};
use password_encryptor::PasswordEncryptor;
use crate::encrypt::rsa;
use std::fs;
use std::path::Path;
use std::sync::Mutex;
use tauri::State;
#[tauri::command]
pub fn save_master_password(
    state: State<'_, Mutex<AppState>>,
    password: &str,
    private_key: Option<&str>,
) -> Result<String, String> {
    let fs = DefaultFileSystem::default();
    let encryptor = store_master_password(&fs, password)?;
    let pk = match private_key {
        Some(pk) => rsa::Encryptor::from_string(pk).map_err(|e| e.to_string())?,
        None => crate::encrypt::create_pk().map_err(|e| e.to_string())?,
    };
    store_pk(&fs, pk, encryptor)?;
    let mut state = state.lock().map_err(|e| e.to_string())?;
    state.master_password = Some(password.to_string());
    state.authenticated = true;

    Ok("Master password saved".to_string())
}

fn store_master_password<T: FileSystem>(
    fs: &T,
    password: &str,
) -> Result<PasswordEncryptor, String> {
    let encryptor = PasswordEncryptor::new(password);
    let encrypted = encryptor
        .encrypt(password.as_bytes())
        .map_err(|e| e.to_string())?;
    let path = fs.master_password();
    fs::write(path, encrypted).map_err(|e| e.to_string())?;
    Ok(encryptor)
}

fn store_pk<T: FileSystem>(
    fs: &T,
    pk: rsa::Encryptor,
    password_encryptor: PasswordEncryptor,
) -> Result<(), String> {
    let master_pk = fs.master_pk();
    let pk_for_default_path = Path::new(&master_pk);

    let pem = pk.private_key_pem().map_err(|e| e.to_string())?;
    let encrypted_pk = password_encryptor
        .encrypt(pem.as_bytes())
        .map_err(|e| e.to_string())?;
    fs::write(pk_for_default_path, &encrypted_pk).map_err(|e| e.to_string())?;
    let public = pk.public_key_pem().map_err(|e| e.to_string())?;
    fs::write(fs.master_pub(), public).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn verify_master_password(
    state: State<'_, Mutex<AppState>>,
    password: &str,
) -> Result<String, String> {
    let fs = DefaultFileSystem::default();
    println!("Verifying master password {}", password);
    match do_verify_password(&fs, password) {
        Ok(_) => {
            let mut state = state.lock().map_err(|e| e.to_string())?;
            state.master_password = Some(password.to_string());
            state.authenticated = true;
            Ok("Master password correct".to_string())
        }
        Err(_) => Err("Master password incorrect".to_string()),
    }
}

fn do_verify_password<T: FileSystem>(fs: &T, password: &str) -> Result<PasswordEncryptor, String> {
    let path = fs.master_password();
    let encoded = fs::read_to_string(path).map_err(|e| e.to_string())?;

    let encryptor =
        PasswordEncryptor::from_encrypted(password, &encoded).map_err(|e| e.to_string())?;

    encryptor.decrypt(&encoded).map_err(|e| e.to_string())?;
    Ok(encryptor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file_system::TestFileSystem;

    fn setup() -> TestFileSystem {
        let fs = TestFileSystem::default();
        fs.init().unwrap();
        fs
    }
    #[test]
    fn test_store_master_password() {
        let password = "secret";
        let fs = setup();
        store_master_password(&fs, password).unwrap();
        do_verify_password(&fs, password).unwrap();
    }

    #[test]
    fn test_verify_password() {
        let fs = setup();
        let result = do_verify_password(&fs, "secret");
        assert!(result.is_err());
    }
}

// pub fn encrypt(password: &str, input: &str) -> Result<String, String> {
//     let encryptor = do_verify_password(password)?;
//     encryptor
//         .encrypt(input.as_bytes())
//         .map_err(|e| e.to_string())
// }

// pub fn decrypt(password: &str, input: &str) -> Result<String, String> {
//     let encryptor = do_verify_password(password)?;
//     let decrypted = encryptor.decrypt(input).map_err(|e| e.to_string())?;
//     Ok(String::from_utf8(decrypted).map_err(|e| e.to_string())?)
// }
