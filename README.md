
# serde_archive

Archive serializer for Serde

## Status

Supports tar and zip formats.
Only simple Serialization at the moment: support for Structures, Map<String, T> and primitive types.

## Usage

To use this library, add the following to your `Cargo.toml`:
```toml
[dependencies]
serde_archive = { version = "0.1", features = ["with-zip", "with-tar"], optional = true }
```

```rust
use serde_archive::ser::zip_writer;
use serde_archive::ser::tar_writer;

let item = Item {
    name: String::from("test"),
    sub: SubItem {
        id: 12
    }
};

let mut file = std::fs::File::create("/tmp/serde_zip-test.zip").unwrap();
let writer = zip_writer::ZipWriter::new(file, zip_writer::ZipCompression::Bzip2);
serde_archive::to_writer(writer, &item);


let mut file = std::fs::File::create("/tmp/serde_tar-test.tar").unwrap();
let writer = tar_writer::TarWriter::new(file);
serde_archive::to_writer(writer, &item);
```
