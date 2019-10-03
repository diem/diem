// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![allow(bare_trait_objects)]

//! Protobuf definitions for data structures sent over the network
mod admission_control;
mod consensus;
mod state_synchronizer;

mod network {
    include!(concat!(env!("OUT_DIR"), "/network.rs"));
}

mod mempool {
    include!(concat!(env!("OUT_DIR"), "/mempool.rs"));
}

use ::types::proto::{ledger_info, transaction, types};

pub use self::{
    admission_control::{AdmissionControlMsg, SubmitTransactionRequest, SubmitTransactionResponse},
    consensus::{
        Block, BlockRetrievalStatus, ConsensusMsg, PacemakerTimeout, PacemakerTimeoutCertificate,
        Proposal, QuorumCert, RequestBlock, RespondBlock, SyncInfo, TimeoutMsg, Vote, VoteData,
    },
    mempool::MempoolSyncMsg,
    network::{
        identity_msg::Role as IdentityMsg_Role, DiscoveryMsg, FullNodePayload, IdentityMsg, Note,
        PeerInfo, Ping, Pong, SignedFullNodePayload, SignedPeerInfo,
    },
    state_synchronizer::{GetChunkRequest, GetChunkResponse, StateSynchronizerMsg},
};
