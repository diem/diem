// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{in_memory::InMemoryStorage, tests::suite};

#[test]
fn in_memory() {
    let mut storage = Box::new(InMemoryStorage::new());
    suite::execute_all_storage_tests(storage.as_mut());
}
