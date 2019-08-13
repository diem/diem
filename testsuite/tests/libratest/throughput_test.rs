// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use benchmark::{
    bin_utils::{create_ac_clients, measure_throughput},
    load_generator::PairwiseTransferTxnGenerator,
    ruben_opt::parse_swarm_config_from_dir,
    Benchmarker,
};
use config_builder::swarm_config::LibraSwarmTopology;
use libra_swarm::swarm::LibraSwarm;
use num::traits::Float;
use rusty_fork::{rusty_fork_id, rusty_fork_test, rusty_fork_test_name};
use statistical::{mean, population_standard_deviation};
use std::fmt::Display;

fn calculate_avg_std<T: Float + Display>(sequence: &[T], name: &str) {
    // Output json-formatted result.
    println!(
        "{{\"{}_mean\": {:.2}, \"{}_stdev\": {:.2}}}",
        name,
        mean(sequence),
        name,
        population_standard_deviation(sequence, None),
    );
}

rusty_fork_test! {
    #[test]
    fn test_throughput() {
        // Choose parameters that is able to run on not-too-powerful local machines,
        // but still generates enough TXNs for testing (2K TXNs per epoch).
        // At submit rate 50 TXNs per second per client, submission takes around 10 seconds.
        let (num_nodes, num_accounts, num_clients) = (4, 32, 4);
        let (num_rounds, num_epochs, stagger_ms) = (2, 4, 1);
        let submit_rate = 50;
        let topology = LibraSwarmTopology::create_validator_network(num_nodes);

        let (faucet_account_keypair, faucet_key_file_path, _temp_dir) =
            generate_keypair::load_faucet_key_or_create_default(None);
        let swarm = LibraSwarm::launch_swarm(
            topology,
            true,   /* disable_logging */
            faucet_account_keypair,
            false,  /* tee_logs */
            None,   /* config_dir */
            None,   /* template_path */
        );
        let swarm_config_dir = String::from(swarm.dir.as_ref().unwrap().as_ref().to_str().unwrap());
        let validator_addresses = parse_swarm_config_from_dir(&swarm_config_dir).unwrap();
        let clients = create_ac_clients(num_clients, &validator_addresses);
        let mut bm = Benchmarker::new(clients, stagger_ms, submit_rate);
        let mut faucet_account = bm.load_faucet_account(&faucet_key_file_path);
        let mut pairwise_generator = PairwiseTransferTxnGenerator::new();
        let results = measure_throughput(&mut bm, &mut pairwise_generator, &mut faucet_account, num_accounts, num_rounds, num_epochs);
        let req_throughput_seq: Vec<_> = results.iter().map(|x| x.req_throughput()).collect();
        let txn_throughput_seq: Vec<_> = results.iter().map(|x| x.txn_throughput()).collect();
        calculate_avg_std(&req_throughput_seq, &"REQ_throughput");
        calculate_avg_std(&txn_throughput_seq, &"TXN_throughput");
    }
}
