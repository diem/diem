// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines physical storage schema for consensus quorum certificate (of a block).
//!
//! Serialized quorum certificate bytes identified by block_hash.
//! ```text
//! |<---key---->|<----value--->|
//! | block_hash |  QuorumCert  |
//! ```

use super::QC_CF_NAME;
use anyhow::Result;
use consensus_types::quorum_cert::QuorumCert;
use libra_crypto::HashValue;
use schemadb::{
    define_schema,
    schema::{KeyCodec, ValueCodec},
};

define_schema!(QCSchema, HashValue, QuorumCert, QC_CF_NAME);

impl KeyCodec<QCSchema> for HashValue {
    fn encode_key(&self) -> Result<Vec<u8>> {
        Ok(self.to_vec())
    }

    fn decode_key(data: &[u8]) -> Result<Self> {
        Ok(HashValue::from_slice(data)?)
    }
}

impl ValueCodec<QCSchema> for QuorumCert {
    fn encode_value(&self) -> Result<Vec<u8>> {
        Ok(lcs::to_bytes(self)?)
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Ok(lcs::from_bytes(data)?)
    }
}

#[cfg(test)]
mod test;
