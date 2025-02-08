use dirs;
use serde::de;
use std::error::Error;
use std::fmt;
use tauri_plugin_fs::FsExt;
mod encrypt;
mod file_system;
mod secrets;
mod master_password;
use std::fs;

#[derive(Debug)]
struct AppError {
    message: String
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for AppError {}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn authenticate(password: &str) -> bool {
    password == "password"
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            file_system::init().map_err(|e| AppError { message: format!("Failed to initialize file system: {}", e) })?;
            let app_dir = file_system::app_data_directory();
            let scope = app.fs_scope();
            scope.allow_directory(&app_dir, false)?;
            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            authenticate,
            secrets::create_secret,
            secrets::get_secrets,
            secrets::get_secret
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
