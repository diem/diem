// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{Error, Policy, Storage, Value};
use libra_crypto::{ed25519::Ed25519PrivateKey, Uniform};
use rand::{rngs::StdRng, SeedableRng};

const KEY_KEY: &str = "key";
const U64_KEY: &str = "u64";

pub fn run_test_suite(mut storage: Box<dyn Storage>, name: &str) {
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
}
