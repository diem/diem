// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use ir_to_bytecode::parser::{ast::Program, parse_program};
use lazy_static::lazy_static;

/// Returns the source code for the add validator transaction script
pub fn add_validator() -> &'static str {
    include_str!("../transaction_scripts/add_validator.mvir")
}

/// Returns the source code for peer-to-peer transaction script.
pub fn peer_to_peer() -> &'static str {
    include_str!("../transaction_scripts/peer_to_peer_transfer.mvir")
}

/// Returns the source code for peer-to-peer transaction script with metadata.
pub fn peer_to_peer_with_metadata() -> &'static str {
    include_str!("../transaction_scripts/peer_to_peer_transfer_with_metadata.mvir")
}

/// Returns the source code for create-account transaction script.
pub fn create_account() -> &'static str {
    include_str!("../transaction_scripts/create_account.mvir")
}

/// Returns the source code for the register validator transaction script
pub fn register_validator() -> &'static str {
    include_str!("../transaction_scripts/register_validator.mvir")
}

/// Returns the source code for the remove validator transaction script
pub fn remove_validator() -> &'static str {
    include_str!("../transaction_scripts/remove_validator.mvir")
}

/// Returns the source code for the rotate-consensus-pubkey script.
pub fn rotate_consensus_pubkey() -> &'static str {
    include_str!("../transaction_scripts/rotate_consensus_pubkey.mvir")
}

/// Returns the source code for the rotate-key transaction script.
pub fn rotate_key() -> &'static str {
    include_str!("../transaction_scripts/rotate_authentication_key.mvir")
}

/// Returns the source code for the mint transaction script.
pub fn mint() -> &'static str {
    include_str!("../transaction_scripts/mint.mvir")
}

/// Returns the source code for the block prologue script
pub fn block_prologue() -> &'static str {
    include_str!("../transaction_scripts/block_prologue.mvir")
}

lazy_static! {
    pub static ref ADD_VALIDATOR_TXN_BODY: Program = { parse_program(add_validator()).unwrap() };
}

lazy_static! {
    pub static ref PEER_TO_PEER_TRANSFER_TXN_BODY: Program =
        { parse_program(peer_to_peer()).unwrap() };
}

lazy_static! {
    pub static ref PEER_TO_PEER_TRANSFER_WITH_METADATA_TXN_BODY: Program =
        { parse_program(peer_to_peer_with_metadata()).unwrap() };
}

lazy_static! {
    pub static ref CREATE_ACCOUNT_TXN_BODY: Program = parse_program(create_account()).unwrap();
}

lazy_static! {
    pub static ref REGISTER_VALIDATOR_TXN_BODY: Program =
        { parse_program(register_validator()).unwrap() };
}

lazy_static! {
    pub static ref REMOVE_VALIDATOR_TXN_BODY: Program =
        { parse_program(remove_validator()).unwrap() };
}

lazy_static! {
    pub static ref ROTATE_CONSENSUS_PUBKEY_TXN_BODY: Program =
        { parse_program(rotate_consensus_pubkey()).unwrap() };
}

lazy_static! {
    pub static ref ROTATE_AUTHENTICATION_KEY_TXN_BODY: Program =
        { parse_program(rotate_key()).unwrap() };
}

lazy_static! {
    pub static ref MINT_TXN_BODY: Program = parse_program(mint()).unwrap();
}

lazy_static! {
    pub static ref BLOCK_PROLOGUE_TXN_BODY: Program = parse_program(block_prologue()).unwrap();
}
