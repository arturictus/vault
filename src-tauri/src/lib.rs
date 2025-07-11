
//! Secure Session Management System for Vault
//! 
//! This module provides a comprehensive secure session management system
//! that addresses critical security vulnerabilities in authentication and
//! session handling.

pub mod auth;
pub mod crypto;
pub mod security;
pub mod session;
pub mod error;

pub use auth::*;
pub use crypto::*;
pub use security::*;
pub use session::*;
pub use error::*;

/// Re-export commonly used types
pub mod prelude {
    pub use crate::{
        SessionManager, SessionToken, AuthManager, SecureMemory,
        CryptoManager, SessionError, AuthError, CryptoError,
    };
    pub use secrecy::{SecretBox, ExposeSecret};
    pub use uuid::Uuid;
}
