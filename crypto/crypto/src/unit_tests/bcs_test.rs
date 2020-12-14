// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use rand::{rngs::StdRng, SeedableRng};
use std::convert::TryFrom;

use crate::{
    ed25519::{
        Ed25519PrivateKey, Ed25519PublicKey, Ed25519Signature, ED25519_PRIVATE_KEY_LENGTH,
        ED25519_PUBLIC_KEY_LENGTH, ED25519_SIGNATURE_LENGTH,
    },
    multi_ed25519::{MultiEd25519PrivateKey, MultiEd25519PublicKey, MultiEd25519Signature},
    test_utils::{TestDiemCrypto, TEST_SEED},
    Signature, SigningKey, Uniform,
};

#[test]
fn ed25519_bcs_material() {
    use std::borrow::Cow;

    let private_key =
        Ed25519PrivateKey::try_from([1u8; ED25519_PRIVATE_KEY_LENGTH].as_ref()).unwrap();
    let public_key = Ed25519PublicKey::from(&private_key);

    let serialized_public_key = bcs::to_bytes(&Cow::Borrowed(&public_key)).unwrap();
    // Expected size should be 1 byte due to BCS length prefix + 32 bytes for the raw key bytes
    assert_eq!(serialized_public_key.len(), 1 + ED25519_PUBLIC_KEY_LENGTH);

    // Ensure public key serialization - deserialization is stable and deterministic
    let deserialized_public_key: Ed25519PublicKey =
        bcs::from_bytes(&serialized_public_key).unwrap();
    assert_eq!(deserialized_public_key, public_key);

    let message = TestDiemCrypto("Hello, World".to_string());
    let signature: Ed25519Signature = private_key.sign(&message);

    let serialized_signature = bcs::to_bytes(&Cow::Borrowed(&signature)).unwrap();
    // Expected size should be 1 byte due to BCS length prefix + 64 bytes for the raw signature bytes
    assert_eq!(serialized_signature.len(), 1 + ED25519_SIGNATURE_LENGTH);

    // Ensure signature serialization - deserialization is stable and deterministic
    let deserialized_signature: Ed25519Signature = bcs::from_bytes(&serialized_signature).unwrap();
    assert_eq!(deserialized_signature, signature);

    // Verify signature
    let verified_signature = signature.verify(&message, &public_key);
    assert!(verified_signature.is_ok())
}

#[test]
fn multi_ed25519_bcs_material() {
    use std::borrow::Cow;

    // Helper function to generate N ed25519 private keys.
    fn generate_keys(n: usize) -> Vec<Ed25519PrivateKey> {
        let mut rng = StdRng::from_seed(TEST_SEED);
        (0..n)
            .map(|_| Ed25519PrivateKey::generate(&mut rng))
            .collect()
    }

    let num_of_keys = 10;
    let threshold = 7;
    let private_keys_10 = generate_keys(num_of_keys);
    let multi_private_key_7of10 = MultiEd25519PrivateKey::new(private_keys_10, threshold).unwrap();
    let multi_public_key_7of10 = MultiEd25519PublicKey::from(&multi_private_key_7of10);

    let serialized_multi_public_key =
        bcs::to_bytes(&Cow::Borrowed(&multi_public_key_7of10)).unwrap();

    // Expected size due to specialization is
    // 2 bytes for BCS length prefix (due to ULEB128)
    // + 10 * single_pub_key_size bytes (each key is the compressed Edwards Y coordinate)
    // + 1 byte for the threshold
    assert_eq!(
        serialized_multi_public_key.len(),
        2 + num_of_keys * ED25519_PUBLIC_KEY_LENGTH + 1
    );

    let deserialized_multi_public_key: MultiEd25519PublicKey =
        bcs::from_bytes(&serialized_multi_public_key).unwrap();
    assert_eq!(deserialized_multi_public_key, multi_public_key_7of10);

    let message = TestDiemCrypto("Hello, World".to_string());

    // Verifying a 7-of-10 signature against a public key with the same threshold should pass.
    let multi_signature_7of10: MultiEd25519Signature = multi_private_key_7of10.sign(&message);

    let serialized_multi_signature = bcs::to_bytes(&Cow::Borrowed(&multi_signature_7of10)).unwrap();
    // Expected size due to specialization is
    // 2 bytes for BCS length prefix (due to ULEB128)
    // + 7 * single_signature_size bytes (each sig is of the form (R,s),
    // a 32B compressed Edwards Y coordinate concatenated with a 32B scalar)
    // + 4 bytes for the bitmap (the bitmap can hold up to 32 bits)
    assert_eq!(
        serialized_multi_signature.len(),
        2 + threshold as usize * ED25519_SIGNATURE_LENGTH + 4
    );

    // Verify bitmap
    assert_eq!(
        multi_signature_7of10.bitmap(),
        &[0b1111_1110, 0u8, 0u8, 0u8]
    );

    // Verify signature
    assert!(multi_signature_7of10
        .verify(&message, &multi_public_key_7of10)
        .is_ok());
}
