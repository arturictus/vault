use std::path::{Path, PathBuf};

use crate::file_system::FileSystem;

pub struct Vault {
    pub name: String,
    pub fs: FileSystem
}

impl Vault {
    pub fn new(name: String, fs: FileSystem) -> Self {
        Self { name, fs }
    }
    pub fn fs(&self) -> FileSystem {
        self.fs.clone()
    }
    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn path(&self) -> PathBuf {   
        Path::new(&self.fs().vault_folder(self.name())).to_path_buf()
    }
    pub fn secret_path(&self, id: &str) -> PathBuf {
        self.path().join(format!("{}.enc", id)).to_path_buf()
    }

    pub fn pk_path(&self) -> PathBuf {
        let fs = self.fs();
        Path::new(&fs.pk_for_vault(self.name())).to_path_buf()
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    fn setup() -> Vault {
        let temp_dir = TempDir::new().unwrap();
        let fs = FileSystem::new_test(temp_dir.path().to_path_buf());
        Vault::new("test".to_string(), fs)
    }

    #[test]
    fn test_vault_for_tests() {
        let vault= setup();
        assert_eq!(vault.name(), "test");
        let path = vault.path();
        assert_eq!(path, vault.fs().root().join("vaults/test.vault"));
        // assert!(path.ends_with("vaults/test.vault"), 
        //     "Expected path to end with 'vaults/test.vault', got '{}'", 
        //     path.display()
        // );
    }
}
