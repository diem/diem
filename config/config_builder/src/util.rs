// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use config::{
    config::{NodeConfig, NodeConfigHelpers},
    trusted_peers::{TrustedPeersConfig, TrustedPeersConfigHelpers},
};
use failure::prelude::*;
use nextgen_crypto::{ed25519::*, test_utils::KeyPair};
use proto_conv::IntoProtoBytes;
use rand::{Rng, SeedableRng};
use std::{convert::TryFrom, fs::File, io::prelude::*, path::Path};
use types::{account_address::AccountAddress, validator_public_keys::ValidatorPublicKeys};
use vm_genesis::encode_genesis_transaction_with_validator;

pub fn gen_genesis_transaction<P: AsRef<Path>>(
    path: P,
    faucet_account_keypair: &KeyPair<Ed25519PrivateKey, Ed25519PublicKey>,
    trusted_peer_config: &TrustedPeersConfig,
) -> Result<()> {
    let validator_set = trusted_peer_config
        .peers
        .iter()
        .map(|(peer_id, peer)| {
            ValidatorPublicKeys::new(
                AccountAddress::try_from(peer_id.clone()).expect("[config] invalid peer_id"),
                peer.get_consensus_public().clone(),
                peer.get_network_signing_public().clone(),
                peer.get_network_identity_public(),
            )
        })
        .collect();
    let transaction = encode_genesis_transaction_with_validator(
        &faucet_account_keypair.private_key,
        faucet_account_keypair.public_key.clone(),
        validator_set,
    );
    let mut file = File::create(path)?;
    file.write_all(&transaction.into_proto_bytes()?)?;
    Ok(())
}

/// Returns the config as well as the genesis keyapir
pub fn get_test_config() -> (NodeConfig, KeyPair<Ed25519PrivateKey, Ed25519PublicKey>) {
    // TODO: test config should be moved here instead of config crate
    let config = NodeConfigHelpers::get_single_node_test_config(true);
    // Those configs should be different on every call. We bypass the
    // costly StdRng initialization
    let mut seed_rng = rand::rngs::OsRng::new().expect("can't access OsRng");
    let seed_buf: [u8; 32] = seed_rng.gen();
    let mut rng = rand::rngs::StdRng::from_seed(seed_buf);
    let (private_key, _) = compat::generate_keypair(&mut rng);
    let keypair = KeyPair::from(private_key);

    gen_genesis_transaction(
        &config.execution.genesis_file_location,
        &keypair,
        &TrustedPeersConfigHelpers::get_test_config(1, None).1,
    )
    .expect("[config] failed to create genesis transaction");
    (config, keypair)
}
