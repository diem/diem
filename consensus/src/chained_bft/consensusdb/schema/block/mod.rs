// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines physical storage schema for consensus block.
//!
//! Serialized block bytes identified by block_hash.
//! ```text
//! |<---key---->|<---value--->|
//! | block_hash |    block    |
//! ```

use super::BLOCK_CF_NAME;
use crate::chained_bft::{common::Payload, consensus_types::block::Block};
use crypto::HashValue;
use failure::prelude::*;
use nextgen_crypto::*;
use proto_conv::{FromProtoBytes, IntoProtoBytes};
use schemadb::schema::{KeyCodec, Schema, ValueCodec};
use std::marker::PhantomData;

pub struct BlockSchema<T: Payload, Sig: Signature> {
    phantom: PhantomData<(T, Sig)>,
}

impl<T: Payload, Sig: Signature> Schema for BlockSchema<T, Sig>
where
    Sig::SigningKeyMaterial: Genesis,
{
    const COLUMN_FAMILY_NAME: schemadb::ColumnFamilyName = BLOCK_CF_NAME;
    type Key = HashValue;
    type Value = Block<T, Sig>;
}

impl<T: Payload, Sig: Signature> KeyCodec<BlockSchema<T, Sig>> for HashValue
where
    Sig::SigningKeyMaterial: Genesis,
{
    fn encode_key(&self) -> Result<Vec<u8>> {
        Ok(self.to_vec())
    }

    fn decode_key(data: &[u8]) -> Result<Self> {
        Ok(HashValue::from_slice(data)?)
    }
}

impl<T: Payload, Sig: Signature> ValueCodec<BlockSchema<T, Sig>> for Block<T, Sig>
where
    Sig::SigningKeyMaterial: Genesis,
{
    fn encode_value(&self) -> Result<Vec<u8>> {
        Ok(self.clone().into_proto_bytes()?)
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Ok(Self::from_proto_bytes(data)?)
    }
}

#[cfg(test)]
mod test;
