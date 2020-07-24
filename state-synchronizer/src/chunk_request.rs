// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_types::{ledger_info::LedgerInfoWithSignatures, transaction::Version};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Deserialize, Serialize, Clone, PartialEq, Eq)]
/// We're currently considering several types of chunk requests depending on the information
/// available on the requesting side.
pub enum TargetType {
    /// The response is built relative to the target (or end of epoch).
    TargetLedgerInfo(LedgerInfoWithSignatures),
    /// The response is built relative to the highest available LedgerInfo (or end of epoch).
    /// The value specifies the timeout in ms to wait for an available response.
    /// This "long poll" approach allows an upstream node to add the request to the list of its
    /// subscriptions for the duration of a timeout until some new information becomes available.
    ///
    /// `target_li`: While asking for the highest available ledger info, this request also provides
    /// the option to the sync requester to specify a target LI.
    /// This is to support the scenario where the sync requester is lagging too much behind the upstream node
    /// in the sync process. If the highest ledger info version keeps advancing on the upstream node,
    /// even though the sync requester continues to receive and sync txns, those txns will never be backed an LI,
    /// since a LI can only be committed once all the transactions up to the LI's version has been received.
    /// (It is important for a transaction to be backed by an LI, because transactions need to be backed by an LI
    /// to be shown as committed upon storage query)
    /// To prevent the above problem where the transactions are never backed by a LI during sync catch-up
    /// (or the difference between synced version and committed LI version keeps growing on sync requester),
    /// this `TargetType` can simultaneously (1) ask for the highest ledger info, and (2) specify a target
    /// to build the requested transactions w.r.t.. With (1), the sync requester can store the LI later to target-sync
    /// once it is ready for that LI after syncing to an earlier target LI via (2).
    ///
    /// If `target_li` is not specified, the upstream node will build the responses against its highest LI
    HighestAvailable {
        target_li: Option<LedgerInfoWithSignatures>,
        timeout_ms: u64,
    },
    /// The response is built relative to a LedgerInfo at a given version.
    Waypoint(Version),
}

impl fmt::Debug for TargetType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

// TODO: This cuts out LedgerInfo, do we need it logging?
impl fmt::Display for TargetType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TargetType::TargetLedgerInfo(ledger_info) => {
                write!(f, "TargetLedgerInfo({})", ledger_info)
            }
            TargetType::HighestAvailable {
                target_li,
                timeout_ms,
            } => write!(
                f,
                "HighestAvailable(timeout:{}, target_li_version:{})",
                timeout_ms,
                target_li.as_ref().map_or_else(
                    || String::from("None"),
                    |li| li.ledger_info().version().to_string()
                )
            ),
            TargetType::Waypoint(version) => write!(f, "Waypoint({})", version),
        }
    }
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Eq)]
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

impl fmt::Debug for GetChunkRequest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
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
