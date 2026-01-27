//! Security utilities and memory protection

use secrecy::{SecretBox, ExposeSecret};
use zeroize::Zeroize;
use serde::{Deserialize, Serialize};
use std::fmt;
use crate::error::SecurityResult;

/// Secure memory wrapper that ensures sensitive data is properly protected
pub struct SecureMemory<T: Zeroize> {
    inner: SecretBox<T>,
}

impl<T: Zeroize> SecureMemory<T> {
    /// Create a new secure memory instance
    pub fn new(value: T) -> Self {
        Self {
            inner: SecretBox::new(Box::new(value)),
        }
    }
    
    /// Expose the secret value temporarily
    pub fn expose<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        f(self.inner.expose_secret())
    }
}

impl<T: Zeroize + fmt::Debug> fmt::Debug for SecureMemory<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SecureMemory")
            .field("inner", &"[REDACTED]")
            .finish()
    }
}

/// Secure string type for passwords and sensitive text
pub type SecureString = SecureMemory<String>;

/// Secure bytes type for cryptographic keys and binary data
pub type SecureBytes = SecureMemory<Vec<u8>>;

impl SecureString {
    /// Create from a string slice
    pub fn from_str(s: &str) -> Self {
        Self::new(s.to_string())
    }
    
    /// Get the length of the string
    pub fn len(&self) -> usize {
        self.expose(|s| s.len())
    }
    
    /// Check if the string is empty
    pub fn is_empty(&self) -> bool {
        self.expose(|s| s.is_empty())
    }
}

impl SecureBytes {
    /// Create from a byte slice
    pub fn from_slice(bytes: &[u8]) -> Self {
        Self::new(bytes.to_vec())
    }
    
    /// Get the length of the bytes
    pub fn len(&self) -> usize {
        self.expose(|b| b.len())
    }
    
    /// Check if the bytes are empty
    pub fn is_empty(&self) -> bool {
        self.expose(|b| b.is_empty())
    }
}

/// Marker trait implementations are handled by the secrecy crate internally

/// Security configuration for the session system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Maximum number of failed login attempts before account lockout
    pub max_failed_attempts: u32,
    
    /// Account lockout duration in seconds
    pub lockout_duration_secs: u64,
    
    /// Session timeout in seconds
    pub session_timeout_secs: u64,
    
    /// Maximum number of concurrent sessions per user
    pub max_concurrent_sessions: u32,
    
    /// Enable secure memory protection
    pub enable_memory_protection: bool,
    
    /// Force session renewal interval in seconds
    pub session_renewal_interval_secs: u64,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            max_failed_attempts: 5,
            lockout_duration_secs: 900, // 15 minutes
            session_timeout_secs: 3600, // 1 hour
            max_concurrent_sessions: 3,
            enable_memory_protection: true,
            session_renewal_interval_secs: 1800, // 30 minutes
        }
    }
}

/// Security utilities
pub struct SecurityUtils;

impl SecurityUtils {
    /// Generate cryptographically secure random bytes
    pub fn generate_random_bytes(length: usize) -> SecurityResult<SecureBytes> {
        use rand::RngCore;
        let mut rng = rand::thread_rng();
        let mut bytes = vec![0u8; length];
        rng.fill_bytes(&mut bytes);
        Ok(SecureBytes::new(bytes))
    }
    
    /// Generate a secure random string of specified length
    pub fn generate_random_string(length: usize) -> SecurityResult<SecureString> {
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        let mut rng = rand::thread_rng();
        
        let random_string: String = (0..length)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();
            
        Ok(SecureString::new(random_string))
    }
    
    /// Constant-time comparison of two byte slices
    pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
        if a.len() != b.len() {
            return false;
        }
        
        let mut result = 0u8;
        for (x, y) in a.iter().zip(b.iter()) {
            result |= x ^ y;
        }
        result == 0
    }
    
    /// Secure memory wipe (additional to zeroize)
    pub fn secure_wipe(data: &mut [u8]) -> SecurityResult<()> {
        // First pass: zeros
        data.fill(0);
        
        // Second pass: ones
        data.fill(0xFF);
        
        // Third pass: random
        use rand::RngCore;
        let mut rng = rand::thread_rng();
        rng.fill_bytes(data);
        
        // Final pass: zeros
        data.fill(0);
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_secure_memory() {
        let secret = SecureString::from_str("test_password");
        assert_eq!(secret.len(), 13);
        assert!(!secret.is_empty());
        
        secret.expose(|s| {
            assert_eq!(s, "test_password");
        });
    }
    
    #[test]
    fn test_random_generation() {
        let bytes = SecurityUtils::generate_random_bytes(32).unwrap();
        assert_eq!(bytes.len(), 32);
        
        let string = SecurityUtils::generate_random_string(16).unwrap();
        assert_eq!(string.len(), 16);
    }
    
    #[test]
    fn test_constant_time_eq() {
        let a = b"hello";
        let b = b"hello";
        let c = b"world";
        
        assert!(SecurityUtils::constant_time_eq(a, b));
        assert!(!SecurityUtils::constant_time_eq(a, c));
        assert!(!SecurityUtils::constant_time_eq(a, b"hell"));
    }
}

// Implement Clone for specific types that support it
impl Clone for SecureString {
    fn clone(&self) -> Self {
        self.expose(|s| Self::new(s.clone()))
    }
}

impl Clone for SecureBytes {
    fn clone(&self) -> Self {
        self.expose(|bytes| Self::new(bytes.clone()))
    }
}
