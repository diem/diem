use admission_control_proto::proto::admission_control_grpc::AdmissionControlClient;
use benchmark::{
    ruben_opt::{Executable, Opt},
    Benchmarker,
};
use client::AccountData;
use debug_interface::NodeDebugClient;
use grpcio::{ChannelBuilder, EnvBuilder};
use logger::{self, prelude::*};
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
fn measure_throughput(
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
pub fn create_ac_clients(
    num_clients: usize,
    validator_addresses: Vec<String>,
) -> Vec<AdmissionControlClient> {
    let mut clients: Vec<AdmissionControlClient> = vec![];
    for i in 0..num_clients {
        let index = i % validator_addresses.len();
        let client = create_ac_client(&validator_addresses[index]);
        clients.push(client);
    }
    clients
}

fn main() {
    let _g = logger::set_default_global_logger(false, Some(256));
    info!("RuBen: the utility to (Ru)n (Ben)chmarker");
    let args = Opt::new_from_args();
    info!("Parsed arguments: {:#?}", args);

    // Create AdmissionControlClient instances.
    let clients = create_ac_clients(args.num_clients, args.validator_addresses);

    // Create NodeDebugClient instance.
    let debug_client = NodeDebugClient::from_socket_addr_str(
        args.debug_address.expect("failed to parse debug_address"),
    );

    // Ready to instantiate Benchmarker.
    let mut bm = Benchmarker::new(clients, debug_client);
    let mut accounts: Vec<AccountData> = bm
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
