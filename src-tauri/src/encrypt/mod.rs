mod error;
mod rsa;
// pub mod rsa;
mod aes;
use crate::master_password::MasterPassword;
use crate::AppState;
pub use error::{Error, Result};
use std::fs;
pub use aes::AES;

#[derive(Debug)]
pub struct Encrypt {
    pk: rsa::RSA,
}

impl From<&AppState> for Encrypt {
    fn from(state: &AppState) -> Self {
        Encrypt::from_state(state).unwrap()
    }
}

impl Encrypt {
    pub fn new() -> Result<Self> {
        let encryptor = rsa::RSA::new()?;
        Ok(Self { pk: encryptor })
    }

    pub fn from_state(state: &AppState) -> Result<Self> {
        let pk = Self::get_pk(state)?;
        Ok(Self { pk })
    }

    pub fn from_string(pk: &str) -> Result<Self> {
        let pk = rsa::RSA::from_string(pk)?;
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
    fn get_pk(state: &AppState) -> Result<rsa::RSA> {
        let fs = state.file_system();
        let password_encryptor = MasterPassword::get_encryptor(state)?;
        let encrypted_pk = fs::read(fs.master_pk())
            .map_err(|_e| Error::Io("Unable to read file with master privatekey".to_string()))?;
        let encrypted_str = String::from_utf8_lossy(&encrypted_pk);
        let raw_pk = password_encryptor.decrypt(&encrypted_str)?;
        let pk = String::from_utf8_lossy(&raw_pk);
        rsa::RSA::from_string(&pk)
    }

}

#[cfg(test)]
mod test {
    use super::*;
    use crate::AppState;

    fn state() -> AppState {
        let password = "password";
        let state = AppState::new_test(&password);
        state
    }

    #[test]
    fn test_from_state() {
        let state = state();
        let error = Encrypt::from_state(&state);
        assert!(error.is_ok());
    }

    #[test]
    fn test_from_trait() {
        let state = state();
        assert!(matches!(Encrypt::from(&state), Encrypt { .. }));
    }    
}
