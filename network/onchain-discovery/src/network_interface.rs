// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Protobuf based interface between OnchainDiscovery and Network layers.
use crate::client::OnchainDiscovery;
use crate::service::OnchainDiscoveryService;
use crate::types::{OnchainDiscoveryMsg, QueryDiscoverySetRequest, QueryDiscoverySetResponse};
use channel::{libra_channel, message_queues::QueueStyle};
use futures::{channel::mpsc, sink::SinkExt, stream::StreamExt};
use libra_config::config::RoleType;
use libra_types::waypoint::Waypoint;
use libra_types::PeerId;
use network::peer_manager::{ConnectionNotification, ConnectionRequest, PeerManagerRequest};
use network::{
    connectivity_manager::ConnectivityRequest,
    peer_manager::{
        conn_notifs_channel, ConnectionRequestSender, PeerManagerNotification,
        PeerManagerRequestSender,
    },
    protocols::{network::NetworkSender, rpc::error::RpcError},
    traits::FromPeerManagerConnectionConnectivityRequestSenders,
    validator_network::network_builder::NETWORK_CHANNEL_SIZE,
    ProtocolId,
};
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::Duration;
use storage_interface::DbReader;
use tokio::{runtime::Handle, time::interval};

/// The interface from OnchainDiscovery to Networking layer.
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
    ) -> Result<Box<QueryDiscoverySetResponse>, RpcError> {
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
        Ok(res_msg)
    }

    pub async fn send_connectivity_request(
        &mut self,
        req: ConnectivityRequest,
    ) -> Result<(), mpsc::SendError> {
        self.conn_mgr_reqs_tx.send(req).await
    }
}

impl FromPeerManagerConnectionConnectivityRequestSenders for OnchainDiscoveryNetworkSender {
    fn from_peer_manager_connection_request_connection_manager_senders(
        pm_reqs_tx: PeerManagerRequestSender,
        connection_reqs_tx: ConnectionRequestSender,
        connectivity_reqs_tx: channel::Sender<ConnectivityRequest>,
    ) -> Self {
        Self::new(pm_reqs_tx, connection_reqs_tx, connectivity_reqs_tx)
    }
}

pub fn setup_onchain_discovery(
    peer_id: PeerId,
    role: RoleType,
    libra_db: Arc<dyn DbReader>,
    waypoint: Waypoint,
    executor: &Handle,
    connection_reqs_tx: libra_channel::Sender<PeerId, ConnectionRequest>,
    pm_reqs_tx: libra_channel::Sender<(PeerId, ProtocolId), PeerManagerRequest>,
    connectivity_reqs_tx: channel::Sender<ConnectivityRequest>,
) -> (
    ProtocolId,
    libra_channel::Sender<(PeerId, ProtocolId), PeerManagerNotification>,
    libra_channel::Sender<PeerId, ConnectionNotification>,
) {
    // add_protocol_handler
    let (network_notifs_tx, network_notifs_rx) = libra_channel::new(
        QueueStyle::LIFO,
        NonZeroUsize::new(NETWORK_CHANNEL_SIZE).unwrap(),
        None,
    );
    let (connection_notifs_tx, connection_notifs_rx) = conn_notifs_channel::new();

    let network_tx = OnchainDiscoveryNetworkSender::from_channel_senders(
        pm_reqs_tx,
        connection_reqs_tx,
        connectivity_reqs_tx,
    );

    // let (network_tx, network_notifs_rx, connection_notifs_rx) =
    //     onchain_discovery::network_interface::add_to_network(network);

    let outbound_rpc_timeout = Duration::from_secs(30);
    let max_concurrent_inbound_queries = 8;

    let onchain_discovery_service = OnchainDiscoveryService::new(
        executor.clone(),
        network_notifs_rx,
        Arc::clone(&libra_db),
        max_concurrent_inbound_queries,
    );
    executor.spawn(onchain_discovery_service.start());

    let onchain_discovery = executor.enter(move || {
        let peer_query_ticker = interval(Duration::from_secs(30)).fuse();
        let storage_query_ticker = interval(Duration::from_secs(30)).fuse();

        OnchainDiscovery::new(
            peer_id,
            role,
            waypoint,
            network_tx,
            connection_notifs_rx,
            libra_db,
            peer_query_ticker,
            storage_query_ticker,
            outbound_rpc_timeout,
        )
    });
    executor.spawn(onchain_discovery.start());
    (
        ProtocolId::OnchainDiscoveryRpc,
        network_notifs_tx,
        connection_notifs_tx,
    )
}
