// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//use crate::errors::*;
use crate::{
    account_address::AccountAddress, byte_array::ByteArray,
    proto::transaction::TransactionArgument_ArgType,
};
use failure::prelude::*;
use libra_canonical_serialization::{
    CanonicalDeserialize, CanonicalDeserializer, CanonicalSerialize, CanonicalSerializer,
};
use protobuf::ProtobufEnum;
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, fmt};

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

#[derive(Clone, Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "ParseError: {}", _0)]
    ParseError(String),
}

/// Parses the given string as address.
pub fn parse_as_address(s: &str) -> Result<TransactionArgument> {
    let mut s = s.to_ascii_lowercase();
    if !s.starts_with("0x") {
        return Err(ErrorKind::ParseError("address must start with '0x'".to_string()).into());
    }
    if s.len() == 2 {
        return Err(ErrorKind::ParseError("address cannot be empty".to_string()).into());
    }
    if s.len() % 2 != 0 {
        s = format!("0x0{}", &s[2..]);
    }
    let mut addr = hex::decode(&s[2..])?;
    if addr.len() > 32 {
        return Err(ErrorKind::ParseError("address must be 32 bytes or less".to_string()).into());
    }
    if addr.len() < 32 {
        addr = vec![0u8; 32 - addr.len()]
            .into_iter()
            .chain(addr.into_iter())
            .collect();
    }
    Ok(TransactionArgument::Address(AccountAddress::try_from(
        addr,
    )?))
}

/// Parses the given string as bytearray.
pub fn parse_as_byte_array(s: &str) -> Result<TransactionArgument> {
    if s.starts_with("b\"") && s.ends_with('"') && s.len() >= 3 {
        let s = &s[2..s.len() - 1];
        if s.is_empty() {
            return Err(ErrorKind::ParseError("byte array cannot be empty".to_string()).into());
        }
        let s = if s.len() % 2 == 0 {
            s.to_string()
        } else {
            format!("0{}", s)
        };
        Ok(TransactionArgument::ByteArray(ByteArray::new(hex::decode(
            &s,
        )?)))
    } else {
        Err(ErrorKind::ParseError(format!("\"{}\" is not a byte array", s)).into())
    }
}

/// Parses the given string as u64.
pub fn parse_as_u64(s: &str) -> Result<TransactionArgument> {
    Ok(TransactionArgument::U64(s.parse::<u64>()?))
}

macro_rules! return_if_ok {
    ($e: expr) => {{
        if let Ok(res) = $e {
            return Ok(res);
        }
    }};
}

/// Parses the given string as any transaction argument type.
pub fn parse_as_transaction_argument(s: &str) -> Result<TransactionArgument> {
    return_if_ok!(parse_as_address(s));
    return_if_ok!(parse_as_u64(s));
    return_if_ok!(parse_as_byte_array(s));
    Err(ErrorKind::ParseError(format!("cannot parse \"{}\" as transaction argument", s)).into())
}

#[cfg(test)]
mod test_transaction_argument {
    use crate::transaction::transaction_argument::*;

    #[test]
    fn parse_u64() {
        for s in &["0", "42", "18446744073709551615"] {
            parse_as_u64(s).unwrap();
        }
        for s in &["xx", "", "-3"] {
            parse_as_u64(s).unwrap_err();
        }
    }

    #[test]
    fn parse_address() {
        for s in &[
            "0x0",
            "0x1",
            "0x00",
            "0x05",
            "0x100",
            "0xAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        ] {
            parse_as_address(s).unwrap();
        }

        for s in &[
            "0x",
            "100",
            "",
            "0xG",
            "0xBBAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        ] {
            parse_as_address(s).unwrap_err();
        }
    }

    #[test]
    fn parse_byte_array() {
        for s in &["0", "00", "deadbeef", "aaa"] {
            parse_as_byte_array(&format!("b\"{}\"", s)).unwrap();
        }

        for s in &["", "b\"\"", "123", "b\"G\""] {
            parse_as_byte_array(s).unwrap_err();
        }
    }

    #[test]
    fn parse_args() {
        for s in &["123", "0xf", "b\"aaa\""] {
            parse_as_transaction_argument(s).unwrap();
        }

        for s in &["garbage", ""] {
            parse_as_transaction_argument(s).unwrap_err();
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
