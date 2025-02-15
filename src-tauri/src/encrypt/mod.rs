pub mod rsa;
use std::path::Path;
mod error;
pub use error::{Error, Result};
use crate::file_system::{FileSystem, DefaultFileSystem};
use crate::master_password;
use std::fs;
use crate::State;

struct Encryptor {
    pk: rsa::Encryptor,
}

impl Encryptor {
    fn from_state(state: State) -> Result<Self> {
        let pk = Self::get_pk(state)?;
        Ok(Self { pk })
    }

    fn encrypt_string(&self, input: &str) -> Result<String> {
        self.pk.encrypt_string(input)
    }

    fn decrypt_string(&self, input: &str) -> Result<String> {
        self.pk.decrypt_string(input)
    }

    // internal functions
    fn get_pk(state: State) -> Result<rsa::Encryptor> {
        let state = state.lock().map_err(|e| Error::StateLock(e.to_string()))?;
        let master_password = state.master_password.as_ref().ok_or(Error::NoMasterPassword)?;
        let fs = DefaultFileSystem::default();
        let pk = Self::do_get_pk(master_password, &fs)?;
        Ok(pk)
    }
    
    fn do_get_pk<T: FileSystem>(master_password: &str, fs: &T) -> Result<rsa::Encryptor> {
        let password_encryptor = master_password::do_get_encryptor(fs, master_password)?;
        let encrypted_pk = fs::read(fs.master_pk())
            .map_err(|_e| Error::Io("Unable to read file with master privatekey".to_string()))?;
        let encrypted_str = String::from_utf8_lossy(&encrypted_pk);
        let raw_pk = password_encryptor.decrypt(&encrypted_str)?;
        let pk = String::from_utf8_lossy(&raw_pk);
        rsa::Encryptor::from_string(&pk)
    }
}

pub fn create_pk() -> Result<rsa::Encryptor> {
    let encryptor = rsa::Encryptor::new()?;
    Ok(encryptor)
}

pub fn encrypt_string(pk_path: &Path, input: &str) -> Result<String> {
    let encryptor = rsa::Encryptor::from_file(pk_path)?;
    encryptor.encrypt_string(input)
}

pub fn decrypt_string(pk_path: &Path, input: &str) -> Result<String> {
    let encryptor = rsa::Encryptor::from_file(pk_path)?;
    encryptor.decrypt_string(input)
}





#[cfg(test)]
mod test {
    use super::*;
    use crate::AppState;
    use std::sync::Mutex;
    
    fn state() -> State<'static> {
        let state = AppState{master_password: Some("password".to_string()), authenticated: true};
        let state = Mutex::new(state);
        State::from(state)
    }

    #[test]
    fn test_get_pk() {
        Encryptor::from_state(state()).unwrap();
    }
}