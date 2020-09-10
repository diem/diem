// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    tests::suite,
    vault::{VaultEngine, VaultStorage},
    Capability, CryptoStorage, Error, Identity, KVStorage, Permission, Policy, Storage,
};
use libra_crypto::{test_utils::TestLibraCrypto, Signature};
use libra_vault_client::dev::{self, ROOT_TOKEN};

/// VaultStorage test constants
const VAULT_NAMESPACE_1: &str = "namespace_1";
const VAULT_NAMESPACE_2: &str = "namespace_2";
const VAULT_NAMESPACE_3: &str = "namespace_3";

/// This holds the canonical list of vault storage tests. This is required because vault tests
/// cannot currently be run in parallel, as each test uses the same vault instance and
/// storage resets can interfere between tests. To avoid this, we run each test sequentially, and
/// reset the storage engine only after each test.
const VAULT_TESTS: &[fn()] = &[
    test_suite_multiple_namespaces,
    test_suite_no_namespaces,
    test_vault_crypto_policies,
    test_vault_key_value_policies,
];

/// A test for verifying VaultStorage properly implements the LibraSecureStorage API and enforces
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
    VaultStorage::new(dev::test_host(), ROOT_TOKEN.into(), namespace, None, None)
}

/// Initializes test policies for a VaultStorage instance and checks the instance is
/// accessible (e.g., by ensuring subsequent read and write operations complete successfully).
fn test_vault_key_value_policies() {
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

    // Provide a TTL to verify that lease renews work
    let ttl = Some(3600);

    // Initialize data and policies

    storage.set("anyone", 1).unwrap();
    storage
        .set_policies("anyone", &VaultEngine::KVSecrets, &anyone)
        .unwrap();

    storage.set("root", 2).unwrap();
    storage
        .set_policies("root", &VaultEngine::KVSecrets, &root)
        .unwrap();

    storage.set("partial", 3).unwrap();
    storage
        .set_policies("partial", &VaultEngine::KVSecrets, &partial)
        .unwrap();

    storage.set("full", 4).unwrap();
    storage
        .set_policies("full", &VaultEngine::KVSecrets, &full)
        .unwrap();

    // Verify initial reading works correctly

    assert_eq!(storage.get::<u64>("anyone").unwrap().value, 1);
    assert_eq!(storage.get::<u64>("root").unwrap().value, 2);
    assert_eq!(storage.get::<u64>("partial").unwrap().value, 3);
    assert_eq!(storage.get::<u64>("full").unwrap().value, 4);

    let writer_token = storage.create_token(vec![&writer]).unwrap();
    let mut writer = VaultStorage::new(dev::test_host(), writer_token, None, None, ttl);
    assert_eq!(writer.get::<u64>("anyone").unwrap().value, 1);
    assert_eq!(writer.get::<u64>("root"), Err(Error::PermissionDenied));
    assert_eq!(writer.get::<u64>("partial").unwrap().value, 3);
    assert_eq!(writer.get::<u64>("full").unwrap().value, 4);

    let reader_token = storage.create_token(vec![&reader]).unwrap();
    let mut reader = VaultStorage::new(dev::test_host(), reader_token, None, None, ttl);
    assert_eq!(reader.get::<u64>("anyone").unwrap().value, 1);
    assert_eq!(reader.get::<u64>("root"), Err(Error::PermissionDenied));
    assert_eq!(reader.get::<u64>("partial").unwrap().value, 3);
    assert_eq!(reader.get::<u64>("full").unwrap().value, 4);

    // Attempt writes followed by reads for correctness

    writer.set("anyone", 5).unwrap();
    assert_eq!(writer.set("root", 6), Err(Error::PermissionDenied));
    writer.set("partial", 7).unwrap();
    writer.set("full", 8).unwrap();

    assert_eq!(storage.get::<u64>("anyone").unwrap().value, 5);
    assert_eq!(storage.get::<u64>("root").unwrap().value, 2);
    assert_eq!(storage.get::<u64>("partial").unwrap().value, 7);
    assert_eq!(storage.get::<u64>("full").unwrap().value, 8);

    reader.set("anyone", 9).unwrap();
    assert_eq!(reader.set("root", 10), Err(Error::PermissionDenied));
    assert_eq!(reader.set("partial", 11), Err(Error::PermissionDenied));
    reader.set("full", 12).unwrap();

    assert_eq!(storage.get::<u64>("anyone").unwrap().value, 9);
    assert_eq!(storage.get::<u64>("root").unwrap().value, 2);
    assert_eq!(storage.get::<u64>("partial").unwrap().value, 7);
    assert_eq!(storage.get::<u64>("full").unwrap().value, 12);
}

fn test_vault_crypto_policies() {
    let mut storage = create_vault_with_namespace(None);
    let exporter: String = "exporter".into();
    let noone: String = "noone".into();
    let reader: String = "reader".into();
    let rotater: String = "rotater".into();
    let signer: String = "signer".into();

    let policy = Policy::new(vec![
        Permission::new(Identity::User(exporter.clone()), vec![Capability::Export]),
        Permission::new(Identity::User(reader.clone()), vec![Capability::Read]),
        Permission::new(
            Identity::User(rotater.clone()),
            vec![Capability::Read, Capability::Rotate],
        ),
        Permission::new(Identity::User(signer.clone()), vec![Capability::Sign]),
    ]);

    // Initialize data and policies
    let key_name = "crypto_key";
    let pubkey = storage.create_key(key_name).unwrap();
    storage
        .set_policies(key_name, &VaultEngine::Transit, &policy)
        .unwrap();
    assert_eq!(storage.get_public_key(key_name).unwrap().public_key, pubkey);

    let message = TestLibraCrypto("Hello, World".to_string());

    // Verify exporter policy
    let exporter_token = storage.create_token(vec![&exporter]).unwrap();
    let mut exporter_store = VaultStorage::new(dev::test_host(), exporter_token, None, None, None);
    exporter_store.export_private_key(key_name).unwrap();
    exporter_store.get_public_key(key_name).unwrap_err();
    exporter_store.rotate_key(key_name).unwrap_err();
    exporter_store.sign(key_name, &message).unwrap_err();

    // Verify noone policy
    let noone_token = storage.create_token(vec![&noone]).unwrap();
    let mut noone_store = VaultStorage::new(dev::test_host(), noone_token, None, None, None);
    noone_store.export_private_key(key_name).unwrap_err();
    noone_store.get_public_key(key_name).unwrap_err();
    noone_store.rotate_key(key_name).unwrap_err();
    noone_store.sign(key_name, &message).unwrap_err();

    // Verify reader policy
    let reader_token = storage.create_token(vec![&reader]).unwrap();
    let mut reader_store = VaultStorage::new(dev::test_host(), reader_token, None, None, None);
    reader_store.export_private_key(key_name).unwrap_err();
    assert_eq!(
        reader_store.get_public_key(key_name).unwrap().public_key,
        pubkey
    );
    reader_store.rotate_key(key_name).unwrap_err();
    reader_store.sign(key_name, &message).unwrap_err();

    // Verify rotater policy
    let rotater_token = storage.create_token(vec![&rotater]).unwrap();
    let mut rotater_store = VaultStorage::new(dev::test_host(), rotater_token, None, None, None);
    rotater_store.export_private_key(key_name).unwrap_err();
    assert_eq!(
        rotater_store.get_public_key(key_name).unwrap().public_key,
        pubkey
    );
    assert_ne!(rotater_store.rotate_key(key_name).unwrap(), pubkey);
    rotater_store.sign(key_name, &message).unwrap_err();

    let new_pubkey = storage.get_public_key(key_name).unwrap().public_key;

    // Verify signer policy
    let signer_token = storage.create_token(vec![&signer]).unwrap();
    let mut signer_store = VaultStorage::new(dev::test_host(), signer_token, None, None, None);
    signer_store.export_private_key(key_name).unwrap_err();
    signer_store.get_public_key(key_name).unwrap_err();
    signer_store.rotate_key(key_name).unwrap_err();
    let signature = signer_store.sign(key_name, &message).unwrap();
    signature.verify(&message, &pubkey).unwrap_err();
    signature.verify(&message, &new_pubkey).unwrap();
}
