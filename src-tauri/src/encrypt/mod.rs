pub mod rsa;
use std::{error::Error, path::Path};

pub fn create_pk_at_path(path: &Path) -> Result<(), Box<dyn Error>> {
    let encryptor = rsa::Encryptor::new()?;
    encryptor.save_to_file(path)?;
    Ok(())
}

pub fn encrypt_string(pk_path: &Path, input: &str) -> Result<String, Box<dyn Error>> {
    let encryptor = rsa::Encryptor::from_file(pk_path)?;
    encryptor.encrypt_string(input)
}