// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::error::{Error, Result};
use serde::{
    de, ser,
    ser::{SerializeMap, SerializeStruct},
    Deserialize, Serialize,
};
use std::collections::{btree_map::Entry, BTreeMap};

/// Description of a Serde-based serialization format.
/// First, the formats of anonymous "value" types.
#[derive(Serialize, Deserialize, Debug, Eq, Clone, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum Format {
    /// Placeholder for an unknown format (e.g. after tracing `None` value).
    Unknown,
    /// The name of a container.
    TypeName(String),

    // The formats of primitive types
    Unit,
    Bool,
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    U128,
    F32,
    F64,
    Char,
    Str,
    Bytes,

    /// The format of `Option<T>`.
    Option(Box<Format>),
    /// A sequence, e.g. the format of `Vec<Foo>`.
    Seq(Box<Format>),
    /// A map, e.g. the format of `BTreeMap<K, V>`.
    #[serde(rename_all = "UPPERCASE")]
    Map {
        key: Box<Format>,
        value: Box<Format>,
    },

    /// A tuple, e.g. the format of `(Foo, Bar)`.
    Tuple(Vec<Format>),
    /// Alias for `(Foo, ... Foo)`.
    /// E.g. the format of `[Foo; N]`.
    #[serde(rename_all = "UPPERCASE")]
    TupleArray {
        content: Box<Format>,
        size: usize,
    },
}

/// Second, the formats of named "container" types.
/// In Rust, those are enums and structs.
#[derive(Serialize, Deserialize, Debug, Eq, Clone, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum ContainerFormat {
    /// An empty struct, e.g. `struct A`.
    UnitStruct,
    /// A struct with a single unnamed parameter, e.g. `struct A(u16)`
    NewTypeStruct(Box<Format>),
    /// A struct with several unnamed parameters, e.g. `struct A(u16, u32)`
    TupleStruct(Vec<Format>),
    /// A struct with named parameters, e.g. `struct A { a: Foo }`.
    Struct(Vec<Named<Format>>),
    /// An enum, that is, an enumeration of variants.
    /// Each variant has a unique name and index within the enum.
    Enum(BTreeMap<u32, Named<VariantFormat>>),
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
/// A named value.
/// Used for named parameters or variant cases.
pub struct Named<T> {
    pub name: String,
    pub value: T,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
/// Description of a variant in an enum.
pub enum VariantFormat {
    /// A variant whose format is yet unknown.
    Unknown,
    /// A variant without parameters, e.g. `A` in `enum X { A }`
    Unit,
    /// A variant with a single unnamed parameter, e.g. `A` in `enum X { A(u16) }`
    NewType(Box<Format>),
    /// A struct with several unnamed parameters, e.g. `A` in `enum X { A(u16, u32) }`
    Tuple(Vec<Format>),
    /// A struct with named parameters, e.g. `A` in `enum X { A { a: Foo } }`
    Struct(Vec<Named<Format>>),
}

pub(crate) trait FormatHolder {
    /// Update `self` to replace unknown types or include newly discovered variants.
    fn merge(&mut self, other: Self) -> Result<()>;

    /// Finalize format values by making sure it has no more `Unknown` leaves
    /// and that all eligible tuples are compressed into a TupleArray.
    fn normalize(&mut self) -> Result<()>;
}

fn merge_error<T>(v1: T, v2: T) -> Error
where
    T: std::fmt::Debug,
{
    Error::Incompatible(format!("{:?}", v1), format!("{:?}", v2))
}

impl FormatHolder for VariantFormat {
    fn merge(&mut self, mut format: VariantFormat) -> Result<()> {
        // Matching `&mut format` instead of `format` because of
        //   "error[E0009]: cannot bind by-move and by-ref in the same pattern"
        // See also https://github.com/rust-lang/rust/issues/68354
        // We make it work using std::mem::take (and the Default trait).
        match (&mut *self, &mut format) {
            (format1 @ Self::Unknown, format2) => {
                let format2 = std::mem::take(format2);
                *format1 = format2;
            }

            (_, Self::Unknown) | (Self::Unit, Self::Unit) => (),

            (Self::NewType(format1), Self::NewType(format2)) => {
                let format2 = std::mem::take(format2.as_mut());
                format1.as_mut().merge(format2)?;
            }

            (Self::Tuple(formats1), Self::Tuple(formats2)) => {
                if formats1.len() != formats2.len() {
                    return Err(merge_error(formats1, formats2));
                }
                let mut formats2 = formats2.iter_mut();
                for format1 in formats1 {
                    let format2 = std::mem::take(formats2.next().unwrap());
                    format1.merge(format2)?;
                }
            }

            (Self::Struct(formats1), Self::Struct(formats2)) => {
                if formats1.len() != formats2.len() {
                    return Err(merge_error(formats1, formats2));
                }
                let mut formats2 = formats2.iter_mut();
                for format1 in formats1 {
                    let format2 = std::mem::take(formats2.next().unwrap());
                    format1.merge(format2)?;
                }
            }

            _ => {
                return Err(merge_error(self, &mut format));
            }
        }
        Ok(())
    }

    fn normalize(&mut self) -> Result<()> {
        match self {
            Self::Unknown => Err(Error::UnknownFormat),
            Self::Unit => Ok(()),
            Self::NewType(format) => format.normalize(),
            Self::Tuple(formats) => {
                for format in formats {
                    format.normalize()?;
                }
                Ok(())
            }
            Self::Struct(named_formats) => {
                for format in named_formats {
                    format.normalize()?;
                }
                Ok(())
            }
        }
    }
}

impl<T> FormatHolder for Named<T>
where
    T: FormatHolder + std::fmt::Debug,
{
    fn merge(&mut self, other: Named<T>) -> Result<()> {
        if self.name != other.name {
            return Err(merge_error(&*self, &other));
        }
        self.value.merge(other.value)
    }

    fn normalize(&mut self) -> Result<()> {
        self.value.normalize()
    }
}

pub(crate) trait ContainerFormatEntry {
    fn merge(self, format: ContainerFormat) -> Result<()>;
}

impl<'a> ContainerFormatEntry for Entry<'a, &'static str, ContainerFormat> {
    fn merge(self, format: ContainerFormat) -> Result<()> {
        match self {
            Entry::Vacant(e) => {
                e.insert(format);
                Ok(())
            }
            Entry::Occupied(e) => e.into_mut().merge(format),
        }
    }
}

impl FormatHolder for ContainerFormat {
    fn merge(&mut self, mut format: ContainerFormat) -> Result<()> {
        // Matching `&mut format` instead of `format` because of
        // "error[E0009]: cannot bind by-move and by-ref in the same pattern"
        match (&mut *self, &mut format) {
            (Self::UnitStruct, Self::UnitStruct) => (),

            (Self::NewTypeStruct(format1), Self::NewTypeStruct(format2)) => {
                let format2 = std::mem::take(format2.as_mut());
                format1.as_mut().merge(format2)?;
            }

            (Self::TupleStruct(formats1), Self::TupleStruct(formats2)) => {
                if formats1.len() != formats2.len() {
                    return Err(merge_error(self, &mut format));
                }
                let mut formats2 = formats2.iter_mut();
                for format1 in formats1 {
                    let format2 = std::mem::take(formats2.next().unwrap());
                    format1.merge(format2)?;
                }
            }

            (Self::Struct(named_formats1), Self::Struct(named_formats2)) => {
                if named_formats1.len() != named_formats2.len() {
                    return Err(merge_error(self, &mut format));
                }
                let mut named_formats2 = named_formats2.iter_mut();
                for format1 in named_formats1 {
                    let format2 = std::mem::take(named_formats2.next().unwrap());
                    format1.merge(format2)?;
                }
            }

            (Self::Enum(variants1), Self::Enum(variants2)) => {
                for (index2, variant2) in variants2.iter_mut() {
                    let variant2 = std::mem::take(variant2);
                    match variants1.entry(*index2) {
                        Entry::Vacant(e) => {
                            // Note that we do not check for name collisions.
                            e.insert(variant2);
                        }
                        Entry::Occupied(mut e) => {
                            e.get_mut().merge(variant2)?;
                        }
                    }
                }
            }

            _ => {
                return Err(merge_error(self, &mut format));
            }
        }
        Ok(())
    }

    fn normalize(&mut self) -> Result<()> {
        match &mut *self {
            Self::UnitStruct => Ok(()),

            Self::NewTypeStruct(format) => format.normalize(),

            Self::TupleStruct(formats) => {
                for format in formats {
                    format.normalize()?;
                }
                Ok(())
            }

            Self::Struct(named_formats) => {
                for format in named_formats {
                    format.normalize()?;
                }
                Ok(())
            }

            Self::Enum(variants) => {
                for variant in variants.values_mut() {
                    variant.normalize()?;
                }
                Ok(())
            }
        }
    }
}

impl FormatHolder for Format {
    /// Merge the newly "traced" value `format` into the current format.
    /// Note that there should be no `TupleArray`s at this point.
    fn merge(&mut self, mut format: Format) -> Result<()> {
        // Matching `&mut format` instead of `format` because of
        // "error[E0009]: cannot bind by-move and by-ref in the same pattern"
        match (&mut *self, &mut format) {
            (format1 @ Self::Unknown, format2) => {
                let format2 = std::mem::take(format2);
                *format1 = format2;
            }

            (_, Self::Unknown)
            | (Self::Unit, Self::Unit)
            | (Self::Bool, Self::Bool)
            | (Self::I8, Self::I8)
            | (Self::I16, Self::I16)
            | (Self::I32, Self::I32)
            | (Self::I64, Self::I64)
            | (Self::U8, Self::U8)
            | (Self::U16, Self::U16)
            | (Self::U32, Self::U32)
            | (Self::U64, Self::U64)
            | (Self::F32, Self::F32)
            | (Self::F64, Self::F64)
            | (Self::Char, Self::Char)
            | (Self::Str, Self::Str)
            | (Self::Bytes, Self::Bytes) => (),

            (Self::TypeName(name1), Self::TypeName(name2)) => {
                if name1 != name2 {
                    return Err(merge_error(self, &mut format));
                }
            }

            (Self::Option(format1), Self::Option(format2))
            | (Self::Seq(format1), Self::Seq(format2)) => {
                let format2 = std::mem::take(format2.as_mut());
                format1.as_mut().merge(format2)?;
            }

            (Self::Tuple(formats1), Self::Tuple(formats2)) => {
                if formats1.len() != formats2.len() {
                    return Err(merge_error(self, &mut format));
                }
                let mut formats2 = formats2.iter_mut();
                for format1 in formats1 {
                    let format2 = std::mem::take(formats2.next().unwrap());
                    format1.merge(format2)?;
                }
            }

            (
                Self::Map {
                    key: key1,
                    value: value1,
                },
                Self::Map {
                    key: key2,
                    value: value2,
                },
            ) => {
                let key2 = std::mem::take(key2.as_mut());
                let value2 = std::mem::take(value2.as_mut());
                key1.as_mut().merge(key2)?;
                value1.as_mut().merge(value2)?;
            }

            _ => {
                return Err(merge_error(self, &mut format));
            }
        }
        Ok(())
    }

    fn normalize(&mut self) -> Result<()> {
        let normalized_tuple;
        match &mut *self {
            Self::TypeName(_)
            | Self::Unit
            | Self::Bool
            | Self::I8
            | Self::I16
            | Self::I32
            | Self::I64
            | Self::I128
            | Self::U8
            | Self::U16
            | Self::U32
            | Self::U64
            | Self::U128
            | Self::F32
            | Self::F64
            | Self::Char
            | Self::Str
            | Self::Bytes => {
                return Ok(());
            }

            Self::Unknown => {
                return Err(Error::UnknownFormat);
            }

            Self::Option(format)
            | Self::Seq(format)
            | Self::TupleArray {
                content: format, ..
            } => {
                return format.normalize();
            }

            Self::Map { key, value } => {
                key.normalize()?;
                return value.normalize();
            }

            // The only case where compression happens.
            Self::Tuple(formats) => {
                for format in &mut *formats {
                    format.normalize()?;
                }
                let size = formats.len();
                if size <= 1 {
                    return Ok(());
                }
                let format0 = &formats[0];
                for format in formats.iter().skip(1) {
                    if format != format0 {
                        return Ok(());
                    }
                }
                normalized_tuple = Self::TupleArray {
                    content: Box::new(std::mem::take(&mut formats[0])),
                    size,
                };
                // fall to final statement to mutate self
            }
        }
        *self = normalized_tuple;
        Ok(())
    }
}

impl Default for Format {
    fn default() -> Self {
        Self::Unknown
    }
}

impl Default for VariantFormat {
    fn default() -> Self {
        Self::Unknown
    }
}

// For better rendering in human readable formats, we wish to serialize
// `Named { key: x, value: y }` as a map `{ x: y }`.
impl<T> Serialize for Named<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        if serializer.is_human_readable() {
            let mut map = serializer.serialize_map(Some(1))?;
            map.serialize_entry(&self.name, &self.value)?;
            map.end()
        } else {
            let mut inner = serializer.serialize_struct("Named", 2)?;
            inner.serialize_field("name", &self.name)?;
            inner.serialize_field("value", &self.value)?;
            inner.end()
        }
    }
}

struct NamedVisitor<T> {
    marker: std::marker::PhantomData<T>,
}

impl<T> NamedVisitor<T> {
    fn new() -> Self {
        Self {
            marker: std::marker::PhantomData,
        }
    }
}

impl<'de, T> de::Visitor<'de> for NamedVisitor<T>
where
    T: Deserialize<'de>,
{
    type Value = Named<T>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a single entry map")
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: de::MapAccess<'de>,
    {
        let named_value = match access.next_entry::<String, T>()? {
            Some((name, value)) => Named { name, value },
            _ => {
                return Err(de::Error::custom("Missing entry"));
            }
        };
        if access.next_entry::<String, T>()?.is_some() {
            return Err(de::Error::custom("Too many entries"));
        }
        Ok(named_value)
    }
}

/// For deserialization of non-human readable `Named` values, we keep it simple and use derive macros.
#[derive(Deserialize)]
#[serde(rename = "Named")]
struct NamedInternal<T> {
    name: String,
    value: T,
}

impl<'de, T> Deserialize<'de> for Named<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Named<T>, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            deserializer.deserialize_map(NamedVisitor::new())
        } else {
            let NamedInternal { name, value } = NamedInternal::deserialize(deserializer)?;
            Ok(Self { name, value })
        }
    }
}
