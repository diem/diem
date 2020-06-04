// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Protobuf based interface between OnchainDiscovery and Network layers.
use crate::types::{OnchainDiscoveryMsg, QueryDiscoverySetRequest, QueryDiscoverySetResponse};
use channel::{libra_channel, message_queues::QueueStyle};
use libra_types::PeerId;
use network::{
    peer_manager::{
        ConnectionNotification, ConnectionRequestSender, PeerManagerNotification,
        PeerManagerRequestSender,
    },
    protocols::{network::NetworkSender, rpc::error::RpcError},
    validator_network::network_builder::{NetworkBuilder, NETWORK_CHANNEL_SIZE},
    ProtocolId,
};
use std::time::Duration;

/// The interface from Network to OnchainDiscovery layer.
///
/// `OnchainDiscoveryNetworkEvents` is a `Stream` of `PeerManagerNotification` where the
/// raw `Bytes` rpc messages are deserialized into
/// `OnchainDiscoveryMsg` types. `OnchainDiscoveryNetworkEvents` is a thin wrapper
/// around an `channel::Receiver<PeerManagerNotification>`.
pub struct OnchainDiscoveryNetworkEvents {
    pub peer_mgr_notifs_rx: libra_channel::Receiver<(PeerId, ProtocolId), PeerManagerNotification>,
    pub connection_notifs_rx: libra_channel::Receiver<PeerId, ConnectionNotification>,
}

impl OnchainDiscoveryNetworkEvents {
    pub fn new(
        peer_mgr_notifs_rx: libra_channel::Receiver<(PeerId, ProtocolId), PeerManagerNotification>,
        connection_notifs_rx: libra_channel::Receiver<PeerId, ConnectionNotification>,
    ) -> Self {
        Self {
            peer_mgr_notifs_rx,
            connection_notifs_rx,
        }
    }
}

/// The interface from OnchainDiscovery to Networking layer.
#[derive(Clone)]
pub struct OnchainDiscoveryNetworkSender {
    inner: NetworkSender<OnchainDiscoveryMsg>,
}

impl OnchainDiscoveryNetworkSender {
    pub fn new(
        peer_mgr_reqs_tx: PeerManagerRequestSender,
        conn_reqs_tx: ConnectionRequestSender,
    ) -> Self {
        Self {
            inner: NetworkSender::new(peer_mgr_reqs_tx, conn_reqs_tx),
        }
    }

    pub async fn query_discovery_set(
        &mut self,
        recipient: PeerId,
        req_msg: QueryDiscoverySetRequest,
        timeout: Duration,
    ) -> Result<Box<QueryDiscoverySetResponse>, RpcError> {
        let protocol = ProtocolId::OnchainDiscoveryRpc;
        let req_msg_enum = OnchainDiscoveryMsg::QueryDiscoverySetRequest(req_msg);

        let res_msg_enum = self
            .inner
            .send_rpc(recipient, protocol, req_msg_enum, timeout)
            .await?;

        let res_msg = match res_msg_enum {
            OnchainDiscoveryMsg::QueryDiscoverySetResponse(res_msg) => res_msg,
            OnchainDiscoveryMsg::QueryDiscoverySetRequest(_) => {
                return Err(RpcError::InvalidRpcResponse);
            }
        };
        Ok(res_msg)
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
            NETWORK_CHANNEL_SIZE,
            // Some(&counters::PENDING_CONSENSUS_NETWORK_EVENTS),
            // TODO(philiphayes): add a counter for onchain discovery
            None,
        );
    (
        OnchainDiscoveryNetworkSender::new(network_sender, conn_reqs_tx),
        OnchainDiscoveryNetworkEvents::new(network_receiver, conn_notifs_rx),
    )
}
