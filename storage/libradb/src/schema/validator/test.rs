// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crypto::signing::generate_keypair;
use itertools::Itertools;
use rand::{thread_rng, Rng};
use schemadb::schema::assert_encode_decode;
use types::transaction::Version;

fn row_with_arbitrary_validator(version: Version) -> (Key, Value) {
    let (_private_key, public_key) = generate_keypair();
    (
        Key {
            version,
            public_key,
        },
        Value,
    )
}

#[test]
fn test_encode_decode() {
    let (k, v) = row_with_arbitrary_validator(1);
    assert_encode_decode::<ValidatorSchema>(&k, &v);
}

#[test]
fn test_order() {
    let mut versions: Vec<u64> = (0..1024).collect();
    thread_rng().shuffle(&mut versions);

    let encoded_sorted: Vec<Vec<u8>> = versions
        .into_iter()
        .map(|v| row_with_arbitrary_validator(v).0.encode_key().unwrap())
        .sorted();

    let decoded_versions: Vec<Version> = encoded_sorted
        .iter()
        .map(|k| Key::decode_key(k).unwrap().version)
        .collect();

    let ordered_versions: Vec<Version> = (0..1024).collect();

    assert_eq!(decoded_versions, ordered_versions)
}
