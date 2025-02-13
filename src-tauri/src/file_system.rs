use std::path::{Path, PathBuf};
use std::sync::Arc;

pub trait FileSystem: Send + Sync {
    fn app_data_directory(&self) -> PathBuf;
    fn vaults_folder(&self) -> PathBuf;
    fn pk_for_vault(&self, vault_name: &str) -> PathBuf;
    fn master_password(&self) -> PathBuf;
    fn master_pk(&self) -> PathBuf;
    fn master_pub(&self) -> PathBuf;
    fn vault_folder(&self, vault_name: &str) -> PathBuf;
    fn create_vault_folder(&self, vault_name: &str) -> Result<(), Box<dyn std::error::Error>>;
    fn init(&self) -> Result<(), Box<dyn std::error::Error>>;
}

pub struct DefaultFileSystem {
    root: PathBuf,
}

impl Default for DefaultFileSystem {
    fn default() -> Self {
        Self {
            root: dirs::home_dir().map(|home| home.join(".vault")).unwrap()
        }
    }
}

impl FileSystem for DefaultFileSystem {
    fn app_data_directory(&self) -> PathBuf {
        self.root.join("app-data")
    }

    fn vaults_folder(&self) -> PathBuf {
        self.root.join("vaults")
    }

    fn pk_for_vault(&self, vault_name: &str) -> PathBuf {
        self.vault_folder(vault_name).join("private_key")
    }

    fn master_password(&self) -> PathBuf {
        self.root.join("master_password.enc")
    }

    fn master_pk(&self) -> PathBuf {
        self.root.join("rsa_master_pk.enc")
    }

    fn master_pub(&self) -> PathBuf {
        self.root.join("rsa_master_pub")
    }

    fn vault_folder(&self, vault_name: &str) -> PathBuf {
        let vault_folder = format!("{}.vault", vault_name);
        self.vaults_folder().join(vault_folder)
    }

    fn create_vault_folder(&self, vault_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let folder = self.vault_folder(vault_name);
        std::fs::create_dir_all(&folder)?;
        Ok(())
    }
    fn init(&self) -> Result<(), Box<dyn std::error::Error>> {
        let app_dir = self.app_data_directory();
        std::fs::create_dir_all(&app_dir)?;
        let vaults_dir = self.vaults_folder();
        std::fs::create_dir_all(&vaults_dir)?;
        let default_vault_dir = self.vault_folder("default");
        std::fs::create_dir_all(&default_vault_dir)?;
        let pk_for_default_vault = self.pk_for_vault("default");
        let pk_for_default_path = Path::new(&pk_for_default_vault);
        if !pk_for_default_path.exists() {
            crate::encrypt::create_pk_at_path(&pk_for_default_path)?;
        }
        Ok(())
    }
}

#[cfg(test)]
pub struct TestFileSystem {
    root: PathBuf,
}

#[cfg(test)]
impl TestFileSystem {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }
}

#[cfg(test)]
impl Default for TestFileSystem {
    fn default() -> Self {
        let temp_dir = tempfile::tempdir().unwrap();
        Self::new(temp_dir.path().to_path_buf())
    }
}

#[cfg(test)]
impl FileSystem for TestFileSystem {
    fn app_data_directory(&self) -> PathBuf {
        self.root.join("app-data")
    }

    fn vaults_folder(&self) -> PathBuf {
        self.root.join("vaults")
    }

    fn pk_for_vault(&self, vault_name: &str) -> PathBuf {
        self.vault_folder(vault_name).join("private_key")
    }

    fn master_password(&self) -> PathBuf {
        self.root.join("master_password.enc")
    }

    fn master_pk(&self) -> PathBuf {
        self.root.join("rsa_master_pk.enc")
    }

    fn master_pub(&self) -> PathBuf {
        self.root.join("rsa_master_pub")
    }

    fn vault_folder(&self, vault_name: &str) -> PathBuf {
        let vault_folder = format!("{}.vault", vault_name);
        self.vaults_folder().join(vault_folder)
    }

    fn create_vault_folder(&self, vault_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let folder = self.vault_folder(vault_name);
        std::fs::create_dir_all(&folder)?;
        Ok(())
    }
    fn init(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_file_system() {
        let temp_dir = TempDir::new().unwrap();
        let fs = TestFileSystem::new(temp_dir.path().to_path_buf());
        
        assert_eq!(fs.app_data_directory(), temp_dir.path().join("app-data"));
        assert_eq!(fs.vaults_folder(), temp_dir.path().join("vaults"));
        assert_eq!(
            fs.vault_folder("test"), 
            temp_dir.path().join("vaults").join("test.vault")
        );
    }
}