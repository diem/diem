// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    network::{StateSynchronizerEvents, StateSynchronizerSender},
    tests::{
        helpers::{MockExecutorProxy, MockRpcHandler, SynchronizerEnvHelper},
        mock_storage::MockStorage,
    },
    StateSyncClient, StateSynchronizer,
};
use anyhow::{bail, Result};
use channel::{libra_channel, message_queues::QueueStyle};
use futures::{executor::block_on, future::FutureExt, StreamExt};
use libra_config::{
    config::{GossipConfig, RoleType},
    network_id::{NetworkContext, NetworkId, NodeNetworkId},
};
use libra_crypto::x25519;
use libra_mempool::mocks::MockSharedMempool;
use libra_network_address::NetworkAddress;
use libra_types::{
    chain_id::ChainId, ledger_info::LedgerInfoWithSignatures, on_chain_config::ValidatorSet,
    transaction::TransactionListWithProof, validator_info::ValidatorInfo,
    validator_signer::ValidatorSigner, validator_verifier::random_validator_verifier,
    waypoint::Waypoint, PeerId,
};
use netcore::transport::{ConnectionOrigin, ConnectionOrigin::*};
use network::{
    constants,
    peer_manager::{
        builder::AuthenticationMode, conn_notifs_channel, ConnectionNotification,
        ConnectionRequestSender, PeerManagerNotification, PeerManagerRequest,
        PeerManagerRequestSender,
    },
    protocols::network::{NewNetworkEvents, NewNetworkSender},
    DisconnectReason, ProtocolId,
};
use network_builder::builder::NetworkBuilder;
use std::{
    collections::{HashMap, HashSet},
    num::NonZeroUsize,
    ops::DerefMut,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, RwLock,
    },
};
use tokio::runtime::Runtime;

struct SynchronizerEnv {
    runtime: Runtime,
    synchronizers: Vec<StateSynchronizer>,
    clients: Vec<Arc<StateSyncClient>>,
    storage_proxies: Vec<Arc<RwLock<MockStorage>>>, // to directly modify peers storage
    signers: Vec<ValidatorSigner>,
    network_keys: Vec<x25519::PrivateKey>,
    public_keys: Vec<ValidatorInfo>,
    network_id: NetworkId,
    peer_ids: Vec<PeerId>,
    peer_addresses: Vec<NetworkAddress>,
    mempools: Vec<MockSharedMempool>,
    network_reqs_rxs:
        HashMap<PeerId, libra_channel::Receiver<(PeerId, ProtocolId), PeerManagerRequest>>,
    network_notifs_txs:
        HashMap<PeerId, libra_channel::Sender<(PeerId, ProtocolId), PeerManagerNotification>>,
    network_conn_event_notifs_txs: HashMap<PeerId, conn_notifs_channel::Sender>,
    multi_peer_ids: Vec<Vec<PeerId>>, // maps peer's synchronizer env index to that peer's PeerIds, to support node with multiple network IDs
}

impl SynchronizerEnv {
    // Moves peer 0 to the next epoch. Note that other peers are not going to be able to discover
    // their new signers: they're going to learn about the new epoch public key through state
    // synchronization, but private keys are discovered separately.
    pub fn move_to_next_epoch(&self) {
        let num_peers = self.public_keys.len();
        let (signers, _verifier) = random_validator_verifier(num_peers, None, true);
        let new_keys = self
            .public_keys
            .iter()
            .enumerate()
            .map(|(idx, validator_keys)| {
                ValidatorInfo::new(
                    signers[idx].author(),
                    validator_keys.consensus_voting_power(),
                    validator_keys.config().clone(),
                )
            })
            .collect::<Vec<ValidatorInfo>>();
        let validator_set = ValidatorSet::new(new_keys);
        self.storage_proxies[0]
            .write()
            .unwrap()
            .move_to_next_epoch(signers[0].clone(), validator_set);
    }

    fn new(num_peers: usize) -> Self {
        ::libra_logger::Logger::init_for_testing();
        let runtime = Runtime::new().unwrap();
        let (signers, public_keys, network_keys) = SynchronizerEnvHelper::initial_setup(num_peers);
        let peer_ids = signers.iter().map(|s| s.author()).collect::<Vec<PeerId>>();

        Self {
            runtime,
            synchronizers: vec![],
            clients: vec![],
            storage_proxies: vec![],
            signers,
            network_id: NetworkId::Validator,
            network_keys,
            public_keys,
            peer_ids,
            peer_addresses: vec![],
            mempools: vec![],
            network_reqs_rxs: HashMap::new(),
            network_notifs_txs: HashMap::new(),
            network_conn_event_notifs_txs: HashMap::new(),
            multi_peer_ids: vec![],
        }
    }

    fn start_next_synchronizer(
        &mut self,
        handler: MockRpcHandler,
        role: RoleType,
        waypoint: Waypoint,
        mock_network: bool,
        upstream_networks: Option<Vec<NetworkId>>,
    ) {
        self.setup_next_synchronizer(
            handler,
            role,
            waypoint,
            60_000,
            120_000,
            mock_network,
            upstream_networks,
        );
    }

    fn setup_next_synchronizer(
        &mut self,
        handler: MockRpcHandler,
        role: RoleType,
        waypoint: Waypoint,
        timeout_ms: u64,
        multicast_timeout_ms: u64,
        mock_network: bool,
        upstream_networks: Option<Vec<NetworkId>>,
    ) {
        let new_peer_idx = self.synchronizers.len();
        let seed_pubkeys: HashMap<_, _> = self
            .public_keys
            .iter()
            .map(|public_keys| {
                let peer_id = *public_keys.account_address();
                let peer_idx = self
                    .peer_ids
                    .iter()
                    .position(|pid| pid == &peer_id)
                    .unwrap();
                let pubkey = self.network_keys[peer_idx].public_key();
                let pubkey_set: HashSet<_> = [pubkey].iter().copied().collect();
                (peer_id, pubkey_set)
            })
            .collect();
        let trusted_peers = Arc::new(RwLock::new(HashMap::new()));

        // set up config
        let mut config = config_builder::test_config().0;
        config.base.role = role;
        config.state_sync.sync_request_timeout_ms = timeout_ms;
        config.state_sync.multicast_timeout_ms = multicast_timeout_ms;

        let network = config.validator_network.unwrap();
        let network_id = if role.is_validator() {
            NetworkId::Validator
        } else {
            NetworkId::vfn_network()
        };
        if !role.is_validator() {
            config.full_node_networks = vec![network];
            config.validator_network = None;
            // setup upstream network for FN
            if let Some(upstream_networks) = &upstream_networks {
                config.upstream.networks = upstream_networks.clone();
            } else if new_peer_idx > 0 {
                config.upstream.networks.push(network_id.clone());
            }
        }

        // setup network
        let mut network_handles = vec![];
        if mock_network {
            let networks = if role.is_validator() {
                vec![NetworkId::Validator, NetworkId::vfn_network()]
            } else {
                vec![
                    NetworkId::vfn_network(),
                    NetworkId::Private("second".to_string()),
                    NetworkId::Public,
                ]
            };
            let mut network_ids = vec![];
            for (idx, network) in networks.into_iter().enumerate() {
                let peer_id = PeerId::random();
                network_ids.push(peer_id);

                // mock the StateSynchronizerEvents and StateSynchronizerSender to allow manually controlling
                // msg delivery in test
                let (network_reqs_tx, network_reqs_rx) =
                    libra_channel::new(QueueStyle::LIFO, NonZeroUsize::new(1).unwrap(), None);
                let (connection_reqs_tx, _) =
                    libra_channel::new(QueueStyle::LIFO, NonZeroUsize::new(1).unwrap(), None);
                let (network_notifs_tx, network_notifs_rx) =
                    libra_channel::new(QueueStyle::LIFO, NonZeroUsize::new(1).unwrap(), None);
                let (conn_status_tx, conn_status_rx) = conn_notifs_channel::new();
                let network_sender = StateSynchronizerSender::new(
                    PeerManagerRequestSender::new(network_reqs_tx),
                    ConnectionRequestSender::new(connection_reqs_tx),
                );
                let network_events =
                    StateSynchronizerEvents::new(network_notifs_rx, conn_status_rx);
                self.network_reqs_rxs.insert(peer_id, network_reqs_rx);
                self.network_notifs_txs.insert(peer_id, network_notifs_tx);
                self.network_conn_event_notifs_txs
                    .insert(peer_id, conn_status_tx);

                network_handles.push((
                    NodeNetworkId::new(network, idx),
                    network_sender,
                    network_events,
                ));
            }

            self.multi_peer_ids.push(network_ids);
        } else {
            let addr: NetworkAddress = "/memory/0".parse().unwrap();
            let mut seed_addrs = HashMap::new();
            if new_peer_idx > 0 {
                seed_addrs.insert(
                    self.peer_ids[new_peer_idx - 1],
                    vec![self.peer_addresses[new_peer_idx - 1].clone()],
                );
            }
            let key = self.network_keys[new_peer_idx].clone();
            let pub_key = key.public_key();
            let authentication_mode = AuthenticationMode::Mutual(key);
            let network_context = Arc::new(NetworkContext::new(
                self.network_id.clone(),
                RoleType::Validator,
                self.peer_ids[new_peer_idx],
            ));
            let mut network_builder = NetworkBuilder::new(
                ChainId::default(),
                trusted_peers.clone(),
                network_context,
                addr.clone(),
                authentication_mode,
                constants::MAX_FRAME_SIZE,
            );
            network_builder
                .add_connectivity_manager(
                    seed_addrs,
                    seed_pubkeys,
                    trusted_peers,
                    constants::MAX_FULLNODE_CONNECTIONS,
                    constants::MAX_CONNECTION_DELAY_MS,
                    constants::CONNECTIVITY_CHECK_INTERNAL_MS,
                    constants::NETWORK_CHANNEL_SIZE,
                )
                .add_gossip_discovery(
                    GossipConfig {
                        advertised_address: addr,
                        discovery_interval_ms: constants::DISCOVERY_INTERVAL_MS,
                    },
                    pub_key,
                );

            let (sender, events) =
                network_builder.add_protocol_handler(crate::network::network_endpoint_config());
            network_builder.build(self.runtime.handle().clone()).start();
            let peer_addr = network_builder.listen_address();
            self.peer_addresses.push(peer_addr);
            network_handles.push((NodeNetworkId::new(network_id, 0), sender, events));
        };

        let genesis_li = SynchronizerEnvHelper::genesis_li(&self.public_keys);
        let storage_proxy = Arc::new(RwLock::new(MockStorage::new(
            genesis_li,
            self.signers[new_peer_idx].clone(),
        )));
        let (mempool_channel, mempool_requests) = futures::channel::mpsc::channel(1_024);
        let synchronizer = StateSynchronizer::bootstrap_with_executor_proxy(
            Runtime::new().unwrap(),
            network_handles,
            mempool_channel,
            role,
            waypoint,
            &config.state_sync,
            config.upstream,
            MockExecutorProxy::new(handler, storage_proxy.clone()),
        );
        self.mempools
            .push(MockSharedMempool::new(Some(mempool_requests)));
        let client = synchronizer.create_client();
        self.synchronizers.push(synchronizer);
        self.clients.push(client);
        self.storage_proxies.push(storage_proxy);
    }

    fn default_handler() -> MockRpcHandler {
        Box::new(|resp| -> Result<TransactionListWithProof> { Ok(resp) })
    }

    fn sync_to(&self, peer_id: usize, target: LedgerInfoWithSignatures) {
        block_on(self.clients[peer_id].sync_to(target)).unwrap()
    }

    // commit new txns up to the given version
    fn commit(&self, peer_id: usize, version: u64) {
        let mut storage = self.storage_proxies[peer_id].write().unwrap();
        let num_txns = version - storage.version();
        assert!(num_txns > 0);
        let (committed_txns, signed_txns) = storage.commit_new_txns(num_txns);
        drop(storage);
        // add txns to mempool
        assert!(self.mempools[peer_id].add_txns(signed_txns.clone()).is_ok());

        // we need to run StateSyncClient::commit on a tokio runtime to support tokio::timeout
        // in commit()
        assert!(Runtime::new()
            .unwrap()
            .block_on(self.clients[peer_id].commit(committed_txns, vec![]))
            .is_ok());
        let mempool_txns = self.mempools[peer_id].read_timeline(0, signed_txns.len());
        for txn in signed_txns.iter() {
            assert!(!mempool_txns.contains(txn));
        }
    }

    fn latest_li(&self, peer_id: usize) -> LedgerInfoWithSignatures {
        self.storage_proxies[peer_id]
            .read()
            .unwrap()
            .highest_local_li()
    }

    // Find LedgerInfo for a epoch boundary version
    fn get_epoch_ending_ledger_info(
        &self,
        peer_id: usize,
        version: u64,
    ) -> Result<LedgerInfoWithSignatures> {
        self.storage_proxies[peer_id]
            .read()
            .unwrap()
            .get_epoch_ending_ledger_info(version)
    }

    fn wait_for_version(
        &self,
        peer_id: usize,
        target_version: u64,
        highest_li_version: Option<u64>,
    ) -> bool {
        let max_retries = 30;
        for _ in 0..max_retries {
            let state = block_on(self.clients[peer_id].get_state()).unwrap();
            if state.synced_trees.version().unwrap_or(0) == target_version {
                return highest_li_version.map_or(true, |li_version| {
                    li_version == state.highest_local_li.ledger_info().version()
                });
            }
            std::thread::sleep(std::time::Duration::from_millis(1000));
        }
        false
    }

    fn wait_until_initialized(&self, peer_id: usize) -> Result<()> {
        block_on(self.synchronizers[peer_id].wait_until_initialized())
    }

    fn send_connection_event(
        &mut self,
        receiver: (usize, usize),
        sender: PeerId,
        notif: ConnectionNotification,
    ) {
        let receiver_id = self.get_peer_network_id(receiver);
        let conn_notifs_tx = self
            .network_conn_event_notifs_txs
            .get_mut(&receiver_id)
            .unwrap();
        conn_notifs_tx.push(sender, notif).unwrap();
    }

    /// Delivers next message from peer with index `sender` in this SynchronizerEnv
    /// Returns the recipient of the msg
    fn deliver_msg(&mut self, sender: (usize, usize)) -> PeerId {
        let sender_id = self.get_peer_network_id(sender);
        let network_reqs_rx = self.network_reqs_rxs.get_mut(&sender_id).unwrap();
        let network_req = block_on(network_reqs_rx.next()).unwrap();

        // await next message from node
        if let PeerManagerRequest::SendMessage(receiver_id, msg) = network_req {
            let receiver_network_notif_tx = self.network_notifs_txs.get_mut(&receiver_id).unwrap();
            receiver_network_notif_tx
                .push(
                    (sender_id, ProtocolId::StateSynchronizerDirectSend),
                    PeerManagerNotification::RecvMessage(sender_id, msg),
                )
                .unwrap();
            receiver_id
        } else {
            panic!("received network request other than PeerManagerRequest");
        }
    }

    // checks that the `env_idx`th peer in this env sends no message to its `network_idx`th network
    fn assert_no_message_sent(&mut self, sender: (usize, usize)) {
        let peer_id = self.get_peer_network_id(sender);
        let network_reqs_rx = self.network_reqs_rxs.get_mut(&peer_id).unwrap();
        assert!(network_reqs_rx.select_next_some().now_or_never().is_none());
    }

    fn get_peer_network_id(&mut self, peer: (usize, usize)) -> PeerId {
        *self
            .multi_peer_ids
            .get(peer.0)
            .expect("env idx out of range for peer")
            .get(peer.1)
            .expect("network idx out of range for peer")
    }

    fn get_env_idx(&self, peer_id: &PeerId) -> usize {
        for (idx, peer_ids) in self.multi_peer_ids.iter().enumerate() {
            if peer_ids.contains(peer_id) {
                return idx;
            }
        }
        panic!("could not find env index for peer");
    }

    fn clone_storage(&mut self, from_idx: usize, to_idx: usize) {
        let storage_0_lock = self.storage_proxies[from_idx].read().unwrap();
        let storage_0 = storage_0_lock;
        let mut storage_1_lock = self.storage_proxies[to_idx].write().unwrap();
        let storage_1 = storage_1_lock.deref_mut();
        *storage_1 = storage_0.clone();
    }

    fn send_peer_event(
        &mut self,
        sender: (usize, usize),
        receiver: (usize, usize),
        new_peer: bool,
        direction: ConnectionOrigin,
    ) {
        let sender_id = self.get_peer_network_id(sender);
        let notif = if new_peer {
            ConnectionNotification::NewPeer(
                sender_id,
                NetworkAddress::mock(),
                direction,
                NetworkContext::mock(),
            )
        } else {
            ConnectionNotification::LostPeer(
                sender_id,
                NetworkAddress::mock(),
                direction,
                DisconnectReason::ConnectionLost,
            )
        };

        self.send_connection_event(receiver, sender_id, notif);
    }
}

#[test]
fn test_basic_catch_up() {
    let mut env = SynchronizerEnv::new(2);
    env.start_next_synchronizer(
        SynchronizerEnv::default_handler(),
        RoleType::Validator,
        Waypoint::default(),
        false,
        None,
    );
    env.start_next_synchronizer(
        SynchronizerEnv::default_handler(),
        RoleType::Validator,
        Waypoint::default(),
        false,
        None,
    );

    // test small sequential syncs
    for version in 1..5 {
        env.commit(0, version);
        let target_li = env.latest_li(0);
        env.sync_to(1, target_li);
        assert_eq!(env.latest_li(1).ledger_info().version(), version);
    }
    // test batch sync for multiple transactions
    env.commit(0, 20);
    env.sync_to(1, env.latest_li(0));
    assert_eq!(env.latest_li(1).ledger_info().version(), 20);

    // test batch sync for multiple chunks
    env.commit(0, 2000);
    env.sync_to(1, env.latest_li(0));
    assert_eq!(env.latest_li(1).ledger_info().version(), 2000);
}

#[test]
fn test_flaky_peer_sync() {
    // create handler that causes error, but has successful retries
    let attempt = AtomicUsize::new(0);
    let handler = Box::new(move |resp| -> Result<TransactionListWithProof> {
        let fail_request = attempt.load(Ordering::Relaxed) == 0;
        attempt.fetch_add(1, Ordering::Relaxed);
        if fail_request {
            bail!("chunk fetch failed")
        } else {
            Ok(resp)
        }
    });
    let mut env = SynchronizerEnv::new(2);
    env.start_next_synchronizer(
        SynchronizerEnv::default_handler(),
        RoleType::Validator,
        Waypoint::default(),
        false,
        None,
    );
    env.start_next_synchronizer(
        handler,
        RoleType::Validator,
        Waypoint::default(),
        false,
        None,
    );
    env.commit(0, 20);
    env.sync_to(1, env.latest_li(0));
    assert_eq!(env.latest_li(1).ledger_info().version(), 20);
}

#[test]
#[should_panic]
fn test_request_timeout() {
    let handler =
        Box::new(move |_| -> Result<TransactionListWithProof> { bail!("chunk fetch failed") });
    let mut env = SynchronizerEnv::new(2);
    env.start_next_synchronizer(
        handler,
        RoleType::Validator,
        Waypoint::default(),
        false,
        None,
    );
    env.setup_next_synchronizer(
        SynchronizerEnv::default_handler(),
        RoleType::Validator,
        Waypoint::default(),
        100,
        300,
        false,
        None,
    );
    env.commit(0, 1);
    env.sync_to(1, env.latest_li(0));
}

#[test]
fn test_full_node() {
    let mut env = SynchronizerEnv::new(2);
    env.start_next_synchronizer(
        SynchronizerEnv::default_handler(),
        RoleType::Validator,
        Waypoint::default(),
        false,
        None,
    );
    env.start_next_synchronizer(
        SynchronizerEnv::default_handler(),
        RoleType::FullNode,
        Waypoint::default(),
        false,
        None,
    );
    env.commit(0, 10);
    // first sync should be fulfilled immediately after peer discovery
    assert!(env.wait_for_version(1, 10, None));
    env.commit(0, 20);
    // second sync will be done via long polling cause first node should send new request
    // after receiving first chunk immediately
    assert!(env.wait_for_version(1, 20, None));
}

#[test]
fn catch_up_through_epochs_validators() {
    let mut env = SynchronizerEnv::new(2);
    env.start_next_synchronizer(
        SynchronizerEnv::default_handler(),
        RoleType::Validator,
        Waypoint::default(),
        false,
        None,
    );
    env.start_next_synchronizer(
        SynchronizerEnv::default_handler(),
        RoleType::Validator,
        Waypoint::default(),
        false,
        None,
    );

    // catch up to the next epoch starting from the middle of the current one
    env.commit(0, 20);
    env.sync_to(1, env.latest_li(0));
    env.commit(0, 40);
    env.move_to_next_epoch();
    env.commit(0, 100);
    env.sync_to(1, env.latest_li(0));
    assert_eq!(env.latest_li(1).ledger_info().version(), 100);
    assert_eq!(env.latest_li(1).ledger_info().epoch(), 2);

    // catch up through multiple epochs
    for epoch in 2..10 {
        env.commit(0, epoch * 100);
        env.move_to_next_epoch();
    }
    env.commit(0, 950); // At this point peer 0 is at epoch 10 and version 950
    env.sync_to(1, env.latest_li(0));
    assert_eq!(env.latest_li(1).ledger_info().version(), 950);
    assert_eq!(env.latest_li(1).ledger_info().epoch(), 10);
}

#[test]
fn catch_up_through_epochs_full_node() {
    let mut env = SynchronizerEnv::new(3);
    env.start_next_synchronizer(
        SynchronizerEnv::default_handler(),
        RoleType::Validator,
        Waypoint::default(),
        false,
        None,
    );
    // catch up through multiple epochs
    for epoch in 1..10 {
        env.commit(0, epoch * 100);
        env.move_to_next_epoch();
    }
    env.commit(0, 950); // At this point peer 0 is at epoch 10 and version 950

    env.start_next_synchronizer(
        SynchronizerEnv::default_handler(),
        RoleType::FullNode,
        Waypoint::default(),
        false,
        None,
    );
    assert!(env.wait_for_version(1, 950, None));
    assert_eq!(env.latest_li(1).ledger_info().epoch(), 10);

    // Peer 2 has peer 1 as its upstream, should catch up from it.
    env.start_next_synchronizer(
        SynchronizerEnv::default_handler(),
        RoleType::FullNode,
        Waypoint::default(),
        false,
        None,
    );
    assert!(env.wait_for_version(2, 950, None));
    assert_eq!(env.latest_li(2).ledger_info().epoch(), 10);
}

#[test]
fn catch_up_with_waypoints() {
    let mut env = SynchronizerEnv::new(3);
    env.start_next_synchronizer(
        SynchronizerEnv::default_handler(),
        RoleType::Validator,
        Waypoint::default(),
        false,
        None,
    );
    for epoch in 1..10 {
        env.commit(0, epoch * 100);
        env.move_to_next_epoch();
    }
    env.commit(0, 950); // At this point peer 0 is at epoch 10 and version 950

    // Create a waypoint based on LedgerInfo of peer 0 at version 700 (epoch 7)
    let waypoint_li = env.get_epoch_ending_ledger_info(0, 700).unwrap();
    let waypoint = Waypoint::new_epoch_boundary(waypoint_li.ledger_info()).unwrap();

    env.start_next_synchronizer(
        SynchronizerEnv::default_handler(),
        RoleType::FullNode,
        waypoint,
        false,
        None,
    );
    env.wait_until_initialized(1).unwrap();
    assert!(env.latest_li(1).ledger_info().version() >= 700);
    assert!(env.latest_li(1).ledger_info().epoch() >= 7);

    // Once caught up with the waypoint peer 1 continues with the regular state sync
    assert!(env.wait_for_version(1, 950, None));
    assert_eq!(env.latest_li(1).ledger_info().epoch(), 10);

    // Peer 2 has peer 1 as its upstream, should catch up from it.
    env.start_next_synchronizer(
        SynchronizerEnv::default_handler(),
        RoleType::FullNode,
        Waypoint::default(),
        false,
        None,
    );
    assert!(env.wait_for_version(2, 950, None));
    assert_eq!(env.latest_li(2).ledger_info().epoch(), 10);
}

// test full node catching up to validator that is also making progress
#[test]
fn test_sync_pending_ledger_infos() {
    let mut env = SynchronizerEnv::new(2);
    env.start_next_synchronizer(
        SynchronizerEnv::default_handler(),
        RoleType::Validator,
        Waypoint::default(),
        true,
        None,
    );
    env.start_next_synchronizer(
        SynchronizerEnv::default_handler(),
        RoleType::FullNode,
        Waypoint::default(),
        true,
        None,
    );

    let validator = (0, 0);
    let full_node = (1, 0);

    // validator discovers fn
    env.send_peer_event(full_node, validator, true, Inbound);

    // fn discovers validator
    env.send_peer_event(validator, full_node, true, Outbound);

    let commit_versions = vec![
        900, 1800, 2800, 3100, 3200, 3300, 3325, 3350, 3400, 3450, 3650, 4300,
    ];

    let expected_states = vec![
        (250, 0),
        (500, 0),
        (750, 0),
        (900, 900),
        (1150, 900),
        (1400, 900),
        (1650, 900),
        (1800, 1800),
        (2050, 1800),
        (2300, 1800),
        (2550, 1800),
        (2800, 2800),
        (3050, 2800),
        (3100, 3100),
        (3350, 3350),
        (3450, 3450),
        (3650, 3650),
        (3900, 3650),
        (4150, 3650),
        (4300, 4300),
    ];

    fn deliver_and_check_chunk_state(env: &mut SynchronizerEnv, expected_state: (u64, u64)) {
        env.deliver_msg((1, 0));
        env.deliver_msg((0, 0));
        let (sync_version, li_version) = expected_state;
        assert!(
            env.wait_for_version(1, sync_version, Some(li_version)),
            "didn't reach synced version {} and highest LI version {}",
            sync_version,
            li_version
        );
    }

    for (idx, expected_state) in expected_states.iter().enumerate() {
        // commit if applicable
        if let Some(version) = commit_versions.get(idx) {
            env.commit(0, *version);
        }
        deliver_and_check_chunk_state(&mut env, *expected_state);
    }
}

#[test]
#[ignore] // TODO: https://github.com/libra/libra/issues/5771
fn test_fn_failover() {
    let mut env = SynchronizerEnv::new(5);
    env.start_next_synchronizer(
        SynchronizerEnv::default_handler(),
        RoleType::Validator,
        Waypoint::default(),
        true,
        None,
    );
    env.setup_next_synchronizer(
        SynchronizerEnv::default_handler(),
        RoleType::FullNode,
        Waypoint::default(),
        1_000,
        60_000,
        true,
        Some(vec![NetworkId::vfn_network(), NetworkId::Public]),
    );

    // start up 3 publicly available VFN
    for _ in 0..3 {
        env.start_next_synchronizer(
            SynchronizerEnv::default_handler(),
            RoleType::FullNode,
            Waypoint::default(),
            true,
            None,
        );
    }

    // connect everyone
    let validator = (0, 1);
    let fn_0_vfn = (1, 0);
    let fn_0_public = (1, 2);
    let fn_1 = (2, 2);
    let fn_2 = (3, 2);
    let fn_3 = (4, 2);

    // vfn network:
    // validator discovers fn_0
    env.send_peer_event(fn_0_vfn, validator, true, Inbound);
    // fn_0 discovers validator
    env.send_peer_event(validator, fn_0_vfn, true, Outbound);

    // public network:
    // fn_0 sends new peer event to all its upstream public peers
    let upstream_peers = [fn_1, fn_2, fn_3];
    for peer in upstream_peers.iter() {
        env.send_peer_event(fn_0_public, *peer, true, Inbound);
        env.send_peer_event(*peer, fn_0_public, true, Outbound);
    }

    // commit some txns on v
    // check that fn_0 sends chunk requests to v only
    for num_commit in 1..=5 {
        env.commit(0, num_commit * 5);
        for public_upstream in 2..=4 {
            // we just directly sync up the storage of all the upstream peers of fn_0
            // for ease of testing
            env.clone_storage(0, public_upstream);
        }
        // deliver fn_0's chunk request
        let recipient = env.deliver_msg(fn_0_vfn);
        assert_eq!(recipient, env.get_peer_network_id(validator));
        env.assert_no_message_sent(fn_0_public);
        // deliver validator's chunk response
        if num_commit < 5 {
            env.deliver_msg(validator);
        }
    }

    // bring down v
    env.send_peer_event(fn_0_vfn, validator, false, Inbound);
    env.send_peer_event(validator, fn_0_vfn, false, Outbound);

    // deliver chunk response to fn_0 after the lost peer event
    // so that the next chunk request is guaranteed to be sent after the lost peer event
    env.deliver_msg(validator);

    let upstream_peer_ids: HashSet<_> = upstream_peers
        .iter()
        .map(|peer| env.get_peer_network_id(*peer))
        .collect();
    // check that vfn sends chunk requests to the failover FNs only
    let mut last_fallback_recipient = None;
    for num_commit in 6..=10 {
        env.commit(0, num_commit * 5);
        for public_upstream in 2..=4 {
            env.clone_storage(0, public_upstream);
        }
        // deliver fn_0's chunk request
        let recipient = env.deliver_msg(fn_0_public);
        assert!(upstream_peer_ids.contains(&recipient));
        env.assert_no_message_sent(fn_0_vfn);
        // deliver validator's chunk response
        if num_commit < 10 {
            let chunk_response_recipient = env.deliver_msg((env.get_env_idx(&recipient), 2));
            assert_eq!(
                chunk_response_recipient,
                env.get_peer_network_id(fn_0_public)
            );
        } else {
            last_fallback_recipient = Some(recipient);
        }
    }

    // bring down two public fallback
    // disconnect fn_1 and fn_0
    env.send_peer_event(fn_0_public, fn_1, false, Inbound);
    env.send_peer_event(fn_1, fn_0_public, false, Outbound);

    // disconnect fn_2 and fn_0
    env.send_peer_event(fn_0_public, fn_2, false, Inbound);
    env.send_peer_event(fn_2, fn_0_public, false, Outbound);

    // deliver chunk response to fn_0 after the lost peer events
    // so that the next chunk request is guaranteed to be sent after the lost peer events
    let chunk_response_recipient =
        env.deliver_msg((env.get_env_idx(&last_fallback_recipient.unwrap()), 2));
    assert_eq!(
        chunk_response_recipient,
        env.get_peer_network_id(fn_0_public)
    );

    // check we only broadcast to the single live fallback peer (fn_3)
    for num_commit in 11..=15 {
        env.commit(0, num_commit * 5);
        for public_upstream in 2..=4 {
            env.clone_storage(0, public_upstream);
        }
        // deliver fn_0's chunk request
        let recipient = env.deliver_msg(fn_0_public);
        assert_eq!(recipient, env.get_peer_network_id(fn_3));
        env.assert_no_message_sent(fn_0_vfn);
        // deliver validator's chunk response
        if num_commit < 15 {
            let chunk_response_recipient = env.deliver_msg(fn_3);
            assert_eq!(
                chunk_response_recipient,
                env.get_peer_network_id(fn_0_public)
            );
        }
    }

    // bring down everyone
    // disconnect fn_3 and fn_0
    env.send_peer_event(fn_3, fn_0_public, false, Outbound);
    env.send_peer_event(fn_0_public, fn_3, false, Inbound);

    // deliver chunk response to fn_0 after the lost peer events
    // so that the next chunk request is guaranteed to be sent after the lost peer events
    let chunk_response_recipient = env.deliver_msg(fn_3);
    assert_eq!(
        chunk_response_recipient,
        env.get_peer_network_id(fn_0_public)
    );

    // check no sync requests are sent (all upstream are down)
    env.assert_no_message_sent(fn_0_vfn);
    env.assert_no_message_sent(fn_0_public);

    // bring back one fallback (fn_2)
    env.send_peer_event(fn_2, fn_0_public, true, Outbound);
    env.send_peer_event(fn_0_public, fn_2, true, Inbound);

    // check we only broadcast to the single live fallback peer (fn_2)
    for num_commit in 16..=20 {
        env.commit(0, num_commit * 5);
        for public_upstream in 2..=4 {
            env.clone_storage(0, public_upstream);
        }
        // deliver fn_0's chunk request
        let recipient = env.deliver_msg(fn_0_public);
        assert_eq!(recipient, env.get_peer_network_id(fn_2));
        env.assert_no_message_sent(fn_0_vfn);
        // deliver validator's chunk response
        if num_commit < 20 {
            let chunk_response_recipient = env.deliver_msg(fn_2);
            assert_eq!(
                chunk_response_recipient,
                env.get_peer_network_id(fn_0_public)
            );
        }
    }

    // bring back v again
    env.send_peer_event(fn_0_vfn, validator, true, Inbound);
    env.send_peer_event(validator, fn_0_vfn, true, Outbound);

    let chunk_response_recipient = env.deliver_msg(fn_2);
    assert_eq!(
        chunk_response_recipient,
        env.get_peer_network_id(fn_0_public)
    );

    // check that vfn sends chunk requests to v only, not fallback upstream
    for num_commit in 21..=25 {
        env.commit(0, num_commit * 5);
        for public_upstream in 2..=4 {
            env.clone_storage(0, public_upstream);
        }
        // deliver fn_0's chunk request
        let recipient = env.deliver_msg(fn_0_vfn);
        assert_eq!(recipient, env.get_peer_network_id(validator));
        env.assert_no_message_sent(fn_0_public);
        if num_commit < 25 {
            // deliver validator's chunk response
            env.deliver_msg(validator);
        }
    }

    // bring back all fallback
    let upstream_peers_to_revive = [(2, 1), (4, 1)];
    for peer in upstream_peers_to_revive.iter() {
        env.send_peer_event(fn_0_public, *peer, true, Inbound);
        env.send_peer_event(*peer, fn_0_public, true, Outbound);
    }

    // deliver validator's chunk response after fallback peers are revived
    env.deliver_msg(validator);

    // check that we only broadcast to v
    // check that vfn sends chunk requests to v only, not fallback upstream
    for num_commit in 26..=30 {
        env.commit(0, num_commit * 5);
        for public_upstream in 2..=4 {
            env.clone_storage(0, public_upstream);
        }
        // deliver fn_0's chunk request
        let recipient = env.deliver_msg(fn_0_vfn);
        assert_eq!(recipient, env.get_peer_network_id(validator));
        env.assert_no_message_sent(fn_0_public);
        // deliver validator's chunk response
        env.deliver_msg(validator);
    }
}

#[test]
#[ignore]
fn test_multicast_failover() {
    let mut env = SynchronizerEnv::new(4);
    env.start_next_synchronizer(
        SynchronizerEnv::default_handler(),
        RoleType::Validator,
        Waypoint::default(),
        true,
        None,
    );

    // set up node with more than 2 upstream networks, which is more than in standard prod setting
    // just to be safe
    let multicast_timeout_ms = 5_000;
    env.setup_next_synchronizer(
        SynchronizerEnv::default_handler(),
        RoleType::FullNode,
        Waypoint::default(),
        1_000,
        multicast_timeout_ms,
        true,
        Some(vec![
            NetworkId::vfn_network(),
            NetworkId::Private("second".to_string()),
            NetworkId::Public,
        ]),
    );

    // setup the other FN upstream peer
    for _ in 0..2 {
        env.start_next_synchronizer(
            SynchronizerEnv::default_handler(),
            RoleType::FullNode,
            Waypoint::default(),
            true,
            None,
        );
    }

    // connect everyone
    let validator = (0, 1);
    let fn_0_vfn = (1, 0);
    let fn_0_second = (1, 1);
    let fn_0_public = (1, 2);
    let fn_1 = (2, 1);
    let fn_2 = (3, 2);

    // vfn network:
    // validator discovers fn_0
    env.send_peer_event(fn_0_vfn, validator, true, Inbound);
    // fn_0 discovers validator
    env.send_peer_event(validator, fn_0_vfn, true, Outbound);

    // second private network: fn_1 is upstream to fn_0
    // fn_1 discovers fn_0
    env.send_peer_event(fn_0_second, fn_1, true, Inbound);
    // fn_0 discovers fn_1
    env.send_peer_event(fn_1, fn_0_second, true, Outbound);

    // public network: fn_2 is upstream to fn_1
    // fn_2 discovers fn_0
    env.send_peer_event(fn_0_public, fn_2, true, Inbound);
    // fn_0 discovers fn_2
    env.send_peer_event(fn_2, fn_0_public, true, Outbound);

    for num_commit in 1..=3 {
        env.commit(0, num_commit * 5);
        // deliver fn_0's chunk request
        let recipient = env.deliver_msg(fn_0_vfn);
        assert_eq!(recipient, env.get_peer_network_id(validator));
        env.assert_no_message_sent(fn_0_second);
        env.assert_no_message_sent(fn_0_public);
        // deliver validator's chunk response
        if num_commit < 3 {
            env.deliver_msg(validator);
        }
    }

    // we don't deliver the validator's last chunk response
    // wait for fn_0's chunk request to time out
    std::thread::sleep(std::time::Duration::from_millis(multicast_timeout_ms));

    // commit some with
    for num_commit in 4..=7 {
        env.commit(0, num_commit * 5);
        for fn_0_upstream in 2..3 {
            env.clone_storage(0, fn_0_upstream);
            env.wait_for_version(fn_0_upstream, num_commit * 5, None);
        }

        // check that fn_0 sends chunk requests to both primary (vfn) and fallback ("second") network
        let primary = env.deliver_msg(fn_0_vfn);
        assert_eq!(primary, env.get_peer_network_id(validator));
        let secondary = env.deliver_msg(fn_0_second);
        assert_eq!(secondary, env.get_peer_network_id(fn_1));
        env.assert_no_message_sent(fn_0_public);

        // deliver validator's chunk response
        if num_commit < 7 {
            env.deliver_msg(fn_1);
        }
    }

    // we don't deliver the validator's or the secondary vfn network's last chunk response
    // wait for fn_0's chunk request to time out
    std::thread::sleep(std::time::Duration::from_millis(multicast_timeout_ms));

    for num_commit in 8..=11 {
        env.commit(0, num_commit * 5);
        for fn_0_upstream in 2..=3 {
            env.clone_storage(0, fn_0_upstream);
            env.wait_for_version(fn_0_upstream, num_commit * 5, None);
        }

        // check that fn_0 sends chunk requests to both primary (vfn) and fallback ("second") network
        let primary = env.deliver_msg(fn_0_vfn);
        assert_eq!(primary, env.get_peer_network_id(validator));
        let secondary = env.deliver_msg(fn_0_second);
        assert_eq!(secondary, env.get_peer_network_id(fn_1));
        let public = env.deliver_msg(fn_0_public);
        assert_eq!(public, env.get_peer_network_id(fn_2));

        // deliver third fallback's chunk response
        env.deliver_msg(fn_2);
    }

    // Test case: deliver chunks from all upstream with third fallback as first responder
    // Expected: next chunk request should still be sent to all upstream because validator did not deliver response first
    env.deliver_msg(validator);
    env.deliver_msg(fn_1);

    let mut num_commit = 12;
    env.commit(0, num_commit * 5);
    for fn_0_upstream in 2..=3 {
        env.clone_storage(0, fn_0_upstream);
        env.wait_for_version(fn_0_upstream, num_commit * 5, None);
    }

    let primary = env.deliver_msg(fn_0_vfn);
    assert_eq!(primary, env.get_peer_network_id(validator));
    env.assert_no_message_sent(fn_0_vfn);
    let secondary = env.deliver_msg(fn_0_second);
    assert_eq!(secondary, env.get_peer_network_id(fn_1));
    let public = env.deliver_msg(fn_0_public);
    assert_eq!(public, env.get_peer_network_id(fn_2));

    // Test case: deliver chunks from all upstream with secondary fallback as first responder
    // Expected: next chunk request should still be multicasted to all upstream because primary did not deliver response first
    env.deliver_msg(fn_1);
    env.deliver_msg(validator);
    env.deliver_msg(fn_2);

    num_commit += 1;
    env.commit(0, num_commit * 5);
    for fn_0_upstream in 2..=3 {
        env.clone_storage(0, fn_0_upstream);
        env.wait_for_version(fn_0_upstream, num_commit * 5, None);
    }

    let primary = env.deliver_msg(fn_0_vfn);
    assert_eq!(primary, env.get_peer_network_id(validator));
    let secondary = env.deliver_msg(fn_0_second);
    assert_eq!(secondary, env.get_peer_network_id(fn_1));
    let public = env.deliver_msg(fn_0_public);
    assert_eq!(public, env.get_peer_network_id(fn_2));

    // Test case: deliver chunks from all upstream with primary as first responder
    // Expected: next chunk request should only be sent to primary network
    env.deliver_msg(validator);
    env.deliver_msg(fn_1);
    env.deliver_msg(fn_2);

    num_commit += 1;
    env.commit(0, num_commit * 5);
    for fn_0_upstream in 2..=3 {
        env.clone_storage(0, fn_0_upstream);
        env.wait_for_version(fn_0_upstream, num_commit * 5, None);
    }

    // because of optimistic chunk requesting, request will still be multicasted to all failover
    let primary = env.deliver_msg(fn_0_vfn);
    assert_eq!(primary, env.get_peer_network_id(validator));
    let secondary = env.deliver_msg(fn_0_second);
    assert_eq!(secondary, env.get_peer_network_id(fn_1));
    let public = env.deliver_msg(fn_0_public);
    assert_eq!(public, env.get_peer_network_id(fn_2));

    // check that next chunk request is only be sent to primary network, i.e.
    // multicasting is over
    env.deliver_msg(validator);
    env.deliver_msg(fn_1);
    env.deliver_msg(fn_2);

    let primary = env.deliver_msg(fn_0_vfn);
    assert_eq!(primary, env.get_peer_network_id(validator));
    env.assert_no_message_sent(fn_0_second);
    env.assert_no_message_sent(fn_0_public);
}
