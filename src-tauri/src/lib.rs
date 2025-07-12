
//! Secure Session Management System for Vault
//! 
//! This module provides a comprehensive secure session management system
//! that addresses critical security vulnerabilities in authentication and
//! session handling.

// Original application modules
mod encrypt;
mod file_system;
mod secrets;
mod app_state;
mod ipc;
pub mod yubikey;

// New secure session management modules
pub mod auth;
pub mod security;
pub mod error;

// Original imports and dependencies
use tauri_plugin_fs::FsExt;
use tauri::Manager;
use std::sync::Mutex;

// Original exports
pub use file_system::FileSystem;
pub use app_state::{AppState, TauriState};
pub use encrypt::MasterPassword;
use ipc::*;

// New secure session management exports
pub use auth::*;

// Re-export error types from both old and new systems
pub use error::{Error, Result}; // Original error types

pub struct W<T>(pub T);

/// Re-export commonly used types for secure session management
pub mod prelude {
    pub use crate::auth::{
        SessionManager, SessionToken, AuthManager, CryptoManager,
    };
    pub use crate::security::SecureMemory;
    pub use crate::error::{SessionError, AuthError, CryptoError};
    pub use secrecy::{SecretBox, ExposeSecret};
    pub use uuid::Uuid;
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
