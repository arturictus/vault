mod error;


use crate::{AppState, FileSystem, State};
pub use error::{Error, Result};
use crate::encrypt::{Encrypt, AES};
use std::fs;
use std::path::Path;

#[tauri::command]
pub fn save_master_password(
    state: State,
    password: &str,
    private_key: Option<&str>,
) -> Result<String> {
    let mut state = state.lock().map_err(|e| Error::StateLock(e.to_string()))?;
    MasterPassword::save(&mut state, password, private_key)
}

#[tauri::command]
pub fn verify_master_password(state: State, password: &str) -> Result<String> {
    let mut state = state.lock().map_err(|e| Error::StateLock(e.to_string()))?;
    MasterPassword::verify(&mut state, password)
}

pub struct MasterPassword;

impl MasterPassword {
    pub fn save(
        state: &mut AppState,
        password: &str,
        private_key: Option<&str>,
    ) -> Result<String> {
        let fs = state.file_system();
        let encryptor = Self::store_master_password(fs, password)?;
        let pk = match private_key {
            Some(pk) => Encrypt::from_string(pk)?,
            None => Encrypt::new()?,
        };
        Self::store_pk(fs, pk, encryptor)?;
        state.set_master_password(password.to_string());
        state.set_authenticated(true);

        Ok("Master password saved".to_string())
    }

    fn store_master_password(fs: &FileSystem, password: &str) -> Result<AES> {
        let encryptor = AES::new(password);
        let encrypted = encryptor.encrypt(password.as_bytes())?;
        let path = fs.master_password();
        fs::write(path, encrypted)?;
        Ok(encryptor)
    }

    fn store_pk(
        fs: &FileSystem,
        pk: Encrypt,
        password_encryptor: AES,
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
    pub fn verify(state: &mut AppState, password: &str) -> Result<String> {
        let fs = state.file_system();
        println!("Verifying master password {}", password);
        match Self::do_verify_password(fs, password) {
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

    fn do_verify_password(fs: &FileSystem, password: &str) -> Result<AES> {
        let path = fs.master_password();
        let encoded = fs::read_to_string(path)?;
    
        let encryptor = AES::from_encrypted(password, &encoded)?;
    
        encryptor.decrypt(&encoded)?;
        Ok(encryptor)
    }

    pub fn get_encryptor(state: &AppState) -> Result<AES> {
        let fs = state.file_system();
        let password = state.master_password().ok_or(Error::Custom(
            "NoMasterPassword in master_password".to_string(),
        ))?;
        Self::do_verify_password(fs, &password)
    }
}


#[cfg(test)]
mod tests {
    use crate::AppState;

    use super::*;

    #[test]
    fn test_save() {
        let mut app_state = AppState::new_unauthenticated_test();
        let password = "secret";
        MasterPassword::save(&mut app_state, password, None).unwrap();
        assert_eq!(app_state.master_password().unwrap(), password);
        assert_eq!(app_state.is_authenticated(), true);
        assert!(app_state.file_system().master_pk().exists());
        assert!(app_state.file_system().master_pub().exists());
    }

    #[test]
    fn test_verify() {
        let password = "secret";
        let mut app_state = AppState::new_test(password);
        assert!(app_state.file_system().master_pk().exists());
        assert!(app_state.file_system().master_pub().exists());
        MasterPassword::verify(&mut app_state, password).unwrap();
        assert_eq!(app_state.master_password().unwrap(), password);
        assert_eq!(app_state.is_authenticated(), true);
    }

    #[test]
    fn test_get_encryptor() {
        let password = "secret";
        let app_state = AppState::new_test(password);
        let encryptor = MasterPassword::get_encryptor(&app_state);
        assert!(encryptor.is_ok());
    }
}
