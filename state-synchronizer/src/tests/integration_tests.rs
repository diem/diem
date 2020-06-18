// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    executor_proxy::ExecutorProxyTrait, tests::mock_storage::MockStorage, PeerId, StateSyncClient,
    StateSynchronizer, SynchronizerState,
};
use anyhow::{bail, Result};
use executor_types::ExecutedTrees;
use futures::executor::block_on;
use libra_config::{
    chain_id::ChainId,
    config::{PeerNetworkId, RoleType},
    network_id::NetworkId,
};
use libra_crypto::{hash::ACCUMULATOR_PLACEHOLDER_HASH, test_utils::TEST_SEED, x25519, Uniform};
use libra_mempool::mocks::MockSharedMempool;
use libra_network_address::{NetworkAddress, RawNetworkAddress};
use libra_types::{
    contract_event::ContractEvent, ledger_info::LedgerInfoWithSignatures,
    on_chain_config::ValidatorSet, proof::TransactionListProof,
    transaction::TransactionListWithProof, validator_config::ValidatorConfig,
    validator_info::ValidatorInfo, validator_signer::ValidatorSigner,
    validator_verifier::random_validator_verifier, waypoint::Waypoint,
};
use network_builder::builder::{AuthenticationMode, NetworkBuilder};
use rand::{rngs::StdRng, SeedableRng};
use std::{
    collections::HashMap,
    convert::TryFrom,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, RwLock,
    },
};
use tokio::runtime::Runtime;

type MockRpcHandler = Box<
    dyn Fn(TransactionListWithProof) -> Result<TransactionListWithProof> + Send + Sync + 'static,
>;

pub struct MockExecutorProxy {
    handler: MockRpcHandler,
    storage: Arc<RwLock<MockStorage>>,
}

impl MockExecutorProxy {
    fn new(handler: MockRpcHandler, storage: Arc<RwLock<MockStorage>>) -> Self {
        Self { handler, storage }
    }
}

impl ExecutorProxyTrait for MockExecutorProxy {
    fn get_local_storage_state(&self) -> Result<SynchronizerState> {
        Ok(self.storage.read().unwrap().get_local_storage_state())
    }

    fn execute_chunk(
        &mut self,
        txn_list_with_proof: TransactionListWithProof,
        ledger_info_with_sigs: LedgerInfoWithSignatures,
        intermediate_end_of_epoch_li: Option<LedgerInfoWithSignatures>,
        _synced_trees: &mut ExecutedTrees,
    ) -> Result<()> {
        self.storage.write().unwrap().add_txns_with_li(
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
    ) -> Result<TransactionListWithProof> {
        let txns = self
            .storage
            .read()
            .unwrap()
            .get_chunk(known_version + 1, limit, target_version);
        let first_txn_version = txns.first().map(|_| known_version + 1);
        let txns_with_proof = TransactionListWithProof::new(
            txns,
            None,
            first_txn_version,
            TransactionListProof::new_empty(),
        );
        (self.handler)(txns_with_proof)
    }

    fn get_epoch_proof(&self, epoch: u64) -> Result<LedgerInfoWithSignatures> {
        Ok(self.storage.read().unwrap().get_epoch_changes(epoch))
    }

    fn get_epoch_ending_ledger_info(&self, version: u64) -> Result<LedgerInfoWithSignatures> {
        self.storage
            .read()
            .unwrap()
            .get_epoch_ending_ledger_info(version)
    }

    fn load_on_chain_configs(&mut self) -> Result<()> {
        Ok(())
    }

    fn publish_on_chain_config_updates(&mut self, _events: Vec<ContractEvent>) -> Result<()> {
        Ok(())
    }
}

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
}

impl SynchronizerEnv {
    // Returns the initial peers with their signatures
    fn initial_setup(
        count: usize,
    ) -> (
        Vec<ValidatorSigner>,
        Vec<ValidatorInfo>,
        Vec<x25519::PrivateKey>,
    ) {
        let (signers, _verifier) = random_validator_verifier(count, None, true);

        // Setup identity public keys.
        let mut rng = StdRng::from_seed(TEST_SEED);
        let network_keys: Vec<_> = (0..count)
            .map(|_| x25519::PrivateKey::generate(&mut rng))
            .collect();

        let mut validators_keys = vec![];
        // The voting power of peer 0 is enough to generate an LI that passes validation.
        for (idx, signer) in signers.iter().enumerate() {
            let voting_power = if idx == 0 { 1000 } else { 1 };
            let addr: NetworkAddress = "/memory/0".parse().unwrap();
            let validator_config = ValidatorConfig::new(
                signer.public_key(),
                network_keys[idx].public_key(),
                RawNetworkAddress::try_from(&addr).unwrap(),
                network_keys[idx].public_key(),
                RawNetworkAddress::try_from(&addr).unwrap(),
                // Vec::<AccountAddress>::new(),
            );
            let validator_info =
                ValidatorInfo::new(signer.author(), voting_power, validator_config);
            validators_keys.push(validator_info);
        }
        (signers, validators_keys, network_keys)
    }

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

    fn genesis_li(validators: &[ValidatorInfo]) -> LedgerInfoWithSignatures {
        LedgerInfoWithSignatures::genesis(
            *ACCUMULATOR_PLACEHOLDER_HASH,
            ValidatorSet::new(validators.to_vec()),
        )
    }

    fn new(num_peers: usize) -> Self {
        ::libra_logger::Logger::new().environment_only(true).init();
        let runtime = Runtime::new().unwrap();
        let (signers, public_keys, network_keys) = Self::initial_setup(num_peers);
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
        }
    }

    fn start_next_synchronizer(
        &mut self,
        handler: MockRpcHandler,
        role: RoleType,
        waypoint: Option<Waypoint>,
    ) {
        self.setup_next_synchronizer(handler, role, waypoint, 60_000);
    }

    fn setup_next_synchronizer(
        &mut self,
        handler: MockRpcHandler,
        role: RoleType,
        waypoint: Option<Waypoint>,
        timeout_ms: u64,
    ) {
        let new_peer_idx = self.synchronizers.len();
        let trusted_peers: HashMap<_, _> = self
            .public_keys
            .iter()
            .map(|public_keys| {
                (
                    *public_keys.account_address(),
                    public_keys.network_identity_public_key(),
                )
            })
            .collect();

        // setup network
        let addr: NetworkAddress = "/memory/0".parse().unwrap();
        let mut seed_peers = HashMap::new();
        if new_peer_idx > 0 {
            seed_peers.insert(
                self.peer_ids[new_peer_idx - 1],
                vec![self.peer_addresses[new_peer_idx - 1].clone()],
            );
        }
        let mut network_builder = NetworkBuilder::new(
            self.runtime.handle().clone(),
            ChainId::default(),
            self.network_id.clone(),
            RoleType::Validator,
            self.peer_ids[new_peer_idx],
            addr,
        );
        network_builder
            .authentication_mode(AuthenticationMode::Mutual(
                self.network_keys[new_peer_idx].clone(),
            ))
            .trusted_peers(trusted_peers)
            .seed_peers(seed_peers)
            .add_connectivity_manager()
            .add_gossip_discovery();

        let (sender, events) =
            network_builder.add_protocol_handler(crate::network::network_endpoint_config());
        let peer_addr = network_builder.build();

        let mut config = config_builder::test_config().0;
        let network = config.validator_network.unwrap();
        let network_id = network.peer_id();
        if !role.is_validator() {
            config.full_node_networks = vec![network];
            config.validator_network = None;
        }
        config.base.role = role;
        config.state_sync.sync_request_timeout_ms = timeout_ms;
        if new_peer_idx > 0 {
            // set the upstream peer in the config
            let upstream_peer = PeerNetworkId(network_id, self.peer_ids[new_peer_idx - 1]);
            config.upstream.upstream_peers.insert(upstream_peer);
        }

        let genesis_li = Self::genesis_li(&self.public_keys);
        let storage_proxy = Arc::new(RwLock::new(MockStorage::new(
            genesis_li,
            self.signers[new_peer_idx].clone(),
        )));
        let (mempool_channel, mempool_requests) = futures::channel::mpsc::channel(1_024);
        let synchronizer = StateSynchronizer::bootstrap_with_executor_proxy(
            Runtime::new().unwrap(),
            vec![(network_id, sender, events)],
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
        self.peer_addresses.push(peer_addr);
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

    fn wait_for_version(&self, peer_id: usize, target_version: u64) -> bool {
        let max_retries = 30;
        for _ in 0..max_retries {
            let state = block_on(self.clients[peer_id].get_state()).unwrap();
            if state.synced_trees.version().unwrap_or(0) == target_version {
                return true;
            }
            std::thread::sleep(std::time::Duration::from_millis(1000));
        }
        false
    }

    fn wait_until_initialized(&self, peer_id: usize) -> Result<()> {
        block_on(self.synchronizers[peer_id].wait_until_initialized())
    }
}

#[test]
fn test_basic_catch_up() {
    let mut env = SynchronizerEnv::new(2);
    env.start_next_synchronizer(
        SynchronizerEnv::default_handler(),
        RoleType::Validator,
        None,
    );
    env.start_next_synchronizer(
        SynchronizerEnv::default_handler(),
        RoleType::Validator,
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
        None,
    );
    env.start_next_synchronizer(handler, RoleType::Validator, None);
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
    env.start_next_synchronizer(handler, RoleType::Validator, None);
    env.setup_next_synchronizer(
        SynchronizerEnv::default_handler(),
        RoleType::Validator,
        None,
        100,
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
        None,
    );
    env.start_next_synchronizer(SynchronizerEnv::default_handler(), RoleType::FullNode, None);
    env.commit(0, 10);
    // first sync should be fulfilled immediately after peer discovery
    assert!(env.wait_for_version(1, 10));
    env.commit(0, 20);
    // second sync will be done via long polling cause first node should send new request
    // after receiving first chunk immediately
    assert!(env.wait_for_version(1, 20));
}

#[test]
fn catch_up_through_epochs_validators() {
    let mut env = SynchronizerEnv::new(2);
    env.start_next_synchronizer(
        SynchronizerEnv::default_handler(),
        RoleType::Validator,
        None,
    );
    env.start_next_synchronizer(
        SynchronizerEnv::default_handler(),
        RoleType::Validator,
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
        None,
    );
    // catch up through multiple epochs
    for epoch in 1..10 {
        env.commit(0, epoch * 100);
        env.move_to_next_epoch();
    }
    env.commit(0, 950); // At this point peer 0 is at epoch 10 and version 950

    env.start_next_synchronizer(SynchronizerEnv::default_handler(), RoleType::FullNode, None);
    assert!(env.wait_for_version(1, 950));
    assert_eq!(env.latest_li(1).ledger_info().epoch(), 10);

    // Peer 2 has peer 1 as its upstream, should catch up from it.
    env.start_next_synchronizer(SynchronizerEnv::default_handler(), RoleType::FullNode, None);
    assert!(env.wait_for_version(2, 950));
    assert_eq!(env.latest_li(2).ledger_info().epoch(), 10);
}

#[test]
fn catch_up_with_waypoints() {
    let mut env = SynchronizerEnv::new(3);
    env.start_next_synchronizer(
        SynchronizerEnv::default_handler(),
        RoleType::Validator,
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
        Some(waypoint),
    );
    env.wait_until_initialized(1).unwrap();
    assert!(env.latest_li(1).ledger_info().version() >= 700);
    assert!(env.latest_li(1).ledger_info().epoch() >= 7);

    // Once caught up with the waypoint peer 1 continues with the regular state sync
    assert!(env.wait_for_version(1, 950));
    assert_eq!(env.latest_li(1).ledger_info().epoch(), 10);

    // Peer 2 has peer 1 as its upstream, should catch up from it.
    env.start_next_synchronizer(SynchronizerEnv::default_handler(), RoleType::FullNode, None);
    assert!(env.wait_for_version(2, 950));
    assert_eq!(env.latest_li(2).ledger_info().epoch(), 10);
}
