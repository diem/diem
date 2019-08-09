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
use crate::chained_bft::consensus_types::quorum_cert::QuorumCert;
use crypto::HashValue;
use failure::prelude::*;
use nextgen_crypto::*;
use proto_conv::{FromProtoBytes, IntoProtoBytes};
use schemadb::schema::{KeyCodec, Schema, ValueCodec};
use std::{fmt::Debug, marker::PhantomData};

// Polymorphic variant of
// define_schema!(QCSchema, HashValue, QuorumCert, QC_CF_NAME);
pub(crate) struct QCSchema<Sig> {
    _marker: PhantomData<Sig>,
}

impl<Sig: Signature> Schema for QCSchema<Sig> {
    const COLUMN_FAMILY_NAME: schemadb::ColumnFamilyName = QC_CF_NAME;
    type Key = HashValue;
    type Value = QuorumCert<Sig>;
}

impl<Sig: Debug + Signature> KeyCodec<QCSchema<Sig>> for HashValue {
    fn encode_key(&self) -> Result<Vec<u8>> {
        Ok(self.to_vec())
    }

    fn decode_key(data: &[u8]) -> Result<Self> {
        Ok(HashValue::from_slice(data)?)
    }
}

impl<Sig: Signature> ValueCodec<QCSchema<Sig>> for QuorumCert<Sig> {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.clone().into_proto_bytes()
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Self::from_proto_bytes(data)
    }
}

#[cfg(test)]
mod test;
