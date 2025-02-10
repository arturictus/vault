use std::path::{Path, PathBuf};

use dirs;

pub fn app_data_directory() -> String {
    main_folder().join("app-data").to_string_lossy().to_string()
}
pub fn vaults_folder() -> String {
    main_folder().join("vaults").to_string_lossy().to_string()
}

// pub fn pks_folder() -> String {
//     main_folder().join("pks").to_string_lossy().to_string()
// }
pub fn pk_for_vault(vault_name: &str) -> String {
    Path::new(&vault_folder(vault_name)).join("private_key").to_string_lossy().to_string()
}

pub fn master_password() -> String {
    main_folder().join("master_password.enc").to_string_lossy().to_string()
}
pub fn master_pk() -> String {
    main_folder().join("rsa_master_pk.enc").to_string_lossy().to_string()
}

pub fn master_pub() -> String {
    main_folder().join("rsa_master_pub").to_string_lossy().to_string()
}
pub fn vault_folder(vault_name: &str) -> String {
    let vault_folder = format!("{}.vault", vault_name);
    Path::new(&vaults_folder()).join(vault_folder).to_string_lossy().to_string()
}

pub fn create_vault_folder(vault_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let folder = vault_folder(vault_name);
    std::fs::create_dir_all(&folder)?;
    Ok(())
}

pub fn main_folder() -> PathBuf {
    dirs::home_dir().map(|home| home.join(".vault")).unwrap()
}

pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    let app_dir = app_data_directory();
    std::fs::create_dir_all(&app_dir)?;
    let vaults_dir = vaults_folder();
    std::fs::create_dir_all(&vaults_dir)?;
    let default_vault_dir = vault_folder("default");
    std::fs::create_dir_all(&default_vault_dir)?;
    Ok(())
}