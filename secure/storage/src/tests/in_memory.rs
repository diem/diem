// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{in_memory::InMemoryStorage, tests::suite};

#[test]
fn in_memory() {
    let mut storage = Box::new(InMemoryStorage::new());
    suite::run_test_suite(storage.as_mut(), "InMemoryStorage");
}

#[test]
fn crypto_key_create_and_get() {
    let mut storage = Box::new(InMemoryStorage::new());
    suite::create_get_and_test_key_pair(storage.as_mut());
}

#[test]
fn crypto_key_create_twice() {
    let mut storage = Box::new(InMemoryStorage::new());
    suite::create_key_pair_twice(storage.as_mut());
}

#[test]
fn crypto_key_get_uncreated() {
    let mut storage = Box::new(InMemoryStorage::new());
    suite::get_uncreated_key_pair(storage.as_mut());
}
