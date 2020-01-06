// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Interface between Admission Control and Network layers.

use crate::{
    interface::NetworkRequest,
    protocols::rpc::error::RpcError,
    validator_network::{NetworkEvents, NetworkSender},
    ProtocolId,
};
use admission_control_proto::proto::admission_control::{
    admission_control_msg::Message as AdmissionControlMsg_oneof, AdmissionControlMsg,
    SubmitTransactionRequest, SubmitTransactionResponse,
};
use channel;
use libra_types::PeerId;
use std::time::Duration;

/// Protocol id for admission control RPC calls
pub const ADMISSION_CONTROL_RPC_PROTOCOL: &[u8] = b"/libra/rpc/0.1.0/admission_control/0.1.0";

/// The interface from Network to Admission Control layer.
///
/// `AdmissionControlNetworkEvents` is a `Stream` of `NetworkNotification` where the
/// raw `Bytes` direct-send and rpc messages are deserialized into
/// `AdmissionControlMsg` types. `AdmissionControlNetworkEvents` is a thin wrapper around
/// an `channel::Receiver<NetworkNotification>`.
pub type AdmissionControlNetworkEvents = NetworkEvents<AdmissionControlMsg>;

/// The interface from Admission Control to Network layer.
///
/// This is a thin wrapper around a `NetworkSender<AdmissionControlMsg>`, which
/// is in turn a thin wrapper around a `channel::Sender<NetworkRequest>`, so it
/// is easy to clone and send off to a separate task. For example, the rpc
/// requests return Futures that encapsulate the whole flow, from sending the
/// request to remote, to finally receiving the response and deserializing. It
/// therefore makes the most sense to make the rpc call on a separate async task,
/// which requires the `AdmissionControlNetworkSender` to be `Clone` and `Send`.
#[derive(Clone)]
pub struct AdmissionControlNetworkSender {
    inner: NetworkSender<AdmissionControlMsg>,
}

impl AdmissionControlNetworkSender {
    pub fn new(inner: channel::Sender<NetworkRequest>) -> Self {
        Self {
            inner: NetworkSender::new(inner),
        }
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
        let req_msg_enum = AdmissionControlMsg {
            message: Some(AdmissionControlMsg_oneof::SubmitTransactionRequest(req_msg)),
        };

        let res_msg_enum = self
            .inner
            .unary_rpc(recipient, protocol, req_msg_enum, timeout)
            .await?;

        if let Some(AdmissionControlMsg_oneof::SubmitTransactionResponse(response)) =
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
        interface::NetworkNotification, protocols::rpc::InboundRpcRequest, utils::MessageExt,
        validator_network::Event,
    };
    use futures::{
        channel::oneshot, executor::block_on, future::try_join, sink::SinkExt, stream::StreamExt,
    };
    use prost::Message as _;

    // `AdmissionControlNetworkEvents` should deserialize inbound RPC requests
    #[test]
    fn test_admission_control_inbound_rpc() {
        let (mut admission_control_tx, admission_control_rx) = channel::new_test(8);
        let mut stream = AdmissionControlNetworkEvents::new(admission_control_rx);

        // build rpc request
        let req_msg = SubmitTransactionRequest::default();
        let req_msg_enum = AdmissionControlMsg {
            message: Some(AdmissionControlMsg_oneof::SubmitTransactionRequest(req_msg)),
        };

        let req_data = req_msg_enum.clone().to_bytes().unwrap();

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
        let expected_event = Event::RpcRequest((peer_id, req_msg_enum, res_tx));
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
        let req_msg = SubmitTransactionRequest::default();
        let f_res_msg =
            sender.send_transaction_upstream(peer_id, req_msg.clone(), Duration::from_secs(5));

        // build rpc response
        let res_msg = SubmitTransactionResponse::default();
        let res_msg_enum = AdmissionControlMsg {
            message: Some(AdmissionControlMsg_oneof::SubmitTransactionResponse(
                res_msg.clone(),
            )),
        };
        let res_data = res_msg_enum.to_bytes().unwrap();

        // the future response
        let f_recv = async move {
            match network_reqs_rx.next().await.unwrap() {
                NetworkRequest::SendRpc(recv_peer_id, req) => {
                    assert_eq!(recv_peer_id, peer_id);
                    assert_eq!(req.protocol.as_ref(), ADMISSION_CONTROL_RPC_PROTOCOL);

                    // check request deserializes
                    let mut req_msg_enum = AdmissionControlMsg::decode(req.data.as_ref()).unwrap();
                    let recv_req_msg = req_msg_enum.message.take();
                    assert_eq!(
                        recv_req_msg,
                        Some(AdmissionControlMsg_oneof::SubmitTransactionRequest(req_msg))
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
