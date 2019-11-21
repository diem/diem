// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Interface between HealthChecker and Network layers.

use crate::{
    error::NetworkError,
    interface::NetworkRequest,
    proto::{HealthCheckerMsg, HealthCheckerMsg_oneof, Ping, Pong},
    protocols::rpc::error::RpcError,
    validator_network::{NetworkEvents, NetworkSender},
    ProtocolId,
};
use channel;
use libra_types::PeerId;
use std::time::Duration;

/// Protocol id for HealthChecker RPC calls
pub const HEALTH_CHECKER_RPC_PROTOCOL: &[u8] = b"/libra/rpc/0.1.0/health-checker/0.1.0";

/// The interface from Network to HealthChecker layer.
///
/// `HealthCheckerNetworkEvents` is a `Stream` of `NetworkNotification` where the
/// raw `Bytes` rpc messages are deserialized into
/// `HealthCheckerMsg` types. `HealthCheckerNetworkEvents` is a thin wrapper
/// around an `channel::Receiver<NetworkNotification>`.
pub type HealthCheckerNetworkEvents = NetworkEvents<HealthCheckerMsg>;

/// The interface from HealthChecker to Networking layer.
///
/// This is a thin wrapper around a `NetworkSender<HealthCheckerMsg>`, which is
/// in turn a thin wrapper around a `channel::Sender<NetworkRequest>`, so it is
/// easy to clone and send off to a separate task. For example, the rpc requests
/// return Futures that encapsulate the whole flow, from sending the request to
/// remote, to finally receiving the response and deserializing. It therefore
/// makes the most sense to make the rpc call on a separate async task, which
/// requires the `HealthCheckerNetworkSender` to be `Clone` and `Send`.
#[derive(Clone)]
pub struct HealthCheckerNetworkSender {
    inner: NetworkSender<HealthCheckerMsg>,
}

impl HealthCheckerNetworkSender {
    pub fn new(inner: channel::Sender<NetworkRequest>) -> Self {
        Self {
            inner: NetworkSender::new(inner),
        }
    }

    /// Send a HealthChecker Ping RPC request to remote peer `recipient`. Returns
    /// the remote peer's future `Pong` reply.
    ///
    /// The rpc request can be canceled at any point by dropping the returned
    /// future.
    pub async fn ping(
        &mut self,
        recipient: PeerId,
        req_msg: Ping,
        timeout: Duration,
    ) -> Result<Pong, RpcError> {
        let protocol = ProtocolId::from_static(HEALTH_CHECKER_RPC_PROTOCOL);
        let req_msg_enum = HealthCheckerMsg {
            message: Some(HealthCheckerMsg_oneof::Ping(req_msg)),
        };

        let res_msg_enum = self
            .inner
            .unary_rpc(recipient, protocol, req_msg_enum, timeout)
            .await?;

        if let Some(HealthCheckerMsg_oneof::Pong(response)) = res_msg_enum.message {
            Ok(response)
        } else {
            // TODO: context
            Err(RpcError::InvalidRpcResponse)
        }
    }

    pub async fn disconnect_peer(&mut self, peer_id: PeerId) -> Result<(), NetworkError> {
        self.inner.disconnect_peer(peer_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        protocols::rpc::InboundRpcRequest,
        utils::MessageExt,
        validator_network::{Event, NetworkNotification},
    };
    use futures::{
        channel::oneshot, executor::block_on, future::try_join, sink::SinkExt, stream::StreamExt,
    };
    use prost::Message as _;

    // `HealthCheckerNetworkEvents` should deserialize inbound RPC requests
    #[test]
    fn test_health_checker_inbound_rpc() {
        let (mut network_reqs_tx, network_reqs_rx) = channel::new_test(8);
        let mut stream = HealthCheckerNetworkEvents::new(network_reqs_rx);

        // build rpc request
        let req_msg = Ping { nonce: 1234 };
        let req_msg_enum = HealthCheckerMsg {
            message: Some(HealthCheckerMsg_oneof::Ping(req_msg)),
        };
        let req_data = req_msg_enum.clone().to_bytes().unwrap();

        let (res_tx, _) = oneshot::channel();
        let rpc_req = InboundRpcRequest {
            protocol: ProtocolId::from_static(HEALTH_CHECKER_RPC_PROTOCOL),
            data: req_data,
            res_tx,
        };

        // mock receiving rpc request
        let peer_id = PeerId::random();
        let event = NetworkNotification::RecvRpc(peer_id, rpc_req);
        block_on(network_reqs_tx.send(event)).unwrap();

        // request should be properly deserialized
        let (res_tx, _) = oneshot::channel();
        let expected_event = Event::RpcRequest((peer_id, req_msg_enum.clone(), res_tx));
        let event = block_on(stream.next()).unwrap().unwrap();
        assert_eq!(event, expected_event);
    }

    // When health_checker sends an rpc request, network should get a
    // `NetworkRequest::SendRpc` with the serialized request.
    #[test]
    fn test_health_checker_outbound_rpc() {
        let (network_reqs_tx, mut network_reqs_rx) = channel::new_test(8);
        let mut sender = HealthCheckerNetworkSender::new(network_reqs_tx);

        // send ping rpc request
        let peer_id = PeerId::random();
        let req_msg = Ping { nonce: 1234 };
        let f_res_msg = sender.ping(peer_id, req_msg.clone(), Duration::from_secs(5));

        // build rpc response
        let res_msg = Pong { nonce: 1234 };
        let res_msg_enum = HealthCheckerMsg {
            message: Some(HealthCheckerMsg_oneof::Pong(res_msg.clone())),
        };
        let res_data = res_msg_enum.to_bytes().unwrap();

        // the future response
        let f_recv = async move {
            match network_reqs_rx.next().await.unwrap() {
                NetworkRequest::SendRpc(recv_peer_id, req) => {
                    assert_eq!(recv_peer_id, peer_id);
                    assert_eq!(req.protocol.as_ref(), HEALTH_CHECKER_RPC_PROTOCOL);

                    // check request deserializes
                    let req_msg_enum = HealthCheckerMsg::decode(req.data.as_ref()).unwrap();
                    assert_eq!(
                        req_msg_enum.message,
                        Some(HealthCheckerMsg_oneof::Ping(req_msg))
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
