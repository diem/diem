// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_address::AccountAddress;

pub const CORE_CODE_ADDRESS: AccountAddress = AccountAddress::DEFAULT;

pub fn association_address() -> AccountAddress {
    AccountAddress::from_hex_literal("0xA550C18")
        .expect("Parsing valid hex literal should always succeed")
}

pub fn transaction_fee_address() -> AccountAddress {
    AccountAddress::from_hex_literal("0xFEE")
        .expect("Parsing valid hex literal should always succeed")
}

pub fn validator_set_address() -> AccountAddress {
    AccountAddress::from_hex_literal("0x1D8")
        .expect("Parsing valid hex literal should always succeed")
}

pub fn discovery_set_address() -> AccountAddress {
    AccountAddress::from_hex_literal("0xD15C0")
        .expect("Parsing valid hex literal should always succeed")
}

pub fn burn_account_address() -> AccountAddress {
    AccountAddress::from_hex_literal("0xD1E")
        .expect("Parsing valid hex literal should always succeed")
}
