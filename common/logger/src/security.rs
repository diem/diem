// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//!
//! The security module gathers security-related logs:
//! logs to detect malicious behavior from other validators.
//!
//! TODO: This likely belongs outside of the logging crate
//!
//! ```
//! use diem_logger::{error, SecurityEvent};
//!
//! error!(
//!     SecurityEvent::InvalidRetrievedBlock,
//!     "some_key" = "some data",
//! );
//! ```
//!

use crate::{Key, Schema, Value, Visitor};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecurityEvent {
    //
    // Mempool
    //
    /// Mempool received a transaction from another peer with an invalid signature
    InvalidTransactionMempool,

    /// Mempool received an invalid network event
    InvalidNetworkEventMempool,

    // Consensus
    // ---------
    /// Consensus received an invalid message (not well-formed, invalid vote data or incorrect signature)
    ConsensusInvalidMessage,

    /// Consensus received an equivocating vote
    ConsensusEquivocatingVote,

    /// Consensus received an invalid proposal
    InvalidConsensusProposal,

    /// Consensus received an invalid new round message
    InvalidConsensusRound,

    /// Consensus received an invalid sync info message
    InvalidSyncInfoMsg,

    /// A received block is invalid
    InvalidRetrievedBlock,

    /// A block being committed or executed is invalid
    InvalidBlock,

    // State-Sync
    // ----------
    /// Invalid chunk of transactions received
    StateSyncInvalidChunk,

    // Health Checker
    // --------------
    /// HealthChecker received an invalid network event
    InvalidNetworkEventHC,

    /// HealthChecker received an invalid message
    InvalidHealthCheckerMsg,

    // Network
    // -------
    /// Network received an invalid message from a remote peer
    InvalidNetworkEvent,

    /// A failed noise handshake that's either a clear bug or indicates some
    /// security issue.
    NoiseHandshake,
}

impl Schema for SecurityEvent {
    fn visit(&self, visitor: &mut dyn Visitor) {
        visitor.visit_pair(Key::new("security-event"), Value::from_serde(self))
    }
}
