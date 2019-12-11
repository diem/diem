// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::tests::mock_storage::MockStorage;
use crate::{
    executor_proxy::ExecutorProxyTrait, PeerId, StateSyncClient, StateSynchronizer,
    SynchronizerState,
};
use anyhow::{bail, Result};
use config_builder;
use executor::ExecutedTrees;
use futures::{executor::block_on, future::FutureExt, Future};
use libra_config::config::RoleType;
use libra_crypto::x25519::X25519StaticPublicKey;
use libra_crypto::{ed25519::*, test_utils::TEST_SEED, x25519, HashValue};
use libra_logger::set_simple_logger;
use libra_types::block_info::BlockInfo;
use libra_types::crypto_proxies::ValidatorPublicKeys;
use libra_types::crypto_proxies::ValidatorSet;
use libra_types::crypto_proxies::{
    random_validator_verifier, ValidatorChangeEventWithProof, ValidatorSigner,
};
use libra_types::{
    crypto_proxies::LedgerInfoWithSignatures, ledger_info::LedgerInfo, proof::TransactionListProof,
    transaction::TransactionListWithProof,
};
use network::{
    validator_network::{
        network_builder::{NetworkBuilder, TransportType},
        STATE_SYNCHRONIZER_DIRECT_SEND_PROTOCOL,
    },
    NetworkPublicKeys, ProtocolId,
};
use parity_multiaddr::Multiaddr;
use rand::{rngs::StdRng, SeedableRng};
use std::sync::RwLock;
use std::{
    collections::{BTreeMap, HashMap},
    pin::Pin,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
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
    fn get_local_storage_state(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<SynchronizerState>> + Send>> {
        let state = self.storage.read().unwrap().get_local_storage_state();
        async move { Ok(state) }.boxed()
    }

    fn execute_chunk(
        &self,
        txn_list_with_proof: TransactionListWithProof,
        ledger_info_with_sigs: LedgerInfoWithSignatures,
        _synced_trees: &mut ExecutedTrees,
    ) -> Result<()> {
        self.storage
            .write()
            .unwrap()
            .add_txns_with_li(txn_list_with_proof.transactions, ledger_info_with_sigs);
        Ok(())
    }

    fn get_chunk(
        &self,
        known_version: u64,
        limit: u64,
        target_version: u64,
    ) -> Pin<Box<dyn Future<Output = Result<TransactionListWithProof>> + Send>> {
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
        let response = (self.handler)(txns_with_proof);
        async move { response }.boxed()
    }

    fn get_epoch_proof(
        &self,
        start_epoch: u64,
        _end_epoch: u64,
    ) -> Result<ValidatorChangeEventWithProof> {
        Ok(self.storage.read().unwrap().get_epoch_changes(start_epoch))
    }
}

struct SynchronizerEnv {
    _runtime: Runtime,
    _synchronizers: Vec<StateSynchronizer>,
    clients: Vec<Arc<StateSyncClient>>,
    storage_proxies: Vec<Arc<RwLock<MockStorage>>>, // to directly modify peers storage
    network_signing_public_keys: Vec<Ed25519PublicKey>,
    network_identity_public_keys: Vec<X25519StaticPublicKey>,
}

impl SynchronizerEnv {
    // Returns the initial peers with their signatures
    fn initial_setup() -> (
        Vec<ValidatorSigner>,
        Vec<Ed25519PrivateKey>,
        Vec<ValidatorPublicKeys>,
    ) {
        let (signers, _verifier) = random_validator_verifier(2, None, true);

        // Setup signing public keys.
        let mut rng = StdRng::from_seed(TEST_SEED);
        let (a_signing_private_key, a_signing_public_key) = compat::generate_keypair(&mut rng);
        let (b_signing_private_key, b_signing_public_key) = compat::generate_keypair(&mut rng);
        // Setup identity public keys.
        let (_a_identity_private_key, a_identity_public_key) =
            x25519::compat::generate_keypair(&mut rng);
        let (_b_identity_private_key, b_identity_public_key) =
            x25519::compat::generate_keypair(&mut rng);

        // The voting power of peer 1 is enough to generate an LI that passes validation.
        let validators_keys = vec![
            ValidatorPublicKeys::new(
                signers[0].author(),
                signers[0].public_key(),
                1,
                a_signing_public_key.clone(),
                a_identity_public_key.clone(),
            ),
            ValidatorPublicKeys::new(
                signers[1].author(),
                signers[1].public_key(),
                3,
                b_signing_public_key.clone(),
                b_identity_public_key.clone(),
            ),
        ];
        (
            signers,
            vec![a_signing_private_key, b_signing_private_key],
            validators_keys,
        )
    }

    fn genesis_li(validators: &[ValidatorPublicKeys]) -> LedgerInfoWithSignatures {
        LedgerInfoWithSignatures::new(
            LedgerInfo::new(
                BlockInfo::new(
                    0,
                    0,
                    HashValue::zero(),
                    HashValue::zero(),
                    0,
                    0,
                    Some(ValidatorSet::new(validators.to_owned())),
                ),
                HashValue::zero(),
            ),
            BTreeMap::new(),
        )
    }

    fn new(handler: MockRpcHandler, role: RoleType) -> Self {
        set_simple_logger("state-sync");
        let runtime = Runtime::new().unwrap();
        let (signers, network_signers, public_keys) = Self::initial_setup();
        let peers = signers.iter().map(|s| s.author()).collect::<Vec<PeerId>>();
        let network_signing_public_keys = public_keys
            .iter()
            .map(|pk| pk.network_signing_public_key().clone())
            .collect::<Vec<Ed25519PublicKey>>();
        let network_identity_public_keys = public_keys
            .iter()
            .map(|pk| pk.network_identity_public_key().clone())
            .collect::<Vec<X25519StaticPublicKey>>();

        // setup network
        let addr: Multiaddr = "/memory/0".parse().unwrap();
        let protocols = vec![ProtocolId::from_static(
            STATE_SYNCHRONIZER_DIRECT_SEND_PROTOCOL,
        )];

        let trusted_peers: HashMap<_, _> = public_keys
            .iter()
            .map(|public_keys| {
                (
                    *public_keys.account_address(),
                    NetworkPublicKeys {
                        signing_public_key: public_keys.network_signing_public_key().clone(),
                        identity_public_key: public_keys.network_identity_public_key().clone(),
                    },
                )
            })
            .collect();

        let (listener_addr, mut network_provider) = NetworkBuilder::new(
            runtime.handle().clone(),
            peers[1],
            addr.clone(),
            RoleType::Validator,
        )
        .signing_keys((
            network_signers[1].clone(),
            public_keys[1].network_signing_public_key().clone(),
        ))
        .trusted_peers(trusted_peers.clone())
        .transport(TransportType::Memory)
        .direct_send_protocols(protocols.clone())
        .build();
        let (sender_b, events_b) = network_provider.add_state_synchronizer(protocols.clone());
        runtime.handle().spawn(network_provider.start());

        let (_dialer_addr, mut network_provider) = NetworkBuilder::new(
            runtime.handle().clone(),
            peers[0],
            addr.clone(),
            RoleType::Validator,
        )
        .transport(TransportType::Memory)
        .signing_keys((
            network_signers[0].clone(),
            public_keys[0].network_signing_public_key().clone(),
        ))
        .trusted_peers(trusted_peers.clone())
        .seed_peers([(peers[1], vec![listener_addr])].iter().cloned().collect())
        .direct_send_protocols(protocols.clone())
        .build();
        let (sender_a, events_a) = network_provider.add_state_synchronizer(protocols);
        runtime.handle().spawn(network_provider.start());

        // create synchronizers
        let mut config = config_builder::test_config().0;
        if !role.is_validator() {
            let mut network = config.validator_network.unwrap();
            network.set_default_peer_id();
            config.full_node_networks = vec![network];
            config.validator_network = None;
        }
        config.base.role = role;
        config
            .state_sync
            .upstream_peers
            .upstream_peers
            .push(peers[1]);

        let genesis_li = Self::genesis_li(&public_keys);
        let storage_proxies = vec![
            Arc::new(RwLock::new(MockStorage::new(
                genesis_li.clone(),
                signers[0].clone(),
            ))),
            Arc::new(RwLock::new(MockStorage::new(
                genesis_li.clone(),
                signers[1].clone(),
            ))),
        ];
        let synchronizers: Vec<StateSynchronizer> = vec![
            StateSynchronizer::bootstrap_with_executor_proxy(
                vec![(sender_a, events_a)],
                role,
                &config.state_sync,
                MockExecutorProxy::new(Self::default_handler(), storage_proxies[0].clone()),
            ),
            StateSynchronizer::bootstrap_with_executor_proxy(
                vec![(sender_b, events_b)],
                role,
                &config_builder::test_config().0.state_sync,
                MockExecutorProxy::new(handler, storage_proxies[1].clone()),
            ),
        ];
        let clients = synchronizers.iter().map(|s| s.create_client()).collect();

        Self {
            clients,
            _synchronizers: synchronizers,
            _runtime: runtime,
            storage_proxies,
            network_signing_public_keys,
            network_identity_public_keys,
        }
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
        storage.commit_new_txns(num_txns);
        block_on(self.clients[peer_id].commit()).unwrap();
    }

    fn latest_li(&self, peer_id: usize) -> LedgerInfoWithSignatures {
        self.storage_proxies[peer_id]
            .read()
            .unwrap()
            .highest_local_li()
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

    // Moves peer 1 to the next epoch. Note that peer 0 is not going to be able to discover its new
    // signer: it'll learn about the new epoch public keys through state synchronization, but the
    // private keys are supposed to be discovered separately.
    pub fn move_to_next_epoch(&self) {
        let (signers, _verifier) = random_validator_verifier(2, None, true);
        let new_keys = vec![
            ValidatorPublicKeys::new(
                signers[0].author(),
                signers[0].public_key(),
                1,
                self.network_signing_public_keys[0].clone(),
                self.network_identity_public_keys[0].clone(),
            ),
            ValidatorPublicKeys::new(
                signers[1].author(),
                signers[1].public_key(),
                3,
                self.network_signing_public_keys[1].clone(),
                self.network_identity_public_keys[1].clone(),
            ),
        ];
        let validator_set = ValidatorSet::new(new_keys);
        self.storage_proxies[1]
            .write()
            .unwrap()
            .move_to_next_epoch(signers[1].clone(), validator_set);
    }
}

#[test]
fn test_basic_catch_up() {
    let env = SynchronizerEnv::new(SynchronizerEnv::default_handler(), RoleType::Validator);

    // test small sequential syncs
    for version in 1..5 {
        env.commit(1, version);
        let target_li = env.latest_li(1);
        env.sync_to(0, target_li);
        assert_eq!(env.latest_li(0).ledger_info().version(), version);
    }
    // test batch sync for multiple transactions
    env.commit(1, 20);
    env.sync_to(0, env.latest_li(1));
    assert_eq!(env.latest_li(0).ledger_info().version(), 20);

    // test batch sync for multiple chunks
    env.commit(1, 2000);
    env.sync_to(0, env.latest_li(1));
    assert_eq!(env.latest_li(0).ledger_info().version(), 2000);
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
    let env = SynchronizerEnv::new(handler, RoleType::Validator);
    env.commit(1, 20);
    env.sync_to(0, env.latest_li(1));
    assert_eq!(env.latest_li(0).ledger_info().version(), 20);
}

#[test]
fn test_full_node() {
    let env = SynchronizerEnv::new(SynchronizerEnv::default_handler(), RoleType::FullNode);
    env.commit(1, 10);
    // first sync should be fulfilled immediately after peer discovery
    assert!(env.wait_for_version(0, 10));
    env.commit(1, 20);
    // second sync will be done via long polling cause first node should send new request
    // after receiving first chunk immediately
    assert!(env.wait_for_version(0, 20));
}

#[test]
fn catch_up_through_epochs_validators() {
    let env = SynchronizerEnv::new(SynchronizerEnv::default_handler(), RoleType::Validator);

    // catch up to the next epoch starting from the middle of the current one
    env.commit(1, 20);
    env.sync_to(0, env.latest_li(1));
    env.commit(1, 40);
    env.move_to_next_epoch();
    env.commit(1, 100);
    env.sync_to(0, env.latest_li(1));
    assert_eq!(env.latest_li(0).ledger_info().version(), 100);
    assert_eq!(env.latest_li(0).ledger_info().epoch(), 2);

    // catch up through multiple epochs
    for epoch in 2..10 {
        env.commit(1, epoch * 100);
        env.move_to_next_epoch();
    }
    env.commit(1, 950); // At this point peer 1 is at epoch 10 and version 950
    env.sync_to(0, env.latest_li(1));
    assert_eq!(env.latest_li(0).ledger_info().version(), 950);
    assert_eq!(env.latest_li(0).ledger_info().epoch(), 10);
}

#[test]
fn catch_up_through_epochs_full_node() {
    let env = SynchronizerEnv::new(SynchronizerEnv::default_handler(), RoleType::FullNode);
    // catch up through multiple epochs
    for epoch in 1..10 {
        env.commit(1, epoch * 100);
        env.move_to_next_epoch();
    }
    env.commit(1, 950); // At this point peer 1 is at epoch 10 and version 950

    assert!(env.wait_for_version(0, 950));
    assert_eq!(env.latest_li(0).ledger_info().epoch(), 10);
}
