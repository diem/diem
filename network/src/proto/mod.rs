// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![allow(bare_trait_objects)]

//! Protobuf definitions for data structures sent over the network
mod consensus;
mod mempool;
mod network;
mod state_synchronizer;

use types::proto::{ledger_info, transaction};

pub use self::{
    consensus::{
        Block, BlockRetrievalStatus, ConsensusMsg, PacemakerTimeout, PacemakerTimeoutCertificate,
        Proposal, QuorumCert, RequestBlock, RespondBlock, SyncInfo, TimeoutMsg, Vote,
    },
    mempool::MempoolSyncMsg,
    network::{DiscoveryMsg, IdentityMsg, IdentityMsg_Role, Note, PeerInfo, Ping, Pong},
    state_synchronizer::{GetChunkRequest, GetChunkResponse, StateSynchronizerMsg},
};
pub use transaction::SignedTransaction;
