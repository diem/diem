// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_address::AccountAddress;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, fmt};
use thiserror::Error;

#[derive(Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum TransactionArgument {
    U64(#[serde(with = "lcs::fixed_size")] u64),
    Address(AccountAddress),
    U8Vector(Vec<u8>),
    Bool(bool),
}

impl fmt::Debug for TransactionArgument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransactionArgument::U64(value) => write!(f, "{{U64: {}}}", value),
            TransactionArgument::Bool(boolean) => write!(f, "{{BOOL: {}}}", boolean),
            TransactionArgument::Address(address) => write!(f, "{{ADDRESS: {:?}}}", address),
            TransactionArgument::U8Vector(vector) => {
                write!(f, "{{U8Vector: 0x{}}}", hex::encode(vector))
            }
        }
    }
}

#[derive(Clone, Debug, Error)]
pub enum ErrorKind {
    #[error("ParseError: {0}")]
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
    if addr.len() > AccountAddress::LENGTH {
        return Err(ErrorKind::ParseError(format!(
            "address must be {} bytes or less",
            AccountAddress::LENGTH
        ))
        .into());
    }
    if addr.len() < AccountAddress::LENGTH {
        addr = vec![0u8; AccountAddress::LENGTH - addr.len()]
            .into_iter()
            .chain(addr.into_iter())
            .collect();
    }
    Ok(TransactionArgument::Address(AccountAddress::try_from(
        addr,
    )?))
}

/// Parses the given string as bytearray.
pub fn parse_as_u8_vector(s: &str) -> Result<TransactionArgument> {
    if s.starts_with("b\"") && s.ends_with('"') && s.len() >= 3 {
        let s = &s[2..s.len() - 1];
        if s.is_empty() {
            return Err(ErrorKind::ParseError("vector<u8> cannot be empty".to_string()).into());
        }
        let s = if s.len() % 2 == 0 {
            s.to_string()
        } else {
            format!("0{}", s)
        };
        Ok(TransactionArgument::U8Vector(hex::decode(&s)?))
    } else {
        Err(ErrorKind::ParseError(format!("\"{}\" is not a vector<u8>", s)).into())
    }
}

/// Parses the given string as u64.
pub fn parse_as_u64(s: &str) -> Result<TransactionArgument> {
    Ok(TransactionArgument::U64(s.parse::<u64>()?))
}

/// Parses the given string as a bool.
pub fn parse_as_bool(s: &str) -> Result<TransactionArgument> {
    Ok(TransactionArgument::Bool(s.parse::<bool>()?))
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
    return_if_ok!(parse_as_bool(s));
    return_if_ok!(parse_as_u8_vector(s));
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
    fn parse_bool() {
        parse_as_bool("true").unwrap();
        parse_as_bool("false").unwrap();
    }

    #[test]
    fn parse_address() {
        for s in &[
            "0x0",
            "0x1",
            "0x00",
            "0x05",
            "0x100",
            "0x0123456789ABCDEF0123456789ABCDEF",
        ] {
            parse_as_address(s).unwrap();
        }

        for s in &[
            "0x",
            "100",
            "",
            "0xG",
            "0x0123456789ABCDEF0123456789ABCDEFA",
        ] {
            parse_as_address(s).unwrap_err();
        }
    }

    #[test]
    fn parse_byte_array() {
        for s in &["0", "00", "deadbeef", "aaa"] {
            parse_as_u8_vector(&format!("b\"{}\"", s)).unwrap();
        }

        for s in &["", "b\"\"", "123", "b\"G\""] {
            parse_as_u8_vector(s).unwrap_err();
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
