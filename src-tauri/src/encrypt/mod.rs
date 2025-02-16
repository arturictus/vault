mod error;
pub mod rsa;
use crate::file_system::FileSystem;
use crate::master_password;
use crate::AppState;
pub use error::{Error, Result};
use std::fs;
use std::path::Path;

#[derive(Debug)]
pub struct Encryptor {
    pk: rsa::Encryptor,
}

impl Encryptor {
    pub fn new() -> Result<Self> {
        let encryptor = rsa::Encryptor::new()?;
        Ok(Self { pk: encryptor })
    }

    pub fn from_state(state: &AppState) -> Result<Self> {
        let pk = Self::get_pk(state)?;
        Ok(Self { pk })
    }

    pub fn from_file(path: &Path) -> Result<Self> {
        let clear_pk = fs::read(path).map_err(|_e| Error::Io("Unable to read file".to_string()))?;
        let pk = String::from_utf8_lossy(&clear_pk);
        let pk = rsa::Encryptor::from_string(&pk)?;
        Ok(Self { pk })
    }

    pub fn from_string(pk: &str) -> Result<Self> {
        let pk = rsa::Encryptor::from_string(pk)?;
        Ok(Self { pk })
    }

    pub fn encrypt_string(&self, input: &str) -> Result<String> {
        self.pk.encrypt_string(input)
    }

    pub fn decrypt_string(&self, input: &str) -> Result<String> {
        self.pk.decrypt_string(input)
    }

    pub fn private_key_pem(&self) -> Result<String> {
        self.pk.private_key_pem()
    }

    pub fn public_key_pem(&self) -> Result<String> {
        self.pk.public_key_pem()
    }

    // internal functions
    fn get_pk(state: &AppState) -> Result<rsa::Encryptor> {
        let master_password = state.master_password().ok_or(Error::Custom("NoMasterPassword in encrypt".to_string()))?;
        let fs = state.file_system();
        let pk = Self::do_get_pk(&master_password, fs)?;
        Ok(pk)
    }

    fn do_get_pk(master_password: &str, fs: &FileSystem) -> Result<rsa::Encryptor> {
        let password_encryptor = master_password::do_get_encryptor(fs, master_password)?;
        let encrypted_pk = fs::read(fs.master_pk())
            .map_err(|_e| Error::Io("Unable to read file with master privatekey".to_string()))?;
        let encrypted_str = String::from_utf8_lossy(&encrypted_pk);
        let raw_pk = password_encryptor.decrypt(&encrypted_str)?;
        let pk = String::from_utf8_lossy(&raw_pk);
        rsa::Encryptor::from_string(&pk)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::master_password;
    use crate::AppState;

    fn state() -> AppState {
        let password = "password";
        let state = AppState::new_test(&password);
        state
    }

    #[test]
    fn test_get_pk() {
        let state = state();
        let error = Encryptor::from_state(&state);
        assert!(error.is_ok());
    }
}
