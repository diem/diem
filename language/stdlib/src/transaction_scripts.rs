// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{compile_script, MOVE_EXTENSION};
use once_cell::sync::Lazy;
use std::path::PathBuf;

// Encodes the absolute path for the transaction scripts directory.
static TXN_SCRIPTS_DIR: Lazy<PathBuf> = Lazy::new(|| {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(PathBuf::from("transaction_scripts"));
    path
});

// Prepends the directory for the transaction scripts and adds the move file suffix to the given
// source location.
fn script_source(file_path: &str) -> String {
    let mut path = TXN_SCRIPTS_DIR.clone();
    path.push(PathBuf::from(file_path));
    path.set_extension(MOVE_EXTENSION);
    path.into_os_string().into_string().unwrap()
}

pub static ADD_VALIDATOR_TXN: Lazy<Vec<u8>> =
    Lazy::new(|| compile_script(script_source("add_validator")));

pub static PEER_TO_PEER_TRANSFER_TXN: Lazy<Vec<u8>> =
    Lazy::new(|| compile_script(script_source("peer_to_peer_transfer")));

pub static PEER_TO_PEER_TRANSFER_WITH_METADATA_TXN: Lazy<Vec<u8>> =
    Lazy::new(|| compile_script(script_source("peer_to_peer_transfer_with_metadata")));

pub static CREATE_ACCOUNT_TXN: Lazy<Vec<u8>> =
    Lazy::new(|| compile_script(script_source("create_account")));

pub static REGISTER_VALIDATOR_TXN: Lazy<Vec<u8>> =
    Lazy::new(|| compile_script(script_source("register_validator")));

pub static REMOVE_VALIDATOR_TXN: Lazy<Vec<u8>> =
    Lazy::new(|| compile_script(script_source("remove_validator")));

pub static ROTATE_CONSENSUS_PUBKEY_TXN: Lazy<Vec<u8>> =
    Lazy::new(|| compile_script(script_source("rotate_consensus_pubkey")));

pub static ROTATE_AUTHENTICATION_KEY_TXN: Lazy<Vec<u8>> =
    Lazy::new(|| compile_script(script_source("rotate_authentication_key")));

pub static MINT_TXN: Lazy<Vec<u8>> = Lazy::new(|| compile_script(script_source("mint")));

pub static BLOCK_PROLOGUE_TXN: Lazy<Vec<u8>> =
    Lazy::new(|| compile_script(script_source("block_prologue")));

pub static EMPTY_TXN: Lazy<Vec<u8>> = Lazy::new(|| compile_script(script_source("empty_script")));
