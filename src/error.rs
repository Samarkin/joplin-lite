use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

pub enum JoplinError {
    Io(std::io::Error),
    SerdeJson(serde_json::Error),
    Decode(String),
    Usage,
}

impl Error for JoplinError {}

impl Display for JoplinError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "I/O error: {}", err),
            Self::SerdeJson(err) => write!(f, "JSON error: {}", err),
            Self::Decode(err) => write!(f, "decode error: {}", err),
            Self::Usage => write!(f, "usage: joplin-lite <path>"),
        }
    }
}

impl Debug for JoplinError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self, f)
    }
}

impl From<std::io::Error> for JoplinError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for JoplinError {
    fn from(value: serde_json::Error) -> Self {
        Self::SerdeJson(value)
    }
}
