// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{compile_script, use_staged, MOVE_EXTENSION, STAGED_EXTENSION, TRANSACTION_SCRIPTS};
use include_dir::{include_dir, Dir};
use once_cell::sync::Lazy;
use std::path::PathBuf;

// Encodes the absolute path for the transaction scripts directory.
static TXN_SCRIPTS_DIR: Lazy<PathBuf> = Lazy::new(|| {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(PathBuf::from(TRANSACTION_SCRIPTS));
    path
});

/// We include the compiled transaction scripts as binaries since the docker files do not copy any
/// source files over. This could then cause issues when the client wishes to use a pre-packaged
/// transaction script.
const STAGED_TXN_SCRIPTS_DIR: Dir = include_dir!("staged/transaction_scripts");

fn compiled_script(script_name: &str) -> Vec<u8> {
    if use_staged() {
        from_staged(script_name)
    } else {
        from_current_source(script_name)
    }
}

fn from_staged(script_name: &str) -> Vec<u8> {
    let mut path = PathBuf::from(script_name);
    path.set_extension(STAGED_EXTENSION);
    STAGED_TXN_SCRIPTS_DIR
        .get_file(path)
        .unwrap()
        .contents()
        .to_vec()
}

// Prepends the directory for the transaction scripts, adds the move file suffix to the given
// source file locatio, and then compiles this.
fn from_current_source(script_name: &str) -> Vec<u8> {
    let mut path = TXN_SCRIPTS_DIR.clone();
    path.push(PathBuf::from(script_name));
    path.set_extension(MOVE_EXTENSION);
    let final_path = path.into_os_string().into_string().unwrap();
    compile_script(final_path)
}

pub static ADD_VALIDATOR_TXN: Lazy<Vec<u8>> = Lazy::new(|| compiled_script("add_validator"));

pub static PEER_TO_PEER_TRANSFER_TXN: Lazy<Vec<u8>> =
    Lazy::new(|| compiled_script("peer_to_peer_transfer"));

pub static PEER_TO_PEER_TRANSFER_WITH_METADATA_TXN: Lazy<Vec<u8>> =
    Lazy::new(|| compiled_script("peer_to_peer_transfer_with_metadata"));

pub static CREATE_ACCOUNT_TXN: Lazy<Vec<u8>> = Lazy::new(|| compiled_script("create_account"));

pub static REGISTER_VALIDATOR_TXN: Lazy<Vec<u8>> =
    Lazy::new(|| compiled_script("register_validator"));

pub static REMOVE_VALIDATOR_TXN: Lazy<Vec<u8>> = Lazy::new(|| compiled_script("remove_validator"));

pub static ROTATE_CONSENSUS_PUBKEY_TXN: Lazy<Vec<u8>> =
    Lazy::new(|| compiled_script("rotate_consensus_pubkey"));

pub static ROTATE_AUTHENTICATION_KEY_TXN: Lazy<Vec<u8>> =
    Lazy::new(|| compiled_script("rotate_authentication_key"));

pub static MINT_TXN: Lazy<Vec<u8>> = Lazy::new(|| compiled_script("mint"));

pub static BLOCK_PROLOGUE_TXN: Lazy<Vec<u8>> = Lazy::new(|| compiled_script("block_prologue"));

pub static EMPTY_TXN: Lazy<Vec<u8>> = Lazy::new(|| compiled_script("empty_script"));
