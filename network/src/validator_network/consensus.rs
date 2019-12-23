// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Interface between Consensus and Network layers.

use crate::{
    common::NetworkPublicKeys,
    connectivity_manager::ConnectivityRequest,
    counters,
    error::NetworkError,
    peer_manager::{PeerManagerRequest, PeerManagerRequestSender},
    proto::{ConsensusMsg, ConsensusMsg_oneof, RequestBlock, RespondBlock},
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
    let (network_sender, network_receiver) = network.add_protocol_handler(
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
        ConsensusNetworkEvents::new(network_receiver),
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

    pub async fn request_block(
        &mut self,
        recipient: PeerId,
        req_msg: RequestBlock,
        timeout: Duration,
    ) -> Result<RespondBlock, RpcError> {
        let protocol = ProtocolId::from_static(CONSENSUS_RPC_PROTOCOL);
        let req_msg_enum = ConsensusMsg {
            message: Some(ConsensusMsg_oneof::RequestBlock(req_msg)),
        };

        let res_msg_enum = self
            .peer_mgr_reqs_tx
            .unary_rpc(recipient, protocol, req_msg_enum, timeout)
            .await?;

        if let Some(ConsensusMsg_oneof::RespondBlock(response)) = res_msg_enum.message {
            Ok(response)
        } else {
            // TODO: context
            Err(RpcError::InvalidRpcResponse)
        }
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
        peer_manager::{PeerManagerNotification, PeerManagerRequest},
        proto::VoteMsg,
        protocols::{direct_send::Message, rpc::InboundRpcRequest},
        utils::MessageExt,
        validator_network::Event,
    };
    use channel::{libra_channel, message_queues::QueueStyle};
    use futures::{channel::oneshot, executor::block_on, future::try_join, stream::StreamExt};
    use parity_multiaddr::Multiaddr;
    use prost::Message as _;
    use std::num::NonZeroUsize;

    fn new_test_vote() -> ConsensusMsg {
        let mut vote_msg = VoteMsg::default();
        vote_msg.bytes = vec![];

        ConsensusMsg {
            message: Some(ConsensusMsg_oneof::VoteMsg(vote_msg)),
        }
    }

    // Direct send messages should get deserialized through the
    // `ConsensusNetworkEvents` stream.
    #[test]
    fn test_consensus_network_events() {
        let (mut consensus_tx, consensus_rx) =
            libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(8).unwrap(), None);
        let mut stream = ConsensusNetworkEvents::new(consensus_rx);

        let peer_id = PeerId::random();
        let consensus_msg = new_test_vote();
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

        // Network notifies consensus about new peer
        consensus_tx
            .push(
                (peer_id, ProtocolId::default()),
                PeerManagerNotification::NewPeer(peer_id, Multiaddr::empty()),
            )
            .unwrap();

        // Consensus should receive notification
        let event = block_on(stream.next()).unwrap().unwrap();
        assert_eq!(event, Event::NewPeer(peer_id));
    }

    // `ConsensusNetworkSender` should serialize outbound messages
    #[test]
    fn test_consensus_network_sender() {
        let (network_reqs_tx, mut network_reqs_rx) =
            libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(8).unwrap(), None);
        let (conn_mgr_reqs_tx, _conn_mgr_reqs_rx) = channel::new_test(8);
        let mut sender = ConsensusNetworkSender::new(network_reqs_tx, conn_mgr_reqs_tx);

        let peer_id = PeerId::random();
        let consensus_msg = new_test_vote();
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
        let (mut consensus_tx, consensus_rx) =
            libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(8).unwrap(), None);
        let mut stream = ConsensusNetworkEvents::new(consensus_rx);

        // build rpc request
        let req_msg = RequestBlock::default();
        let req_msg_enum = ConsensusMsg {
            message: Some(ConsensusMsg_oneof::RequestBlock(req_msg)),
        };
        let req_data = req_msg_enum.clone().to_bytes().unwrap();

        let (res_tx, _) = oneshot::channel();
        let rpc_req = InboundRpcRequest {
            protocol: ProtocolId::from_static(CONSENSUS_RPC_PROTOCOL),
            data: req_data,
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
        let expected_event = Event::RpcRequest((peer_id, req_msg_enum, res_tx));
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
        let req_msg = RequestBlock::default();
        let f_res_msg = sender.request_block(peer_id, req_msg.clone(), Duration::from_secs(5));

        // build rpc response
        let res_msg = RespondBlock::default();
        let res_msg_enum = ConsensusMsg {
            message: Some(ConsensusMsg_oneof::RespondBlock(res_msg.clone())),
        };
        let res_data = res_msg_enum.to_bytes().unwrap();

        // the future response
        let f_recv = async move {
            match network_reqs_rx.next().await.unwrap() {
                PeerManagerRequest::SendRpc(recv_peer_id, req) => {
                    assert_eq!(recv_peer_id, peer_id);
                    assert_eq!(req.protocol.as_ref(), CONSENSUS_RPC_PROTOCOL);

                    // check request deserializes
                    let req_msg_enum = ConsensusMsg::decode(req.data.as_ref()).unwrap();
                    assert_eq!(
                        req_msg_enum.message,
                        Some(ConsensusMsg_oneof::RequestBlock(req_msg))
                    );

                    // remote replies with some response message
                    req.res_tx.send(Ok(res_data)).unwrap();
                    Ok(())
                }
                event => panic!("Unexpected event: {:?}", event),
            }
        };

        let (recv_res_msg, _) = block_on(try_join(f_res_msg, f_recv)).unwrap();
        assert_eq!(recv_res_msg, res_msg);
    }
}
