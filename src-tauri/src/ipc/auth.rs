use crate::auth::{AuthManager, SessionManager, SessionMiddleware, SessionToken};
use crate::security::SecureString;
use crate::Result;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn login(
    auth_manager: State<'_, Arc<AuthManager>>,
    session_manager: State<'_, Arc<SessionManager>>,
    username: String,
    password: String,
    user_agent: Option<String>,
) -> Result<SessionToken> {
    // Create middleware instance on the fly or use a managed one
    let middleware = SessionMiddleware::new(
        session_manager.inner().clone(),
        auth_manager.inner().clone(),
    );

    // We don't have IP address in Tauri easily, so passing None
    let token = middleware
        .login(username, password, None, user_agent)
        .await?;

    Ok(token)
}

#[tauri::command]
pub async fn register(
    auth_manager: State<'_, Arc<AuthManager>>,
    username: String,
    password: String,
) -> Result<String> {
    let secure_password = SecureString::from_str(&password);
    let user_id = auth_manager
        .register_user(username, secure_password)
        .await?;
    Ok(user_id.to_string())
}

#[tauri::command]
pub async fn logout(session_manager: State<'_, Arc<SessionManager>>, token: String) -> Result<()> {
    let session_token = SessionToken::from_string(token);
    session_manager.destroy_session(&session_token).await?;
    Ok(())
}

#[tauri::command]
pub async fn validate_session(
    session_manager: State<'_, Arc<SessionManager>>,
    token: String,
) -> Result<crate::auth::SessionData> {
    let session_token = SessionToken::from_string(token);
    let session = session_manager.validate_session(&session_token).await?;
    Ok(session)
}

#[tauri::command]
pub async fn renew_session(
    session_manager: State<'_, Arc<SessionManager>>,
    token: String,
) -> Result<()> {
    let session_token = SessionToken::from_string(token);
    session_manager.renew_session(&session_token).await?;
    Ok(())
}
