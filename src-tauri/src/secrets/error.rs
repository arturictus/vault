use thiserror::Error;
pub type Result<T> = core::result::Result<T, Error>;

#[derive(Error, Debug, serde::Serialize)]
pub enum Error {
    Json(String),
    EncryptMod(String),
    Io(String),
    AppStateLock(String),
    MasterPassword(String),
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Json(e.to_string())
    }
}
impl From<crate::encrypt::Error> for Error {
    fn from(e: crate::encrypt::Error) -> Self {
        Error::EncryptMod(e.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e.to_string())
    }
}

impl From<crate::master_password::Error> for Error {
    fn from(e: crate::master_password::Error) -> Self {
        Error::MasterPassword(e.to_string())
    }
    
}

impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "secrets::{self:?}")
    }
}
