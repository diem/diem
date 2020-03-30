// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    tests::suite, vault::VaultStorage, Capability, Error, Identity, KVStorage, Permission, Policy,
    Value,
};

/// VaultStorage test constants
const VAULT_HOST: &str = "http://localhost:8200";
const VAULT_ROOT_TOKEN: &str = "root_token";
const VAULT_NAMESPACE_1: &str = "namespace_1";
const VAULT_NAMESPACE_2: &str = "namespace_2";
const VAULT_NAMESPACE_3: &str = "namespace_3";

/// Storage data constants for testing purposes.
const U64_KEY_1: &str = "U64 Key 1";
const U64_KEY_2: &str = "U64 Key 2";
const U64_VALUE_1: u64 = 10;
const U64_VALUE_2: u64 = 304;

/// This holds the canonical list of vault storage tests. This is required because vault tests
/// cannot currently be run in parallel, as each test uses the same vault instance and
/// storage resets can interfere between tests. To avoid this, we run each test sequentially, and
/// reset the storage engine only after each test.
const VAULT_TESTS: &[fn()] = &[
    test_suite_multiple_namespaces,
    test_suite_no_namespaces,
    test_vault_namespace_reset,
    test_vault_no_namespace_reset,
    test_vault_tokens_and_basic_ops,
];

/// A test for verifying VaultStorage properly implements the LibraSecureStorage API and enforces
/// strict separation between unique namespaces. This test depends on running Vault, which can be
/// done by using the provided docker run script in `docker/vault/run.sh`
#[test]
#[ignore]
fn execute_storage_tests_vault() {
    let mut storage = create_vault_with_namespace(None);
    storage
        .reset_and_clear()
        .expect("Failed to reset VaultStorage after creation!");

    for test in VAULT_TESTS.iter() {
        test();
        storage
            .reset_and_clear()
            .expect("Failed to reset storage engine between tests!");
    }
}

/// Verifies that when calling reset on a VaultStorage instance, if the instance was created with
/// a namespace, only the secrets within the namespace are removed. This helps to ensure operations
/// across namespaces do not interfere.
fn test_vault_namespace_reset() {
    let mut storage_1 = create_vault_with_namespace(Some(VAULT_NAMESPACE_1.into()));
    let mut storage_2 = create_vault_with_namespace(Some(VAULT_NAMESPACE_2.into()));
    storage_1
        .create(U64_KEY_1, Value::U64(U64_VALUE_1), &Policy::public())
        .unwrap();
    storage_2
        .create(U64_KEY_1, Value::U64(U64_VALUE_1), &Policy::public())
        .unwrap();

    // Verify resets do not occur across namespaces
    storage_1
        .reset_and_clear()
        .expect("Failed to reset VaultStorage with namespace!");
    assert!(storage_1.get(U64_KEY_1).is_err());
    assert_eq!(
        storage_2.get(U64_KEY_1).unwrap().value.u64().unwrap(),
        U64_VALUE_1
    );
}

/// Verifies that when calling reset on a VaultStorage instance, if the instance was created without
/// a namespace, all data in the storage engine will be cleared
fn test_vault_no_namespace_reset() {
    let mut storage_1 = create_vault_with_namespace(Some(VAULT_NAMESPACE_1.into()));
    let mut storage_2 = create_vault_with_namespace(Some(VAULT_NAMESPACE_2.into()));
    storage_1
        .create(U64_KEY_1, Value::U64(U64_VALUE_1), &Policy::public())
        .unwrap();
    storage_2
        .create(U64_KEY_2, Value::U64(U64_VALUE_2), &Policy::public())
        .unwrap();

    // Create a VaultStorage without a namespace, reset the instance, and ensure all data is cleared
    // regardless of namespaces.
    create_vault_with_namespace(None)
        .reset_and_clear()
        .expect("Failed to reset VaultStorage without namespace!");
    assert!(storage_1.get(U64_KEY_1).is_err());
    assert!(storage_2.get(U64_KEY_2).is_err());
}

/// Runs the test suite on a VaultStorage instance that does not use distinct namespaces
fn test_suite_no_namespaces() {
    let mut storage = create_vault_with_namespace(None);
    suite::execute_all_storage_tests(&mut storage);
}

/// Runs the test suite on a VaultStorage instance that supports multiple distinct namespaces.
/// Tests should be able to run across namespaces without interfering.
fn test_suite_multiple_namespaces() {
    let mut storage_1 = create_vault_with_namespace(Some(VAULT_NAMESPACE_1.into()));
    let mut storage_2 = create_vault_with_namespace(Some(VAULT_NAMESPACE_2.into()));
    let mut storage_3 = create_vault_with_namespace(Some(VAULT_NAMESPACE_3.into()));

    suite::execute_all_storage_tests(&mut storage_1);
    suite::execute_all_storage_tests(&mut storage_2);
    suite::execute_all_storage_tests(&mut storage_3);
}

/// Creates and initializes a VaultStorage instance for testing. If a namespace is specified, the
/// instance will perform all storage operations under that namespace.
fn create_vault_with_namespace(namespace: Option<String>) -> VaultStorage {
    VaultStorage::new(VAULT_HOST.into(), VAULT_ROOT_TOKEN.into(), namespace)
}

/// Initializes test policies for a VaultStorage instance and checks the instance is
/// accessible (e.g., by ensuring subsequent read and write operations complete successfully).
fn test_vault_tokens_and_basic_ops() {
    // TODO(davidiw,joshlind): evaluate other systems and determine if create_token can be on the
    // Storage / KV interface. And then refactor this method to make it cleaner and easier to reason
    // about.
    let mut storage = create_vault_with_namespace(None);
    let reader: String = "reader".into();
    let writer: String = "writer".into();

    let anyone = Policy::public();
    let root = Policy::new(vec![]);
    let partial = Policy::new(vec![
        Permission::new(Identity::User(reader.clone()), vec![Capability::Read]),
        Permission::new(
            Identity::User(writer.clone()),
            vec![Capability::Read, Capability::Write],
        ),
    ]);
    let full = Policy::new(vec![
        Permission::new(
            Identity::User(reader.clone()),
            vec![Capability::Read, Capability::Write],
        ),
        Permission::new(
            Identity::User(writer.clone()),
            vec![Capability::Read, Capability::Write],
        ),
    ]);

    // Initialize data and policies

    storage.create("anyone", Value::U64(1), &anyone).unwrap();
    storage.create("root", Value::U64(2), &root).unwrap();
    storage.create("partial", Value::U64(3), &partial).unwrap();
    storage.create("full", Value::U64(4), &full).unwrap();

    // Verify initial reading works correctly

    assert_eq!(storage.get("anyone").unwrap().value, Value::U64(1));
    assert_eq!(storage.get("root").unwrap().value, Value::U64(2));
    assert_eq!(storage.get("partial").unwrap().value, Value::U64(3));
    assert_eq!(storage.get("full").unwrap().value, Value::U64(4));

    let writer_token = storage.create_token(vec![&writer]).unwrap();
    let mut writer = VaultStorage::new(VAULT_HOST.into(), writer_token, storage.namespace());
    assert_eq!(writer.get("anyone").unwrap().value, Value::U64(1));
    assert_eq!(writer.get("root"), Err(Error::PermissionDenied));
    assert_eq!(writer.get("partial").unwrap().value, Value::U64(3));
    assert_eq!(writer.get("full").unwrap().value, Value::U64(4));

    let reader_token = storage.create_token(vec![&reader]).unwrap();
    let mut reader = VaultStorage::new(VAULT_HOST.into(), reader_token, storage.namespace());
    assert_eq!(reader.get("anyone").unwrap().value, Value::U64(1));
    assert_eq!(reader.get("root"), Err(Error::PermissionDenied));
    assert_eq!(reader.get("partial").unwrap().value, Value::U64(3));
    assert_eq!(reader.get("full").unwrap().value, Value::U64(4));

    // Attempt writes followed by reads for correctness

    writer.set("anyone", Value::U64(5)).unwrap();
    assert_eq!(
        writer.set("root", Value::U64(6)),
        Err(Error::PermissionDenied)
    );
    writer.set("partial", Value::U64(7)).unwrap();
    writer.set("full", Value::U64(8)).unwrap();

    assert_eq!(storage.get("anyone").unwrap().value, Value::U64(5));
    assert_eq!(storage.get("root").unwrap().value, Value::U64(2));
    assert_eq!(storage.get("partial").unwrap().value, Value::U64(7));
    assert_eq!(storage.get("full").unwrap().value, Value::U64(8));

    reader.set("anyone", Value::U64(9)).unwrap();
    assert_eq!(
        reader.set("root", Value::U64(10)),
        Err(Error::PermissionDenied)
    );
    assert_eq!(
        reader.set("partial", Value::U64(11)),
        Err(Error::PermissionDenied)
    );
    reader.set("full", Value::U64(12)).unwrap();

    assert_eq!(storage.get("anyone").unwrap().value, Value::U64(9));
    assert_eq!(storage.get("root").unwrap().value, Value::U64(2));
    assert_eq!(storage.get("partial").unwrap().value, Value::U64(7));
    assert_eq!(storage.get("full").unwrap().value, Value::U64(12));
}
