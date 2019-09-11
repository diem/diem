// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    loaded_data::{struct_def::StructDef, types::Type},
    value::{MutVal, Value},
};
use canonical_serialization::*;
use failure::prelude::*;
use std::convert::TryFrom;
use types::{account_address::AccountAddress, byte_array::ByteArray, vm_error::StatusCode};
use vm::{errors::*, vm_string::VMString};

impl Value {
    /// Serialize this value using `SimpleSerializer`.
    pub fn simple_serialize(&self) -> Option<Vec<u8>> {
        SimpleSerializer::<Vec<u8>>::serialize(self).ok()
    }

    /// Deserialize this value using `SimpleDeserializer` and a provided struct definition.
    pub fn simple_deserialize(blob: &[u8], resource: StructDef) -> VMResult<Value> {
        let mut deserializer = SimpleDeserializer::new(blob);
        deserialize_struct(&mut deserializer, &resource)
    }
}

fn deserialize_struct(
    deserializer: &mut SimpleDeserializer,
    struct_def: &StructDef,
) -> VMResult<Value> {
    let mut s_vals: Vec<MutVal> = Vec::new();
    for field_type in struct_def.field_definitions() {
        match field_type {
            Type::Bool => {
                if let Ok(b) = deserializer.decode_bool() {
                    s_vals.push(MutVal::new(Value::Bool(b)));
                } else {
                    return Err(vm_error(Location::new(), StatusCode::DATA_FORMAT_ERROR));
                }
            }
            Type::U64 => {
                if let Ok(val) = deserializer.decode_u64() {
                    s_vals.push(MutVal::new(Value::U64(val)));
                } else {
                    return Err(vm_error(Location::new(), StatusCode::DATA_FORMAT_ERROR));
                }
            }
            Type::String => {
                if let Ok(bytes) = deserializer.decode_bytes() {
                    if let Ok(s) = VMString::from_utf8(bytes) {
                        s_vals.push(MutVal::new(Value::String(s)));
                        continue;
                    }
                }
                return Err(vm_error(Location::new(), StatusCode::DATA_FORMAT_ERROR));
            }
            Type::ByteArray => {
                if let Ok(bytes) = deserializer.decode_bytes() {
                    s_vals.push(MutVal::new(Value::ByteArray(ByteArray::new(bytes))));
                    continue;
                }
                return Err(vm_error(Location::new(), StatusCode::DATA_FORMAT_ERROR));
            }
            Type::Address => {
                if let Ok(bytes) = deserializer.decode_bytes() {
                    if let Ok(addr) = AccountAddress::try_from(bytes) {
                        s_vals.push(MutVal::new(Value::Address(addr)));
                        continue;
                    }
                }
                return Err(vm_error(Location::new(), StatusCode::DATA_FORMAT_ERROR));
            }
            Type::Struct(s_fields) => {
                if let Ok(s) = deserialize_struct(deserializer, s_fields) {
                    s_vals.push(MutVal::new(s));
                } else {
                    return Err(vm_error(Location::new(), StatusCode::DATA_FORMAT_ERROR));
                }
            }
            Type::Reference(_) => return Err(vm_error(Location::new(), StatusCode::INVALID_DATA)),
            Type::MutableReference(_) => {
                return Err(vm_error(Location::new(), StatusCode::INVALID_DATA))
            }
            Type::TypeVariable(_) => {
                // This case is not possible as we disallow calls like borrow_global<Foo<Coin>>()
                // for now.
                return Err(vm_error(Location::new(), StatusCode::INVALID_DATA));
            }
        }
    }
    Ok(Value::Struct(s_vals))
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
        }
        Ok(())
    }
}
