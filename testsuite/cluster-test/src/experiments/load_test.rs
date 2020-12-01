// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::instance::Instance;
use crate::{
    cluster::Cluster,
    experiments::{Context, Experiment, ExperimentParam},
    tx_emitter::{gen_transfer_txn_request, AccountData, EmitJobRequest},
};
use anyhow::Result;
use async_trait::async_trait;
use futures::executor::block_on;
use futures::future::join_all;
use futures::lock::Mutex;
use futures::{sink::SinkExt, FutureExt, StreamExt};
use libra_config::{config::NodeConfig, network_id::NetworkId};
use libra_crypto::x25519;
use libra_logger::info;
use libra_mempool::network::{MempoolNetworkEvents, MempoolNetworkSender};
use libra_network_address::NetworkAddress;
use libra_types::{
    account_config::libra_root_address, chain_id::ChainId, transaction::SignedTransaction,
};
use network::{
    connectivity_manager::DiscoverySource, protocols::network::Event, ConnectivityRequest,
};
use network_builder::builder::NetworkBuilder;
use rand::{rngs::ThreadRng, seq::IteratorRandom};
use state_synchronizer::network::{StateSynchronizerEvents, StateSynchronizerSender};
use std::borrow::{Borrow, BorrowMut};
use std::ops::Deref;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::{
    cmp::min,
    collections::{HashMap, HashSet},
    fmt, thread,
    time::{Duration, Instant},
};
use structopt::StructOpt;
use tokio::runtime::{Builder, Runtime};
use tokio::time;

const MAX_TXN_BATCH_SIZE: usize = 100; // Max transactions in mempool
const SEND_AMOUNT: u64 = 1;

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
    #[structopt(
        long,
        default_value = "100",
        help = "Number of accounts to mint before starting the experiment"
    )]
    pub num_accounts_to_mint: usize,
    #[structopt(long, default_value = "1", help = "Number of stubbed nodes")]
    pub num_of_stubbed_nodes: usize,
}

pub struct LoadTest {
    mempool: bool,
    state_sync: bool,
    emit_txn: bool,
    duration: u64,
    num_accounts_to_mint: usize,
    random_instances: Vec<Instance>,
}

impl ExperimentParam for LoadTestParams {
    type E = LoadTest;
    fn build(self, cluster: &Cluster) -> Self::E {
        let (ran, _) = cluster.split_n_fullnodes_random(self.num_of_stubbed_nodes);
        let random_instances = ran.into_fullnode_instances();
        info!("hhhhhh0 {}", random_instances.len());
        LoadTest {
            mempool: self.mempool,
            state_sync: self.state_sync,
            emit_txn: self.emit_txn,
            duration: self.duration,
            num_accounts_to_mint: self.num_accounts_to_mint,
            random_instances,
        }
    }
}

#[async_trait]
impl Experiment for LoadTest {
    fn affected_validators(&self) -> HashSet<String> {
        HashSet::new()
    }

    async fn run(&mut self, context: &mut Context<'_>) -> anyhow::Result<()> {
        // spin up StubbedNodes
        //let futures = self.random_instances.iter().enumerate().map(|(index, inst)|get_stubbed_node(inst, index));
        //let nodes = join_all(futures).await;
        let mut node1 = get_stubbed_node(&context.cluster.fullnode_instances()[0], 0).await;
        //let mut node2 = get_stubbed_node(&context.cluster.fullnode_instances()[1], 1).await;
        info!(
            "hhhhh pod name {:?}",
            context.cluster.fullnode_instances()[0]
                .instance_config()
                .pod_name()
        );
        //info!("hhhhh pod name {:?}", context.cluster.fullnode_instances()[1].instance_config().pod_name());

        let mut emit_job = None;
        let mut mempool_task = vec![];
        let mut state_sync_task = vec![];
        let mut report = vec![];
        let duration = Duration::from_secs(self.duration);
        // Mint a number of accounts
        context
            .tx_emitter
            .mint_accounts(
                &EmitJobRequest::for_instances(
                    context.cluster.validator_instances().to_vec(),
                    context.global_emit_job_request,
                    0,
                ),
                self.num_accounts_to_mint,
            )
            .await?;

        if self.emit_txn {
            // emit txns to JSON RPC
            // spawn future
            emit_job = Some(
                context
                    .tx_emitter
                    .start_job(EmitJobRequest::for_instances(
                        context.cluster.validator_instances().to_vec(),
                        context.global_emit_job_request,
                        0,
                    ))
                    .await?,
            );
        }

        if let Some(j) = emit_job {
            // await on all spawned tasks
            tokio::time::delay_for(Duration::from_secs(self.duration)).await;
            let stats = context.tx_emitter.stop_job(j).await;
            let full_node = context.cluster.random_fullnode_instance();
            let full_node_client = full_node.json_rpc_client();
            let mut sender = context
                .tx_emitter
                .load_libra_root_account(&full_node_client)
                .await?;
            let receiver = libra_root_address();
            let dummy_tx = gen_transfer_txn_request(&mut sender, &receiver, 0, ChainId::test(), 0);
            let total_byte = dummy_tx.raw_txn_bytes_len() as u64 * stats.submitted;
            report.push(format!(
                "Total tx emitter stats: {}, bytes: {}",
                stats, total_byte
            ));
            report.push(format!(
                "Average rate: {}, {} bytes/s",
                stats.rate(Duration::from_secs(self.duration)),
                total_byte / Duration::from_secs(self.duration).as_secs()
            ));
        }

        if self.mempool {
            // spawn mempool load test
            let (mempool_sender, mempool_events) = node1
                .mempool_handle
                .take()
                .expect("missing mempool network handles");
            mempool_task.push(Some(tokio::task::spawn(mempool_load_test(
                duration,
                mempool_sender,
                mempool_events,
                context.tx_emitter.accounts.clone(),
                context.cluster.chain_id,
            ))));
        }

        if self.state_sync {
            // spawn state sync load test
            let (state_sync_sender, state_sync_events) = node1
                .state_sync_handle
                .take()
                .expect("missing state sync network handles");
            state_sync_task.push(Some(tokio::task::spawn(state_sync_load_test(
                duration,
                state_sync_sender,
                state_sync_events,
            ))));
        }

        /*if let Some(t) = mempool_task {
            let stats = t.await?.expect("failed mempool load test task");
            report.push(format!("Total mempool stats: {}", stats));
            report.push(format!(
                "Average rate: {}",
                stats.rate(Duration::from_secs(self.duration))
            ));
        }*/
        for task in mempool_task {
            if let Some(t) = task {
                info!("hhhhhhh state syncs");
                let stats = t.await?.expect("");
                report.push(format!("Total mempool stats: {}", stats));
                report.push(format!(
                    "Average rate: {}",
                    stats.rate(Duration::from_secs(self.duration))
                ));
            } else {
                info!("hhhhhhh state syncs break out");
                break;
            }
        }

        for task in state_sync_task {
            if let Some(t) = task {
                let stats = t.await?.expect("");
                report.push(format!("Total state sync stats: {}", stats));
                report.push(format!(
                    "Average rate: {}",
                    stats.rate(Duration::from_secs(self.duration))
                ));
            } else {
                break;
            }
        }

        /*if let Some(t) = state_sync_task {
            let stats = t.await?.expect("failed state sync load test task");
            report.push(format!("Total state sync stats: {}", stats));
            report.push(format!(
                "Average rate: {}",
                stats.rate(Duration::from_secs(self.duration))
            ));
        }*/

        // create blocking context to drop stubbed node's runtime in
        // We cannot drop a runtime in an async context where blocking is not allowed - otherwise,
        // this thread will panic.
        tokio::task::spawn_blocking(move || {
            drop(node1);
        })
        .await?;

        for s in report {
            info!("{}", s);
        }

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

async fn get_stubbed_node(instance: &Instance, index: usize) -> StubbedNode {
    let vfn_endpoint = format!("http://{}:{}/v1", instance.ip(), instance.ac_port());
    StubbedNode::launch(vfn_endpoint, index).await
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
    async fn launch(node_endpoint: String, index: usize) -> Self {
        // generate seed peers config from querying node endpoint
        info!("hhhhhh endpoint {:?}, index = {}", node_endpoint, index);
        let seed_peers = seed_peer_generator::utils::gen_seed_peer_config(node_endpoint);

        // build sparse network runner

        let pfn_config = NodeConfig::default_for_public_full_node();

        let network_config = &pfn_config.full_node_networks[index];
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
    accounts: Vec<AccountData>,
    chain_id: ChainId,
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
    let mut tx_num = 0_u64;
    let task_start = Instant::now();
    while Instant::now().duration_since(task_start) < duration {
        let tx = gen_requests(accounts.clone(), chain_id);
        tx_num += tx.len() as u64;
        let msg = libra_mempool::network::MempoolSyncMsg::BroadcastTransactionsRequest {
            request_id: lcs::to_bytes("request_id")?,
            transactions: tx,
        };
        // TODO log stats for bandwidth sent to remote peer to MempoolResult
        bytes += lcs::to_bytes(&msg)?.len() as u64;
        msg_num += 1;
        sender.send_to(vfn, msg)?;

        // await ACK from remote peer
        let _response = events.select_next_some().await;
    }

    Ok(MempoolStats {
        bytes,
        tx_num,
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

impl fmt::Display for MempoolStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "submitted {} txs, exchanged {} messages, {} bytes",
            self.tx_num, self.msg_num, self.bytes,
        )
    }
}

impl fmt::Display for MempoolStatsRate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "submitted {} txs/s, exchanged {} messages/s, {} bytes/s",
            self.tx_num, self.msg_num, self.bytes,
        )
    }
}

async fn state_sync_load_test(
    duration: Duration,
    mut sender: StateSynchronizerSender,
    mut events: StateSynchronizerEvents,
) -> Result<StateSyncStats> {
    let new_peer_event = events.select_next_some().await;
    info!("hhhhhh111");
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
        1000,
        state_synchronizer::chunk_request::TargetType::HighestAvailable {
            target_li: None,
            timeout_ms: 10_000,
        },
    );

    let task_start = Instant::now();
    static served_txns1: AtomicU64 = AtomicU64::new(0);
    static bytes1: AtomicU64 = AtomicU64::new(0);
    static msg_num1: AtomicU64 = AtomicU64::new(0);
    let send = Arc::new(AtomicU64::new(0));
    let ack = send.clone();

    thread::spawn(move || {
        while Instant::now().duration_since(task_start) < duration {
            let msg = state_synchronizer::network::StateSynchronizerMsg::GetChunkRequest(Box::new(
                chunk_request.clone(),
            ));
            bytes1.fetch_add(lcs::to_bytes(&msg).unwrap().len() as u64, Ordering::SeqCst);
            msg_num1.fetch_add(1, Ordering::SeqCst);
            while send.load(Ordering::SeqCst) >= 5 {}
            let add = send.fetch_add(1, Ordering::SeqCst);
            info!("aaa pending ack: {}", add + 1);
            sender.send_to(vfn, msg);
        }
        info!("send thread ends");
    });
    while let Some(response) = events.next().await {
        info!("hhhhhhh");
        if let Event::Message(_remote_peer, payload) = response {
            if let state_synchronizer::network::StateSynchronizerMsg::GetChunkResponse(
                chunk_response,
            ) = payload
            {
                let temp = chunk_response.txn_list_with_proof.transactions.len() as u64;
                // TODO analyze response and update StateSyncResult with stats accordingly
                served_txns1.fetch_add(temp, Ordering::SeqCst);
                let sub = ack.fetch_sub(1, Ordering::SeqCst);
                info!("bbb pending ack: {}", sub - 1);
            }
        }
    }

    let served_txns = served_txns1.load(Ordering::Relaxed);
    let bytes = bytes1.load(Ordering::Relaxed);
    let msg_num = msg_num1.load(Ordering::Relaxed);

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

fn gen_requests(accounts: Vec<AccountData>, chain_id: ChainId) -> Vec<SignedTransaction> {
    let mut rng = ThreadRng::default();
    let batch_size = min(MAX_TXN_BATCH_SIZE, accounts.len());
    let mut accounts = accounts.into_iter().choose_multiple(&mut rng, batch_size);
    let addresses: Vec<_> = accounts.iter().map(|d| d.address).collect();
    let mut requests = Vec::with_capacity(accounts.len());
    for sender in &mut accounts {
        let receiver = addresses.clone().into_iter().choose(&mut rng).expect("");
        let request = gen_transfer_txn_request(sender, &receiver, SEND_AMOUNT, chain_id, 0);
        requests.push(request);
    }
    requests
}
