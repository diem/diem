// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    load_generator::{gen_repeated_requests, LoadGenerator},
    ruben_opt::RubenOpt,
    BenchSummary, Benchmarker,
};
use admission_control_proto::proto::admission_control_grpc::AdmissionControlClient;
use client::AccountData;
use grpcio::{ChannelBuilder, EnvBuilder};
use logger::{self, prelude::*};
use metrics::metric_server::start_server;
use std::sync::Arc;

/// Creates a client for AC with a unique user-agent.
///
/// In a benchmark environment, we want to emulate multiple concurrent
/// clients, even though all of them originate from the same benchmarker
/// process. We add a unique user-agent to bypass some of gRPC default
/// optimization that group connections from the same source onto
/// a single completion queue (making requests sequntial).
///
/// index: unique identifier for the channel, to uniquify clients
fn create_ac_client(index: usize, conn_addr: &str) -> AdmissionControlClient {
    let env_builder = Arc::new(EnvBuilder::new().name_prefix("ac-grpc-").build());
    let ch = ChannelBuilder::new(env_builder)
        .primary_user_agent(&format!("grpc/benchmark-client-{}", index))
        .connect(&conn_addr);
    AdmissionControlClient::new(ch)
}

/// Creat a vector of AdmissionControlClient and connect them to validators.
pub fn create_ac_clients(
    num_clients: usize,
    validator_addresses: &[String],
) -> Vec<AdmissionControlClient> {
    let mut clients: Vec<AdmissionControlClient> = vec![];
    for i in 0..num_clients {
        let index = i % validator_addresses.len();
        let client = create_ac_client(i, &validator_addresses[index]);
        clients.push(client);
    }
    clients
}

pub fn create_benchmarker_from_opt(args: &RubenOpt) -> Benchmarker {
    // Create AdmissionControlClient instances.
    let clients = create_ac_clients(args.num_clients, &args.validator_addresses);
    let submit_rate = args.parse_submit_rate();
    // Ready to instantiate Benchmarker.
    Benchmarker::new(clients, args.stagger_range_ms, submit_rate)
}

/// Benchmarker is not a long-lived job, so starting a server and expecting it to be polled
/// continuously is not ideal. Directly pushing metrics when benchmarker is running
/// can be achieved by using Pushgateway.
pub fn try_start_metrics_server(args: &RubenOpt) {
    if let Some(metrics_server_address) = &args.metrics_server_address {
        let address = metrics_server_address.clone();
        std::thread::spawn(move || {
            start_server(address);
        });
    }
}

/// Play given TXNs with Benchmarker for several epochs and measure burst throughput,
/// e.g., the average committed txns per second. Since time is counted from submission
/// until all TXNs are committed, this measurement is in a sense the user-side throughput.
/// Each epoch plays the given TXN pattern sequence repeatedly for several rounds.
pub fn measure_throughput<T: LoadGenerator + ?Sized>(
    bm: &mut Benchmarker,
    generator: &mut T,
    faucet_account: &mut AccountData,
    num_accounts: u64,
    num_rounds: u64,
    num_epochs: u64,
) -> std::vec::Vec<BenchSummary> {
    // Generate testing accounts.
    let mut accounts: Vec<AccountData> = generator.gen_accounts(num_accounts);
    bm.register_accounts(&accounts);

    // Submit minting TXNs.
    let mint_txns = generator.gen_setup_requests(faucet_account, &mut accounts);
    bm.mint_accounts(&mint_txns, faucet_account);

    // Submit testing TXNs and measure throughput.
    let mut results = vec![];
    let mut throughput_seq = vec![];
    for _ in 0..num_epochs {
        let repeated_txn_reqs = gen_repeated_requests(generator, &mut accounts, num_rounds);
        let result = bm.measure_txn_throughput(&repeated_txn_reqs, &mut accounts, None);
        throughput_seq.push((result.req_throughput(), result.txn_throughput()));
        results.push(result);
    }
    info!(
        "{} epoch(s) of REQ/TXN throughputs: {:?}",
        num_epochs, throughput_seq,
    );
    results
}
