// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chained_bft::{test_utils, QuorumCert},
    state_replication::ExecutedState,
    state_synchronizer::{
        coordinator::SyncStatus,
        mocks::{gen_txn_list, MockExecutorProxy},
        PeerId, StateSynchronizer,
    },
};
use bytes::Bytes;
use config::config::NodeConfig;
use config_builder::util::get_test_config;
use crypto::{signing, x25519, HashValue};
use failure::prelude::*;
use futures::{
    executor::block_on,
    future::{join_all, TryFutureExt},
    stream::StreamExt,
    FutureExt,
};
use metrics::get_all_metrics;
use network::{
    proto::{ConsensusMsg, RespondChunk},
    validator_network::{
        network_builder::{NetworkBuilder, TransportType},
        Event, RpcError, CONSENSUS_RPC_PROTOCOL,
    },
    NetworkPublicKeys, ProtocolId,
};
use parity_multiaddr::Multiaddr;
use proto_conv::IntoProto;
use protobuf::Message;
use rusty_fork::{rusty_fork_id, rusty_fork_test, rusty_fork_test_name};
use std::{
    collections::HashMap,
    sync::atomic::{AtomicUsize, Ordering},
};
use tokio::runtime::Runtime;
use types::{
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    transaction::TransactionListWithProof,
};

struct SynchronizerEnv {
    synchronizers: Vec<StateSynchronizer>,
    peers: Vec<PeerId>,
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
        let mut runtime = test_utils::consensus_runtime();
        let config = opt_config.unwrap_or_else(|| {
            let (config_inner, _) = get_test_config();
            config_inner
        });
        let peers = vec![PeerId::random(), PeerId::random()];

        // setup network
        let addr: Multiaddr = "/memory/0".parse().unwrap();
        let protocols = vec![ProtocolId::from_static(CONSENSUS_RPC_PROTOCOL)];

        // Setup signing public keys.
        let (a_signing_private_key, a_signing_public_key) = signing::generate_keypair();
        let (b_signing_private_key, b_signing_public_key) = signing::generate_keypair();
        // Setup identity public keys.
        let (a_identity_private_key, a_identity_public_key) = x25519::generate_keypair();
        let (b_identity_private_key, b_identity_public_key) = x25519::generate_keypair();

        let trusted_peers: HashMap<_, _> = vec![
            (
                peers[0],
                NetworkPublicKeys {
                    signing_public_key: a_signing_public_key,
                    identity_public_key: a_identity_public_key,
                },
            ),
            (
                peers[1],
                NetworkPublicKeys {
                    signing_public_key: b_signing_public_key,
                    identity_public_key: b_identity_public_key,
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
            StateSynchronizer::new(
                sender_a,
                runtime.executor(),
                &config,
                MockExecutorProxy::default(),
            ),
            StateSynchronizer::new(
                sender_b,
                runtime.executor(),
                &config,
                MockExecutorProxy::default(),
            ),
        ];

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
            synchronizers,
            peers,
            _runtime: runtime,
        }
    }

    fn gen_commit(&self, version: u64) -> QuorumCert {
        let ledger_info = LedgerInfo::new(
            version,
            HashValue::zero(),
            HashValue::zero(),
            HashValue::zero(),
            0,
            0,
        );
        let mut signatures = HashMap::new();
        let private_key = signing::generate_genesis_keypair().0;
        let signature = signing::sign_message(HashValue::zero(), &private_key).unwrap();
        signatures.insert(self.peers[1], signature);
        QuorumCert::new(
            HashValue::zero(),
            ExecutedState::state_for_genesis(),
            0,
            LedgerInfoWithSignatures::new(ledger_info, signatures),
        )
    }
}

#[test]
fn test_basic_flow() {
    let env = SynchronizerEnv::new();

    // test small sequential syncs
    for version in 1..5 {
        let status = block_on(env.synchronizers[0].sync_to(env.gen_commit(version)));
        assert_eq!(status.unwrap(), SyncStatus::Finished);
    }
    // test batch sync for multiple transactions
    let status = block_on(env.synchronizers[0].sync_to(env.gen_commit(10)));
    assert_eq!(status.unwrap(), SyncStatus::Finished);
}

rusty_fork_test! {
#[test]
fn test_concurrent_requests() {
    let env = SynchronizerEnv::new();

    let requests = vec![
        env.synchronizers[0].sync_to(env.gen_commit(1)),
        env.synchronizers[0].sync_to(env.gen_commit(2)),
        env.synchronizers[0].sync_to(env.gen_commit(3)),
    ];
    // ensure we can execute requests in parallel
    block_on(join_all(requests));
    // ensure we downloaded each chunk exactly 1 time
    let metrics = get_all_metrics();
    assert_eq!(metrics["consensus{op=download}"].parse::<i32>().unwrap(), 3);
}
}

#[test]
fn test_download_failure() {
    // create handler that causes errors
    let handler = Box::new(|| -> Result<TransactionListWithProof> { bail!("chunk fetch failed") });

    let env = SynchronizerEnv::new_with(handler, None);
    let status = block_on(env.synchronizers[0].sync_to(env.gen_commit(5)));
    assert_eq!(status.unwrap(), SyncStatus::DownloadFailed);
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
    let status = block_on(env.synchronizers[0].sync_to(env.gen_commit(1)));
    assert_eq!(status.unwrap(), SyncStatus::Finished);
}
