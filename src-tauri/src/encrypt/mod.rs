pub mod error;
mod rsa;
mod aes;
mod master_password;
pub use error::{Error, Result};
pub use aes::AES;
pub use rsa::{RsaKeyPair, PublicKey};
pub use master_password::MasterPassword;

// TODO: Implement Encrypt trait
// pub trait Encrypt {
//     fn encrypt(&self, data: impl Into<String>) -> Result<String>;
//     fn decrypt(&self, data: impl Into<String>) -> Result<String>;
//     fn encrypt_u8(&self, data: &[u8]) -> Result<Vec<u8>>;
//     fn decrypt_u8(&self, data: &[u8]) -> Result<Vec<u8>>;
// }