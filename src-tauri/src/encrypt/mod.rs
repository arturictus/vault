mod rsa;
use std::{error::Error, path::Path};

#[tauri::command]
pub fn create_new_rsa_key(name: &str) -> Result<String, String> {
    let app_dir: String = crate::file_system::pks_folder().to_string();
    let path = Path::new(&app_dir).join(format!("rsa_{}", name));
    println!("Creating new RSA key at {:?}", path);
    match do_create_new_rsa_key(&path) {
        Ok(_encryptor) => Ok("ok".to_string()),
        Err(e) => Err(e.to_string()),
    }
}

fn do_create_new_rsa_key(path: &Path) -> Result<rsa::Encryptor, Box<dyn Error>> {
    let encryptor = rsa::Encryptor::new()?;
    encryptor.save_to_file(path)?;
    Ok(encryptor)
}
