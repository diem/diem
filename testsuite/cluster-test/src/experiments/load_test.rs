// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::{
    cluster::Cluster,
    experiments::{Context, Experiment, ExperimentParam},
    tx_emitter::{gen_transfer_txn_request, EmitJobRequest},
};
use anyhow::Result;
use async_trait::async_trait;
use diem_config::{config::NodeConfig, network_id::NetworkId};
use diem_crypto::x25519;
use diem_logger::info;
use diem_mempool::network::{MempoolNetworkEvents, MempoolNetworkSender};
use diem_network_address::NetworkAddress;
use diem_types::{account_config::diem_root_address, chain_id::ChainId};
use futures::{sink::SinkExt, StreamExt, FutureExt};
use network::{
    connectivity_manager::DiscoverySource, protocols::network::Event, ConnectivityRequest,
};
use network_builder::builder::NetworkBuilder;
use state_synchronizer::network::{StateSynchronizerEvents, StateSynchronizerSender, StateSynchronizerMsg};
use std::{
    collections::{HashMap, HashSet},
    fmt,
    time::{Duration, Instant},
};
use structopt::StructOpt;
use tokio::runtime::{Builder, Runtime};
use network::constants::MAX_CONCURRENT_NETWORK_REQS;

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
    #[structopt(long, default_value = "100")]
    pub debug: u64,
}

pub struct LoadTest {
    mempool: bool,
    state_sync: bool,
    emit_txn: bool,
    duration: u64,
    debug: u64,
}

impl ExperimentParam for LoadTestParams {
    type E = LoadTest;
    fn build(self, _cluster: &Cluster) -> Self::E {
        LoadTest {
            mempool: self.mempool,
            state_sync: self.state_sync,
            emit_txn: self.emit_txn,
            duration: self.duration,
            debug: self.debug,
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
                self.debug,
            )));
        }

        // await on all spawned tasks
        tokio::time::delay_for(Duration::from_secs(self.duration)).await;
        if let Some(j) = emit_job {
            let stats = context.tx_emitter.stop_job(j).await;
            let full_node = context.cluster.random_fullnode_instance();
            let full_node_client = full_node.json_rpc_client();
            let mut sender = context
                .tx_emitter
                .load_diem_root_account(&full_node_client)
                .await?;
            let receiver = diem_root_address();
            let dummy_tx = gen_transfer_txn_request(&mut sender, &receiver, 0, ChainId::test(), 0);
            let total_byte = dummy_tx.raw_txn_bytes_len() as u64 * stats.submitted;
            info!("Total tx emitter stats: {}, bytes: {}", stats, total_byte);
            info!(
                "Average rate: {}, {} bytes/s",
                stats.rate(Duration::from_secs(self.duration)),
                total_byte / Duration::from_secs(self.duration).as_secs()
            );
        }

        if let Some(t) = mempool_task {
            let stats = t.await?.expect("failed mempool load test task");
            info!("Total mempool stats: {}", stats);
            info!(
                "Average rate: {}",
                stats.rate(Duration::from_secs(self.duration))
            );
        }

        if let Some(t) = state_sync_task {
            let stats = t.await?.expect("failed state sync load test task");
            info!("Total state sync stats: {}", stats);
            info!(
                "Average rate: {}",
                stats.rate(Duration::from_secs(self.duration))
            );
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

// An actor that can participate in DiemNet
// Connects to VFN via on-chain discovery and interact with it via mempool and state sync protocol
// It is 'stubbed' in the sense that it has no real node components running and only network stubs
// that interact with the remote VFN via DiemNet mempool and state sync protocol
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

        let mut config = state_synchronizer::network::network_endpoint_config();
        config.3 = MAX_CONCURRENT_NETWORK_REQS;
        let state_sync_handle = Some(
            network_builder
                .add_protocol_handler(config),
        );

        let mempool_handle = Some(network_builder.add_protocol_handler(
            diem_mempool::network::network_endpoint_config(
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
) -> Result<MempoolStats> {
    let new_peer_event = events.select_next_some().await;
    let vfn = if let Event::NewPeer(peer_id, _) = new_peer_event {
        peer_id
    } else {
        return Err(anyhow::format_err!(
            "received unexpected network event for mempool load test"
        ));
    };

    let mut bytes = 0_u64;
    let mut msg_num = 0_u64;
    let task_start = Instant::now();
    let mut pending = 0_usize;
    while Instant::now().duration_since(task_start) < duration {
        if pending == 0 {
            let msg = diem_mempool::network::MempoolSyncMsg::BroadcastTransactionsRequest {
                request_id: lcs::to_bytes("request_id")?,
                transactions: vec![], // TODO submit actual txns
            };
            bytes += lcs::to_bytes(&msg)?.len() as u64;
            sender.send_to(vfn, msg)?;
            pending += 1;
        }
        // await response from remote peer
        while let Some(_response) = events.select_next_some().now_or_never() {
            pending -= 1;
            msg_num += 1;
        }
    }
    // wait for all msgs are processed and acks back
    while pending > 0 {
        let response = events.select_next_some().await;
        if let Event::Message(_remote_peer, _payload) = response {
            msg_num += 1;
            pending -= 1;
        }
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
    tx_num: u64, // TODO
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

fn process_poll_msg(response: Event<StateSynchronizerMsg>) -> Option<usize> {
    if let Event::Message(_remote_peer, payload) = response {
        if let state_synchronizer::network::StateSynchronizerMsg::GetChunkResponse(
            chunk_response,
        ) = payload
        {
            return Some(chunk_response.txn_list_with_proof.transactions.len());
        }
    }
    // this msg is not an ack back
    None
}

async fn state_sync_load_test(
    duration: Duration,
    mut sender: StateSynchronizerSender,
    mut events: StateSynchronizerEvents,
    debug: u64,
) -> Result<StateSyncStats> {
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
        1,
        250,
        state_synchronizer::chunk_request::TargetType::HighestAvailable {
            target_li: None,
            timeout_ms: 10_000,
        },
    );

    let task_start = Instant::now();
    let mut served_txns = 0_u64;
    let mut bytes = 0_u64;
    let mut msg_num = 0_u64;
    let mut pending = 0_usize;
    while Instant::now().duration_since(task_start) < duration {
        if pending == 0 {
            let msg = state_synchronizer::network::StateSynchronizerMsg::GetChunkRequest(Box::new(
                chunk_request.clone(),
            ));
            bytes += lcs::to_bytes(&msg)?.len() as u64;
            sender.send_to(vfn, msg)?;
            pending += 1;
        }
        // await response from remote peer
        while let Some(response) = events.select_next_some().now_or_never() {
            if let Some(txs_num) = process_poll_msg(response) {
                served_txns += txs_num as u64;
                pending -= 1;
                msg_num += 1;
                info!("hhhhhh txn = {}, pending = {}", txs_num, pending);
            }
        }
    }
    info!("hhhhhhh pending = {}", pending);
    // wait for all msgs are processed and acks back
    while pending > 0 {
        let response = events.select_next_some().await;
        info!("hhhhh get inside");
        if let Some(txs_num) = process_poll_msg(response) {
            info!("hhhhh get inside tx = {}", txs_num);
            served_txns += txs_num as u64;
            pending -= 1;
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
