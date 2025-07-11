
//! Error types for the secure session management system

use thiserror::Error;

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

/// Result type aliases for convenience
pub type SessionResult<T> = Result<T, SessionError>;
pub type AuthResult<T> = Result<T, AuthError>;
pub type CryptoResult<T> = Result<T, CryptoError>;
pub type SecurityResult<T> = Result<T, SecurityError>;
