use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct FileSystem {
    root: PathBuf,
}

impl Default for FileSystem {
    fn default() -> Self {
        let root = dirs::home_dir().map(|home| home.join(".vault")).unwrap();
        Self { root }
    }
}

impl FileSystem {

    #[cfg(test)] 
    pub fn new_test(temp_dir: PathBuf) -> Self {

        let inst = Self {
            root: temp_dir.join(".vault"),
        };
        inst.init().unwrap();
        inst
    }

    pub fn root(&self) -> PathBuf {
        self.root.clone()
    }

    pub fn app_data_directory(&self) -> PathBuf {
        self.root().join("app-data")
    }

    pub fn vaults_folder(&self) -> PathBuf {
        self.root().join("vaults")
    }

    pub fn pk_for_vault(&self, vault_name: &str) -> PathBuf {
        self.vault_folder(vault_name).join("private_key")
    }

    pub fn master_password(&self) -> PathBuf {
        self.root().join("master_password.enc")
    }

    pub fn master_pk(&self) -> PathBuf {
        self.root().join("rsa_master_pk.enc")
    }

    pub fn master_pub(&self) -> PathBuf {
        self.root().join("rsa_master_pub")
    }

    pub fn vault_folder(&self, vault_name: &str) -> PathBuf {
        let vault_folder = format!("{}.vault", vault_name);
        self.vaults_folder().join(vault_folder)
    }

    // TODO: Change Result to crate::Error::TauriInit
    pub fn init(&self) -> Result<(), Box<dyn std::error::Error>> {
        let app_dir = self.app_data_directory();
        std::fs::create_dir_all(&app_dir)?;
        let vaults_dir = self.vaults_folder();
        std::fs::create_dir_all(&vaults_dir)?;
        let default_vault_dir = self.vault_folder("default");
        std::fs::create_dir_all(&default_vault_dir)?;
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    fn setup() -> FileSystem {
        let temp_dir = TempDir::new().unwrap();
        FileSystem::new_test(temp_dir.path().to_path_buf())
    }
    #[test]
    fn test_file_system() {
        let fs = setup();
        let temp_dir = fs.root();

        assert_eq!(fs.app_data_directory(), temp_dir.join("app-data"));
        assert_eq!(fs.vaults_folder(), temp_dir.join("vaults"));
        assert_eq!(
            fs.vault_folder("test"),
            temp_dir.join("vaults").join("test.vault")
        );
    }
}
