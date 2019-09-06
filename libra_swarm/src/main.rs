// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use config::config::RoleType;
use config_builder::swarm_config::LibraSwarmTopology;
use libra_swarm::{client, swarm::LibraSwarm};
use std::path::Path;
use structopt::StructOpt;
use tools::tempdir::TempPath;

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
    /// Enable logging
    #[structopt(short = "l", long = "enable_logging")]
    pub enable_logging: bool,
    /// Start client
    #[structopt(short = "s", long = "start_client")]
    pub start_client: bool,
    /// Directory used by launch_swarm to output LibraNodes' config files, logs, libradb, etc,
    /// such that user can still inspect them after exit.
    /// If unspecified, a temporary dir will be used and auto deleted.
    #[structopt(short = "c", long = "config_dir")]
    pub config_dir: Option<String>,
    /// If specified, load faucet key from this file. Otherwise generate new keypair file.
    #[structopt(short = "f", long = "faucet_key_path")]
    pub faucet_key_path: Option<String>,
}

fn main() {
    let args = Args::from_args();
    let num_nodes = args.num_nodes.unwrap_or(1);
    // topology indicates structure of the validator network
    // e.g. num of validators, num of full nodes and their children
    let topology = LibraSwarmTopology::create_validator_network(num_nodes);

    let (faucet_account_keypair, faucet_key_file_path, _temp_dir) =
        generate_keypair::load_faucet_key_or_create_default(args.faucet_key_path);

    println!(
        "Faucet account created in (loaded from) file {:?}",
        faucet_key_file_path
    );

    let swarm = LibraSwarm::launch_swarm(
        topology,
        !args.enable_logging,
        faucet_account_keypair,
        args.config_dir.clone(),
        None, /* template_path */
    );

    let config = &swarm.config.get_configs()[0].1;
    let validator_set_file = &config.network.trusted_peers_file;
    println!("To run the Libra CLI client in a separate process and connect to the local cluster of nodes you just spawned, use this command:");
    println!(
        "\tcargo run --bin client -- -a localhost -p {} -s {:?} -m {:?}",
        config.admission_control.admission_control_service_port,
        swarm
            .dir
            .as_ref()
            .expect("fail to access output dir")
            .as_ref()
            .join(validator_set_file),
        faucet_key_file_path,
    );

    let tmp_mnemonic_file = TempPath::new();
    tmp_mnemonic_file.create_as_file().unwrap();
    if args.start_client {
        let client = client::InteractiveClient::new_with_inherit_io(
            swarm.get_ac_port(0, RoleType::Validator),
            Path::new(&faucet_key_file_path),
            &tmp_mnemonic_file.path(),
            swarm.get_trusted_peers_config_path(),
        );
        println!("Loading client...");
        let _output = client.output().expect("Failed to wait on child");
        println!("Exit client.");
    } else {
        // Explicitly capture CTRL-C to drop LibraSwarm.
        let (tx, rx) = std::sync::mpsc::channel();
        ctrlc::set_handler(move || {
            tx.send(())
                .expect("failed to send unit when handling CTRL-C");
        })
        .expect("failed to set CTRL-C handler");
        println!("CTRL-C to exit.");
        rx.recv()
            .expect("failed to receive unit when handling CTRL-C");
    }
    if let Some(dir) = &args.config_dir {
        println!("Please manually cleanup {:?} after inspection", dir);
    }
    println!("Exit libra_swarm.");
}
