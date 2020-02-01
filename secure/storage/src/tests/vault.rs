// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{tests::suite, vault::VaultStorage};

#[ignore]
#[test]
fn vault() {
    let host = "http://localhost:8200".to_string();
    let token = "root_token".to_string();
    let storage = VaultStorage::new(host, token);
    storage.reset().unwrap();
    suite::run_test_suite(Box::new(storage), "VaultStorage");
}
