use thiserror::Error;
pub type Result<T> = core::result::Result<T, Error>;

#[derive(Error, Debug, serde::Serialize)]
pub enum Error {
    Rsa(String),
    Base64(String),
    BadUTF8(String),
    Io(String),
}

// --- Rsa errors
impl From<rsa::errors::Error> for Error {
    fn from(e: rsa::errors::Error) -> Self {
        Error::Rsa(e.to_string())
    }
}

impl From<base64::DecodeError> for Error {
    fn from(e: base64::DecodeError) -> Self {
        Error::Base64(e.to_string())
    }
}

impl From<rsa::pkcs8::Error> for Error {
    fn from(e: rsa::pkcs8::Error) -> Self {
        Error::Rsa(e.to_string())
    }
}
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e.to_string())
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(e: std::string::FromUtf8Error) -> Self {
        Error::BadUTF8(e.to_string())
    }
}

impl From<rsa::pkcs8::spki::Error> for Error {
    fn from(e: rsa::pkcs8::spki::Error) -> Self {
        Error::Rsa(e.to_string())
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}
