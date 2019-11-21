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
use libra_crypto::hash::CryptoHash;
use libra_crypto::{ed25519::*, test_utils::TEST_SEED, x25519, HashValue};
use libra_logger::set_simple_logger;
use libra_types::block_info::BlockInfo;
use libra_types::crypto_proxies::{
    random_validator_verifier, ValidatorChangeEventWithProof, ValidatorSigner, ValidatorVerifier,
};
use libra_types::{
    account_address::AccountAddress,
    crypto_proxies::LedgerInfoWithSignatures,
    ledger_info::LedgerInfo,
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
use tokio::runtime::Runtime;
use transaction_builder::encode_transfer_script;
use vm_genesis::GENESIS_KEYPAIR;

type MockRpcHandler = Box<
    dyn Fn(TransactionListWithProof) -> Result<TransactionListWithProof> + Send + Sync + 'static,
>;

// To play with the storage values
pub struct MockStorage {
    signer: ValidatorSigner,
    verifier: ValidatorVerifier,
    version: u64,
}

impl MockStorage {
    fn new(signer: ValidatorSigner, verifier: ValidatorVerifier, version: u64) -> Self {
        Self {
            signer,
            verifier,
            version,
        }
    }

    fn commit(&mut self, val: u64) {
        self.version = std::cmp::max(self.version, val);
    }

    fn gen_ledger_info(&self, version: u64) -> LedgerInfoWithSignatures {
        let ledger_info = LedgerInfo::new(
            BlockInfo::new(
                1,
                version,
                HashValue::zero(),
                HashValue::zero(),
                version,
                0,
                None,
            ),
            HashValue::zero(),
        );
        let signature = self.signer.sign_message(ledger_info.hash()).unwrap();
        let mut signatures = BTreeMap::new();
        signatures.insert(self.signer.author(), signature);
        LedgerInfoWithSignatures::new(ledger_info, signatures)
    }

    fn trusted_verifier(&self) -> ValidatorVerifier {
        self.verifier.clone()
    }
}

pub struct MockExecutorProxy {
    handler: MockRpcHandler,
    storage: Arc<RwLock<MockStorage>>,
}

impl MockExecutorProxy {
    fn new(handler: MockRpcHandler, storage: Arc<RwLock<MockStorage>>) -> Self {
        Self { handler, storage }
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
        let highest_local_li = self
            .storage
            .read()
            .unwrap()
            .gen_ledger_info(highest_synced_version);
        let state = SynchronizerState::new(
            highest_local_li,
            highest_synced_version,
            self.storage.read().unwrap().trusted_verifier(),
        );
        async move { Ok(state) }.boxed()
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

    fn get_epoch_proof(&self, _start_epoch: u64) -> Result<ValidatorChangeEventWithProof> {
        unimplemented!("get epoch proof not supported for mock executor proxy");
    }
}

struct SynchronizerEnv {
    _runtime: Runtime,
    _synchronizers: Vec<StateSynchronizer>,
    clients: Vec<Arc<StateSyncClient>>,
    storage_proxies: Vec<Arc<RwLock<MockStorage>>>, // to directly modify peers storage
}

impl SynchronizerEnv {
    fn new(handler: MockRpcHandler, role: RoleType) -> Self {
        set_simple_logger("state-sync");
        let runtime = Runtime::new().unwrap();
        // Generate a verifier with a quorum voting power of 1 (a single signature on LI is enough
        // to pass verification).
        let (signers, verifier) = random_validator_verifier(2, Some(1), true);
        let peers = signers.iter().map(|s| s.author()).collect::<Vec<PeerId>>();

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
            runtime.handle().clone(),
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
        runtime.handle().spawn(network_provider.start());

        let (_dialer_addr, mut network_provider) = NetworkBuilder::new(
            runtime.handle().clone(),
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
        runtime.handle().spawn(network_provider.start());

        // create synchronizers
        let mut config = get_test_config().0;
        if !role.is_validator() {
            let mut network = config.validator_network.unwrap();
            network.set_default_peer_id();
            config.full_node_networks = vec![network];
            config.validator_network = None;
        }
        config.set_role(role).expect("Unable to set role");
        config
            .state_sync
            .upstream_peers
            .upstream_peers
            .push(peers[1]);
        let storage_proxies = vec![
            Arc::new(RwLock::new(MockStorage::new(
                signers[0].clone(),
                verifier.clone(),
                0,
            ))),
            Arc::new(RwLock::new(MockStorage::new(
                signers[1].clone(),
                verifier.clone(),
                0,
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
                &get_test_config().0.state_sync,
                MockExecutorProxy::new(handler, storage_proxies[1].clone()),
            ),
        ];
        let clients = synchronizers.iter().map(|s| s.create_client()).collect();

        Self {
            clients,
            _synchronizers: synchronizers,
            _runtime: runtime,
            storage_proxies,
        }
    }

    fn default_handler() -> MockRpcHandler {
        Box::new(|resp| -> Result<TransactionListWithProof> { Ok(resp) })
    }

    fn sync_to(&self, peer_id: usize, target: LedgerInfoWithSignatures) {
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
        // peer 0 syncs to a target signed by peer 1
        env.sync_to(
            0,
            env.storage_proxies[1]
                .read()
                .unwrap()
                .gen_ledger_info(version),
        );
    }
    // test batch sync for multiple transactions
    env.sync_to(
        0,
        env.storage_proxies[1].read().unwrap().gen_ledger_info(10),
    );
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
    env.sync_to(0, env.storage_proxies[1].read().unwrap().gen_ledger_info(1));
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
