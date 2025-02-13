use std::error::Error;
use std::fmt;
use tauri_plugin_fs::FsExt;
mod encrypt;
mod file_system;
mod master_password;
mod secrets;
use file_system::{FileSystem, DefaultFileSystem};

use std::sync::Mutex;

use tauri::{Builder, Manager};

struct AppState {
    master_password: Option<String>,
    authenticated: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            master_password: None,
            authenticated: false,
        }
    }
}

#[tauri::command]
fn is_authenticated(state: tauri::State<'_, Mutex<AppState>>) -> Result<bool, String> {
    println!("Checking if authenticated");
    let state = state.lock().map_err(|e| e.to_string())?;
    if state.authenticated {
        println!("====> true");
        return Ok(true)
    } else {
        println!("====> false");
        return Ok(false);
    }
}

#[derive(Debug)]
struct AppError {
    message: String,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for AppError {}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // Initialize AppState first
            app.manage(Mutex::new(AppState::default()));
            let fs = DefaultFileSystem::default();
            // Initialize file system
            fs.init().map_err(|e| AppError {
                message: format!("Failed to initialize file system: {}", e),
            })?;

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
        ])
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
