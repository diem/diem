// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_address::AccountAddress;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum TransactionArgument {
    U8(u8),
    U64(u64),
    U128(u128),
    Address(AccountAddress),
    U8Vector(#[serde(with = "serde_bytes")] Vec<u8>),
    Bool(bool),

    U64Vector(Vec<u64>),
    U128Vector(Vec<u128>),
    BoolVector(Vec<bool>),
    AddressVector(Vec<AccountAddress>),

    /// Generic vector for the remaining cases.
    /// Note that the specialized vectors above (U8Vector, U64Vector, etc) must be used when applicable.
    Vector(Vec<TransactionArgument>),
}

impl fmt::Debug for TransactionArgument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransactionArgument::U8(value) => write!(f, "{{U8: {}}}", value),
            TransactionArgument::U64(value) => write!(f, "{{U64: {}}}", value),
            TransactionArgument::U128(value) => write!(f, "{{U128: {}}}", value),
            TransactionArgument::Bool(boolean) => write!(f, "{{BOOL: {}}}", boolean),
            TransactionArgument::Address(address) => write!(f, "{{ADDRESS: {:?}}}", address),
            TransactionArgument::U8Vector(vector) => {
                write!(f, "{{U8Vector: 0x{}}}", hex::encode(vector))
            }
            TransactionArgument::U64Vector(vector) => write!(f, "{{U64Vector: {:?}}}", vector),
            TransactionArgument::U128Vector(vector) => write!(f, "{{U128Vector: {:?}}}", vector),
            TransactionArgument::BoolVector(vector) => write!(f, "{{BoolVector: {:?}}}", vector),
            TransactionArgument::AddressVector(vector) => {
                write!(f, "{{AddressVector: {:?}}}", vector)
            }
            TransactionArgument::Vector(values) => write!(f, "{{Vector: {:?}}}", values),
        }
    }
}
