#![cfg(feature = "all")]

use serde::{Deserialize, Serialize};
use serde_archive::ser::tar_writer::TarWriter;
use serde_archive::ser::zip_writer::{ZipCompression, ZipWriter};
use std::collections::HashMap;
use std::io::Cursor;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Item {
    name: String,
    sub: SubItem,
    tags: Vec<String>,
    meta: HashMap<String, i32>,
    note: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct SubItem {
    id: i32,
}

fn sample() -> Item {
    let mut meta = HashMap::new();
    meta.insert("a".into(), 1);
    meta.insert("b".into(), 2);
    Item {
        name: "test".into(),
        sub: SubItem { id: 12 },
        tags: vec!["x".into(), "y".into()],
        meta,
        note: None,
    }
}

#[test]
fn zip_roundtrip() {
    let item = sample();
    let mut buf = Cursor::new(Vec::new());
    {
        let writer = ZipWriter::new(&mut buf, ZipCompression::Deflated);
        serde_archive::to_writer(writer, &item).unwrap();
    }
    buf.set_position(0);
    let decoded: Item = serde_archive::from_zip(buf).unwrap();
    assert_eq!(decoded, item);
}

#[test]
fn tar_roundtrip() {
    let item = sample();
    let mut buf = Cursor::new(Vec::new());
    {
        let writer = TarWriter::new(&mut buf);
        serde_archive::to_writer(writer, &item).unwrap();
    }
    buf.set_position(0);
    let decoded: Item = serde_archive::from_tar(buf).unwrap();
    assert_eq!(decoded, item);
}

#[test]
fn rejects_path_traversal_key() {
    let mut map = HashMap::new();
    map.insert("../etc".to_string(), 1);
    let mut buf = Cursor::new(Vec::new());
    let writer = ZipWriter::new(&mut buf, ZipCompression::None);
    let err = serde_archive::to_writer(writer, &map).unwrap_err();
    assert!(err.to_string().contains("invalid path") || err.to_string().contains("forbidden"));
}
