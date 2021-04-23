// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use move_oncall_trainer::{experiments, run_experiment, NodeInfo};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// Path to the output serialized bytes
    #[structopt(long, short)]
    experiment: String,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();

    let node = NodeInfo::new_local();
    println!("A Diem Validator node has been spawned! Here's the info you need:");
    println!("{}", node);

    println!("Here's the command to start a few tools that might be handy for you:");
    println!(
        "cargo run --bin diem-transaction-replay -- --url {}",
        node.json_rpc
    );
    println!("cargo run --bin cli -- --url {}", node.json_rpc);
    println!(
        "cd testsuite/cli; cargo run -- --chain-id {} --url {} --waypoint {} -m {:?}",
        node.chain_id,
        node.json_rpc,
        node.waypoint,
        node.root_key_path.as_path()
    );

    let mut client = node.get_client();

    run_experiment(
        &mut client,
        experiments()
            .get(opt.experiment.as_str())
            .expect("Cannot find experiment")
            .clone(),
    )
}
