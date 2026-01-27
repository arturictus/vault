//! Secure Session Management System for Vault
//!
//! This module provides a comprehensive secure session management system
//! that addresses critical security vulnerabilities in authentication and
//! session handling.

// Original application modules
mod app_state;
mod encrypt;
mod file_system;
mod ipc;
mod secrets;
pub mod yubikey;

// New secure session management modules
pub mod auth;
pub mod error;
pub mod security;

// Original imports and dependencies
use std::sync::{Arc, Mutex};
use tauri::async_runtime::block_on;
use tauri::Manager;
use tauri_plugin_fs::FsExt;

// Original exports
pub use app_state::{AppState, TauriState};
pub use encrypt::MasterPassword;
pub use file_system::FileSystem;
use ipc::*;

// New secure session management exports
pub use auth::*;

// Re-export error types from both old and new systems
pub use error::{Error, Result}; // Original error types

pub struct W<T>(pub T);

/// Re-export commonly used types for secure session management
pub mod prelude {
    pub use crate::auth::{AuthManager, CryptoManager, SessionManager, SessionToken};
    pub use crate::error::{AuthError, CryptoError, SessionError};
    pub use crate::security::SecureMemory;
    pub use secrecy::{ExposeSecret, SecretBox};
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

            // Initialize Secure Session Management
            let security_config = security::SecurityConfig::default();
            let auth_manager = Arc::new(AuthManager::new(security_config.clone(), fs.clone()));
            let session_manager = Arc::new(block_on(SessionManager::new(security_config))?);

            app.manage(auth_manager);
            app.manage(session_manager.clone());

            // Start session cleanup task
            tauri::async_runtime::spawn(async move {
                SessionMiddleware::start_cleanup_task(session_manager).await;
            });

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
            // Auth commands
            login,
            register,
            logout,
            validate_session,
            renew_session,
        ])
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
