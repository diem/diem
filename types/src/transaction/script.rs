// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_address::AccountAddress,
    byte_array::ByteArray,
    proto::transaction::{TransactionArgument as ProtoArgument, TransactionArgument_ArgType},
    transaction::transaction_argument::TransactionArgument,
};
use byteorder::{LittleEndian, WriteBytesExt};
use canonical_serialization::{
    CanonicalDeserialize, CanonicalDeserializer, CanonicalSerialize, CanonicalSerializer,
};
use failure::prelude::*;
use proto_conv::{FromProto, IntoProto};
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, fmt};

#[allow(dead_code)]
pub const SCRIPT_HASH_LENGTH: usize = 32;

#[derive(Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Script {
    code: Vec<u8>,
    args: Vec<TransactionArgument>,
}

impl Script {
    pub fn new(code: Vec<u8>, args: Vec<TransactionArgument>) -> Self {
        Script { code, args }
    }

    pub fn code(&self) -> &[u8] {
        &self.code
    }

    pub fn args(&self) -> &[TransactionArgument] {
        &self.args
    }

    pub fn into_inner(self) -> (Vec<u8>, Vec<TransactionArgument>) {
        (self.code, self.args)
    }
}

impl fmt::Debug for Script {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // XXX note that "code" will eventually be encoded bytecode and will no longer be a
        // UTF8-ish string -- at that point the from_utf8_lossy will stop making sense.
        f.debug_struct("Script")
            .field("code", &String::from_utf8_lossy(&self.code))
            .field("args", &self.args)
            .finish()
    }
}

impl FromProto for Script {
    type ProtoType = crate::proto::transaction::Script;

    fn from_proto(proto_script: Self::ProtoType) -> Result<Self> {
        let mut args = vec![];
        for arg in proto_script.get_arguments() {
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
        Ok(Script::new(proto_script.get_code().to_vec(), args))
    }
}

impl IntoProto for Script {
    type ProtoType = crate::proto::transaction::Script;

    fn into_proto(self) -> Self::ProtoType {
        let mut proto_script = Self::ProtoType::new();
        proto_script.set_code(self.code);
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
            proto_script.mut_arguments().push(argument);
        }
        proto_script
    }
}

impl CanonicalSerialize for Script {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer.encode_vec(&self.code)?;
        serializer.encode_vec(&self.args)?;
        Ok(())
    }
}

impl CanonicalDeserialize for Script {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self> {
        let code: Vec<u8> = deserializer.decode_vec()?;
        let args: Vec<TransactionArgument> = deserializer.decode_vec()?;
        Ok(Script::new(code, args))
    }
}
