// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    ruben_opt::RubenOpt,
    txn_generator::{convert_load_to_txn_requests, gen_repeated_txn_load, LoadGenerator},
    Benchmarker,
};
use admission_control_proto::proto::admission_control_grpc::AdmissionControlClient;
use client::AccountData;
use grpcio::{ChannelBuilder, EnvBuilder};
use logger::{self, prelude::*};
use metrics::metric_server::start_server;
use std::{sync::Arc, time};

const COMMIT_RATIO_THRESHOLD: f64 = 0.7;

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

pub fn gen_and_mint_accounts<T: LoadGenerator + ?Sized>(
    bm: &mut Benchmarker,
    txn_generator: &mut T,
    faucet_account: &mut AccountData,
    num_accounts: u64,
) -> Vec<AccountData> {
    // Generate testing accounts.
    let mut accounts: Vec<AccountData> = txn_generator.gen_accounts(num_accounts);
    bm.register_accounts(&accounts);

    // Submit setup/minting TXN requests.
    let setup_requests = txn_generator.gen_setup_txn_requests(faucet_account, &mut accounts);
    let mint_txns = convert_load_to_txn_requests(setup_requests);
    bm.mint_accounts(&mint_txns, faucet_account);
    accounts
}

/// Play given TXN pattern with Benchmarker for several epochs and measure burst throughput,
/// e.g., the average committed txns per second. Since time is counted from submission
/// until all TXNs are committed, this measurement is in a sense the user-side throughput.
/// Each epoch plays the given TXN request pattern sequence repeatedly for several rounds.
pub fn measure_throughput<T: LoadGenerator + ?Sized>(
    bm: &mut Benchmarker,
    txn_generator: &mut T,
    faucet_account: &mut AccountData,
    num_accounts: u64,
    num_rounds: u64,
    num_epochs: u64,
) -> std::vec::Vec<(f64, f64)> {
    // Generate testing accounts.
    let mut accounts = gen_and_mint_accounts(bm, txn_generator, faucet_account, num_accounts);

    // Submit TXN load and measure throughput.
    let mut txn_throughput_seq = vec![];
    for _ in 0..num_epochs {
        let repeated_tx_reqs = gen_repeated_txn_load(txn_generator, &mut accounts, num_rounds);
        let (_, _, request_throughput, txn_throughput) =
            bm.measure_txn_throughput(&repeated_tx_reqs, &mut accounts, None);
        txn_throughput_seq.push((request_throughput, txn_throughput));
    }
    info!(
        "{} epoch(s) of REQ/TXN throughput = {:?}",
        num_epochs, txn_throughput_seq
    );
    txn_throughput_seq
}

/// Run benchmarker at constant submission rate for several epochs. Use the averaged result
/// to check if Libra network is able to absorb TXNs at the submission speed.
fn run_benchmarker_at_const_rate<T: LoadGenerator + ?Sized>(
    bm: &mut Benchmarker,
    accounts: &mut [AccountData],
    txn_generator: &mut T,
    rate: u64,
    num_accounts: u64,
    num_rounds: u64,
    num_epochs: u64,
) -> (usize, usize, f64, f64) {
    let (mut total_submitted, mut total_committed) = (0, 0);
    let (mut avg_req_throughput, mut avg_txn_throughput) = (0.0f64, 0.0f64);
    for _ in 0..num_epochs {
        let mut txn_reqs = vec![];
        let now = time::Instant::now();
        for account_chunk in accounts.chunks_mut(num_accounts as usize) {
            let txn_req_chunk = gen_repeated_txn_load(txn_generator, account_chunk, num_rounds);
            txn_reqs.extend(txn_req_chunk.into_iter());
        }
        info!(
            "Generate {} TXNs within {} ms",
            txn_reqs.len(),
            now.elapsed().as_millis()
        );
        let (_, num_committed, req_throughput, txn_throughput) =
            bm.measure_txn_throughput(&txn_reqs, accounts, Some(rate));
        total_submitted += txn_reqs.len();
        total_committed += num_committed;
        avg_req_throughput += req_throughput;
        avg_txn_throughput += txn_throughput;
    }
    avg_req_throughput /= num_epochs as f64;
    avg_txn_throughput /= num_epochs as f64;
    (
        total_submitted,
        total_committed,
        avg_req_throughput,
        avg_txn_throughput,
    )
}

/// Search the maximum throughput between range SEARCH_UPPER_BOUND * [1/10, 1].
/// Also consider the runs that fails the check as sometimes request throughput is hard to meet.
/// Return the request/TXN throughput pairs for success runs.
pub fn linear_search_max_throughput<T: LoadGenerator + ?Sized>(
    bm: &mut Benchmarker,
    txn_generator: &mut T,
    faucet_account: &mut AccountData,
    lower_bound: u64,
    upper_bound: u64,
    inc_step: u64,
    num_accounts: u64,
    num_rounds: u64,
    num_epochs: u64,
) -> (f64, f64) {
    let mut rate = lower_bound;
    let mut max_result = (0.0f64, 0.0f64);
    let mut growing_accounts = vec![];
    while rate <= upper_bound {
        let accounts = gen_and_mint_accounts(bm, txn_generator, faucet_account, num_accounts);
        growing_accounts.extend(accounts.into_iter());
        info!(
            "Sending at constant rate {} TPS per client with {} accounts.",
            rate,
            growing_accounts.len()
        );
        let (num_submitted, num_committed, req_throughput, txn_throughput) =
            run_benchmarker_at_const_rate(
                bm,
                &mut growing_accounts,
                txn_generator,
                rate,
                num_accounts,
                num_rounds,
                num_epochs,
            );
        if max_result.1 < txn_throughput {
            max_result.0 = req_throughput;
            max_result.1 = txn_throughput;
        }
        info!(
            "#submitted = {}, #committed = {}, Avg REQ/TXN throughput = {:.2}/{:.2}.",
            num_submitted, num_committed, req_throughput, txn_throughput
        );
        let commit_ratio = num_committed as f64 / num_submitted as f64;
        info!(
            "Commit ratio at submit rate {} per client is {:.4}",
            rate, commit_ratio
        );
        if commit_ratio < COMMIT_RATIO_THRESHOLD {
            info!(
                "Search ends ealier as commit ratio {:.4} < {:.2}",
                commit_ratio, COMMIT_RATIO_THRESHOLD,
            );
            break;
        }
        rate += inc_step;
    }
    info!(
        "Search result: max REQ/TXN throughput = {:?} TPS",
        max_result
    );
    max_result
}
