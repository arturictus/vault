use std::fmt;
use std::sync::Mutex;
use crate::file_system::FileSystem;

#[derive(Default)]
pub struct ProductionState {
    master_password: Option<String>,
    authenticated: bool,
    fs: FileSystem,
}

#[cfg(test)]
pub struct TestState {
    master_password: Option<String>,
    authenticated: bool,
    fs: FileSystem,
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
        use crate::master_password::MasterPassword;
        // Initialize the empty state
        let mut state = Self::new_unauthenticated_test();
        // Save the master password and set authenticated to true
        MasterPassword::save(&mut state, password, None).unwrap();
        state
    }
    
    #[cfg(test)]
    pub fn new_unauthenticated_test() -> Self {
        use std::fs;
        // Initialize a temp directory for testing
        let temp_dir = tempfile::TempDir::new().unwrap();
        let fs = FileSystem::new_test(temp_dir.path().to_path_buf());
        fs::create_dir_all(fs.root()).unwrap();
        // Initialize the empty state
        AppState::Test(TestState {
            master_password: None,
            authenticated: false,
            fs,
            _temp_dir: temp_dir,
        })
    }

    #[cfg(test)]
    pub fn new_tauri_test() -> Mutex<AppState> {
        let state = Self::new_test("password");
        Mutex::new(state)
    }

    pub fn master_password(&self) -> Option<String> {
        match self {
            AppState::Production(state) => state.master_password.clone(),
            #[cfg(test)]
            AppState::Test(state) => state.master_password.clone(),
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

    pub fn file_system(&self) -> &FileSystem {
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
