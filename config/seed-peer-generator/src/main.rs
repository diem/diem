// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use libra_config::config::PersistableConfig;
use libra_logger::prelude::*;
use libra_network_address::NetworkAddress;
use libra_secure_json_rpc::JsonRpcClient;
use libra_types::{
    account_config::libra_root_address, on_chain_config::ValidatorSet,
    validator_info::ValidatorInfo, PeerId,
};
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
    let validator_set = get_validator_set(args.endpoint, libra_root_address())
        .unwrap()
        .expect("No validator set found.");

    // Collect the validators
    let seed_peers_config = validator_set
        .payload()
        .iter()
        .filter_map(|v| match to_seed_peer(v) {
            Ok(result) => Some(result),
            Err(error) => {
                warn!(
                    "Unable to read peer id for validator {} {}",
                    v.account_address(),
                    error
                );
                None
            }
        })
        .collect::<SeedPeersConfig>();

    // Save to a file for loading later
    seed_peers_config
        .save_config(args.output_dir.join("seed_peers.yaml"))
        .expect("Unable to save seed peers config");
}

/// Retrieve validator set from the JSON-RPC endpoint
fn get_validator_set(endpoint: String, peer_id: PeerId) -> anyhow::Result<Option<ValidatorSet>> {
    let json_rpc = JsonRpcClient::new(endpoint);
    let account_state = json_rpc.get_account_state(peer_id, None)?;
    Ok(account_state.get_validator_set()?)
}

/// Convert ValidatorInfo to a seed peer
fn to_seed_peer(
    validator_info: &ValidatorInfo,
) -> Result<(PeerId, Vec<NetworkAddress>), lcs::Error> {
    let peer_id = *validator_info.account_address();
    let cb = |err| {
        println!(
            "Failed to parse fullnode network address: peer: {}, err: {}",
            peer_id, err
        )
    };
    let addrs = validator_info
        .config()
        .full_node_network_addresses(Some(Box::new(cb)))?;
    Ok((peer_id, addrs))
}
