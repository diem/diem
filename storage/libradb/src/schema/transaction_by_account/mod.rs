// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines physical storage schema for a transaction index via which the version of a
//! transaction sent by `account_address` with `sequence_number` can be found. With the version one
//! can resort to `TransactionSchema` for the transaction content.
//!
//! ```text
//! |<-------key------->|<-value->|
//! | address | seq_num | txn_ver |
//! ```

use crate::schema::{ensure_slice_len_eq, TRANSACTION_BY_ACCOUNT_CF_NAME};
use anyhow::Result;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use libra_types::{
    account_address::{AccountAddress, ADDRESS_LENGTH},
    transaction::Version,
};
use schemadb::{
    define_schema,
    schema::{KeyCodec, ValueCodec},
};
use std::{convert::TryFrom, mem::size_of};

define_schema!(
    TransactionByAccountSchema,
    Key,
    Version,
    TRANSACTION_BY_ACCOUNT_CF_NAME
);

type SeqNum = u64;
type Key = (AccountAddress, SeqNum);

impl KeyCodec<TransactionByAccountSchema> for Key {
    fn encode_key(&self) -> Result<Vec<u8>> {
        let (ref account_address, seq_num) = *self;

        let mut encoded = account_address.to_vec();
        encoded.write_u64::<BigEndian>(seq_num)?;

        Ok(encoded)
    }

    fn decode_key(data: &[u8]) -> Result<Self> {
        ensure_slice_len_eq(data, size_of::<Self>())?;

        let address = AccountAddress::try_from(&data[..ADDRESS_LENGTH])?;
        let seq_num = (&data[ADDRESS_LENGTH..]).read_u64::<BigEndian>()?;

        Ok((address, seq_num))
    }
}

impl ValueCodec<TransactionByAccountSchema> for Version {
    fn encode_value(&self) -> Result<Vec<u8>> {
        Ok(self.to_be_bytes().to_vec())
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        ensure_slice_len_eq(data, size_of::<Self>())?;

        Ok((&data[..]).read_u64::<BigEndian>()?)
    }
}

#[cfg(test)]
mod test;
