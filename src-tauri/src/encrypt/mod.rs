pub mod rsa;
use std::{error::Error, path::Path};

pub fn create_pk_at_path(path: &Path) -> Result<(), Box<dyn Error>> {
    let encryptor = rsa::Encryptor::new()?;
    encryptor.save_to_file(path)?;
    Ok(())
}