//! Core session management with secure tokens and timeouts

use std::sync::Arc;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use dashmap::DashMap;

use super::auth_manager::{AuthManager, AuthenticationResult};
use super::crypto::{CryptoManager, EncryptedData};
use crate::security::{SecureBytes, SecurityConfig, SecurityUtils};
use crate::error::{SessionError, SessionResult};

/// Session token that uniquely identifies a session
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionToken {
    pub token: String,
}

impl SessionToken {
    /// Create a new random session token
    pub fn new() -> SessionResult<Self> {
        let random_string = SecurityUtils::generate_random_string(64)
            .map_err(|e| SessionError::CreationFailed {
                reason: format!("Failed to generate token: {}", e),
            })?;
        
        Ok(Self {
            token: random_string.expose(|s| s.clone()),
        })
    }
    
    /// Create from existing token string
    pub fn from_string(token: String) -> Self {
        Self { token }
    }
}

impl std::fmt::Display for SessionToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.token)
    }
}

/// Session data containing user information and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub username: String,
    pub roles: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub is_renewed: bool,
}

impl SessionData {
    /// Create new session data
    pub fn new(
        auth_result: AuthenticationResult,
        timeout_secs: u64,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            session_id: Uuid::new_v4(),
            user_id: auth_result.user_id,
            username: auth_result.username,
            roles: auth_result.roles,
            created_at: now,
            last_accessed: now,
            expires_at: now + Duration::seconds(timeout_secs as i64),
            ip_address,
            user_agent,
            is_renewed: false,
        }
    }
    
    /// Check if the session is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
    
    /// Check if the session needs renewal
    pub fn needs_renewal(&self, renewal_interval_secs: u64) -> bool {
        let renewal_time = self.last_accessed + Duration::seconds(renewal_interval_secs as i64);
        Utc::now() > renewal_time
    }
    
    /// Update last accessed time
    pub fn touch(&mut self) {
        self.last_accessed = Utc::now();
    }
    
    /// Renew the session with new expiration time
    pub fn renew(&mut self, timeout_secs: u64) {
        let now = Utc::now();
        self.last_accessed = now;
        self.expires_at = now + Duration::seconds(timeout_secs as i64);
        self.is_renewed = true;
    }
    
    /// Check if user has a specific role
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.contains(&role.to_string())
    }
}

/// Encrypted session storage
#[derive(Debug, Clone)]
struct EncryptedSession {
    encrypted_data: EncryptedData,
    created_at: DateTime<Utc>,
}

/// Session manager for handling user sessions
pub struct SessionManager {
    sessions: Arc<DashMap<SessionToken, EncryptedSession>>,
    user_sessions: Arc<DashMap<Uuid, Vec<SessionToken>>>,
    crypto: CryptoManager,
    encryption_key: SecureBytes,
    config: SecurityConfig,
}

impl SessionManager {
    /// Create a new session manager
    pub async fn new(config: SecurityConfig) -> SessionResult<Self> {
        let crypto = CryptoManager::new();
        let encryption_key = crypto.generate_key()
            .map_err(|e| SessionError::CreationFailed {
                reason: format!("Failed to generate encryption key: {}", e),
            })?;
        
        Ok(Self {
            sessions: Arc::new(DashMap::new()),
            user_sessions: Arc::new(DashMap::new()),
            crypto,
            encryption_key,
            config,
        })
    }
    
    /// Create a new session for an authenticated user
    pub async fn create_session(
        &self,
        auth_result: AuthenticationResult,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> SessionResult<SessionToken> {
        // Check concurrent session limit
        if let Some(user_tokens) = self.user_sessions.get(&auth_result.user_id) {
            if user_tokens.len() >= self.config.max_concurrent_sessions as usize {
                // Remove oldest session
                if let Some(oldest_token) = user_tokens.first() {
                    self.destroy_session(oldest_token).await?;
                }
            }
        }
        
        // Create session data
        let session_data = SessionData::new(
            auth_result,
            self.config.session_timeout_secs,
            ip_address,
            user_agent,
        );
        
        // Serialize and encrypt session data
        let serialized = serde_json::to_vec(&session_data)?;
        let encrypted = self.crypto.encrypt(&self.encryption_key, &serialized)
            .map_err(SessionError::CryptoError)?;
        
        // Generate session token
        let token = SessionToken::new()?;
        
        // Store encrypted session
        let encrypted_session = EncryptedSession {
            encrypted_data: encrypted,
            created_at: Utc::now(),
        };
        
        self.sessions.insert(token.clone(), encrypted_session);
        
        // Track user sessions
        self.user_sessions
            .entry(session_data.user_id)
            .or_insert_with(Vec::new)
            .push(token.clone());
        
        Ok(token)
    }
    
    /// Validate and retrieve session data
    pub async fn validate_session(&self, token: &SessionToken) -> SessionResult<SessionData> {
        let encrypted_session = self.sessions.get(token)
            .ok_or_else(|| SessionError::SessionNotFound {
                session_id: token.token.clone(),
            })?;
        
        // Decrypt session data
        let decrypted = self.crypto.decrypt(&self.encryption_key, &encrypted_session.encrypted_data)
            .map_err(SessionError::CryptoError)?;
        
        let mut session_data: SessionData = serde_json::from_slice(&decrypted)?;
        
        // Check if session is expired
        if session_data.is_expired() {
            self.destroy_session(token).await?;
            return Err(SessionError::SessionExpired {
                session_id: token.token.clone(),
            });
        }
        
        // Update last accessed time
        session_data.touch();
        
        // Re-encrypt and store updated session data
        let serialized = serde_json::to_vec(&session_data)?;
        let encrypted = self.crypto.encrypt(&self.encryption_key, &serialized)
            .map_err(SessionError::CryptoError)?;
        
        let updated_session = EncryptedSession {
            encrypted_data: encrypted,
            created_at: encrypted_session.created_at,
        };
        
        self.sessions.insert(token.clone(), updated_session);
        
        Ok(session_data)
    }
    
    /// Renew a session with new expiration time
    pub async fn renew_session(&self, token: &SessionToken) -> SessionResult<()> {
        let mut session_data = self.validate_session(token).await?;
        
        // Renew the session
        session_data.renew(self.config.session_timeout_secs);
        
        // Re-encrypt and store updated session data
        let serialized = serde_json::to_vec(&session_data)?;
        let encrypted = self.crypto.encrypt(&self.encryption_key, &serialized)
            .map_err(SessionError::CryptoError)?;
        
        let updated_session = EncryptedSession {
            encrypted_data: encrypted,
            created_at: Utc::now(),
        };
        
        self.sessions.insert(token.clone(), updated_session);
        
        Ok(())
    }
    
    /// Destroy a session
    pub async fn destroy_session(&self, token: &SessionToken) -> SessionResult<()> {
        if let Some((_, encrypted_session)) = self.sessions.remove(token) {
            // Decrypt to get user ID for cleanup
            if let Ok(decrypted) = self.crypto.decrypt(&self.encryption_key, &encrypted_session.encrypted_data) {
                if let Ok(session_data) = serde_json::from_slice::<SessionData>(&decrypted) {
                    // Remove from user sessions tracking
                    if let Some(mut user_tokens) = self.user_sessions.get_mut(&session_data.user_id) {
                        user_tokens.retain(|t| t != token);
                        if user_tokens.is_empty() {
                            drop(user_tokens);
                            self.user_sessions.remove(&session_data.user_id);
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Destroy all sessions for a user
    pub async fn destroy_user_sessions(&self, user_id: Uuid) -> SessionResult<()> {
        if let Some((_, tokens)) = self.user_sessions.remove(&user_id) {
            for token in tokens {
                self.sessions.remove(&token);
            }
        }
        
        Ok(())
    }
    
    /// Clean up expired sessions
    pub async fn cleanup_expired_sessions(&self) -> SessionResult<usize> {
        let mut expired_tokens = Vec::new();
        
        // Find expired sessions
        for entry in self.sessions.iter() {
            let token = entry.key();
            let encrypted_session = entry.value();
            
            if let Ok(decrypted) = self.crypto.decrypt(&self.encryption_key, &encrypted_session.encrypted_data) {
                if let Ok(session_data) = serde_json::from_slice::<SessionData>(&decrypted) {
                    if session_data.is_expired() {
                        expired_tokens.push(token.clone());
                    }
                }
            }
        }
        
        // Remove expired sessions
        let count = expired_tokens.len();
        for token in expired_tokens {
            self.destroy_session(&token).await?;
        }
        
        Ok(count)
    }
    
    /// Get session count for a user
    pub async fn get_user_session_count(&self, user_id: Uuid) -> usize {
        self.user_sessions.get(&user_id)
            .map(|tokens| tokens.len())
            .unwrap_or(0)
    }
    
    /// Get total active session count
    pub async fn get_total_session_count(&self) -> usize {
        self.sessions.len()
    }
    
    /// List all sessions for a user (admin function)
    pub async fn list_user_sessions(&self, user_id: Uuid) -> SessionResult<Vec<SessionData>> {
        let mut sessions = Vec::new();
        
        if let Some(tokens) = self.user_sessions.get(&user_id) {
            for token in tokens.iter() {
                if let Ok(session_data) = self.validate_session(token).await {
                    sessions.push(session_data);
                }
            }
        }
        
        Ok(sessions)
    }
}

/// Session middleware for automatic session management
pub struct SessionMiddleware {
    session_manager: Arc<SessionManager>,
    auth_manager: Arc<AuthManager>,
}

impl SessionMiddleware {
    /// Create new session middleware
    pub fn new(session_manager: Arc<SessionManager>, auth_manager: Arc<AuthManager>) -> Self {
        Self {
            session_manager,
            auth_manager,
        }
    }
    
    /// Authenticate and create session
    pub async fn login(
        &self,
        username: String,
        password: String,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> SessionResult<SessionToken> {
        use super::auth_manager::Credentials;
        
        let credentials = Credentials::new(username, password);
        let auth_result = self.auth_manager.authenticate(credentials).await
            .map_err(SessionError::AuthError)?;
        
        self.session_manager.create_session(auth_result, ip_address, user_agent).await
    }
    
    /// Logout and destroy session
    pub async fn logout(&self, token: &SessionToken) -> SessionResult<()> {
        self.session_manager.destroy_session(token).await
    }
    
    /// Validate session and return user data
    pub async fn validate(&self, token: &SessionToken) -> SessionResult<SessionData> {
        self.session_manager.validate_session(token).await
    }
    
    /// Start background cleanup task
    pub async fn start_cleanup_task(session_manager: Arc<SessionManager>) {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300)); // 5 minutes
        
        loop {
            interval.tick().await;
            if let Ok(count) = session_manager.cleanup_expired_sessions().await {
                if count > 0 {
                    println!("Cleaned up {} expired sessions", count);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::auth_manager::AuthManager;
    use crate::security::{SecurityConfig, SecureString};
    
    #[tokio::test]
    async fn test_session_creation() {
        let config = SecurityConfig::default();
        let session_manager = SessionManager::new(config).await.unwrap();
        
        let auth_result = AuthenticationResult {
            user_id: Uuid::new_v4(),
            username: "testuser".to_string(),
            roles: vec!["user".to_string()],
            authenticated_at: Utc::now(),
        };
        
        let token = session_manager.create_session(
            auth_result,
            Some("127.0.0.1".to_string()),
            Some("Test Agent".to_string()),
        ).await.unwrap();
        
        assert!(!token.token.is_empty());
    }
    
    #[tokio::test]
    async fn test_session_validation() {
        let config = SecurityConfig::default();
        let session_manager = SessionManager::new(config).await.unwrap();
        
        let auth_result = AuthenticationResult {
            user_id: Uuid::new_v4(),
            username: "testuser".to_string(),
            roles: vec!["user".to_string()],
            authenticated_at: Utc::now(),
        };
        
        let token = session_manager.create_session(auth_result.clone(), None, None).await.unwrap();
        let session_data = session_manager.validate_session(&token).await.unwrap();
        
        assert_eq!(session_data.user_id, auth_result.user_id);
        assert_eq!(session_data.username, auth_result.username);
        assert!(!session_data.is_expired());
    }
    
    #[tokio::test]
    async fn test_session_destruction() {
        let config = SecurityConfig::default();
        let session_manager = SessionManager::new(config).await.unwrap();
        
        let auth_result = AuthenticationResult {
            user_id: Uuid::new_v4(),
            username: "testuser".to_string(),
            roles: vec!["user".to_string()],
            authenticated_at: Utc::now(),
        };
        
        let token = session_manager.create_session(auth_result, None, None).await.unwrap();
        session_manager.destroy_session(&token).await.unwrap();
        
        let result = session_manager.validate_session(&token).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_session_middleware() {
        let config = SecurityConfig::default();
        let auth_manager = Arc::new(AuthManager::new(config.clone()));
        let session_manager = Arc::new(SessionManager::new(config).await.unwrap());
        
        // Register a user
        let password = SecureString::from_str("test_password_123");
        auth_manager.register_user("testuser".to_string(), password).await.unwrap();
        
        let middleware = SessionMiddleware::new(session_manager, auth_manager);
        
        // Login
        let token = middleware.login(
            "testuser".to_string(),
            "test_password_123".to_string(),
            Some("127.0.0.1".to_string()),
            Some("Test Agent".to_string()),
        ).await.unwrap();
        
        // Validate
        let session_data = middleware.validate(&token).await.unwrap();
        assert_eq!(session_data.username, "testuser");
        
        // Logout
        middleware.logout(&token).await.unwrap();
        
        // Should fail after logout
        let result = middleware.validate(&token).await;
        assert!(result.is_err());
    }
}
