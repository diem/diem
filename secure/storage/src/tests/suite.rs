// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{Error, Policy, Storage, Value};
use libra_crypto::{ed25519::Ed25519PrivateKey, Uniform};
use rand::{rngs::StdRng, SeedableRng};

const KEY_KEY: &str = "key";
const U64_KEY: &str = "u64";

/// This helper function checks various features of the secure storage behaviour relating to K/V
/// support, e.g., ensuring errors are returned when attempting to retrieve missing keys, and
/// verifying sets and gets are implemented correctly.
pub fn run_test_suite(storage: &mut dyn Storage, name: &str) {
    assert!(
        storage.available(),
        eprintln!("Backend storage, {}, is not available", name)
    );

    let public = Policy::public();
    let u64_value_0 = 5;
    let u64_value_1 = 2322;

    let mut rng = StdRng::from_seed([13u8; 32]);
    let key_value_0 = Ed25519PrivateKey::generate_for_testing(&mut rng);
    let key_value_1 = Ed25519PrivateKey::generate_for_testing(&mut rng);

    assert_eq!(
        storage.get(KEY_KEY).unwrap_err(),
        Error::KeyNotSet(KEY_KEY.to_string())
    );
    assert_eq!(
        storage.get(U64_KEY).unwrap_err(),
        Error::KeyNotSet(U64_KEY.to_string())
    );

    assert_eq!(
        storage
            .set(KEY_KEY, Value::Ed25519PrivateKey(key_value_0.clone()))
            .unwrap_err(),
        Error::KeyNotSet(KEY_KEY.to_string())
    );
    assert_eq!(
        storage.set(U64_KEY, Value::U64(u64_value_0)).unwrap_err(),
        Error::KeyNotSet(U64_KEY.to_string())
    );

    storage
        .create_if_not_exists(U64_KEY, Value::U64(u64_value_1), &public)
        .unwrap();
    storage
        .create(
            KEY_KEY,
            Value::Ed25519PrivateKey(key_value_1.clone()),
            &public,
        )
        .unwrap();

    assert_eq!(storage.get(U64_KEY).unwrap().u64().unwrap(), u64_value_1);
    assert_eq!(
        &storage.get(KEY_KEY).unwrap().ed25519_private_key().unwrap(),
        &key_value_1
    );

    storage.set(U64_KEY, Value::U64(u64_value_0)).unwrap();
    storage
        .set(KEY_KEY, Value::Ed25519PrivateKey(key_value_0.clone()))
        .unwrap();

    assert_eq!(&storage.get(U64_KEY).unwrap().u64().unwrap(), &u64_value_0);
    assert_eq!(
        &storage.get(KEY_KEY).unwrap().ed25519_private_key().unwrap(),
        &key_value_0
    );

    // Should not affect the above computation
    storage
        .create_if_not_exists(U64_KEY, Value::U64(u64_value_1), &public)
        .unwrap();
    storage
        .create_if_not_exists(KEY_KEY, Value::Ed25519PrivateKey(key_value_1), &public)
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
        storage.get(KEY_KEY).unwrap().u64().unwrap_err(),
        Error::UnexpectedValueType
    );

    // Attempt to perform a u64_key creation twice (i.e., for a key that already exists!)
    assert!(storage
        .create(U64_KEY, Value::U64(u64_value_1), &public)
        .is_err());
}

/// This helper function: (i) creates a new named test key pair; (ii) retrieves the public key for
/// the created key pair; (iii) compares the public keys returned by the create call and the
/// retrieval call.
pub fn create_get_and_test_key_pair(storage: &mut dyn Storage) {
    let key_pair_name = "Test Key";
    let public_key = storage
        .generate_new_ed25519_key_pair(key_pair_name, &Policy::public())
        .expect("Failed to create a test Ed25519 key pair!");
    let retrieved_public_key = storage
        .get_public_key_for(key_pair_name)
        .expect("Failed to fetch the test key pair!");
    assert_eq!(public_key, retrieved_public_key);
}

/// This helper function attempts to create two named key pairs using the same name, and asserts
/// that the second creation call (i.e., the duplicate), fails.
pub fn create_key_pair_twice(storage: &mut dyn Storage) {
    let key_pair_name = "Test Key";
    let policy = Policy::public();
    let _ = storage
        .generate_new_ed25519_key_pair(key_pair_name, &policy)
        .expect("Failed to create a test Ed25519 key pair!");
    assert!(
        storage
            .generate_new_ed25519_key_pair(key_pair_name, &policy)
            .is_err(),
        "The second call to generate_ed25519_key_pair() should have failed!"
    );
}

/// This helper function tries to get the public key of a key pair that has not yet been created. As
/// such, it asserts that this attempt fails.
pub fn get_uncreated_key_pair(storage: &mut dyn Storage) {
    assert!(
        storage.get_public_key_for("Non-existent key").is_err(),
        "Accessing a key that has not yet been created should have failed!"
    );
}
