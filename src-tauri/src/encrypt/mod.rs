pub mod rsa;
use std::path::Path;
mod error;
pub use error::{Error, Result};

pub fn create_pk_at_path(path: &Path) -> Result<()> {
    let encryptor = rsa::Encryptor::new()?;
    encryptor.save_to_file(path)?;
    Ok(())
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