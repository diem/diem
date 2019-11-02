// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    cli_opt::BenchOpt,
    load_generator::{gen_repeated_requests, LoadGenerator},
    BenchSummary, Benchmarker,
};
use admission_control_proto::proto::admission_control::AdmissionControlClient;
use client::AccountData;
use grpcio::{ChannelBuilder, EnvBuilder};
use libra_logger::{self, prelude::*};
use libra_metrics::metric_server::start_server;
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

pub fn create_benchmarker_from_opt(args: &BenchOpt) -> Benchmarker {
    // Create AdmissionControlClient instances.
    let clients = create_ac_clients(args.num_clients, &args.validator_addresses);

    let submit_rate = args.parse_submit_rate();
    // Ready to instantiate Benchmarker.
    Benchmarker::new(clients, args.stagger_range_ms, submit_rate)
}

/// Benchmarker is not a long-lived job, so starting a server and expecting it to be polled
/// continuously is not ideal. Directly pushing metrics when benchmarker is running
/// can be achieved by using Pushgateway.
pub fn try_start_metrics_server(args: &BenchOpt) {
    if let Some(metrics_server_address) = &args.metrics_server_address {
        let address = metrics_server_address.clone();
        std::thread::spawn(move || {
            start_server(address, 9101, false);
        });
    }
}

/// Generate a group of new accounts, and mint them using Benchmarker before returning them.
pub fn gen_and_mint_accounts<T: LoadGenerator + ?Sized>(
    bm: &mut Benchmarker,
    generator: &mut T,
    faucet_account: &mut AccountData,
    num_accounts: u64,
) -> Vec<AccountData> {
    // Generate testing accounts.
    let mut accounts: Vec<AccountData> = generator.gen_accounts(num_accounts);
    bm.register_accounts(&accounts);
    // Mint generated accounts
    let setup_requests = generator.gen_setup_requests(faucet_account, &mut accounts);
    bm.mint_accounts(&setup_requests, faucet_account);
    accounts
}

/// Play given TXNs with Benchmarker for several epochs and measure throughput,
/// e.g., the average committed txns per second. Since time is counted from submission
/// until all TXNs are committed, this measurement is in a sense the user-side throughput.
/// Each epoch plays the given TXN pattern sequence repeatedly for several rounds.
/// Return list of summaries for all epochs.
pub fn measure_throughput<T: LoadGenerator + ?Sized>(
    bm: &mut Benchmarker,
    generator: &mut T,
    faucet_account: &mut AccountData,
    num_accounts: u64,
    num_rounds: u64,
    num_epochs: u64,
) -> std::vec::Vec<BenchSummary> {
    // Generate testing accounts.
    let mut accounts = gen_and_mint_accounts(bm, generator, faucet_account, num_accounts);
    let account_chunk_size = accounts.len();
    let mut results = vec![];
    let mut throughput_seq = vec![];
    for _ in 0..num_epochs {
        // Submit testing TXNs and measure throughput.
        let result = run_benchmarker_at_const_rate(
            bm,
            &mut accounts,
            generator,
            None, /* submit_rate is included in bm */
            account_chunk_size,
            num_rounds,
            1, /* record result epoch by epoch */
        );
        throughput_seq.push((result.req_throughput(), result.txn_throughput()));
        results.push(result);
    }
    info!(
        "{} epoch(s) of REQ/TXN throughputs: {:?}",
        num_epochs, throughput_seq,
    );
    results
}

/// Generate TXNs, submit them at constant submission rate, and measure TXN throughput.
/// Run this process for several epochs.
/// Aggregate each epoch's running result into a single BenchSummary and return it.
fn run_benchmarker_at_const_rate<T: LoadGenerator + ?Sized>(
    bm: &mut Benchmarker,
    accounts: &mut [AccountData],
    generator: &mut T,
    rate: Option<u64>,
    account_chunk_size: usize,
    num_rounds: u64,
    num_epochs: u64,
) -> BenchSummary {
    let (mut total_submitted, mut total_accepted, mut total_committed) = (0, 0, 0);
    let (mut total_submit_duration, mut total_wait_duration) = (0, 0);
    for _ in 0..num_epochs {
        let mut txn_reqs = vec![];
        let now = time::Instant::now();
        // Generate new TXNs with only a chunk of accounts to avoid exceeding mempool limit.
        for account_chunk in accounts.chunks_mut(account_chunk_size as usize) {
            let txn_req_chunk = gen_repeated_requests(generator, account_chunk, num_rounds);
            txn_reqs.extend(txn_req_chunk.into_iter());
        }
        info!(
            "Generate {} TXNs within {} ms.",
            txn_reqs.len(),
            now.elapsed().as_millis()
        );
        let result = bm.measure_txn_throughput(&txn_reqs, accounts, rate);
        total_submitted += txn_reqs.len();
        total_accepted += result.num_accepted;
        total_committed += result.num_committed;
        total_submit_duration += result.submit_duration_ms;
        total_wait_duration += result.wait_duration_ms;
    }
    BenchSummary {
        num_submitted: total_submitted,
        num_accepted: total_accepted,
        num_committed: total_committed,
        submit_duration_ms: total_submit_duration,
        wait_duration_ms: total_wait_duration,
    }
}

/// Search the maximum throughput by increasing submission rate from lower_bound to upper_bound.
/// Search may end earilier if commit ratio < COMMIT_RATIO_THRESHOLD.
/// Return the max observed TXN throughput, along with the request throughput.
pub fn linear_search_max_throughput<T: LoadGenerator + ?Sized>(
    bm: &mut Benchmarker,
    generator: &mut T,
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
        let accounts = gen_and_mint_accounts(bm, generator, faucet_account, num_accounts);
        growing_accounts.extend(accounts.into_iter());
        info!(
            "Sending at constant rate {} TPS per client with {} accounts.",
            rate,
            growing_accounts.len()
        );
        let result = run_benchmarker_at_const_rate(
            bm,
            &mut growing_accounts,
            generator,
            Some(rate),
            num_accounts as usize,
            num_rounds,
            num_epochs,
        );
        let (req_throughput, txn_throughput) = (result.req_throughput(), result.txn_throughput());
        let (num_submitted, num_committed) = (result.num_submitted, result.num_committed);
        if max_result.1 < txn_throughput {
            max_result = (req_throughput, txn_throughput);
        }
        info!(
            "#submitted = {}, #committed = {}, avg REQ/TXN throughput = {:.2}/{:.2}.",
            num_submitted, num_committed, req_throughput, txn_throughput
        );
        let commit_ratio = num_committed as f64 / num_submitted as f64;
        info!(
            "Commit ratio at submit rate {} per client is {:.4}.",
            rate, commit_ratio
        );
        if commit_ratio < COMMIT_RATIO_THRESHOLD {
            info!(
                "Search ends ealier as commit ratio {:.4} < {:.2}.",
                commit_ratio, COMMIT_RATIO_THRESHOLD,
            );
            break;
        }
        rate += inc_step;
    }
    info!(
        "Search result: max REQ/TXN throughput = {:?} TPS.",
        max_result
    );
    max_result
}
