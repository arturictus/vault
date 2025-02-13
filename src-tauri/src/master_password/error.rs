use thiserror::Error;
pub type Result<T> = core::result::Result<T, Error>;

#[derive(Error, Debug, serde::Serialize)]
pub enum Error {
    EncryptPassword(String),
    DecryptPassword(String),
    EncryptMod(String),
    Base64(String),
    StateLock(String),
    WrongPassword(String),
    Io(String)
}

impl From<crate::encrypt::Error> for Error {
    fn from(e: crate::encrypt::Error) -> Self {
        Error::EncryptMod(e.to_string())
    }
}

impl From<base64::DecodeError> for Error {
    fn from(e: base64::DecodeError) -> Self {
        Error::Base64(e.to_string())
    }
}
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e.to_string())
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}
