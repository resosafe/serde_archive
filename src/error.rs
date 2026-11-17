use std::fmt::{self, Display};
use std::io;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Message(String),
    UnsupportedOperation(String),
    Io(String),
    InvalidPath(String),
    MissingField(String),
    Parse(String),
}

impl Error {
    pub fn message(msg: impl Display) -> Self {
        Error::Message(msg.to_string())
    }

    pub fn unsupported(op: impl Display) -> Self {
        Error::UnsupportedOperation(op.to_string())
    }

    pub fn invalid_path(msg: impl Display) -> Self {
        Error::InvalidPath(msg.to_string())
    }
}

impl serde::ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl serde::de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Message(msg) => f.write_str(msg),
            Error::UnsupportedOperation(msg) => write!(f, "unsupported operation: {msg}"),
            Error::Io(msg) => write!(f, "I/O error: {msg}"),
            Error::InvalidPath(msg) => write!(f, "invalid path: {msg}"),
            Error::MissingField(msg) => write!(f, "missing field: {msg}"),
            Error::Parse(msg) => write!(f, "parse error: {msg}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err.to_string())
    }
}

#[cfg(feature = "with-zip")]
impl From<zip::result::ZipError> for Error {
    fn from(err: zip::result::ZipError) -> Self {
        Error::Io(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
