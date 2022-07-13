extern crate serde;

#[cfg(test)]
#[macro_use]
extern crate serde_derive;

pub mod error;
pub mod ser;

pub use error::Error;
pub use ser::{to_writer, Serializer};
