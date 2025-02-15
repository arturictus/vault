mod password_encryptor;
mod error;
use crate::{file_system, AppState};
use password_encryptor::PasswordEncryptor;
use std::path::Path;
use std::sync::Mutex;
use std::fs;
use tauri::State;
pub use error::{Error, Result};

#[tauri::command]
pub fn save_master_password(
    state: State<'_, Mutex<AppState>>,
    password: &str,
    private_key: Option<&str>,
) -> Result<String> {
    let encryptor = PasswordEncryptor::new(password);
    let encrypted = encryptor
        .encrypt(password.as_bytes())
        .map_err(|e| e.to_string())?;
    let path = file_system::master_password();
    fs::write(path, encrypted).map_err(|e| e.to_string())?;
    let master_pk = file_system::master_pk();
    let pk_for_default_path = Path::new(&master_pk);
    match private_key {
        Some(pk) => {
            let pk = encryptor
                .encrypt(pk.as_bytes())
                .map_err(|e| e.to_string())?;
            fs::write(pk_for_default_path, pk).map_err(|e| e.to_string())?;
        }
        None => {
            if !pk_for_default_path.exists() {
                let pk = crate::encrypt::create_pk().map_err(|e| e.to_string())?;
                let pem = pk.private_key_pem().map_err(|e| e.to_string())?;
                let encrypted_pk = encryptor
                    .encrypt(pem.as_bytes())
                    .map_err(|e| e.to_string())?;
                fs::write(pk_for_default_path, encrypted_pk.to_string()).map_err(|e| e.to_string())?;
                let public = pk.public_key_pem().map_err(|e| e.to_string())?;
                fs::write(file_system::master_pub(), public).map_err(|e| e.to_string())?;
            }
        }
    }
    let mut state = state.lock().map_err(|e| e.to_string())?;
    state.master_password = Some(password.to_string());
    state.authenticated = true;

    Ok("Master password saved".to_string())
}
fn store_master_password(password: &str) -> Result<()>{
    let encryptor = PasswordEncryptor::new(password);
    let encrypted = encryptor
        .encrypt(password.as_bytes())
        .map_err(|e| e.to_string())?;
    let path = file_system::master_password();
    fs::write(path, encrypted).map_err(|e| e.to_string())?;
    Ok(())
}
#[tauri::command]
pub fn verify_master_password(
    state: State<'_, Mutex<AppState>>,
    password: &str,
) -> Result<String, String> {
    println!("Verifying master password {}", password);
    match do_verify_password(password) {
        Ok(_) => {
            let mut state = state.lock().map_err(|e| e.to_string())?;
            state.master_password = Some(password.to_string());
            state.authenticated = true;
            Ok("Master password correct".to_string())
        }
        Err(_) => Err("Master password incorrect".to_string()),
    }
}

fn do_verify_password(password: &str) -> Result<PasswordEncryptor, String> {
    println!("HELLOOOOOOOO");
    let path = file_system::master_password();
    let encrypted = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let encryptor = PasswordEncryptor::new(password);
    let decrypted = encryptor.decrypt(&encrypted).map_err(|e| e.to_string())?;
    println!("decrypted password: {:?}", &decrypted);
    if decrypted == password.as_bytes() {
        Ok(encryptor)
    } else {
        Err("Master password incorrect".to_string())
    }
}

pub fn encrypt(password: &str, input: &str) -> Result<String, String> {
    let encryptor = do_verify_password(password)?;
    encryptor
        .encrypt(input.as_bytes())
        .map_err(|e| e.to_string())
}

pub fn decrypt(password: &str, input: &str) -> Result<String, String> {
    let encryptor = do_verify_password(password)?;
    let decrypted = encryptor.decrypt(input).map_err(|e| e.to_string())?;
    Ok(String::from_utf8(decrypted).map_err(|e| e.to_string())?)
}

#[test]
fn test_store_master_password() {
    let password = "secret";
    store_master_password(&password).unwrap();
    do_verify_password(password).unwrap();
}

#[test]
fn test_verify_password() {
    let result = do_verify_password("secret");
    result.unwrap();
    
}

#[test]
fn test_password_is_well_stored() {
    let password = "secret";
    let path = file_system::master_password();
    println!("path to master_password {}", path);
    let encrypted = fs::read_to_string(path).map_err(|e| e.to_string()).unwrap();
    let encryptor = PasswordEncryptor::new(password);
    let decrypted = encryptor.decrypt(&encrypted).unwrap();
    println!("decrypted pass: {:?}", decrypted);
    assert_eq!(password.as_bytes(), decrypted)

}