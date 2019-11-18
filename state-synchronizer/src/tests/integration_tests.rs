// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    executor_proxy::ExecutorProxyTrait, PeerId, StateSyncClient, StateSynchronizer,
    SynchronizerState,
};
use config_builder::util::get_test_config;
use failure::{prelude::*, Result};
use futures::{executor::block_on, future::FutureExt, Future};
use libra_config::config::RoleType;
use libra_crypto::{
    ed25519::*, test_utils::TEST_SEED, traits::Genesis, x25519, HashValue, SigningKey,
};
use libra_types::block_info::BlockInfo;
use libra_types::crypto_proxies::ValidatorChangeEventWithProof;
use libra_types::{
    account_address::AccountAddress,
    crypto_proxies::LedgerInfoWithSignatures,
    ledger_info::LedgerInfo as TypesLedgerInfo,
    proof::TransactionListProof,
    test_helpers::transaction_test_helpers::get_test_signed_txn,
    transaction::{Transaction, TransactionListWithProof},
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
use tokio::runtime::{Builder, Runtime};
use transaction_builder::encode_transfer_script;
use vm_genesis::GENESIS_KEYPAIR;

type MockRpcHandler = Box<
    dyn Fn(TransactionListWithProof) -> Result<TransactionListWithProof> + Send + Sync + 'static,
>;

// To play with the storage values
pub struct MockStorage {
    version: u64,
}

impl MockStorage {
    fn new(version: u64) -> Self {
        Self { version }
    }

    fn commit(&mut self, val: u64) {
        self.version = std::cmp::max(self.version, val);
    }
}

pub struct MockExecutorProxy {
    peer_id: PeerId,
    handler: MockRpcHandler,
    storage: Arc<RwLock<MockStorage>>,
}

impl MockExecutorProxy {
    fn new(peer_id: PeerId, handler: MockRpcHandler, storage: Arc<RwLock<MockStorage>>) -> Self {
        Self {
            peer_id,
            handler,
            storage,
        }
    }

    fn mock_ledger_info(peer_id: PeerId, version: u64) -> LedgerInfoWithSignatures {
        let ledger_info = TypesLedgerInfo::new(
            BlockInfo::new(0, 0, HashValue::zero(), HashValue::zero(), version, 0, None),
            HashValue::zero(),
        );
        let mut signatures = BTreeMap::new();
        let private_key = Ed25519PrivateKey::genesis();
        let signature = private_key.sign_message(&HashValue::zero());
        signatures.insert(peer_id, signature);
        LedgerInfoWithSignatures::new(ledger_info, signatures)
    }

    fn mock_chunk_response(&self, version: u64) -> TransactionListWithProof {
        let sender = AccountAddress::from_public_key(&GENESIS_KEYPAIR.1);
        let receiver = AccountAddress::new([0xff; 32]);
        let program = encode_transfer_script(&receiver, 1);
        let transaction = Transaction::UserTransaction(get_test_signed_txn(
            sender,
            version + 1,
            GENESIS_KEYPAIR.0.clone(),
            GENESIS_KEYPAIR.1.clone(),
            Some(program),
        ));

        let proof = TransactionListProof::new_empty();
        TransactionListWithProof::new(vec![transaction], None, Some(version + 1), proof)
    }
}

impl ExecutorProxyTrait for MockExecutorProxy {
    fn get_local_storage_state(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<SynchronizerState>> + Send>> {
        let highest_synced_version = self.storage.read().unwrap().version;
        let highest_local_li = Self::mock_ledger_info(self.peer_id, highest_synced_version);
        async move {
            Ok(SynchronizerState {
                highest_local_li,
                highest_synced_version,
            })
        }
            .boxed()
    }

    fn execute_chunk(
        &self,
        _txn_list_with_proof: TransactionListWithProof,
        ledger_info_with_sigs: LedgerInfoWithSignatures,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> {
        let version = ledger_info_with_sigs.ledger_info().version();
        self.storage.write().unwrap().version = version;
        async move { Ok(()) }.boxed()
    }

    fn get_chunk(
        &self,
        known_version: u64,
        _: u64,
        _: u64,
    ) -> Pin<Box<dyn Future<Output = Result<TransactionListWithProof>> + Send>> {
        let response = (self.handler)(self.mock_chunk_response(known_version));
        async move { response }.boxed()
    }

    fn validate_ledger_info(&self, _target: &LedgerInfoWithSignatures) -> Result<()> {
        Ok(())
    }

    fn get_epoch_proof(&self, _start_epoch: u64) -> Result<ValidatorChangeEventWithProof> {
        unimplemented!("get epoch proof not supported for mock executor proxy");
    }
}

struct SynchronizerEnv {
    _runtime: Runtime,
    _synchronizers: Vec<StateSynchronizer>,
    peers: Vec<PeerId>,
    clients: Vec<Arc<StateSyncClient>>,
    storage_proxies: Vec<Arc<RwLock<MockStorage>>>, // to directly modify peers storage
}

impl SynchronizerEnv {
    fn new(handler: MockRpcHandler, role: RoleType) -> Self {
        let runtime = Builder::new().build().unwrap();
        let peers = vec![PeerId::random(), PeerId::random()];

        // setup network
        let addr: Multiaddr = "/memory/0".parse().unwrap();
        let protocols = vec![ProtocolId::from_static(
            STATE_SYNCHRONIZER_DIRECT_SEND_PROTOCOL,
        )];

        // Setup signing public keys.
        let mut rng = StdRng::from_seed(TEST_SEED);
        let (a_signing_private_key, a_signing_public_key) = compat::generate_keypair(&mut rng);
        let (b_signing_private_key, b_signing_public_key) = compat::generate_keypair(&mut rng);
        // Setup identity public keys.
        let (_a_identity_private_key, a_identity_public_key) =
            x25519::compat::generate_keypair(&mut rng);
        let (_b_identity_private_key, b_identity_public_key) =
            x25519::compat::generate_keypair(&mut rng);

        let trusted_peers: HashMap<_, _> = vec![
            (
                peers[0],
                NetworkPublicKeys {
                    signing_public_key: a_signing_public_key.clone(),
                    identity_public_key: a_identity_public_key.clone(),
                },
            ),
            (
                peers[1],
                NetworkPublicKeys {
                    signing_public_key: b_signing_public_key.clone(),
                    identity_public_key: b_identity_public_key.clone(),
                },
            ),
        ]
        .into_iter()
        .collect();

        let (listener_addr, mut network_provider) = NetworkBuilder::new(
            runtime.executor(),
            peers[1],
            addr.clone(),
            RoleType::Validator,
        )
        .signing_keys((b_signing_private_key, b_signing_public_key))
        .trusted_peers(trusted_peers.clone())
        .transport(TransportType::Memory)
        .direct_send_protocols(protocols.clone())
        .build();
        let (sender_b, events_b) = network_provider.add_state_synchronizer(protocols.clone());
        runtime.executor().spawn(network_provider.start());

        let (_dialer_addr, mut network_provider) = NetworkBuilder::new(
            runtime.executor(),
            peers[0],
            addr.clone(),
            RoleType::Validator,
        )
        .transport(TransportType::Memory)
        .signing_keys((a_signing_private_key, a_signing_public_key))
        .trusted_peers(trusted_peers.clone())
        .seed_peers([(peers[1], vec![listener_addr])].iter().cloned().collect())
        .direct_send_protocols(protocols.clone())
        .build();
        let (sender_a, events_a) = network_provider.add_state_synchronizer(protocols);
        runtime.executor().spawn(network_provider.start());

        // create synchronizers
        let mut config = get_test_config().0;
        // TODO: If node is a full node, set correct config.
        config.networks.get_mut(0).unwrap().role = role;
        config
            .state_sync
            .upstream_peers
            .upstream_peers
            .push(peers[1].to_string());
        let storage_proxies = vec![
            Arc::new(RwLock::new(MockStorage::new(0))),
            Arc::new(RwLock::new(MockStorage::new(0))),
        ];
        let synchronizers: Vec<StateSynchronizer> = vec![
            StateSynchronizer::bootstrap_with_executor_proxy(
                vec![(sender_a, events_a)],
                role,
                &config.state_sync,
                MockExecutorProxy::new(
                    peers[0],
                    Self::default_handler(),
                    storage_proxies[0].clone(),
                ),
            ),
            StateSynchronizer::bootstrap_with_executor_proxy(
                vec![(sender_b, events_b)],
                role,
                &get_test_config().0.state_sync,
                MockExecutorProxy::new(peers[1], handler, storage_proxies[1].clone()),
            ),
        ];
        let clients = synchronizers.iter().map(|s| s.create_client()).collect();

        Self {
            peers,
            clients,
            _synchronizers: synchronizers,
            _runtime: runtime,
            storage_proxies,
        }
    }

    fn default_handler() -> MockRpcHandler {
        Box::new(|resp| -> Result<TransactionListWithProof> { Ok(resp) })
    }

    fn sync_to(&self, peer_id: usize, version: u64) {
        let target = MockExecutorProxy::mock_ledger_info(self.peers[1], version);
        block_on(self.clients[peer_id].sync_to(target)).unwrap()
    }

    fn commit(&self, peer_id: usize, version: u64) {
        self.storage_proxies[peer_id]
            .write()
            .unwrap()
            .commit(version);
        block_on(self.clients[peer_id].commit()).unwrap();
    }

    fn wait_for_version(&self, peer_id: usize, target_version: u64) -> bool {
        let max_retries = 30;
        for _ in 0..max_retries {
            let state = block_on(self.clients[peer_id].get_state()).unwrap();
            if state.highest_synced_version == target_version {
                return true;
            }
            std::thread::sleep(std::time::Duration::from_millis(1000));
        }
        false
    }
}

#[test]
fn test_basic_catch_up() {
    let env = SynchronizerEnv::new(SynchronizerEnv::default_handler(), RoleType::Validator);

    // test small sequential syncs
    for version in 1..5 {
        env.sync_to(0, version);
    }
    // test batch sync for multiple transactions
    env.sync_to(0, 10);
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
    env.sync_to(0, 1);
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
