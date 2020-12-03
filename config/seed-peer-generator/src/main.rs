// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use diem_config::config::PersistableConfig;
use diem_network_address::NetworkAddress;
use diem_types::PeerId;
use std::{collections::HashMap, path::PathBuf};
use structopt::StructOpt;

// TODO: Use the definition from network?
pub type SeedPeersConfig = HashMap<PeerId, Vec<NetworkAddress>>;

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
}

fn main() {
    let args = Args::from_args();

    let seed_peers_config = seed_peer_generator::utils::gen_seed_peer_config(args.endpoint);

    // Save to a file for loading later
    seed_peers_config
        .save_config(args.output_dir.join("seed_peers.yaml"))
        .expect("Unable to save seed peers config");
}
