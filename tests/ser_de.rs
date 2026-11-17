use serde::{Deserialize, Serialize};
use serde_archive::de::Archive;
use serde_archive::ser::ArchiveWriter;
use serde_archive::{from_archive, to_writer, Error};
use std::collections::HashMap;

#[derive(Default)]
struct CollectingWriter {
    archive: Archive,
}

impl ArchiveWriter for &mut CollectingWriter {
    fn write(&mut self, path: &[String], data: &[u8]) -> Result<(), Error> {
        self.archive.insert(path.join("/"), data.to_vec());
        Ok(())
    }

    fn finish(&mut self) -> Result<(), Error> {
        Ok(())
    }
}

fn serialize<T: Serialize>(value: &T) -> Archive {
    let mut sink = CollectingWriter::default();
    to_writer(&mut sink, value).expect("serialize");
    sink.archive
}

fn assert_roundtrip<T>(value: &T)
where
    T: Serialize + for<'de> Deserialize<'de> + PartialEq + std::fmt::Debug,
{
    let archive = serialize(value);
    let decoded: T = from_archive(&archive).expect("deserialize");
    assert_eq!(&decoded, value);
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Item {
    name: String,
    sub: SubItem,
    tags: Vec<String>,
    meta: HashMap<String, i32>,
    note: Option<String>,
    flag: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct SubItem {
    id: i32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct WithOption {
    value: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Newtype(i64);

#[derive(Debug, Serialize, Deserialize, PartialEq)]
enum Color {
    Red,
    Green,
    Rgb(u8, u8, u8),
    Named { name: String },
}

#[test]
fn struct_paths_use_field_names_only() {
    let item = Item {
        name: "test".into(),
        sub: SubItem { id: 12 },
        tags: vec!["x".into()],
        meta: HashMap::from([("a".into(), 1)]),
        note: None,
        flag: true,
    };
    let archive = serialize(&item);

    assert_eq!(archive.get("name"), Some(b"test".as_ref()));
    assert_eq!(archive.get("sub/id"), Some(b"12".as_ref()));
    assert_eq!(archive.get("tags/0"), Some(b"x".as_ref()));
    assert_eq!(archive.get("meta/a"), Some(b"1".as_ref()));
    assert_eq!(archive.get("flag"), Some(b"true".as_ref()));
    assert!(archive.get("note").is_none());
    assert!(archive.get("Item/name").is_none());
    assert!(archive.get("sub/SubItem/id").is_none());
}

#[test]
fn struct_roundtrip() {
    let item = Item {
        name: "hello".into(),
        sub: SubItem { id: -7 },
        tags: vec!["a".into(), "b".into(), "c".into()],
        meta: HashMap::from([("k".into(), 42)]),
        note: Some("n".into()),
        flag: false,
    };
    assert_roundtrip(&item);
}

#[test]
fn option_none_omits_entry_and_roundtrips() {
    let value = WithOption { value: None };
    let archive = serialize(&value);
    assert!(archive.get("value").is_none());
    assert_roundtrip(&value);
}

#[test]
fn option_some_roundtrip() {
    assert_roundtrip(&WithOption {
        value: Some("present".into()),
    });
}

#[test]
fn empty_vec_roundtrip() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Wrap {
        items: Vec<i32>,
    }
    assert_roundtrip(&Wrap { items: vec![] });
}

#[test]
fn nested_vec_roundtrip() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Wrap {
        items: Vec<Vec<String>>,
    }
    assert_roundtrip(&Wrap {
        items: vec![vec!["a".into(), "b".into()], vec!["c".into()]],
    });
}

#[test]
fn map_roundtrip() {
    let mut map = HashMap::<String, i32>::new();
    map.insert("one".into(), 1);
    map.insert("two".into(), 2);
    assert_roundtrip(&map);
}

#[test]
fn empty_map_roundtrip() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Wrap {
        meta: HashMap<String, i32>,
    }
    assert_roundtrip(&Wrap {
        meta: HashMap::new(),
    });
}

#[test]
fn newtype_roundtrip() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Wrap {
        n: Newtype,
    }
    assert_roundtrip(&Wrap { n: Newtype(99) });
}

#[test]
fn unit_enum_roundtrip() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Wrap {
        color: Color,
    }
    assert_roundtrip(&Wrap { color: Color::Red });
    assert_roundtrip(&Wrap {
        color: Color::Green,
    });
}

#[test]
fn tuple_enum_roundtrip() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Wrap {
        color: Color,
    }
    assert_roundtrip(&Wrap {
        color: Color::Rgb(1, 2, 3),
    });
}

#[test]
fn struct_enum_roundtrip() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Wrap {
        color: Color,
    }
    assert_roundtrip(&Wrap {
        color: Color::Named {
            name: "blue".into(),
        },
    });
}

#[test]
fn tuple_struct_roundtrip() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Pair(String, i32);
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Wrap {
        pair: Pair,
    }
    assert_roundtrip(&Wrap {
        pair: Pair("x".into(), 5),
    });
}

#[test]
fn primitives_in_struct_roundtrip() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Prim {
        b: bool,
        i: i64,
        u: u32,
        f: f64,
        c: char,
        s: String,
    }
    assert_roundtrip(&Prim {
        b: true,
        i: -100,
        u: 7,
        f: 1.5,
        c: 'é',
        s: "hi".into(),
    });
}

#[test]
fn rejects_dot_segment() {
    let mut map = HashMap::new();
    map.insert(".".to_string(), 1);
    let mut sink = CollectingWriter::default();
    let err = to_writer(&mut sink, &map).unwrap_err();
    assert!(matches!(err, Error::InvalidPath(_)));
}

#[test]
fn rejects_dotdot_segment() {
    let mut map = HashMap::new();
    map.insert("..".to_string(), 1);
    let mut sink = CollectingWriter::default();
    let err = to_writer(&mut sink, &map).unwrap_err();
    assert!(matches!(err, Error::InvalidPath(_)));
}

#[test]
fn rejects_separator_in_key() {
    let mut map = HashMap::new();
    map.insert("a/b".to_string(), 1);
    let mut sink = CollectingWriter::default();
    let err = to_writer(&mut sink, &map).unwrap_err();
    assert!(matches!(err, Error::InvalidPath(_)));
}

#[test]
fn missing_required_field_fails_deserialize() {
    let mut archive = Archive::new();
    archive.insert("name".into(), b"only-name".to_vec());
    let err = from_archive::<Item>(&archive).unwrap_err();
    assert!(matches!(err, Error::MissingField(_)) || matches!(err, Error::Message(_)));
}
