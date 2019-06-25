// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_swarm::{client, swarm::LibraSwarm};
use std::path::Path;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "libra_swarm",
    author = "Libra",
    about = "Libra swarm to start local nodes"
)]
struct Args {
    /// Number of nodes to start (1 by default)
    #[structopt(short = "n", long = "num_nodes")]
    pub num_nodes: Option<usize>,
    /// Disable logging (for performance testing)"
    #[structopt(short = "d", long = "disable_logging")]
    pub disable_logging: bool,
    /// Start client
    #[structopt(short = "s", long = "start_client")]
    pub start_client: bool,
}

fn main() {
    let args = Args::from_args();
    let num_nodes = args.num_nodes.unwrap_or(1);

    let (faucet_account_keypair, faucet_key_file_path, _temp_dir) =
        generate_keypair::load_faucet_key_or_create_default(None);

    println!("Faucet account created in file {:?}", faucet_key_file_path);

    let swarm = LibraSwarm::launch_swarm(
        num_nodes,
        args.disable_logging,
        faucet_account_keypair,
        false, /* tee_logs */
    );

    let tmp_mnemonic_file = tempfile::NamedTempFile::new().unwrap();;
    if args.start_client {
        let client = client::InteractiveClient::new_with_inherit_io(
            *swarm.get_validators_public_ports().get(0).unwrap(),
            Path::new(&faucet_key_file_path),
            &tmp_mnemonic_file.into_temp_path(),
            swarm.get_trusted_peers_config_path(),
        );
        println!("Loading client...");
        let _output = client.output().expect("Failed to wait on child");
        println!("Exit client.");
    } else {
        println!("CTRL-C to exit.");
        loop {
            std::thread::park();
        }
    }
}
