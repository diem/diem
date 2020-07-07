// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//!
//! The security module gathers security-related logs
//!
//! ```
//! use libra_logger::prelude::*;
//!
//! send_struct_log!(
//!   security_log(security_events::INVALID_RETRIEVED_BLOCK)
//!     .data("some_data", "the data")
//! );
//! ```
//!

use crate::StructuredLogEntry;

/// helper function to create a security log
pub fn security_log(name: &'static str) -> StructuredLogEntry {
    StructuredLogEntry::new_named("security", &name)
        // set the level to critical
        .critical()
    // set the error description
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

    /// Consensus received a transaction with an invalid signature
    pub const INVALID_TRANSACTION_CONSENSUS: &str = "InvalidTransactionConsensus";

    /// Consensus received an invalid vote
    pub const DUPLICATE_CONSENSUS_VOTE: &str = "DuplicateConsensusVote";

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

    // Executor
    // --------

    /// Executor received an invalid transactions chunk
    pub const INVALID_CHUNK_EXECUTOR: &str = "InvalidChunkExecutor";

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

    /// Network discovery received an invalid DiscoveryMsg
    pub const INVALID_DISCOVERY_MSG: &str = "InvalidDiscoveryMsg";
}
