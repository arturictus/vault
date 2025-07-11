
//! Error types for both the original application and the secure session management system

use thiserror::Error;
use std::sync::PoisonError;
use std::sync::MutexGuard;

// Original application error types
pub type Result<T> = core::result::Result<T, Error>;

#[derive(Error, Debug, serde::Serialize)]
pub enum Error {
  TauriInit(String),
  Secrets(String),
  Custom(String),
  Encryption(String),
  Io(String),
  StateLock(String),
  MasterPassword(String),
  YubiKeyError(String),
}

impl core::fmt::Display for Error {
	fn fmt(
		&self,
		fmt: &mut core::fmt::Formatter,
	) -> core::result::Result<(), core::fmt::Error> {
		write!(fmt, "{self:?}")
	}
}

impl<T> From<PoisonError<MutexGuard<'_, T>>> for Error {
    fn from(_: PoisonError<MutexGuard<'_, T>>) -> Self {
        Error::TauriInit("Mutex lock poisoned".to_string())
    }
}

impl From<String> for Error {
    fn from(e: String) -> Self {
        Error::Custom(e)
    }
}

impl From<crate::encrypt::Error> for Error {
    fn from(e: crate::encrypt::Error) -> Self {
        Error::Encryption(e.to_string())
    }
}

impl From<crate::secrets::Error> for Error {
    fn from(e: crate::secrets::Error) -> Self {
        Error::Custom(e.to_string())
    }
}

// New secure session management error types

/// Main error type for session operations
#[derive(Debug, Error)]
pub enum SessionError {
    #[error("Session not found: {session_id}")]
    SessionNotFound { session_id: String },
    
    #[error("Session expired: {session_id}")]
    SessionExpired { session_id: String },
    
    #[error("Invalid session token")]
    InvalidToken,
    
    #[error("Session creation failed: {reason}")]
    CreationFailed { reason: String },
    
    #[error("Session validation failed: {reason}")]
    ValidationFailed { reason: String },
    
    #[error("Cryptographic error: {0}")]
    CryptoError(#[from] CryptoError),
    
    #[error("Authentication error: {0}")]
    AuthError(#[from] AuthError),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

/// Authentication-specific errors
#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    
    #[error("User not found: {username}")]
    UserNotFound { username: String },
    
    #[error("Password verification failed")]
    PasswordVerificationFailed,
    
    #[error("Account locked: {username}")]
    AccountLocked { username: String },
    
    #[error("Too many failed attempts")]
    TooManyAttempts,
    
    #[error("Cryptographic error: {0}")]
    CryptoError(#[from] CryptoError),
}

/// Cryptographic operation errors
#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("Key derivation failed: {reason}")]
    KeyDerivationFailed { reason: String },
    
    #[error("Encryption failed: {reason}")]
    EncryptionFailed { reason: String },
    
    #[error("Decryption failed: {reason}")]
    DecryptionFailed { reason: String },
    
    #[error("Invalid key length: expected {expected}, got {actual}")]
    InvalidKeyLength { expected: usize, actual: usize },
    
    #[error("Random number generation failed")]
    RandomGenerationFailed,
    
    #[error("Hash verification failed")]
    HashVerificationFailed,
    
    #[error("Argon2 error: {0}")]
    Argon2Error(String),
    
    #[error("AES-GCM error: {0}")]
    AesGcmError(String),
}

/// Security-related errors
#[derive(Debug, Error)]
pub enum SecurityError {
    #[error("Memory protection failed: {reason}")]
    MemoryProtectionFailed { reason: String },
    
    #[error("Secure wipe failed")]
    SecureWipeFailed,
    
    #[error("Access denied: {reason}")]
    AccessDenied { reason: String },
}

// Result type aliases for convenience
pub type SessionResult<T> = core::result::Result<T, SessionError>;
pub type AuthResult<T> = core::result::Result<T, AuthError>;
pub type CryptoResult<T> = core::result::Result<T, CryptoError>;
pub type SecurityResult<T> = core::result::Result<T, SecurityError>;
