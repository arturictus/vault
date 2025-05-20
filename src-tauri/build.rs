use std::env;
use std::process::Command;

fn main() {
    
    // Only run the checks if the yubikey feature is enabled
    // if cfg!(feature = "yubikey") {
        println!("cargo:rerun-if-changed=build.rs");
        
        // Check if we're on a supported platform
        let os = env::var("CARGO_CFG_TARGET_OS").unwrap();
        
        match os.as_str() {
            "macos" => check_opensc_macos(),
            "linux" => check_opensc_linux(),
            "windows" => check_opensc_windows(),
            _ => println!("cargo:warning=YubiKey support may not work on this platform: {}", os),
        }
    // }
    tauri_build::build();
}

fn check_opensc_macos() {
    // Check Homebrew installation
    if Command::new("brew").arg("--version").status().is_ok() {
        let output = Command::new("brew")
            .args(&["list", "opensc"])
            .output()
            .expect("Failed to execute brew command");
        
        if !output.status.success() {
            println!("cargo:warning=OpenSC not found. Install it with: brew install opensc");
        } else {
            // Get OpenSC installation path
            let opensc_path = Command::new("brew")
                .args(&["--prefix", "opensc"])
                .output()
                .expect("Failed to get OpenSC path");
                
            if opensc_path.status.success() {
                let path_str = String::from_utf8_lossy(&opensc_path.stdout).trim().to_string();
                println!("cargo:rustc-env=OPENSC_PATH={}/lib/pkcs11/opensc-pkcs11.so", path_str);
            }
        }
    } else {
        println!("cargo:warning=Homebrew not found. Please install OpenSC manually.");
    }
}

fn check_opensc_linux() {
    // Check for common package managers
    if Command::new("apt-get").arg("--version").status().is_ok() {
        println!("cargo:warning=If OpenSC is not installed, install it with: sudo apt-get install opensc");
    } else if Command::new("dnf").arg("--version").status().is_ok() {
        println!("cargo:warning=If OpenSC is not installed, install it with: sudo dnf install opensc");
    }
}

fn check_opensc_windows() {
    println!("cargo:warning=On Windows, download OpenSC installer from https://github.com/OpenSC/OpenSC/releases");
}
