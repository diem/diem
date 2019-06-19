use admission_control_proto::proto::admission_control_grpc::AdmissionControlClient;
use benchmark::Benchmarker;
use client::{client_proxy::ClientProxy, AccountData};
use debug_interface::NodeDebugClient;
use grpcio::{ChannelBuilder, EnvBuilder};
use libra_swarm::swarm::LibraSwarm;
use logger::{self, prelude::*};
use std::sync::Arc;

/// Start a 4-node Libra, adding 16 accounts, submit TXNs that transfer coins in a circular manner.
fn main() {
    let _g = logger::set_default_global_logger(false, Some(256));
    // TODO(yjq): use arguments to let user set num_validators and num_accounts.
    let (mint_account_keypair, mint_key_file_path, _temp_dir) =
        generate_keypair::load_faucet_key_or_create_default(None);
    let swarm = LibraSwarm::launch_swarm(4, true, mint_account_keypair.clone(), true);
    let validator_port = *swarm.get_validators_public_ports().get(0).unwrap();
    // Create ClientProxy instance.
    let path = swarm.get_trusted_peers_config_path();
    let client_proxy = ClientProxy::new(
        "localhost",
        validator_port.to_string().as_str(),
        &path,
        &mint_key_file_path,
        None,
        None,
    )
    .unwrap();
    // Create a AdmissionControlClient instance.
    let conn_addr = format!("localhost:{}", validator_port);
    let env = Arc::new(EnvBuilder::new().name_prefix("ac-grpc-").build());
    let ch = ChannelBuilder::new(env).connect(&conn_addr);
    let client = AdmissionControlClient::new(ch);
    // Create NodeDebugClient instance.
    let debug_port = *swarm.get_validators_debug_ports().get(0).unwrap();
    let debug_client = NodeDebugClient::new("localhost", debug_port);
    // Ready to instantiate Benchmarker with 16 accounts.
    let mut bm = Benchmarker::new(client, client_proxy, debug_client);
    let mut accounts: Vec<AccountData> = bm.gen_and_mint_accounts(16).unwrap();
    let txn_reqs = bm.gen_txn_reqs(&mut accounts).unwrap();
    info!(
        "Generated {} txn reqs from {} accounts.",
        txn_reqs.len(),
        accounts.len()
    );
    bm.submit_and_wait_txn_reqs(&txn_reqs, false).unwrap();
}
