// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::{
    cluster::Cluster,
    experiments::{Context, Experiment, ExperimentParam},
};
use anyhow::Result;
use async_trait::async_trait;
use diem_config::{config::NodeConfig, network_id::NetworkId};
use diem_logger::*;
use diem_mempool::network::{MempoolNetworkEvents, MempoolNetworkSender};
use diem_sdk::transaction_builder::TransactionFactory;
use diem_time_service::TimeService;
use diem_types::{account_config::diem_root_address, chain_id::ChainId};
use forge::{gen_transfer_txn_request, TxnEmitter};
use futures::{sink::SinkExt, StreamExt};
use network::{
    application::storage::PeerMetadataStorage,
    connectivity_manager::DiscoverySource,
    protocols::network::{ApplicationNetworkSender, Event},
    ConnectivityRequest,
};
use network_builder::builder::NetworkBuilder;
use rand::{prelude::StdRng, rngs::OsRng, Rng, SeedableRng};
use state_sync_v1::network::{StateSyncEvents, StateSyncSender};
use std::{
    collections::HashSet,
    fmt,
    ops::Add,
    time::{Duration, Instant},
};
use structopt::StructOpt;
use tokio::runtime::{Builder, Handle};

const EXPERIMENT_BUFFER_SECS: u64 = 900;

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
    #[structopt(long, default_value = "1", help = "Number of stubbed nodes")]
    pub num_stubbed: usize,
}

pub struct LoadTest {
    mempool: bool,
    state_sync: bool,
    emit_txn: bool,
    duration: u64,
    num_stubbed: usize,
}

impl ExperimentParam for LoadTestParams {
    type E = LoadTest;
    fn build(self, _cluster: &Cluster) -> Self::E {
        LoadTest {
            mempool: self.mempool,
            state_sync: self.state_sync,
            emit_txn: self.emit_txn,
            duration: self.duration,
            num_stubbed: self.num_stubbed,
        }
    }
}

#[async_trait]
impl Experiment for LoadTest {
    fn affected_validators(&self) -> HashSet<String> {
        HashSet::new()
    }

    async fn run(&mut self, context: &mut Context<'_>) -> anyhow::Result<()> {
        let mut txn_emitter = TxnEmitter::new(
            &mut context.treasury_compliance_account,
            &mut context.designated_dealer_account,
            context.cluster.random_validator_instance().rest_client(),
            TransactionFactory::new(context.cluster.chain_id),
            StdRng::from_seed(OsRng.gen()),
        );

        // spin up StubbedNode
        let vfn = context.cluster.random_fullnode_instance();
        info!("Node {:?} is selected", vfn.peer_name());
        let vfn_endpoint = format!("http://{}:{}/v1", vfn.ip(), vfn.ac_port());
        let network_runtime = Builder::new_multi_thread()
            .thread_name("stubbed-node-network")
            .enable_all()
            .build()
            .expect("Failed to start runtime. Won't be able to start networking.");
        let mut stubbed_node = get_stubbed_nodes(
            vfn_endpoint,
            network_runtime.handle().clone(),
            self.num_stubbed,
        )
        .await;
        let mut emit_job = None;
        let mut mempool_handlers: Vec<_> = vec![];
        let mut state_sync_handlers: Vec<_> = vec![];
        let mut mempool_task = vec![];
        let mut state_sync_task = vec![];
        let duration = Duration::from_secs(self.duration);

        for node in &mut stubbed_node {
            mempool_handlers.push(
                node.mempool_handle
                    .take()
                    .expect("missing mempool network handles"),
            );
            state_sync_handlers.push(
                node.state_sync_handle
                    .take()
                    .expect("missing state sync network handles"),
            );
        }

        if self.emit_txn {
            // emit txns to JSON RPC
            // spawn future
            emit_job = Some(
                txn_emitter
                    .start_job(crate::util::emit_job_request_for_instances(
                        context.cluster.fullnode_instances().to_vec(),
                        context.global_emit_job_request,
                        0,
                        0,
                    ))
                    .await?,
            );
        }

        if self.mempool {
            // spawn mempool load test
            for (mempool_sender, mempool_events) in mempool_handlers {
                mempool_task.push(tokio::task::spawn(mempool_load_test(
                    duration,
                    mempool_sender,
                    mempool_events,
                )));
            }
        }

        if self.state_sync {
            // spawn state sync load test
            for (state_sync_sender, state_sync_events) in state_sync_handlers {
                state_sync_task.push(tokio::task::spawn(state_sync_load_test(
                    duration,
                    state_sync_sender,
                    state_sync_events,
                )));
            }
        }

        // await on all spawned tasks
        tokio::time::sleep(Duration::from_secs(self.duration)).await;
        if let Some(j) = emit_job {
            let stats = txn_emitter.stop_job(j).await;
            let mut sender = &mut context.root_account;
            let receiver = diem_root_address();
            let tx_factory = TransactionFactory::new(ChainId::test());
            let dummy_tx = gen_transfer_txn_request(&mut sender, &receiver, 0, &tx_factory, 0);
            let total_byte = dummy_tx.raw_txn_bytes_len() as u64 * stats.submitted;
            info!("Total tx emitter stats: {}, bytes: {}", stats, total_byte);
            info!(
                "Average rate: {}, {} bytes/s",
                stats.rate(Duration::from_secs(self.duration)),
                total_byte / Duration::from_secs(self.duration).as_secs()
            );
        }

        let mut mempool_stats = MempoolStats::default();
        for task in mempool_task {
            let stats = task.await?.expect("failed mempool load test task");
            mempool_stats = mempool_stats + stats;
        }
        if self.mempool {
            info!("Total mempool stats: {}", mempool_stats);
            info!(
                "Average rate: {}",
                mempool_stats.rate(Duration::from_secs(self.duration))
            );
        }

        let mut state_sync_stats = StateSyncStats::default();
        for task in state_sync_task {
            let stats = task.await?.expect("failed state sync load test task");
            state_sync_stats = state_sync_stats + stats;
        }
        if self.state_sync {
            info!("Total state sync stats: {}", state_sync_stats);
            info!(
                "Average rate: {}",
                state_sync_stats.rate(Duration::from_secs(self.duration))
            );
        }

        // create blocking context to drop stubbed node's runtime in
        // We cannot drop a runtime in an async context where blocking is not allowed - otherwise,
        // this thread will panic.
        tokio::task::spawn_blocking(move || {
            drop(network_runtime);
        })
        .await?;

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

async fn get_stubbed_nodes(
    endpoint: String,
    runtime_handle: Handle,
    num_of_nodes: usize,
) -> Vec<StubbedNode> {
    let mut nodes = vec![];
    for i in 0..num_of_nodes {
        nodes.push(StubbedNode::launch(endpoint.clone(), runtime_handle.clone(), i).await);
    }
    nodes
}

// An actor that can participate in DiemNet
// Connects to VFN via on-chain discovery and interact with it via mempool and state sync protocol
// It is 'stubbed' in the sense that it has no real node components running and only network stubs
// that interact with the remote VFN via DiemNet mempool and state sync protocol
struct StubbedNode {
    pub mempool_handle: Option<(MempoolNetworkSender, MempoolNetworkEvents)>,
    pub state_sync_handle: Option<(StateSyncSender, StateSyncEvents)>,
}

impl StubbedNode {
    async fn launch(node_endpoint: String, runtime_handle: Handle, index: usize) -> Self {
        // generate seed peers config from querying node endpoint
        let seed_peers =
            seed_peer_generator::utils::gen_validator_full_node_seed_peer_config(node_endpoint)
                .unwrap();

        // build sparse network runner

        let mut pfn_config = NodeConfig::default_for_public_full_node();

        // some sanity checks on the network the stubbed node will be running in
        assert_eq!(
            pfn_config.full_node_networks.len(),
            1,
            "expected only one fn network for PFN"
        );
        let network_config = &mut pfn_config.full_node_networks[0];
        // this dummy listen address is not used
        network_config.listen_address = format!("/ip4/127.0.0.1/tcp/{}", 6180 + index)
            .parse()
            .unwrap();
        assert_eq!(network_config.network_id, NetworkId::Public);

        let mut network_builder = NetworkBuilder::create(
            ChainId::test(),
            pfn_config.base.role,
            network_config,
            TimeService::real(),
            None,
            PeerMetadataStorage::new(&[network_config.network_id]),
        );

        let state_sync_handle = Some(
            network_builder.add_p2p_service(&state_sync_v1::network::network_endpoint_config()),
        );

        let mempool_handle = Some(network_builder.add_p2p_service(
            &diem_mempool::network::network_endpoint_config(
                pfn_config.mempool.max_broadcasts_per_peer,
            ),
        ));

        network_builder.build(runtime_handle);
        network_builder.start();

        // feed the network builder the seed peer config
        let mut conn_req_tx = network_builder
            .conn_mgr_reqs_tx()
            .expect("expecting connectivity mgr to exist after adding protocol handler");

        conn_req_tx
            .send(ConnectivityRequest::UpdateDiscoveredPeers(
                DiscoverySource::OnChainValidatorSet,
                seed_peers,
            ))
            .await
            .expect("failed to send conn req");

        Self {
            mempool_handle,
            state_sync_handle,
        }
    }
}

async fn mempool_load_test(
    duration: Duration,
    sender: MempoolNetworkSender,
    mut events: MempoolNetworkEvents,
) -> Result<MempoolStats> {
    let new_peer_event = events.select_next_some().await;
    let vfn = if let Event::NewPeer(metadata) = new_peer_event {
        metadata.remote_peer_id
    } else {
        return Err(anyhow::format_err!(
            "received unexpected network event for mempool load test"
        ));
    };

    let mut bytes = 0_u64;
    let mut msg_num = 0_u64;
    let task_start = Instant::now();
    while Instant::now().duration_since(task_start) < duration {
        let msg = diem_mempool::network::MempoolSyncMsg::BroadcastTransactionsRequest {
            request_id: bcs::to_bytes("request_id")?,
            transactions: vec![], // TODO submit actual txns
        };
        // TODO log stats for bandwidth sent to remote peer to MempoolResult
        bytes += bcs::to_bytes(&msg)?.len() as u64;
        msg_num += 1;
        sender.send_to(vfn, msg)?;

        // await ACK from remote peer
        let _response = events.select_next_some().await;
    }

    Ok(MempoolStats {
        bytes,
        tx_num: 0,
        msg_num,
    })
}

#[derive(Debug, Default)]
struct MempoolStats {
    bytes: u64,
    tx_num: u64,
    msg_num: u64,
}

#[derive(Debug, Default)]
pub struct MempoolStatsRate {
    pub bytes: u64,
    pub tx_num: u64,
    pub msg_num: u64,
}

impl MempoolStats {
    pub fn rate(&self, window: Duration) -> MempoolStatsRate {
        MempoolStatsRate {
            bytes: self.bytes / window.as_secs(),
            tx_num: self.tx_num / window.as_secs(),
            msg_num: self.msg_num / window.as_secs(),
        }
    }
}

impl Add for MempoolStats {
    type Output = MempoolStats;

    fn add(self, other: MempoolStats) -> MempoolStats {
        MempoolStats {
            bytes: self.bytes + other.bytes,
            tx_num: self.tx_num + other.tx_num,
            msg_num: self.msg_num + other.msg_num,
        }
    }
}

impl fmt::Display for MempoolStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "exchanged {} messages, {} bytes",
            self.msg_num, self.bytes,
        )
    }
}

impl fmt::Display for MempoolStatsRate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "exchanged {} messages/s, {} bytes/s",
            self.msg_num, self.bytes,
        )
    }
}

async fn state_sync_load_test(
    duration: Duration,
    sender: StateSyncSender,
    mut events: StateSyncEvents,
) -> Result<StateSyncStats> {
    let new_peer_event = events.select_next_some().await;
    let vfn = if let Event::NewPeer(metadata) = new_peer_event {
        metadata.remote_peer_id
    } else {
        return Err(anyhow::format_err!(
            "received unexpected network event for state sync load test"
        ));
    };

    let chunk_request = state_sync_v1::chunk_request::GetChunkRequest::new(
        1,
        1,
        1000,
        state_sync_v1::chunk_request::TargetType::HighestAvailable {
            target_li: None,
            timeout_ms: 10_000,
        },
    );

    let task_start = Instant::now();
    let mut served_txns = 0_u64;
    let mut bytes = 0_u64;
    let mut msg_num = 0_u64;
    while Instant::now().duration_since(task_start) < duration {
        use state_sync_v1::network::StateSyncMessage::*;

        let msg = GetChunkRequest(Box::new(chunk_request.clone()));
        bytes += bcs::to_bytes(&msg)?.len() as u64;
        msg_num += 1;
        sender.send_to(vfn, msg)?;

        // await response from remote peer
        let response = events.select_next_some().await;
        if let Event::Message(_remote_peer, GetChunkResponse(chunk_response)) = response {
            // TODO analyze response and update StateSyncResult with stats accordingly
            served_txns += chunk_response.txn_list_with_proof.transactions.len() as u64;
        }
    }
    Ok(StateSyncStats {
        served_txns,
        bytes,
        msg_num,
    })
}

#[derive(Debug, Default)]
struct StateSyncStats {
    served_txns: u64,
    bytes: u64,
    msg_num: u64,
}

#[derive(Debug, Default)]
pub struct StateSyncStatsRate {
    pub served_txns: u64,
    pub bytes: u64,
    pub msg_num: u64,
}

impl Add for StateSyncStats {
    type Output = StateSyncStats;

    fn add(self, other: StateSyncStats) -> StateSyncStats {
        StateSyncStats {
            served_txns: self.served_txns + other.served_txns,
            bytes: self.bytes + other.bytes,
            msg_num: self.msg_num + other.msg_num,
        }
    }
}

impl StateSyncStats {
    pub fn rate(&self, window: Duration) -> StateSyncStatsRate {
        StateSyncStatsRate {
            served_txns: self.served_txns / window.as_secs(),
            bytes: self.bytes / window.as_secs(),
            msg_num: self.msg_num / window.as_secs(),
        }
    }
}

impl fmt::Display for StateSyncStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "received {} txs, exchanged {} messages, {} bytes, ",
            self.served_txns, self.msg_num, self.bytes
        )
    }
}

impl fmt::Display for StateSyncStatsRate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "received {} txs/s, exchanged {} msg/s, {} bytes/s, ",
            self.served_txns, self.msg_num, self.bytes,
        )
    }
}
