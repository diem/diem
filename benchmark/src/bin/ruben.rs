// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

/// To run benchmarking experiment, RuBen creates two key required components:
/// * An object that implements LoadGenerator trait, which generates accounts and offline TXNs
///   that to be submitted during both setup stage and load-testing stage.
/// * Benchmarker: responsible for playing given requests (submit and wait TXN committed).
///
/// then it drives TXN submission/waiting process in the following flow:
///
///     // Generate accounts
///     faucet_account = bm.load_faucet_account();
///     accounts = txn_generator.gen_accounts(num_accounts);
///     bm.register_accounts(accounts);
///
///     // Generate and run setup requests
///     mint_txns = txn_generator.gen_setup_txn_requests(faucet_account, &mut accounts);
///     bm.submit_and_wait_txn_committed(faucet_account, setup_requests);
///
///     // Generate and run load requests
///     load_txns = txn_generator.gen_signed_txn_load(accounts);
///     bm.submit_and_wait_txn_committed(accounts, load_txns);
///
/// By conforming to the LoadGenerator APIs,
/// this flow is basically the same for different LoadGenerators/experiments.
use admission_control_proto::proto::admission_control_grpc::AdmissionControlClient;
use benchmark::{
    ruben_opt::{Opt, TransactionPattern},
    txn_generator::{
        convert_load_to_txn_requests, gen_repeated_txn_load, LoadGenerator,
        PairwiseTransferTxnGenerator, RingTransferTxnGenerator,
    },
    Benchmarker,
};
use client::AccountData;
use grpcio::{ChannelBuilder, EnvBuilder};
use logger::{self, prelude::*};
use metrics::metric_server::start_server;
use std::{ops::DerefMut, sync::Arc};

/// Play given TXN pattern with Benchmarker for several epochs and measure burst throughput,
/// e.g., the average committed txns per second. Since time is counted from submission
/// until all TXNs are committed, this measurement is in a sense the user-side throughput.
/// Each epoch plays the given TXN request pattern sequence repeatedly for several rounds.
pub(crate) fn measure_throughput<T: LoadGenerator + ?Sized>(
    bm: &mut Benchmarker,
    txn_generator: &mut T,
    faucet_account: &mut AccountData,
    args: &Opt,
) {
    // Generate testing accounts.
    let mut accounts: Vec<AccountData> = txn_generator.gen_accounts(args.num_accounts);
    bm.register_accounts(&accounts);

    // Submit setup/minting TXN requests.
    let setup_requests = txn_generator.gen_setup_txn_requests(faucet_account, &mut accounts);
    let mint_txns = convert_load_to_txn_requests(setup_requests);
    bm.mint_accounts(&mint_txns, faucet_account);

    // Submit TXN load and measure throughput.
    let mut txn_throughput_seq = vec![];
    for _ in 0..args.num_epochs {
        let repeated_tx_reqs = gen_repeated_txn_load(txn_generator, &mut accounts, args.num_rounds);
        let txn_throughput = bm.measure_txn_throughput(&repeated_tx_reqs, &mut accounts);
        txn_throughput_seq.push(txn_throughput);
    }
    info!(
        "{} epoch(s) of REQ/TXN throughput = {:?}",
        args.num_epochs, txn_throughput_seq
    );
}

fn create_ac_client(conn_addr: &str) -> AdmissionControlClient {
    let env_builder = Arc::new(EnvBuilder::new().name_prefix("ac-grpc-").build());
    let ch = ChannelBuilder::new(env_builder).connect(&conn_addr);
    AdmissionControlClient::new(ch)
}

/// Creat a vector of AdmissionControlClient and connect them to validators.
fn create_ac_clients(
    num_clients: usize,
    validator_addresses: &[String],
) -> Vec<AdmissionControlClient> {
    let mut clients: Vec<AdmissionControlClient> = vec![];
    for i in 0..num_clients {
        let index = i % validator_addresses.len();
        let client = create_ac_client(&validator_addresses[index]);
        clients.push(client);
    }
    clients
}

pub(crate) fn create_benchmarker_from_opt(args: &Opt) -> Benchmarker {
    // Create AdmissionControlClient instances.
    let clients = create_ac_clients(args.num_clients, &args.validator_addresses);
    let submit_rate = args.parse_submit_rate();
    // Ready to instantiate Benchmarker.
    Benchmarker::new(clients, args.stagger_range_ms, submit_rate)
}

/// Benchmarker is not a long-lived job, so starting a server and expecting it to be polled
/// continuously is not ideal. Directly pushing metrics when benchmarker is running
/// can be achieved by using Pushgateway.
fn try_start_metrics_server(args: &Opt) {
    if let Some(metrics_server_address) = &args.metrics_server_address {
        let address = metrics_server_address.clone();
        std::thread::spawn(move || {
            start_server(address);
        });
    }
}

fn main() {
    let _g = logger::set_default_global_logger(false, Some(256));
    info!("RuBen: the utility to (Ru)n (Ben)chmarker");
    let args = Opt::new_from_args();
    info!("Parsed arguments: {:#?}", args);
    try_start_metrics_server(&args);
    let mut bm = create_benchmarker_from_opt(&args);
    let mut faucet_account = bm.load_faucet_account(&args.faucet_key_file_path);
    let mut generator: Box<dyn LoadGenerator> = match args.txn_pattern {
        TransactionPattern::Ring => Box::new(RingTransferTxnGenerator::new()),
        TransactionPattern::Pairwise => Box::new(PairwiseTransferTxnGenerator::new()),
    };
    measure_throughput(&mut bm, generator.deref_mut(), &mut faucet_account, &args);
}

#[cfg(test)]
mod tests {
    use crate::{create_benchmarker_from_opt, measure_throughput};
    use benchmark::{
        ruben_opt::{Opt, TransactionPattern},
        txn_generator::RingTransferTxnGenerator,
        OP_COUNTER,
    };
    use libra_swarm::swarm::LibraSwarm;
    use rusty_fork::{rusty_fork_id, rusty_fork_test, rusty_fork_test_name};

    rusty_fork_test! {
        #[test]
        fn test_benchmarker_counters() {
            let (faucet_account_keypair, faucet_key_file_path, _temp_dir) =
                generate_keypair::load_faucet_key_or_create_default(None);
            let swarm = LibraSwarm::launch_swarm(
                4,      /* num_nodes */
                true,   /* disable_logging */
                faucet_account_keypair,
                false,  /* tee_logs */
                None,   /* config_dir */
                None,   /* template_path */
            );
            let mut args = Opt {
                validator_addresses: Vec::new(),
                debug_address: None,
                swarm_config_dir: Some(String::from(
                    swarm.dir.as_ref().unwrap().as_ref().to_str().unwrap(),
                )),
                // Don't start metrics server as we are not testing with prometheus.
                metrics_server_address: None,
                faucet_key_file_path,
                num_accounts: 4,
                free_lunch: 10_000_000,
                num_clients: 4,
                stagger_range_ms: 1,
                num_rounds: 4,
                num_epochs: 2,
                txn_pattern: TransactionPattern::Ring,
                submit_rate: None,
            };
            args.try_parse_validator_addresses();
            let mut bm = create_benchmarker_from_opt(&args);
            let mut faucet_account = bm.load_faucet_account(&args.faucet_key_file_path);
            let mut ring_generator = RingTransferTxnGenerator::new();
            measure_throughput(
                &mut bm,
                &mut ring_generator,
                &mut faucet_account,
                &args
            );
            let requested_txns = OP_COUNTER.counter("requested_txns").get();
            let created_txns = OP_COUNTER.counter("created_txns").get();
            let sign_failed_txns = OP_COUNTER.counter("sign_failed_txns").get();
            assert_eq!(requested_txns, created_txns + sign_failed_txns);
            let accepted_txns = OP_COUNTER.counter("submit_txns.Accepted").get();
            let committed_txns = OP_COUNTER.counter("committed_txns").get();
            let timedout_txns = OP_COUNTER.counter("timedout_txns").get();
            // Why `<=`: timedout TXNs in previous epochs can be committed in the next epoch.
            assert!(accepted_txns <= committed_txns + timedout_txns);
        }
    }
}
