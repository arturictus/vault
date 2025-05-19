pub fn setup_yubikey(serial: u32, pin: String) -> Result<()> {
    // Open YubiKey by serial number
    let serial = yubikey::Serial::from(serial);
    let mut yubikey = YubiKey::open_by_serial(serial)
        .map_err(|e| Error::YubiKeyError(format!("Failed to open YubiKey: {}", e)))?;

    // Verify PIN
    yubikey.verify_pin(pin.as_bytes())
        .map_err(|e| Error::YubiKeyError(format!("PIN verification failed: {}", e)))?;
}

#[cfg(test)]
mod test {
    use super::*;
    use yubikey::{
        YubiKey,
        piv::{AlgorithmId, SlotId, decrypt_data},
    };

    #[test]
    fn test_setup_yubikey() {
        let serial = 32233649;
        let pin = "123456".to_string();
        let result = setup_yubikey(serial, pin);
        println!("{:?}", result);
        assert!(result.is_ok());
    }
}