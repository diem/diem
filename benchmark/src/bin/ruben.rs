// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

/// To run benchmarking experiment, RuBen creates two key required components:
/// * An object that implements LoadGenerator trait, which generates accounts and offline
///   requests that to be submitted during both setup stage and testing stage.
/// * Benchmarker: responsible for playing given requests (submit requests and wait TXNs
///   committed).
///
/// then it drives requests submission/waiting process in the following flow:
///
///     // Generate accounts
///     faucet_account = bm.load_faucet_account();
///     accounts = generator.gen_accounts(num_accounts);
///     bm.register_accounts(accounts);
///
///     // Generate and run setup requests
///     setup_requests = generator.gen_setup_requests(faucet_account, &mut accounts);
///     bm.submit_requests_and_wait_txns_committed(faucet_account, setup_requests);
///
///     // Generate and run requests
///     requests = generator.gen_requests(accounts);
///     bm.submit_requests_and_wait_txns_committed(accounts, requests);
///
/// By conforming to the LoadGenerator APIs,
/// this flow is basically the same for different LoadGenerators/experiments.
use benchmark::{
    bin_utils::{create_benchmarker_from_opt, measure_throughput, try_start_metrics_server},
    cli_opt::{RubenOpt, TransactionPattern},
    load_generator::{LoadGenerator, PairwiseTransferTxnGenerator, RingTransferTxnGenerator},
};
use logger::{self, prelude::*};
use std::ops::DerefMut;

fn main() {
    let _g = logger::set_default_global_logger(false, Some(256));
    info!("RuBen: the utility to (Ru)n (Ben)chmarker");
    let args = RubenOpt::new_from_args();
    info!("Parsed arguments: {:#?}", args);
    try_start_metrics_server(&args.bench_opt);
    let mut bm = create_benchmarker_from_opt(&args.bench_opt);
    let mut faucet_account = bm.load_faucet_account(&args.bench_opt.faucet_key_file_path);
    let mut generator: Box<dyn LoadGenerator> = match args.txn_pattern {
        TransactionPattern::Ring => Box::new(RingTransferTxnGenerator::new()),
        TransactionPattern::Pairwise => Box::new(PairwiseTransferTxnGenerator::new()),
    };
    measure_throughput(
        &mut bm,
        generator.deref_mut(),
        &mut faucet_account,
        args.num_accounts,
        args.num_rounds,
        args.num_epochs,
    );
}

#[cfg(test)]
mod tests {
    use crate::{create_benchmarker_from_opt, measure_throughput};
    use benchmark::{
        cli_opt::BenchOpt,
        load_generator::{
            gen_get_txn_by_sequnece_number_request, LoadGenerator, Request,
            RingTransferTxnGenerator,
        },
        OP_COUNTER,
    };
    use client::AccountData;
    use config_builder::swarm_config::LibraSwarmTopology;
    use libra_swarm::swarm::LibraSwarm;
    use rusty_fork::{rusty_fork_id, rusty_fork_test, rusty_fork_test_name};
    use std::ops::Range;
    use tempdir::TempDir;

    /// Start libra_swarm and create a BenchOpt struct for testing.
    /// Must return the TempDir otherwise it will be freed somehow.
    fn start_swarm_and_setup_arguments() -> (LibraSwarm, BenchOpt, Option<TempDir>) {
        let (faucet_account_keypair, faucet_key_file_path, temp_dir) =
            generate_keypair::load_faucet_key_or_create_default(None);
        let topology = LibraSwarmTopology::create_validator_network(4);
        let swarm = LibraSwarm::launch_swarm(
            topology, /* num_nodes */
            true,     /* disable_logging */
            faucet_account_keypair,
            None, /* config_dir */
            None, /* template_path */
        );
        let mut args = BenchOpt {
            validator_addresses: Vec::new(),
            swarm_config_dir: Some(String::from(
                swarm.dir.as_ref().unwrap().as_ref().to_str().unwrap(),
            )),
            // Don't start metrics server as we are not testing with prometheus.
            metrics_server_address: None,
            faucet_key_file_path,
            num_clients: 4,
            stagger_range_ms: 1,
            submit_rate: Some(50),
        };
        args.try_parse_validator_addresses();
        (swarm, args, temp_dir)
    }

    rusty_fork_test! {
        #[test]
        fn test_benchmarker_counters() {
            let (_swarm, args, _temp_dir) = start_swarm_and_setup_arguments();
            let mut bm = create_benchmarker_from_opt(&args);
            let mut faucet_account = bm.load_faucet_account(&args.faucet_key_file_path);
            let mut ring_generator = RingTransferTxnGenerator::new();
            let (num_accounts, num_rounds, num_epochs) = (4, 4, 2);
            measure_throughput(
                &mut bm,
                &mut ring_generator,
                &mut faucet_account,
                num_accounts,
                num_rounds,
                num_epochs,
            );
            let created_txns = OP_COUNTER.counter("create_txn_request.success").get();
            let failed_to_create = OP_COUNTER.counter("create_txn_request.failure").get();
            assert!(created_txns + failed_to_create == (4 * 4 * 2 + 4));
            let accepted_txns = OP_COUNTER.counter("submit_txns.success").get();
            assert!(accepted_txns <= created_txns);
            let committed_txns = OP_COUNTER.counter("committed_txns").get();
            let timedout_txns = OP_COUNTER.counter("timedout_txns").get();
            // Why `<=`: timedout TXNs in previous epochs can be committed in the next epoch.
            assert!(accepted_txns <= committed_txns + timedout_txns);
        }
    }

    /// Generate read requests for each account using a range of sequence numbers.
    fn gen_test_read_requests(
        accounts: &[AccountData],
        sequence_number_range: Range<u64>,
    ) -> Vec<Request> {
        let mut results = vec![];
        for sequence_number in sequence_number_range {
            let read_requests: Vec<_> = accounts
                .iter()
                .map(|account| {
                    gen_get_txn_by_sequnece_number_request(account.address, sequence_number)
                })
                .collect();
            results.extend(read_requests.into_iter());
        }
        results
    }

    #[test]
    fn test_benchmarker_read_requests() {
        let (_swarm, args, _temp_dir) = start_swarm_and_setup_arguments();
        let mut bm = create_benchmarker_from_opt(&args);
        let mut ring_generator = RingTransferTxnGenerator::new();
        let accounts: Vec<AccountData> = ring_generator.gen_accounts(16 /* num_accounts */);
        let read_requests = gen_test_read_requests(&accounts, Range { start: 0, end: 10 });
        bm.submit_requests(&read_requests, args.submit_rate.unwrap());
    }
}
