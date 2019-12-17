// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Interface between Mempool and Network layers.

use crate::{
    error::NetworkError,
    interface::NetworkRequest,
    proto::{
        BroadcastTransactionsRequest, BroadcastTransactionsResponse, MempoolSyncMsg,
        MempoolSyncMsg_oneof,
    },
    protocols::rpc::error::RpcError,
    validator_network::{NetworkEvents, NetworkSender},
    ProtocolId,
};
use channel;
use libra_types::PeerId;
use std::time::Duration;

/// Protocol id for mempool direct-send calls
pub const MEMPOOL_DIRECT_SEND_PROTOCOL: &[u8] = b"/libra/direct-send/0.1.0/mempool/0.1.0";
pub const MEMPOOL_RPC_PROTOCOL: &[u8] = b"/libra/rpc/0.1.0/mempool/0.1.0";

/// The interface from Network to Mempool layer.
///
/// `MempoolNetworkEvents` is a `Stream` of `NetworkNotification` where the
/// raw `Bytes` direct-send and rpc messages are deserialized into
/// `MempoolMessage` types. `MempoolNetworkEvents` is a thin wrapper around an
/// `channel::Receiver<NetworkNotification>`.
pub type MempoolNetworkEvents = NetworkEvents<MempoolSyncMsg>;

/// The interface from Mempool to Networking layer.
///
/// This is a thin wrapper around a `NetworkSender<MempoolSyncMsg>`, which is in
/// turn a thin wrapper around a `channel::Sender<NetworkRequest>`, so it is
/// easy to clone and send off to a separate task. For example, the rpc requests
/// return Futures that encapsulate the whole flow, from sending the request to
/// remote, to finally receiving the response and deserializing. It therefore
/// makes the most sense to make the rpc call on a separate async task, which
/// requires the `MempoolNetworkSender` to be `Clone` and `Send`.
#[derive(Clone)]
pub struct MempoolNetworkSender {
    inner: NetworkSender<MempoolSyncMsg>,
}

impl MempoolNetworkSender {
    pub fn new(inner: channel::Sender<NetworkRequest>) -> Self {
        Self {
            inner: NetworkSender::new(inner),
        }
    }

    /// for direct sends
    pub async fn send_to(
        &mut self,
        recipient: PeerId,
        message: MempoolSyncMsg,
    ) -> Result<(), NetworkError> {
        let protocol = ProtocolId::from_static(MEMPOOL_DIRECT_SEND_PROTOCOL);
        self.inner.send_to(recipient, protocol, message).await
    }

    /// Send a `BroadcastTransactionsRequest` RPC request to remote peer `recipient`. Returns the
    /// future `BroadcastTransactionsResponse` returned by the remote peer.
    ///
    /// The rpc request can be canceled at any point by dropping the returned
    /// future.
    pub async fn broadcast_transactions(
        &mut self,
        recipient: PeerId,
        req_msg: BroadcastTransactionsRequest,
        timeout: Duration,
    ) -> Result<BroadcastTransactionsResponse, RpcError> {
        let protocol = ProtocolId::from_static(MEMPOOL_RPC_PROTOCOL);
        let req_msg_enum = MempoolSyncMsg {
            message: Some(MempoolSyncMsg_oneof::BroadcastTransactionsRequest(req_msg)),
        };

        let res_msg_enum = self
            .inner
            .unary_rpc(recipient, protocol, req_msg_enum, timeout)
            .await?;
        if let Some(MempoolSyncMsg_oneof::BroadcastTransactionsResponse(response)) =
            res_msg_enum.message
        {
            Ok(response)
        } else {
            // TODO: context
            Err(RpcError::InvalidRpcResponse)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        interface::NetworkNotification,
        proto::{BroadcastTransactionsRequest, MempoolSyncMsg_oneof},
        protocols::{direct_send::Message, rpc::InboundRpcRequest},
        utils::MessageExt,
        validator_network::Event,
    };
    use futures::{
        channel::oneshot, executor::block_on, future::try_join, sink::SinkExt, stream::StreamExt,
    };
    use prost::Message as _;

    fn new_test_sync_req_msg(peer_id: PeerId) -> MempoolSyncMsg {
        let mut submit_txns_req = BroadcastTransactionsRequest::default();
        submit_txns_req.peer_id = peer_id.into();

        MempoolSyncMsg {
            message: Some(MempoolSyncMsg_oneof::BroadcastTransactionsRequest(
                submit_txns_req,
            )),
        }
    }

    // Direct send messages should get deserialized through the
    // `MempoolNetworkEvents` stream.
    #[test]
    fn test_mempool_network_events() {
        let (mut mempool_tx, mempool_rx) = channel::new_test(8);
        let mut stream = MempoolNetworkEvents::new(mempool_rx);

        let peer_id = PeerId::random();
        let mempool_msg = new_test_sync_req_msg(peer_id);
        let network_msg = Message {
            protocol: ProtocolId::from_static(MEMPOOL_DIRECT_SEND_PROTOCOL),
            mdata: mempool_msg.clone().to_bytes().unwrap(),
        };

        block_on(mempool_tx.send(NetworkNotification::RecvMessage(peer_id, network_msg))).unwrap();
        let event = block_on(stream.next()).unwrap().unwrap();
        assert_eq!(event, Event::Message((peer_id, mempool_msg)));

        block_on(mempool_tx.send(NetworkNotification::NewPeer(peer_id))).unwrap();
        let event = block_on(stream.next()).unwrap().unwrap();
        assert_eq!(event, Event::NewPeer(peer_id));
    }

    // `MempoolNetworkSender` should serialize outbound messages
    #[test]
    fn test_mempool_network_sender() {
        let (network_reqs_tx, mut network_reqs_rx) = channel::new_test(8);
        let mut sender = MempoolNetworkSender::new(network_reqs_tx);

        let peer_id = PeerId::random();
        let mempool_msg = new_test_sync_req_msg(peer_id);
        let expected_network_msg = Message {
            protocol: ProtocolId::from_static(MEMPOOL_DIRECT_SEND_PROTOCOL),
            mdata: mempool_msg.clone().to_bytes().unwrap(),
        };

        // Send the message to network layer
        block_on(sender.send_to(peer_id, mempool_msg)).unwrap();

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

    #[test]
    fn test_shared_mempool_inbound_rpc() {
        let (mut smp_tx, smp_rx) = channel::new_test(8);
        let mut stream = MempoolNetworkEvents::new(smp_rx);

        // build rpc request
        let req_msg = BroadcastTransactionsRequest::default();
        let req_msg_enum = MempoolSyncMsg {
            message: Some(MempoolSyncMsg_oneof::BroadcastTransactionsRequest(req_msg)),
        };
        let req_data = req_msg_enum.clone().to_bytes().unwrap();
        let (res_tx, _) = oneshot::channel();
        let rpc_req = InboundRpcRequest {
            protocol: ProtocolId::from_static(MEMPOOL_RPC_PROTOCOL),
            data: req_data,
            res_tx,
        };

        // mock receiving rpc request
        let peer_id = PeerId::random();
        let event = NetworkNotification::RecvRpc(peer_id, rpc_req);
        block_on(smp_tx.send(event)).unwrap();

        // request should be properly deserialized
        let (res_tx, _) = oneshot::channel();
        let expected_event = Event::RpcRequest((peer_id, req_msg_enum.clone(), res_tx));
        let event = block_on(stream.next()).unwrap().unwrap();
        assert_eq!(event, expected_event);
    }

    #[test]
    fn test_shared_mempool_outbound_rpc() {
        let (network_reqs_tx, mut network_reqs_rx) = channel::new_test(8);
        let mut sender = MempoolNetworkSender::new(network_reqs_tx);

        // make submit_transaction_request rpc request
        let peer_id = PeerId::random();
        let req_msg = BroadcastTransactionsRequest::default();
        let f_res_msg =
            sender.broadcast_transactions(peer_id, req_msg.clone(), Duration::from_secs(5));

        // build rpc response
        let res_msg = BroadcastTransactionsResponse::default();
        let res_msg_enum = MempoolSyncMsg {
            message: Some(MempoolSyncMsg_oneof::BroadcastTransactionsResponse(
                res_msg.clone(),
            )),
        };
        let res_data = res_msg_enum.to_bytes().unwrap();

        // the future response
        let f_recv = async move {
            match network_reqs_rx.next().await.unwrap() {
                NetworkRequest::SendRpc(recv_peer_id, req) => {
                    assert_eq!(recv_peer_id, peer_id);
                    assert_eq!(req.protocol.as_ref(), MEMPOOL_RPC_PROTOCOL);

                    // check request deserializes
                    let mut req_msg_enum = MempoolSyncMsg::decode(req.data.as_ref()).unwrap();
                    let recv_req_msg = req_msg_enum.message.take();
                    assert_eq!(
                        recv_req_msg,
                        Some(MempoolSyncMsg_oneof::BroadcastTransactionsRequest(req_msg))
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
