// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::{
    cluster::Cluster,
    experiments::{Context, Experiment, ExperimentParam},
    tx_emitter::EmitJobRequest,
};
use anyhow::Result;
use async_trait::async_trait;
use futures::{sink::SinkExt, StreamExt};
use libra_config::{config::NodeConfig, network_id::NetworkId};
use libra_crypto::x25519;
use libra_mempool::network::{MempoolNetworkEvents, MempoolNetworkSender};
use libra_network_address::NetworkAddress;
use libra_types::chain_id::ChainId;
use network::{
    connectivity_manager::DiscoverySource, protocols::network::Event, ConnectivityRequest,
};
use network_builder::builder::NetworkBuilder;
use state_synchronizer::network::{StateSynchronizerEvents, StateSynchronizerSender};
use std::{
    collections::{HashMap, HashSet},
    fmt,
    time::{Duration, Instant},
};
use structopt::StructOpt;
use tokio::runtime::{Builder, Runtime};

const EXPERIMENT_BUFFER_SECS: u64 = 10;

#[derive(StructOpt, Debug)]
pub struct LoadTestParams {
    #[structopt(long, help = "run load test on mempool")]
    pub mempool: bool,
    #[structopt(long, help = "run load test on state sync")]
    pub state_sync: bool,
    #[structopt(long, help = "emit p2p transfer txns during experiment")]
    pub emit_txn: bool,
    #[structopt(
        long,
        help = "duration (in seconds) to run load test for. All specified components (mempool, state sync) will be load tested simultaneously"
    )]
    pub duration: u64,
}

pub struct LoadTest {
    mempool: bool,
    state_sync: bool,
    emit_txn: bool,
    duration: u64,
}

impl ExperimentParam for LoadTestParams {
    type E = LoadTest;
    fn build(self, _cluster: &Cluster) -> Self::E {
        LoadTest {
            mempool: self.mempool,
            state_sync: self.state_sync,
            emit_txn: self.emit_txn,
            duration: self.duration,
        }
    }
}

#[async_trait]
impl Experiment for LoadTest {
    fn affected_validators(&self) -> HashSet<String> {
        HashSet::new()
    }
    async fn run(&mut self, context: &mut Context<'_>) -> anyhow::Result<()> {
        // spin up StubbedNode
        let vfn = context.cluster.random_fullnode_instance();
        let vfn_endpoint = format!("http://{}:{}/v1", vfn.ip(), vfn.ac_port());

        let mut stubbed_node = StubbedNode::launch(vfn_endpoint).await;

        let mut emit_job = None;
        let mut mempool_task = None;
        let mut state_sync_task = None;
        let duration = Duration::from_secs(self.duration);

        if self.emit_txn {
            // emit txns to JSON RPC
            // spawn future
            emit_job = Some(
                context
                    .tx_emitter
                    .start_job(EmitJobRequest::for_instances(
                        context.cluster.fullnode_instances().to_vec(),
                        context.global_emit_job_request,
                        0,
                    ))
                    .await?,
            );
        }

        if self.mempool {
            // spawn mempool load test
            let (mempool_sender, mempool_events) = stubbed_node
                .mempool_handle
                .take()
                .expect("missing mempool network handles");
            mempool_task = Some(tokio::task::spawn(mempool_load_test(
                duration,
                mempool_sender,
                mempool_events,
            )));
        }

        if self.state_sync {
            // spawn state sync load test
            let (state_sync_sender, state_sync_events) = stubbed_node
                .state_sync_handle
                .take()
                .expect("missing state sync network handles");
            state_sync_task = Some(tokio::task::spawn(state_sync_load_test(
                duration,
                state_sync_sender,
                state_sync_events,
            )));
        }

        // await on all spawned tasks
        tokio::time::delay_for(Duration::from_secs(self.duration)).await;
        if let Some(j) = emit_job {
            let _ = context.tx_emitter.stop_job(j).await;
        }

        if let Some(t) = mempool_task {
            let _ = t.await.expect("failed mempool load test task");
        }

        if let Some(t) = state_sync_task {
            let _ = t.await.expect("failed state sync load test task");
        }

        // create blocking context to drop stubbed node's runtime in
        // We cannot drop a runtime in an async context where blocking is not allowed - otherwise,
        // this thread will panic.
        tokio::task::spawn_blocking(move || {
            drop(stubbed_node);
        })
        .await?;

        // TODO log per-component experiment results

        Ok(())
    }
    fn deadline(&self) -> Duration {
        Duration::from_secs(self.duration + EXPERIMENT_BUFFER_SECS)
    }
}

impl fmt::Display for LoadTest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Load test components: mempool: {}, state sync: {}, emit txns: {}",
            self.mempool, self.state_sync, self.emit_txn,
        )
    }
}

// An actor that can participate in LibraNet
// Connects to VFN via on-chain discovery and interact with it via mempool and state sync protocol
// It is 'stubbed' in the sense that it has no real node components running and only network stubs
// that interact with the remote VFN via LibraNet mempool and state sync protocol
struct StubbedNode {
    pub network_runtime: Runtime,
    pub mempool_handle: Option<(MempoolNetworkSender, MempoolNetworkEvents)>,
    pub state_sync_handle: Option<(StateSynchronizerSender, StateSynchronizerEvents)>,
}

impl StubbedNode {
    async fn launch(node_endpoint: String) -> Self {
        // generate seed peers config from querying node endpoint
        let seed_peers = seed_peer_generator::utils::gen_seed_peer_config(node_endpoint);

        // build sparse network runner

        let pfn_config = NodeConfig::default_for_public_full_node();

        // some sanity checks on the network the stubbed node will be running in
        assert_eq!(
            pfn_config.full_node_networks.len(),
            1,
            "expected only one fn network for PFN"
        );
        let network_config = &pfn_config.full_node_networks[0];
        assert_eq!(network_config.network_id, NetworkId::Public);

        let mut network_builder =
            NetworkBuilder::create(ChainId::test(), pfn_config.base.role, network_config);

        let state_sync_handle = Some(
            network_builder
                .add_protocol_handler(state_synchronizer::network::network_endpoint_config()),
        );

        let mempool_handle = Some(network_builder.add_protocol_handler(
            libra_mempool::network::network_endpoint_config(
                pfn_config.mempool.max_broadcasts_per_peer,
            ),
        ));
        let network_runtime = Builder::new()
            .thread_name("stubbed-node-network")
            .threaded_scheduler()
            .enable_all()
            .build()
            .expect("Failed to start runtime. Won't be able to start networking.");

        network_builder.build(network_runtime.handle().clone());

        network_builder.start();

        // feed the network builder the seed peer config
        let mut conn_req_tx = network_builder
            .conn_mgr_reqs_tx()
            .expect("expecting connectivity mgr to exist after adding protocol handler");

        let new_peer_pubkeys: HashMap<_, _> = seed_peers
            .iter()
            .map(|(peer_id, addrs)| {
                let pubkeys: HashSet<x25519::PublicKey> = addrs
                    .iter()
                    .filter_map(NetworkAddress::find_noise_proto)
                    .collect();
                (*peer_id, pubkeys)
            })
            .collect();

        let conn_reqs = vec![
            ConnectivityRequest::UpdateAddresses(DiscoverySource::OnChain, seed_peers),
            ConnectivityRequest::UpdateEligibleNodes(DiscoverySource::OnChain, new_peer_pubkeys),
        ];

        for update in conn_reqs {
            conn_req_tx
                .send(update)
                .await
                .expect("failed to send conn req");
        }

        Self {
            network_runtime,
            mempool_handle,
            state_sync_handle,
        }
    }
}

async fn mempool_load_test(
    duration: Duration,
    mut sender: MempoolNetworkSender,
    mut events: MempoolNetworkEvents,
) -> Result<MempoolResult> {
    let new_peer_event = events.select_next_some().await;
    let vfn = if let Event::NewPeer(peer_id, _) = new_peer_event {
        peer_id
    } else {
        return Err(anyhow::format_err!(
            "received unexpected network event for mempool load test"
        ));
    };

    let task_start = Instant::now();
    while Instant::now().duration_since(task_start) < duration {
        let msg = libra_mempool::network::MempoolSyncMsg::BroadcastTransactionsRequest {
            request_id: lcs::to_bytes("request_id")?,
            transactions: vec![], // TODO submit actual txns
        };
        // TODO log stats for bandwidth sent to remote peer to MempoolResult
        sender.send_to(vfn, msg)?;

        // await ACK from remote peer
        let _response = events.select_next_some().await;
    }

    Ok(MempoolResult)
}

// TODO store more stats
struct MempoolResult;

async fn state_sync_load_test(
    duration: Duration,
    mut sender: StateSynchronizerSender,
    mut events: StateSynchronizerEvents,
) -> Result<StateSyncResult> {
    let new_peer_event = events.select_next_some().await;
    let vfn = if let Event::NewPeer(peer_id, _) = new_peer_event {
        peer_id
    } else {
        return Err(anyhow::format_err!(
            "received unexpected network event for state sync load test"
        ));
    };

    let chunk_request = state_synchronizer::chunk_request::GetChunkRequest::new(
        1,
        0,
        250,
        state_synchronizer::chunk_request::TargetType::HighestAvailable {
            target_li: None,
            timeout_ms: 10_000,
        },
    );

    let task_start = Instant::now();
    let mut served_txns = 0;
    while Instant::now().duration_since(task_start) < duration {
        let msg = state_synchronizer::network::StateSynchronizerMsg::GetChunkRequest(Box::new(
            chunk_request.clone(),
        ));
        sender.send_to(vfn, msg)?;

        // await response from remote peer
        let response = events.select_next_some().await;
        if let Event::Message(_remote_peer, payload) = response {
            if let state_synchronizer::network::StateSynchronizerMsg::GetChunkResponse(
                chunk_response,
            ) = payload
            {
                // TODO analyze response and update StateSyncResult with stats accordingly
                served_txns += chunk_response.txn_list_with_proof.transactions.len();
            }
        }
    }
    Ok(StateSyncResult { served_txns })
}

// TODO store more stats here
struct StateSyncResult {
    pub served_txns: usize,
}
