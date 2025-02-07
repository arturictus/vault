use dirs;

pub fn application_data_directory() -> String {
    dirs::home_dir().map(|home| home.join(".vault").join("app-data").to_string_lossy().to_string()).unwrap()
}

pub fn pks_folder() -> String {
    dirs::home_dir().map(|home| home.join(".vault").join("pks").to_string_lossy().to_string()).unwrap()
}

pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    let app_dir = application_data_directory();
    let pks_dir = pks_folder();
    std::fs::create_dir_all(&app_dir)?;
    std::fs::create_dir_all(&pks_dir)?;
    Ok(())
}