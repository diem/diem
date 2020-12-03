// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{tests::suite, OnDiskStorage, Storage};
use diem_temppath::TempPath;

#[test]
fn on_disk() {
    let path_buf = TempPath::new().path().to_path_buf();
    let mut storage = Storage::from(OnDiskStorage::new(path_buf));
    suite::execute_all_storage_tests(&mut storage);
}
