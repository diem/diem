// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    executor_proxy::ExecutorProxyTrait, LedgerInfo, PeerId, StateSyncClient, StateSynchronizer,
};
use config::config::RoleType;
use config_builder::util::get_test_config;
use crypto::{ed25519::*, test_utils::TEST_SEED, traits::Genesis, x25519, HashValue, SigningKey};
use execution_proto::proto::execution::{ExecuteChunkRequest, ExecuteChunkResponse};
use failure::{prelude::*, Result};
use futures::{
    executor::block_on,
    future::{FutureExt, TryFutureExt},
    Future,
};
use network::{
    proto::GetChunkResponse,
    validator_network::{
        network_builder::{NetworkBuilder, TransportType},
        STATE_SYNCHRONIZER_MSG_PROTOCOL,
    },
    NetworkPublicKeys, ProtocolId,
};
use parity_multiaddr::Multiaddr;
use proto_conv::{FromProto, IntoProto};
use rand::{rngs::StdRng, SeedableRng};
use std::{
    collections::HashMap,
    pin::Pin,
    sync::{
        atomic::{AtomicU64, AtomicUsize, Ordering},
        Arc,
    },
};
use tokio::runtime::{Builder, Runtime};
use types::{
    account_address::AccountAddress,
    ledger_info::{LedgerInfo as TypesLedgerInfo, LedgerInfoWithSignatures},
    proof::AccumulatorProof,
    test_helpers::transaction_test_helpers::get_test_signed_txn,
    transaction::{SignedTransaction, TransactionInfo, TransactionListWithProof},
};
use vm_genesis::{encode_transfer_program, GENESIS_KEYPAIR};

type MockRpcHandler =
    Box<dyn Fn(GetChunkResponse) -> Result<GetChunkResponse> + Send + Sync + 'static>;

pub struct MockExecutorProxy {
    peer_id: PeerId,
    handler: MockRpcHandler,
    version: AtomicU64,
}

impl MockExecutorProxy {
    fn new(peer_id: PeerId, handler: MockRpcHandler) -> Self {
        Self {
            peer_id,
            handler,
            version: AtomicU64::new(0),
        }
    }

    fn mock_ledger_info(peer_id: PeerId, version: u64) -> LedgerInfo {
        let ledger_info = TypesLedgerInfo::new(
            version,
            HashValue::zero(),
            HashValue::zero(),
            HashValue::zero(),
            0,
            0,
            None,
        );
        let mut signatures = HashMap::new();
        let private_key = Ed25519PrivateKey::genesis();
        let signature = private_key.sign_message(&HashValue::zero());
        signatures.insert(peer_id, signature);
        LedgerInfoWithSignatures::new(ledger_info, signatures)
    }

    fn mock_chunk_response(&self, version: u64) -> GetChunkResponse {
        let target = Self::mock_ledger_info(self.peer_id, version + 1);

        let sender = AccountAddress::from_public_key(&GENESIS_KEYPAIR.1);
        let receiver = AccountAddress::new([0xff; 32]);
        let program = encode_transfer_program(&receiver, 1);
        let transaction = get_test_signed_txn(
            sender,
            version + 1,
            GENESIS_KEYPAIR.0.clone(),
            GENESIS_KEYPAIR.1.clone(),
            Some(program),
        );

        let txn_info =
            TransactionInfo::new(HashValue::zero(), HashValue::zero(), HashValue::zero(), 0);
        let accumulator_proof = AccumulatorProof::new(vec![]);
        let txns = TransactionListWithProof::new(
            vec![(
                SignedTransaction::from_proto(transaction).unwrap(),
                txn_info,
            )],
            None,
            Some(version + 1),
            Some(accumulator_proof),
            None,
        );

        let mut resp = GetChunkResponse::new();
        resp.set_txn_list_with_proof(txns.into_proto());
        resp.set_ledger_info_with_sigs(target.into_proto());
        resp
    }
}

impl ExecutorProxyTrait for MockExecutorProxy {
    fn get_latest_ledger_info(&self) -> Pin<Box<dyn Future<Output = Result<LedgerInfo>> + Send>> {
        let version = self.version.load(Ordering::Relaxed);
        let response = Self::mock_ledger_info(self.peer_id, version);
        async move { Ok(response) }.boxed()
    }

    fn get_latest_version(&self) -> Pin<Box<dyn Future<Output = Result<u64>> + Send>> {
        let version = self.version.load(Ordering::Relaxed);
        async move { Ok(version) }.boxed()
    }

    fn execute_chunk(
        &self,
        request: ExecuteChunkRequest,
    ) -> Pin<Box<dyn Future<Output = Result<ExecuteChunkResponse>> + Send>> {
        let version = request
            .get_ledger_info_with_sigs()
            .get_ledger_info()
            .version;
        self.version.store(version, Ordering::Relaxed);
        async move { Ok(ExecuteChunkResponse::new()) }.boxed()
    }

    fn get_chunk(
        &self,
        known_version: u64,
        _: u64,
        _: LedgerInfo,
    ) -> Pin<Box<dyn Future<Output = Result<GetChunkResponse>> + Send>> {
        let response = (self.handler)(self.mock_chunk_response(known_version));
        async move { response }.boxed()
    }

    fn validate_ledger_info(&self, _target: &LedgerInfo) -> Result<()> {
        Ok(())
    }
}

struct SynchronizerEnv {
    peers: Vec<PeerId>,
    clients: Vec<Arc<StateSyncClient>>,
    _synchronizers: Vec<StateSynchronizer>,
    _runtime: Runtime,
}

impl SynchronizerEnv {
    fn new(handler: MockRpcHandler, role: RoleType) -> Self {
        let runtime = Builder::new().build().unwrap();
        let peers = vec![PeerId::random(), PeerId::random()];

        // setup network
        let addr: Multiaddr = "/memory/0".parse().unwrap();
        let protocols = vec![ProtocolId::from_static(STATE_SYNCHRONIZER_MSG_PROTOCOL)];

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
        runtime
            .executor()
            .spawn(network_provider.start().unit_error().compat());

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
        runtime
            .executor()
            .spawn(network_provider.start().unit_error().compat());

        // create synchronizers
        let mut config = get_test_config().0;
        if role == RoleType::FullNode {
            config.network.role = "full_node".to_string();
        }
        let synchronizers: Vec<StateSynchronizer> = vec![
            StateSynchronizer::bootstrap_with_executor_proxy(
                vec![(sender_a, events_a)],
                &config,
                MockExecutorProxy::new(peers[0], Self::default_handler()),
                vec![peers[1]],
            ),
            StateSynchronizer::bootstrap_with_executor_proxy(
                vec![(sender_b, events_b)],
                &get_test_config().0,
                MockExecutorProxy::new(peers[1], handler),
                vec![],
            ),
        ];
        let clients = synchronizers.iter().map(|s| s.create_client()).collect();

        Self {
            peers,
            clients,
            _synchronizers: synchronizers,
            _runtime: runtime,
        }
    }

    fn default_handler() -> MockRpcHandler {
        Box::new(|resp| -> Result<GetChunkResponse> { Ok(resp) })
    }

    fn sync_to(&self, peer_id: usize, version: u64) -> bool {
        let target = MockExecutorProxy::mock_ledger_info(self.peers[1], version);
        block_on(self.clients[peer_id].sync_to(target)).unwrap()
    }

    fn commit(&self, peer_id: usize, version: u64) {
        block_on(self.clients[peer_id].commit(version)).unwrap();
    }

    fn wait_for_version(&self, peer_id: usize, target_version: u64) -> bool {
        let max_retries = 30;
        for _ in 0..max_retries {
            let state = block_on(self.clients[peer_id].get_state()).unwrap();
            if state == target_version {
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
        assert!(env.sync_to(0, version));
    }
    // test batch sync for multiple transactions
    assert!(env.sync_to(0, 10));
}

#[test]
fn test_flaky_peer_sync() {
    // create handler that causes error, but has successful retries
    let attempt = AtomicUsize::new(0);
    let handler = Box::new(move |resp| -> Result<GetChunkResponse> {
        let fail_request = attempt.load(Ordering::Relaxed) == 0;
        attempt.fetch_add(1, Ordering::Relaxed);
        if fail_request {
            bail!("chunk fetch failed")
        } else {
            Ok(resp)
        }
    });
    let env = SynchronizerEnv::new(handler, RoleType::Validator);
    assert!(env.sync_to(0, 1));
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
