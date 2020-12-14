// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::storage::FileHandle;
use anyhow::{ensure, Result};
use diem_types::transaction::Version;
use serde::{Deserialize, Serialize};

/// A chunk of a transaction backup manifest to represent the
/// [`first_version`, `last_version`] range (right side inclusive).
#[derive(Clone, Deserialize, Serialize)]
pub struct TransactionChunk {
    pub first_version: Version,
    pub last_version: Version,
    /// Repeated `len(record) + record`, where `record` is BCS serialized tuple
    /// `(Transaction, TransactionInfo)`
    pub transactions: FileHandle,
    /// BCS serialized `(TransactionAccumulatorRangeProof, LedgerInfoWithSignatures)`.
    /// The `TransactionAccumulatorRangeProof` links the transactions to the
    /// `LedgerInfoWithSignatures`, and the `LedgerInfoWithSignatures` can be verified by the
    /// signatures it carries, against the validator set in the epoch. (Hence proper
    /// `EpochEndingBackup` is needed for verification.)
    pub proof: FileHandle,
}

/// Transaction backup manifest, representing transactions in the
/// [`first_version`, `last_version`] range (right side inclusive).
#[derive(Deserialize, Serialize)]
pub struct TransactionBackup {
    pub first_version: Version,
    pub last_version: Version,
    pub chunks: Vec<TransactionChunk>,
}

impl TransactionBackup {
    pub fn verify(&self) -> Result<()> {
        // check number of waypoints
        ensure!(
            self.first_version <= self.last_version,
            "Bad version range: [{}, {}]",
            self.first_version,
            self.last_version,
        );

        // check chunk ranges
        ensure!(!self.chunks.is_empty(), "No chunks.");

        let mut next_version = self.first_version;
        for chunk in &self.chunks {
            ensure!(
                chunk.first_version == next_version,
                "Chunk ranges not continuous. Expected first version: {}, actual: {}.",
                next_version,
                chunk.first_version,
            );
            ensure!(
                chunk.last_version >= chunk.first_version,
                "Chunk range invalid. [{}, {}]",
                chunk.first_version,
                chunk.last_version,
            );
            next_version = chunk.last_version + 1;
        }

        // check last version in chunk matches manifest
        ensure!(
            next_version - 1 == self.last_version, // okay to -1 because chunks is not empty.
            "Last version in chunks: {}, in manifest: {}",
            next_version - 1,
            self.last_version,
        );

        Ok(())
    }
}
