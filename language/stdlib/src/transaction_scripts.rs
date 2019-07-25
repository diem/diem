// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use ir_to_bytecode::parser::{ast::Program, parse_program};
use lazy_static::lazy_static;

/// Returns the source code for peer-to-peer transaction script.
pub fn peer_to_peer() -> &'static str {
    include_str!("../transaction_scripts/peer_to_peer_transfer.mvir")
}

/// Returns the source code for create-account transaction script.
pub fn create_account() -> &'static str {
    include_str!("../transaction_scripts/create_account.mvir")
}

/// Returns the source code for the rotate-key transaction script.
pub fn rotate_key() -> &'static str {
    include_str!("../transaction_scripts/rotate_authentication_key.mvir")
}

/// Returns the source code for the mint transaction script.
pub fn mint() -> &'static str {
    include_str!("../transaction_scripts/mint.mvir")
}

lazy_static! {
    pub static ref PEER_TO_PEER_TRANSFER_TXN_BODY: Program =
        { parse_program(peer_to_peer()).unwrap() };
}

lazy_static! {
    pub static ref CREATE_ACCOUNT_TXN_BODY: Program = parse_program(create_account()).unwrap();
}

lazy_static! {
    pub static ref ROTATE_AUTHENTICATION_KEY_TXN_BODY: Program =
        { parse_program(rotate_key()).unwrap() };
}

lazy_static! {
    pub static ref MINT_TXN_BODY: Program = parse_program(mint()).unwrap();
}
