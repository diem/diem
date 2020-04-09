// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Protobuf based interface between OnchainDiscovery and Network layers.
use crate::types::{
    OnchainDiscoveryMsg, QueryDiscoverySetRequest, QueryDiscoverySetResponseWithEvent,
};
use channel::message_queues::QueueStyle;
use futures::{channel::mpsc, sink::SinkExt};
use libra_types::PeerId;
use network::{
    connectivity_manager::ConnectivityRequest,
    peer_manager::{ConnectionRequestSender, PeerManagerRequestSender},
    protocols::{
        network::{NetworkEvents, NetworkSender},
        rpc::error::RpcError,
    },
    validator_network::network_builder::NetworkBuilder,
    ProtocolId,
};
use std::{convert::TryFrom, time::Duration};

/// The interface from Network to OnchainDiscovery module.
///
/// `OnchainDiscoveryNetworkEvents` is a `Stream` of `NetworkNotification` where
/// the raw `Bytes` rpc messages are deserialized into `OnchainDiscoveryMsg`
/// types. `OnchainDiscoveryNetworkEvents` is a thin wrapper around a
/// `channel::Receiver<NetworkNotification>`.
pub type OnchainDiscoveryNetworkEvents = NetworkEvents<OnchainDiscoveryMsg>;

/// The interface from OnchainDiscovery to Networking layer.
///
/// This is a thin wrapper around a `NetworkSender<OnchainDiscoveryMsg>`, which
/// is in turn a thin wrapper around a `channel::Sender<NetworkRequest>`, so it
/// is easy to clone and send off to a separate task. For example, the rpc
/// requests return Futures that encapsulate the whole flow, from sending the
/// request to remote, to finally receiving the response and deserializing. It
/// therefore makes the most sense to make the rpc call on a separate async
/// task, which requires the `OnchainDiscoveryNetworkSender` to be `Clone` and
/// `Send`.
#[derive(Clone)]
pub struct OnchainDiscoveryNetworkSender {
    network_sender: NetworkSender<OnchainDiscoveryMsg>,
    conn_mgr_reqs_tx: channel::Sender<ConnectivityRequest>,
}

impl OnchainDiscoveryNetworkSender {
    pub fn new(
        peer_mgr_reqs_tx: PeerManagerRequestSender,
        conn_reqs_tx: ConnectionRequestSender,
        conn_mgr_reqs_tx: channel::Sender<ConnectivityRequest>,
    ) -> Self {
        Self {
            network_sender: NetworkSender::new(peer_mgr_reqs_tx, conn_reqs_tx),
            conn_mgr_reqs_tx,
        }
    }

    pub async fn query_discovery_set(
        &mut self,
        recipient: PeerId,
        req_msg: QueryDiscoverySetRequest,
        timeout: Duration,
    ) -> Result<QueryDiscoverySetResponseWithEvent, RpcError> {
        let protocol = ProtocolId::OnchainDiscoveryRpc;
        let req_msg_enum = OnchainDiscoveryMsg::QueryDiscoverySetRequest(req_msg);

        let res_msg_enum = self
            .network_sender
            .send_rpc(recipient, protocol, req_msg_enum, timeout)
            .await?;

        let res_msg = match res_msg_enum {
            OnchainDiscoveryMsg::QueryDiscoverySetResponse(res_msg) => res_msg,
            OnchainDiscoveryMsg::QueryDiscoverySetRequest(_) => {
                return Err(RpcError::InvalidRpcResponse);
            }
        };
        let res_msg = QueryDiscoverySetResponseWithEvent::try_from(res_msg)
            .map_err(RpcError::ApplicationError)?;
        Ok(res_msg)
    }

    pub async fn send_connectivity_request(
        &mut self,
        req: ConnectivityRequest,
    ) -> Result<(), mpsc::SendError> {
        self.conn_mgr_reqs_tx.send(req).await
    }
}

/// Construct OnchainDiscoveryNetworkSender/Events and register them with the
/// given network builder.
pub fn add_to_network(
    network: &mut NetworkBuilder,
) -> (OnchainDiscoveryNetworkSender, OnchainDiscoveryNetworkEvents) {
    let (network_sender, network_receiver, conn_reqs_tx, conn_notifs_rx) = network
        .add_protocol_handler(
            vec![ProtocolId::OnchainDiscoveryRpc],
            vec![],
            QueueStyle::LIFO,
            // Some(&counters::PENDING_CONSENSUS_NETWORK_EVENTS),
            // TODO(philiphayes): add a counter for onchain discovery
            None,
        );
    (
        OnchainDiscoveryNetworkSender::new(
            network_sender,
            conn_reqs_tx,
            network
                .conn_mgr_reqs_tx()
                .expect("ConnecitivtyManager not enabled"),
        ),
        OnchainDiscoveryNetworkEvents::new(network_receiver, conn_notifs_rx),
    )
}
