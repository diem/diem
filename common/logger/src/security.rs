// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//!
//! The security module gathers security-related logs:
//! logs to detect malicious behavior from other validators.
//!
//! ```
//! use libra_logger::prelude::*;
//!
//! sl_error!(
//!   security_log(security_events::INVALID_RETRIEVED_BLOCK)
//!     .data("some_data", "the data")
//! );
//! ```
//!

use crate::StructuredLogEntry;

/// helper function to create a security log
pub fn security_log(name: &'static str) -> StructuredLogEntry {
    StructuredLogEntry::new_named("security", &name)
}

/// Security events that are possible
pub mod security_events {
    // Mempool
    // -------

    /// Mempool received a transaction from another peer with an invalid signature
    pub const INVALID_TRANSACTION_MP: &str = "InvalidTransactionMP";

    /// Mempool received an invalid network event
    pub const INVALID_NETWORK_EVENT_MP: &str = "INVALID_NETWORK_EVENT_MP";

    // Consensus
    // ---------

    /// Consensus received an invalid message (not well-formed or incorrect signature)
    pub const CONSENSUS_INVALID_MESSAGE: &str = "ConsensusInvalidMessage";

    /// Consensus received an equivocating vote
    pub const CONSENSUS_EQUIVOCATING_VOTE: &str = "ConsensusEquivocatingVote";

    /// Consensus received an invalid proposal
    pub const INVALID_CONSENSUS_PROPOSAL: &str = "InvalidConsensusProposal";

    /// Consensus received an invalid vote
    pub const INVALID_CONSENSUS_VOTE: &str = "InvalidConsensusVote";

    /// Consensus received an invalid new round message
    pub const INVALID_CONSENSUS_ROUND: &str = "InvalidConsensusRound";

    /// Consensus received an invalid sync info message
    pub const INVALID_SYNC_INFO_MSG: &str = "InvalidSyncInfoMsg";

    /// A received block is invalid
    pub const INVALID_RETRIEVED_BLOCK: &str = "InvalidRetrievedBlock";

    /// A block being committed or executed is invalid
    pub const INVALID_BLOCK: &str = "InvalidBlock";

    // State-Sync
    // ----------

    /// Invalid chunk of transactions received
    pub const STATE_SYNC_INVALID_CHUNK: &str = "InvalidChunk";

    // Health Checker
    // --------------

    /// HealthChecker received an invalid network event
    pub const INVALID_NETWORK_EVENT_HC: &str = "InvalidNetworkEventHC";

    /// HealthChecker received an invalid message
    pub const INVALID_HEALTHCHECKER_MSG: &str = "InvalidHealthCheckerMsg";

    // Network
    // -------

    /// Network identified an invalid peer
    pub const INVALID_NETWORK_PEER: &str = "InvalidNetworkPeer";

    /// Network couldn't negotiate
    pub const INVALID_NETWORK_HANDSHAKE_MSG: &str = "InvalidNetworkHandshakeMsg";
}
