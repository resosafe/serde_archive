use crate::error::{Error, Result};
use crate::path::join_path;
use serde::de::{
    self, Deserialize, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess,
    VariantAccess, Visitor,
};
use std::collections::BTreeMap;
#[cfg(feature = "with-zip")]
use std::io::{Read, Seek};
#[cfg(all(feature = "with-tar", not(feature = "with-zip")))]
use std::io::Read;
use std::str;

#[cfg(any(doc, feature = "with-zip"))]
pub mod zip_reader;

#[cfg(any(doc, feature = "with-tar"))]
pub mod tar_reader;

/// In-memory archive index: entry path → file contents.
#[derive(Clone, Debug, Default)]
pub struct Archive {
    files: BTreeMap<String, Vec<u8>>,
}

impl Archive {
    pub fn new() -> Self {
        Self {
            files: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, path: String, data: Vec<u8>) {
        if path.ends_with('/') {
            return;
        }
        self.files.insert(path, data);
    }

    pub fn get(&self, path: &str) -> Option<&[u8]> {
        self.files.get(path).map(|v| v.as_slice())
    }

    pub fn contains_prefix(&self, prefix: &str) -> bool {
        let dir = format!("{prefix}/");
        self.files
            .keys()
            .any(|k| k == prefix || k.starts_with(&dir))
    }

    /// Immediate child names under `prefix` (empty prefix = archive root).
    pub fn child_names(&self, prefix: &str) -> Vec<String> {
        let mut names = Vec::new();
        for key in self.files.keys() {
            let rest = if prefix.is_empty() {
                key.as_str()
            } else if let Some(r) = key.strip_prefix(prefix) {
                r.strip_prefix('/').unwrap_or(r)
            } else {
                continue;
            };
            if rest.is_empty() {
                continue;
            }
            let name = rest.split('/').next().unwrap_or(rest).to_owned();
            if !names.contains(&name) {
                names.push(name);
            }
        }
        names
    }
}

pub fn from_archive<T>(archive: &Archive) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    from_archive_owned(archive.clone())
}

fn from_archive_owned<T>(archive: Archive) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let mut de = Deserializer {
        archive,
        path: Vec::new(),
    };
    T::deserialize(&mut de)
}

#[cfg(feature = "with-zip")]
pub fn from_zip<R, T>(reader: R) -> Result<T>
where
    R: Read + Seek,
    T: for<'de> Deserialize<'de>,
{
    from_archive_owned(zip_reader::read_archive(reader)?)
}

#[cfg(feature = "with-tar")]
pub fn from_tar<R, T>(reader: R) -> Result<T>
where
    R: Read,
    T: for<'de> Deserialize<'de>,
{
    from_archive_owned(tar_reader::read_archive(reader)?)
}

struct Deserializer {
    archive: Archive,
    path: Vec<String>,
}

impl Deserializer {
    fn current_path(&self) -> String {
        join_path(&self.path)
    }

    fn current_bytes(&self) -> Option<&[u8]> {
        self.archive.get(&self.current_path())
    }

    fn exists(&self) -> bool {
        let path = self.current_path();
        self.archive.get(&path).is_some() || self.archive.contains_prefix(&path)
    }

    fn child_names(&self) -> Vec<String> {
        self.archive.child_names(&self.current_path())
    }

    fn seq_len(&self) -> usize {
        self.child_names()
            .iter()
            .filter_map(|n| n.parse::<usize>().ok())
            .max()
            .map(|m| m + 1)
            .unwrap_or(0)
    }

    fn read_bytes(&self) -> Result<&[u8]> {
        self.current_bytes()
            .ok_or_else(|| Error::MissingField(self.current_path()))
    }

    fn read_str(&self) -> Result<String> {
        let bytes = self.read_bytes()?;
        str::from_utf8(bytes)
            .map(|s| s.to_owned())
            .map_err(|e| Error::Parse(e.to_string()))
    }

    fn parse_prim<T: str::FromStr>(&self) -> Result<T>
    where
        T::Err: std::fmt::Display,
    {
        self.read_str()?
            .parse()
            .map_err(|e: T::Err| Error::Parse(e.to_string()))
    }

    fn enter(&mut self, name: &str) {
        self.path.push(name.to_owned());
    }

    fn exit(&mut self) {
        self.path.pop();
    }
}

impl<'de> de::Deserializer<'de> for &mut Deserializer {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        if let Some(data) = self.current_bytes() {
            if let Ok(s) = str::from_utf8(data) {
                if s == "true" || s == "false" {
                    return self.deserialize_bool(visitor);
                }
                if s.parse::<i64>().is_ok() {
                    return self.deserialize_i64(visitor);
                }
                if s.parse::<f64>().is_ok() {
                    return self.deserialize_f64(visitor);
                }
                return visitor.visit_string(s.to_owned());
            }
            return visitor.visit_byte_buf(data.to_vec());
        }
        if !self.child_names().is_empty() {
            return self.deserialize_map(visitor);
        }
        Err(Error::MissingField(self.current_path()))
    }

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.read_str()?.as_str() {
            "true" => visitor.visit_bool(true),
            "false" => visitor.visit_bool(false),
            other => Err(Error::Parse(format!("invalid bool: {other}"))),
        }
    }

    fn deserialize_i8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_i8(self.parse_prim()?)
    }
    fn deserialize_i16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_i16(self.parse_prim()?)
    }
    fn deserialize_i32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_i32(self.parse_prim()?)
    }
    fn deserialize_i64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_i64(self.parse_prim()?)
    }
    fn deserialize_u8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u8(self.parse_prim()?)
    }
    fn deserialize_u16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u16(self.parse_prim()?)
    }
    fn deserialize_u32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u32(self.parse_prim()?)
    }
    fn deserialize_u64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u64(self.parse_prim()?)
    }
    fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_f32(self.parse_prim()?)
    }
    fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_f64(self.parse_prim()?)
    }

    fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let s = self.read_str()?;
        let mut chars = s.chars();
        match (chars.next(), chars.next()) {
            (Some(c), None) => visitor.visit_char(c),
            _ => Err(Error::Parse(format!("invalid char: {s}"))),
        }
    }

    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_string(self.read_str()?)
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_string(self.read_str()?)
    }

    fn deserialize_bytes<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_byte_buf(self.read_bytes()?.to_vec())
    }

    fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_byte_buf(self.read_bytes()?.to_vec())
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        if self.exists() {
            visitor.visit_some(self)
        } else {
            visitor.visit_none()
        }
    }

    fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let _ = self.current_bytes();
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value> {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let len = self.seq_len();
        visitor.visit_seq(SeqDeserializer {
            de: self,
            index: 0,
            len,
        })
    }

    fn deserialize_tuple<V: Visitor<'de>>(self, len: usize, visitor: V) -> Result<V::Value> {
        visitor.visit_seq(SeqDeserializer {
            de: self,
            index: 0,
            len,
        })
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value> {
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let keys = self.child_names();
        visitor.visit_map(MapDeserializer {
            de: self,
            keys,
            index: 0,
        })
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        if let Some(data) = self.current_bytes() {
            let s = str::from_utf8(data)
                .map_err(|e| Error::Parse(e.to_string()))?
                .to_owned();
            return visitor.visit_enum(s.into_deserializer());
        }
        let children = self.child_names();
        if children.len() == 1 {
            let variant = children[0].clone();
            return visitor.visit_enum(EnumDeserializer {
                de: self,
                variant,
            });
        }
        Err(Error::Parse(format!(
            "cannot determine enum variant at {}",
            self.current_path()
        )))
    }

    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_string(visitor)
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_unit()
    }
}

struct SeqDeserializer<'a> {
    de: &'a mut Deserializer,
    index: usize,
    len: usize,
}

impl<'de> SeqAccess<'de> for SeqDeserializer<'_> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        if self.index >= self.len {
            return Ok(None);
        }
        let name = self.index.to_string();
        self.de.enter(&name);
        self.index += 1;
        let value = seed.deserialize(&mut *self.de)?;
        self.de.exit();
        Ok(Some(value))
    }
}

struct MapDeserializer<'a> {
    de: &'a mut Deserializer,
    keys: Vec<String>,
    index: usize,
}

impl<'de> MapAccess<'de> for MapDeserializer<'_> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        if self.index >= self.keys.len() {
            return Ok(None);
        }
        let key = self.keys[self.index].clone();
        seed.deserialize(key.into_deserializer()).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        let key = self.keys[self.index].clone();
        self.de.enter(&key);
        self.index += 1;
        let value = seed.deserialize(&mut *self.de)?;
        self.de.exit();
        Ok(value)
    }
}

struct EnumDeserializer<'a> {
    de: &'a mut Deserializer,
    variant: String,
}

impl<'de> EnumAccess<'de> for EnumDeserializer<'_> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'de>,
    {
        let deser: de::value::StringDeserializer<Error> =
            self.variant.clone().into_deserializer();
        let v = seed.deserialize(deser)?;
        Ok((v, self))
    }
}

impl<'de> VariantAccess<'de> for EnumDeserializer<'_> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        self.de.enter(&self.variant);
        let value = seed.deserialize(&mut *self.de)?;
        self.de.exit();
        Ok(value)
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.de.enter(&self.variant);
        let value = de::Deserializer::deserialize_tuple(&mut *self.de, len, visitor)?;
        self.de.exit();
        Ok(value)
    }

    fn struct_variant<V>(self, fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.de.enter(&self.variant);
        let value = de::Deserializer::deserialize_struct(&mut *self.de, "", fields, visitor)?;
        self.de.exit();
        Ok(value)
    }
}
