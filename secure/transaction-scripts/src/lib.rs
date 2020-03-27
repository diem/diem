// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This crates includes the compiled transactions scripts to mitigate issues around binary /
//! transaction script versions and to simplify deployment.

use include_dir::{include_dir, Dir};
use once_cell::sync::Lazy;
use std::path::PathBuf;

const STAGED_EXTENSION: &str = "mv";
const STAGED_TXN_SCRIPTS_DIR: Dir =
    include_dir!("../../language/stdlib/staged/transaction_scripts");

fn script(script_name: &str) -> Vec<u8> {
    let mut path = PathBuf::from(script_name);
    path.set_extension(STAGED_EXTENSION);
    STAGED_TXN_SCRIPTS_DIR
        .get_file(path)
        .unwrap()
        .contents()
        .to_vec()
}

pub static ADD_VALIDATOR_TXN: Lazy<Vec<u8>> = Lazy::new(|| script("add_validator"));

pub static PEER_TO_PEER_TXN: Lazy<Vec<u8>> = Lazy::new(|| script("peer_to_peer"));

pub static PEER_TO_PEER_WITH_METADATA_TXN: Lazy<Vec<u8>> =
    Lazy::new(|| script("peer_to_peer_with_metadata"));

pub static CREATE_ACCOUNT_TXN: Lazy<Vec<u8>> = Lazy::new(|| script("create_account"));

pub static REGISTER_VALIDATOR_TXN: Lazy<Vec<u8>> = Lazy::new(|| script("register_validator"));

pub static REMOVE_VALIDATOR_TXN: Lazy<Vec<u8>> = Lazy::new(|| script("remove_validator"));

pub static ROTATE_CONSENSUS_PUBKEY_TXN: Lazy<Vec<u8>> =
    Lazy::new(|| script("rotate_consensus_pubkey"));

pub static ROTATE_AUTHENTICATION_KEY_TXN: Lazy<Vec<u8>> =
    Lazy::new(|| script("rotate_authentication_key"));

pub static MINT_TXN: Lazy<Vec<u8>> = Lazy::new(|| script("mint"));

pub static EMPTY_TXN: Lazy<Vec<u8>> = Lazy::new(|| script("empty_script"));

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn verify_txn_scripts() {
        let txn_scripts = &[
            &ADD_VALIDATOR_TXN,
            &PEER_TO_PEER_TXN,
            &PEER_TO_PEER_WITH_METADATA_TXN,
            &CREATE_ACCOUNT_TXN,
            &REGISTER_VALIDATOR_TXN,
            &REMOVE_VALIDATOR_TXN,
            &ROTATE_CONSENSUS_PUBKEY_TXN,
            &ROTATE_AUTHENTICATION_KEY_TXN,
            &MINT_TXN,
            &EMPTY_TXN,
        ];

        for txn_script in txn_scripts.iter() {
            assert!(txn_script.len() > 0);
        }
    }
}
