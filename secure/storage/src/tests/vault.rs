// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{tests::suite, vault::VaultStorage, CryptoStorage, KVStorage, Storage};
use diem_vault_client::dev::{self, ROOT_TOKEN};

/// VaultStorage namespace constants
const VAULT_NAMESPACE_1: &str = "namespace_1";
const VAULT_NAMESPACE_2: &str = "namespace_2";
const VAULT_NAMESPACE_3: &str = "namespace_3";

/// VaultStorage Transit key names
const CRYPTO_KEY: &str = "crypto_key";

/// This holds the canonical list of vault storage tests. This is required because vault tests
/// cannot currently be run in parallel, as each test uses the same vault instance and
/// storage resets can interfere between tests. To avoid this, we run each test sequentially, and
/// reset the storage engine only after each test.
const VAULT_TESTS: &[fn()] = &[
    test_suite_multiple_namespaces,
    test_suite_no_namespaces,
    test_vault_cas,
    test_vault_key_trimming,
];

/// A test for verifying VaultStorage properly implements the DiemSecureStorage API and enforces
/// strict separation between unique namespaces. This test depends on running Vault, which can be
/// done by using the provided docker run script in `docker/vault/run.sh`
#[test]
fn execute_storage_tests_vault() {
    if dev::test_host_safe().is_none() {
        return;
    }
    let mut storage = create_vault_with_namespace(None);
    storage.reset_and_clear().unwrap();

    for test in VAULT_TESTS.iter() {
        test();
        storage.reset_and_clear().unwrap();
    }
}

/// Runs the test suite on a VaultStorage instance that does not use distinct namespaces
fn test_suite_no_namespaces() {
    let mut storage = Storage::from(create_vault_with_namespace(None));
    suite::execute_all_storage_tests(&mut storage);
}

/// Runs the test suite on a VaultStorage instance that supports multiple distinct namespaces.
/// Tests should be able to run across namespaces without interfering.
fn test_suite_multiple_namespaces() {
    let mut storage_1 = Storage::from(create_vault_with_namespace(Some(VAULT_NAMESPACE_1.into())));
    let mut storage_2 = Storage::from(create_vault_with_namespace(Some(VAULT_NAMESPACE_2.into())));
    let mut storage_3 = Storage::from(create_vault_with_namespace(Some(VAULT_NAMESPACE_3.into())));

    suite::execute_all_storage_tests(&mut storage_1);
    suite::execute_all_storage_tests(&mut storage_2);
    suite::execute_all_storage_tests(&mut storage_3);
}

/// Creates and initializes a VaultStorage instance for testing. If a namespace is specified, the
/// instance will perform all storage operations under that namespace.
fn create_vault_with_namespace(namespace: Option<String>) -> VaultStorage {
    create_vault_storage(ROOT_TOKEN.into(), namespace, None, true)
}

fn create_vault_storage(
    token: String,
    namespace: Option<String>,
    renew_ttl_secs: Option<u32>,
    use_cas: bool,
) -> VaultStorage {
    VaultStorage::new(
        dev::test_host(),
        token,
        namespace,
        None,
        renew_ttl_secs,
        use_cas,
        None,
        None,
    )
}

fn test_vault_cas() {
    let mut with_cas = create_vault_with_namespace(None);
    let mut without_cas = create_vault_storage(ROOT_TOKEN.into(), None, None, false);
    // Test initial write with no version
    with_cas.set("test", 1).unwrap();
    assert_eq!(with_cas.get::<u64>("test").unwrap().value, 1);

    // Test subsequent write with version
    with_cas.set("test", 2).unwrap();
    assert_eq!(with_cas.get::<u64>("test").unwrap().value, 2);

    // Test that version is updated on writes
    with_cas.set("test", 3).unwrap();
    with_cas.set("test", 4).unwrap();
    assert_eq!(with_cas.get::<u64>("test").unwrap().value, 4);

    // Test that CAS is not used if disabled
    without_cas.set("test", 5).unwrap();
    assert_eq!(without_cas.get::<u64>("test").unwrap().value, 5);

    // Test that write fails if version doesn't match
    with_cas.set("test", 6).unwrap_err();

    // Test that reading updates the version
    assert_eq!(with_cas.get::<u64>("test").unwrap().value, 5);
    with_cas.set("test", 6).unwrap();
    assert_eq!(with_cas.get::<u64>("test").unwrap().value, 6);
}

fn test_vault_key_trimming() {
    let mut storage = create_vault_with_namespace(None);

    // Create key
    let _ = storage.create_key(CRYPTO_KEY).unwrap();

    // Verify only one key version exists
    let key_versions = storage.get_all_key_versions(CRYPTO_KEY).unwrap();
    assert_eq!(1, key_versions.len());
    assert_eq!(1, key_versions.first().unwrap().version);

    // Rotate key 3 times and verify incrementing versions
    for i in 2u32..4 {
        let new_key = storage.rotate_key(CRYPTO_KEY).unwrap();

        let key_versions = storage.get_all_key_versions(CRYPTO_KEY).unwrap();
        let max_version = key_versions.iter().map(|resp| resp.version).max().unwrap();
        let min_version = key_versions.iter().map(|resp| resp.version).min().unwrap();

        assert_eq!(i, key_versions.len() as u32);
        assert_eq!(i, max_version);
        assert_eq!(1, min_version);

        // Verify the key returned by rotate_key() has the highest version
        assert_eq!(
            new_key,
            key_versions
                .iter()
                .find(|resp| resp.version == max_version)
                .unwrap()
                .value
        );
    }

    // Rotate key 10 more times and verify key trimming occurs
    for i in 4u32..13 {
        let new_key = storage.rotate_key(CRYPTO_KEY).unwrap();

        let key_versions = storage.get_all_key_versions(CRYPTO_KEY).unwrap();
        let max_version = key_versions.iter().map(|resp| resp.version).max().unwrap();
        let min_version = key_versions.iter().map(|resp| resp.version).min().unwrap();

        assert_eq!(4, key_versions.len() as u32);
        assert_eq!(i, max_version);
        assert_eq!(i - 3, min_version);

        // Verify the key returned by rotate_key() has the highest version
        assert_eq!(
            new_key,
            key_versions
                .iter()
                .find(|resp| resp.version == max_version)
                .unwrap()
                .value
        );
    }
}
