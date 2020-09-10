// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module provides traits that define the behavior of a schema and its associated key and
//! value types, along with helpers to define a new schema with ease.
use crate::ColumnFamilyName;
use anyhow::Result;
use std::fmt::Debug;

/// Macro for defining a SchemaDB schema.
///
/// `define_schema!` allows a schema to be defined in the following syntax:
/// ```
/// use anyhow::Result;
/// use schemadb::{
///     define_schema,
///     schema::{KeyCodec, SeekKeyCodec, ValueCodec},
/// };
///
/// // Define key type and value type for a schema with derived traits (Clone, Debug, Eq, PartialEq)
/// #[derive(Clone, Debug, Eq, PartialEq)]
/// pub struct Key;
/// #[derive(Clone, Debug, Eq, PartialEq)]
/// pub struct Value;
///
/// // Implement KeyCodec/ValueCodec traits for key and value types
/// impl KeyCodec<ExampleSchema> for Key {
///     fn encode_key(&self) -> Result<Vec<u8>> {
///         Ok(vec![])
///     }
///
///     fn decode_key(data: &[u8]) -> Result<Self> {
///         Ok(Key)
///     }
/// }
///
/// impl ValueCodec<ExampleSchema> for Value {
///     fn encode_value(&self) -> Result<Vec<u8>> {
///         Ok(vec![])
///     }
///
///     fn decode_value(data: &[u8]) -> Result<Self> {
///         Ok(Value)
///     }
/// }
///
/// // And finally define a schema type and associate it with key and value types, as well as the
/// // column family name, by generating code that implements the `Schema` trait for the type.
/// define_schema!(ExampleSchema, Key, Value, "exmaple_cf_name");
///
/// // SeekKeyCodec is automatically implemented for KeyCodec,
/// // so you can seek an iterator with the Key type:
/// // iter.seek(&Key);
///
/// // Or if seek-by-prefix is desired, you can implement your own SeekKey
/// #[derive(Clone, Eq, PartialEq, Debug)]
/// pub struct PrefixSeekKey;
///
/// impl SeekKeyCodec<ExampleSchema> for PrefixSeekKey {
///     fn encode_seek_key(&self) -> Result<Vec<u8>> {
///         Ok(vec![])
///     }
/// }
/// // and seek like this:
/// // iter.seek(&PrefixSeekKey);
/// ```
#[macro_export]
macro_rules! define_schema {
    ($schema_type: ident, $key_type: ty, $value_type: ty, $cf_name: expr) => {
        pub(crate) struct $schema_type;

        impl $crate::schema::Schema for $schema_type {
            const COLUMN_FAMILY_NAME: $crate::ColumnFamilyName = $cf_name;
            type Key = $key_type;
            type Value = $value_type;
        }
    };
}

/// This trait defines a type that can serve as a [`Schema::Key`].
pub trait KeyCodec<S: Schema + ?Sized>: Sized + PartialEq + Debug {
    /// Converts `self` to bytes to be stored in DB.
    fn encode_key(&self) -> Result<Vec<u8>>;
    /// Converts bytes fetched from DB to `Self`.
    fn decode_key(data: &[u8]) -> Result<Self>;
}

/// This trait defines a type that can serve as a [`Schema::Value`].
pub trait ValueCodec<S: Schema + ?Sized>: Sized + PartialEq + Debug {
    /// Converts `self` to bytes to be stored in DB.
    fn encode_value(&self) -> Result<Vec<u8>>;
    /// Converts bytes fetched from DB to `Self`.
    fn decode_value(data: &[u8]) -> Result<Self>;
}

/// This defines a type that can be used to seek a [`SchemaIterator`](crate::SchemaIterator), via
/// interfaces like [`seek`](crate::SchemaIterator::seek).
pub trait SeekKeyCodec<S: Schema + ?Sized>: Sized {
    /// Converts `self` to bytes which is used to seek the underlying raw iterator.
    fn encode_seek_key(&self) -> Result<Vec<u8>>;
}

/// All keys can automatically be used as seek keys.
impl<S, K> SeekKeyCodec<S> for K
where
    S: Schema,
    K: KeyCodec<S>,
{
    /// Delegates to [`KeyCodec::encode_key`].
    fn encode_seek_key(&self) -> Result<Vec<u8>> {
        <K as KeyCodec<S>>::encode_key(&self)
    }
}

/// This trait defines a schema: an association of a column family name, the key type and the value
/// type.
pub trait Schema {
    /// The column family name associated with this struct.
    /// Note: all schemas within the same SchemaDB must have distinct column family names.
    const COLUMN_FAMILY_NAME: ColumnFamilyName;

    /// Type of the key.
    type Key: KeyCodec<Self>;
    /// Type of the value.
    type Value: ValueCodec<Self>;
}

/// Helper used in tests to assert a (key, value) pair for a certain [`Schema`] is able to convert
/// to bytes and convert back.
pub fn assert_encode_decode<S: Schema>(key: &S::Key, value: &S::Value) {
    {
        let encoded = key.encode_key().expect("Encoding key should work.");
        let decoded = S::Key::decode_key(&encoded).expect("Decoding key should work.");
        assert_eq!(*key, decoded);
    }
    {
        let encoded = value.encode_value().expect("Encoding value should work.");
        let decoded = S::Value::decode_value(&encoded).expect("Decoding value should work.");
        assert_eq!(*value, decoded);
    }
}
