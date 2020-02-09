// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Interface between Consensus and Network layers.

use crate::{
    common::NetworkPublicKeys,
    connectivity_manager::ConnectivityRequest,
    counters,
    error::NetworkError,
    peer_manager::{PeerManagerRequest, PeerManagerRequestSender},
    proto::ConsensusMsg,
    protocols::rpc::error::RpcError,
    validator_network::network_builder::NetworkBuilder,
    validator_network::{NetworkEvents, NetworkSender},
    ProtocolId,
};
use channel::{libra_channel, message_queues::QueueStyle};
use futures::sink::SinkExt;
use libra_types::{crypto_proxies::ValidatorPublicKeys, PeerId};
use std::time::Duration;

/// Protocol id for consensus RPC calls
pub const CONSENSUS_RPC_PROTOCOL: &[u8] = b"/libra/rpc/0.1.0/consensus/0.1.0";
/// Protocol id for consensus direct-send calls
pub const CONSENSUS_DIRECT_SEND_PROTOCOL: &[u8] = b"/libra/direct-send/0.1.0/consensus/0.1.0";

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
    pub fn new(
        peer_mgr_reqs_tx: libra_channel::Sender<(PeerId, ProtocolId), PeerManagerRequest>,
        conn_mgr_reqs_tx: channel::Sender<ConnectivityRequest>,
    ) -> Self {
        Self {
            peer_mgr_reqs_tx: NetworkSender::new(PeerManagerRequestSender::new(peer_mgr_reqs_tx)),
            conn_mgr_reqs_tx,
        }
    }

    pub fn send_to(
        &mut self,
        recipient: PeerId,
        message: ConsensusMsg,
    ) -> Result<(), NetworkError> {
        let protocol = ProtocolId::from_static(CONSENSUS_DIRECT_SEND_PROTOCOL);
        self.peer_mgr_reqs_tx.send_to(recipient, protocol, message)
    }

    pub fn send_to_many(
        &mut self,
        recipients: impl Iterator<Item = PeerId>,
        message: ConsensusMsg,
    ) -> Result<(), NetworkError> {
        let protocol = ProtocolId::from_static(CONSENSUS_DIRECT_SEND_PROTOCOL);
        self.peer_mgr_reqs_tx
            .send_to_many(recipients, protocol, message)
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        peer_manager::{self, conn_status_channel, PeerManagerNotification, PeerManagerRequest},
        protocols::{direct_send::Message, rpc::InboundRpcRequest},
        utils::MessageExt,
        validator_network::Event,
    };
    use channel::{libra_channel, message_queues::QueueStyle};
    use futures::{channel::oneshot, executor::block_on, future::try_join, stream::StreamExt};
    use parity_multiaddr::Multiaddr;
    use prost::Message as _;
    use std::num::NonZeroUsize;

    // Direct send messages should get deserialized through the
    // `ConsensusNetworkEvents` stream.
    #[test]
    fn test_consensus_network_events() {
        let (mut control_notifs_tx, control_notifs_rx) = conn_status_channel::new();
        let (mut consensus_tx, consensus_rx) =
            libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(8).unwrap(), None);
        let mut stream = ConsensusNetworkEvents::new(consensus_rx, control_notifs_rx);

        let consensus_msg = ConsensusMsg::default();
        let peer_id = PeerId::random();
        let network_msg = Message {
            protocol: ProtocolId::from_static(CONSENSUS_DIRECT_SEND_PROTOCOL),
            mdata: consensus_msg.clone().to_bytes().unwrap(),
        };

        // Network sends inbound message to consensus
        consensus_tx
            .push(
                (
                    peer_id,
                    ProtocolId::from_static(CONSENSUS_DIRECT_SEND_PROTOCOL),
                ),
                PeerManagerNotification::RecvMessage(peer_id, network_msg),
            )
            .unwrap();

        // Consensus should receive deserialized message event
        let event = block_on(stream.next()).unwrap().unwrap();
        assert_eq!(event, Event::Message((peer_id, consensus_msg)));

        let f = async {
            // Network notifies consensus about new peer.
            control_notifs_tx
                .push(
                    peer_id,
                    peer_manager::ConnectionStatusNotification::NewPeer(
                        peer_id,
                        Multiaddr::empty(),
                    ),
                )
                .unwrap();
            // Consensus should receive notification.
            let event = stream.next().await.unwrap().unwrap();
            assert_eq!(event, Event::NewPeer(peer_id));
        };
        block_on(f);
    }

    // `ConsensusNetworkSender` should serialize outbound messages
    #[test]
    fn test_consensus_network_sender() {
        let (network_reqs_tx, mut network_reqs_rx) =
            libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(8).unwrap(), None);
        let (conn_mgr_reqs_tx, _conn_mgr_reqs_rx) = channel::new_test(8);
        let mut sender = ConsensusNetworkSender::new(network_reqs_tx, conn_mgr_reqs_tx);

        let consensus_msg = ConsensusMsg::default();
        let peer_id = PeerId::random();
        let expected_network_msg = Message {
            protocol: ProtocolId::from_static(CONSENSUS_DIRECT_SEND_PROTOCOL),
            mdata: consensus_msg.clone().to_bytes().unwrap(),
        };

        // Send the message to network layer
        sender.send_to(peer_id, consensus_msg).unwrap();

        // Network layer should receive serialized message to send out
        let event = block_on(network_reqs_rx.next()).unwrap();
        match event {
            PeerManagerRequest::SendMessage(recv_peer_id, network_msg) => {
                assert_eq!(recv_peer_id, peer_id);
                assert_eq!(network_msg, expected_network_msg);
            }
            event => panic!("Unexpected event: {:?}", event),
        }
    }

    // `ConsensusNetworkEvents` should deserialize inbound RPC requests
    #[test]
    fn test_consensus_inbound_rpc() {
        let (_, control_notifs_rx) = conn_status_channel::new();
        let (mut consensus_tx, consensus_rx) =
            libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(8).unwrap(), None);
        let mut stream = ConsensusNetworkEvents::new(consensus_rx, control_notifs_rx);

        // build rpc request
        let (res_tx, _) = oneshot::channel();
        let rpc_req = InboundRpcRequest {
            protocol: ProtocolId::from_static(CONSENSUS_RPC_PROTOCOL),
            data: ConsensusMsg::default().to_bytes().unwrap(),
            res_tx,
        };

        // mock receiving rpc request
        let peer_id = PeerId::random();
        let event = PeerManagerNotification::RecvRpc(peer_id, rpc_req);
        consensus_tx
            .push(
                (peer_id, ProtocolId::from_static(CONSENSUS_RPC_PROTOCOL)),
                event,
            )
            .unwrap();

        // request should be properly deserialized
        let (res_tx, _) = oneshot::channel();
        let expected_event = Event::RpcRequest((peer_id, ConsensusMsg::default(), res_tx));
        let event = block_on(stream.next()).unwrap().unwrap();
        assert_eq!(event, expected_event);
    }

    // When consensus sends an rpc request, network should get a `PeerManagerRequest::SendRpc`
    // with the serialized request.
    #[test]
    fn test_consensus_outbound_rpc() {
        let (network_reqs_tx, mut network_reqs_rx) =
            libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(8).unwrap(), None);
        let (conn_mgr_reqs_tx, _conn_mgr_reqs_rx) = channel::new_test(8);
        let mut sender = ConsensusNetworkSender::new(network_reqs_tx, conn_mgr_reqs_tx);

        // send get_block rpc request
        let peer_id = PeerId::random();
        let req_msg = ConsensusMsg::default();
        let f_res_msg = sender.send_rpc(peer_id, req_msg, Duration::from_secs(5));

        // build rpc response
        let res_msg = ConsensusMsg::default().to_bytes().unwrap();

        // the future response
        let f_recv = async move {
            match network_reqs_rx.next().await.unwrap() {
                PeerManagerRequest::SendRpc(recv_peer_id, req) => {
                    assert_eq!(recv_peer_id, peer_id);
                    assert_eq!(req.protocol.as_ref(), CONSENSUS_RPC_PROTOCOL);

                    // check request deserializes
                    let req_msg_enum = ConsensusMsg::decode(req.data.as_ref()).unwrap();
                    assert_eq!(req_msg_enum, ConsensusMsg::default());

                    // remote replies with some response message
                    req.res_tx.send(Ok(res_msg)).unwrap();
                    Ok(())
                }
                event => panic!("Unexpected event: {:?}", event),
            }
        };

        let (recv_res_msg, _) = block_on(try_join(f_res_msg, f_recv)).unwrap();
        assert_eq!(recv_res_msg, ConsensusMsg::default());
    }
}
