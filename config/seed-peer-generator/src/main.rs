// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use diem_config::config::{PersistableConfig, RoleType};
use std::path::PathBuf;
use structopt::StructOpt;

#[allow(clippy::large_enum_variant)]
#[derive(Debug, StructOpt)]
#[structopt(about = "Tool to generate configs from chain")]
struct Args {
    #[structopt(short = "o", long, parse(from_os_str))]
    /// The output directory
    output_dir: PathBuf,
    #[structopt(short = "e", long)]
    /// JSON RPC endpoint
    endpoint: String,
    #[structopt(short = "r", long)]
    role: RoleType,
}

fn main() {
    let args = Args::from_args();

    let seed_peers_config = match args.role {
        RoleType::FullNode => {
            seed_peer_generator::utils::gen_validator_full_node_seed_peer_config(args.endpoint)
        }
        _ => panic!("{} not yet supported", args.role),
    }
    .expect("Should have generated a trusted peer set");

    // Save to a file for loading later
    seed_peers_config
        .save_config(args.output_dir.join("seed_peers.yaml"))
        .expect("Unable to save seed peers config");
}
