// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{on_disk::OnDiskStorage, tests::suite};
use libra_temppath::TempPath;

#[test]
fn on_disk() {
    let temp_path = TempPath::new();
    let mut storage = Box::new(OnDiskStorage::new(temp_path.path().to_path_buf()));
    suite::run_test_suite(storage.as_mut(), "OnDiskStorage");
}

#[test]
fn crypto_key_create_and_get() {
    let temp_path = TempPath::new();
    let mut storage = Box::new(OnDiskStorage::new(temp_path.path().to_path_buf()));
    suite::create_get_and_test_key_pair(storage.as_mut());
}

#[test]
fn crypto_key_create_twice() {
    let temp_path = TempPath::new();
    let mut storage = Box::new(OnDiskStorage::new(temp_path.path().to_path_buf()));
    suite::create_key_pair_twice(storage.as_mut());
}

#[test]
fn crypto_key_get_uncreated() {
    let temp_path = TempPath::new();
    let mut storage = Box::new(OnDiskStorage::new(temp_path.path().to_path_buf()));
    suite::get_uncreated_key_pair(storage.as_mut());
}
