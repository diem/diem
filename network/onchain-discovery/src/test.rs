// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::{
    client::OnchainDiscovery,
    network_interface::OnchainDiscoveryNetworkSender,
    service::OnchainDiscoveryService,
    types::{
        DiscoveryInfoInternal, DiscoverySetInternal, OnchainDiscoveryMsg, QueryDiscoverySetRequest,
        QueryDiscoverySetResponse,
    },
};
use channel::{libra_channel, message_queues::QueueStyle};
use executor::{db_bootstrapper::bootstrap_db_if_empty, Executor};
use futures::{channel::oneshot, sink::SinkExt, stream::StreamExt};
use libra_config::config::{NodeConfig, RoleType};
use libra_network_address::NetworkAddress;
use libra_types::{
    account_config,
    account_state::AccountState,
    on_chain_config::ValidatorSet,
    trusted_state::{TrustedState, TrustedStateChange},
    waypoint::Waypoint,
    PeerId,
};
use libra_vm::LibraVM;
use libradb::LibraDB;
use network::{
    connectivity_manager::ConnectivityRequest,
    peer_manager::{
        ConnectionNotification, ConnectionRequestSender, PeerManagerNotification,
        PeerManagerRequest, PeerManagerRequestSender,
    },
    protocols::rpc::{InboundRpcRequest, OutboundRpcRequest},
    ProtocolId,
};
use std::{
    collections::HashMap, convert::TryFrom, num::NonZeroUsize, str::FromStr, sync::Arc,
    time::Duration,
};
use storage_interface::{DbReader, DbReaderWriter};
use tokio::{
    runtime::{Handle, Runtime},
    task::JoinHandle,
};

struct MockOnchainDiscoveryNetworkSender {
    peer_mgr_notifs_tx: libra_channel::Sender<(PeerId, ProtocolId), PeerManagerNotification>,
    conn_notifs_tx: libra_channel::Sender<PeerId, ConnectionNotification>,
}

impl MockOnchainDiscoveryNetworkSender {
    fn new(
        peer_mgr_notifs_tx: libra_channel::Sender<(PeerId, ProtocolId), PeerManagerNotification>,
        conn_notifs_tx: libra_channel::Sender<PeerId, ConnectionNotification>,
    ) -> Self {
        Self {
            peer_mgr_notifs_tx,
            conn_notifs_tx,
        }
    }

    async fn query_discovery_set(
        &mut self,
        recipient: PeerId,
        req_msg: QueryDiscoverySetRequest,
    ) -> Box<QueryDiscoverySetResponse> {
        let req_msg = OnchainDiscoveryMsg::QueryDiscoverySetRequest(req_msg);
        let req_bytes = lcs::to_bytes(&req_msg).unwrap();
        let (res_tx, res_rx) = oneshot::channel();
        let inbound_rpc_req = InboundRpcRequest {
            protocol: ProtocolId::OnchainDiscoveryRpc,
            data: req_bytes.into(),
            res_tx,
        };

        let (delivered_tx, delivered_rx) = oneshot::channel();
        self.peer_mgr_notifs_tx
            .push_with_feedback(
                (recipient, ProtocolId::OnchainDiscoveryRpc),
                PeerManagerNotification::RecvRpc(recipient, inbound_rpc_req),
                Some(delivered_tx),
            )
            .unwrap();
        delivered_rx.await.unwrap();

        let res_bytes = res_rx.await.unwrap().unwrap();
        let res_msg: OnchainDiscoveryMsg = lcs::from_bytes(&res_bytes).unwrap();
        match res_msg {
            OnchainDiscoveryMsg::QueryDiscoverySetResponse(res_msg) => res_msg,
            OnchainDiscoveryMsg::QueryDiscoverySetRequest(_) => {
                panic!("Unexpected request msg, expected response msg")
            }
        }
    }

    async fn new_peer(&mut self, peer_id: PeerId) {
        let addr = NetworkAddress::from_str("/ip4/127.0.0.1/tcp/1234").unwrap();
        let notif = ConnectionNotification::NewPeer(peer_id, addr);
        self.send_connection_notif(peer_id, notif).await;
    }

    async fn send_connection_notif(&mut self, peer_id: PeerId, notif: ConnectionNotification) {
        let (delivered_tx, delivered_rx) = oneshot::channel();
        self.conn_notifs_tx
            .push_with_feedback(peer_id, notif, Some(delivered_tx))
            .unwrap();
        delivered_rx.await.unwrap();
    }

    async fn forward_outbound_rpc(&mut self, peer_id: PeerId, outbound_req: PeerManagerRequest) {
        let (protocol, inbound_req) = match outbound_req {
            PeerManagerRequest::SendRpc(
                _peer_id,
                OutboundRpcRequest {
                    protocol,
                    data,
                    res_tx,
                    ..
                },
            ) => (
                protocol,
                PeerManagerNotification::RecvRpc(
                    peer_id,
                    InboundRpcRequest {
                        protocol,
                        data,
                        res_tx,
                    },
                ),
            ),
            _ => panic!("Unexpected request, expected SendRpc: {:?}", outbound_req),
        };
        self.send_peer_mgr_notif(peer_id, protocol, inbound_req)
            .await;
    }

    async fn send_peer_mgr_notif(
        &mut self,
        peer_id: PeerId,
        protocol: ProtocolId,
        notif: PeerManagerNotification,
    ) {
        let (delivered_tx, delivered_rx) = oneshot::channel();
        self.peer_mgr_notifs_tx
            .push_with_feedback((peer_id, protocol), notif, Some(delivered_tx))
            .unwrap();
        delivered_rx.await.unwrap();
    }
}

fn read_validator_set(libra_db: &Arc<dyn DbReader>) -> ValidatorSet {
    let account_state_blob = libra_db
        .get_latest_account_state(account_config::validator_set_address())
        .unwrap()
        .unwrap();

    AccountState::try_from(&account_state_blob)
        .unwrap()
        .get_validator_set()
        .unwrap()
        .unwrap()
}

fn gen_configs(count: usize) -> Vec<NodeConfig> {
    config_builder::ValidatorConfig::new()
        .validators(count)
        .build_common(true, false)
        .unwrap()
        .0
}

fn setup_storage_service_and_executor(
    config: &NodeConfig,
) -> (Arc<dyn DbReader>, Executor<LibraVM>, Waypoint) {
    let (db_r, db_rw) = DbReaderWriter::wrap(LibraDB::new_for_test(&config.storage.dir()));
    let genesis_tx = config.execution.genesis.as_ref().unwrap();
    let waypoint = bootstrap_db_if_empty::<LibraVM>(&db_rw, genesis_tx)
        .expect("Db-bootstrapper should not fail.")
        .unwrap();
    let executor = Executor::new(db_rw);

    (db_r, executor, waypoint)
}

fn setup_onchain_discovery(
    executor: Handle,
    peer_id: PeerId,
    role: RoleType,
    libra_db: Arc<dyn DbReader>,
    waypoint: Waypoint,
) -> (
    JoinHandle<()>,
    JoinHandle<()>,
    MockOnchainDiscoveryNetworkSender,
    channel::Sender<()>,
    channel::Sender<()>,
    libra_channel::Receiver<(PeerId, ProtocolId), PeerManagerRequest>,
    channel::Receiver<ConnectivityRequest>,
) {
    let (peer_mgr_reqs_tx, peer_mgr_reqs_rx) =
        libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(8).unwrap(), None);
    let peer_mgr_reqs_tx = PeerManagerRequestSender::new(peer_mgr_reqs_tx);
    let (peer_mgr_notifs_tx, peer_mgr_notifs_rx) =
        libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(8).unwrap(), None);
    let (conn_notifs_tx, conn_notifs_rx) =
        libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(8).unwrap(), None);
    let (conn_mgr_reqs_tx, conn_mgr_reqs_rx) = channel::new_test(8);
    let (conn_reqs_tx, _) =
        libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(8).unwrap(), None);
    let conn_reqs_tx = ConnectionRequestSender::new(conn_reqs_tx);
    let network_reqs_tx =
        OnchainDiscoveryNetworkSender::new(peer_mgr_reqs_tx, conn_reqs_tx, conn_mgr_reqs_tx);
    // let network_notifs_rx = OnchainDiscoveryNetworkEvents::new(peer_mgr_notifs_rx, conn_notifs_rx);
    let (peer_query_ticker_tx, peer_query_ticker_rx) = channel::new_test::<()>(1);
    let (storage_query_ticker_tx, storage_query_ticker_rx) = channel::new_test::<()>(1);

    let outbound_rpc_timeout = Duration::from_secs(30);
    let max_concurrent_inbound_rpcs = 8;

    let onchain_discovery = OnchainDiscovery::new(
        peer_id,
        role,
        waypoint,
        network_reqs_tx,
        conn_notifs_rx,
        Arc::clone(&libra_db),
        peer_query_ticker_rx,
        storage_query_ticker_rx,
        outbound_rpc_timeout,
    );
    let f_onchain_discovery = executor.spawn(onchain_discovery.start());

    let service = OnchainDiscoveryService::new(
        executor.clone(),
        peer_mgr_notifs_rx,
        libra_db,
        max_concurrent_inbound_rpcs,
    );
    let f_service = executor.spawn(service.start());

    let mock_network_sender =
        MockOnchainDiscoveryNetworkSender::new(peer_mgr_notifs_tx, conn_notifs_tx);

    (
        f_onchain_discovery,
        f_service,
        mock_network_sender,
        peer_query_ticker_tx,
        storage_query_ticker_tx,
        peer_mgr_reqs_rx,
        conn_mgr_reqs_rx,
    )
}

#[test]
fn service_handles_remote_query() {
    ::libra_logger::Logger::new().environment_only(true).init();
    let mut rt = Runtime::new().unwrap();
    let config = gen_configs(1).swap_remove(0);
    let self_peer_id = config.validator_network.as_ref().unwrap().peer_id;
    let role = config.base.role;

    let (libra_db, _executor, waypoint) = setup_storage_service_and_executor(&config);
    let validator_set = read_validator_set(&libra_db);
    let expected_validator_set = DiscoverySetInternal::from_validator_set(role, validator_set);

    let (
        f_onchain_discovery,
        f_service,
        mut mock_network_tx,
        peer_query_ticker_tx,
        storage_query_ticker_tx,
        _peer_mgr_reqs_rx,
        _conn_mgr_reqs_rx,
    ) = setup_onchain_discovery(rt.handle().clone(), self_peer_id, role, libra_db, waypoint);

    let trusted_state = TrustedState::from(waypoint);

    // query the onchain discovery storage service
    let query_req = QueryDiscoverySetRequest {
        known_version: trusted_state.latest_version(),
    };
    let other_peer_id = PeerId::random();
    let query_res =
        rt.block_on(mock_network_tx.query_discovery_set(other_peer_id, query_req.clone()));

    // verify response and ratchet epoch_info
    let (trusted_state_change, opt_validator_set) = query_res
        .verify_and_ratchet(&query_req, &trusted_state)
        .unwrap();

    assert!(
        matches!(trusted_state_change, TrustedStateChange::Epoch { .. }),
        "Unexpected trusted_state_change, expected epoch change: actual change: {:?}",
        trusted_state_change
    );

    // verify validator set is the same as genesis validator set
    let actual_validator_set =
        DiscoverySetInternal::from_validator_set(role, opt_validator_set.unwrap());
    assert_eq!(expected_validator_set, actual_validator_set);

    // shutdown
    drop(mock_network_tx);
    drop(peer_query_ticker_tx);
    drop(storage_query_ticker_tx);

    rt.block_on(f_onchain_discovery).unwrap();
    rt.block_on(f_service).unwrap();
}

#[test]
fn queries_storage_on_tick() {
    ::libra_logger::Logger::new().environment_only(true).init();
    let mut rt = Runtime::new().unwrap();
    let config = gen_configs(1).swap_remove(0);
    let self_peer_id = config.validator_network.as_ref().unwrap().peer_id;
    let role = config.base.role;

    let (libra_db, _executor, waypoint) = setup_storage_service_and_executor(&config);
    let validator_set = read_validator_set(&libra_db);

    let (
        f_onchain_discovery,
        f_service,
        mock_network_tx,
        peer_query_ticker_tx,
        mut storage_query_ticker_tx,
        _peer_mgr_reqs_rx,
        conn_mgr_reqs_rx,
    ) = setup_onchain_discovery(rt.handle().clone(), self_peer_id, role, libra_db, waypoint);

    // trigger storage tick so onchain discovery queries its own storage
    rt.block_on(storage_query_ticker_tx.send(())).unwrap();

    // shutdown all channels so onchain discovery will drop conn_mgr_reqs_tx
    // when its done sending updates
    drop(mock_network_tx);
    drop(peer_query_ticker_tx);
    drop(storage_query_ticker_tx);

    // expect updates for all other nodes except ourselves
    let validator_set = DiscoverySetInternal::from_validator_set(role, validator_set);
    let expected_update_reqs = validator_set
        .0
        .into_iter()
        .filter(|(peer_id, _validator_info)| &self_peer_id != peer_id)
        .map(|(peer_id, DiscoveryInfoInternal(_id_pubkey, addrs))| (peer_id, addrs))
        .collect::<HashMap<_, _>>();

    // onchain discovery should notify connectivity manager about new peer infos
    let update_reqs = rt.block_on(conn_mgr_reqs_rx.collect::<Vec<_>>());
    let update_reqs = update_reqs
        .into_iter()
        .map(|req| match req {
            ConnectivityRequest::UpdateAddresses(_src, peer_id, addrs) => (peer_id, addrs),
            _ => panic!(
                "Unexpected ConnectivityRequest, expected UpdateAddresses: {:?}",
                req
            ),
        })
        .collect::<HashMap<_, _>>();

    assert_eq!(expected_update_reqs, update_reqs);

    // onchain discovery actor should terminate
    rt.block_on(f_onchain_discovery).unwrap();
    rt.block_on(f_service).unwrap();
}

#[test]
fn queries_peers_on_tick() {
    ::libra_logger::Logger::new().environment_only(true).init();
    let mut rt = Runtime::new().unwrap();
    let mut configs = gen_configs(5);

    // server setup

    let server_config = configs.swap_remove(0);
    let server_peer_id = server_config.validator_network.as_ref().unwrap().peer_id;
    let server_role = server_config.base.role;

    let (server_libra_db, _server_executor, waypoint) =
        setup_storage_service_and_executor(&server_config);

    let (
        f_server_onchain_discovery,
        f_server_service,
        mut server_network_tx,
        server_peer_query_ticker_tx,
        server_storage_query_ticker_tx,
        _server_peer_mgr_reqs_rx,
        _server_conn_mgr_reqs_rx,
    ) = setup_onchain_discovery(
        rt.handle().clone(),
        server_peer_id,
        server_role,
        server_libra_db,
        waypoint,
    );

    // client setup

    let client_config = configs.swap_remove(0);
    let client_peer_id = client_config.validator_network.as_ref().unwrap().peer_id;
    let client_role = client_config.base.role;

    let (libra_db, _executor, waypoint) = setup_storage_service_and_executor(&client_config);
    let validator_set = read_validator_set(&libra_db);

    let (
        f_client_onchain_discovery,
        f_client_service,
        mut client_network_tx,
        mut client_peer_query_ticker_tx,
        client_storage_query_ticker_tx,
        mut client_peer_mgr_reqs_rx,
        client_conn_mgr_reqs_rx,
    ) = setup_onchain_discovery(
        rt.handle().clone(),
        client_peer_id,
        client_role,
        libra_db,
        waypoint,
    );

    // notify client of new connection to server
    rt.block_on(client_network_tx.new_peer(server_peer_id));

    // trigger peer tick so client queries server peer
    rt.block_on(client_peer_query_ticker_tx.send(())).unwrap();

    // client should send a query discovery set request
    let outbound_req = rt.block_on(client_peer_mgr_reqs_rx.next()).unwrap();

    // forward the rpc request to the server
    rt.block_on(server_network_tx.forward_outbound_rpc(client_peer_id, outbound_req));

    // shutdown all channels so client onchain discovery will drop
    // conn_mgr_reqs_tx when its done sending updates
    drop(client_network_tx);
    drop(client_peer_query_ticker_tx);
    drop(client_storage_query_ticker_tx);

    // expect updates for all other nodes except ourselves
    let validator_set = DiscoverySetInternal::from_validator_set(client_role, validator_set);
    let expected_update_reqs = validator_set
        .0
        .into_iter()
        .filter(|(peer_id, _discovery_info)| &client_peer_id != peer_id)
        .map(|(peer_id, DiscoveryInfoInternal(_id_pubkey, addrs))| (peer_id, addrs))
        .collect::<HashMap<_, _>>();

    // client should notify its connectivity manager about new peer infos
    let update_reqs = rt.block_on(client_conn_mgr_reqs_rx.collect::<Vec<_>>());
    let update_reqs = update_reqs
        .into_iter()
        .map(|req| match req {
            ConnectivityRequest::UpdateAddresses(_src, peer_id, addrs) => (peer_id, addrs),
            _ => panic!(
                "Unexpected ConnectivityRequest, expected UpdateAddresses: {:?}",
                req
            ),
        })
        .collect::<HashMap<_, _>>();

    assert_eq!(expected_update_reqs, update_reqs);

    // client should shutdown completely
    rt.block_on(f_client_onchain_discovery).unwrap();
    rt.block_on(f_client_service).unwrap();

    // server shutdown
    drop(server_network_tx);
    drop(server_peer_query_ticker_tx);
    drop(server_storage_query_ticker_tx);

    rt.block_on(f_server_onchain_discovery).unwrap();
    rt.block_on(f_server_service).unwrap();
}
