// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Interface between Consensus and Network layers.

use crate::counters;
use channel::message_queues::QueueStyle;
use consensus_types::{
    block_retrieval::{BlockRetrievalRequest, BlockRetrievalResponse},
    common::Payload,
    epoch_retrieval::EpochRetrievalRequest,
    proposal_msg::ProposalMsg,
    sync_info::SyncInfo,
    vote_msg::VoteMsg,
};
use futures::sink::SinkExt;
use libra_types::{crypto_proxies::ValidatorInfo, validator_change::ValidatorChangeProof, PeerId};
use network::{
    common::NetworkPublicKeys,
    connectivity_manager::ConnectivityRequest,
    error::NetworkError,
    peer_manager::{ConnectionRequestSender, PeerManagerRequestSender},
    protocols::{
        network::{NetworkEvents, NetworkSender},
        rpc::error::RpcError,
    },
    validator_network::network_builder::NetworkBuilder,
    ProtocolId,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Network type for consensus
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ConsensusMsg<T> {
    /// RPC to get a chain of block of the given length starting from the given block id.
    BlockRetrievalRequest(Box<BlockRetrievalRequest>),
    #[serde(bound = "T: Payload")]
    /// Carries the returned blocks and the retrieval status.
    BlockRetrievalResponse(Box<BlockRetrievalResponse<T>>),
    /// Request to get a ValidatorChangeProof from current_epoch to target_epoch
    EpochRetrievalRequest(Box<EpochRetrievalRequest>),
    #[serde(bound = "T: Payload")]
    /// ProposalMsg contains the required information for the proposer election protocol to make
    /// its choice (typically depends on round and proposer info).
    ProposalMsg(Box<ProposalMsg<T>>),
    /// This struct describes basic synchronization metadata.
    SyncInfo(Box<SyncInfo>),
    /// A vector of LedgerInfo with contiguous increasing epoch numbers to prove a sequence of
    /// validator changes from the first LedgerInfo's epoch.
    ValidatorChangeProof(Box<ValidatorChangeProof>),
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
pub type ConsensusNetworkEvents<T> = NetworkEvents<ConsensusMsg<T>>;

/// The interface from Consensus to Networking layer.
///
/// This is a thin wrapper around a `NetworkSender<ConsensusMsg>`, so it is easy
/// to clone and send off to a separate task. For example, the rpc requests
/// return Futures that encapsulate the whole flow, from sending the request to
/// remote, to finally receiving the response and deserializing. It therefore
/// makes the most sense to make the rpc call on a separate async task, which
/// requires the `ConsensusNetworkSender` to be `Clone` and `Send`.
#[derive(Clone)]
pub struct ConsensusNetworkSender<T> {
    network_sender: NetworkSender<ConsensusMsg<T>>,
    conn_mgr_reqs_tx: channel::Sender<ConnectivityRequest>,
}

/// Create a new Sender that only sends for the `CONSENSUS_DIRECT_SEND_PROTOCOL` and
/// `CONSENSUS_RPC_PROTOCOL` ProtocolId and a Receiver (Events) that explicitly returns only said
/// ProtocolId.
pub fn add_to_network<T: Payload>(
    network: &mut NetworkBuilder,
) -> (ConsensusNetworkSender<T>, ConsensusNetworkEvents<T>) {
    let (network_sender, network_receiver, connection_reqs_tx, connection_notifs_rx) = network
        .add_protocol_handler(
            vec![ProtocolId::ConsensusRpc],
            vec![ProtocolId::ConsensusDirectSend],
            QueueStyle::LIFO,
            Some(&counters::PENDING_CONSENSUS_NETWORK_EVENTS),
        );
    (
        ConsensusNetworkSender::new(
            network_sender,
            connection_reqs_tx,
            network
                .conn_mgr_reqs_tx()
                .expect("ConnecitivtyManager not enabled"),
        ),
        ConsensusNetworkEvents::new(network_receiver, connection_notifs_rx),
    )
}

impl<T: Payload> ConsensusNetworkSender<T> {
    /// Returns a Sender that only sends for the `CONSENSUS_DIRECT_SEND_PROTOCOL` and
    /// `CONSENSUS_RPC_PROTOCOL` ProtocolId.
    pub fn new(
        peer_mgr_reqs_tx: PeerManagerRequestSender,
        connection_reqs_tx: ConnectionRequestSender,
        conn_mgr_reqs_tx: channel::Sender<ConnectivityRequest>,
    ) -> Self {
        Self {
            network_sender: NetworkSender::new(peer_mgr_reqs_tx, connection_reqs_tx),
            conn_mgr_reqs_tx,
        }
    }

    /// Send a single message to the destination peer using the `CONSENSUS_DIRECT_SEND_PROTOCOL`
    /// ProtocolId.
    pub fn send_to(
        &mut self,
        recipient: PeerId,
        message: ConsensusMsg<T>,
    ) -> Result<(), NetworkError> {
        let protocol = ProtocolId::ConsensusDirectSend;
        self.network_sender.send_to(recipient, protocol, message)
    }

    /// Send a single message to the destination peers using the `CONSENSUS_DIRECT_SEND_PROTOCOL`
    /// ProtocolId.
    pub fn send_to_many(
        &mut self,
        recipients: impl Iterator<Item = PeerId>,
        message: ConsensusMsg<T>,
    ) -> Result<(), NetworkError> {
        let protocol = ProtocolId::ConsensusDirectSend;
        self.network_sender
            .send_to_many(recipients, protocol, message)
    }

    /// Send a RPC to the destination peer using the `CONSENSUS_RPC_PROTOCOL` ProtocolId.
    pub async fn send_rpc(
        &mut self,
        recipient: PeerId,
        message: ConsensusMsg<T>,
        timeout: Duration,
    ) -> Result<ConsensusMsg<T>, RpcError> {
        let protocol = ProtocolId::ConsensusRpc;
        self.network_sender
            .send_rpc(recipient, protocol, message, timeout)
            .await
    }

    /// Update set of nodes eligible to join the network. In the future, this should be handled by
    /// the unified reconfiguration event.
    pub async fn update_eligible_nodes(
        &mut self,
        validators: Vec<ValidatorInfo>,
    ) -> Result<(), NetworkError> {
        self.conn_mgr_reqs_tx
            .send(ConnectivityRequest::UpdateEligibleNodes(
                validators
                    .into_iter()
                    .map(|keys| {
                        (
                            *keys.account_address(),
                            NetworkPublicKeys {
                                identity_public_key: keys.network_identity_public_key().clone(),
                                signing_public_key: keys.network_signing_public_key().clone(),
                            },
                        )
                    })
                    .collect(),
            ))
            .await?;
        Ok(())
    }
}
