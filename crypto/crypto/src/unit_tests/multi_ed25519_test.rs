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
use crate::CryptoMaterialError::{ValidationError, WrongLengthError};

// Helper function to generate N key pairs.
fn generate_key_pairs(n: usize) -> Vec<(Ed25519PrivateKey, Ed25519PublicKey)> {
    let mut rng = StdRng::from_seed(TEST_SEED);
    (0..n).map(|_| compat::generate_keypair(&mut rng)).collect()
}

// Reused assertions in our tests.
fn test_successful_public_key_serialization(original_keys: &[Ed25519PublicKey], threshold: u8) {
    let n = original_keys.len();
    let public_key: MultiEd25519PublicKey =
        MultiEd25519PublicKey::new(original_keys.to_vec(), threshold).unwrap();
    assert_eq!(public_key.threshold(), &threshold);
    assert_eq!(public_key.public_keys().len(), n);
    assert_eq!(public_key.public_keys(), &original_keys.to_vec());
    let serialized = public_key.to_bytes();
    assert_eq!(serialized.len(), n * ED25519_PUBLIC_KEY_LENGTH + 1);
    let reserialized = MultiEd25519PublicKey::try_from(&serialized[..]);
    assert!(reserialized.is_ok());
    assert_eq!(public_key, reserialized.unwrap());
}

fn test_failed_public_key_serialization(
    result: std::result::Result<MultiEd25519PublicKey, CryptoMaterialError>,
    expected_error: CryptoMaterialError,
) {
    assert!(result.is_err());
    assert_eq!(result.err().unwrap(), expected_error);
}

// Test multi-sig Ed25519 public key serialization.
#[allow(clippy::redundant_clone)]
#[test]
fn test_multi_ed25519_public_key_serialization() {
    let pub_keys_1: Vec<Ed25519PublicKey> =
        generate_key_pairs(1).iter().map(|x| x.1.clone()).collect();
    let pub_keys_10: Vec<Ed25519PublicKey> =
        generate_key_pairs(10).iter().map(|x| x.1.clone()).collect();
    let pub_keys_32: Vec<Ed25519PublicKey> =
        generate_key_pairs(32).iter().map(|x| x.1.clone()).collect();
    let pub_keys_33: Vec<Ed25519PublicKey> =
        generate_key_pairs(33).iter().map(|x| x.1.clone()).collect();

    // Test 1-of-1
    test_successful_public_key_serialization(&pub_keys_1, 1);
    // Test 1-of-10
    test_successful_public_key_serialization(&pub_keys_10, 1);
    // Test 7-of-10
    test_successful_public_key_serialization(&pub_keys_10, 7);
    // Test 10-of-10
    test_successful_public_key_serialization(&pub_keys_10, 10);
    // Test 2-of-32
    test_successful_public_key_serialization(&pub_keys_32, 2);
    // Test 32-of-32
    test_successful_public_key_serialization(&pub_keys_32, 32);

    // Test 11-of-10 (should fail).
    let multi_key_11of10 = MultiEd25519PublicKey::new(pub_keys_10.clone(), 11);
    test_failed_public_key_serialization(multi_key_11of10, ValidationError);

    // Test 0-of-10 (should fail).
    let multi_key_0of10 = MultiEd25519PublicKey::new(pub_keys_10.clone(), 0);
    test_failed_public_key_serialization(multi_key_0of10, ValidationError);

    // Test 1-of-33 (should fail).
    let multi_key_1of33 = MultiEd25519PublicKey::new(pub_keys_33.clone(), 1);
    test_failed_public_key_serialization(multi_key_1of33, WrongLengthError);

    // Test try_from empty bytes (should fail).
    let multi_key_empty_bytes = MultiEd25519PublicKey::try_from(&[] as &[u8]);
    test_failed_public_key_serialization(multi_key_empty_bytes, WrongLengthError);

    // Test try_from 1 byte (should fail).
    let multi_key_1_byte = MultiEd25519PublicKey::try_from(&[0u8][..]);
    test_failed_public_key_serialization(multi_key_1_byte, WrongLengthError);

    // Test try_from 31 bytes (should fail).
    let multi_key_31_bytes =
        MultiEd25519PublicKey::try_from(&[0u8; ED25519_PUBLIC_KEY_LENGTH - 1][..]);
    test_failed_public_key_serialization(multi_key_31_bytes, WrongLengthError);

    // Test try_from 32 bytes (should fail) because we always need ED25519_PUBLIC_KEY_LENGTH * N + 1
    // bytes (thus 32N + 1).
    let multi_key_32_bytes = MultiEd25519PublicKey::try_from(&[0u8; ED25519_PUBLIC_KEY_LENGTH][..]);
    test_failed_public_key_serialization(multi_key_32_bytes, WrongLengthError);

    // Test try_from 34 bytes (should fail).
    let multi_key_34_bytes =
        MultiEd25519PublicKey::try_from(&[0u8; ED25519_PUBLIC_KEY_LENGTH + 2][..]);
    test_failed_public_key_serialization(multi_key_34_bytes, WrongLengthError);

    // Test try_from 33 all zero bytes (size is fine, but it should fail due to
    // validation issues).
    let multi_key_33_zero_bytes =
        MultiEd25519PublicKey::try_from(&[0u8; ED25519_PUBLIC_KEY_LENGTH + 1][..]);
    test_failed_public_key_serialization(multi_key_33_zero_bytes, ValidationError);

    let keypairs_10 = generate_key_pairs(10);
    let priv_keys_10: Vec<Ed25519PrivateKey> = keypairs_10.iter().map(|x| x.0.clone()).collect();
    let pub_keys_10: Vec<Ed25519PublicKey> = keypairs_10.iter().map(|x| x.1.clone()).collect();

    let multi_private_key_7of10 = MultiEd25519PrivateKey::new(priv_keys_10.clone(), 7).unwrap();
    let multi_public_key_7of10 = MultiEd25519PublicKey::new(pub_keys_10.clone(), 7).unwrap();

    // Check that MultiEd25519PublicKey::from MultiEd25519PrivateKey works as expected.
    let multi_public_key_7of10_from_multi_private_key =
        MultiEd25519PublicKey::from(&multi_private_key_7of10);
    assert_eq!(
        multi_public_key_7of10_from_multi_private_key,
        multi_public_key_7of10
    );

    // Check that MultiEd25519PublicKey::from Ed25519PublicKey works as expected.
    let multi_public_key_from_ed25519 = MultiEd25519PublicKey::from(
        &multi_public_key_7of10_from_multi_private_key.public_keys()[0],
    );
    assert_eq!(multi_public_key_from_ed25519.public_keys().len(), 1);
    assert_eq!(
        &multi_public_key_from_ed25519.public_keys()[0],
        &multi_public_key_7of10_from_multi_private_key.public_keys()[0]
    );
    assert_eq!(multi_public_key_from_ed25519.threshold(), &1u8);
}

// Test against known small subgroup public key.
#[test]
fn test_publickey_smallorder() {
    // A small group point with threshold 1 (last byte).
    // See EIGHT_TORSION in ed25519_test.rs for more about small group points.
    let torsion_point_with_threshold_1: [u8; 33] = [
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 1,
    ];

    let torsion_key = MultiEd25519PublicKey::try_from(&torsion_point_with_threshold_1[..]);
    assert!(torsion_key.is_err());
    assert_eq!(
        torsion_key.err().unwrap(),
        CryptoMaterialError::SmallSubgroupError
    );
}

// Test multi-sig Ed25519 signature verification.
#[allow(clippy::redundant_clone)]
#[test]
fn test_multi_ed25519_signature_verification() {
    let keypairs_10 = generate_key_pairs(10);
    let priv_keys_10: Vec<Ed25519PrivateKey> = keypairs_10.iter().map(|x| x.0.clone()).collect();
    let pub_keys_10: Vec<Ed25519PublicKey> = keypairs_10.iter().map(|x| x.1.clone()).collect();

    let multi_private_key_7of10 = MultiEd25519PrivateKey::new(priv_keys_10.clone(), 7).unwrap();
    let multi_public_key_7of10 = MultiEd25519PublicKey::from(&multi_private_key_7of10);

    let message = b"Test Message";
    let message_hash = HashValue::from_sha3_256(message);

    // Verifying a 7-of-10 signature against a public key with the same threshold should pass.
    let multi_signature_7of10 = multi_private_key_7of10.sign_message(&message_hash);
    assert!(multi_signature_7of10
        .verify(&message_hash, &multi_public_key_7of10)
        .is_ok());

    // Verifying a 7-of-10 signature against a public key with bigger threshold (i.e., 8) should fail.
    let multi_public_key_8of10 = MultiEd25519PublicKey::new(pub_keys_10.clone(), 8).unwrap();
    assert!(multi_signature_7of10
        .verify(&message_hash, &multi_public_key_8of10)
        .is_err());

    // Verifying a 7-of-10 signature against a public key with smaller threshold (i.e., 6) should pass.
    let multi_public_key_6of10 = MultiEd25519PublicKey::new(pub_keys_10.clone(), 6).unwrap();
    assert!(multi_signature_7of10
        .verify(&message_hash, &multi_public_key_6of10)
        .is_ok());

    // Verifying a 7-of-10 signature against a reordered MultiEd25519PublicKey should fail.
    // To deterministically simulate reshuffling, we use a reversed vector of 10 keys.
    // Note that because 10 is an even number, all of they keys will change position.
    let mut pub_keys_10_reversed = pub_keys_10.clone();
    pub_keys_10_reversed.reverse();
    let multi_public_key_7of10_reversed =
        MultiEd25519PublicKey::new(pub_keys_10_reversed, 7).unwrap();
    assert!(multi_signature_7of10
        .verify(&message_hash, &multi_public_key_7of10_reversed)
        .is_err());
}
