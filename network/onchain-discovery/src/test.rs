// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::{
    network_interface::{OnchainDiscoveryNetworkEvents, OnchainDiscoveryNetworkSender},
    types::{
        DiscoveryInfoInternal, DiscoverySetInternal, OnchainDiscoveryMsg, QueryDiscoverySetRequest,
        QueryDiscoverySetResponseWithEvent,
    },
    OnchainDiscovery,
};
use channel::{libra_channel, message_queues::QueueStyle};
use executor::{db_bootstrapper::maybe_bootstrap_db, Executor};
use futures::{channel::oneshot, sink::SinkExt, stream::StreamExt};
use libra_config::{
    config::{NodeConfig, RoleType},
    generator::{self, ValidatorSwarm},
};
use libra_types::{
    discovery_set::DiscoverySet, trusted_state::TrustedState, validator_set::ValidatorSet, PeerId,
};
use libra_vm::LibraVM;
use network::{
    connectivity_manager::ConnectivityRequest,
    peer_manager::{
        ConnectionRequestSender, ConnectionStatusNotification, PeerManagerNotification,
        PeerManagerRequest, PeerManagerRequestSender,
    },
    protocols::rpc::{InboundRpcRequest, OutboundRpcRequest},
    ProtocolId,
};
use parity_multiaddr::Multiaddr;
use std::{
    collections::HashMap, convert::TryFrom, num::NonZeroUsize, str::FromStr, sync::Arc,
    time::Duration,
};
use storage_client::{StorageRead, StorageReadServiceClient, SyncStorageClient};
use storage_service::{init_libra_db, start_storage_service_with_db};
use tokio::{
    runtime::{Handle, Runtime},
    task::JoinHandle,
};
use vm_genesis::{encode_genesis_transaction_with_validator, GENESIS_KEYPAIR};

struct MockOnchainDiscoveryNetworkSender {
    peer_mgr_notifs_tx: libra_channel::Sender<(PeerId, ProtocolId), PeerManagerNotification>,
    conn_notifs_tx: libra_channel::Sender<PeerId, ConnectionStatusNotification>,
}

impl MockOnchainDiscoveryNetworkSender {
    fn new(
        peer_mgr_notifs_tx: libra_channel::Sender<(PeerId, ProtocolId), PeerManagerNotification>,
        conn_notifs_tx: libra_channel::Sender<PeerId, ConnectionStatusNotification>,
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
    ) -> QueryDiscoverySetResponseWithEvent {
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
        let res_msg = match res_msg {
            OnchainDiscoveryMsg::QueryDiscoverySetResponse(res_msg) => res_msg,
            OnchainDiscoveryMsg::QueryDiscoverySetRequest(_) => {
                panic!("Unexpected request msg, expected response msg")
            }
        };
        let res_msg = QueryDiscoverySetResponseWithEvent::try_from(res_msg).unwrap();
        res_msg
    }

    async fn new_peer(&mut self, peer_id: PeerId) {
        let addr = Multiaddr::from_str("/ip4/127.0.0.1/tcp/1234").unwrap();
        let notif = ConnectionStatusNotification::NewPeer(peer_id, addr);
        self.send_connection_notif(peer_id, notif).await;
    }

    async fn send_connection_notif(
        &mut self,
        peer_id: PeerId,
        notif: ConnectionStatusNotification,
    ) {
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
                protocol.clone(),
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

fn gen_configs(count: usize) -> (Vec<NodeConfig>, ValidatorSet, DiscoverySet) {
    let config_template = NodeConfig::default();
    let config_seed = [42; 32];
    let randomize_service_ports = false;
    let randomize_libranet_ports = false;

    let ValidatorSwarm {
        mut nodes,
        validator_set,
        discovery_set,
    } = generator::validator_swarm(
        &config_template,
        count,
        config_seed,
        randomize_service_ports,
        randomize_libranet_ports,
    );

    let vm_publishing_option = None;
    let genesis = encode_genesis_transaction_with_validator(
        &GENESIS_KEYPAIR.0,
        GENESIS_KEYPAIR.1.clone(),
        &nodes[..],
        validator_set.clone(),
        discovery_set.clone(),
        vm_publishing_option,
    );

    for node in &mut nodes {
        node.execution.genesis = Some(genesis.clone());
    }

    (nodes, validator_set, discovery_set)
}

fn gen_single_config(count: usize) -> (NodeConfig, ValidatorSet, DiscoverySet) {
    let (mut nodes, validator_set, discovery_set) = gen_configs(count);
    let node = nodes.swap_remove(0);
    (node, validator_set, discovery_set)
}

fn setup_storage_service_and_executor(
    config: &NodeConfig,
) -> (Runtime, Arc<dyn StorageRead>, Executor<LibraVM>) {
    let (arc_db, db_reader_writer) = init_libra_db(config);
    maybe_bootstrap_db::<LibraVM>(db_reader_writer, config)
        .expect("Db-bootstrapper should not fail.");
    let storage_runtime = start_storage_service_with_db(config, arc_db);

    let storage_read_client = Arc::new(StorageReadServiceClient::new(&config.storage.address));
    let executor = Executor::new(SyncStorageClient::new(&config.storage.address).into());

    (storage_runtime, storage_read_client, executor)
}

fn setup_onchain_discovery(
    executor: Handle,
    peer_id: PeerId,
    role: RoleType,
    storage_read_client: Arc<dyn StorageRead>,
) -> (
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
    let network_notifs_rx = OnchainDiscoveryNetworkEvents::new(peer_mgr_notifs_rx, conn_notifs_rx);
    let (peer_query_ticker_tx, peer_query_ticker_rx) = channel::new_test::<()>(1);
    let (storage_query_ticker_tx, storage_query_ticker_rx) = channel::new_test::<()>(1);

    let waypoint = None;
    let outbound_rpc_timeout = Duration::from_secs(30);
    let max_concurrent_inbound_rpcs = 8;

    let onchain_discovery = OnchainDiscovery::new(
        executor.clone(),
        peer_id,
        role,
        waypoint,
        network_reqs_tx,
        network_notifs_rx,
        storage_read_client,
        peer_query_ticker_rx,
        storage_query_ticker_rx,
        outbound_rpc_timeout,
        max_concurrent_inbound_rpcs,
    );

    let f_onchain_discovery = executor.spawn(onchain_discovery.start());

    let mock_network_sender =
        MockOnchainDiscoveryNetworkSender::new(peer_mgr_notifs_tx, conn_notifs_tx);

    (
        f_onchain_discovery,
        mock_network_sender,
        peer_query_ticker_tx,
        storage_query_ticker_tx,
        peer_mgr_reqs_rx,
        conn_mgr_reqs_rx,
    )
}

#[test]
fn handles_remote_query() {
    ::libra_logger::Logger::new().environment_only(true).init();
    let mut rt = Runtime::new().unwrap();
    let (config, _, discovery_set) = gen_single_config(5);
    let self_peer_id = config.validator_network.as_ref().unwrap().peer_id;
    let role = config.base.role;

    let (_storage_runtime, storage_read_client, _executor) =
        setup_storage_service_and_executor(&config);

    let (
        f_onchain_discovery,
        mut mock_network_tx,
        peer_query_ticker_tx,
        storage_query_ticker_tx,
        _peer_mgr_reqs_rx,
        _conn_mgr_reqs_rx,
    ) = setup_onchain_discovery(rt.handle().clone(), self_peer_id, role, storage_read_client);

    // query the onchain discovery actor
    let query_req = QueryDiscoverySetRequest {
        client_known_version: 0,
        client_known_seq_num: 0,
    };

    let other_peer_id = PeerId::random();
    let query_res =
        rt.block_on(mock_network_tx.query_discovery_set(other_peer_id, query_req.clone().into()));

    // verify response and ratchet epoch_info
    let trusted_state = TrustedState::new_trust_any_genesis_WARNING_UNSAFE();
    query_res
        .update_to_latest_ledger_response
        .verify(&trusted_state, &query_req.into())
        .unwrap();

    // verify discovery set is the same as genesis discovery set
    let discovery_set_event = query_res.event.unwrap();
    assert_eq!(0, discovery_set_event.event_seq_num);

    let expected_discovery_set = DiscoverySetInternal::from_discovery_set(role, discovery_set);
    let actual_discovery_set =
        DiscoverySetInternal::from_discovery_set(role, discovery_set_event.discovery_set);
    assert_eq!(expected_discovery_set, actual_discovery_set);

    // shutdown
    drop(mock_network_tx);
    drop(peer_query_ticker_tx);
    drop(storage_query_ticker_tx);

    rt.block_on(f_onchain_discovery).unwrap();
}

#[test]
fn queries_storage_on_tick() {
    ::libra_logger::Logger::new().environment_only(true).init();
    let mut rt = Runtime::new().unwrap();
    let (config, _, discovery_set) = gen_single_config(5);
    let self_peer_id = config.validator_network.as_ref().unwrap().peer_id;
    let role = config.base.role;

    let (_storage_runtime, storage_read_client, _executor) =
        setup_storage_service_and_executor(&config);

    let (
        f_onchain_discovery,
        mock_network_tx,
        peer_query_ticker_tx,
        mut storage_query_ticker_tx,
        _peer_mgr_reqs_rx,
        conn_mgr_reqs_rx,
    ) = setup_onchain_discovery(rt.handle().clone(), self_peer_id, role, storage_read_client);

    // trigger storage tick so onchain discovery queries its own storage
    rt.block_on(storage_query_ticker_tx.send(())).unwrap();

    // shutdown all channels so onchain discovery will drop conn_mgr_reqs_tx
    // when its done sending updates
    drop(mock_network_tx);
    drop(peer_query_ticker_tx);
    drop(storage_query_ticker_tx);

    // expect updates for all other nodes except ourselves
    let discovery_set = DiscoverySetInternal::from_discovery_set(role, discovery_set);
    let expected_update_reqs = discovery_set
        .0
        .into_iter()
        .filter(|(peer_id, _discovery_info)| &self_peer_id != peer_id)
        .map(|(peer_id, DiscoveryInfoInternal(_id_pubkey, addrs))| (peer_id, addrs))
        .collect::<HashMap<_, _>>();

    // onchain discovery should notify connectivity manager about new peer infos
    let update_reqs = rt.block_on(conn_mgr_reqs_rx.collect::<Vec<_>>());
    let update_reqs = update_reqs
        .into_iter()
        .map(|req| match req {
            ConnectivityRequest::UpdateAddresses(peer_id, addrs) => (peer_id, addrs),
            _ => panic!(
                "Unexpected ConnectivityRequest, expected UpdateAddresses: {:?}",
                req
            ),
        })
        .collect::<HashMap<_, _>>();

    assert_eq!(expected_update_reqs, update_reqs);

    // onchain discovery actor should terminate
    rt.block_on(f_onchain_discovery).unwrap();
}

#[test]
fn queries_peers_on_tick() {
    ::libra_logger::Logger::new().environment_only(true).init();
    let mut rt = Runtime::new().unwrap();
    let (mut configs, _, discovery_set) = gen_configs(5);

    // server setup

    let server_config = configs.swap_remove(0);
    let server_peer_id = server_config.validator_network.as_ref().unwrap().peer_id;
    let server_role = server_config.base.role;

    let (_server_storage_runtime, server_storage_read_client, _server_executor) =
        setup_storage_service_and_executor(&server_config);

    let (
        f_server_onchain_discovery,
        mut server_network_tx,
        server_peer_query_ticker_tx,
        server_storage_query_ticker_tx,
        _server_peer_mgr_reqs_rx,
        _server_conn_mgr_reqs_rx,
    ) = setup_onchain_discovery(
        rt.handle().clone(),
        server_peer_id,
        server_role,
        server_storage_read_client,
    );

    // client setup

    let client_config = configs.swap_remove(0);
    let client_peer_id = client_config.validator_network.as_ref().unwrap().peer_id;
    let client_role = client_config.base.role;

    let (_storage_runtime, storage_read_client, _executor) =
        setup_storage_service_and_executor(&client_config);

    let (
        f_client_onchain_discovery,
        mut client_network_tx,
        mut client_peer_query_ticker_tx,
        client_storage_query_ticker_tx,
        mut client_peer_mgr_reqs_rx,
        client_conn_mgr_reqs_rx,
    ) = setup_onchain_discovery(
        rt.handle().clone(),
        client_peer_id,
        client_role,
        storage_read_client,
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
    let discovery_set = DiscoverySetInternal::from_discovery_set(client_role, discovery_set);
    let expected_update_reqs = discovery_set
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
            ConnectivityRequest::UpdateAddresses(peer_id, addrs) => (peer_id, addrs),
            _ => panic!(
                "Unexpected ConnectivityRequest, expected UpdateAddresses: {:?}",
                req
            ),
        })
        .collect::<HashMap<_, _>>();

    assert_eq!(expected_update_reqs, update_reqs);

    // client should shutdown completely
    rt.block_on(f_client_onchain_discovery).unwrap();

    // server shutdown
    drop(server_network_tx);
    drop(server_peer_query_ticker_tx);
    drop(server_storage_query_ticker_tx);

    rt.block_on(f_server_onchain_discovery).unwrap();
}
