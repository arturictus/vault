
use std::fmt;
use std::sync::Mutex;

pub type State<'a> = tauri::State<'a, Mutex<AppState<DefaultFileSystem>>>;
use crate::file_system::{DefaultFileSystem, FileSystem, TestFileSystem};


#[derive(serde::Serialize)]
#[derive(Default)]
struct AppState<T: FileSystem> {
    pub master_password: Option<String>,
    pub authenticated: bool,
    pub fs: T
}

impl fmt::Debug for AppState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        serde_json::to_string(self)
            .map_err(|_e| fmt::Error)
            .and_then(|s| write!(f, "{}", s))
    }
}
#[cfg(test)]
use tempfile::TempDir;

#[cfg(test)]
struct TestState {
    state: Mutex<AppState>,
    _temp_dir: TempDir, // Keep temp_dir alive for test duration
}

#[cfg(test)]
impl TestState {
    fn new(password: &str) -> Self {
        let temp_dir = TempDir::new().unwrap();
        let fs = TestFileSystem::new(temp_dir.path().to_path_buf());

        // Initialize file system with test keys
        let encryptor = rsa::Encryptor::new().unwrap();
        fs::create_dir_all(fs.root()).unwrap();

        // Save test master key
        let pk = encryptor.private_key_pem().unwrap();
        fs::write(fs.master_pk(), pk).unwrap();

        // Create app state
        let state = AppState {
            master_password: Some(password.to_string()),
            authenticated: true,
        };

        Self {
            state: Mutex::new(state),
            _temp_dir: temp_dir,
        }
    }

    fn state(&self) -> State<'_, Mutex<AppState>> {
        State::from(&self.state)
    }
}
