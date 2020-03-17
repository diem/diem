// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    ed25519::{compat, Ed25519PrivateKey, Ed25519PublicKey, ED25519_PUBLIC_KEY_LENGTH},
    hash::HashValue,
    multi_ed25519::{MultiEd25519PrivateKey, MultiEd25519PublicKey},
    test_utils::TEST_SEED,
    traits::*,
    CryptoMaterialError::{ValidationError, WrongLengthError},
};

use crate::multi_ed25519::MultiEd25519Signature;
use core::convert::TryFrom;
use once_cell::sync::Lazy;
use rand::{rngs::StdRng, SeedableRng};

static MESSAGE_HASH: Lazy<HashValue> = Lazy::new(|| HashValue::from_sha3_256(b"Test Message"));

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

fn test_successful_signature_serialization(private_keys: &[Ed25519PrivateKey], threshold: u8) {
    let multi_private_key = MultiEd25519PrivateKey::new(private_keys.to_vec(), threshold).unwrap();
    let multi_public_key = MultiEd25519PublicKey::from(&multi_private_key);
    let multi_signature = multi_private_key.sign_message(&MESSAGE_HASH);

    // Serialize then Deserialize.
    let multi_signature_serialized =
        MultiEd25519Signature::try_from(&multi_signature.to_bytes()[..]);
    assert!(multi_signature_serialized.is_ok());
    let multi_signature_serialized_unwrapped = multi_signature_serialized.unwrap();
    assert_eq!(multi_signature, multi_signature_serialized_unwrapped);
    assert!(multi_signature_serialized_unwrapped
        .verify(&MESSAGE_HASH, &multi_public_key)
        .is_ok());
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
    let multi_key_1of33 = MultiEd25519PublicKey::new(pub_keys_33, 1);
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

    let multi_private_key_7of10 = MultiEd25519PrivateKey::new(priv_keys_10, 7).unwrap();
    let multi_public_key_7of10 = MultiEd25519PublicKey::new(pub_keys_10, 7).unwrap();

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

// Test multi-sig Ed25519 signature serialization.
#[allow(clippy::redundant_clone)]
#[test]
fn test_multi_ed25519_signature_serialization() {
    let keypairs_3 = generate_key_pairs(3);
    let priv_keys_3: Vec<Ed25519PrivateKey> = keypairs_3.iter().map(|x| x.0.clone()).collect();

    // Test 1 of 3
    test_successful_signature_serialization(&priv_keys_3, 1);
    // Test 2 of 3
    test_successful_signature_serialization(&priv_keys_3, 2);
    // Test 3 of 3
    test_successful_signature_serialization(&priv_keys_3, 3);

    let keypairs_32 = generate_key_pairs(32);
    let priv_keys_32: Vec<Ed25519PrivateKey> = keypairs_32.iter().map(|x| x.0.clone()).collect();
    // Test 1 of 32
    test_successful_signature_serialization(&priv_keys_32, 1);
    // Test 32 of 32
    test_successful_signature_serialization(&priv_keys_32, 32);

    // Construct from single Ed25519Signature.
    let single_signature = priv_keys_3[0].sign_message(&MESSAGE_HASH);
    let multi_signature = MultiEd25519Signature::from(&single_signature);
    assert_eq!(1, multi_signature.signatures().len());
    assert_eq!(multi_signature.signatures()[0], single_signature);
    let multi_priv_key_1of3 = MultiEd25519PrivateKey::new(priv_keys_3.to_vec(), 1).unwrap();
    let multi_pub_key_1of3 = MultiEd25519PublicKey::from(&multi_priv_key_1of3);
    assert!(multi_signature
        .verify(&MESSAGE_HASH, &multi_pub_key_1of3)
        .is_ok());

    // We can construct signatures from 32 single signatures.
    let sigs_32 = vec![single_signature.clone(); 32];
    let indices: Vec<u8> = (0..32).collect();
    let sig32_tuple = sigs_32.into_iter().zip(indices.into_iter()).collect();

    let multi_sig32 = MultiEd25519Signature::new(sig32_tuple);
    assert!(multi_sig32.is_ok());
    let pub_key_32 = vec![priv_keys_3[0].public_key(); 32];
    let multi_pub_key_32 = MultiEd25519PublicKey::new(pub_key_32, 32).unwrap();
    assert!(multi_sig32
        .unwrap()
        .verify(&MESSAGE_HASH, &multi_pub_key_32)
        .is_ok());

    // Fail to construct a MultiEd25519Signature object from 33 or more single signatures.
    let sigs_33 = vec![single_signature.clone(); 33];
    let indices: Vec<u8> = (0..33).collect();
    let sig33_tuple = sigs_33.into_iter().zip(indices.into_iter()).collect();

    let multi_sig33 = MultiEd25519Signature::new(sig33_tuple);
    assert!(multi_sig33.is_err());
    assert_eq!(
        multi_sig33.err().unwrap(),
        CryptoMaterialError::ValidationError
    );

    // Fail to construct a MultiEd25519Signature object if there are duplicated indexes.
    let sigs_3 = vec![single_signature; 3];
    let indices_with_duplicate = vec![0u8, 1u8, 1u8];
    let sig3_tuple = sigs_3
        .clone()
        .into_iter()
        .zip(indices_with_duplicate.into_iter())
        .collect();

    let multi_sig3 = MultiEd25519Signature::new(sig3_tuple);
    assert!(multi_sig3.is_err());
    assert_eq!(
        multi_sig3.err().unwrap(),
        CryptoMaterialError::BitVecError("Duplicate signature index".to_string())
    );

    // Fail to construct a MultiEd25519Signature object if an index is out of range.
    let indices_with_out_of_range = vec![0u8, 33u8, 1u8];
    let sig3_tuple = sigs_3
        .into_iter()
        .zip(indices_with_out_of_range.into_iter())
        .collect();

    let multi_sig3 = MultiEd25519Signature::new(sig3_tuple);
    assert!(multi_sig3.is_err());
    assert_eq!(
        multi_sig3.err().unwrap(),
        CryptoMaterialError::BitVecError("Signature index is out of range".to_string())
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

    // Verifying a 7-of-10 signature against a public key with the same threshold should pass.
    let multi_signature_7of10 = multi_private_key_7of10.sign_message(&MESSAGE_HASH);
    assert!(multi_signature_7of10
        .verify(&MESSAGE_HASH, &multi_public_key_7of10)
        .is_ok());

    // Verifying a 7-of-10 signature against a public key with bigger threshold (i.e., 8) should fail.
    let multi_public_key_8of10 = MultiEd25519PublicKey::new(pub_keys_10.clone(), 8).unwrap();
    assert!(multi_signature_7of10
        .verify(&MESSAGE_HASH, &multi_public_key_8of10)
        .is_err());

    // Verifying a 7-of-10 signature against a public key with smaller threshold (i.e., 6) should pass.
    let multi_public_key_6of10 = MultiEd25519PublicKey::new(pub_keys_10.clone(), 6).unwrap();
    assert!(multi_signature_7of10
        .verify(&MESSAGE_HASH, &multi_public_key_6of10)
        .is_ok());

    // Verifying a 7-of-10 signature against a reordered MultiEd25519PublicKey should fail.
    // To deterministically simulate reshuffling, we use a reversed vector of 10 keys.
    // Note that because 10 is an even number, all of they keys will change position.
    let mut pub_keys_10_reversed = pub_keys_10;
    pub_keys_10_reversed.reverse();
    let multi_public_key_7of10_reversed =
        MultiEd25519PublicKey::new(pub_keys_10_reversed, 7).unwrap();
    assert!(multi_signature_7of10
        .verify(&MESSAGE_HASH, &multi_public_key_7of10_reversed)
        .is_err());

    let keypairs_3 = generate_key_pairs(3);
    let priv_keys_3: Vec<Ed25519PrivateKey> = keypairs_3.iter().map(|x| x.0.clone()).collect();

    let multi_private_key_1of3 = MultiEd25519PrivateKey::new(priv_keys_3.clone(), 1).unwrap();
    let multi_public_key_1of3 = MultiEd25519PublicKey::from(&multi_private_key_1of3);

    // Signing with the 2nd key must succeed.
    let sig_with_2nd_key = priv_keys_3[1].sign_message(&MESSAGE_HASH);
    let multi_sig_signed_by_2nd_key =
        MultiEd25519Signature::new(vec![(sig_with_2nd_key.clone(), 1)]);
    assert!(multi_sig_signed_by_2nd_key.is_ok());
    assert!(multi_sig_signed_by_2nd_key
        .unwrap()
        .verify(&MESSAGE_HASH, &multi_public_key_1of3)
        .is_ok());

    // Signing with the 2nd key but using wrong index will fail.
    let sig_with_2nd_key = priv_keys_3[1].sign_message(&MESSAGE_HASH);
    let multi_sig_signed_by_2nd_key_wrong_index =
        MultiEd25519Signature::new(vec![(sig_with_2nd_key.clone(), 2)]);
    assert!(multi_sig_signed_by_2nd_key_wrong_index.is_ok());
    let failed_multi_sig_signed_by_2nd_key_wrong_index = multi_sig_signed_by_2nd_key_wrong_index
        .unwrap()
        .verify(&MESSAGE_HASH, &multi_public_key_1of3);
    assert!(failed_multi_sig_signed_by_2nd_key_wrong_index.is_err());

    // Signing with the 2nd and 3rd keys must succeed, even if we surpass the threshold.
    let sig_with_3rd_key = priv_keys_3[2].sign_message(&MESSAGE_HASH);
    let multi_sig_signed_by_2nd_and_3rd_key = MultiEd25519Signature::new(vec![
        (sig_with_2nd_key.clone(), 1),
        (sig_with_3rd_key.clone(), 2),
    ]);
    assert!(multi_sig_signed_by_2nd_and_3rd_key.is_ok());
    assert!(multi_sig_signed_by_2nd_and_3rd_key
        .unwrap()
        .verify(&MESSAGE_HASH, &multi_public_key_1of3)
        .is_ok());

    // Signing with the 2nd and 3rd keys will fail if we swap indexes.
    let multi_sig_signed_by_2nd_and_3rd_key_swapped =
        MultiEd25519Signature::new(vec![(sig_with_2nd_key.clone(), 2), (sig_with_3rd_key, 1)]);
    let failed_multi_sig_signed_by_2nd_and_3rd_key_swapped =
        multi_sig_signed_by_2nd_and_3rd_key_swapped
            .unwrap()
            .verify(&MESSAGE_HASH, &multi_public_key_1of3);
    assert!(failed_multi_sig_signed_by_2nd_and_3rd_key_swapped.is_err());

    // Signing with the 2nd and an unrelated key. Although threshold is met, it should fail as
    // we don't accept invalid signatures.
    let sig_with_unrelated_key = priv_keys_10[9].sign_message(&MESSAGE_HASH);
    let multi_sig_signed_by_2nd_and_unrelated_key =
        MultiEd25519Signature::new(vec![(sig_with_2nd_key, 1), (sig_with_unrelated_key, 2)]);
    assert!(multi_sig_signed_by_2nd_and_unrelated_key.is_ok());
    let failed_verified_sig = multi_sig_signed_by_2nd_and_unrelated_key
        .unwrap()
        .verify(&MESSAGE_HASH, &multi_public_key_1of3);
    assert!(failed_verified_sig.is_err());
}
