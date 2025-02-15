mod error;
mod password_encryptor;
use crate::encrypt::Encryptor;
use crate::{file_system::DefaultFileSystem, file_system::FileSystem, AppState};
pub use error::{Error, Result};
use password_encryptor::PasswordEncryptor;
use std::fs;
use std::path::Path;
use std::sync::Mutex;
use tauri::State;

#[tauri::command]
pub fn save_master_password(
    state: State<'_, Mutex<AppState>>,
    password: &str,
    private_key: Option<&str>,
) -> Result<String> {
    let fs = DefaultFileSystem::default();
    let encryptor = store_master_password(&fs, password)?;
    let pk = match private_key {
        Some(pk) => Encryptor::from_string(pk)?,
        None => Encryptor::new()?,
    };
    store_pk(&fs, pk, encryptor)?;
    let mut state = state.lock().map_err(|e| Error::StateLock(e.to_string()))?;
    state.set_master_password(password.to_string());
    state.set_authenticated(true);

    Ok("Master password saved".to_string())
}

fn store_master_password<T: FileSystem>(fs: &T, password: &str) -> Result<PasswordEncryptor> {
    let encryptor = PasswordEncryptor::new(password);
    let encrypted = encryptor.encrypt(password.as_bytes())?;
    let path = fs.master_password();
    fs::write(path, encrypted)?;
    Ok(encryptor)
}

fn store_pk<T: FileSystem>(
    fs: &T,
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
pub fn verify_master_password(state: State<'_, Mutex<AppState>>, password: &str) -> Result<String> {
    let fs = DefaultFileSystem::default();
    println!("Verifying master password {}", password);
    match do_verify_password(&fs, password) {
        Ok(_) => {
            let mut state = state.lock().map_err(|e| Error::StateLock(e.to_string()))?;
            state.set_master_password(password.to_string());
            state.set_authenticated(true);
            Ok("Master password correct".to_string())
        }
        Err(_) => Err(Error::WrongPassword(
            "Master password incorrect".to_string(),
        )),
    }
}



fn do_verify_password<T: FileSystem>(fs: &T, password: &str) -> Result<PasswordEncryptor> {
    let path = fs.master_password();
    let encoded = fs::read_to_string(path)?;

    let encryptor = PasswordEncryptor::from_encrypted(password, &encoded)?;

    encryptor.decrypt(&encoded)?;
    Ok(encryptor)
}

// TODO: test
pub fn get_encryptor(state: State<'_, Mutex<AppState>>) -> Result<PasswordEncryptor> {
    let fs = DefaultFileSystem::default();
    let state = state.lock().map_err(|e| Error::StateLock(e.to_string()))?;
    let master_password = state.master_password()
        .ok_or(Error::NoMasterPassword)?;
    let encryptor = do_get_encryptor(&fs, &master_password)?;
    Ok(encryptor)
}

// TODO: test
pub fn do_get_encryptor<T: FileSystem>(fs: &T, password: &str) -> Result<PasswordEncryptor> {
    do_verify_password(fs, password)
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
