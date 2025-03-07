use std::io::{self, Write};
use std::io::Read;
use vault_lib::yubikey::{list_yubikeys, encrypt_with_yubikey, authenticate_with_yubikey, generate_yubikey_challenge};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("YubiKey Tool");
    println!("------------");
    
    // List all YubiKeys
    println!("Scanning for YubiKeys...");
    
    match list_yubikeys() {
        Ok(keys) => {
            if keys.is_empty() {
                println!("No YubiKeys detected. Please insert a YubiKey and try again.");
                return Ok(());
            }
            
            println!("Found {} YubiKey(s):", keys.len());
            for (i, key) in keys.iter().enumerate() {
                println!("{}. {} (Serial: {:?}, Version: {:?}, Form Factor: {})",
                         i + 1,
                         key.name,
                         key.serial,
                         key.version,
                         key.form_factor);
            }
            
            // Select a YubiKey
            print!("\nSelect a YubiKey (1-{}): ", keys.len());
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            
            let index = input.trim().parse::<usize>().unwrap_or(0);
            if index < 1 || index > keys.len() {
                println!("Invalid selection.");
                return Ok(());
            }
            
            let selected_key = &keys[index - 1];
            println!("Selected: {} (Serial: {:?})", selected_key.name, selected_key.serial);
            
            if selected_key.serial.is_none() {
                println!("Error: Selected YubiKey has no serial number.");
                return Ok(());
            }
            
            // Main menu
            loop {
                println!("\nChoose an operation:");
                println!("1. Generate challenge and authenticate");
                println!("2. Encrypt text");
                println!("3. Exit");
                
                print!("Selection: ");
                io::stdout().flush()?;
                
                let mut operation = String::new();
                io::stdin().read_line(&mut operation)?;
                
                match operation.trim().parse::<usize>().unwrap_or(0) {
                    1 => {
                        // Authentication
                        println!("Generating challenge...");
                        let challenge = generate_yubikey_challenge()?;
                        println!("Challenge: {}", challenge);
                        
                        println!("Authenticating with YubiKey...");
                        println!("When prompted, enter your PIN");
                        
                        match authenticate_with_yubikey(std::sync::Mutex::new(()), selected_key.serial.unwrap(), &challenge) {
                            Ok(result) => {
                                if result {
                                    println!("Authentication successful! ✅");
                                } else {
                                    println!("Authentication failed! ❌");
                                }
                            },
                            Err(err) => println!("Authentication error: {}", err),
                        }
                    },
                    2 => {
                        // Encryption
                        println!("Enter text to encrypt (press Ctrl+D on a new line when done):");
                        let mut text = String::new();
                        io::stdin().read_to_string(&mut text)?;
                        
                        println!("Encrypting text with YubiKey...");
                        println!("When prompted, enter your PIN");
                        
                        match encrypt_with_yubikey(std::sync::Mutex::new(()), selected_key.serial.unwrap(), &text) {
                            Ok(encrypted) => {
                                println!("Encrypted result:");
                                println!("{}", encrypted);
                            },
                            Err(err) => println!("Encryption error: {}", err),
                        }
                    },
                    3 => {
                        println!("Exiting...");
                        break;
                    },
                    _ => println!("Invalid option."),
                }
            }
        },
        Err(err) => {
            println!("Error listing YubiKeys: {}", err);
        }
    }
    
    Ok(())
}
