[![Build Status](https://travis-ci.org/resosafe/serde_archive.svg?branch=master)](https://travis-ci.org/github/resosafe/serde_archive)
![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)

# serde_archive

Archive serializer for Serde

## Status

Supports tar and zip formats.
Only simple Serialization at the moment: support for Structures, Map<String, T> and primitive types.

## Usage


```rust

    let item = Item {
        name: String::from("test"),
        sub: SubItem {
            id: 12
        }
    };

    let mut file = std::fs::File::create("/tmp/serde_zip-test.zip").unwrap();
    let writer = serde_archive::ser::zip_writer::ZipWriter::new(file);
    serde_archive::to_writer(writer, &item);
    

    let mut file = std::fs::File::create("/tmp/serde_tar-test.tar").unwrap();
    let writer = serde_archive::ser::tar_writer::TarWriter::new(file);
    serde_archive::to_writer(writer, &item);

```# serde_zip
