mod password_encryptor;
use password_encryptor::PasswordEncryptor;
use std::{error::Error, fs};
use crate::file_system;

#[tauri::command]
pub fn save_master_password(password: &str) -> Result<String, String> {
    let encryptor = PasswordEncryptor::new(password);
    let encrypted = encryptor.encrypt(password.as_bytes()).map_err(|e| e.to_string())?;
    let path = file_system::master_password();
    fs::write(path, encrypted).map_err(|e| e.to_string())?;
    Ok("Master password saved".to_string())
}

#[tauri::command]
pub fn verify_master_password(password: &str) -> Result<String, String> {
    match do_verify_password(password) {
        Ok(_) => Ok("Master password correct".to_string()),
        Err(_) => Err("Master password incorrect".to_string())   
    }
}

fn do_verify_password(password: &str) -> Result<PasswordEncryptor, String> {
    let path = file_system::master_password();
    let encrypted = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let encryptor = PasswordEncryptor::new(password);
    let decrypted = encryptor.decrypt(&encrypted).map_err(|e| e.to_string())?;
    if decrypted == password.as_bytes() {
        Ok(encryptor)
    } else {
        Err("Master password incorrect".to_string())
    }
}

pub fn encrypt(password: &str, input: &str) -> Result<String, String> {
    let encryptor = do_verify_password(password)?;
    encryptor.encrypt(input.as_bytes()).map_err(|e| e.to_string())
}

pub fn decrypt(password: &str, input: &str) -> Result<String, String> {
    let encryptor = do_verify_password(password)?;
    let decrypted = encryptor.decrypt(input).map_err(|e| e.to_string())?;
    Ok(String::from_utf8(decrypted).map_err(|e| e.to_string())?)
}