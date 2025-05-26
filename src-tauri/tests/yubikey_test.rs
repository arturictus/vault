#[cfg(test)]
mod yubikey_tests {
    use std::sync::Mutex;
    use vault_lib::AppState;
    use vault_lib::yubikey::{list_yubikeys, generate_yubikey_challenge};
    use base64::Engine;

    // This test will only pass if a YubiKey is connected
    #[tokio::test]
    #[ignore] // Ignore by default as it requires hardware
    async fn test_list_yubikeys() {
        let keys = list_yubikeys().unwrap();
        println!("Found YubiKeys: {:?}", keys);
        assert!(!keys.is_empty(), "No YubiKeys detected. Make sure at least one YubiKey is inserted.");
    }

    #[tokio::test]
    async fn test_generate_challenge() {
        let challenge = generate_yubikey_challenge().unwrap();
        println!("Generated challenge: {}", challenge);
        assert!(!challenge.is_empty(), "Challenge should not be empty");
        
        // Decode the base64 challenge to ensure it's valid
        let decoded = base64::engine::general_purpose::STANDARD.decode(challenge.as_bytes()).unwrap();
        assert_eq!(decoded.len(), 32, "Challenge should be 32 bytes (256 bits)");
    }
}
