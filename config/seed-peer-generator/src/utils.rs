// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use diem_logger::prelude::*;
use diem_network_address::NetworkAddress;
use diem_secure_json_rpc::JsonRpcClient;
use diem_types::{
    account_config::diem_root_address, on_chain_config::ValidatorSet,
    validator_info::ValidatorInfo, PeerId,
};
use std::collections::HashMap;

// TODO: Use the definition from network?
pub type SeedPeersConfig = HashMap<PeerId, Vec<NetworkAddress>>;

pub fn gen_seed_peer_config(client_endpoint: String) -> SeedPeersConfig {
    let validator_set = get_validator_set(client_endpoint, diem_root_address())
        .unwrap()
        .expect("No validator set found.");

    // Collect the validators
    validator_set
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
        .collect::<SeedPeersConfig>()
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
) -> Result<(PeerId, Vec<NetworkAddress>), bcs::Error> {
    let peer_id = *validator_info.account_address();
    let addrs = validator_info.config().fullnode_network_addresses()?;
    Ok((peer_id, addrs))
}
