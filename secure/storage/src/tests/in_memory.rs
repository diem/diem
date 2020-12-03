// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{tests::suite, InMemoryStorage, Storage};

#[test]
fn in_memory() {
    let mut storage = Storage::from(InMemoryStorage::new());
    suite::execute_all_storage_tests(&mut storage);
}
