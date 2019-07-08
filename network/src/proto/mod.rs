// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![allow(bare_trait_objects)]

//! Protobuf definitions for data structures sent over the network
mod consensus;
mod mempool;
mod network;

use types::proto::{ledger_info, transaction};

pub use self::{
    consensus::{
        Block, BlockRetrievalStatus, ConsensusMsg, PacemakerTimeout, PacemakerTimeoutCertificate,
        Proposal, QuorumCert, RequestBlock, RequestChunk, RespondBlock, RespondChunk, TimeoutMsg,
        Vote,
    },
    mempool::MempoolSyncMsg,
    network::{DiscoveryMsg, IdentityMsg, Note, PeerInfo, Ping, Pong},
};
pub use transaction::SignedTransaction;
