use crate::error::{Error, Result};
use serde::ser::{self, Impossible, Serialize};
use std::fmt::Display;

#[cfg(any(doc, feature = "with-zip"))]
pub mod zip_writer;

#[cfg(any(doc, feature = "with-tar"))]
pub mod tar_writer;

pub fn to_writer<W: ArchiveWriter, S: Serialize>(writer: W, value: &S) -> Result<()> {
    let mut ser = Serializer::new(writer);
    value.serialize(&mut ser)?;
    ser.finish()?;
    Ok(())
}

pub trait ArchiveWriter {
    fn write(&mut self, path: &Vec<String>, data: &[u8]) -> Result<()>;

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
    pub fn new(writer: W) -> Self where {
        Self { writer, path: Vec::new() }
    }

    pub fn finish(&mut self) -> Result<()> {
        self.writer.finish()
    }

    pub fn start_entry(&mut self, name: &str) {
        self.path.push(String::from(name));
    }

    pub fn end_entry(&mut self) {
        self.path.pop();
    }

    fn add_primitive<P: Display>(&mut self, primitive: P) -> Result<()> {
        self.add_data(primitive.to_string().as_bytes())
    }

    pub fn add_data(&mut self, data: &[u8]) -> Result<()> {
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

    type SerializeSeq = Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;
    type SerializeMap = Map<'w, W>;
    type SerializeStruct = Struct<'w, W>;
    type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        if v {
            self.add_data(b"true")?;
        } else {
            self.add_data(b"false")?;
        }

        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        self.add_primitive(&v)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok> {
        self.add_primitive(&v)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
        self.add_primitive(&v)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok> {
        self.add_primitive(&v)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        self.add_primitive(&v)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        self.add_primitive(&v)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
        self.add_primitive(&v)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
        self.add_primitive(&v)
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
        self.add_primitive(&v)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
        self.add_primitive(&v)
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        self.add_primitive(&v)
    }

    fn serialize_str(self, value: &str) -> Result<Self::Ok> {
        self.add_data(value.as_bytes())
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<Self::Ok> {
        self.add_data(value)
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        Ok(())
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Self::Ok> {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        self.serialize_none()
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok> {
        Err(Error::UnsupportedOperation("serialize_unit_struct".to_string()))
    }

    fn serialize_unit_variant(self, name: &'static str, variant_index: u32, variant: &'static str) -> Result<Self::Ok> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(self, name: &'static str, value: &T) -> Result<Self::Ok> {
        Err(Error::UnsupportedOperation("serialize_newtype_struct".to_string()))
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(self, name: &'static str, variant_index: u32, variant: &'static str, value: &T) -> Result<Self::Ok> {
        Err(Error::UnsupportedOperation("serialize_newtype_variant".to_string()))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(Error::UnsupportedOperation("serialize_seq".to_string()))
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        Err(Error::UnsupportedOperation("serialize_tuple".to_string()))
    }

    fn serialize_tuple_struct(self, name: &'static str, len: usize) -> Result<Self::SerializeTupleStruct> {
        Err(Error::UnsupportedOperation("serialize_tuple_struct".to_string()))
    }

    fn serialize_tuple_variant(self, name: &'static str, variant_index: u32, variant: &'static str, len: usize) -> Result<Self::SerializeTupleVariant> {
        Err(Error::UnsupportedOperation("serialize_tuple_variant".to_string()))
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(Map::new(self))
    }

    fn serialize_struct(self, name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.start_entry(name);
        Ok(Struct::new(self, name))
    }

    fn serialize_struct_variant(self, name: &'static str, variant_index: u32, variant: &'static str, len: usize) -> Result<Self::SerializeStructVariant> {
        Err(Error::UnsupportedOperation("Result".to_string()))
    }
}

pub struct Map<'w, W>
where
    W: 'w + ArchiveWriter,
{
    parent: &'w mut Serializer<W>,
}

impl<'w, W> Map<'w, W>
where
    W: 'w + ArchiveWriter,
{
    pub fn new(parent: &'w mut Serializer<W>) -> Map<'w, W> {
        Map { parent }
    }
}

impl<'w, W> ser::SerializeMap for Map<'w, W>
where
    W: 'w + ArchiveWriter,
{
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, _: &T) -> Result<()> {
        panic!("impossible to serialize the key on its own, please use serialize_entry()")
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        value.serialize(&mut *self.parent)
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }

    fn serialize_entry<K: ?Sized + Serialize, V: ?Sized + Serialize>(&mut self, key: &K, value: &V) -> Result<()> {
        let c = key.serialize(MapKeySerializer)?;
        self.parent.start_entry(&c);
        value.serialize(&mut *self.parent)?;
        self.parent.end_entry();

        Ok(())
    }
}

struct MapKeySerializer;

fn key_must_be_a_string() -> Error {
    Error::UnsupportedOperation("key must be a string".to_string())
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
    fn serialize_unit_variant(self, _name: &'static str, _variant_index: u32, variant: &'static str) -> Result<String> {
        Ok(variant.to_owned())
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
        Ok(value.to_owned())
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

    fn serialize_newtype_variant<T>(self, _name: &'static str, _variant_index: u32, _variant: &'static str, _value: &T) -> Result<String>
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

    fn serialize_tuple_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeTupleStruct> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple_variant(self, _name: &'static str, _variant_index: u32, _variant: &'static str, _len: usize) -> Result<Self::SerializeTupleVariant> {
        Err(key_must_be_a_string())
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(key_must_be_a_string())
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Err(key_must_be_a_string())
    }

    fn serialize_struct_variant(self, _name: &'static str, _variant_index: u32, _variant: &'static str, _len: usize) -> Result<Self::SerializeStructVariant> {
        Err(key_must_be_a_string())
    }

    fn collect_str<T: ?Sized>(self, value: &T) -> Result<String>
    where
        T: Display,
    {
        Ok(value.to_string())
    }
}

pub struct Struct<'w, W>
where
    W: 'w + ArchiveWriter,
{
    parent: &'w mut Serializer<W>,
    name: &'w str,
}

impl<'w, W> Struct<'w, W>
where
    W: 'w + ArchiveWriter,
{
    pub fn new(parent: &'w mut Serializer<W>, name: &'w str) -> Struct<'w, W> {
        Struct { parent, name }
    }
}

impl<'w, W> ser::SerializeStruct for Struct<'w, W>
where
    W: 'w + ArchiveWriter,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, key: &'static str, value: &T) -> Result<()> {
        self.parent.start_entry(key);
        value.serialize(&mut *self.parent)?;
        self.parent.end_entry();

        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        self.parent.end_entry();
        Ok(())
    }
}
