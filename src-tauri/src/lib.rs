
use tauri_plugin_fs::FsExt;
mod encrypt;
mod file_system;
mod master_password;
mod secrets;
mod error;
mod app_state;
pub use file_system::FileSystem;
pub use error::{Error, Result};
use tauri::Manager;

use std::sync::Mutex;

pub use app_state::{AppState, State};


#[tauri::command]
fn is_authenticated(state: State) -> Result<bool> {
    println!("Checking if authenticated");
    let state = state.lock()?;
    if state.is_authenticated() {
        println!("====> true");
        Ok(true)
    } else {
        println!("====> false");
        Ok(false)
    }
}
#[tauri::command]
fn log_out(state: State) -> Result<()> {
    let mut state = state.lock()?;
    state.log_out();
    Ok(())
}

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
            secrets::create_secret,
            secrets::get_secrets,
            secrets::get_secret,
            master_password::save_master_password,
            master_password::verify_master_password,
            log_out
        ])
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
