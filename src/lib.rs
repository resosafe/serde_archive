//! Archive serializer/deserializer for Serde (tar and zip).
//!
//! Each leaf value becomes a file; nested fields become path segments
//! (e.g. `sub/id`), without embedding Rust type names.

pub mod de;
pub mod error;
mod path;
pub mod ser;

pub use error::{Error, Result};
pub use ser::{to_writer, ArchiveWriter, Serializer};

pub use de::{from_archive, Archive};

#[cfg(feature = "with-zip")]
pub use de::from_zip;

#[cfg(feature = "with-tar")]
pub use de::from_tar;
