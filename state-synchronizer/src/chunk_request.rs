// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_types::{ledger_info::LedgerInfoWithSignatures, transaction::Version};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
/// We're currently considering several types of chunk requests depending on the information
/// available on the requesting side.
pub enum TargetType {
    /// The response is built relative to the target (or end of epoch).
    TargetLedgerInfo(LedgerInfoWithSignatures),
    /// The response is built relative to the highest available LedgerInfo (or end of epoch).
    /// The value specifies the timeout in ms to wait for an available response.
    /// This "long poll" approach allows an upstream node to add the request to the list of its
    /// subscriptions for the duration of a timeout until some new information becomes available.
    HighestAvailable {
        /// Sometimes if the sync requester is lagging too much behind the sync responder, the requester will
        /// want to sync to a version earlier than the highest LI version on the sync responder.
        /// While it is catching up, it will want to ask all the intermediary highest ledger infos
        /// on the responder as the responder continues to advance in committing blocks
        /// so that as the sync requester is catching up, it will use these intermediary highest LIs
        /// to commit with the synced txns, so that they show up as actually committed, not just synced.
        target_li: Option<LedgerInfoWithSignatures>,
        timeout_ms: u64,
    },
    /// The response is built relative to a LedgerInfo at a given version.
    Waypoint(Version),
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct GetChunkRequest {
    /// The response should start with `known_version + 1`.
    pub known_version: Version,
    /// Epoch the chunk response is supposed to belong to (i.e., epoch of known_version + 1).
    pub current_epoch: u64,
    /// Max size of a chunk response.
    pub limit: u64,
    /// The target of the given request.
    target: TargetType,
}

impl GetChunkRequest {
    pub fn new(known_version: Version, current_epoch: u64, limit: u64, target: TargetType) -> Self {
        Self {
            known_version,
            current_epoch,
            limit,
            target,
        }
    }

    pub fn target(&self) -> &TargetType {
        &self.target
    }
}

impl fmt::Display for GetChunkRequest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[ChunkRequest: known version: {}, epoch: {}, limit: {}, target: {:?}]",
            self.known_version,
            self.current_epoch,
            self.limit,
            self.target(),
        )
    }
}
