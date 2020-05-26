// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::storage::FileHandle;
use libra_crypto::HashValue;
use libra_types::transaction::Version;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct StateSnapshotChunk {
    /// index of the first account in this chunk over all accounts.
    pub first_idx: usize,
    /// index of the last account in this chunk over all accounts.
    pub last_idx: usize,
    /// key of the first account in this chunk.
    pub first_key: HashValue,
    /// key of the last account in this chunk.
    pub last_key: HashValue,
    /// Repeated `len(record) + record` where `record` is LCS serialized tuple
    /// `(key, account_state_blob)`
    pub blobs: FileHandle,
    /// LCS serialized `SparseMerkleRangeProof` that proves this chunk adds up to the root hash
    /// indicated in the backup (`StateSnapshotBackup::root_hash`).
    pub proof: FileHandle,
}

#[derive(Deserialize, Serialize)]
pub struct StateSnapshotBackup {
    /// Version at which this state snapshot is taken.
    pub version: Version,
    /// Hash of the state tree root.
    pub root_hash: HashValue,
    /// All account blobs in chunks.
    pub chunks: Vec<StateSnapshotChunk>,
    // LCS serialized
    // `Tuple(TransactionInfoWithProof, LedgerInfoWithSignatures)`.
    //   - The `TransactionInfoWithProof` is at `Version` above, and carries the same `root_hash`
    // above; It proves that at specified version the root hash is as specified in a chain
    // represented by the LedgerInfo below.
    //   - The signatures on the `LedgerInfoWithSignatures` has a version greater than or equal to
    // the version of this backup but is within the same epoch, so the signatures on it can be
    // verified by the validator set in the same epoch, which can be provided by an
    // `EpochStateBackup` recovered prior to this to the DB; Requiring it to be in the same epoch
    // limits the requirement on such `EpochStateBackup` to no older than the same epoch.
    pub proof: FileHandle,
}
