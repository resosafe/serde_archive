#![cfg(any(feature = "with-zip", feature = "all"))]

use serde::{Deserialize, Serialize};
use serde_archive::ser::zip_writer::{ZipCompression, ZipWriter};
use std::collections::HashMap;
use std::io::{Seek, SeekFrom};

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
    Item {
        name: "tempfile-test".into(),
        sub: SubItem { id: 42 },
        tags: vec!["zip".into(), "disk".into()],
        meta: HashMap::from([("n".into(), 7)]),
        note: Some("on disk".into()),
    }
}

fn write_and_read_zip(compression: ZipCompression, item: &Item) -> Item {
    let mut tmp = tempfile::Builder::new()
        .prefix("serde_archive-")
        .suffix(".zip")
        .tempfile()
        .expect("create tempfile");

    {
        let writer = ZipWriter::new(tmp.as_file_mut(), compression);
        serde_archive::to_writer(writer, item).expect("write zip");
    }

    assert!(
        tmp.path().metadata().map(|m| m.len() > 0).unwrap_or(false),
        "zip tempfile should be non-empty: {:?}",
        tmp.path()
    );

    tmp.as_file_mut()
        .seek(SeekFrom::Start(0))
        .expect("seek start");
    serde_archive::from_zip(tmp.as_file_mut()).expect("read zip")
}

#[test]
fn zip_tempfile_deflated_roundtrip() {
    let item = sample();
    let decoded = write_and_read_zip(ZipCompression::Deflated, &item);
    assert_eq!(decoded, item);
}

#[test]
fn zip_tempfile_stored_roundtrip() {
    let item = sample();
    let decoded = write_and_read_zip(ZipCompression::None, &item);
    assert_eq!(decoded, item);
}

#[test]
fn zip_tempfile_bzip2_roundtrip() {
    let item = sample();
    let decoded = write_and_read_zip(ZipCompression::Bzip2, &item);
    assert_eq!(decoded, item);
}

#[test]
fn zip_tempfile_reopen_by_path() {
    let item = sample();
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("archive.zip");

    {
        let file = std::fs::File::create(&path).expect("create zip file");
        let writer = ZipWriter::new(file, ZipCompression::Deflated);
        serde_archive::to_writer(writer, &item).expect("write zip");
    }

    let file = std::fs::File::open(&path).expect("open zip file");
    let decoded: Item = serde_archive::from_zip(file).expect("read zip");
    assert_eq!(decoded, item);
}

#[test]
fn zip_tempfile_lists_expected_entries() {
    let item = sample();
    let mut tmp = tempfile::Builder::new()
        .suffix(".zip")
        .tempfile()
        .expect("create tempfile");

    {
        let writer = ZipWriter::new(tmp.as_file_mut(), ZipCompression::Deflated);
        serde_archive::to_writer(writer, &item).expect("write zip");
    }

    tmp.as_file_mut()
        .seek(SeekFrom::Start(0))
        .expect("seek start");
    let mut zip = zip::ZipArchive::new(tmp.as_file_mut()).expect("open archive");

    let mut names: Vec<String> = (0..zip.len())
        .map(|i| zip.by_index(i).unwrap().name().to_owned())
        .filter(|n| !n.ends_with('/'))
        .collect();
    names.sort();

    assert_eq!(
        names,
        vec![
            "meta/n".to_string(),
            "name".to_string(),
            "note".to_string(),
            "sub/id".to_string(),
            "tags/0".to_string(),
            "tags/1".to_string(),
        ]
    );
}
