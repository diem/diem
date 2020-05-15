// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use language_e2e_tests::account::Account;
use libra_types::{account_config::transaction_fee_address, on_chain_config::config_address};
use std::collections::BTreeMap;

// These are special-cased since they are generated in genesis, and therefore we don't want
// their account states to be generated.
pub const ASSOCIATION_NAME: &str = "association";
pub const TXN_FEE_NAME: &str = "fees";
pub const CONFIG_NAME: &str = "config";

pub fn make_genesis_accounts() -> BTreeMap<String, Account> {
    let mut m = BTreeMap::new();
    m.insert(ASSOCIATION_NAME.to_string(), Account::new_association());
    m.insert(
        TXN_FEE_NAME.to_string(),
        Account::new_genesis_account(transaction_fee_address()),
    );
    m.insert(
        CONFIG_NAME.to_string(),
        Account::new_genesis_account(config_address()),
    );
    m
}
