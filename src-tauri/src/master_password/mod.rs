mod error;
mod password_encryptor;
use crate::encrypt::Encryptor;
use crate::{FileSystem, State, AppState};
pub use error::{Error, Result};
use password_encryptor::PasswordEncryptor;
use std::fs;
use std::path::Path;

#[tauri::command]
pub fn save_master_password(
    state: State,
    password: &str,
    private_key: Option<&str>,
) -> Result<String> {
    let mut state = state.lock().map_err(|e| Error::StateLock(e.to_string()))?;
    let fs = state.file_system();
    let encryptor = store_master_password(&fs, password)?;
    let pk = match private_key {
        Some(pk) => Encryptor::from_string(pk)?,
        None => Encryptor::new()?,
    };
    store_pk(&fs, pk, encryptor)?;
    state.set_master_password(password.to_string());
    state.set_authenticated(true);

    Ok("Master password saved".to_string())
}

pub fn store_master_password(fs: &FileSystem, password: &str) -> Result<PasswordEncryptor> {
    let encryptor = PasswordEncryptor::new(password);
    let encrypted = encryptor.encrypt(password.as_bytes())?;
    let path = fs.master_password();
    fs::write(path, encrypted)?;
    Ok(encryptor)
}

pub fn store_pk(
    fs: &FileSystem,
    pk: Encryptor,
    password_encryptor: PasswordEncryptor,
) -> Result<()> {
    let master_pk = fs.master_pk();
    let pk_for_default_path = Path::new(&master_pk);

    let pem = pk.private_key_pem()?;
    let encrypted_pk = password_encryptor.encrypt(pem.as_bytes())?;
    fs::write(pk_for_default_path, &encrypted_pk)?;
    let public = pk.public_key_pem()?;
    fs::write(fs.master_pub(), public)?;

    Ok(())
}

#[tauri::command]
pub fn verify_master_password(state: State, password: &str) -> Result<String> {
    let mut state = state.lock().map_err(|e| Error::StateLock(e.to_string()))?;
    let fs = state.file_system();
    println!("Verifying master password {}", password);
    match do_verify_password(&fs, password) {
        Ok(_) => {
            state.set_master_password(password.to_string());
            state.set_authenticated(true);
            Ok("Master password correct".to_string())
        }
        Err(_) => Err(Error::WrongPassword(
            "Master password incorrect".to_string(),
        )),
    }
}



fn do_verify_password(fs: &FileSystem, password: &str) -> Result<PasswordEncryptor> {
    let path = fs.master_password();
    let encoded = fs::read_to_string(path)?;

    let encryptor = PasswordEncryptor::from_encrypted(password, &encoded)?;

    encryptor.decrypt(&encoded)?;
    Ok(encryptor)
}

// TODO: test
pub fn get_encryptor(state: &AppState) -> Result<PasswordEncryptor> {
    let fs = state.file_system();
    let master_password = state.master_password()
        .ok_or(Error::Custom("NoMasterPassword in master_password".to_string()))?;
    let encryptor = do_get_encryptor(&fs, &master_password)?;
    Ok(encryptor)
}

// TODO: test
pub fn do_get_encryptor(fs: &FileSystem, password: &str) -> Result<PasswordEncryptor> {
    let encryptor = do_verify_password(fs, password)?;
    Ok(encryptor)
}

// #[cfg(test)]
// pub fn test_setup(state: &AppState, password: &str) -> Result<()> {
//     store_master_password(state.file_system(), password)?;
//     Ok(())
// }

#[cfg(test)]
mod tests {
    use crate::AppState;

    use super::*;

    fn setup() -> AppState {
        AppState::new_test("secret")
    }
    #[test]
    fn test_store_master_password() {
        let password = "secret";
        let app_state = setup();
        let fs = app_state.file_system();
        store_master_password(fs, password).unwrap();
        do_verify_password(fs, password).unwrap();
    }

    #[test]
    fn test_verify_password() {
        let app_state = setup();
        let fs = app_state.file_system();
        let result = do_verify_password(fs, "secret");
        assert!(result.is_ok());
        let result = do_verify_password(fs, "wrong");
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
