use std::fmt;
use std::sync::Mutex;
use crate::file_system::{DefaultFileSystem, FileSystem};
#[cfg(test)]
use crate::file_system::TestFileSystem;

pub trait AppStateCommon {
    type FS: FileSystem;
    fn master_password(&self) -> Option<&String>;
    fn is_authenticated(&self) -> bool;
    fn file_system(&self) -> &Self::FS;
}

#[derive(Default)]
pub struct ProductionState {
    master_password: Option<String>,
    authenticated: bool,
    fs: DefaultFileSystem,
}

#[cfg(test)]
pub struct TestState {
    master_password: Option<String>,
    authenticated: bool,
    fs: TestFileSystem,
    _temp_dir: tempfile::TempDir, // Keep temp_dir alive for test duration
}

pub enum AppState {
    Production(ProductionState),
    #[cfg(test)]
    Test(TestState),
}

impl Default for AppState {
    fn default() -> Self {
        AppState::Production(ProductionState::default())
    }
}

impl AppState {
    #[cfg(test)]
    pub fn new_test(password: &str) -> Self {
        use std::fs;
        use crate::encrypt::rsa;
        
        let temp_dir = tempfile::TempDir::new().unwrap();
        let fs = TestFileSystem::new(temp_dir.path().to_path_buf());

        // Initialize file system with test keys
        let encryptor = rsa::Encryptor::new().unwrap();
        fs::create_dir_all(fs.root()).unwrap();

        // Save test master key
        let pk = encryptor.private_key_pem().unwrap();
        fs::write(fs.master_pk(), pk).unwrap();

        AppState::Test(TestState {
            master_password: Some(password.to_string()),
            authenticated: true,
            fs,
            _temp_dir: temp_dir,
        })
    }

    pub fn master_password(&self) -> Option<&String> {
        match self {
            AppState::Production(state) => state.master_password.as_ref(),
            #[cfg(test)]
            AppState::Test(state) => state.master_password.as_ref(),
        }
    }

    pub fn set_master_password(&mut self, password: String) {
        match self {
            AppState::Production(state) => state.master_password = Some(password),
            #[cfg(test)]
            AppState::Test(state) => state.master_password = Some(password),
        }
    }

    pub fn is_authenticated(&self) -> bool {
        match self {
            AppState::Production(state) => state.authenticated,
            #[cfg(test)]
            AppState::Test(state) => state.authenticated,
        }
    }

    pub fn set_authenticated(&mut self, authenticated: bool) {
        match self {
            AppState::Production(state) => state.authenticated = authenticated,
            #[cfg(test)]
            AppState::Test(state) => state.authenticated = authenticated,
        }
    }

    pub fn file_system(&self) -> &dyn FileSystem {
        match self {
            AppState::Production(state) => &state.fs,
            #[cfg(test)]
            AppState::Test(state) => &state.fs,
        }
    }
}

impl fmt::Debug for AppState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppState::Production(state) => f.debug_struct("ProductionState")
                .field("master_password", &state.master_password)
                .field("authenticated", &state.authenticated)
                .finish(),
            #[cfg(test)]
            AppState::Test(state) => f.debug_struct("TestState")
                .field("master_password", &state.master_password)
                .field("authenticated", &state.authenticated)
                .finish(),
        }
    }
}

pub type State<'a> = tauri::State<'a, Mutex<AppState>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_production_state() {
        let state = AppState::default();
        assert!(!state.is_authenticated());
        assert!(state.master_password().is_none());
    }

    #[test]
    fn test_test_state() {
        let state = AppState::new_test("test-password");
        assert!(state.is_authenticated());
        assert_eq!(state.master_password().unwrap(), "test-password");
    }

    #[test]
    fn test_state_modifications() {
        let mut state = AppState::default();
        assert!(!state.is_authenticated());
        
        state.set_authenticated(true);
        assert!(state.is_authenticated());
        
        state.set_master_password("password".to_string());
        assert_eq!(state.master_password().unwrap(), "password");
    }
}
