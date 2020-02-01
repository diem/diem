// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{in_memory::InMemoryStorage, tests::suite};

#[test]
fn in_memory() {
    let storage = Box::new(InMemoryStorage::new());
    suite::run_test_suite(storage, "InMemoryStorage");
}
