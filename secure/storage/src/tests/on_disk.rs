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

#[cfg(unix)]
#[test]
fn on_disk_file_is_owner_only() {
    use crate::KVStorage;
    use std::os::unix::fs::PermissionsExt;

    let path_buf = TempPath::new().path().to_path_buf();
    let mut storage = OnDiskStorage::new(path_buf.clone());
    let mode = std::fs::metadata(&path_buf).unwrap().permissions().mode();
    assert_eq!(mode & 0o777, 0o600);

    storage.set("key", "value").unwrap();
    let mode = std::fs::metadata(&path_buf).unwrap().permissions().mode();
    assert_eq!(mode & 0o777, 0o600);
}
