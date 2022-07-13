use std::fmt::Display;

#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    Message(String),
    UnsupportedOperation(String),
}

impl serde::ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Message(msg) => formatter.write_str(msg),
            Error::UnsupportedOperation(msg) => formatter.write_str(&format!("unsuported operation: {}", msg)),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
