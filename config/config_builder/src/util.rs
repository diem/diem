// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use config::{
    config::{NodeConfig, NodeConfigHelpers},
    trusted_peers::{TrustedPeersConfig, TrustedPeersConfigHelpers},
};
use crypto::signing::{self, KeyPair};
use failure::prelude::*;
use proto_conv::IntoProtoBytes;
use std::{convert::TryFrom, fs::File, io::prelude::*, path::Path};
use types::{account_address::AccountAddress, validator_public_keys::ValidatorPublicKeys};
use vm_genesis::encode_genesis_transaction_with_validator;

pub fn gen_genesis_transaction<P: AsRef<Path>>(
    path: P,
    faucet_account_keypair: &KeyPair,
    trusted_peer_config: &TrustedPeersConfig,
) -> Result<()> {
    let validator_set = trusted_peer_config
        .peers
        .iter()
        .map(|(peer_id, peer)| {
            ValidatorPublicKeys::new(
                AccountAddress::try_from(peer_id.clone()).expect("[config] invalid peer_id"),
                peer.get_consensus_public().into(),
                peer.get_network_signing_public().into(),
                peer.get_network_identity_public(),
            )
        })
        .collect();
    let transaction = encode_genesis_transaction_with_validator(
        faucet_account_keypair.private_key(),
        faucet_account_keypair.public_key(),
        validator_set,
    );
    let mut file = File::create(path)?;
    file.write_all(&transaction.into_proto_bytes()?)?;
    Ok(())
}

/// Returns the config as well as the genesis keyapir
pub fn get_test_config() -> (NodeConfig, KeyPair) {
    // TODO: test config should be moved here instead of config crate
    let config = NodeConfigHelpers::get_single_node_test_config(true);
    let (private_key, _) = signing::generate_keypair();
    let keypair = KeyPair::new(private_key);

    gen_genesis_transaction(
        &config.execution.genesis_file_location,
        &keypair,
        &TrustedPeersConfigHelpers::get_test_config(1, None).1,
    )
    .expect("[config] failed to create genesis transaction");
    (config, keypair)
}
