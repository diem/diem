// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Interface between Consensus and Network layers.

use crate::{
    error::NetworkError,
    interface::{NetworkNotification, NetworkRequest},
    proto::{ConsensusMsg, RequestBlock, RequestChunk, RespondBlock, RespondChunk},
    protocols::{
        direct_send::Message,
        rpc::{self, error::RpcError},
    },
    validator_network::Event,
    NetworkPublicKeys, ProtocolId,
};
use bytes::Bytes;
use channel;
use futures::{
    stream::Map,
    task::{Context, Poll},
    SinkExt, Stream, StreamExt,
};
use pin_utils::unsafe_pinned;
use protobuf::Message as proto_msg;
use std::{pin::Pin, time::Duration};
use types::{validator_public_keys::ValidatorPublicKeys, PeerId};

/// Protocol id for consensus RPC calls
pub const CONSENSUS_RPC_PROTOCOL: &[u8] = b"/libra/consensus/rpc/0.1.0";
/// Protocol id for consensus direct-send calls
pub const CONSENSUS_DIRECT_SEND_PROTOCOL: &[u8] = b"/libra/consensus/direct-send/0.1.0";

/// The interface from Network to Consensus layer.
///
/// `ConsensusNetworkEvents` is a `Stream` of `NetworkNotification` where the
/// raw `Bytes` direct-send and rpc messages are deserialized into
/// `ConsensusMessage` types. `ConsensusNetworkEvents` is a thin wrapper around
/// an `channel::Receiver<NetworkNotification>`.
pub struct ConsensusNetworkEvents {
    inner: Map<
        channel::Receiver<NetworkNotification>,
        fn(NetworkNotification) -> Result<Event<ConsensusMsg>, NetworkError>,
    >,
}

impl ConsensusNetworkEvents {
    // This use of `unsafe_pinned` is safe because:
    //   1. This struct does not implement [`Drop`]
    //   2. This struct does not implement [`Unpin`]
    //   3. This struct is not `#[repr(packed)]`
    unsafe_pinned!(
        inner:
            Map<
                channel::Receiver<NetworkNotification>,
                fn(NetworkNotification) -> Result<Event<ConsensusMsg>, NetworkError>,
            >
    );

    pub fn new(receiver: channel::Receiver<NetworkNotification>) -> Self {
        let inner = receiver.map::<_, fn(_) -> _>(|notification| match notification {
            NetworkNotification::NewPeer(peer_id) => Ok(Event::NewPeer(peer_id)),
            NetworkNotification::LostPeer(peer_id) => Ok(Event::LostPeer(peer_id)),
            NetworkNotification::RecvRpc(peer_id, rpc_req) => {
                let req_msg = ::protobuf::parse_from_bytes(rpc_req.data.as_ref())?;
                Ok(Event::RpcRequest((peer_id, req_msg, rpc_req.res_tx)))
            }
            NetworkNotification::RecvMessage(peer_id, msg) => {
                let msg = ::protobuf::parse_from_bytes(msg.mdata.as_ref())?;
                Ok(Event::Message((peer_id, msg)))
            }
        });

        Self { inner }
    }
}

impl Stream for ConsensusNetworkEvents {
    type Item = Result<Event<ConsensusMsg>, NetworkError>;

    fn poll_next(self: Pin<&mut Self>, context: &mut Context) -> Poll<Option<Self::Item>> {
        self.inner().poll_next(context)
    }
}

/// The interface from Consensus to Networking layer.
///
/// This is a thin wrapper around an `channel::Sender<NetworkRequest>`, so it is
/// easy to clone and send off to a separate task. For example, the rpc requests
/// return Futures that encapsulate the whole flow, from sending the request to
/// remote, to finally receiving the response and deserializing. It therefore
/// makes the most sense to make the rpc call on a separate async task, which
/// requires the `ConsensusNetworkSender` to be `Clone` and `Send`.
#[derive(Clone)]
pub struct ConsensusNetworkSender {
    inner: channel::Sender<NetworkRequest>,
}

impl ConsensusNetworkSender {
    pub fn new(inner: channel::Sender<NetworkRequest>) -> Self {
        Self { inner }
    }

    /// Send a fire-and-forget "direct-send" message to remote peer `recipient`.
    ///
    /// Currently, the returned Future simply resolves when the message has been
    /// enqueued on the network actor's event queue. It therefore makes no
    /// reliable delivery guarantees.
    pub async fn send_to(
        &mut self,
        recipient: PeerId,
        message: ConsensusMsg,
    ) -> Result<(), NetworkError> {
        self.inner
            .send(NetworkRequest::SendMessage(
                recipient,
                Message {
                    protocol: ProtocolId::from_static(CONSENSUS_DIRECT_SEND_PROTOCOL),
                    mdata: Bytes::from(message.write_to_bytes().unwrap()),
                },
            ))
            .await?;
        Ok(())
    }

    /// Send a RequestBlock RPC request to remote peer `recipient`. Returns the
    /// future `RespondBlock` returned by the remote peer.
    ///
    /// The rpc request can be canceled at any point by dropping the returned
    /// future.
    pub async fn request_block(
        &mut self,
        recipient: PeerId,
        req_msg: RequestBlock,
        timeout: Duration,
    ) -> Result<RespondBlock, RpcError> {
        let protocol = ProtocolId::from_static(CONSENSUS_RPC_PROTOCOL);
        let mut req_msg_enum = ConsensusMsg::new();
        req_msg_enum.set_request_block(req_msg);
        let mut res_msg_enum = rpc::utils::unary_rpc(
            self.inner.clone(),
            recipient,
            protocol,
            req_msg_enum,
            timeout,
        )
        .await?;

        if res_msg_enum.has_respond_block() {
            Ok(res_msg_enum.take_respond_block())
        } else {
            // TODO: context
            Err(RpcError::InvalidRpcResponse)
        }
    }

    /// Send a RequestChunk RPC request to remote peer `recipient`. Returns the
    /// future `RespondChunk` returned by the remote peer.
    ///
    /// The rpc request can be canceled at any point by dropping the returned
    /// future.
    pub async fn request_chunk(
        &mut self,
        recipient: PeerId,
        req_msg: RequestChunk,
        timeout: Duration,
    ) -> Result<RespondChunk, RpcError> {
        let protocol = ProtocolId::from_static(CONSENSUS_RPC_PROTOCOL);
        let mut req_msg_enum = ConsensusMsg::new();
        req_msg_enum.set_request_chunk(req_msg);

        let mut res_msg_enum = rpc::utils::unary_rpc(
            self.inner.clone(),
            recipient,
            protocol,
            req_msg_enum,
            timeout,
        )
        .await?;

        if res_msg_enum.has_respond_chunk() {
            Ok(res_msg_enum.take_respond_chunk())
        } else {
            // TODO: context
            Err(RpcError::InvalidRpcResponse)
        }
    }

    pub async fn update_eligible_nodes(
        &mut self,
        validators: Vec<ValidatorPublicKeys>,
    ) -> Result<(), NetworkError> {
        self.inner
            .send(NetworkRequest::UpdateEligibleNodes(
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
    use crate::{proto::Vote, protocols::rpc::InboundRpcRequest};
    use futures::{channel::oneshot, executor::block_on, future::try_join};

    fn new_test_vote() -> ConsensusMsg {
        let mut vote = Vote::new();
        vote.set_proposed_block_id(Bytes::new());
        vote.set_executed_state_id(Bytes::new());
        vote.set_author(Bytes::new());
        vote.set_signature(Bytes::new());

        let mut consensus_msg = ConsensusMsg::new();
        consensus_msg.set_vote(vote);
        consensus_msg
    }

    // Direct send messages should get deserialized through the
    // `ConsensusNetworkEvents` stream.
    #[test]
    fn test_consensus_network_events() {
        let (mut consensus_tx, consensus_rx) = channel::new_test(8);
        let mut stream = ConsensusNetworkEvents::new(consensus_rx);

        let peer_id = PeerId::random();
        let consensus_msg = new_test_vote();
        let network_msg = Message {
            protocol: ProtocolId::from_static(CONSENSUS_DIRECT_SEND_PROTOCOL),
            mdata: consensus_msg.clone().write_to_bytes().unwrap().into(),
        };

        // Network sends inbound message to consensus
        block_on(consensus_tx.send(NetworkNotification::RecvMessage(peer_id, network_msg)))
            .unwrap();

        // Consensus should receive deserialized message event
        let event = block_on(stream.next()).unwrap().unwrap();
        assert_eq!(event, Event::Message((peer_id.into(), consensus_msg)));

        // Network notifies consensus about new peer
        block_on(consensus_tx.send(NetworkNotification::NewPeer(peer_id))).unwrap();

        // Consensus should receive notification
        let event = block_on(stream.next()).unwrap().unwrap();
        assert_eq!(event, Event::NewPeer(peer_id.into()));
    }

    // `ConsensusNetworkSender` should serialize outbound messages
    #[test]
    fn test_consensus_network_sender() {
        let (network_reqs_tx, mut network_reqs_rx) = channel::new_test(8);
        let mut sender = ConsensusNetworkSender::new(network_reqs_tx);

        let peer_id = PeerId::random();
        let consensus_msg = new_test_vote();
        let expected_network_msg = Message {
            protocol: ProtocolId::from_static(CONSENSUS_DIRECT_SEND_PROTOCOL),
            mdata: consensus_msg.clone().write_to_bytes().unwrap().into(),
        };

        // Send the message to network layer
        block_on(sender.send_to(peer_id.into(), consensus_msg)).unwrap();

        // Network layer should receive serialized message to send out
        let event = block_on(network_reqs_rx.next()).unwrap();
        match event {
            NetworkRequest::SendMessage(recv_peer_id, network_msg) => {
                assert_eq!(recv_peer_id, peer_id);
                assert_eq!(network_msg, expected_network_msg);
            }
            event => panic!("Unexpected event: {:?}", event),
        }
    }

    // `ConsensusNetworkEvents` should deserialize inbound RPC requests
    #[test]
    fn test_consensus_inbound_rpc() {
        let (mut consensus_tx, consensus_rx) = channel::new_test(8);
        let mut stream = ConsensusNetworkEvents::new(consensus_rx);

        // build rpc request
        let req_msg = RequestBlock::new();
        let mut req_msg_enum = ConsensusMsg::new();
        req_msg_enum.set_request_block(req_msg);
        let req_data = req_msg_enum.clone().write_to_bytes().unwrap().into();

        let (res_tx, _) = oneshot::channel();
        let rpc_req = InboundRpcRequest {
            protocol: ProtocolId::from_static(CONSENSUS_RPC_PROTOCOL),
            data: req_data,
            res_tx,
        };

        // mock receiving rpc request
        let peer_id = PeerId::random();
        let event = NetworkNotification::RecvRpc(peer_id, rpc_req);
        block_on(consensus_tx.send(event)).unwrap();

        // request should be properly deserialized
        let (res_tx, _) = oneshot::channel();
        let expected_event = Event::RpcRequest((peer_id.into(), req_msg_enum.clone(), res_tx));
        let event = block_on(stream.next()).unwrap().unwrap();
        assert_eq!(event, expected_event);
    }

    // When consensus sends an rpc request, network should get a `NetworkRequest::SendRpc`
    // with the serialized request.
    #[test]
    fn test_consensus_outbound_rpc() {
        let (network_reqs_tx, mut network_reqs_rx) = channel::new_test(8);
        let mut sender = ConsensusNetworkSender::new(network_reqs_tx);

        // send get_block rpc request
        let peer_id = PeerId::random();
        let req_msg = RequestBlock::new();
        let f_res_msg =
            sender.request_block(peer_id.into(), req_msg.clone(), Duration::from_secs(5));

        // build rpc response
        let res_msg = RespondBlock::new();
        let mut res_msg_enum = ConsensusMsg::new();
        res_msg_enum.set_respond_block(res_msg.clone());
        let res_data = res_msg_enum.write_to_bytes().unwrap().into();

        // the future response
        let f_recv = async move {
            match network_reqs_rx.next().await.unwrap() {
                NetworkRequest::SendRpc(recv_peer_id, req) => {
                    assert_eq!(recv_peer_id, peer_id);
                    assert_eq!(req.protocol.as_ref(), CONSENSUS_RPC_PROTOCOL);

                    // check request deserializes
                    let mut req_msg_enum: ConsensusMsg =
                        ::protobuf::parse_from_bytes(req.data.as_ref()).unwrap();
                    let recv_req_msg = req_msg_enum.take_request_block();
                    assert_eq!(recv_req_msg, req_msg);

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
