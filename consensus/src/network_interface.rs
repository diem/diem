// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Interface between Consensus and Network layers.

use crate::counters;
use channel::message_queues::QueueStyle;
use consensus_types::{
    block_retrieval::{BlockRetrievalRequest, BlockRetrievalResponse},
    epoch_retrieval::EpochRetrievalRequest,
    proposal_msg::ProposalMsg,
    sync_info::SyncInfo,
    vote_msg::VoteMsg,
};
use libra_metrics::IntCounterVec;
use libra_types::{epoch_change::EpochChangeProof, PeerId};
use network::{
    constants::NETWORK_CHANNEL_SIZE,
    error::NetworkError,
    peer_manager::{ConnectionRequestSender, PeerManagerRequestSender},
    protocols::{
        network::{NetworkEvents, NetworkSender, NewNetworkSender},
        rpc::error::RpcError,
    },
    ProtocolId,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Network type for consensus
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ConsensusMsg {
    /// RPC to get a chain of block of the given length starting from the given block id.
    BlockRetrievalRequest(Box<BlockRetrievalRequest>),
    /// Carries the returned blocks and the retrieval status.
    BlockRetrievalResponse(Box<BlockRetrievalResponse>),
    /// Request to get a EpochChangeProof from current_epoch to target_epoch
    EpochRetrievalRequest(Box<EpochRetrievalRequest>),
    /// ProposalMsg contains the required information for the proposer election protocol to make
    /// its choice (typically depends on round and proposer info).
    ProposalMsg(Box<ProposalMsg>),
    /// This struct describes basic synchronization metadata.
    SyncInfo(Box<SyncInfo>),
    /// A vector of LedgerInfo with contiguous increasing epoch numbers to prove a sequence of
    /// epoch changes from the first LedgerInfo's epoch.
    EpochChangeProof(Box<EpochChangeProof>),
    /// VoteMsg is the struct that is ultimately sent by the voter in response for receiving a
    /// proposal.
    VoteMsg(Box<VoteMsg>),
}

/// The interface from Network to Consensus layer.
///
/// `ConsensusNetworkEvents` is a `Stream` of `PeerManagerNotification` where the
/// raw `Bytes` direct-send and rpc messages are deserialized into
/// `ConsensusMessage` types. `ConsensusNetworkEvents` is a thin wrapper around
/// an `channel::Receiver<PeerManagerNotification>`.
pub type ConsensusNetworkEvents = NetworkEvents<ConsensusMsg>;

/// The interface from Consensus to Networking layer.
///
/// This is a thin wrapper around a `NetworkSender<ConsensusMsg>`, so it is easy
/// to clone and send off to a separate task. For example, the rpc requests
/// return Futures that encapsulate the whole flow, from sending the request to
/// remote, to finally receiving the response and deserializing. It therefore
/// makes the most sense to make the rpc call on a separate async task, which
/// requires the `ConsensusNetworkSender` to be `Clone` and `Send`.
#[derive(Clone)]
pub struct ConsensusNetworkSender {
    network_sender: NetworkSender<ConsensusMsg>,
}

/// Configuration for the network endpoints to support consensus.
pub fn network_endpoint_config() -> (
    Vec<ProtocolId>,
    Vec<ProtocolId>,
    QueueStyle,
    usize,
    Option<&'static IntCounterVec>,
) {
    (
        vec![ProtocolId::ConsensusRpc],
        vec![ProtocolId::ConsensusDirectSend],
        QueueStyle::LIFO,
        NETWORK_CHANNEL_SIZE,
        Some(&counters::PENDING_CONSENSUS_NETWORK_EVENTS),
    )
}

impl NewNetworkSender for ConsensusNetworkSender {
    /// Returns a Sender that only sends for the `CONSENSUS_DIRECT_SEND_PROTOCOL` and
    /// `CONSENSUS_RPC_PROTOCOL` ProtocolId.
    fn new(
        peer_mgr_reqs_tx: PeerManagerRequestSender,
        connection_reqs_tx: ConnectionRequestSender,
    ) -> Self {
        Self {
            network_sender: NetworkSender::new(peer_mgr_reqs_tx, connection_reqs_tx),
        }
    }
}

impl ConsensusNetworkSender {
    /// Send a single message to the destination peer using the `CONSENSUS_DIRECT_SEND_PROTOCOL`
    /// ProtocolId.
    pub fn send_to(
        &mut self,
        recipient: PeerId,
        message: ConsensusMsg,
    ) -> Result<(), NetworkError> {
        let protocol = ProtocolId::ConsensusDirectSend;
        self.network_sender.send_to(recipient, protocol, message)
    }

    /// Send a single message to the destination peers using the `CONSENSUS_DIRECT_SEND_PROTOCOL`
    /// ProtocolId.
    pub fn send_to_many(
        &mut self,
        recipients: impl Iterator<Item = PeerId>,
        message: ConsensusMsg,
    ) -> Result<(), NetworkError> {
        let protocol = ProtocolId::ConsensusDirectSend;
        self.network_sender
            .send_to_many(recipients, protocol, message)
    }

    /// Send a RPC to the destination peer using the `CONSENSUS_RPC_PROTOCOL` ProtocolId.
    pub async fn send_rpc(
        &mut self,
        recipient: PeerId,
        message: ConsensusMsg,
        timeout: Duration,
    ) -> Result<ConsensusMsg, RpcError> {
        let protocol = ProtocolId::ConsensusRpc;
        self.network_sender
            .send_rpc(recipient, protocol, message, timeout)
            .await
    }
}
