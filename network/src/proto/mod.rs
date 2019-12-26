// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![allow(bare_trait_objects)]
#![allow(clippy::large_enum_variant)]

//! Protobuf definitions for data structures sent over the network
pub mod consensus {
    include!(concat!(env!("OUT_DIR"), "/consensus.rs"));
}
mod network {
    include!(concat!(env!("OUT_DIR"), "/network.rs"));
}
mod mempool {
    include!(concat!(env!("OUT_DIR"), "/mempool.rs"));
}
mod state_synchronizer {
    include!(concat!(env!("OUT_DIR"), "/state_synchronizer.rs"));
}
mod health_checker {
    include!(concat!(env!("OUT_DIR"), "/health_checker.rs"));
}
mod chain_state {
    include!(concat!(env!("OUT_DIR"), "/chain_state.rs"));
}

use ::libra_types::proto::types;

pub use self::{
    chain_state::{
        chain_state_msg::Message as ChainStateMsg_oneof, ChainStateMsg, ChainStateRequest,
        ChainStateResponse,
    },
    consensus::{
        consensus_msg::Message as ConsensusMsg_oneof, Block, BlockPayloadExt, ConsensusMsg,
        Proposal, RequestBlock, RequestEpoch, RespondBlock, SyncInfo, VoteMsg, VoteProposal,
    },
    health_checker::{
        health_checker_msg::Message as HealthCheckerMsg_oneof, HealthCheckerMsg, Ping, Pong,
    },
    mempool::MempoolSyncMsg,
    network::{
        identity_msg::Role as IdentityMsg_Role, DiscoveryMsg, FullNodePayload, IdentityMsg, Note,
        PeerInfo, SignedFullNodePayload, SignedPeerInfo,
    },
    state_synchronizer::{
        state_synchronizer_msg::Message as StateSynchronizerMsg_oneof, GetChunkRequest,
        GetChunkResponse, StateSynchronizerMsg,
    },
};
