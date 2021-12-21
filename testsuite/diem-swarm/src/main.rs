// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use diem_swarm::faucet;
use diem_types::chain_id::ChainId;
use forge::{LocalSwarm, LocalVersion, Swarm, Version};
use std::{collections::HashMap, fs::File, io::Write, num::NonZeroUsize, path::Path, sync::Arc};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "Diem swarm to start local nodes")]
struct Args {
    /// Number of nodes to start (1 by default)
    #[structopt(short = "n", long, default_value = "1")]
    pub num_nodes: NonZeroUsize,
    /// Start client
    #[structopt(short = "s", long, requires("cli-path"))]
    pub start_client: bool,
    /// Directory used by launch_swarm to output DiemNodes' config files, logs, diemdb, etc,
    /// such that user can still inspect them after exit.
    /// If unspecified, a temporary dir will be used and auto deleted.
    #[structopt(short = "c", long)]
    pub config_dir: Option<String>,
    /// Start with faucet service for minting coins, this flag disables cli's dev commands.
    /// Used for manual testing faucet service integration.
    #[structopt(short = "m", long, requires("faucet-path"))]
    pub start_faucet: bool,
    /// Path to the diem-node binary
    #[structopt(long)]
    pub diem_node: String,

    /// Path to the faucet binary
    #[structopt(long)]
    pub faucet_path: Option<String>,
}

fn main() {
    let args = Args::from_args();

    diem_logger::Logger::new().init();

    let mut versions = HashMap::new();
    let version = LocalVersion::new(
        "unknown".into(),
        args.diem_node.into(),
        Version::new(0, "unknown".into()),
    );
    versions.insert(version.version(), version);

    let mut builder = LocalSwarm::builder(Arc::new(versions)).number_of_validators(args.num_nodes);

    if let Some(dir) = &args.config_dir {
        builder = builder.dir(dir);
    }

    let mut swarm = builder
        .build(rand::rngs::OsRng)
        .expect("Failed to launch validator swarm");

    let diem_root_key_path = swarm.dir().join("mint.key");
    let serialized_key = bcs::to_bytes(swarm.chain_info().root_account.private_key()).unwrap();
    let mut key_file = File::create(&diem_root_key_path).unwrap();
    key_file.write_all(&serialized_key).unwrap();

    let validator_config = swarm.validators().next().unwrap().config();
    let waypoint = validator_config.base.waypoint.waypoint();

    println!(
        "json-rpc: {}",
        format!(
            "http://localhost:{}",
            validator_config.json_rpc.address.port()
        ),
    );
    println!("root key path: {:?}", diem_root_key_path);
    println!("waypoint: {}", waypoint);
    println!("chain_id: {}", ChainId::test().id());

    let ports = swarm
        .validators()
        .map(|v| {
            let validator_config = v.config();
            let port = validator_config.json_rpc.address.port();
            let debug_interface_port = validator_config
                .debug_interface
                .admission_control_node_debug_port;
            (port, debug_interface_port)
        })
        .collect::<Vec<_>>();

    let node_address_list = ports
        .iter()
        .map(|port| format!("localhost:{}", port.0))
        .collect::<Vec<String>>()
        .join(",");

    println!("To run transaction generator run:");
    println!(
        "\tcluster-test --mint-file {:?} --swarm --peers {:?} --emit-tx --workers-per-ac 1",
        diem_root_key_path, node_address_list,
    );

    let node_address_list = ports
        .iter()
        .map(|port| format!("localhost:{}:{}", port.0, port.1))
        .collect::<Vec<String>>()
        .join(",");

    println!("To run health check:");
    println!(
        "\tcluster-test --mint-file {:?} --swarm --peers {:?} --health-check --duration 30",
        diem_root_key_path, node_address_list,
    );

    let _faucet = if args.start_faucet {
        let faucet_port = diem_config::utils::get_available_port();
        let server_port = ports[0].0;
        println!("Starting faucet service at port: {}", faucet_port);
        let process = faucet::Process::start(
            args.faucet_path.as_ref().unwrap().as_ref(),
            faucet_port,
            server_port,
            Path::new(&diem_root_key_path),
        );
        println!("Waiting for faucet connectivity");
        process
            .wait_for_connectivity()
            .expect("Failed to start Faucet");
        Some(process)
    } else {
        None
    };

    // Explicitly capture CTRL-C to drop DiemSwarm.
    let (tx, rx) = std::sync::mpsc::channel();
    ctrlc::set_handler(move || {
        tx.send(())
            .expect("failed to send unit when handling CTRL-C");
    })
    .expect("failed to set CTRL-C handler");
    println!("CTRL-C to exit.");
    rx.recv()
        .expect("failed to receive unit when handling CTRL-C");

    if let Some(dir) = &args.config_dir {
        println!("Please manually cleanup {:?} after inspection", dir);
    }

    println!("Exit diem-swarm.");
}
