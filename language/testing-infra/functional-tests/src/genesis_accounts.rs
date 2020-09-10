// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use language_e2e_tests::account::Account;
use libra_types::account_config;
use std::collections::BTreeMap;

// These are special-cased since they are generated in genesis, and therefore we don't want
// their account states to be generated.
pub const LIBRA_ROOT_NAME: &str = "libraroot";
pub const TREASURY_COMPLIANCE_NAME: &str = "blessed";
pub const TESTNET_DD: &str = "testnetdd";

pub fn make_genesis_accounts() -> BTreeMap<String, Account> {
    let mut m = BTreeMap::new();
    m.insert(LIBRA_ROOT_NAME.to_string(), Account::new_libra_root());
    m.insert(
        TESTNET_DD.to_string(),
        Account::new_genesis_account(account_config::testnet_dd_account_address()),
    );
    m.insert(
        TREASURY_COMPLIANCE_NAME.to_string(),
        Account::new_genesis_account(account_config::treasury_compliance_account_address()),
    );
    m
}
