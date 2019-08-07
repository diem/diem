// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_address::AccountAddress,
    byte_array::ByteArray,
    proto::transaction::{TransactionArgument as ProtoArgument, TransactionArgument_ArgType},
};
use byteorder::{LittleEndian, WriteBytesExt};
use canonical_serialization::{
    CanonicalDeserialize, CanonicalDeserializer, CanonicalSerialize, CanonicalSerializer,
};
use failure::prelude::*;
use proto_conv::{FromProto, IntoProto};
use protobuf::ProtobufEnum;
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, fmt};

pub const SCRIPT_HASH_LENGTH: usize = 32;

#[derive(Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Program {
    code: Vec<u8>,
    args: Vec<TransactionArgument>,
    modules: Vec<Vec<u8>>,
}

impl Program {
    pub fn new(code: Vec<u8>, modules: Vec<Vec<u8>>, args: Vec<TransactionArgument>) -> Program {
        Program {
            code,
            modules,
            args,
        }
    }

    pub fn code(&self) -> &[u8] {
        &self.code
    }

    pub fn args(&self) -> &[TransactionArgument] {
        &self.args
    }

    pub fn modules(&self) -> &[Vec<u8>] {
        &self.modules
    }

    pub fn into_inner(self) -> (Vec<u8>, Vec<TransactionArgument>, Vec<Vec<u8>>) {
        (self.code, self.args, self.modules)
    }
}

impl fmt::Debug for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // XXX note that "code" will eventually be encoded bytecode and will no longer be a
        // UTF8-ish string -- at that point the from_utf8_lossy will stop making sense.
        f.debug_struct("Program")
            .field("code", &String::from_utf8_lossy(&self.code))
            .field("args", &self.args)
            .finish()
    }
}

impl FromProto for Program {
    type ProtoType = crate::proto::transaction::Program;

    fn from_proto(proto_program: Self::ProtoType) -> Result<Self> {
        let mut args = vec![];
        for arg in proto_program.get_arguments() {
            let argument = match arg.get_field_type() {
                TransactionArgument_ArgType::U64 => {
                    let mut bytes = [0u8; 8];
                    let data = arg.get_data();
                    ensure!(
                        bytes.len() == data.len(),
                        "data has incorrect length: expected {} bytes, found {} bytes",
                        bytes.len(),
                        data.len()
                    );
                    bytes.copy_from_slice(arg.get_data());
                    let amount = u64::from_le_bytes(bytes);
                    TransactionArgument::U64(amount)
                }
                TransactionArgument_ArgType::ADDRESS => {
                    TransactionArgument::Address(AccountAddress::try_from(arg.get_data())?)
                }
                TransactionArgument_ArgType::STRING => {
                    TransactionArgument::String(String::from_utf8(arg.get_data().to_vec())?)
                }
                TransactionArgument_ArgType::BYTEARRAY => {
                    TransactionArgument::ByteArray(ByteArray::new(arg.get_data().to_vec()))
                }
            };
            args.push(argument);
        }
        let mut modules = vec![];
        for m in proto_program.get_modules() {
            modules.push(m.to_vec());
        }
        Ok(Program::new(
            proto_program.get_code().to_vec(),
            modules,
            args,
        ))
    }
}

impl IntoProto for Program {
    type ProtoType = crate::proto::transaction::Program;

    fn into_proto(self) -> Self::ProtoType {
        let mut proto_program = Self::ProtoType::new();
        proto_program.set_code(self.code);
        for arg in self.args {
            let mut argument = ProtoArgument::new();

            match arg {
                TransactionArgument::U64(amount) => {
                    argument.set_field_type(TransactionArgument_ArgType::U64);
                    let mut amount_vec = vec![];
                    amount_vec
                        .write_u64::<LittleEndian>(amount)
                        .expect("Writing to a vec is guaranteed to work");
                    argument.set_data(amount_vec);
                }
                TransactionArgument::Address(address) => {
                    argument.set_field_type(TransactionArgument_ArgType::ADDRESS);
                    argument.set_data(address.as_ref().to_vec());
                }
                TransactionArgument::String(string) => {
                    argument.set_field_type(TransactionArgument_ArgType::STRING);
                    argument.set_data(string.into_bytes());
                }
                TransactionArgument::ByteArray(byte_array) => {
                    argument.set_field_type(TransactionArgument_ArgType::BYTEARRAY);
                    argument.set_data(byte_array.as_bytes().to_vec())
                }
            }
            proto_program.mut_arguments().push(argument);
        }
        for m in self.modules {
            proto_program.mut_modules().push(m);
        }
        proto_program
    }
}

#[derive(Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum TransactionArgument {
    U64(u64),
    Address(AccountAddress),
    ByteArray(ByteArray),
    String(String),
}

impl fmt::Debug for TransactionArgument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransactionArgument::U64(value) => write!(f, "{{U64: {}}}", value),
            TransactionArgument::Address(address) => write!(f, "{{ADDRESS: {:?}}}", address),
            TransactionArgument::String(string) => write!(f, "{{STRING: {}}}", string),
            TransactionArgument::ByteArray(byte_array) => {
                write!(f, "{{ByteArray: 0x{}}}", byte_array)
            }
        }
    }
}

impl CanonicalSerialize for TransactionArgument {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        match self {
            TransactionArgument::U64(value) => {
                serializer.encode_u32(TransactionArgument_ArgType::U64 as u32)?;
                serializer.encode_u64(*value)?;
            }
            TransactionArgument::Address(address) => {
                serializer.encode_u32(TransactionArgument_ArgType::ADDRESS as u32)?;
                serializer.encode_struct(address)?;
            }
            TransactionArgument::String(string) => {
                serializer.encode_u32(TransactionArgument_ArgType::STRING as u32)?;
                serializer.encode_string(string)?;
            }
            TransactionArgument::ByteArray(byte_array) => {
                serializer.encode_u32(TransactionArgument_ArgType::BYTEARRAY as u32)?;
                serializer.encode_struct(byte_array)?;
            }
        }

        Ok(())
    }
}

impl CanonicalDeserialize for TransactionArgument {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self> {
        let decoded_value = deserializer.decode_u32()? as i32;
        let arg_type = TransactionArgument_ArgType::from_i32(decoded_value);
        match arg_type {
            Some(TransactionArgument_ArgType::U64) => {
                Ok(TransactionArgument::U64(deserializer.decode_u64()?))
            }
            Some(TransactionArgument_ArgType::ADDRESS) => {
                Ok(TransactionArgument::Address(deserializer.decode_struct()?))
            }
            Some(TransactionArgument_ArgType::STRING) => {
                Ok(TransactionArgument::String(deserializer.decode_string()?))
            }
            Some(TransactionArgument_ArgType::BYTEARRAY) => Ok(TransactionArgument::ByteArray(
                deserializer.decode_struct()?,
            )),
            None => Err(format_err!(
                "ParseError: Unable to decode TransactionArgument_ArgType, found {}",
                decoded_value
            )),
        }
    }
}
