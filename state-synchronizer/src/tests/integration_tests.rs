// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    network::{StateSynchronizerEvents, StateSynchronizerMsg, StateSynchronizerSender},
    state_synchronizer::{StateSyncBootstrapper, StateSyncClient},
    tests::{
        mock_executor_proxy::{MockExecutorProxy, MockRpcHandler},
        mock_storage::MockStorage,
    },
};
use anyhow::{bail, Result};
use channel::{diem_channel, message_queues::QueueStyle};
use diem_config::{
    config::{NodeConfig, RoleType, HANDSHAKE_VERSION},
    network_id::{NetworkContext, NetworkId, NodeNetworkId},
};
use diem_crypto::{hash::ACCUMULATOR_PLACEHOLDER_HASH, test_utils::TEST_SEED, x25519, Uniform};
use diem_infallible::RwLock;
use diem_mempool::mocks::MockSharedMempool;
use diem_network_address::{
    encrypted::{TEST_SHARED_VAL_NETADDR_KEY, TEST_SHARED_VAL_NETADDR_KEY_VERSION},
    parse_memory, NetworkAddress, Protocol,
};
use diem_types::{
    chain_id::ChainId, ledger_info::LedgerInfoWithSignatures, on_chain_config::ValidatorSet,
    transaction::TransactionListWithProof, validator_config::ValidatorConfig,
    validator_info::ValidatorInfo, validator_signer::ValidatorSigner,
    validator_verifier::random_validator_verifier, waypoint::Waypoint, PeerId,
};
use futures::{executor::block_on, future::FutureExt, StreamExt};
use memsocket::MemoryListener;
use netcore::transport::{ConnectionOrigin, ConnectionOrigin::*};
use network::{
    peer_manager::{
        builder::AuthenticationMode, conn_notifs_channel, ConnectionNotification,
        ConnectionRequestSender, PeerManagerNotification, PeerManagerRequest,
        PeerManagerRequestSender,
    },
    protocols::{
        direct_send::Message,
        network::{NewNetworkEvents, NewNetworkSender},
    },
    transport::ConnectionMetadata,
    DisconnectReason, ProtocolId,
};
use network_builder::builder::NetworkBuilder;
use rand::{rngs::StdRng, SeedableRng};
use std::{
    cell::{RefCell, RefMut},
    collections::HashMap,
    ops::DerefMut,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use tokio::runtime::Runtime;

struct SynchronizerPeer {
    client: Option<StateSyncClient>,
    mempool: Option<MockSharedMempool>,
    multi_peer_ids: Option<Vec<PeerId>>, // Holds the peer's PeerIds, to support nodes with multiple network IDs.
    network_addr: NetworkAddress,
    network_key: x25519::PrivateKey,
    peer_id: PeerId,
    public_key: ValidatorInfo,
    signer: ValidatorSigner,
    storage_proxy: Option<Arc<RwLock<MockStorage>>>,
    synchronizer: Option<StateSyncBootstrapper>,
}

impl SynchronizerPeer {
    fn commit(&self, version: u64) {
        let num_txns = version - self.storage_proxy.as_ref().unwrap().write().version();
        assert!(num_txns > 0);

        let (committed_txns, signed_txns) = self
            .storage_proxy
            .as_ref()
            .unwrap()
            .write()
            .commit_new_txns(num_txns);
        let mempool = self.mempool.as_ref().unwrap();
        assert!(mempool.add_txns(signed_txns.clone()).is_ok());

        // Run StateSyncClient::commit on a tokio runtime to support tokio::timeout
        // in commit().
        assert!(Runtime::new()
            .unwrap()
            .block_on(self.client.as_ref().unwrap().commit(committed_txns, vec![]))
            .is_ok());
        let mempool_txns = mempool.read_timeline(0, signed_txns.len());
        for txn in signed_txns.iter() {
            assert!(!mempool_txns.contains(txn));
        }
    }

    fn get_peer_id(&self, network_id: usize) -> PeerId {
        self.multi_peer_ids.as_ref().unwrap()[network_id]
    }

    fn latest_li(&self) -> LedgerInfoWithSignatures {
        self.storage_proxy
            .as_ref()
            .unwrap()
            .read()
            .highest_local_li()
    }

    fn sync_to(&self, target: LedgerInfoWithSignatures) {
        block_on(self.client.as_ref().unwrap().sync_to(target)).unwrap()
    }

    fn wait_for_version(&self, target_version: u64, highest_li_version: Option<u64>) -> bool {
        let max_retries = 30;
        for _ in 0..max_retries {
            let state = block_on(self.client.as_ref().unwrap().get_state()).unwrap();
            if state.synced_version() == target_version {
                return highest_li_version
                    .map_or(true, |li_version| li_version == state.committed_version());
            }
            std::thread::sleep(std::time::Duration::from_millis(1000));
        }
        false
    }
}

struct SynchronizerEnv {
    network_conn_event_notifs_txs: HashMap<PeerId, conn_notifs_channel::Sender>,
    network_id: NetworkId,
    network_notifs_txs:
        HashMap<PeerId, diem_channel::Sender<(PeerId, ProtocolId), PeerManagerNotification>>,
    network_reqs_rxs:
        HashMap<PeerId, diem_channel::Receiver<(PeerId, ProtocolId), PeerManagerRequest>>,
    peers: Vec<RefCell<SynchronizerPeer>>,
    runtime: Runtime,
}

impl SynchronizerEnv {
    // Moves peer 0 to the next epoch. Note that other peers are not going to be able to discover
    // their new signers: they're going to learn about the new epoch public key through state
    // synchronization, but private keys are discovered separately.
    pub fn move_to_next_epoch(&self) {
        let num_peers = self.peers.len();
        let (signers, _verifier) = random_validator_verifier(num_peers, None, true);
        let new_keys = self
            .peers
            .iter()
            .enumerate()
            .map(|(idx, peer)| {
                let peer = peer.borrow();
                ValidatorInfo::new(
                    signers[idx].author(),
                    peer.public_key.consensus_voting_power(),
                    peer.public_key.config().clone(),
                )
            })
            .collect::<Vec<ValidatorInfo>>();
        let validator_set = ValidatorSet::new(new_keys);
        self.peers[0]
            .borrow()
            .storage_proxy
            .as_ref()
            .unwrap()
            .write()
            .move_to_next_epoch(signers[0].clone(), validator_set);
    }

    fn get_synchronizer_peer(&self, index: usize) -> RefMut<SynchronizerPeer> {
        self.peers[index].borrow_mut()
    }

    fn new(num_peers: usize) -> Self {
        ::diem_logger::Logger::init_for_testing();

        let (signers, public_keys, network_keys, network_addrs) = initial_setup(num_peers);

        let mut peers = vec![];
        for peer_index in 0..num_peers {
            let peer = SynchronizerPeer {
                client: None,
                mempool: None,
                multi_peer_ids: None,
                network_addr: network_addrs[peer_index].clone(),
                network_key: network_keys[peer_index].clone(),
                peer_id: signers[peer_index].author(),
                public_key: public_keys[peer_index].clone(),
                signer: signers[peer_index].clone(),
                storage_proxy: None,
                synchronizer: None,
            };
            peers.push(RefCell::new(peer));
        }

        Self {
            network_conn_event_notifs_txs: HashMap::new(),
            network_id: NetworkId::Validator,
            network_notifs_txs: HashMap::new(),
            network_reqs_rxs: HashMap::new(),
            peers,
            runtime: Runtime::new().unwrap(),
        }
    }

    // Starts a new state sync peer with the validator role.
    fn start_validator_peer(&mut self, peer_index: usize, mock_network: bool) {
        self.start_synchronizer_peer(
            peer_index,
            default_handler(),
            RoleType::Validator,
            Waypoint::default(),
            mock_network,
            None,
        );
    }

    // Starts a new state sync peer with the fullnode role.
    fn start_fullnode_peer(&mut self, peer_index: usize, mock_network: bool) {
        self.start_synchronizer_peer(
            peer_index,
            default_handler(),
            RoleType::FullNode,
            Waypoint::default(),
            mock_network,
            None,
        );
    }

    // Sets up and starts the synchronizer at the given node index.
    fn start_synchronizer_peer(
        &mut self,
        index: usize,
        handler: MockRpcHandler,
        role: RoleType,
        waypoint: Waypoint,
        mock_network: bool,
        upstream_networks: Option<Vec<NetworkId>>,
    ) {
        self.setup_synchronizer_peer(
            index,
            handler,
            role,
            waypoint,
            60_000,
            120_000,
            mock_network,
            upstream_networks,
        );
    }

    fn setup_synchronizer_peer(
        &mut self,
        index: usize,
        handler: MockRpcHandler,
        role: RoleType,
        waypoint: Waypoint,
        timeout_ms: u64,
        multicast_timeout_ms: u64,
        mock_network: bool,
        upstream_networks: Option<Vec<NetworkId>>,
    ) {
        let (config, network_id) = setup_state_sync_config(
            index,
            role,
            timeout_ms,
            multicast_timeout_ms,
            &upstream_networks,
        );
        let network_handles = self.setup_network_handles(index, &role, mock_network, network_id);
        let validators: Vec<ValidatorInfo> = self
            .peers
            .iter()
            .map(|peer| peer.borrow().public_key.clone())
            .collect();

        let mut peer = self.peers[index].borrow_mut();
        let storage_proxy = Arc::new(RwLock::new(MockStorage::new(
            LedgerInfoWithSignatures::genesis(
                *ACCUMULATOR_PLACEHOLDER_HASH,
                ValidatorSet::new(validators.to_vec()),
            ),
            peer.signer.clone(),
        )));

        let (mempool_channel, mempool_requests) = futures::channel::mpsc::channel(1_024);
        let synchronizer = StateSyncBootstrapper::bootstrap_with_executor_proxy(
            Runtime::new().unwrap(),
            network_handles,
            mempool_channel,
            role,
            waypoint,
            &config.state_sync,
            config.upstream,
            MockExecutorProxy::new(handler, storage_proxy.clone()),
        );

        peer.client = Some(synchronizer.create_client());
        peer.mempool = Some(MockSharedMempool::new(Some(mempool_requests)));
        peer.synchronizer = Some(synchronizer);
        peer.storage_proxy = Some(storage_proxy);
    }

    fn setup_network_handles(
        &mut self,
        index: usize,
        role: &RoleType,
        mock_network: bool,
        network_id: NetworkId,
    ) -> Vec<(
        NodeNetworkId,
        StateSynchronizerSender,
        StateSynchronizerEvents,
    )> {
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
                    diem_channel::new(QueueStyle::LIFO, 1, None);
                let (connection_reqs_tx, _) = diem_channel::new(QueueStyle::LIFO, 1, None);
                let (network_notifs_tx, network_notifs_rx) =
                    diem_channel::new(QueueStyle::LIFO, 1, None);
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

            let mut peer = self.peers[index].borrow_mut();
            peer.multi_peer_ids = Some(network_ids);
        } else {
            let peer = self.peers[index].borrow();
            let auth_mode = AuthenticationMode::Mutual(peer.network_key.clone());
            let network_context = Arc::new(NetworkContext::new(
                self.network_id.clone(),
                RoleType::Validator,
                peer.peer_id,
            ));

            let seed_addrs: HashMap<_, _> = self
                .peers
                .iter()
                .map(|peer| {
                    let peer = peer.borrow();
                    (peer.peer_id, vec![peer.network_addr.clone()])
                })
                .collect();
            let seed_pubkeys = HashMap::new();
            let trusted_peers = Arc::new(RwLock::new(HashMap::new()));

            // Recover the base address we bound previously.
            let addr_protos = peer.network_addr.as_slice();
            let (port, _suffix) = parse_memory(addr_protos).unwrap();
            let base_addr = NetworkAddress::from(Protocol::Memory(port));

            let mut network_builder = NetworkBuilder::new_for_test(
                ChainId::default(),
                seed_addrs,
                seed_pubkeys,
                trusted_peers,
                network_context,
                base_addr,
                auth_mode,
            );

            let (sender, events) =
                network_builder.add_protocol_handler(crate::network::network_endpoint_config());
            network_builder.build(self.runtime.handle().clone()).start();
            network_handles.push((NodeNetworkId::new(network_id, 0), sender, events));
        };
        network_handles
    }

    /// Delivers next message from peer with index `sender` in this SynchronizerEnv
    /// Returns the recipient of the msg
    fn deliver_msg(&mut self, sender_peer_id: PeerId) -> (PeerId, Message) {
        let network_reqs_rx = self.network_reqs_rxs.get_mut(&sender_peer_id).unwrap();
        let network_req = block_on(network_reqs_rx.next()).unwrap();

        // await next message from node
        if let PeerManagerRequest::SendMessage(receiver_id, msg) = network_req {
            let receiver_network_notif_tx = self.network_notifs_txs.get_mut(&receiver_id).unwrap();
            receiver_network_notif_tx
                .push(
                    (sender_peer_id, ProtocolId::StateSynchronizerDirectSend),
                    PeerManagerNotification::RecvMessage(sender_peer_id, msg.clone()),
                )
                .unwrap();
            (receiver_id, msg)
        } else {
            panic!("received network request other than PeerManagerRequest");
        }
    }

    // checks that the `env_idx`th peer in this env sends no message to its `network_idx`th network
    fn assert_no_message_sent(&mut self, sender_peer_id: PeerId) {
        let network_reqs_rx = self.network_reqs_rxs.get_mut(&sender_peer_id).unwrap();
        assert!(network_reqs_rx.select_next_some().now_or_never().is_none());
    }

    fn clone_storage(&mut self, from_idx: usize, to_idx: usize) {
        let from_peer = self.peers[from_idx].borrow();
        let from_storage = from_peer.storage_proxy.as_ref().unwrap();
        let from_storage = from_storage.read();

        let to_peer = self.peers[to_idx].borrow();
        let to_storage = to_peer.storage_proxy.as_ref().unwrap();
        let mut to_storage = to_storage.write();
        let to_storage = to_storage.deref_mut();

        *to_storage = from_storage.clone();
    }

    fn send_peer_event(
        &mut self,
        sender_peer_id: PeerId,
        receiver_peer_id: PeerId,
        new_peer: bool,
        direction: ConnectionOrigin,
    ) {
        let mut metadata = ConnectionMetadata::mock(sender_peer_id);
        metadata.origin = direction;

        let notif = if new_peer {
            ConnectionNotification::NewPeer(metadata, NetworkContext::mock())
        } else {
            ConnectionNotification::LostPeer(
                metadata,
                NetworkContext::mock(),
                DisconnectReason::ConnectionLost,
            )
        };

        let conn_notifs_tx = self
            .network_conn_event_notifs_txs
            .get_mut(&receiver_peer_id)
            .unwrap();
        conn_notifs_tx.push(sender_peer_id, notif).unwrap();
    }
}

fn check_chunk_request(msg: StateSynchronizerMsg, known_version: u64, target_version: Option<u64>) {
    match msg {
        StateSynchronizerMsg::GetChunkRequest(req) => {
            assert_eq!(req.known_version, known_version);
            assert_eq!(req.target.version(), target_version);
        }
        StateSynchronizerMsg::GetChunkResponse(_) => {
            panic!("received chunk response when expecting chunk request");
        }
    }
}

fn check_chunk_response(
    msg: StateSynchronizerMsg,
    response_li_version: u64,
    chunk_start_version: u64,
    chunk_length: usize,
) {
    match msg {
        StateSynchronizerMsg::GetChunkRequest(_) => {
            panic!("received chunk response when expecting chunk request");
        }
        StateSynchronizerMsg::GetChunkResponse(resp) => {
            assert_eq!(resp.response_li.version(), response_li_version);
            assert_eq!(
                resp.txn_list_with_proof.first_transaction_version.unwrap(),
                chunk_start_version
            );
            assert_eq!(resp.txn_list_with_proof.transactions.len(), chunk_length)
        }
    }
}

fn default_handler() -> MockRpcHandler {
    Box::new(|resp| -> Result<TransactionListWithProof> { Ok(resp) })
}

// Returns the initial peers with their signatures
fn initial_setup(
    count: usize,
) -> (
    Vec<ValidatorSigner>,
    Vec<ValidatorInfo>,
    Vec<x25519::PrivateKey>,
    Vec<NetworkAddress>,
) {
    let (signers, _verifier) = random_validator_verifier(count, None, true);

    // Setup identity public keys.
    let mut rng = StdRng::from_seed(TEST_SEED);
    let network_keys: Vec<_> = (0..count)
        .map(|_| x25519::PrivateKey::generate(&mut rng))
        .collect();

    let mut validator_infos = vec![];
    let mut network_addrs = vec![];

    for (idx, signer) in signers.iter().enumerate() {
        let peer_id = signer.author();

        // Reserve an unused `/memory/<port>` address by binding port 0; we
        // can immediately discard the listener here and safely rebind to this
        // address later.
        let port = MemoryListener::bind(0).unwrap().local_addr();
        let addr = NetworkAddress::from(Protocol::Memory(port));
        let addr = addr.append_prod_protos(network_keys[idx].public_key(), HANDSHAKE_VERSION);

        let enc_addr = addr.clone().encrypt(
            &TEST_SHARED_VAL_NETADDR_KEY,
            TEST_SHARED_VAL_NETADDR_KEY_VERSION,
            &peer_id,
            0, /* seq_num */
            0, /* addr_idx */
        );

        // The voting power of peer 0 is enough to generate an LI that passes validation.
        let voting_power = if idx == 0 { 1000 } else { 1 };
        let validator_config = ValidatorConfig::new(
            signer.public_key(),
            bcs::to_bytes(&vec![enc_addr.unwrap()]).unwrap(),
            bcs::to_bytes(&vec![addr.clone()]).unwrap(),
        );
        let validator_info = ValidatorInfo::new(peer_id, voting_power, validator_config);
        validator_infos.push(validator_info);
        network_addrs.push(addr);
    }
    (signers, validator_infos, network_keys, network_addrs)
}

fn setup_state_sync_config(
    index: usize,
    role: RoleType,
    timeout_ms: u64,
    multicast_timeout_ms: u64,
    upstream_networks: &Option<Vec<NetworkId>>,
) -> (NodeConfig, NetworkId) {
    let mut config = diem_config::config::NodeConfig::default_for_validator();
    config.base.role = role;
    config.state_sync.sync_request_timeout_ms = timeout_ms;
    config.state_sync.multicast_timeout_ms = multicast_timeout_ms;

    // Too many tests expect this, so we overwrite the value
    config.state_sync.chunk_limit = 250;

    let network_id = if role.is_validator() {
        NetworkId::Validator
    } else {
        NetworkId::vfn_network()
    };

    if !role.is_validator() {
        config.full_node_networks = vec![config.validator_network.unwrap()];
        config.validator_network = None;
        // setup upstream network for FN
        if let Some(upstream_networks) = upstream_networks {
            config.upstream.networks = upstream_networks.clone();
        } else if index > 0 {
            config.upstream.networks.push(network_id.clone());
        }
    }

    (config, network_id)
}

#[test]
fn test_basic_catch_up() {
    let mut env = SynchronizerEnv::new(2);

    env.start_validator_peer(0, false);
    env.start_validator_peer(1, false);

    let validator_0 = env.get_synchronizer_peer(0);
    let validator_1 = env.get_synchronizer_peer(1);

    // Test small sequential syncs, batch sync for multiple transactions and
    // batch sync for multiple chunks.
    let synced_versions = vec![1, 2, 3, 4, 5, 20, 2000];
    for version in synced_versions {
        validator_0.commit(version);
        let target_li = validator_0.latest_li();

        validator_1.sync_to(target_li);
        assert_eq!(validator_1.latest_li().ledger_info().version(), version);
    }
}

#[test]
fn test_flaky_peer_sync() {
    let mut env = SynchronizerEnv::new(2);

    env.start_validator_peer(0, false);

    // Create handler that causes error, but has successful retries
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
    env.start_synchronizer_peer(
        1,
        handler,
        RoleType::Validator,
        Waypoint::default(),
        false,
        None,
    );

    let validator_0 = env.get_synchronizer_peer(0);
    let validator_1 = env.get_synchronizer_peer(1);

    let synced_version = 20;
    validator_0.commit(synced_version);
    validator_1.sync_to(validator_0.latest_li());
    assert_eq!(
        validator_1.latest_li().ledger_info().version(),
        synced_version
    );
}

#[test]
#[should_panic]
fn test_request_timeout() {
    let mut env = SynchronizerEnv::new(2);

    let handler =
        Box::new(move |_| -> Result<TransactionListWithProof> { bail!("chunk fetch failed") });
    env.start_synchronizer_peer(
        0,
        handler,
        RoleType::Validator,
        Waypoint::default(),
        false,
        None,
    );
    env.setup_synchronizer_peer(
        1,
        default_handler(),
        RoleType::Validator,
        Waypoint::default(),
        100,
        300,
        false,
        None,
    );

    let validator_0 = env.get_synchronizer_peer(0);
    let validator_1 = env.get_synchronizer_peer(0);

    validator_0.commit(1);
    validator_1.sync_to(validator_0.latest_li());
}

#[test]
fn test_full_node() {
    let mut env = SynchronizerEnv::new(2);

    env.start_validator_peer(0, false);
    env.start_fullnode_peer(1, false);

    let validator = env.get_synchronizer_peer(0);
    let fullnode = env.get_synchronizer_peer(1);

    validator.commit(10);
    // first sync should be fulfilled immediately after peer discovery
    assert!(fullnode.wait_for_version(10, None));

    validator.commit(20);
    // second sync will be done via long polling cause first node should send new request
    // after receiving first chunk immediately
    assert!(fullnode.wait_for_version(20, None));
}

#[test]
fn catch_up_through_epochs_validators() {
    let mut env = SynchronizerEnv::new(2);

    env.start_validator_peer(0, false);
    env.start_validator_peer(1, false);

    // catch up to the next epoch starting from the middle of the current one
    env.get_synchronizer_peer(0).commit(20);
    env.get_synchronizer_peer(1)
        .sync_to(env.get_synchronizer_peer(0).latest_li());
    env.get_synchronizer_peer(0).commit(40);

    env.move_to_next_epoch();

    env.get_synchronizer_peer(0).commit(100);
    env.get_synchronizer_peer(1)
        .sync_to(env.get_synchronizer_peer(0).latest_li());
    assert_eq!(
        env.get_synchronizer_peer(1)
            .latest_li()
            .ledger_info()
            .version(),
        100
    );
    assert_eq!(
        env.get_synchronizer_peer(1)
            .latest_li()
            .ledger_info()
            .epoch(),
        2
    );

    // catch up through multiple epochs
    for epoch in 2..10 {
        env.get_synchronizer_peer(0).commit(epoch * 100);
        env.move_to_next_epoch();
    }
    env.get_synchronizer_peer(0).commit(950); // At this point peer 0 is at epoch 10 and version 950
    env.get_synchronizer_peer(1)
        .sync_to(env.get_synchronizer_peer(0).latest_li());
    assert_eq!(
        env.get_synchronizer_peer(1)
            .latest_li()
            .ledger_info()
            .version(),
        950
    );
    assert_eq!(
        env.get_synchronizer_peer(1)
            .latest_li()
            .ledger_info()
            .epoch(),
        10
    );
}

#[test]
fn catch_up_through_epochs_full_node() {
    let mut env = SynchronizerEnv::new(3);

    env.start_validator_peer(0, false);

    // catch up through multiple epochs
    for epoch in 1..10 {
        env.get_synchronizer_peer(0).commit(epoch * 100);
        env.move_to_next_epoch();
    }
    env.get_synchronizer_peer(0).commit(950); // At this point validator_0 is at epoch 10 and version 950

    env.start_fullnode_peer(1, false);
    assert!(env.get_synchronizer_peer(1).wait_for_version(950, None));
    assert_eq!(
        env.get_synchronizer_peer(1)
            .latest_li()
            .ledger_info()
            .epoch(),
        10
    );

    // Peer 2 has peer 1 as its upstream, should catch up from it.
    env.start_fullnode_peer(2, false);
    assert!(env.get_synchronizer_peer(2).wait_for_version(950, None));
    assert_eq!(
        env.get_synchronizer_peer(2)
            .latest_li()
            .ledger_info()
            .epoch(),
        10
    );
}

#[test]
fn catch_up_with_waypoints() {
    let mut env = SynchronizerEnv::new(3);

    env.start_validator_peer(0, false);

    let mut curr_version = 0;
    for _ in 1..10 {
        curr_version += 100;
        env.get_synchronizer_peer(0).commit(curr_version);
        env.move_to_next_epoch();

        curr_version += 400;
        // this creates an epoch that spans >1 chunk (chunk_size = 250)
        env.get_synchronizer_peer(0).commit(curr_version);
        env.move_to_next_epoch();
    }
    env.get_synchronizer_peer(0).commit(5250); // At this point validator is at epoch 19 and version 5250

    // Create a waypoint based on LedgerInfo of peer 0 at version 3500 (epoch 14)
    let waypoint_li = env
        .get_synchronizer_peer(0)
        .storage_proxy
        .as_ref()
        .unwrap()
        .read()
        .get_epoch_ending_ledger_info(3500)
        .unwrap();
    let waypoint = Waypoint::new_epoch_boundary(waypoint_li.ledger_info()).unwrap();

    env.start_synchronizer_peer(
        1,
        default_handler(),
        RoleType::FullNode,
        waypoint,
        false,
        None,
    );
    let fullnode = env.get_synchronizer_peer(1);
    block_on(fullnode.client.as_ref().unwrap().wait_until_initialized()).unwrap();
    assert!(fullnode.latest_li().ledger_info().version() >= 3500);
    assert!(fullnode.latest_li().ledger_info().epoch() >= 14);

    // Once caught up with the waypoint fullnode 1 continues with the regular state sync
    assert!(fullnode.wait_for_version(5250, None));
    assert_eq!(fullnode.latest_li().ledger_info().epoch(), 19);
    drop(fullnode);

    // Peer 2 has peer 1 as its upstream, should catch up from it.
    env.start_fullnode_peer(2, false);
    let fullnode = env.get_synchronizer_peer(2);
    assert!(fullnode.wait_for_version(5250, None));
    assert_eq!(fullnode.latest_li().ledger_info().epoch(), 19);
}

#[test]
fn test_lagging_upstream_long_poll() {
    let mut env = SynchronizerEnv::new(4);

    env.start_validator_peer(0, true);
    env.setup_synchronizer_peer(
        1,
        default_handler(),
        RoleType::FullNode,
        Waypoint::default(),
        10_000,
        1_000_000,
        true,
        Some(vec![NetworkId::vfn_network(), NetworkId::Public]),
    );
    env.start_synchronizer_peer(
        2,
        default_handler(),
        RoleType::FullNode,
        Waypoint::default(),
        true,
        Some(vec![NetworkId::vfn_network()]),
    );
    // we treat this a standalone node whose local state we use as the baseline
    // to clone state to the other nodes
    env.start_validator_peer(3, true);

    // network handles for each node
    let validator_peer_id = env.get_synchronizer_peer(0).get_peer_id(0);
    let full_node_vfn_network_peer_id = env.get_synchronizer_peer(1).get_peer_id(0);
    let full_node_failover_network_peer_id = env.get_synchronizer_peer(1).get_peer_id(2);
    let failover_fn_vfn_network_peer_id = env.get_synchronizer_peer(2).get_peer_id(0);
    let failover_fn_peer_id = env.get_synchronizer_peer(2).get_peer_id(2);

    env.get_synchronizer_peer(0).commit(400);

    // validator discovers FN
    env.send_peer_event(
        full_node_vfn_network_peer_id,
        validator_peer_id,
        true,
        Inbound,
    );
    // fn discovers validator
    env.send_peer_event(
        validator_peer_id,
        full_node_vfn_network_peer_id,
        true,
        Outbound,
    );

    // FN discovers failover upstream
    env.send_peer_event(
        full_node_failover_network_peer_id,
        failover_fn_peer_id,
        true,
        Inbound,
    );
    env.send_peer_event(
        failover_fn_peer_id,
        full_node_failover_network_peer_id,
        true,
        Outbound,
    );

    let (_, msg) = env.deliver_msg(full_node_vfn_network_peer_id);
    // expected: known_version 0, epoch 1, no target LI version
    let req: StateSynchronizerMsg =
        bcs::from_bytes(&msg.mdata).expect("failed bcs deserialization");
    check_chunk_request(req, 0, None);

    let (_, msg) = env.deliver_msg(validator_peer_id);
    let resp: StateSynchronizerMsg =
        bcs::from_bytes(&msg.mdata).expect("failed bcs deserialization");
    check_chunk_response(resp, 400, 1, 250);
    env.get_synchronizer_peer(1).wait_for_version(250, None);

    // validator loses FN
    env.send_peer_event(
        full_node_vfn_network_peer_id,
        validator_peer_id,
        false,
        Inbound,
    );
    // fn loses validator
    env.send_peer_event(
        validator_peer_id,
        full_node_vfn_network_peer_id,
        false,
        Outbound,
    );

    // full_node sends chunk request to failover upstream for known_version 250 and target LI 400
    let (_, msg) = env.deliver_msg(full_node_failover_network_peer_id);
    let msg: StateSynchronizerMsg =
        bcs::from_bytes(&msg.mdata).expect("failed bcs deserialization");
    check_chunk_request(msg, 250, Some(400));

    // update failover VFN from lagging state to updated state
    // so it can deliver full_node's long-poll subscription
    env.get_synchronizer_peer(0).commit(500);
    // we directly sync up the storage of the failover upstream with this validator's for ease of testing
    env.clone_storage(0, 2);
    env.get_synchronizer_peer(2)
        .wait_for_version(500, Some(500));

    // connect the validator and the failover vfn so FN can sync to validator
    // validator discovers FN
    env.send_peer_event(
        failover_fn_vfn_network_peer_id,
        validator_peer_id,
        true,
        Inbound,
    );
    // fn discovers validator
    env.send_peer_event(
        validator_peer_id,
        failover_fn_vfn_network_peer_id,
        true,
        Outbound,
    );

    // trigger another commit so that the failover fn's commit will trigger subscription delivery
    env.get_synchronizer_peer(0).commit(600);
    // failover fn sends chunk request to validator
    let (_, msg) = env.deliver_msg(failover_fn_vfn_network_peer_id);
    let msg: StateSynchronizerMsg =
        bcs::from_bytes(&msg.mdata).expect("failed bcs deserialization");
    check_chunk_request(msg, 500, None);
    let (_, msg) = env.deliver_msg(validator_peer_id);
    let resp: StateSynchronizerMsg =
        bcs::from_bytes(&msg.mdata).expect("failed bcs deserialization");
    check_chunk_response(resp, 600, 501, 100);

    // failover sends long-poll subscription to fullnode
    let (_, msg) = env.deliver_msg(failover_fn_peer_id);
    let resp: StateSynchronizerMsg =
        bcs::from_bytes(&msg.mdata).expect("failed bcs deserialization");
    check_chunk_response(resp, 600, 251, 250);

    // full_node sends chunk request to failover upstream for known_version 250 and target LI 400
    let (_, msg) = env.deliver_msg(full_node_failover_network_peer_id);
    let msg: StateSynchronizerMsg =
        bcs::from_bytes(&msg.mdata).expect("failed bcs deserialization");
    // here we check that the next requested version is not the older target LI 400 - that should be
    // pruned out from PendingLedgerInfos since it becomes outdated after the known_version advances to 500
    check_chunk_request(msg, 500, None);

    // check that fullnode successfully finishes sync to 600
    let (_, msg) = env.deliver_msg(failover_fn_peer_id);
    let resp: StateSynchronizerMsg =
        bcs::from_bytes(&msg.mdata).expect("failed bcs deserialization");
    check_chunk_response(resp, 600, 501, 100);
    env.get_synchronizer_peer(1)
        .wait_for_version(600, Some(600));
}

// test full node catching up to validator that is also making progress
#[test]
fn test_sync_pending_ledger_infos() {
    let mut env = SynchronizerEnv::new(2);

    env.start_validator_peer(0, true);
    env.start_fullnode_peer(1, true);

    let validator_peer_id = env.get_synchronizer_peer(0).get_peer_id(0);
    let fullnode_peer_id = env.get_synchronizer_peer(1).get_peer_id(0);

    // validator discovers fn
    env.send_peer_event(fullnode_peer_id, validator_peer_id, true, Inbound);

    // fn discovers validator
    env.send_peer_event(validator_peer_id, fullnode_peer_id, true, Outbound);

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

    for (idx, expected_state) in expected_states.iter().enumerate() {
        // commit if applicable
        if let Some(version) = commit_versions.get(idx) {
            env.get_synchronizer_peer(0).commit(*version);
        }
        env.deliver_msg(fullnode_peer_id);
        env.deliver_msg(validator_peer_id);
        let (sync_version, li_version) = expected_state;
        assert!(
            env.get_synchronizer_peer(1)
                .wait_for_version(*sync_version, Some(*li_version)),
            "didn't reach synced version {} and highest LI version {}",
            sync_version,
            li_version
        );
    }
}

#[test]
#[ignore] // TODO: https://github.com/diem/diem/issues/5771
fn test_fn_failover() {
    let mut env = SynchronizerEnv::new(5);

    env.start_validator_peer(0, true);
    env.setup_synchronizer_peer(
        1,
        default_handler(),
        RoleType::FullNode,
        Waypoint::default(),
        1_000,
        60_000,
        true,
        Some(vec![NetworkId::vfn_network(), NetworkId::Public]),
    );

    // start up 3 publicly available VFN
    env.start_fullnode_peer(2, true);
    env.start_fullnode_peer(3, true);
    env.start_fullnode_peer(4, true);

    // connect everyone
    let validator_peer_id = env.get_synchronizer_peer(0).get_peer_id(1);
    let fn_0_vfn_peer_id = env.get_synchronizer_peer(1).get_peer_id(0);
    let fn_0_public_peer_id = env.get_synchronizer_peer(1).get_peer_id(2);
    let fn_1_peer_id = env.get_synchronizer_peer(2).get_peer_id(2);
    let fn_2_peer_id = env.get_synchronizer_peer(3).get_peer_id(2);
    let fn_3_peer_id = env.get_synchronizer_peer(4).get_peer_id(2);

    // vfn network:
    // validator discovers fn_0
    env.send_peer_event(fn_0_vfn_peer_id, validator_peer_id, true, Inbound);
    // fn_0 discovers validator
    env.send_peer_event(validator_peer_id, fn_0_vfn_peer_id, true, Outbound);

    // public network:
    // fn_0 sends new peer event to all its upstream public peers
    let upstream_peer_ids = [fn_1_peer_id, fn_2_peer_id, fn_3_peer_id];
    for peer in upstream_peer_ids.iter() {
        env.send_peer_event(fn_0_public_peer_id, *peer, true, Inbound);
        env.send_peer_event(*peer, fn_0_public_peer_id, true, Outbound);
    }

    // commit some txns on v
    // check that fn_0 sends chunk requests to v only
    for num_commit in 1..=5 {
        env.get_synchronizer_peer(0).commit(num_commit * 5);
        for public_upstream in 2..=4 {
            // we just directly sync up the storage of all the upstream peers of fn_0
            // for ease of testing
            env.clone_storage(0, public_upstream);
        }
        // deliver fn_0's chunk request
        let (recipient, _) = env.deliver_msg(fn_0_vfn_peer_id);
        assert_eq!(recipient, validator_peer_id);
        env.assert_no_message_sent(fn_0_public_peer_id);
        // deliver validator's chunk response
        if num_commit < 5 {
            env.deliver_msg(validator_peer_id);
        }
    }

    // bring down v
    env.send_peer_event(fn_0_vfn_peer_id, validator_peer_id, false, Inbound);
    env.send_peer_event(validator_peer_id, fn_0_vfn_peer_id, false, Outbound);

    // deliver chunk response to fn_0 after the lost peer event
    // so that the next chunk request is guaranteed to be sent after the lost peer event
    env.deliver_msg(validator_peer_id);

    // check that vfn sends chunk requests to the failover FNs only
    let mut last_fallback_recipient = None;
    for num_commit in 6..=10 {
        env.get_synchronizer_peer(0).commit(num_commit * 5);
        for public_upstream in 2..=4 {
            env.clone_storage(0, public_upstream);
        }
        // deliver fn_0's chunk request
        let (recipient, _) = env.deliver_msg(fn_0_public_peer_id);
        assert!(upstream_peer_ids.contains(&recipient));
        env.assert_no_message_sent(fn_0_vfn_peer_id);
        // deliver validator's chunk response
        if num_commit < 10 {
            let (chunk_response_recipient, _) = env.deliver_msg(recipient);
            assert_eq!(chunk_response_recipient, fn_0_public_peer_id);
        } else {
            last_fallback_recipient = Some(recipient);
        }
    }

    // bring down two public fallback
    // disconnect fn_1 and fn_0
    env.send_peer_event(fn_0_public_peer_id, fn_1_peer_id, false, Inbound);
    env.send_peer_event(fn_1_peer_id, fn_0_public_peer_id, false, Outbound);

    // disconnect fn_2 and fn_0
    env.send_peer_event(fn_0_public_peer_id, fn_2_peer_id, false, Inbound);
    env.send_peer_event(fn_2_peer_id, fn_0_public_peer_id, false, Outbound);

    // deliver chunk response to fn_0 after the lost peer events
    // so that the next chunk request is guaranteed to be sent after the lost peer events
    let (chunk_response_recipient, _) = env.deliver_msg(last_fallback_recipient.unwrap());
    assert_eq!(chunk_response_recipient, fn_0_public_peer_id);

    // check we only broadcast to the single live fallback peer (fn_3)
    for num_commit in 11..=15 {
        env.get_synchronizer_peer(0).commit(num_commit * 5);
        for public_upstream in 2..=4 {
            env.clone_storage(0, public_upstream);
        }
        // deliver fn_0's chunk request
        let (recipient, _) = env.deliver_msg(fn_0_public_peer_id);
        assert_eq!(recipient, fn_3_peer_id);
        env.assert_no_message_sent(fn_0_vfn_peer_id);
        // deliver validator's chunk response
        if num_commit < 15 {
            let (chunk_response_recipient, _) = env.deliver_msg(fn_3_peer_id);
            assert_eq!(chunk_response_recipient, fn_0_public_peer_id);
        }
    }

    // bring down everyone
    // disconnect fn_3 and fn_0
    env.send_peer_event(fn_3_peer_id, fn_0_public_peer_id, false, Outbound);
    env.send_peer_event(fn_0_public_peer_id, fn_3_peer_id, false, Inbound);

    // deliver chunk response to fn_0 after the lost peer events
    // so that the next chunk request is guaranteed to be sent after the lost peer events
    let (chunk_response_recipient, _) = env.deliver_msg(fn_3_peer_id);
    assert_eq!(chunk_response_recipient, fn_0_public_peer_id);

    // check no sync requests are sent (all upstream are down)
    env.assert_no_message_sent(fn_0_vfn_peer_id);
    env.assert_no_message_sent(fn_0_public_peer_id);

    // bring back one fallback (fn_2)
    env.send_peer_event(fn_2_peer_id, fn_0_public_peer_id, true, Outbound);
    env.send_peer_event(fn_0_public_peer_id, fn_2_peer_id, true, Inbound);

    // check we only broadcast to the single live fallback peer (fn_2)
    for num_commit in 16..=20 {
        env.get_synchronizer_peer(0).commit(num_commit * 5);
        for public_upstream in 2..=4 {
            env.clone_storage(0, public_upstream);
        }
        // deliver fn_0's chunk request
        let (recipient, _) = env.deliver_msg(fn_0_public_peer_id);
        assert_eq!(recipient, fn_2_peer_id);
        env.assert_no_message_sent(fn_0_vfn_peer_id);
        // deliver validator's chunk response
        if num_commit < 20 {
            let (chunk_response_recipient, _) = env.deliver_msg(fn_2_peer_id);
            assert_eq!(chunk_response_recipient, fn_0_public_peer_id);
        }
    }

    // bring back v again
    env.send_peer_event(fn_0_vfn_peer_id, validator_peer_id, true, Inbound);
    env.send_peer_event(validator_peer_id, fn_0_vfn_peer_id, true, Outbound);

    let (chunk_response_recipient, _) = env.deliver_msg(fn_2_peer_id);
    assert_eq!(chunk_response_recipient, fn_0_public_peer_id);

    // check that vfn sends chunk requests to v only, not fallback upstream
    for num_commit in 21..=25 {
        env.get_synchronizer_peer(0).commit(num_commit * 5);
        for public_upstream in 2..=4 {
            env.clone_storage(0, public_upstream);
        }
        // deliver fn_0's chunk request
        let (recipient, _) = env.deliver_msg(fn_0_vfn_peer_id);
        assert_eq!(recipient, validator_peer_id);
        env.assert_no_message_sent(fn_0_public_peer_id);
        if num_commit < 25 {
            // deliver validator's chunk response
            env.deliver_msg(validator_peer_id);
        }
    }

    // bring back all fallback
    let upstream_peers_to_revive = [
        env.get_synchronizer_peer(2).get_peer_id(1),
        env.get_synchronizer_peer(4).get_peer_id(1),
    ];
    for peer in upstream_peers_to_revive.iter() {
        env.send_peer_event(fn_0_public_peer_id, *peer, true, Inbound);
        env.send_peer_event(*peer, fn_0_public_peer_id, true, Outbound);
    }

    // deliver validator's chunk response after fallback peers are revived
    env.deliver_msg(validator_peer_id);

    // check that we only broadcast to v
    // check that vfn sends chunk requests to v only, not fallback upstream
    for num_commit in 26..=30 {
        env.get_synchronizer_peer(0).commit(num_commit * 5);
        for public_upstream in 2..=4 {
            env.clone_storage(0, public_upstream);
        }
        // deliver fn_0's chunk request
        let (recipient, _) = env.deliver_msg(fn_0_vfn_peer_id);
        assert_eq!(recipient, validator_peer_id);
        env.assert_no_message_sent(fn_0_public_peer_id);
        // deliver validator's chunk response
        env.deliver_msg(validator_peer_id);
    }
}

#[test]
#[ignore]
fn test_multicast_failover() {
    let mut env = SynchronizerEnv::new(5);

    env.start_validator_peer(0, true);

    // set up node with more than 2 upstream networks, which is more than in standard prod setting
    // just to be safe
    let multicast_timeout_ms = 5_000;
    env.setup_synchronizer_peer(
        1,
        default_handler(),
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
    env.start_fullnode_peer(2, true);
    env.start_fullnode_peer(3, true);
    env.start_fullnode_peer(4, true);

    // connect everyone
    let validator_peer_id = env.get_synchronizer_peer(0).get_peer_id(1);
    let fn_0_vfn_peer_id = env.get_synchronizer_peer(1).get_peer_id(0);
    let fn_0_second_peer_id = env.get_synchronizer_peer(1).get_peer_id(1);
    let fn_0_public_peer_id = env.get_synchronizer_peer(1).get_peer_id(2);
    let fn_1_peer_id = env.get_synchronizer_peer(2).get_peer_id(1);
    let fn_2_peer_id = env.get_synchronizer_peer(3).get_peer_id(2);

    // vfn network:
    // validator discovers fn_0
    env.send_peer_event(fn_0_vfn_peer_id, validator_peer_id, true, Inbound);
    // fn_0 discovers validator
    env.send_peer_event(validator_peer_id, fn_0_vfn_peer_id, true, Outbound);

    // second private network: fn_1 is upstream to fn_0
    // fn_1 discovers fn_0
    env.send_peer_event(fn_0_second_peer_id, fn_1_peer_id, true, Inbound);
    // fn_0 discovers fn_1
    env.send_peer_event(fn_1_peer_id, fn_0_second_peer_id, true, Outbound);

    // public network: fn_2 is upstream to fn_1
    // fn_2 discovers fn_0
    env.send_peer_event(fn_0_public_peer_id, fn_2_peer_id, true, Inbound);
    // fn_0 discovers fn_2
    env.send_peer_event(fn_2_peer_id, fn_0_public_peer_id, true, Outbound);

    for num_commit in 1..=3 {
        env.get_synchronizer_peer(0).commit(num_commit * 5);
        // deliver fn_0's chunk request
        let (recipient, _) = env.deliver_msg(fn_0_vfn_peer_id);
        assert_eq!(recipient, validator_peer_id);
        env.assert_no_message_sent(fn_0_second_peer_id);
        env.assert_no_message_sent(fn_0_public_peer_id);
        // deliver validator's chunk response
        if num_commit < 3 {
            env.deliver_msg(validator_peer_id);
        }
    }

    // we don't deliver the validator's last chunk response
    // wait for fn_0's chunk request to time out
    std::thread::sleep(std::time::Duration::from_millis(multicast_timeout_ms));

    // commit some with
    for num_commit in 4..=7 {
        env.get_synchronizer_peer(0).commit(num_commit * 5);
        env.clone_storage(0, 2);
        env.get_synchronizer_peer(2)
            .wait_for_version(num_commit * 5, None);

        // check that fn_0 sends chunk requests to both primary (vfn) and fallback ("second") network
        let (primary, _) = env.deliver_msg(fn_0_vfn_peer_id);
        assert_eq!(primary, validator_peer_id);
        let (secondary, _) = env.deliver_msg(fn_0_second_peer_id);
        assert_eq!(secondary, fn_1_peer_id);
        env.assert_no_message_sent(fn_0_public_peer_id);

        // deliver validator's chunk response
        if num_commit < 7 {
            env.deliver_msg(fn_1_peer_id);
        }
    }

    // we don't deliver the validator's or the secondary vfn network's last chunk response
    // wait for fn_0's chunk request to time out
    std::thread::sleep(std::time::Duration::from_millis(multicast_timeout_ms));

    for num_commit in 8..=11 {
        env.get_synchronizer_peer(0).commit(num_commit * 5);
        env.clone_storage(0, 2);
        env.get_synchronizer_peer(2)
            .wait_for_version(num_commit * 5, None);
        env.clone_storage(0, 3);
        env.get_synchronizer_peer(3)
            .wait_for_version(num_commit * 5, None);

        // check that fn_0 sends chunk requests to both primary (vfn) and fallback ("second") network
        let (primary, _) = env.deliver_msg(fn_0_vfn_peer_id);
        assert_eq!(primary, validator_peer_id);
        let (secondary, _) = env.deliver_msg(fn_0_second_peer_id);
        assert_eq!(secondary, fn_1_peer_id);
        let (public, _) = env.deliver_msg(fn_0_public_peer_id);
        assert_eq!(public, fn_2_peer_id);

        // deliver third fallback's chunk response
        env.deliver_msg(fn_2_peer_id);
    }

    // Test case: deliver chunks from all upstream with third fallback as first responder
    // Expected: next chunk request should still be sent to all upstream because validator did not deliver response first
    env.deliver_msg(validator_peer_id);
    env.deliver_msg(fn_1_peer_id);

    let mut num_commit = 12;
    env.get_synchronizer_peer(0).commit(num_commit * 5);
    env.clone_storage(0, 2);
    env.get_synchronizer_peer(2)
        .wait_for_version(num_commit * 5, None);
    env.clone_storage(0, 3);
    env.get_synchronizer_peer(3)
        .wait_for_version(num_commit * 5, None);

    let (primary, _) = env.deliver_msg(fn_0_vfn_peer_id);
    assert_eq!(primary, validator_peer_id);
    env.assert_no_message_sent(fn_0_vfn_peer_id);
    let (secondary, _) = env.deliver_msg(fn_0_second_peer_id);
    assert_eq!(secondary, fn_1_peer_id);
    let (public, _) = env.deliver_msg(fn_0_public_peer_id);
    assert_eq!(public, fn_2_peer_id);

    // Test case: deliver chunks from all upstream with secondary fallback as first responder
    // Expected: next chunk request should still be multicasted to all upstream because primary did not deliver response first
    env.deliver_msg(fn_1_peer_id);
    env.deliver_msg(validator_peer_id);
    env.deliver_msg(fn_2_peer_id);

    num_commit += 1;
    env.get_synchronizer_peer(0).commit(num_commit * 5);
    env.clone_storage(0, 2);
    env.get_synchronizer_peer(2)
        .wait_for_version(num_commit * 5, None);
    env.clone_storage(0, 3);
    env.get_synchronizer_peer(3)
        .wait_for_version(num_commit * 5, None);

    let (primary, _) = env.deliver_msg(fn_0_vfn_peer_id);
    assert_eq!(primary, validator_peer_id);
    let (secondary, _) = env.deliver_msg(fn_0_second_peer_id);
    assert_eq!(secondary, fn_1_peer_id);
    let (public, _) = env.deliver_msg(fn_0_public_peer_id);
    assert_eq!(public, fn_2_peer_id);

    // Test case: deliver chunks from all upstream with primary as first responder
    // Expected: next chunk request should only be sent to primary network
    env.deliver_msg(validator_peer_id);
    env.deliver_msg(fn_1_peer_id);
    env.deliver_msg(fn_2_peer_id);

    num_commit += 1;
    env.get_synchronizer_peer(0).commit(num_commit * 5);
    env.clone_storage(0, 2);
    env.get_synchronizer_peer(2)
        .wait_for_version(num_commit * 5, None);
    env.clone_storage(0, 3);
    env.get_synchronizer_peer(3)
        .wait_for_version(num_commit * 5, None);

    // because of optimistic chunk requesting, request will still be multicasted to all failover
    let (primary, _) = env.deliver_msg(fn_0_vfn_peer_id);
    assert_eq!(primary, validator_peer_id);
    let (secondary, _) = env.deliver_msg(fn_0_second_peer_id);
    assert_eq!(secondary, fn_1_peer_id);
    let (public, _) = env.deliver_msg(fn_0_public_peer_id);
    assert_eq!(public, fn_2_peer_id);

    // check that next chunk request is only be sent to primary network, i.e.
    // multicasting is over
    env.deliver_msg(validator_peer_id);
    env.deliver_msg(fn_1_peer_id);
    env.deliver_msg(fn_2_peer_id);

    let (primary, _) = env.deliver_msg(fn_0_vfn_peer_id);
    assert_eq!(primary, validator_peer_id);
    env.assert_no_message_sent(fn_0_second_peer_id);
    env.assert_no_message_sent(fn_0_public_peer_id);
}
