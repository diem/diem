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
use consensus_types::quorum_cert::QuorumCert;
use failure::prelude::*;
use libra_crypto::HashValue;
use libra_prost_ext::MessageExt;
use prost::Message;
use schemadb::{
    define_schema,
    schema::{KeyCodec, ValueCodec},
};
use std::convert::TryInto;

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
        let cert: network::proto::QuorumCert = self.clone().into();
        Ok(cert.to_vec()?)
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        network::proto::QuorumCert::decode(data)?.try_into()
    }
}

#[cfg(test)]
mod test;
