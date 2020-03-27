// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{tests::suite, OnDiskStorage};
use libra_temppath::TempPath;

#[test]
fn on_disk() {
    let path_buf = TempPath::new().path().to_path_buf();
    let mut storage = OnDiskStorage::new_storage(path_buf);
    suite::execute_all_storage_tests(storage.as_mut());
}
