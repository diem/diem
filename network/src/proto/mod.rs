// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![allow(bare_trait_objects)]

//! Protobuf definitions for data structures sent over the network
mod consensus;
mod mempool;
mod network;
mod state_synchronizer;
mod admission_control;

mod network_prost {
    include!(concat!(env!("OUT_DIR"), "/network.rs"));
}

use types::proto::{ledger_info, transaction};

pub use self::{
    admission_control::{AdmissionControlMsg, SubmitTransactionRequest, SubmitTransactionResponse},
    consensus::{
        Block, BlockRetrievalStatus, ConsensusMsg, PacemakerTimeout, PacemakerTimeoutCertificate,
        Proposal, QuorumCert, RequestBlock, RespondBlock, SyncInfo, TimeoutMsg, Vote, VoteData,
    },
    mempool::MempoolSyncMsg,
    network::{
        DiscoveryMsg, FullNodePayload, IdentityMsg, IdentityMsg_Role, Note, PeerInfo,
        SignedFullNodePayload, SignedPeerInfo,
    },
    network_prost::{Ping, Pong},
    state_synchronizer::{GetChunkRequest, GetChunkResponse, StateSynchronizerMsg},
};
pub use transaction::SignedTransaction;
