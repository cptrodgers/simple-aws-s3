use std::fmt;

use hmac::crypto_mac::InvalidKeyLength;

#[derive(Debug)]
pub enum Error {
    SignError(String),
    RequestError(reqwest::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            Self::SignError(msg) => format!("Sign Error: {}", msg),
            Self::RequestError(e) => format!("Execute Request Error: {}", e),
        };
        write!(f, "{}", msg)
    }
}

impl From<InvalidKeyLength> for Error {
    fn from(e: InvalidKeyLength) -> Self {
        Self::SignError(e.to_string())
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Self::RequestError(e)
    }
}
