//! Improved authentication system with secure memory protection

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::crypto::CryptoManager;
use crate::error::{AuthError, AuthResult};
use crate::file_system::FileSystem;
use crate::security::{SecureString, SecurityConfig};

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
    fs: FileSystem,
}

impl AuthManager {
    /// Create a new authentication manager
    pub fn new(config: SecurityConfig, fs: FileSystem) -> Self {
        // Try to load existing users
        let users_map = Self::load_users(&fs).unwrap_or_else(|e| {
            println!("Failed to load users: {:?}", e);
            HashMap::new()
        });

        Self {
            users: Arc::new(RwLock::new(users_map)),
            crypto: CryptoManager::new(),
            config,
            fs,
        }
    }

    /// Load users from disk
    fn load_users(fs: &FileSystem) -> AuthResult<HashMap<String, UserAccount>> {
        let path = fs.users_file();
        if !path.exists() {
            return Ok(HashMap::new());
        }

        let content = std::fs::read_to_string(path).map_err(|e| {
            AuthError::CryptoError(crate::error::CryptoError::AesGcmError(e.to_string()))
        })?;

        // If file is empty, ignore
        if content.is_empty() {
            return Ok(HashMap::new());
        }

        let users_list: Vec<UserAccount> = serde_json::from_str(&content).map_err(|e| {
            AuthError::CryptoError(crate::error::CryptoError::AesGcmError(e.to_string()))
        })?;

        let mut users_map = HashMap::new();
        for user in users_list {
            users_map.insert(user.username.clone(), user);
        }

        Ok(users_map)
    }

    /// Save users to disk
    async fn save_users(&self) -> AuthResult<()> {
        let users = self.users.read().await;
        let users_list: Vec<&UserAccount> = users.values().collect();

        let json = serde_json::to_string_pretty(&users_list).map_err(|e| {
            AuthError::CryptoError(crate::error::CryptoError::AesGcmError(e.to_string()))
        })?;

        let path = self.fs.users_file();
        tokio::fs::write(path, json).await.map_err(|e| {
            AuthError::CryptoError(crate::error::CryptoError::AesGcmError(e.to_string()))
        })?;

        Ok(())
    }

    /// Register a new user
    pub async fn register_user(
        &self,
        username: String,
        password: SecureString,
    ) -> AuthResult<Uuid> {
        let mut users = self.users.write().await;

        // Check if user already exists
        if users.contains_key(&username) {
            return Err(AuthError::UserAlreadyExists { username });
        }

        // Hash the password
        // Hash the password in a blocking task
        let crypto = self.crypto.clone();
        let password_clone = password.clone();

        let password_hash = tokio::task::spawn_blocking(move || {
            crypto
                .hash_password(&password_clone)
                .map_err(AuthError::CryptoError)
        })
        .await
        .map_err(|e| AuthError::CryptoError(crate::error::CryptoError::Internal(e.into())))??;

        // Create new user account
        let user = UserAccount::new(username.clone(), password_hash);
        let user_id = user.id;

        users.insert(username, user);

        // Release lock before saving to avoid deadlocks (though save_users takes read lock)
        drop(users);
        self.save_users().await?;

        Ok(user_id)
    }

    /// Authenticate a user
    pub async fn authenticate(&self, credentials: Credentials) -> AuthResult<AuthenticationResult> {
        let mut users = self.users.write().await;

        let user = users
            .get_mut(&credentials.username)
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
        // Verify password in blocking task
        let crypto = self.crypto.clone();
        let password = credentials.password.clone();
        let hash = user.password_hash.clone();

        let password_valid = tokio::task::spawn_blocking(move || {
            crypto
                .verify_password(&password, &hash)
                .map_err(AuthError::CryptoError)
        })
        .await
        .map_err(|e| AuthError::CryptoError(crate::error::CryptoError::Internal(e.into())))??;

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

        // Clone data needed for result
        let result = AuthenticationResult {
            user_id: user.id,
            username: user.username.clone(),
            roles: user.roles.clone(),
            authenticated_at: Utc::now(),
        };

        // Save state (last login updated)
        drop(users);
        self.save_users().await?;

        Ok(result)
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
    pub async fn update_password(
        &self,
        username: &str,
        new_password: SecureString,
    ) -> AuthResult<()> {
        let mut users = self.users.write().await;

        let user = users
            .get_mut(username)
            .ok_or_else(|| AuthError::UserNotFound {
                username: username.to_string(),
            })?;

        // Hash the new password
        // Hash the new password in blocking task
        let crypto = self.crypto.clone();
        let password_clone = new_password.clone();

        let password_hash = tokio::task::spawn_blocking(move || {
            crypto
                .hash_password(&password_clone)
                .map_err(AuthError::CryptoError)
        })
        .await
        .map_err(|e| AuthError::CryptoError(crate::error::CryptoError::Internal(e.into())))??;

        user.password_hash = password_hash;

        drop(users);
        self.save_users().await?;

        Ok(())
    }

    /// Deactivate user account
    pub async fn deactivate_user(&self, username: &str) -> AuthResult<()> {
        let mut users = self.users.write().await;

        let user = users
            .get_mut(username)
            .ok_or_else(|| AuthError::UserNotFound {
                username: username.to_string(),
            })?;

        user.is_active = false;

        drop(users);
        self.save_users().await?;

        Ok(())
    }

    /// Unlock user account
    pub async fn unlock_user(&self, username: &str) -> AuthResult<()> {
        let mut users = self.users.write().await;

        let user = users
            .get_mut(username)
            .ok_or_else(|| AuthError::UserNotFound {
                username: username.to_string(),
            })?;

        user.unlock_account();

        drop(users);
        self.save_users().await?;

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
        let temp_dir = tempfile::TempDir::new().unwrap();
        let fs = FileSystem::new_test(temp_dir.path().to_path_buf());
        let auth_manager = AuthManager::new(config, fs);

        let password = SecureString::from_str("test_password_123");
        let user_id = auth_manager
            .register_user("testuser".to_string(), password)
            .await
            .unwrap();

        let user = auth_manager.get_user("testuser").await.unwrap().unwrap();
        assert_eq!(user.id, user_id);
        assert_eq!(user.username, "testuser");
        assert!(user.is_active);
    }

    #[tokio::test]
    async fn test_authentication() {
        let config = SecurityConfig::default();
        let temp_dir = tempfile::TempDir::new().unwrap();
        let fs = FileSystem::new_test(temp_dir.path().to_path_buf());
        let auth_manager = AuthManager::new(config, fs);

        // Register user
        let password = SecureString::from_str("test_password_123");
        auth_manager
            .register_user("testuser".to_string(), password)
            .await
            .unwrap();

        // Authenticate with correct credentials
        let credentials = Credentials::new("testuser".to_string(), "test_password_123".to_string());
        let result = auth_manager.authenticate(credentials).await.unwrap();

        assert_eq!(result.username, "testuser");
        assert!(result.roles.contains(&"user".to_string()));

        // Authenticate with wrong password
        let wrong_credentials =
            Credentials::new("testuser".to_string(), "wrong_password".to_string());
        let result = auth_manager.authenticate(wrong_credentials).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_account_locking() {
        let mut config = SecurityConfig::default();
        config.max_failed_attempts = 2;
        let temp_dir = tempfile::TempDir::new().unwrap();
        let fs = FileSystem::new_test(temp_dir.path().to_path_buf());
        let auth_manager = AuthManager::new(config, fs);

        // Register user
        let password = SecureString::from_str("test_password_123");
        auth_manager
            .register_user("testuser".to_string(), password)
            .await
            .unwrap();

        // Make failed attempts
        for _ in 0..3 {
            let wrong_credentials =
                Credentials::new("testuser".to_string(), "wrong_password".to_string());
            let _ = auth_manager.authenticate(wrong_credentials).await;
        }

        // Account should be locked now
        let correct_credentials =
            Credentials::new("testuser".to_string(), "test_password_123".to_string());
        let result = auth_manager.authenticate(correct_credentials).await;
        assert!(matches!(result, Err(AuthError::AccountLocked { .. })));
    }
}
