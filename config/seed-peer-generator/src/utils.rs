// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use anyhow::Error;
use diem_config::config::{TrustedPeer, TrustedPeerSet};
use diem_logger::prelude::*;
use diem_network_address::NetworkAddress;
use diem_secure_json_rpc::JsonRpcClient;
use diem_types::{
    account_config::diem_root_address, on_chain_config::ValidatorSet,
    validator_info::ValidatorInfo, PeerId,
};

/// Retrieve the Fullnode seed peers from JSON-RPC
pub fn gen_full_node_seed_peer_config(client_endpoint: String) -> anyhow::Result<TrustedPeerSet> {
    let validator_set = get_validator_set(client_endpoint)?;

    gen_trusted_peers(&validator_set, to_fullnode_addresses)
}

pub(crate) fn to_fullnode_addresses(
    validator_info: &ValidatorInfo,
) -> Result<Vec<NetworkAddress>, bcs::Error> {
    validator_info.config().fullnode_network_addresses()
}

/// Retrieve the validator set from a JSON RPC endpoint
fn get_validator_set(client_endpoint: String) -> anyhow::Result<ValidatorSet> {
    let root_account_address = diem_root_address();
    let json_rpc = JsonRpcClient::new(client_endpoint);
    let account_state = json_rpc.get_account_state(root_account_address, None)?;
    if let Some(val) = account_state.get_validator_set()? {
        Ok(val)
    } else {
        Err(Error::msg("No validator set"))
    }
}

// TODO: Merge with OnchainDiscovery
pub(crate) fn gen_trusted_peers<
    ToAddresses: Fn(&ValidatorInfo) -> Result<Vec<NetworkAddress>, bcs::Error>,
>(
    validator_set: &ValidatorSet,
    to_addresses: ToAddresses,
) -> anyhow::Result<TrustedPeerSet> {
    let set = validator_set
        .payload()
        .iter()
        .filter_map(|validator_info| {
            to_seed_peer(validator_info, &to_addresses).map_or_else(
                |error| {
                    warn!(
                        "Unable to generate seed for validator {} {}",
                        validator_info.account_address(),
                        error
                    );
                    None
                },
                Some,
            )
        })
        .collect::<TrustedPeerSet>();

    if set.is_empty() {
        Err(Error::msg("No seed peers were generated"))
    } else {
        Ok(set)
    }
}

/// Convert ValidatorInfo to a seed peer
fn to_seed_peer<T: Fn(&ValidatorInfo) -> Result<Vec<NetworkAddress>, bcs::Error>>(
    validator_info: &ValidatorInfo,
    to_addresses: &T,
) -> Result<(PeerId, TrustedPeer), bcs::Error> {
    let peer_id = *validator_info.account_address();
    let addrs = to_addresses(validator_info)?;
    Ok((peer_id, TrustedPeer::from(addrs)))
}

#[cfg(test)]
mod tests {
    use crate::utils::{gen_trusted_peers, to_fullnode_addresses};
    use diem_config::config::{TrustedPeer, TrustedPeerSet, HANDSHAKE_VERSION};
    use diem_crypto::{ed25519::Ed25519PrivateKey, x25519, PrivateKey as PK, Uniform};
    use diem_network_address::NetworkAddress;
    use diem_types::{
        on_chain_config::ValidatorSet, validator_config::ValidatorConfig,
        validator_info::ValidatorInfo, PeerId,
    };
    use rand::{prelude::StdRng, SeedableRng};

    fn validator_set(
        peer_id: PeerId,
        fullnode_network_addresses: &[NetworkAddress],
    ) -> ValidatorSet {
        let consensus_private_key = Ed25519PrivateKey::generate_for_testing();
        let consensus_pubkey = consensus_private_key.public_key();
        let validator_config = ValidatorConfig::new(
            consensus_pubkey,
            vec![],
            bcs::to_bytes(fullnode_network_addresses).unwrap(),
        );
        let validator_info = ValidatorInfo::new(peer_id, 0, validator_config);
        ValidatorSet::new(vec![validator_info])
    }

    fn generate_network_addresses(num: u64) -> Vec<NetworkAddress> {
        let mut addresses = Vec::new();
        let mut rng: StdRng = SeedableRng::seed_from_u64(0);
        for _ in 0..num {
            let private_key = x25519::PrivateKey::generate(&mut rng);
            let pubkey = private_key.public_key();
            let network_address =
                NetworkAddress::mock().append_prod_protos(pubkey, HANDSHAKE_VERSION);
            addresses.push(network_address);
        }

        addresses
    }

    #[test]
    fn fullnode_test() {
        let peer_id = PeerId::random();
        let fullnode_addresses = generate_network_addresses(3);
        let validator_set = validator_set(peer_id, &fullnode_addresses);
        let result = gen_trusted_peers(&validator_set, to_fullnode_addresses).unwrap();

        let mut expected_trusted_peers = TrustedPeerSet::new();
        expected_trusted_peers.insert(peer_id, TrustedPeer::from(fullnode_addresses));
        assert_eq!(expected_trusted_peers, result);
    }
}
