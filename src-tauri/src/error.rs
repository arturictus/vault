
use thiserror::Error;
use std::sync::PoisonError;
use std::sync::MutexGuard;
pub type Result<T> = core::result::Result<T, Error>;

#[derive(Error, Debug, serde::Serialize)]
pub enum Error {
  TauriInit(String),
  Custom(String),
  Encryption(String),
  Io(String),
}

impl core::fmt::Display for Error {
	fn fmt(
		&self,
		fmt: &mut core::fmt::Formatter,
	) -> core::result::Result<(), core::fmt::Error> {
		write!(fmt, "{self:?}")
	}
}


impl<T> From<PoisonError<MutexGuard<'_, T>>> for Error {
    fn from(_: PoisonError<MutexGuard<'_, T>>) -> Self {
        Error::TauriInit("Mutex lock poisoned".to_string())
    }
}

impl From<String> for Error {
    fn from(e: String) -> Self {
        Error::Custom(e)
    }
}

// // --- RSA errors
// impl From<rsa::errors::Error> for Error {
//     fn from(e: rsa::errors::Error) -> Self {
//         Error::Encryption(e.to_string())
//     }
// }

// impl From<base64::DecodeError> for Error {
//     fn from(e: base64::DecodeError) -> Self {
//         Error::Encryption(e.to_string())
//     }
// }

// impl From<rsa::pkcs8::Error> for Error {
//     fn from(e: rsa::pkcs8::Error) -> Self {
//         Error::Encryption(e.to_string())
//     }
// }
// impl From<std::io::Error> for Error {
//     fn from(e: std::io::Error) -> Self {
//         Error::Encryption(e.to_string())
//     }
    
// }

// impl From<std::string::FromUtf8Error> for Error {
//     fn from(e: std::string::FromUtf8Error) -> Self {
//         Error::Encryption(e.to_string())
//     }
// }

// impl From<rsa::pkcs8::spki::Error> for Error {
//     fn from(e: rsa::pkcs8::spki::Error) -> Self {
//         Error::Encryption(e.to_string())
//     }
// }
// END RSA