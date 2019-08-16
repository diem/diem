// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{coordinator::ExecutorProxyTrait, PeerId, StateSyncClient, StateSynchronizer};
use config_builder::util::get_test_config;
use crypto::HashValue;
use execution_proto::proto::execution::{ExecuteChunkRequest, ExecuteChunkResponse};
use failure::{prelude::*, Result};
use futures::{
    executor::block_on,
    future::{FutureExt, TryFutureExt},
    stream::StreamExt,
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
use nextgen_crypto::{ed25519::*, test_utils::TEST_SEED, traits::Genesis, x25519, SigningKey};
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
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
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

    fn mock_ledger_info(
        peer_id: PeerId,
        version: u64,
    ) -> LedgerInfoWithSignatures<Ed25519Signature> {
        let ledger_info = LedgerInfo::new(
            version,
            HashValue::zero(),
            HashValue::zero(),
            HashValue::zero(),
            0,
            0,
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
            sender.into(),
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
            Some(0),
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
    fn get_latest_version(&self) -> Pin<Box<dyn Future<Output = Result<u64>> + Send>> {
        let version = self.version.load(Ordering::Relaxed);
        async move { Ok(version) }.boxed()
    }

    fn execute_chunk(
        &self,
        _request: ExecuteChunkRequest,
    ) -> Pin<Box<dyn Future<Output = Result<ExecuteChunkResponse>> + Send>> {
        self.version.fetch_add(1, Ordering::Relaxed);
        async move { Ok(ExecuteChunkResponse::new()) }.boxed()
    }

    fn get_chunk(
        &self,
        known_version: u64,
        _: u64,
        _: LedgerInfoWithSignatures<Ed25519Signature>,
    ) -> Pin<Box<dyn Future<Output = Result<GetChunkResponse>> + Send>> {
        let response = (self.handler)(self.mock_chunk_response(known_version));
        async move { response }.boxed()
    }
}

struct SynchronizerEnv {
    peers: Vec<PeerId>,
    clients: Vec<Arc<StateSyncClient>>,
    _synchronizers: Vec<StateSynchronizer>,
    _runtime: Runtime,
}

impl SynchronizerEnv {
    fn new(handler: MockRpcHandler) -> Self {
        let runtime = Builder::new().build().unwrap();
        let config = get_test_config().0;
        let peers = vec![PeerId::random(), PeerId::random()];

        // setup network
        let addr: Multiaddr = "/memory/0".parse().unwrap();
        let protocols = vec![ProtocolId::from_static(STATE_SYNCHRONIZER_MSG_PROTOCOL)];

        // Setup signing public keys.
        let mut rng = StdRng::from_seed(TEST_SEED);
        let (a_signing_private_key, a_signing_public_key) = compat::generate_keypair(&mut rng);
        let (b_signing_private_key, b_signing_public_key) = compat::generate_keypair(&mut rng);
        // Setup identity public keys.
        let (a_identity_private_key, a_identity_public_key) =
            x25519::compat::generate_keypair(&mut rng);
        let (b_identity_private_key, b_identity_public_key) =
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

        let (listener_addr, mut network_provider) =
            NetworkBuilder::new(runtime.executor(), peers[1], addr.clone())
                .signing_keys((b_signing_private_key, b_signing_public_key))
                .identity_keys((b_identity_private_key, b_identity_public_key))
                .trusted_peers(trusted_peers.clone())
                .transport(TransportType::Memory)
                .direct_send_protocols(protocols.clone())
                .build();
        let (sender_b, mut events_b) = network_provider.add_state_synchronizer(protocols.clone());
        runtime
            .executor()
            .spawn(network_provider.start().unit_error().compat());

        let (_dialer_addr, mut network_provider) =
            NetworkBuilder::new(runtime.executor(), peers[0], addr.clone())
                .transport(TransportType::Memory)
                .signing_keys((a_signing_private_key, a_signing_public_key))
                .identity_keys((a_identity_private_key, a_identity_public_key))
                .trusted_peers(trusted_peers.clone())
                .seed_peers([(peers[1], vec![listener_addr])].iter().cloned().collect())
                .direct_send_protocols(protocols.clone())
                .build();
        let (sender_a, mut events_a) = network_provider.add_state_synchronizer(protocols);
        runtime
            .executor()
            .spawn(network_provider.start().unit_error().compat());

        // await peer discovery
        block_on(events_a.next()).unwrap().unwrap();
        block_on(events_b.next()).unwrap().unwrap();

        let default_handler = Box::new(|resp| -> Result<GetChunkResponse> { Ok(resp) });
        // create synchronizers
        let synchronizers: Vec<StateSynchronizer> = vec![
            StateSynchronizer::bootstrap_with_executor_proxy(
                sender_a,
                events_a,
                &config,
                MockExecutorProxy::new(peers[0], default_handler),
            ),
            StateSynchronizer::bootstrap_with_executor_proxy(
                sender_b,
                events_b,
                &config,
                MockExecutorProxy::new(peers[1], handler),
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

    fn sync_to(&self, peer_id: usize, version: u64) -> bool {
        let target = MockExecutorProxy::mock_ledger_info(self.peers[1], version);
        block_on(self.clients[peer_id].sync_to(target)).unwrap()
    }
}

#[test]
fn test_basic_catch_up() {
    let handler = Box::new(|resp| -> Result<GetChunkResponse> { Ok(resp) });
    let env = SynchronizerEnv::new(handler);

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
    let env = SynchronizerEnv::new(handler);
    assert!(env.sync_to(0, 1));
}
