// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    traits::*,
};

use core::convert::TryFrom;
use rand::{rngs::StdRng, SeedableRng};

use crate::ed25519::{compat, ED25519_PUBLIC_KEY_LENGTH};
use crate::hash::HashValue;
use crate::multi_ed25519::{MultiEd25519PrivateKey, MultiEd25519PublicKey};
use crate::test_utils::TEST_SEED;

// Test multi-sig Ed25519 serialization.
#[test]
fn test_multi_ed25519_public_key_serialization() {
    let mut rng = StdRng::from_seed(TEST_SEED);

    let mut pub_keys: Vec<Ed25519PublicKey> = vec![];
    for _ in 0..10 {
        pub_keys.push(compat::generate_keypair(&mut rng).1);
    }

    #[allow(clippy::redundant_clone)]
    let multi_key_1of10 = MultiEd25519PublicKey::new(pub_keys.clone(), 1);
    let serialized_1of10 = MultiEd25519PublicKey::to_bytes(&multi_key_1of10.unwrap());
    assert_eq!(serialized_1of10.len(), 10 * ED25519_PUBLIC_KEY_LENGTH + 1);

    #[allow(clippy::redundant_clone)]
    let multi_key_10of10 = MultiEd25519PublicKey::new(pub_keys.clone(), 10);
    let mut serialized_10of10 = MultiEd25519PublicKey::to_bytes(&multi_key_10of10.unwrap());
    assert_eq!(serialized_10of10.len(), 10 * ED25519_PUBLIC_KEY_LENGTH);

    let multi_key_1of10_new = MultiEd25519PublicKey::try_from(serialized_1of10.as_slice());
    let multi_key_10of10_new = MultiEd25519PublicKey::try_from(serialized_10of10.as_slice());

    assert!(&multi_key_1of10_new.is_ok());
    assert!(&multi_key_10of10_new.is_ok());

    let multi_key_1of10_new = multi_key_1of10_new.unwrap();
    let multi_key_10of10_new = multi_key_10of10_new.unwrap();

    for (index, pk) in pub_keys.iter().enumerate() {
        assert_eq!(pk, &multi_key_1of10_new.0[index]);
        assert_eq!(pk, &multi_key_10of10_new.0[index]);
    }
    assert_eq!(multi_key_1of10_new.1, 1);
    assert_eq!(multi_key_10of10_new.1, 10);

    // Add threshold at the end of 10of10.
    serialized_10of10.push(10u8);
    let multi_key_10of10_new = MultiEd25519PublicKey::try_from(serialized_10of10.as_slice());
    assert!(&multi_key_10of10_new.is_err());
    assert_eq!(
        multi_key_10of10_new.err().unwrap(),
        CryptoMaterialError::WrongLengthError
    );

    serialized_10of10.pop();
    serialized_10of10.push(11u8);
    let multi_key_10of10_new = MultiEd25519PublicKey::try_from(serialized_10of10.as_slice());
    assert!(&multi_key_10of10_new.is_err());
    assert_eq!(
        multi_key_10of10_new.err().unwrap(),
        CryptoMaterialError::WrongLengthError
    );

    // Cannot add zero threshold.
    #[allow(clippy::redundant_clone)]
    let multi_key_0of10 = MultiEd25519PublicKey::new(pub_keys.clone(), 0);
    assert!(multi_key_0of10.is_err());
    assert_eq!(
        multi_key_0of10.err().unwrap(),
        CryptoMaterialError::WrongLengthError
    );

    // Cannot add bigger than N threshold.
    #[allow(clippy::redundant_clone)]
    let multi_key_11of10 = MultiEd25519PublicKey::new(pub_keys.clone(), 11);
    assert!(multi_key_11of10.is_err());
    assert_eq!(
        multi_key_11of10.err().unwrap(),
        CryptoMaterialError::WrongLengthError
    );
}

// Test multi-sig Ed25519 signature verification.
#[test]
fn test_multi_ed25519_signature_verification() {
    let mut rng = StdRng::from_seed(TEST_SEED);

    let mut private_keys: Vec<Ed25519PrivateKey> = vec![];
    let mut public_keys: Vec<Ed25519PublicKey> = vec![];
    for _ in 0..10 {
        let (private_key, public_key) = compat::generate_keypair(&mut rng);
        private_keys.push(private_key);
        public_keys.push(public_key);
    }
    #[allow(clippy::redundant_clone)]
    let multi_private_key_7of10 = MultiEd25519PrivateKey::new(private_keys.clone(), 7).unwrap();
    let multi_public_key_7of10 = MultiEd25519PublicKey::from(&multi_private_key_7of10);

    // Check if MultiEd25519PublicKey::from works as expected.
    #[allow(clippy::redundant_clone)]
    let multi_public_key_7of10_construct =
        MultiEd25519PublicKey::new(public_keys.clone(), 7).unwrap();
    assert_eq!(multi_public_key_7of10, multi_public_key_7of10_construct);
    let message = b"Hello";
    let message_hash = HashValue::from_sha3_256(message);

    let multi_signature_7of10 = multi_private_key_7of10.sign_message(&message_hash);
    assert!(multi_signature_7of10
        .verify(&message_hash, &multi_public_key_7of10)
        .is_ok());

    // A Public key with bigger threshold should fail.
    #[allow(clippy::redundant_clone)]
    let multi_public_key_8of10 = MultiEd25519PublicKey::new(public_keys.clone(), 8).unwrap();
    assert!(multi_signature_7of10
        .verify(&message_hash, &multi_public_key_8of10)
        .is_err());

    // A Public key with smaller threshold should pass.
    #[allow(clippy::redundant_clone)]
    let multi_public_key_6of10 = MultiEd25519PublicKey::new(public_keys.clone(), 6).unwrap();
    assert!(multi_signature_7of10
        .verify(&message_hash, &multi_public_key_6of10)
        .is_ok());
}
