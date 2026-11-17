use crate::error::{Error, Result};
use crate::path::sanitize_segment;
use serde::ser::{self, Impossible, Serialize};
use std::fmt::Display;

#[cfg(any(doc, feature = "with-zip"))]
pub mod zip_writer;

#[cfg(any(doc, feature = "with-tar"))]
pub mod tar_writer;

/// Serialize `value` into an archive writer, always attempting to finish the archive.
pub fn to_writer<W: ArchiveWriter, S: Serialize>(writer: W, value: &S) -> Result<()> {
    let mut ser = Serializer::new(writer);
    let serialize_result = value.serialize(&mut ser);
    let finish_result = ser.finish();
    match (serialize_result, finish_result) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(e), _) | (Ok(()), Err(e)) => Err(e),
    }
}

pub trait ArchiveWriter {
    fn write(&mut self, path: &[String], data: &[u8]) -> Result<()>;

    fn finish(&mut self) -> Result<()>;
}

pub struct Serializer<W: ArchiveWriter> {
    path: Vec<String>,
    writer: W,
}

impl<W> Serializer<W>
where
    W: ArchiveWriter,
{
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            path: Vec::new(),
        }
    }

    pub fn finish(&mut self) -> Result<()> {
        self.writer.finish()
    }

    pub fn start_entry(&mut self, name: &str) -> Result<()> {
        self.path.push(sanitize_segment(name)?);
        Ok(())
    }

    pub fn end_entry(&mut self) {
        self.path.pop();
    }

    fn add_primitive<P: Display>(&mut self, primitive: P) -> Result<()> {
        self.add_data(primitive.to_string().as_bytes())
    }

    pub fn add_data(&mut self, data: &[u8]) -> Result<()> {
        if self.path.is_empty() {
            return Err(Error::invalid_path(
                "refusing to write archive entry with empty path",
            ));
        }
        self.writer.write(&self.path, data)
    }
}

#[allow(unused_variables)]
impl<'w, W> ser::Serializer for &'w mut Serializer<W>
where
    W: ArchiveWriter,
{
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Seq<'w, W>;
    type SerializeTuple = Seq<'w, W>;
    type SerializeTupleStruct = Seq<'w, W>;
    type SerializeTupleVariant = Seq<'w, W>;
    type SerializeMap = Map<'w, W>;
    type SerializeStruct = Struct<'w, W>;
    type SerializeStructVariant = Struct<'w, W>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        if v {
            self.add_data(b"true")
        } else {
            self.add_data(b"false")
        }
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        self.add_primitive(v)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok> {
        self.add_primitive(v)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
        self.add_primitive(v)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok> {
        self.add_primitive(v)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        self.add_primitive(v)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        self.add_primitive(v)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
        self.add_primitive(v)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
        self.add_primitive(v)
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
        self.add_primitive(v)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
        self.add_primitive(v)
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        self.add_primitive(v)
    }

    fn serialize_str(self, value: &str) -> Result<Self::Ok> {
        self.add_data(value.as_bytes())
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<Self::Ok> {
        self.add_data(value)
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        // Option::None omits the entry; deserialization treats a missing path as None.
        Ok(())
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Self::Ok> {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        self.add_data(b"")
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        self.add_data(b"")
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok> {
        self.start_entry(variant)?;
        value.serialize(&mut *self)?;
        self.end_entry();
        Ok(())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(Seq::new(self))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Ok(Seq::new(self))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Ok(Seq::new(self))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.start_entry(variant)?;
        Ok(Seq::new_variant(self))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(Map::new(self))
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        // Field names alone form archive paths (e.g. `sub/id`), not Rust type names.
        Ok(Struct::new(self))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.start_entry(variant)?;
        Ok(Struct::new_variant(self))
    }
}

pub struct Seq<'w, W>
where
    W: 'w + ArchiveWriter,
{
    parent: &'w mut Serializer<W>,
    index: usize,
    pop_variant: bool,
}

impl<'w, W> Seq<'w, W>
where
    W: 'w + ArchiveWriter,
{
    pub fn new(parent: &'w mut Serializer<W>) -> Self {
        Self {
            parent,
            index: 0,
            pop_variant: false,
        }
    }

    fn new_variant(parent: &'w mut Serializer<W>) -> Self {
        Self {
            parent,
            index: 0,
            pop_variant: true,
        }
    }
}

impl<'w, W> ser::SerializeSeq for Seq<'w, W>
where
    W: 'w + ArchiveWriter,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        self.parent.start_entry(&self.index.to_string())?;
        value.serialize(&mut *self.parent)?;
        self.parent.end_entry();
        self.index += 1;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        // Empty sequences still need a path marker so deserialization sees the field.
        if self.index == 0 {
            self.parent.add_data(b"")?;
        }
        if self.pop_variant {
            self.parent.end_entry();
        }
        Ok(())
    }
}

impl<'w, W> ser::SerializeTuple for Seq<'w, W>
where
    W: 'w + ArchiveWriter,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok> {
        ser::SerializeSeq::end(self)
    }
}

impl<'w, W> ser::SerializeTupleStruct for Seq<'w, W>
where
    W: 'w + ArchiveWriter,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok> {
        ser::SerializeSeq::end(self)
    }
}

impl<'w, W> ser::SerializeTupleVariant for Seq<'w, W>
where
    W: 'w + ArchiveWriter,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok> {
        ser::SerializeSeq::end(self)
    }
}

pub struct Map<'w, W>
where
    W: 'w + ArchiveWriter,
{
    parent: &'w mut Serializer<W>,
    pending_key: Option<String>,
    entries: usize,
}

impl<'w, W> Map<'w, W>
where
    W: 'w + ArchiveWriter,
{
    pub fn new(parent: &'w mut Serializer<W>) -> Map<'w, W> {
        Map {
            parent,
            pending_key: None,
            entries: 0,
        }
    }
}

impl<'w, W> ser::SerializeMap for Map<'w, W>
where
    W: 'w + ArchiveWriter,
{
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<()> {
        self.pending_key = Some(key.serialize(MapKeySerializer)?);
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        let key = self
            .pending_key
            .take()
            .ok_or_else(|| Error::unsupported("serialize_value without serialize_key"))?;
        self.parent.start_entry(&key)?;
        value.serialize(&mut *self.parent)?;
        self.parent.end_entry();
        self.entries += 1;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        // Empty maps still need a path marker so deserialization sees the field.
        if self.entries == 0 {
            self.parent.add_data(b"")?;
        }
        Ok(())
    }

    fn serialize_entry<K: ?Sized + Serialize, V: ?Sized + Serialize>(
        &mut self,
        key: &K,
        value: &V,
    ) -> Result<()> {
        let c = key.serialize(MapKeySerializer)?;
        self.parent.start_entry(&c)?;
        value.serialize(&mut *self.parent)?;
        self.parent.end_entry();
        self.entries += 1;
        Ok(())
    }
}

struct MapKeySerializer;

fn key_must_be_a_string() -> Error {
    Error::unsupported("key must be a string")
}

impl serde::Serializer for MapKeySerializer {
    type Ok = String;
    type Error = Error;

    type SerializeSeq = Impossible<String, Error>;
    type SerializeTuple = Impossible<String, Error>;
    type SerializeTupleStruct = Impossible<String, Error>;
    type SerializeTupleVariant = Impossible<String, Error>;
    type SerializeMap = Impossible<String, Error>;
    type SerializeStruct = Impossible<String, Error>;
    type SerializeStructVariant = Impossible<String, Error>;

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<String> {
        sanitize_segment(variant)
    }

    #[inline]
    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<String>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_bool(self, _value: bool) -> Result<String> {
        Err(key_must_be_a_string())
    }

    fn serialize_i8(self, value: i8) -> Result<String> {
        Ok(value.to_string())
    }

    fn serialize_i16(self, value: i16) -> Result<String> {
        Ok(value.to_string())
    }

    fn serialize_i32(self, value: i32) -> Result<String> {
        Ok(value.to_string())
    }

    fn serialize_i64(self, value: i64) -> Result<String> {
        Ok(value.to_string())
    }

    fn serialize_u8(self, value: u8) -> Result<String> {
        Ok(value.to_string())
    }

    fn serialize_u16(self, value: u16) -> Result<String> {
        Ok(value.to_string())
    }

    fn serialize_u32(self, value: u32) -> Result<String> {
        Ok(value.to_string())
    }

    fn serialize_u64(self, value: u64) -> Result<String> {
        Ok(value.to_string())
    }

    fn serialize_f32(self, value: f32) -> Result<String> {
        Ok(value.to_string())
    }

    fn serialize_f64(self, value: f64) -> Result<String> {
        Ok(value.to_string())
    }

    #[inline]
    fn serialize_char(self, value: char) -> Result<String> {
        self.serialize_str(&value.to_string())
    }

    #[inline]
    fn serialize_str(self, value: &str) -> Result<String> {
        sanitize_segment(value)
    }

    fn serialize_bytes(self, _value: &[u8]) -> Result<String> {
        Err(key_must_be_a_string())
    }

    fn serialize_unit(self) -> Result<String> {
        Err(key_must_be_a_string())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<String> {
        Err(key_must_be_a_string())
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<String>
    where
        T: ?Sized + Serialize,
    {
        Err(key_must_be_a_string())
    }

    fn serialize_none(self) -> Result<String> {
        Err(key_must_be_a_string())
    }

    fn serialize_some<T>(self, _value: &T) -> Result<String>
    where
        T: ?Sized + Serialize,
    {
        Err(key_must_be_a_string())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(key_must_be_a_string())
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(key_must_be_a_string())
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Err(key_must_be_a_string())
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(key_must_be_a_string())
    }

    fn collect_str<T: ?Sized + Display>(self, value: &T) -> Result<String> {
        sanitize_segment(&value.to_string())
    }
}

pub struct Struct<'w, W>
where
    W: 'w + ArchiveWriter,
{
    parent: &'w mut Serializer<W>,
    pop_variant: bool,
}

impl<'w, W> Struct<'w, W>
where
    W: 'w + ArchiveWriter,
{
    pub fn new(parent: &'w mut Serializer<W>) -> Struct<'w, W> {
        Struct {
            parent,
            pop_variant: false,
        }
    }

    fn new_variant(parent: &'w mut Serializer<W>) -> Struct<'w, W> {
        Struct {
            parent,
            pop_variant: true,
        }
    }
}

impl<'w, W> ser::SerializeStruct for Struct<'w, W>
where
    W: 'w + ArchiveWriter,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, key: &'static str, value: &T) -> Result<()> {
        self.parent.start_entry(key)?;
        value.serialize(&mut *self.parent)?;
        self.parent.end_entry();
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        if self.pop_variant {
            self.parent.end_entry();
        }
        Ok(())
    }
}

impl<'w, W> ser::SerializeStructVariant for Struct<'w, W>
where
    W: 'w + ArchiveWriter,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, key: &'static str, value: &T) -> Result<()> {
        ser::SerializeStruct::serialize_field(self, key, value)
    }

    fn end(self) -> Result<Self::Ok> {
        ser::SerializeStruct::end(self)
    }
}
