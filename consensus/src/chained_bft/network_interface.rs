// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Interface between Consensus and Network layers.

use crate::counters;
use channel::{libra_channel, message_queues::QueueStyle};
use futures::sink::SinkExt;
use libra_types::{crypto_proxies::ValidatorPublicKeys, PeerId};
use network::{
    common::NetworkPublicKeys,
    connectivity_manager::ConnectivityRequest,
    error::NetworkError,
    peer_manager::{PeerManagerRequest, PeerManagerRequestSender},
    proto::ConsensusMsg,
    protocols::rpc::error::RpcError,
    validator_network::{network_builder::NetworkBuilder, NetworkEvents, NetworkSender},
    ProtocolId,
};
use std::time::Duration;

/// Protocol id for consensus direct-send calls
pub const CONSENSUS_DIRECT_SEND_PROTOCOL: &[u8] = b"/libra/direct-send/0.1.0/consensus/0.1.0";
/// Protocol id for consensus RPC calls
pub const CONSENSUS_RPC_PROTOCOL: &[u8] = b"/libra/rpc/0.1.0/consensus/0.1.0";

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
    peer_mgr_reqs_tx: NetworkSender<ConsensusMsg>,
    conn_mgr_reqs_tx: channel::Sender<ConnectivityRequest>,
}

/// Create a new Sender that only sends for the `CONSENSUS_DIRECT_SEND_PROTOCOL` and
/// `CONSENSUS_RPC_PROTOCOL` ProtocolId and a Receiver (Events) that explicitly returns only said
/// ProtocolId.
pub fn add_to_network(
    network: &mut NetworkBuilder,
) -> (ConsensusNetworkSender, ConsensusNetworkEvents) {
    let (network_sender, network_receiver, control_notifs_rx) = network.add_protocol_handler(
        vec![ProtocolId::from_static(CONSENSUS_RPC_PROTOCOL)],
        vec![ProtocolId::from_static(CONSENSUS_DIRECT_SEND_PROTOCOL)],
        QueueStyle::LIFO,
        Some(&counters::PENDING_CONSENSUS_NETWORK_EVENTS),
    );
    (
        ConsensusNetworkSender::new(
            network_sender,
            network
                .conn_mgr_reqs_tx()
                .expect("ConnecitivtyManager not enabled"),
        ),
        ConsensusNetworkEvents::new(network_receiver, control_notifs_rx),
    )
}

impl ConsensusNetworkSender {
    /// Returns a Sender that only sends for the `CONSENSUS_DIRECT_SEND_PROTOCOL` and
    /// `CONSENSUS_RPC_PROTOCOL` ProtocolId.
    pub fn new(
        peer_mgr_reqs_tx: libra_channel::Sender<(PeerId, ProtocolId), PeerManagerRequest>,
        conn_mgr_reqs_tx: channel::Sender<ConnectivityRequest>,
    ) -> Self {
        Self {
            peer_mgr_reqs_tx: NetworkSender::new(PeerManagerRequestSender::new(peer_mgr_reqs_tx)),
            conn_mgr_reqs_tx,
        }
    }

    /// Send a single message to the destination peer using the `CONSENSUS_DIRECT_SEND_PROTOCOL`
    /// ProtocolId.
    pub fn send_to(
        &mut self,
        recipient: PeerId,
        message: ConsensusMsg,
    ) -> Result<(), NetworkError> {
        let protocol = ProtocolId::from_static(CONSENSUS_DIRECT_SEND_PROTOCOL);
        self.peer_mgr_reqs_tx.send_to(recipient, protocol, message)
    }

    /// Send a single message to the destination peers using the `CONSENSUS_DIRECT_SEND_PROTOCOL`
    /// ProtocolId.
    pub fn send_to_many(
        &mut self,
        recipients: impl Iterator<Item = PeerId>,
        message: ConsensusMsg,
    ) -> Result<(), NetworkError> {
        let protocol = ProtocolId::from_static(CONSENSUS_DIRECT_SEND_PROTOCOL);
        self.peer_mgr_reqs_tx
            .send_to_many(recipients, protocol, message)
    }

    /// Send a RPC to the destination peer using the `CONSENSUS_RPC_PROTOCOL` ProtocolId.
    pub async fn send_rpc(
        &mut self,
        recipient: PeerId,
        message: ConsensusMsg,
        timeout: Duration,
    ) -> Result<ConsensusMsg, RpcError> {
        let protocol = ProtocolId::from_static(CONSENSUS_RPC_PROTOCOL);
        self.peer_mgr_reqs_tx
            .unary_rpc(recipient, protocol, message, timeout)
            .await
    }

    /// Update set of nodes eligible to join the network. In the future, this should be handled by
    /// the unified reconfiguration event.
    pub async fn update_eligible_nodes(
        &mut self,
        validators: Vec<ValidatorPublicKeys>,
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
