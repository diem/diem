// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{Error, Policy, Storage, Value};
use libra_crypto::{ed25519::Ed25519PrivateKey, Uniform};
use rand::{rngs::StdRng, SeedableRng};

/// This suite contains tests for secure storage backends. We test the correct functionality
/// of both key/value and cryptographic operations for storage implementations. All storage backend
/// implementations should be tested using the tests in this suite.

/// This holds the canonical list of secure storage tests. It allows different callers
/// of the test suite to ensure they're executing all tests.
/// Note: this is required because: (i) vault tests cannot be run in the usual fashion (i.e., vault
/// tests rely on first running the vault docker script in `docker/vault/run.sh`); and (ii) vault
/// tests cannot currently be run in parallel, as each test uses the same vault instance.
const STORAGE_TESTS: &[fn(&mut dyn Storage)] = &[
    test_ensure_storage_is_available,
    test_create_get_set_unwrap,
    test_create_key_value_twice,
    test_get_set_non_existent,
    test_verify_incorrect_value_types_unwrap,
    test_create_get_and_test_key_pair,
    test_create_key_pair_twice,
    test_get_uncreated_key_pair,
];

/// Storage data constants for testing purposes.
const CRYPTO_KEY: &str = "Private Key";
const U64_KEY: &str = "U64 Key";
const CRYPTO_KEYPAIR_NAME: &str = "Test Key Name";

/// Executes all storage tests on a given storage backend.
pub fn execute_all_storage_tests(storage: &mut dyn Storage) {
    for test in STORAGE_TESTS.iter() {
        test(storage);
        storage
            .reset_and_clear()
            .expect("Failed to reset storage engine between tests!");
    }
}

/// This test tries to get and set non-existent keys in storage and asserts that the correct
/// errors are returned on these operations.
fn test_get_set_non_existent(storage: &mut dyn Storage) {
    let crypto_value = Value::Ed25519PrivateKey(create_ed25519_key_for_testing());
    let u64_value = Value::U64(10);

    assert_eq!(
        storage.get(CRYPTO_KEY).unwrap_err(),
        Error::KeyNotSet(CRYPTO_KEY.to_string())
    );
    assert_eq!(
        storage.get(U64_KEY).unwrap_err(),
        Error::KeyNotSet(U64_KEY.to_string())
    );
    assert_eq!(
        storage.set(CRYPTO_KEY, crypto_value).unwrap_err(),
        Error::KeyNotSet(CRYPTO_KEY.to_string())
    );
    assert_eq!(
        storage.set(U64_KEY, u64_value).unwrap_err(),
        Error::KeyNotSet(U64_KEY.to_string())
    );
}

/// This test stores various key/value pairs in storage, updates them, retrieves the values and
/// unwraps them to ensure the correct value types are returned.
fn test_create_get_set_unwrap(storage: &mut dyn Storage) {
    let crypto_private_1 = create_ed25519_key_for_testing();
    let crypto_private_2 = create_ed25519_key_for_testing();
    let u64_1 = 10;
    let u64_2 = 647;
    let policy = Policy::public();

    storage
        .create_if_not_exists(U64_KEY, Value::U64(u64_1), &policy)
        .unwrap();
    storage
        .create(
            CRYPTO_KEY,
            Value::Ed25519PrivateKey(crypto_private_1.clone()),
            &policy,
        )
        .unwrap();

    assert_eq!(storage.get(U64_KEY).unwrap().u64().unwrap(), u64_1);
    assert_eq!(
        &storage
            .get(CRYPTO_KEY)
            .unwrap()
            .ed25519_private_key()
            .unwrap(),
        &crypto_private_1
    );

    storage.set(U64_KEY, Value::U64(u64_2)).unwrap();
    storage
        .set(
            CRYPTO_KEY,
            Value::Ed25519PrivateKey(crypto_private_2.clone()),
        )
        .unwrap();

    assert_eq!(storage.get(U64_KEY).unwrap().u64().unwrap(), u64_2);
    assert_eq!(
        &storage
            .get(CRYPTO_KEY)
            .unwrap()
            .ed25519_private_key()
            .unwrap(),
        &crypto_private_2
    );
}

/// This test stores different types of values into storage, retrieves them, and asserts
/// that the value unwrap functions return an unexpected type error on an incorrect unwrap.
fn test_verify_incorrect_value_types_unwrap(storage: &mut dyn Storage) {
    let crypto_value = Value::Ed25519PrivateKey(create_ed25519_key_for_testing());
    let u64_value = Value::U64(10);
    let policy = Policy::public();

    storage
        .create_if_not_exists(U64_KEY, u64_value, &policy)
        .unwrap();
    storage
        .create_if_not_exists(CRYPTO_KEY, crypto_value, &policy)
        .unwrap();

    assert_eq!(
        storage
            .get(U64_KEY)
            .unwrap()
            .ed25519_private_key()
            .unwrap_err(),
        Error::UnexpectedValueType
    );
    assert_eq!(
        storage.get(CRYPTO_KEY).unwrap().u64().unwrap_err(),
        Error::UnexpectedValueType
    );
}

/// This test attempts to create a key/value pair twice, asserting that the second
/// creation attempt fails and returns a key exists error.
fn test_create_key_value_twice(storage: &mut dyn Storage) {
    let u64_value = 10;
    let policy = Policy::public();

    assert_eq!(
        storage.create(U64_KEY, Value::U64(u64_value), &policy),
        Ok(())
    );
    match storage.create(U64_KEY, Value::U64(u64_value), &policy) {
        Err(Error::KeyAlreadyExists(_)) => (/* Expected error code */),
        _ => panic!("The second call to create() should have failed!"),
    }
}

/// This test: (i) creates a new named test key pair; (ii) retrieves the public key for
/// the created key pair; (iii) compares the public keys returned by the create call and the
/// retrieval call.
fn test_create_get_and_test_key_pair(storage: &mut dyn Storage) {
    let public_key = storage
        .generate_new_ed25519_key_pair(CRYPTO_KEYPAIR_NAME, &Policy::public())
        .expect("Failed to create a test Ed25519 key pair!");
    let retrieved_public_key = storage
        .get_public_key_for(CRYPTO_KEYPAIR_NAME)
        .expect("Failed to fetch the test key pair!");
    assert_eq!(public_key, retrieved_public_key);
}

/// This test attempts to create two named key pairs using the same name, and asserts
/// that the second creation call (i.e., the duplicate), fails.
fn test_create_key_pair_twice(storage: &mut dyn Storage) {
    let policy = Policy::public();
    let _ = storage
        .generate_new_ed25519_key_pair(CRYPTO_KEYPAIR_NAME, &policy)
        .expect("Failed to create a test Ed25519 key pair!");
    assert!(
        storage
            .generate_new_ed25519_key_pair(CRYPTO_KEYPAIR_NAME, &policy)
            .is_err(),
        "The second call to generate_ed25519_key_pair() should have failed!"
    );
}

/// This test tries to get the public key of a key pair that has not yet been created. As
/// such, it asserts that this attempt fails.
fn test_get_uncreated_key_pair(storage: &mut dyn Storage) {
    let key_pair_name = "Non-existent Key";
    assert!(
        storage.get_public_key_for(key_pair_name).is_err(),
        "Accessing a key that has not yet been created should have failed!"
    );
}

/// This test verifies the storage engine is up and running.
fn test_ensure_storage_is_available(storage: &mut dyn Storage) {
    assert!(
        storage.available(),
        eprintln!("Backend storage is not available")
    );
}

fn create_ed25519_key_for_testing() -> Ed25519PrivateKey {
    let mut rng = StdRng::from_seed([13u8; 32]);
    Ed25519PrivateKey::generate_for_testing(&mut rng)
}
