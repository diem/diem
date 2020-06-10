// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{tests::suite, BoxedStorage, InMemoryStorage};

#[test]
fn in_memory() {
    let mut storage = BoxedStorage::from(InMemoryStorage::new());
    suite::execute_all_storage_tests(&mut storage);
}
