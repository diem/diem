// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::errors::*;
use std::convert::TryFrom;
use types::{
    account_address::AccountAddress, byte_array::ByteArray, transaction::TransactionArgument,
};

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
