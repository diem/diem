// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Interface between Admission Control and Network layers.

use crate::{
    error::NetworkError,
    interface::{NetworkNotification, NetworkRequest},
    proto::{AdmissionControlMsg, SubmitTransactionRequest, SubmitTransactionResponse},
    protocols::rpc::{self, error::RpcError},
    validator_network::Event,
    ProtocolId,
};
use channel;
use futures::{
    stream::Map,
    task::{Context, Poll},
    Stream, StreamExt,
};
use pin_utils::unsafe_pinned;
use std::{pin::Pin, time::Duration};
use types::PeerId;

/// Protocol id for admission control RPC calls
pub const ADMISSION_CONTROL_RPC_PROTOCOL: &[u8] = b"/libra/admission_control/rpc/0.1.0";

/// The interface from Network to Admission Control layer.
///
/// `AdmissionControlNetworkEvents` is a `Stream` of `NetworkNotification` where the
/// raw `Bytes` direct-send and rpc messages are deserialized into
/// `AdmissionControlMsg` types. `AdmissionControlNetworkEvents` is a thin wrapper around
/// an `channel::Receiver<NetworkNotification>`.
pub struct AdmissionControlNetworkEvents {
    inner: Map<
        channel::Receiver<NetworkNotification>,
        fn(NetworkNotification) -> Result<Event<AdmissionControlMsg>, NetworkError>,
    >,
}

impl AdmissionControlNetworkEvents {
    // This use of `unsafe_pinned` is safe because:
    //   1. This struct does not implement [`Drop`]
    //   2. This struct does not implement [`Unpin`]
    //   3. This struct is not `#[repr(packed)]`
    unsafe_pinned!(
        inner:
            Map<
                channel::Receiver<NetworkNotification>,
                fn(NetworkNotification) -> Result<Event<AdmissionControlMsg>, NetworkError>,
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

impl Stream for AdmissionControlNetworkEvents {
    type Item = Result<Event<AdmissionControlMsg>, NetworkError>;

    fn poll_next(self: Pin<&mut Self>, context: &mut Context) -> Poll<Option<Self::Item>> {
        self.inner().poll_next(context)
    }
}

/// The interface from Admission Control to Network layer.
///
/// This is a thin wrapper around an `channel::Sender<NetworkRequest>`, so it is
/// easy to clone and send off to a separate task. For example, the rpc requests
/// return Futures that encapsulate the whole flow, from sending the request to
/// remote, to finally receiving the response and deserializing. It therefore
/// makes the most sense to make the rpc call on a separate async task, which
/// requires the `AdmissionControlNetworkSender` to be `Clone` and `Send`.
#[derive(Clone)]
pub struct AdmissionControlNetworkSender {
    inner: channel::Sender<NetworkRequest>,
}

impl AdmissionControlNetworkSender {
    pub fn new(inner: channel::Sender<NetworkRequest>) -> Self {
        Self { inner }
    }

    /// Send a SubmitTransactionRequest RPC request to remote peer `recipient`. Returns the
    /// future `SubmitTransactionResponse` returned by the remote peer.
    ///
    /// The rpc request can be canceled at any point by dropping the returned
    /// future.
    pub async fn send_transaction_upstream(
        &mut self,
        recipient: PeerId,
        req_msg: SubmitTransactionRequest,
        timeout: Duration,
    ) -> Result<SubmitTransactionResponse, RpcError> {
        let protocol = ProtocolId::from_static(ADMISSION_CONTROL_RPC_PROTOCOL);
        let mut send_txn_req_msg_enum = AdmissionControlMsg::new();
        send_txn_req_msg_enum.set_submit_transaction_request(req_msg);
        let mut res_msg_enum = rpc::utils::unary_rpc(
            self.inner.clone(),
            recipient,
            protocol,
            send_txn_req_msg_enum,
            timeout,
        )
        .await?;

        if res_msg_enum.has_submit_transaction_response() {
            Ok(res_msg_enum.take_submit_transaction_response())
        } else {
            // TODO: context
            Err(RpcError::InvalidRpcResponse)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocols::rpc::InboundRpcRequest;
    use futures::{channel::oneshot, executor::block_on, future::try_join, SinkExt};
    use protobuf::Message as proto_msg;

    // `AdmissionControlNetworkEvents` should deserialize inbound RPC requests
    #[test]
    fn test_admission_control_inbound_rpc() {
        let (mut admission_control_tx, admission_control_rx) = channel::new_test(8);
        let mut stream = AdmissionControlNetworkEvents::new(admission_control_rx);

        // build rpc request
        let req_msg = SubmitTransactionRequest::new();
        let mut req_msg_enum = AdmissionControlMsg::new();
        req_msg_enum.set_submit_transaction_request(req_msg);
        let req_data = req_msg_enum.clone().write_to_bytes().unwrap().into();

        let (res_tx, _) = oneshot::channel();
        let rpc_req = InboundRpcRequest {
            protocol: ProtocolId::from_static(ADMISSION_CONTROL_RPC_PROTOCOL),
            data: req_data,
            res_tx,
        };

        // mock receiving rpc request
        let peer_id = PeerId::random();
        let event = NetworkNotification::RecvRpc(peer_id, rpc_req);
        block_on(admission_control_tx.send(event)).unwrap();

        // request should be properly deserialized
        let (res_tx, _) = oneshot::channel();
        let expected_event = Event::RpcRequest((peer_id, req_msg_enum.clone(), res_tx));
        let event = block_on(stream.next()).unwrap().unwrap();
        assert_eq!(event, expected_event);
    }

    // When AC sends an rpc request, network should get a `NetworkRequest::SendRpc`
    // with the serialized request.
    #[test]
    fn test_admission_control_outbound_rpc() {
        let (network_reqs_tx, mut network_reqs_rx) = channel::new_test(8);
        let mut sender = AdmissionControlNetworkSender::new(network_reqs_tx);

        // make submit_transaction_request rpc request
        let peer_id = PeerId::random();
        let req_msg = SubmitTransactionRequest::new();
        let f_res_msg =
            sender.send_transaction_upstream(peer_id, req_msg.clone(), Duration::from_secs(5));

        // build rpc response
        let res_msg = SubmitTransactionResponse::new();
        let mut res_msg_enum = AdmissionControlMsg::new();
        res_msg_enum.set_submit_transaction_response(res_msg.clone());
        let res_data = res_msg_enum.write_to_bytes().unwrap().into();

        // the future response
        let f_recv = async move {
            match network_reqs_rx.next().await.unwrap() {
                NetworkRequest::SendRpc(recv_peer_id, req) => {
                    assert_eq!(recv_peer_id, peer_id);
                    assert_eq!(req.protocol.as_ref(), ADMISSION_CONTROL_RPC_PROTOCOL);

                    // check request deserializes
                    let mut req_msg_enum: AdmissionControlMsg =
                        ::protobuf::parse_from_bytes(req.data.as_ref()).unwrap();
                    let recv_req_msg = req_msg_enum.take_submit_transaction_request();
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
