// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_address::AccountAddress;

pub use move_core_types::language_storage::CORE_CODE_ADDRESS;

pub fn association_address() -> AccountAddress {
    AccountAddress::from_hex_literal("0xA550C18")
        .expect("Parsing valid hex literal should always succeed")
}

pub fn transaction_fee_address() -> AccountAddress {
    AccountAddress::from_hex_literal("0xFEE")
        .expect("Parsing valid hex literal should always succeed")
}

pub fn validator_set_address() -> AccountAddress {
    crate::on_chain_config::config_address()
}

pub fn treasury_compliance_account_address() -> AccountAddress {
    AccountAddress::from_hex_literal("0xB1E55ED")
        .expect("Parsing valid hex literal should always succeed")
}
