// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    coordinator::{ExecutorProxyTrait, SyncStatus},
    PeerId, StateSyncClient, StateSynchronizer,
};
use bytes::Bytes;
use config::config::NodeConfig;
use config_builder::util::get_test_config;
use crypto::HashValue;
use execution_proto::proto::execution::{ExecuteChunkRequest, ExecuteChunkResponse};
use failure::{prelude::*, Result};
use futures::{executor::block_on, future::TryFutureExt, stream::StreamExt, Future, FutureExt};
use network::{
    proto::{ConsensusMsg, RespondChunk},
    validator_network::{
        network_builder::{NetworkBuilder, TransportType},
        Event, RpcError, CONSENSUS_RPC_PROTOCOL,
    },
    NetworkPublicKeys, ProtocolId,
};
use nextgen_crypto::{ed25519::*, test_utils::TEST_SEED, traits::Genesis, x25519, SigningKey};
use parity_multiaddr::Multiaddr;
use proto_conv::{FromProto, IntoProto};
use protobuf::Message;
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

#[derive(Default)]
pub struct MockExecutorProxy {
    version: AtomicU64,
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
}

pub fn gen_txn_list(sequence_number: u64) -> TransactionListWithProof {
    let sender = AccountAddress::from_public_key(&GENESIS_KEYPAIR.1);
    let receiver = AccountAddress::new([0xff; 32]);
    let program = encode_transfer_program(&receiver, 1);
    let transaction = get_test_signed_txn(
        sender.into(),
        sequence_number,
        GENESIS_KEYPAIR.0.clone(),
        GENESIS_KEYPAIR.1.clone(),
        Some(program),
    );

    let txn_info = TransactionInfo::new(HashValue::zero(), HashValue::zero(), HashValue::zero(), 0);
    let accumulator_proof = AccumulatorProof::new(vec![]);
    TransactionListWithProof::new(
        vec![(
            SignedTransaction::from_proto(transaction).unwrap(),
            txn_info,
        )],
        None,
        Some(0),
        Some(accumulator_proof),
        None,
    )
}

struct SynchronizerEnv {
    peers: Vec<PeerId>,
    clients: Vec<Arc<StateSyncClient>>,
    _synchronizers: Vec<StateSynchronizer>,
    _runtime: Runtime,
}

impl SynchronizerEnv {
    fn new() -> Self {
        let handler = Box::new(|| -> Result<TransactionListWithProof> { Ok(gen_txn_list(0)) });
        Self::new_with(handler, None)
    }

    fn new_with(
        handler: Box<dyn Fn() -> Result<TransactionListWithProof> + Send + 'static>,
        opt_config: Option<NodeConfig>,
    ) -> Self {
        let mut runtime = Builder::new().build().unwrap();
        let config = opt_config.unwrap_or_else(|| {
            let (config_inner, _) = get_test_config();
            config_inner
        });
        let peers = vec![PeerId::random(), PeerId::random()];

        // setup network
        let addr: Multiaddr = "/memory/0".parse().unwrap();
        let protocols = vec![ProtocolId::from_static(CONSENSUS_RPC_PROTOCOL)];

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

        let (_, (sender_b, mut events_b), listener_addr) =
            NetworkBuilder::new(runtime.executor(), peers[1], addr.clone())
                .signing_keys((b_signing_private_key, b_signing_public_key))
                .identity_keys((b_identity_private_key, b_identity_public_key))
                .trusted_peers(trusted_peers.clone())
                .transport(TransportType::Memory)
                .consensus_protocols(protocols.clone())
                .rpc_protocols(protocols.clone())
                .build();

        let (sender_a, mut events_a) =
            NetworkBuilder::new(runtime.executor(), peers[0], addr.clone())
                .transport(TransportType::Memory)
                .signing_keys((a_signing_private_key, a_signing_public_key))
                .identity_keys((a_identity_private_key, a_identity_public_key))
                .trusted_peers(trusted_peers.clone())
                .seed_peers([(peers[1], vec![listener_addr])].iter().cloned().collect())
                .consensus_protocols(protocols.clone())
                .rpc_protocols(protocols)
                .build()
                .1;

        // await peer discovery
        block_on(events_a.next()).unwrap().unwrap();
        block_on(events_b.next()).unwrap().unwrap();

        // create synchronizers
        let synchronizers = vec![
            StateSynchronizer::bootstrap_with_executor_proxy(
                sender_a,
                &config,
                MockExecutorProxy::default(),
            ),
            StateSynchronizer::bootstrap_with_executor_proxy(
                sender_b,
                &config,
                MockExecutorProxy::default(),
            ),
        ];
        let clients = synchronizers
            .iter()
            .map(|s| s.create_client(&config))
            .collect();

        let rpc_handler = async move {
            while let Some(event) = events_b.next().await {
                if let Ok(Event::RpcRequest((_, _, callback))) = event {
                    match handler() {
                        Ok(txn_list) => {
                            let mut response_msg = ConsensusMsg::new();
                            let mut response = RespondChunk::new();
                            response.set_txn_list_with_proof(txn_list.into_proto());
                            response_msg.set_respond_chunk(response);
                            let response_data = Bytes::from(response_msg.write_to_bytes().unwrap());
                            callback.send(Ok(response_data)).unwrap();
                        }
                        Err(err) => {
                            callback.send(Err(RpcError::ApplicationError(err))).unwrap();
                        }
                    }
                }
            }
        };
        runtime.spawn(rpc_handler.boxed().unit_error().compat());

        Self {
            peers,
            clients,
            _synchronizers: synchronizers,
            _runtime: runtime,
        }
    }

    fn sync_to(&self, peer_id: usize, version: u64) -> SyncStatus {
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
        signatures.insert(self.peers[1], signature);
        let target = LedgerInfoWithSignatures::new(ledger_info, signatures);
        block_on(self.clients[peer_id].sync_to(target)).unwrap()
    }
}

#[test]
fn test_basic_flow() {
    let env = SynchronizerEnv::new();

    // test small sequential syncs
    for version in 1..5 {
        let status = env.sync_to(0, version);
        assert_eq!(status, SyncStatus::Finished);
    }
    // test batch sync for multiple transactions
    let status = env.sync_to(0, 10);
    assert_eq!(status, SyncStatus::Finished);
}

#[test]
fn test_download_failure() {
    // create handler that causes errors
    let handler = Box::new(|| -> Result<TransactionListWithProof> { bail!("chunk fetch failed") });

    let env = SynchronizerEnv::new_with(handler, None);
    let status = env.sync_to(0, 5);
    assert_eq!(status, SyncStatus::DownloadFailed);
}

#[test]
fn test_download_retry() {
    // create handler that causes error, but has successful retries
    let attempt = AtomicUsize::new(0);
    let handler = Box::new(move || -> Result<TransactionListWithProof> {
        let fail_request = attempt.load(Ordering::Relaxed) == 0;
        attempt.fetch_add(1, Ordering::Relaxed);
        if fail_request {
            bail!("chunk fetch failed")
        } else {
            Ok(gen_txn_list(0))
        }
    });
    let env = SynchronizerEnv::new_with(handler, None);
    let status = env.sync_to(0, 1);
    assert_eq!(status, SyncStatus::Finished);
}
