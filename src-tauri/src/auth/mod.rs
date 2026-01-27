//! Authentication module
//! 
//! This module contains all authentication-related functionality including:
//! - User authentication and authorization
//! - Cryptographic operations
//! - Session management

pub mod auth_manager;
pub mod crypto;
pub mod session;

// Re-export main types for convenience
pub use auth_manager::*;
pub use crypto::*;
pub use session::*;
