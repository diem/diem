// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    loaded_data::{struct_def::StructDef, types::Type},
    native_structs::serializer::deserialize_native,
    value::{MutVal, Value},
};
use canonical_serialization::*;
use failure::prelude::*;
use std::convert::TryFrom;
use types::{account_address::AccountAddress, byte_array::ByteArray};
use vm::errors::*;

impl Value {
    /// Serialize this value using `SimpleSerializer`.
    pub fn simple_serialize(&self) -> Option<Vec<u8>> {
        SimpleSerializer::<Vec<u8>>::serialize(self).ok()
    }

    /// Deserialize this value using `SimpleDeserializer` and a provided struct definition.
    pub fn simple_deserialize(blob: &[u8], resource: StructDef) -> VMRuntimeResult<Value> {
        let mut deserializer = SimpleDeserializer::new(blob);
        deserialize_struct(&mut deserializer, &resource)
    }
}

pub(crate) fn deserialize_value(
    deserializer: &mut SimpleDeserializer,
    ty: &Type,
) -> VMRuntimeResult<Value> {
    match ty {
        Type::Bool => deserializer.decode_bool().map(Value::Bool),
        Type::U64 => deserializer.decode_u64().map(Value::U64),
        Type::String => {
            if let Ok(bytes) = deserializer.decode_bytes() {
                if let Ok(s) = String::from_utf8(bytes) {
                    return Ok(Value::String(s));
                }
            }
            return Err(VMRuntimeError {
                loc: Location::new(),
                err: VMErrorKind::InvalidData,
            });
        }
        Type::ByteArray => deserializer
            .decode_bytes()
            .map(|bytes| Value::ByteArray(ByteArray::new(bytes))),
        Type::Address => deserializer
            .decode_bytes()
            .and_then(AccountAddress::try_from)
            .map(Value::Address),
        Type::Struct(s_fields) => Ok(deserialize_struct(deserializer, s_fields)?),
        Type::Reference(_) | Type::MutableReference(_) | Type::TypeVariable(_) => {
            // Case TypeVariable is not possible as all type variable has to be materialized before
            // serialization.
            return Err(VMRuntimeError {
                loc: Location::new(),
                err: VMErrorKind::InvalidData,
            });
        }
    }
    .map_err(|_| VMRuntimeError {
        loc: Location::new(),
        err: VMErrorKind::InvalidData,
    })
}

pub(crate) fn deserialize_struct(
    deserializer: &mut SimpleDeserializer,
    struct_def: &StructDef,
) -> VMRuntimeResult<Value> {
    match struct_def {
        StructDef::Struct(s) => {
            let mut s_vals: Vec<MutVal> = Vec::new();
            for field_type in s.field_definitions() {
                s_vals.push(MutVal::new(deserialize_value(deserializer, field_type)?));
            }
            Ok(Value::Struct(s_vals))
        }
        StructDef::Native(ty) => Ok(Value::Native(deserialize_native(deserializer, ty)?)),
    }
}

impl CanonicalSerialize for Value {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        match self {
            Value::Address(addr) => {
                // TODO: this is serializing as a vector but we want just raw bytes
                // however the AccountAddress story is a bit difficult to work with right now
                serializer.encode_bytes(addr.as_ref())?;
            }
            Value::Bool(b) => {
                serializer.encode_bool(*b)?;
            }
            Value::U64(val) => {
                serializer.encode_u64(*val)?;
            }
            Value::String(s) => {
                // TODO: must define an api for canonical serializations of string.
                // Right now we are just using Rust to serialize the string
                serializer.encode_bytes(s.as_bytes())?;
            }
            Value::Struct(vals) => {
                for mut_val in vals {
                    (*mut_val.peek()).serialize(serializer)?;
                }
            }
            Value::ByteArray(bytearray) => {
                serializer.encode_bytes(bytearray.as_bytes())?;
            }
            Value::Native(v) => {
                serializer.encode_struct(v)?;
            }
        }
        Ok(())
    }
}
