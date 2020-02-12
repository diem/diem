// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use libra_config::config::{NodeConfig, RoleType, VMPublishingOption};
use libra_swarm::{client, swarm::LibraSwarm};
use libra_temppath::TempPath;
use std::path::Path;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "Libra swarm to start local nodes")]
struct Args {
    /// Number of nodes to start (1 by default)
    #[structopt(short = "n", long, default_value = "1")]
    pub num_nodes: usize,
    /// Enable logging, by default spawned nodes will not perform logging
    #[structopt(short = "l", long)]
    pub enable_logging: bool,
    /// Start client
    #[structopt(short = "s", long)]
    pub start_client: bool,
    /// Directory used by launch_swarm to output LibraNodes' config files, logs, libradb, etc,
    /// such that user can still inspect them after exit.
    /// If unspecified, a temporary dir will be used and auto deleted.
    #[structopt(short = "c", long)]
    pub config_dir: Option<String>,
    /// If greater than 0, starts a full node swarm connected to the first node in the validator
    /// swarm.
    #[structopt(short = "f", long, default_value = "0")]
    pub num_full_nodes: usize,
}

fn main() {
    let args = Args::from_args();
    let num_nodes = args.num_nodes;
    let num_full_nodes = args.num_full_nodes;
    let mut dev_config = NodeConfig::default();
    dev_config.vm_config.publishing_options = VMPublishingOption::Open;

    libra_logger::init_for_e2e_testing();

    let mut validator_swarm = LibraSwarm::configure_swarm(
        num_nodes,
        RoleType::Validator,
        args.config_dir.clone(),
        Some(dev_config.clone()), /* template config */
        None,                     /* upstream_config_dir */
    )
    .expect("Failed to configure validator swarm");

    let mut full_node_swarm = if num_full_nodes > 0 {
        Some(
            LibraSwarm::configure_swarm(
                num_full_nodes,
                RoleType::FullNode,
                None,             /* config dir */
                Some(dev_config), /* template config */
                Some(String::from(
                    validator_swarm
                        .dir
                        .as_ref()
                        .join("0")
                        .to_str()
                        .expect("Failed to convert std::fs::Path to String"),
                )),
            )
            .expect("Failed to configure full node swarm"),
        )
    } else {
        None
    };
    validator_swarm
        .launch_attempt(RoleType::Validator, !args.enable_logging)
        .expect("Failed to launch validator swarm");
    if let Some(ref mut swarm) = full_node_swarm {
        swarm
            .launch_attempt(RoleType::FullNode, !args.enable_logging)
            .expect("Failed to launch full node swarm");
    }

    let faucet_key_file_path = &validator_swarm.config.faucet_key_path;
    let validator_config = NodeConfig::load(&validator_swarm.config.config_files[0]).unwrap();
    println!("To run the Libra CLI client in a separate process and connect to the validator nodes you just spawned, use this command:");
    println!(
        "\tcargo run --bin cli -- -a localhost -p {} -m {:?}",
        validator_config.admission_control.address.port(),
        faucet_key_file_path,
    );
    let node_address_list = validator_swarm
        .config
        .config_files
        .iter()
        .map(|config| {
            let port = NodeConfig::load(config)
                .unwrap()
                .admission_control
                .address
                .port();
            format!("localhost:{}", port)
        })
        .collect::<Vec<String>>()
        .join(",");
    println!("To run transaction generator run:");
    println!(
        "\tcargo run -p cluster-test -- --mint-file {:?} --swarm --peers {:?}  --emit-tx",
        faucet_key_file_path, node_address_list,
    );
    if let Some(ref swarm) = full_node_swarm {
        let full_node_config = NodeConfig::load(&swarm.config.config_files[0]).unwrap();
        println!("To connect to the full nodes you just spawned, use this command:");
        println!(
            "\tcargo run --bin cli -- -a localhost -p {} -m {:?}",
            full_node_config.admission_control.address.port(),
            faucet_key_file_path,
        );
    }

    let tmp_mnemonic_file = TempPath::new();
    tmp_mnemonic_file.create_as_file().unwrap();
    if args.start_client {
        let client = client::InteractiveClient::new_with_inherit_io(
            validator_swarm.get_ac_port(0),
            Path::new(&faucet_key_file_path),
            &tmp_mnemonic_file.path(),
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
    println!("Exit libra-swarm.");
}
