
//! Improved authentication system with secure memory protection

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::crypto::CryptoManager;
use crate::security::{SecureString, SecurityConfig};
use crate::error::{AuthError, AuthResult};

/// User account information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAccount {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub failed_attempts: u32,
    pub locked_until: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub roles: Vec<String>,
}

impl UserAccount {
    /// Create a new user account
    pub fn new(username: String, password_hash: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            username,
            password_hash,
            created_at: Utc::now(),
            last_login: None,
            failed_attempts: 0,
            locked_until: None,
            is_active: true,
            roles: vec!["user".to_string()],
        }
    }
    
    /// Check if the account is currently locked
    pub fn is_locked(&self) -> bool {
        if let Some(locked_until) = self.locked_until {
            Utc::now() < locked_until
        } else {
            false
        }
    }
    
    /// Lock the account for the specified duration
    pub fn lock_account(&mut self, duration_secs: u64) {
        self.locked_until = Some(Utc::now() + Duration::seconds(duration_secs as i64));
    }
    
    /// Unlock the account
    pub fn unlock_account(&mut self) {
        self.locked_until = None;
        self.failed_attempts = 0;
    }
    
    /// Record a failed login attempt
    pub fn record_failed_attempt(&mut self) {
        self.failed_attempts += 1;
    }
    
    /// Record a successful login
    pub fn record_successful_login(&mut self) {
        self.last_login = Some(Utc::now());
        self.failed_attempts = 0;
        self.locked_until = None;
    }
}

/// Authentication credentials
#[derive(Debug)]
pub struct Credentials {
    pub username: String,
    pub password: SecureString,
}

impl Credentials {
    pub fn new(username: String, password: String) -> Self {
        Self {
            username,
            password: SecureString::from_str(&password),
        }
    }
}

/// Authentication result
#[derive(Debug, Clone)]
pub struct AuthenticationResult {
    pub user_id: Uuid,
    pub username: String,
    pub roles: Vec<String>,
    pub authenticated_at: DateTime<Utc>,
}

/// Authentication manager
pub struct AuthManager {
    users: Arc<RwLock<HashMap<String, UserAccount>>>,
    crypto: CryptoManager,
    config: SecurityConfig,
}

impl AuthManager {
    /// Create a new authentication manager
    pub fn new(config: SecurityConfig) -> Self {
        Self {
            users: Arc::new(RwLock::new(HashMap::new())),
            crypto: CryptoManager::new(),
            config,
        }
    }
    
    /// Register a new user
    pub async fn register_user(&self, username: String, password: SecureString) -> AuthResult<Uuid> {
        let mut users = self.users.write().await;
        
        // Check if user already exists
        if users.contains_key(&username) {
            return Err(AuthError::InvalidCredentials);
        }
        
        // Hash the password
        let password_hash = self.crypto.hash_password(&password)
            .map_err(AuthError::CryptoError)?;
        
        // Create new user account
        let user = UserAccount::new(username.clone(), password_hash);
        let user_id = user.id;
        
        users.insert(username, user);
        
        Ok(user_id)
    }
    
    /// Authenticate a user
    pub async fn authenticate(&self, credentials: Credentials) -> AuthResult<AuthenticationResult> {
        let mut users = self.users.write().await;
        
        let user = users.get_mut(&credentials.username)
            .ok_or_else(|| AuthError::UserNotFound {
                username: credentials.username.clone(),
            })?;
        
        // Check if account is active
        if !user.is_active {
            return Err(AuthError::AccountLocked {
                username: credentials.username,
            });
        }
        
        // Check if account is locked
        if user.is_locked() {
            return Err(AuthError::AccountLocked {
                username: credentials.username,
            });
        }
        
        // Verify password
        let password_valid = self.crypto.verify_password(&credentials.password, &user.password_hash)
            .map_err(AuthError::CryptoError)?;
        
        if !password_valid {
            user.record_failed_attempt();
            
            // Lock account if too many failed attempts
            if user.failed_attempts >= self.config.max_failed_attempts {
                user.lock_account(self.config.lockout_duration_secs);
                return Err(AuthError::TooManyAttempts);
            }
            
            return Err(AuthError::PasswordVerificationFailed);
        }
        
        // Successful authentication
        user.record_successful_login();
        
        Ok(AuthenticationResult {
            user_id: user.id,
            username: user.username.clone(),
            roles: user.roles.clone(),
            authenticated_at: Utc::now(),
        })
    }
    
    /// Get user by username
    pub async fn get_user(&self, username: &str) -> AuthResult<Option<UserAccount>> {
        let users = self.users.read().await;
        Ok(users.get(username).cloned())
    }
    
    /// Get user by ID
    pub async fn get_user_by_id(&self, user_id: Uuid) -> AuthResult<Option<UserAccount>> {
        let users = self.users.read().await;
        Ok(users.values().find(|u| u.id == user_id).cloned())
    }
    
    /// Update user password
    pub async fn update_password(&self, username: &str, new_password: SecureString) -> AuthResult<()> {
        let mut users = self.users.write().await;
        
        let user = users.get_mut(username)
            .ok_or_else(|| AuthError::UserNotFound {
                username: username.to_string(),
            })?;
        
        // Hash the new password
        let password_hash = self.crypto.hash_password(&new_password)
            .map_err(AuthError::CryptoError)?;
        
        user.password_hash = password_hash;
        
        Ok(())
    }
    
    /// Deactivate user account
    pub async fn deactivate_user(&self, username: &str) -> AuthResult<()> {
        let mut users = self.users.write().await;
        
        let user = users.get_mut(username)
            .ok_or_else(|| AuthError::UserNotFound {
                username: username.to_string(),
            })?;
        
        user.is_active = false;
        
        Ok(())
    }
    
    /// Unlock user account
    pub async fn unlock_user(&self, username: &str) -> AuthResult<()> {
        let mut users = self.users.write().await;
        
        let user = users.get_mut(username)
            .ok_or_else(|| AuthError::UserNotFound {
                username: username.to_string(),
            })?;
        
        user.unlock_account();
        
        Ok(())
    }
    
    /// List all users (admin function)
    pub async fn list_users(&self) -> AuthResult<Vec<UserAccount>> {
        let users = self.users.read().await;
        Ok(users.values().cloned().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::SecurityConfig;
    
    #[tokio::test]
    async fn test_user_registration() {
        let config = SecurityConfig::default();
        let auth_manager = AuthManager::new(config);
        
        let password = SecureString::from_str("test_password_123");
        let user_id = auth_manager.register_user("testuser".to_string(), password).await.unwrap();
        
        let user = auth_manager.get_user("testuser").await.unwrap().unwrap();
        assert_eq!(user.id, user_id);
        assert_eq!(user.username, "testuser");
        assert!(user.is_active);
    }
    
    #[tokio::test]
    async fn test_authentication() {
        let config = SecurityConfig::default();
        let auth_manager = AuthManager::new(config);
        
        // Register user
        let password = SecureString::from_str("test_password_123");
        auth_manager.register_user("testuser".to_string(), password).await.unwrap();
        
        // Authenticate with correct credentials
        let credentials = Credentials::new("testuser".to_string(), "test_password_123".to_string());
        let result = auth_manager.authenticate(credentials).await.unwrap();
        
        assert_eq!(result.username, "testuser");
        assert!(result.roles.contains(&"user".to_string()));
        
        // Authenticate with wrong password
        let wrong_credentials = Credentials::new("testuser".to_string(), "wrong_password".to_string());
        let result = auth_manager.authenticate(wrong_credentials).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_account_locking() {
        let mut config = SecurityConfig::default();
        config.max_failed_attempts = 2;
        let auth_manager = AuthManager::new(config);
        
        // Register user
        let password = SecureString::from_str("test_password_123");
        auth_manager.register_user("testuser".to_string(), password).await.unwrap();
        
        // Make failed attempts
        for _ in 0..3 {
            let wrong_credentials = Credentials::new("testuser".to_string(), "wrong_password".to_string());
            let _ = auth_manager.authenticate(wrong_credentials).await;
        }
        
        // Account should be locked now
        let correct_credentials = Credentials::new("testuser".to_string(), "test_password_123".to_string());
        let result = auth_manager.authenticate(correct_credentials).await;
        assert!(matches!(result, Err(AuthError::AccountLocked { .. })));
    }
}
