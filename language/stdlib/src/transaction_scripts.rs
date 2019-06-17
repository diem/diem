// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use compiler::parser::{ast::Program, parse_program};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref PEER_TO_PEER_TRANSFER_TXN_BODY: Program = {
        let txn_body = include_str!("../transaction_scripts/peer_to_peer_transfer.mvir");
        parse_program(txn_body).unwrap()
    };
}

lazy_static! {
    pub static ref CREATE_ACCOUNT_TXN_BODY: Program = {
        let txn_body = include_str!("../transaction_scripts/create_account.mvir");
        parse_program(txn_body).unwrap()
    };
}

lazy_static! {
    pub static ref ROTATE_AUTHENTICATION_KEY_TXN_BODY: Program = {
        let txn_body = include_str!("../transaction_scripts/rotate_authentication_key.mvir");
        parse_program(txn_body).unwrap()
    };
}

lazy_static! {
    pub static ref MINT_TXN_BODY: Program = {
        let txn_body = include_str!("../transaction_scripts/mint.mvir");
        parse_program(txn_body).unwrap()
    };
}
