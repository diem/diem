// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use channel::{diem_channel, message_queues::QueueStyle};
use claim::assert_ok;
use consensus_notifications::{ConsensusNotificationSender, ConsensusNotifier};
use diem_config::{
    config::{NodeConfig, Peer, PeerRole, RoleType, HANDSHAKE_VERSION},
    network_id::{NetworkContext, NetworkId},
};
use diem_crypto::{
    hash::ACCUMULATOR_PLACEHOLDER_HASH, test_utils::TEST_SEED, x25519, HashValue, Uniform,
};
use diem_infallible::RwLock;
use diem_mempool::mocks::MockSharedMempool;
use diem_time_service::TimeService;
use diem_transaction_builder::stdlib::encode_peer_to_peer_with_metadata_script;
use diem_types::{
    account_address::AccountAddress,
    account_config::xus_tag,
    block_info::BlockInfo,
    chain_id::ChainId,
    contract_event::ContractEvent,
    epoch_state::EpochState,
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    network_address::{
        encrypted::{TEST_SHARED_VAL_NETADDR_KEY, TEST_SHARED_VAL_NETADDR_KEY_VERSION},
        parse_memory, NetworkAddress, Protocol,
    },
    on_chain_config::ValidatorSet,
    proof::TransactionInfoListWithProof,
    test_helpers::transaction_test_helpers::get_test_signed_txn,
    transaction::{
        authenticator::AuthenticationKey, default_protocol::TransactionListWithProof,
        SignedTransaction, Transaction, TransactionPayload,
    },
    validator_config::ValidatorConfig,
    validator_info::ValidatorInfo,
    validator_signer::ValidatorSigner,
    validator_verifier::random_validator_verifier,
    waypoint::Waypoint,
    PeerId,
};
use executor_types::ExecutedTrees;
use futures::{executor::block_on, future::FutureExt, StreamExt};
use memsocket::MemoryListener;
use netcore::transport::ConnectionOrigin;
use network::{
    application::storage::PeerMetadataStorage,
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
use state_sync_v1::{
    bootstrapper::StateSyncBootstrapper,
    client::StateSyncClient,
    error::Error,
    executor_proxy::ExecutorProxyTrait,
    network::{StateSyncEvents, StateSyncSender},
    shared_components::SyncState,
};
use std::{
    cell::{Ref, RefCell},
    collections::{BTreeMap, HashMap},
    ops::DerefMut,
    sync::Arc,
    time::Duration,
};
use tokio::runtime::Runtime;
use vm_genesis::GENESIS_KEYPAIR;

// Networks for validators and fullnodes.

pub struct StateSyncPeer {
    bootstrapper: Option<StateSyncBootstrapper>,
    client: Option<StateSyncClient>,
    consensus_notifier: Option<ConsensusNotifier>,
    mempool: Option<MockSharedMempool>,
    multi_peer_ids: HashMap<NetworkId, PeerId>, // Holds the peer's PeerIds (to support nodes with multiple network IDs).
    network_addr: NetworkAddress,
    network_key: x25519::PrivateKey,
    peer_id: PeerId,
    public_key: ValidatorInfo,
    signer: ValidatorSigner,
    storage_proxy: Option<Arc<RwLock<MockStorage>>>,
}

impl StateSyncPeer {
    pub fn commit(&self, version: u64) {
        let num_txns = version - self.storage_proxy.as_ref().unwrap().write().version();
        assert!(num_txns > 0);

        let (committed_txns, signed_txns) = self
            .storage_proxy
            .as_ref()
            .unwrap()
            .write()
            .commit_new_txns(num_txns);
        let mempool = self.mempool.as_ref().unwrap();
        assert_ok!(mempool.add_txns(signed_txns.clone()));

        assert_ok!(Runtime::new().unwrap().block_on(
            self.consensus_notifier
                .as_ref()
                .unwrap()
                .notify_new_commit(committed_txns, vec![]),
        ));

        // TODO(joshlind): clean this up once we migrate to a new test harness.
        std::thread::sleep(Duration::from_secs(1));
        let mempool_txns = mempool.read_timeline(0, signed_txns.len());
        for txn in signed_txns.iter() {
            assert!(!mempool_txns.contains(txn));
        }
    }

    pub fn get_peer_id(&self, network_id: NetworkId) -> PeerId {
        *self.multi_peer_ids.get(&network_id).unwrap()
    }

    pub fn get_epoch_ending_ledger_info(&self, version: u64) -> LedgerInfoWithSignatures {
        self.storage_proxy
            .as_ref()
            .unwrap()
            .read()
            .get_epoch_ending_ledger_info(version)
            .unwrap()
    }

    pub fn get_validator_info(&self) -> ValidatorInfo {
        self.public_key.clone()
    }

    pub fn latest_li(&self) -> LedgerInfoWithSignatures {
        self.storage_proxy
            .as_ref()
            .unwrap()
            .read()
            .highest_local_li()
    }

    // Moves to the next epoch. Note that other peers are not going to be able to discover
    // their new signers: they're going to learn about the new epoch public key through state
    // sync. Private keys are discovered separately.
    pub fn move_to_next_epoch(&self, validator_infos: Vec<ValidatorInfo>, validator_index: usize) {
        let (validator_set, validator_signers) = create_new_validator_set(validator_infos);
        self.storage_proxy
            .as_ref()
            .unwrap()
            .write()
            .move_to_next_epoch(validator_signers[validator_index].clone(), validator_set);
    }

    pub fn sync_to(&self, target: LedgerInfoWithSignatures) {
        block_on(
            self.consensus_notifier
                .as_ref()
                .unwrap()
                .sync_to_target(target),
        )
        .unwrap();
    }

    pub fn wait_for_version(&self, target_version: u64, highest_li_version: Option<u64>) -> bool {
        let max_retries = 30;
        for _ in 0..max_retries {
            let state = block_on(self.client.as_ref().unwrap().get_state()).unwrap();
            if state.synced_version() == target_version {
                return highest_li_version.map_or(true, |highest_li_version| {
                    highest_li_version == state.committed_version()
                });
            }
            std::thread::sleep(std::time::Duration::from_millis(1000));
        }
        false
    }

    pub fn wait_until_initialized(&self) {
        block_on(self.client.as_ref().unwrap().wait_until_initialized()).unwrap();
    }
}

pub struct StateSyncEnvironment {
    network_conn_event_notifs_txs: HashMap<PeerId, conn_notifs_channel::Sender>,
    network_notifs_txs:
        HashMap<PeerId, diem_channel::Sender<(PeerId, ProtocolId), PeerManagerNotification>>,
    network_reqs_rxs:
        HashMap<PeerId, diem_channel::Receiver<(PeerId, ProtocolId), PeerManagerRequest>>,
    peers: Vec<RefCell<StateSyncPeer>>,
    runtime: Runtime,
}

impl StateSyncEnvironment {
    pub fn get_state_sync_peer(&self, index: usize) -> Ref<StateSyncPeer> {
        self.peers[index].borrow()
    }

    pub fn new(num_peers: usize) -> Self {
        ::diem_logger::Logger::init_for_testing();

        let (signers, public_keys, network_keys, network_addrs) = initial_setup(num_peers);

        let mut peers = vec![];
        for peer_index in 0..num_peers {
            let peer = StateSyncPeer {
                client: None,
                mempool: None,
                consensus_notifier: None,
                multi_peer_ids: HashMap::new(),
                network_addr: network_addrs[peer_index].clone(),
                network_key: network_keys[peer_index].clone(),
                peer_id: signers[peer_index].author(),
                public_key: public_keys[peer_index].clone(),
                signer: signers[peer_index].clone(),
                storage_proxy: None,
                bootstrapper: None,
            };
            peers.push(RefCell::new(peer));
        }

        Self {
            network_conn_event_notifs_txs: HashMap::new(),
            network_notifs_txs: HashMap::new(),
            network_reqs_rxs: HashMap::new(),
            peers,
            runtime: Runtime::new().unwrap(),
        }
    }

    // Starts a new state sync peer with the validator role.
    pub fn start_validator_peer(&mut self, peer_index: usize, mock_network: bool) {
        self.start_state_sync_peer(
            peer_index,
            default_handler(),
            RoleType::Validator,
            Waypoint::default(),
            mock_network,
        );
    }

    // Starts a new state sync peer with the fullnode role.
    pub fn start_fullnode_peer(&mut self, peer_index: usize, mock_network: bool) {
        self.start_state_sync_peer(
            peer_index,
            default_handler(),
            RoleType::FullNode,
            Waypoint::default(),
            mock_network,
        );
    }

    // Sets up and starts the state sync peer at the given node index.
    pub fn start_state_sync_peer(
        &mut self,
        index: usize,
        handler: MockRpcHandler,
        role: RoleType,
        waypoint: Waypoint,
        mock_network: bool,
    ) {
        self.setup_state_sync_peer(
            index,
            handler,
            role,
            waypoint,
            60_000,
            120_000,
            mock_network,
            false,
        );
    }

    pub fn setup_state_sync_peer(
        &mut self,
        index: usize,
        handler: MockRpcHandler,
        role: RoleType,
        waypoint: Waypoint,
        timeout_ms: u64,
        multicast_timeout_ms: u64,
        mock_network: bool,
        read_only_mode: bool,
    ) {
        let (config, network_id) = setup_state_sync_config(role, timeout_ms, multicast_timeout_ms);
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

        let mempool = MockSharedMempool::new();
        let mempool_notifier = mempool.mempool_notifier.clone();
        peer.mempool = Some(mempool);

        let (consensus_notifier, consensus_listener) =
            consensus_notifications::new_consensus_notifier_listener_pair(
                config.state_sync.client_commit_timeout_ms,
            );

        let bootstrapper = StateSyncBootstrapper::bootstrap_with_executor_proxy(
            Runtime::new().unwrap(),
            network_handles,
            mempool_notifier,
            consensus_listener,
            &config,
            waypoint,
            MockExecutorProxy::new(handler, storage_proxy.clone()),
            read_only_mode,
        );
        peer.client = Some(bootstrapper.create_client());
        peer.consensus_notifier = Some(consensus_notifier);
        peer.bootstrapper = Some(bootstrapper);
        peer.storage_proxy = Some(storage_proxy);
    }

    fn setup_network_handles(
        &mut self,
        index: usize,
        role: &RoleType,
        mock_network: bool,
        network_id: NetworkId,
    ) -> Vec<(NetworkId, StateSyncSender, StateSyncEvents)> {
        let mut network_handles = vec![];
        if mock_network {
            let networks = if role.is_validator() {
                vec![NetworkId::Validator, NetworkId::Vfn]
            } else {
                vec![NetworkId::Vfn, NetworkId::Public]
            };

            let mut peer = self.peers[index].borrow_mut();
            for network in networks.into_iter() {
                let peer_id = PeerId::random();
                peer.multi_peer_ids.insert(network, peer_id);

                // mock the StateSyncEvents and StateSyncSender to allow manually controlling
                // msg delivery in test
                let (network_reqs_tx, network_reqs_rx) =
                    diem_channel::new(QueueStyle::LIFO, 1, None);
                let (connection_reqs_tx, _) = diem_channel::new(QueueStyle::LIFO, 1, None);
                let (network_notifs_tx, network_notifs_rx) =
                    diem_channel::new(QueueStyle::LIFO, 1, None);
                let (conn_status_tx, conn_status_rx) = conn_notifs_channel::new();
                let network_sender = StateSyncSender::new(
                    PeerManagerRequestSender::new(network_reqs_tx),
                    ConnectionRequestSender::new(connection_reqs_tx),
                );
                let network_events = StateSyncEvents::new(network_notifs_rx, conn_status_rx);
                self.network_reqs_rxs.insert(peer_id, network_reqs_rx);
                self.network_notifs_txs.insert(peer_id, network_notifs_tx);
                self.network_conn_event_notifs_txs
                    .insert(peer_id, conn_status_tx);

                network_handles.push((network, network_sender, network_events));
            }
        } else {
            let peer = self.peers[index].borrow();
            let auth_mode = AuthenticationMode::Mutual(peer.network_key.clone());
            // TODO: This network is setup incorrectly, all nodes are on the Validator network ignoring the input
            let network_context = NetworkContext::new(*role, NetworkId::Validator, peer.peer_id);

            let seeds: HashMap<_, _> = self
                .peers
                .iter()
                .map(|peer| {
                    let peer = peer.borrow();
                    (
                        peer.peer_id,
                        Peer::from_addrs(PeerRole::Validator, vec![peer.network_addr.clone()]),
                    )
                })
                .collect();
            let trusted_peers = Arc::new(RwLock::new(HashMap::new()));

            // Recover the base address we bound previously.
            let addr_protos = peer.network_addr.as_slice();
            let (port, _suffix) = parse_memory(addr_protos).unwrap();
            let base_addr = NetworkAddress::from(Protocol::Memory(port));

            let mut network_builder = NetworkBuilder::new_for_test(
                ChainId::default(),
                seeds,
                trusted_peers,
                network_context,
                TimeService::real(),
                base_addr,
                auth_mode,
                PeerMetadataStorage::new(&[NetworkId::Validator]),
            );

            let (sender, events) =
                network_builder.add_p2p_service(&state_sync_v1::network::network_endpoint_config());
            network_builder.build(self.runtime.handle().clone()).start();
            network_handles.push((network_id, sender, events));
        };
        network_handles
    }

    /// Delivers next message from peer with index `sender` in this StateSyncEnvironment
    /// Returns the recipient of the msg
    pub fn deliver_msg(&mut self, sender_peer_id: PeerId) -> (PeerId, Message) {
        let network_reqs_rx = self.network_reqs_rxs.get_mut(&sender_peer_id).unwrap();
        let network_req = block_on(network_reqs_rx.next()).unwrap();

        // await next message from node
        if let PeerManagerRequest::SendDirectSend(receiver_id, msg) = network_req {
            let receiver_network_notif_tx = self.network_notifs_txs.get_mut(&receiver_id).unwrap();
            receiver_network_notif_tx
                .push(
                    (sender_peer_id, ProtocolId::StateSyncDirectSend),
                    PeerManagerNotification::RecvMessage(sender_peer_id, msg.clone()),
                )
                .unwrap();
            (receiver_id, msg)
        } else {
            panic!("received network request other than PeerManagerRequest");
        }
    }

    // checks that the `env_idx`th peer in this env sends no message to its `network_idx`th network
    pub fn assert_no_message_sent(&mut self, sender_peer_id: PeerId) {
        let network_reqs_rx = self.network_reqs_rxs.get_mut(&sender_peer_id).unwrap();
        assert!(network_reqs_rx.select_next_some().now_or_never().is_none());
    }

    pub fn clone_storage(&mut self, from_idx: usize, to_idx: usize) {
        let from_peer = self.peers[from_idx].borrow();
        let from_storage = from_peer.storage_proxy.as_ref().unwrap();
        let from_storage = from_storage.read();

        let to_peer = self.peers[to_idx].borrow();
        let to_storage = to_peer.storage_proxy.as_ref().unwrap();
        let mut to_storage = to_storage.write();
        let to_storage = to_storage.deref_mut();

        *to_storage = from_storage.clone();
    }

    pub fn send_peer_event(
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

pub fn default_handler() -> MockRpcHandler {
    Box::new(|resp| -> Result<TransactionListWithProof, Error> { Ok(resp) })
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

pub fn create_new_validator_set(
    validator_infos: Vec<ValidatorInfo>,
) -> (ValidatorSet, Vec<ValidatorSigner>) {
    let num_validators = validator_infos.len();
    let (signers, _) = random_validator_verifier(num_validators, None, true);
    let new_validator_infos = validator_infos
        .iter()
        .enumerate()
        .map(|(index, validator_info)| {
            ValidatorInfo::new(
                signers[index].author(),
                validator_info.consensus_voting_power(),
                validator_info.config().clone(),
            )
        })
        .collect::<Vec<ValidatorInfo>>();
    (ValidatorSet::new(new_validator_infos), signers)
}

fn setup_state_sync_config(
    role: RoleType,
    timeout_ms: u64,
    multicast_timeout_ms: u64,
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
        NetworkId::Vfn
    };

    if !role.is_validator() {
        config.full_node_networks = vec![config.validator_network.unwrap()];
        config.validator_network = None;
    }

    (config, network_id)
}

#[derive(Clone)]
pub struct MockStorage {
    // some mock transactions in the storage
    transactions: Vec<Transaction>,
    // the executed trees after applying the txns above.
    synced_trees: ExecutedTrees,
    // latest ledger info per epoch
    ledger_infos: HashMap<u64, LedgerInfoWithSignatures>,
    // latest epoch number (starts with 1)
    epoch_num: u64,
    // Validator signer of the latest epoch
    // All epochs are built s.t. a single signature is enough for quorum cert
    signer: ValidatorSigner,
    // A validator verifier of the latest epoch
    epoch_state: EpochState,
}

impl MockStorage {
    pub fn new(genesis_li: LedgerInfoWithSignatures, signer: ValidatorSigner) -> Self {
        let epoch_state = genesis_li.ledger_info().next_epoch_state().unwrap().clone();
        let epoch_num = genesis_li.ledger_info().epoch() + 1;
        let mut ledger_infos = HashMap::new();
        ledger_infos.insert(0, genesis_li);
        Self {
            transactions: vec![],
            synced_trees: ExecutedTrees::new_empty(),
            ledger_infos,
            epoch_num,
            signer,
            epoch_state,
        }
    }

    fn add_txns(&mut self, txns: &mut Vec<Transaction>) {
        self.transactions.append(txns);
        let num_leaves = self.transactions.len() + 1;
        let frozen_subtree_roots = vec![HashValue::zero(); num_leaves.count_ones() as usize];
        self.synced_trees = ExecutedTrees::new(
            HashValue::zero(), /* dummy_state_root */
            frozen_subtree_roots,
            num_leaves as u64,
        );
    }

    pub fn version(&self) -> u64 {
        self.transactions.len() as u64
    }

    pub fn synced_trees(&self) -> &ExecutedTrees {
        &self.synced_trees
    }

    pub fn epoch_num(&self) -> u64 {
        self.epoch_num
    }

    pub fn highest_local_li(&self) -> LedgerInfoWithSignatures {
        let cur_epoch = self.epoch_num();
        let epoch_with_li = if self.ledger_infos.contains_key(&cur_epoch) {
            cur_epoch
        } else {
            cur_epoch - 1
        };
        self.ledger_infos.get(&epoch_with_li).unwrap().clone()
    }

    pub fn get_local_storage_state(&self) -> SyncState {
        SyncState::new(
            self.highest_local_li(),
            self.synced_trees().clone(),
            self.epoch_state.clone(),
        )
    }

    pub fn get_epoch_changes(&self, known_epoch: u64) -> Result<LedgerInfoWithSignatures, Error> {
        match self.ledger_infos.get(&known_epoch) {
            None => Err(Error::UnexpectedError(
                "Mock storage missing epoch change ledger info!".into(),
            )),
            Some(li) => Ok(li.clone()),
        }
    }

    pub fn get_chunk(
        &self,
        start_version: u64,
        limit: u64,
        target_version: u64,
    ) -> Vec<Transaction> {
        let mut res = vec![];
        if target_version < start_version || start_version == 0 {
            return res;
        }
        let mut version = start_version;
        let limit = std::cmp::min(limit, target_version - start_version + 1);
        while version - 1 < self.transactions.len() as u64 && version - start_version < limit {
            res.push(self.transactions[(version - 1) as usize].clone());
            version += 1;
        }
        res
    }

    pub fn add_txns_with_li(
        &mut self,
        mut transactions: Vec<Transaction>,
        verified_target_li: LedgerInfoWithSignatures,
        intermediate_end_of_epoch_li: Option<LedgerInfoWithSignatures>,
    ) {
        self.add_txns(&mut transactions);
        if let Some(li) = intermediate_end_of_epoch_li {
            self.epoch_num = li.ledger_info().epoch() + 1;
            self.ledger_infos.insert(li.ledger_info().epoch(), li);
            return;
        }
        if verified_target_li.ledger_info().epoch() != self.epoch_num() {
            return;
        }

        // store ledger info only if version matches last tx
        if verified_target_li.ledger_info().version() == self.version() {
            self.ledger_infos.insert(
                verified_target_li.ledger_info().epoch(),
                verified_target_li.clone(),
            );
            if let Some(next_epoch_state) = verified_target_li.ledger_info().next_epoch_state() {
                self.epoch_num = next_epoch_state.epoch;
                self.epoch_state = next_epoch_state.clone();
            }
        }
    }

    // Generate new dummy txns and updates the LI
    // with the version corresponding to the new transactions, signed by this storage signer.
    pub fn commit_new_txns(&mut self, num_txns: u64) -> (Vec<Transaction>, Vec<SignedTransaction>) {
        let mut committed_txns = vec![];
        let mut signed_txns = vec![];
        for _ in 0..num_txns {
            let txn = Self::gen_mock_user_txn();
            self.add_txns(&mut vec![txn.clone()]);
            committed_txns.push(txn.clone());
            if let Transaction::UserTransaction(signed_txn) = txn {
                signed_txns.push(signed_txn);
            }
        }
        self.add_li(None);
        (committed_txns, signed_txns)
    }

    fn gen_mock_user_txn() -> Transaction {
        let sender = AccountAddress::random();
        let receiver = AuthenticationKey::random();
        let program = encode_peer_to_peer_with_metadata_script(
            xus_tag(),
            receiver.derived_address(),
            1,
            vec![],
            vec![],
        );
        Transaction::UserTransaction(get_test_signed_txn(
            sender,
            0, // sequence number
            &GENESIS_KEYPAIR.0,
            GENESIS_KEYPAIR.1.clone(),
            Some(TransactionPayload::Script(program)),
        ))
    }

    // add the LI to the current highest version and sign it
    fn add_li(&mut self, validator_set: Option<ValidatorSet>) {
        let epoch_state = validator_set.map(|set| EpochState {
            epoch: self.epoch_num() + 1,
            verifier: (&set).into(),
        });
        let ledger_info = LedgerInfo::new(
            BlockInfo::new(
                self.epoch_num(),
                self.version(),
                HashValue::zero(),
                HashValue::zero(),
                self.version(),
                0,
                epoch_state,
            ),
            HashValue::zero(),
        );
        let signature = self.signer.sign(&ledger_info);
        let mut signatures = BTreeMap::new();
        signatures.insert(self.signer.author(), signature);
        self.ledger_infos.insert(
            self.epoch_num(),
            LedgerInfoWithSignatures::new(ledger_info, signatures),
        );
    }

    // This function is applying the LedgerInfo with the next epoch info to the existing version
    // (yes, it's different from reality, we're not adding any real reconfiguration txn,
    // just adding a new LedgerInfo).
    // The validator set is different only in the consensus public / private keys, network data
    // remains the same.
    pub fn move_to_next_epoch(&mut self, signer: ValidatorSigner, validator_set: ValidatorSet) {
        self.add_li(Some(validator_set));
        self.epoch_num += 1;
        self.signer = signer;
        self.epoch_state = self
            .highest_local_li()
            .ledger_info()
            .next_epoch_state()
            .unwrap()
            .clone();
    }

    // Find LedgerInfo for an epoch boundary version.
    pub fn get_epoch_ending_ledger_info(
        &self,
        version: u64,
    ) -> Result<LedgerInfoWithSignatures, Error> {
        for li in self.ledger_infos.values() {
            if li.ledger_info().version() == version && li.ledger_info().ends_epoch() {
                return Ok(li.clone());
            }
        }
        Err(Error::UnexpectedError(format!(
            "No ledger info found for version: {:?}",
            version
        )))
    }
}

pub type MockRpcHandler = Box<
    dyn Fn(TransactionListWithProof) -> Result<TransactionListWithProof, Error>
        + Send
        + Sync
        + 'static,
>;

pub struct MockExecutorProxy {
    handler: MockRpcHandler,
    storage: Arc<RwLock<MockStorage>>,
}

impl MockExecutorProxy {
    pub fn new(handler: MockRpcHandler, storage: Arc<RwLock<MockStorage>>) -> Self {
        Self { handler, storage }
    }
}

impl ExecutorProxyTrait for MockExecutorProxy {
    fn get_local_storage_state(&self) -> Result<SyncState, Error> {
        Ok(self.storage.read().get_local_storage_state())
    }

    fn execute_chunk(
        &mut self,
        txn_list_with_proof: TransactionListWithProof,
        ledger_info_with_sigs: LedgerInfoWithSignatures,
        intermediate_end_of_epoch_li: Option<LedgerInfoWithSignatures>,
    ) -> Result<(), Error> {
        self.storage.write().add_txns_with_li(
            txn_list_with_proof.transactions,
            ledger_info_with_sigs,
            intermediate_end_of_epoch_li,
        );
        Ok(())
    }

    fn get_chunk(
        &self,
        known_version: u64,
        limit: u64,
        target_version: u64,
    ) -> Result<TransactionListWithProof, Error> {
        let start_version = known_version
            .checked_add(1)
            .ok_or_else(|| Error::IntegerOverflow("Known version has overflown!".into()))?;
        let txns = self
            .storage
            .read()
            .get_chunk(start_version, limit, target_version);
        let first_txn_version = txns.first().map(|_| start_version);
        let txns_with_proof = TransactionListWithProof::new(
            txns,
            None,
            first_txn_version,
            TransactionInfoListWithProof::new_empty(),
        );
        (self.handler)(txns_with_proof)
    }

    fn get_epoch_change_ledger_info(&self, epoch: u64) -> Result<LedgerInfoWithSignatures, Error> {
        self.storage.read().get_epoch_changes(epoch)
    }

    fn get_epoch_ending_ledger_info(
        &self,
        version: u64,
    ) -> Result<LedgerInfoWithSignatures, Error> {
        self.storage.read().get_epoch_ending_ledger_info(version)
    }

    fn get_version_timestamp(&self, _version: u64) -> Result<u64, Error> {
        // Only used for logging purposes so no point in mocking
        Ok(0)
    }

    fn publish_event_notifications(&mut self, _events: Vec<ContractEvent>) -> Result<(), Error> {
        Ok(())
    }
}
