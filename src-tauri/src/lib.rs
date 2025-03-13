
mod encrypt;
mod file_system;
mod secrets;
mod error;
mod app_state;
mod ipc;
pub mod yubikey;
use tauri_plugin_fs::FsExt;
use tauri::Manager;
use std::sync::Mutex;

pub use file_system::FileSystem;
pub use error::{Error, Result};
pub use app_state::{AppState, TauriState};
pub use encrypt::MasterPassword;
use ipc::*;

pub struct W<T>(pub T);


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // Initialize AppState first
            app.manage(Mutex::new(AppState::default()));
            let fs = FileSystem::default();
            // Initialize file system
            fs.init()?;

            let app_dir = fs.app_data_directory();
            let scope = app.fs_scope();
            scope.allow_directory(&app_dir, false)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            is_authenticated,
            create_secret,
            get_secrets,
            get_secret,
            save_master_password,
            verify_master_password,
            log_out,
            list_yubikeys,
            encrypt_with_yubikey,
            save_yubikey_settings,
        ])
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
