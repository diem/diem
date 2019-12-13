// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines physical storage schema for TransactionInfo structure.
//!
//! Serialized signed transaction bytes identified by version.
//! ```text
//! |<--key-->|<-----value---->|
//! | version | txn_info bytes |
//! ```
//!
//! `Version` is serialized in big endian so that records in RocksDB will be in order of it's
//! numeric value.

use crate::schema::TRANSACTION_INFO_CF_NAME;
use anyhow::{ensure, Result};
use byteorder::{BigEndian, ReadBytesExt};
use libra_prost_ext::MessageExt;
use libra_types::transaction::{TransactionInfo, Version};
use prost::Message;
use schemadb::{
    define_schema,
    schema::{KeyCodec, ValueCodec},
};
use std::convert::TryInto;
use std::mem::size_of;

define_schema!(
    TransactionInfoSchema,
    Version,
    TransactionInfo,
    TRANSACTION_INFO_CF_NAME
);

impl KeyCodec<TransactionInfoSchema> for Version {
    fn encode_key(&self) -> Result<Vec<u8>> {
        Ok(self.to_be_bytes().to_vec())
    }

    fn decode_key(data: &[u8]) -> Result<Self> {
        ensure!(
            data.len() == size_of::<Version>(),
            "Bad num of bytes: {}",
            data.len()
        );
        Ok((&data[..]).read_u64::<BigEndian>()?)
    }
}

impl ValueCodec<TransactionInfoSchema> for TransactionInfo {
    fn encode_value(&self) -> Result<Vec<u8>> {
        let event: libra_types::proto::types::TransactionInfo = self.clone().into();
        Ok(event.to_vec()?)
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        libra_types::proto::types::TransactionInfo::decode(data)?.try_into()
    }
}

#[cfg(test)]
mod test;
