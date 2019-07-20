use admission_control_proto::proto::admission_control_grpc::AdmissionControlClient;
use benchmark::{
    ruben_opt::{Executable, Opt},
    Benchmarker,
};
use client::AccountData;
use debug_interface::NodeDebugClient;
use grpcio::{ChannelBuilder, EnvBuilder};
use logger::{self, prelude::*};
use metrics::metric_server::start_server;
use std::sync::Arc;

/// Simply submit some TXNs to test the liveness of the network. Here we use ring TXN pattern
/// to generate request, which scales linear with the number of accounts.
/// In one epoch, we play the sequence of ring TXN requests repeatedly for num_rounds times.
fn test_liveness(
    bm: &mut Benchmarker,
    accounts: &mut [AccountData],
    num_rounds: u64,
    num_epochs: u64,
) {
    for _ in 0..num_epochs {
        let mut repeated_tx_reqs = vec![];
        for _ in 0..num_rounds {
            let tx_reqs = bm.gen_ring_txn_requests(accounts);
            repeated_tx_reqs.extend(tx_reqs.into_iter());
        }
        bm.submit_and_wait_txn_requests(&repeated_tx_reqs);
    }
}

/// Measure both `burst` and `epoch` throughput with pairwise TXN pattern.
/// * `burst throughput`: the average committed txns per second during one run of playing TXNs
///   (e.g., Benchmarker::submit_and_wait_txn_requests). Since time is counted from submission until
///   all TXNs are committed, this measurement is in a sense the user-side throughput. In one epoch,
///   we play the pairwise TXN request sequence repeatedly for num_rounds times.
/// * `epoch throughput`: Since single run of playing TXNs may have high variance, we can repeat
///   playing TXNs many times and calculate the averaged `burst throughput` along with standard
///   deviation (will be added shortly).
pub(crate) fn measure_throughput(
    bm: &mut Benchmarker,
    accounts: &mut [AccountData],
    num_rounds: u64,
    num_epochs: u64,
) {
    let mut txn_throughput_seq = vec![];
    for _ in 0..num_epochs {
        let mut repeated_tx_reqs = vec![];
        for _ in 0..num_rounds {
            let tx_reqs = bm.gen_pairwise_txn_requests(accounts);
            repeated_tx_reqs.extend(tx_reqs.into_iter());
        }
        let txn_throughput = bm.measure_txn_throughput(&repeated_tx_reqs);
        txn_throughput_seq.push(txn_throughput);
    }
    println!(
        "{:?} epoch(s) of TXN throughput = {:?}",
        num_epochs, txn_throughput_seq
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
    // Create NodeDebugClient instance.
    let debug_client = NodeDebugClient::from_socket_addr_str(
        args.debug_address
            .as_ref()
            .expect("failed to parse debug_address"),
    );
    // Ready to instantiate Benchmarker.
    Benchmarker::new(clients, debug_client)
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
    let mut accounts = bm
        .gen_and_mint_accounts(&args.faucet_key_file_path, args.num_accounts)
        .expect("failed to generate and mint all accounts");
    match args.executable {
        Executable::TestLiveness => {
            test_liveness(&mut bm, &mut accounts, args.num_rounds, args.num_epochs);
        }
        Executable::MeasureThroughput => {
            measure_throughput(&mut bm, &mut accounts, args.num_rounds, args.num_epochs);
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::{create_benchmarker_from_opt, measure_throughput};
    use benchmark::{
        ruben_opt::{Executable, Opt},
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
                num_accounts: 32,
                free_lunch: 10_000_000,
                num_clients: 4,
                num_rounds: 8,
                num_epochs: 1,
                executable: Executable::MeasureThroughput,
            };
            args.try_parse_validator_addresses();
            let mut bm = create_benchmarker_from_opt(&args);
            let mut accounts = bm
                .gen_and_mint_accounts(&args.faucet_key_file_path, args.num_accounts)
                .expect("failed to generate and mint all accounts");
            measure_throughput(&mut bm, &mut accounts, args.num_rounds, args.num_epochs);
            let created_txns = OP_COUNTER.counter("created_txns").get();
            let requested_txns = OP_COUNTER.counter("requested_txns").get();
            let failed_submissions = OP_COUNTER.counter("failed_submissions").get();
            let accepted_txns = OP_COUNTER.counter("accepted_txns").get();
            let failed_responses = OP_COUNTER.counter("failed_responses").get();
            assert_eq!(created_txns, requested_txns + failed_submissions);
            assert_eq!(requested_txns, accepted_txns + failed_responses);
        }
    }
}
