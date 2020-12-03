// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    tests::suite,
    vault::{VaultEngine, VaultStorage},
    Capability, CryptoStorage, Error, Identity, KVStorage, Permission, Policy, Storage,
};
use diem_crypto::{test_utils::TestDiemCrypto, Signature};
use diem_vault_client::dev::{self, ROOT_TOKEN};

/// VaultStorage namespace constants
const VAULT_NAMESPACE_1: &str = "namespace_1";
const VAULT_NAMESPACE_2: &str = "namespace_2";
const VAULT_NAMESPACE_3: &str = "namespace_3";

/// VaultStorage KV data names
const ANYONE: &str = "anyone";
const ROOT: &str = "root";
const PARTIAL: &str = "partial";
const FULL: &str = "full";

/// VaultStorage Transit key names
const CRYPTO_KEY: &str = "crypto_key";

/// VaultStorage role names
const EXPORTER: &str = "exporter";
const NOONE: &str = "noone";
const READER: &str = "reader";
const ROTATER: &str = "rotater";
const SIGNER: &str = "signer";
const WRITER: &str = "writer";

/// This holds the canonical list of vault storage tests. This is required because vault tests
/// cannot currently be run in parallel, as each test uses the same vault instance and
/// storage resets can interfere between tests. To avoid this, we run each test sequentially, and
/// reset the storage engine only after each test.
const VAULT_TESTS: &[fn()] = &[
    test_suite_multiple_namespaces,
    test_suite_no_namespaces,
    test_vault_cas,
    test_vault_crypto_policies,
    test_vault_key_trimming,
    test_vault_key_value_policies,
    test_vault_tokens,
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
    VaultStorage::new(
        dev::test_host(),
        ROOT_TOKEN.into(),
        namespace,
        None,
        None,
        true,
    )
}

/// Initializes test policies for a VaultStorage instance and checks the instance is
/// accessible (e.g., by ensuring subsequent read and write operations complete successfully).
fn test_vault_key_value_policies() {
    let mut storage = create_vault_with_namespace(None);

    let anyone = Policy::public();
    let root = Policy::new(vec![]);
    let partial = Policy::new(vec![
        Permission::new(Identity::User(READER.into()), vec![Capability::Read]),
        Permission::new(
            Identity::User(WRITER.into()),
            vec![Capability::Read, Capability::Write],
        ),
    ]);
    let full = Policy::new(vec![
        Permission::new(
            Identity::User(READER.into()),
            vec![Capability::Read, Capability::Write],
        ),
        Permission::new(
            Identity::User(WRITER.into()),
            vec![Capability::Read, Capability::Write],
        ),
    ]);

    // Provide a TTL to verify that lease renews work
    let ttl = Some(3600);

    // Initialize data and policies
    storage.set(ANYONE, 1).unwrap();
    storage
        .set_policies(ANYONE, &VaultEngine::KVSecrets, &anyone)
        .unwrap();

    storage.set(ROOT, 2).unwrap();
    storage
        .set_policies(ROOT, &VaultEngine::KVSecrets, &root)
        .unwrap();

    storage.set(PARTIAL, 3).unwrap();
    storage
        .set_policies(PARTIAL, &VaultEngine::KVSecrets, &partial)
        .unwrap();

    storage.set(FULL, 4).unwrap();
    storage
        .set_policies(FULL, &VaultEngine::KVSecrets, &full)
        .unwrap();

    // Verify initial reading works correctly
    assert_eq!(storage.get::<u64>(ANYONE).unwrap().value, 1);
    assert_eq!(storage.get::<u64>(ROOT).unwrap().value, 2);
    assert_eq!(storage.get::<u64>(PARTIAL).unwrap().value, 3);
    assert_eq!(storage.get::<u64>(FULL).unwrap().value, 4);

    let writer_token = storage.create_token(vec![&WRITER]).unwrap();
    let mut writer = VaultStorage::new(
        dev::test_host(),
        writer_token.clone(),
        None,
        None,
        ttl,
        false,
    );
    assert_eq!(writer.get::<u64>(ANYONE).unwrap().value, 1);
    assert_eq!(writer.get::<u64>(ROOT), Err(Error::PermissionDenied));
    assert_eq!(writer.get::<u64>(PARTIAL).unwrap().value, 3);
    assert_eq!(writer.get::<u64>(FULL).unwrap().value, 4);

    let reader_token = storage.create_token(vec![&READER]).unwrap();
    let mut reader = VaultStorage::new(
        dev::test_host(),
        reader_token.clone(),
        None,
        None,
        ttl,
        false,
    );
    assert_eq!(reader.get::<u64>(ANYONE).unwrap().value, 1);
    assert_eq!(reader.get::<u64>(ROOT), Err(Error::PermissionDenied));
    assert_eq!(reader.get::<u64>(PARTIAL).unwrap().value, 3);
    assert_eq!(reader.get::<u64>(FULL).unwrap().value, 4);

    // Attempt writes followed by reads for correctness
    writer.set(ANYONE, 5).unwrap();
    assert_eq!(writer.set(ROOT, 6), Err(Error::PermissionDenied));
    writer.set(PARTIAL, 7).unwrap();
    writer.set(FULL, 8).unwrap();

    assert_eq!(storage.get::<u64>(ANYONE).unwrap().value, 5);
    assert_eq!(storage.get::<u64>(ROOT).unwrap().value, 2);
    assert_eq!(storage.get::<u64>(PARTIAL).unwrap().value, 7);
    assert_eq!(storage.get::<u64>(FULL).unwrap().value, 8);

    reader.set(ANYONE, 9).unwrap();
    assert_eq!(reader.set(ROOT, 10), Err(Error::PermissionDenied));
    assert_eq!(reader.set(PARTIAL, 11), Err(Error::PermissionDenied));
    reader.set(FULL, 12).unwrap();

    assert_eq!(storage.get::<u64>(ANYONE).unwrap().value, 9);
    assert_eq!(storage.get::<u64>(ROOT).unwrap().value, 2);
    assert_eq!(storage.get::<u64>(PARTIAL).unwrap().value, 7);
    assert_eq!(storage.get::<u64>(FULL).unwrap().value, 12);

    // Verify a reader with another namespace has no permission for the operations
    let mut reader_with_namespace = VaultStorage::new(
        dev::test_host(),
        reader_token,
        Some(VAULT_NAMESPACE_1.into()),
        None,
        ttl,
        false,
    );
    assert_eq!(
        reader_with_namespace.get::<u64>(ANYONE),
        Err(Error::PermissionDenied)
    );
    assert_eq!(
        reader_with_namespace.set(ANYONE, 5),
        Err(Error::PermissionDenied)
    );

    // Verify a writer with another namespace has no permission for the operations
    let mut writer_with_namespace = VaultStorage::new(
        dev::test_host(),
        writer_token,
        Some(VAULT_NAMESPACE_1.into()),
        None,
        ttl,
        false,
    );
    assert_eq!(
        writer_with_namespace.get::<u64>(ANYONE),
        Err(Error::PermissionDenied)
    );
    assert_eq!(
        writer_with_namespace.set(ANYONE, 5),
        Err(Error::PermissionDenied)
    );
}

fn test_vault_crypto_policies() {
    let mut storage = create_vault_with_namespace(None);

    let policy = Policy::new(vec![
        Permission::new(Identity::User(EXPORTER.into()), vec![Capability::Export]),
        Permission::new(Identity::User(READER.into()), vec![Capability::Read]),
        Permission::new(
            Identity::User(ROTATER.into()),
            vec![Capability::Read, Capability::Rotate],
        ),
        Permission::new(Identity::User(SIGNER.into()), vec![Capability::Sign]),
    ]);

    // Initialize data and policies
    let pubkey = storage.create_key(CRYPTO_KEY).unwrap();
    storage
        .set_policies(CRYPTO_KEY, &VaultEngine::Transit, &policy)
        .unwrap();
    assert_eq!(
        storage.get_public_key(CRYPTO_KEY).unwrap().public_key,
        pubkey
    );

    let message = TestDiemCrypto("Hello, World".to_string());

    // Verify exporter policy
    let exporter_token = storage.create_token(vec![&EXPORTER]).unwrap();
    let mut exporter_store = VaultStorage::new(
        dev::test_host(),
        exporter_token.clone(),
        None,
        None,
        None,
        true,
    );
    exporter_store.export_private_key(CRYPTO_KEY).unwrap();
    exporter_store.get_public_key(CRYPTO_KEY).unwrap_err();
    exporter_store.rotate_key(CRYPTO_KEY).unwrap_err();
    exporter_store.sign(CRYPTO_KEY, &message).unwrap_err();
    // Verify a exporter with another namespace has no permission for the operations
    let exporter_store_with_namespace = VaultStorage::new(
        dev::test_host(),
        exporter_token,
        Some(VAULT_NAMESPACE_1.into()),
        None,
        None,
        true,
    );
    exporter_store_with_namespace
        .export_private_key(CRYPTO_KEY)
        .unwrap_err();

    // Verify noone policy
    let noone_token = storage.create_token(vec![&NOONE]).unwrap();
    let mut noone_store = VaultStorage::new(dev::test_host(), noone_token, None, None, None, true);
    noone_store.export_private_key(CRYPTO_KEY).unwrap_err();
    noone_store.get_public_key(CRYPTO_KEY).unwrap_err();
    noone_store.rotate_key(CRYPTO_KEY).unwrap_err();
    noone_store.sign(CRYPTO_KEY, &message).unwrap_err();

    // Verify reader policy
    let reader_token = storage.create_token(vec![&READER]).unwrap();
    let mut reader_store = VaultStorage::new(
        dev::test_host(),
        reader_token.clone(),
        None,
        None,
        None,
        true,
    );
    reader_store.export_private_key(CRYPTO_KEY).unwrap_err();
    assert_eq!(
        reader_store.get_public_key(CRYPTO_KEY).unwrap().public_key,
        pubkey
    );
    reader_store.rotate_key(CRYPTO_KEY).unwrap_err();
    reader_store.sign(CRYPTO_KEY, &message).unwrap_err();
    // Verify a reader with another namespace has no permission for the operations
    let reader_store_with_namespace = VaultStorage::new(
        dev::test_host(),
        reader_token,
        Some(VAULT_NAMESPACE_1.into()),
        None,
        None,
        true,
    );
    reader_store_with_namespace
        .get_public_key(CRYPTO_KEY)
        .unwrap_err();

    // Verify rotater policy
    let rotater_token = storage.create_token(vec![&ROTATER]).unwrap();
    let mut rotater_store = VaultStorage::new(
        dev::test_host(),
        rotater_token.clone(),
        None,
        None,
        None,
        true,
    );
    rotater_store.export_private_key(CRYPTO_KEY).unwrap_err();
    assert_eq!(
        rotater_store.get_public_key(CRYPTO_KEY).unwrap().public_key,
        pubkey
    );
    assert_ne!(rotater_store.rotate_key(CRYPTO_KEY).unwrap(), pubkey);
    rotater_store.sign(CRYPTO_KEY, &message).unwrap_err();
    // Verify a rotater with another namespace has no permission for the operations
    let mut rotater_store_with_namespace = VaultStorage::new(
        dev::test_host(),
        rotater_token,
        Some(VAULT_NAMESPACE_1.into()),
        None,
        None,
        true,
    );
    rotater_store_with_namespace
        .rotate_key(CRYPTO_KEY)
        .unwrap_err();

    let new_pubkey = storage.get_public_key(CRYPTO_KEY).unwrap().public_key;

    // Verify signer policy
    let signer_token = storage.create_token(vec![&SIGNER]).unwrap();
    let mut signer_store = VaultStorage::new(
        dev::test_host(),
        signer_token.clone(),
        None,
        None,
        None,
        true,
    );
    signer_store.export_private_key(CRYPTO_KEY).unwrap_err();
    signer_store.get_public_key(CRYPTO_KEY).unwrap_err();
    signer_store.rotate_key(CRYPTO_KEY).unwrap_err();
    let signature = signer_store.sign(CRYPTO_KEY, &message).unwrap();
    signature.verify(&message, &pubkey).unwrap_err();
    signature.verify(&message, &new_pubkey).unwrap();
    // Verify a signer with another namespace has no permission for the operations
    let signer_store_with_namespace = VaultStorage::new(
        dev::test_host(),
        signer_token,
        Some(VAULT_NAMESPACE_1.into()),
        None,
        None,
        true,
    );
    signer_store_with_namespace
        .sign(CRYPTO_KEY, &message)
        .unwrap_err();
}

fn test_vault_tokens() {
    let mut storage = create_vault_with_namespace(Some(VAULT_NAMESPACE_1.into()));

    let partial = Policy::new(vec![Permission::new(
        Identity::User(WRITER.into()),
        vec![Capability::Read, Capability::Write],
    )]);

    // Initialize data and policies
    storage.set(PARTIAL, 3).unwrap();
    storage
        .set_policies(PARTIAL, &VaultEngine::KVSecrets, &partial)
        .unwrap();

    let writer_token = storage.create_token(vec![&WRITER]).unwrap();
    let mut writer = VaultStorage::new(
        dev::test_host(),
        writer_token.clone(),
        Some(VAULT_NAMESPACE_1.into()),
        None,
        None,
        true,
    );

    // Verify reads and write succeed
    assert_eq!(writer.get::<u64>(PARTIAL).unwrap().value, 3);
    writer.set::<u64>(PARTIAL, 5).unwrap();

    // Verify a writer without a namespace has no permission for the operations
    let mut writer_without_namespace =
        VaultStorage::new(dev::test_host(), writer_token, None, None, None, true);
    assert_eq!(
        writer_without_namespace.get::<u64>(PARTIAL),
        Err(Error::PermissionDenied)
    );
    assert_eq!(
        writer_without_namespace.set::<u64>(PARTIAL, 5),
        Err(Error::PermissionDenied)
    );

    // Revoke the token and verify failure
    writer.revoke_token_self().unwrap();
    assert_eq!(writer.get::<u64>(PARTIAL), Err(Error::PermissionDenied));

    // Try to use an invalid token and verify failure
    let writer = VaultStorage::new(
        dev::test_host(),
        "INVALID TOKEN".into(),
        Some(VAULT_NAMESPACE_1.into()),
        None,
        None,
        true,
    );
    assert_eq!(writer.get::<u64>(PARTIAL), Err(Error::PermissionDenied));
}

fn test_vault_cas() {
    let mut with_cas = create_vault_with_namespace(None);
    let mut without_cas =
        VaultStorage::new(dev::test_host(), ROOT_TOKEN.into(), None, None, None, false);

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
