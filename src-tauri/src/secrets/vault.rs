use std::path::{Path, PathBuf};

use crate::file_system::FileSystem;
use crate::file_system::DefaultFileSystem;

pub struct Vault {
    pub name: String 
}

impl Vault {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

pub trait VaultFs<T: FileSystem> {
    fn fs(&self) -> T;
    fn name(&self) -> &String;
    fn path(&self) -> PathBuf {   
        Path::new(&self.fs().vault_folder(self.name())).to_path_buf()
    }
    fn secret_path(&self, id: &str) -> PathBuf {
        self.path().join(format!("{}.enc", id)).to_path_buf()
    }

    fn pk_path(&self) -> PathBuf {
        let fs = self.fs();
        Path::new(&fs.pk_for_vault(self.name())).to_path_buf()
    }
}

impl VaultFs<DefaultFileSystem> for Vault {
    fn fs(&self) -> DefaultFileSystem {
        DefaultFileSystem::default()
    }
    fn name(&self) -> &String {
        &self.name
    }
}



#[cfg(test)]
use crate::file_system::TestFileSystem;
#[cfg(test)]
pub struct TestVault {
    name: String,
    fs: TestFileSystem
}

#[cfg(test)]
impl TestVault {
    pub fn new(name: String) -> Self {
        Self { name,  fs: TestFileSystem::default() }
    }
}

#[cfg(test)]
impl VaultFs<TestFileSystem> for TestVault {
    fn fs(&self) -> TestFileSystem {
        self.fs.clone()
    }
    fn name(&self) -> &String {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> TestVault {
        TestVault::new("test".to_string())
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
