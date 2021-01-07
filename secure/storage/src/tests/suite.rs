// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{CryptoStorage, Error, KVStorage, Storage};

use diem_crypto::{
    ed25519::Ed25519PrivateKey, test_utils::TestDiemCrypto, HashValue, PrivateKey, Signature,
    Uniform,
};

/// This suite contains tests for secure storage backends. We test the correct functionality
/// of both key/value and cryptographic operations for storage implementations. All storage backend
/// implementations should be tested using the tests in this suite.

/// This holds the canonical list of secure storage tests. It allows different callers
/// of the test suite to ensure they're executing all tests.
/// Note: this is required because: (i) vault tests cannot be run in the usual fashion (i.e., vault
/// tests rely on first running the vault docker script in `docker/vault/run.sh`); and (ii) vault
/// tests cannot currently be run in parallel, as each test uses the same vault instance.
const STORAGE_TESTS: &[fn(&mut Storage)] = &[
    test_set_reset_get,
    test_create_and_get_non_existent_version,
    test_create_get_key_pair,
    test_create_key_pair_and_perform_rotations,
    test_create_sign_rotate_sign,
    test_ensure_storage_is_available,
    test_get_non_existent,
    test_get_public_key_previous_version,
    test_get_set,
    test_get_uncreated_key_pair,
    test_hash_value,
    test_incremental_timestamp,
    test_import_key,
    test_verify_incorrect_value_types,
];

/// Storage data constants for testing purposes.
const CRYPTO_KEY: &str = "Private_Key";
const U64_KEY: &str = "U64_Key";
const CRYPTO_NAME: &str = "Test_Key_Name";

/// Executes all storage tests on a given storage backend.
pub fn execute_all_storage_tests(storage: &mut Storage) {
    storage.reset_and_clear().unwrap();
    for test in STORAGE_TESTS.iter() {
        test(storage);
        storage.reset_and_clear().unwrap();
    }
}

/// This test tries to set a key, reset the storage and then retrieve its
// value, checking that the reset is indeed performed in testing mode. It
/// should be performed first, as other tests will depend on it.
fn test_set_reset_get(storage: &mut Storage) {
    let u64_1 = 10;
    storage.set(U64_KEY, u64_1).unwrap();
    storage.reset_and_clear().unwrap();
    assert_eq!(
        storage.get::<u64>(U64_KEY).unwrap_err(),
        Error::KeyNotSet(U64_KEY.to_string())
    );
}

/// This test tries to get and set non-existent keys in storage and asserts that the correct
/// errors are returned on these operations.
fn test_get_non_existent(storage: &mut Storage) {
    assert_eq!(
        storage.get::<Ed25519PrivateKey>(CRYPTO_KEY).unwrap_err(),
        Error::KeyNotSet(CRYPTO_KEY.to_string())
    );
    assert_eq!(
        storage.get::<u64>(U64_KEY).unwrap_err(),
        Error::KeyNotSet(U64_KEY.to_string())
    );
}

/// This test tries to get previous versions of the public key after multiple rotations have
/// occurred. It also checks that the previous versions returned can be used to fetch the correct
/// private keys.
fn test_get_public_key_previous_version(storage: &mut Storage) {
    let num_rotations = 10;

    let mut public_key = storage.create_key(CRYPTO_NAME).unwrap();
    let mut private_key = storage.export_private_key(CRYPTO_NAME).unwrap();

    // Verify no previous version exists
    assert!(storage
        .get_public_key_previous_version(CRYPTO_NAME)
        .is_err());

    for _ in 0..num_rotations {
        let new_public_key = storage.rotate_key(CRYPTO_NAME).unwrap();
        let new_private_key = storage.export_private_key(CRYPTO_NAME).unwrap();

        // Verify the correct previous public key is returned
        let public_key_previous_version = storage
            .get_public_key_previous_version(CRYPTO_NAME)
            .unwrap();
        assert_eq!(public_key, public_key_previous_version);
        assert_eq!(private_key.public_key(), public_key_previous_version);

        // Verify the previous public key can be used to fetch the corresponding public key
        let private_key_previous_version = storage
            .export_private_key_for_version(CRYPTO_NAME, public_key_previous_version)
            .unwrap();
        assert_eq!(private_key, private_key_previous_version);

        public_key = new_public_key;
        private_key = new_private_key;
    }
}

/// This test stores various key/value pairs in storage, updates them, retrieves the values to
/// ensure the correct value types are returned.
fn test_get_set(storage: &mut Storage) {
    let crypto_private_1 = Ed25519PrivateKey::generate_for_testing();
    let crypto_private_2 = Ed25519PrivateKey::generate_for_testing();
    let u64_1 = 10;
    let u64_2 = 647;

    storage.set(U64_KEY, u64_1).unwrap();
    storage.set(CRYPTO_KEY, crypto_private_1.clone()).unwrap();

    assert_eq!(storage.get::<u64>(U64_KEY).unwrap().value, u64_1);
    assert_eq!(
        storage.get::<Ed25519PrivateKey>(CRYPTO_KEY).unwrap().value,
        crypto_private_1
    );

    storage.set(U64_KEY, u64_2).unwrap();
    storage.set(CRYPTO_KEY, crypto_private_2.clone()).unwrap();

    assert_eq!(storage.get::<u64>(U64_KEY).unwrap().value, u64_2);
    assert_eq!(
        storage.get::<Ed25519PrivateKey>(CRYPTO_KEY).unwrap().value,
        crypto_private_2
    );
}

/// This test ensures that a key can reasonably be imported.
fn test_import_key(storage: &mut Storage) {
    let key_name = "key";
    let imported_key_name = "imported_key";

    // Prepare key

    storage.create_key(key_name).unwrap();
    let key = storage.export_private_key(key_name).unwrap();
    let public_key = storage.get_public_key(key_name).unwrap().public_key;

    // Restore and verify key

    storage
        .import_private_key(imported_key_name, key.clone())
        .unwrap();
    let imported_key = storage.export_private_key(imported_key_name).unwrap();
    let imported_public_key = storage
        .get_public_key(imported_key_name)
        .unwrap()
        .public_key;

    assert_eq!(key, imported_key);
    assert_eq!(public_key, imported_public_key);

    // Verify valid keys

    let message = TestDiemCrypto("Hello, World".to_string());
    let message_signature = storage.sign(imported_key_name, &message).unwrap();
    message_signature
        .verify(&message, &imported_public_key)
        .unwrap();

    // Ensure rotation still works

    storage.rotate_key(imported_key_name).unwrap();
    let rotated_imported_key = storage.export_private_key(imported_key_name).unwrap();
    let rotated_imported_public_key = storage
        .get_public_key(imported_key_name)
        .unwrap()
        .public_key;

    let rotated_message_signature = storage.sign(imported_key_name, &message).unwrap();
    rotated_message_signature
        .verify(&message, &rotated_imported_public_key)
        .unwrap();

    assert_ne!(imported_key, rotated_imported_key);
    assert_ne!(imported_public_key, rotated_imported_public_key);
    assert_ne!(message_signature, rotated_message_signature);
}

/// This test stores different types of values into storage, retrieves them, and asserts
/// that the value unwrap functions return an unexpected type error on an incorrect unwrap.
fn test_verify_incorrect_value_types(storage: &mut Storage) {
    storage.set(U64_KEY, 10).unwrap();
    storage
        .set(CRYPTO_KEY, Ed25519PrivateKey::generate_for_testing())
        .unwrap();

    storage.get::<Ed25519PrivateKey>(U64_KEY).unwrap_err();
    storage.get::<u64>(CRYPTO_KEY).unwrap_err();
}

/// This test: (i) creates a new named test key pair; (ii) retrieves the public key for
/// the created key pair; (iii) compares the public keys returned by the create call and the
/// retrieval call.
fn test_create_get_key_pair(storage: &mut Storage) {
    let public_key = storage.create_key(CRYPTO_NAME).unwrap();
    let retrieved_public_key_response = storage.get_public_key(CRYPTO_NAME).unwrap();
    assert_eq!(public_key, retrieved_public_key_response.public_key);
}

/// This test tries to get the public key of a key pair that has not yet been created. As
/// such, it asserts that this attempt fails.
fn test_get_uncreated_key_pair(storage: &mut Storage) {
    let key_pair_name = "Non-existent Key";
    assert!(
        storage.get_public_key(key_pair_name).is_err(),
        "Accessing a key that has not yet been created should have failed!"
    );
}

/// Verify HashValues work correctly
fn test_hash_value(storage: &mut Storage) {
    let hash_value_key = "HashValue";
    let hash_value_value = HashValue::random();

    storage.set(hash_value_key, hash_value_value).unwrap();
    let out_value = storage.get(hash_value_key).unwrap().value;
    assert_eq!(hash_value_value, out_value);
}

/// This test verifies the storage engine is up and running.
fn test_ensure_storage_is_available(storage: &mut Storage) {
    storage.available().unwrap();
}

/// This test creates a new named key pair and attempts to get a non-existent version of the public
/// and private keys. As such, these calls should fail.
fn test_create_and_get_non_existent_version(storage: &mut Storage) {
    // Create new named key pair
    let _ = storage.create_key(CRYPTO_NAME).unwrap();

    // Get a non-existent version of the new key pair and verify failure
    let non_existent_public_key = Ed25519PrivateKey::generate_for_testing().public_key();
    assert!(
        storage.export_private_key_for_version(CRYPTO_NAME, non_existent_public_key).is_err(),
        "We have tried to retrieve a non-existent private key version -- the call should have failed!",
    );
}

/// This test creates a new key pair and performs multiple key rotations, ensuring that
/// storage updates key pair versions appropriately.
fn test_create_key_pair_and_perform_rotations(storage: &mut Storage) {
    let num_rotations = 10;

    let mut public_key = storage.create_key(CRYPTO_NAME).unwrap();
    let mut private_key = storage.export_private_key(CRYPTO_NAME).unwrap();

    for _ in 0..num_rotations {
        let new_public_key = storage.rotate_key(CRYPTO_NAME).unwrap();
        let new_private_key = storage.export_private_key(CRYPTO_NAME).unwrap();

        let exported_key = storage
            .export_private_key_for_version(CRYPTO_NAME, public_key)
            .unwrap();
        assert_eq!(exported_key, private_key);
        assert_eq!(new_public_key, new_private_key.public_key());

        public_key = new_public_key;
        private_key = new_private_key;
    }
}

/// This test creates a new key pair, signs a message using the key pair, rotates the key pair,
/// re-signs the message using the previous key pair version, and asserts the same signature is
/// produced.
fn test_create_sign_rotate_sign(storage: &mut Storage) {
    // Generate new key pair
    let public_key = storage.create_key(CRYPTO_NAME).unwrap();

    // Create then sign message and verify correct signature
    let message = TestDiemCrypto("Hello, World".to_string());
    let message_signature = storage.sign(CRYPTO_NAME, &message).unwrap();
    assert!(message_signature.verify(&message, &public_key).is_ok());

    // Rotate the key pair and sign the message again using the previous key pair version
    let _ = storage.rotate_key(CRYPTO_NAME).unwrap();
    let message_signature_previous = storage
        .sign_using_version(CRYPTO_NAME, public_key, &message)
        .unwrap();

    // Verify signatures match and are valid
    assert_eq!(message_signature, message_signature_previous);
}

/// This test verifies that timestamps increase with successive writes
fn test_incremental_timestamp(storage: &mut Storage) {
    let key = "timestamp_u64";
    let value0 = 442;
    let value1 = 450;

    storage.set(key, value0).unwrap();
    let first = storage.get::<u64>(key).unwrap();
    std::thread::sleep(std::time::Duration::from_secs(1));
    storage.set(key, value1).unwrap();
    let second = storage.get::<u64>(key).unwrap();

    assert_ne!(first.value, second.value);
    assert!(first.last_update < second.last_update);
}
