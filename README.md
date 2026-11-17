# serde_archive

Archive serializer/deserializer for Serde.

## Status

Supports **tar** and **zip**.

Each leaf value is stored as a file. Nested fields become path segments using
**field names only** (not Rust type names), e.g. `sub/id`.

Supported shapes: structs, maps with string-like keys, sequences/tuples
(indexed as `0`, `1`, …), options (`None` omits the entry), newtypes, and
common enums.

## Usage

```toml
[dependencies]
serde_archive = { version = "0.1", features = ["with-zip", "with-tar"] }
# or: features = ["all"]
```

```rust
use serde::{Deserialize, Serialize};
use serde_archive::ser::zip_writer::{ZipCompression, ZipWriter};
use serde_archive::ser::tar_writer::TarWriter;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Item {
    name: String,
    sub: SubItem,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct SubItem {
    id: i32,
}

let item = Item {
    name: String::from("test"),
    sub: SubItem { id: 12 },
};

// Zip
let mut buf = std::io::Cursor::new(Vec::new());
let writer = ZipWriter::new(&mut buf, ZipCompression::Deflated);
serde_archive::to_writer(writer, &item)?;
buf.set_position(0);
let decoded: Item = serde_archive::from_zip(buf)?;

// Tar
let mut buf = std::io::Cursor::new(Vec::new());
let writer = TarWriter::new(&mut buf);
serde_archive::to_writer(writer, &item)?;
buf.set_position(0);
let decoded: Item = serde_archive::from_tar(buf)?;
```

Archive layout for the example above:

```
name
sub/id
```
