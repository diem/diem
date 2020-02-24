// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    bls12381::{
        BLS12381PrivateKey, BLS12381PublicKey, BLS12381Signature, BLS12381_PRIVATE_KEY_LENGTH,
        BLS12381_PUBLIC_KEY_LENGTH, BLS12381_SIGNATURE_LENGTH,
    },
    hash::HashValue,
    traits::*,
    unit_tests::uniform_keypair_strategy,
};
use proptest::prelude::*;
use std::convert::TryFrom;

#[test]
fn test_genesis() {
    let mut zeros = [0u8; 32];
    zeros[31] = 1;
    assert_eq!(zeros, BLS12381PrivateKey::genesis().to_bytes()[..]);
}

proptest! {

    #[test]
    fn test_keys_encode(keypair in uniform_keypair_strategy::<BLS12381PrivateKey, BLS12381PublicKey>()) {
        {
            let serialized = lcs::to_bytes(&keypair.private_key).unwrap();
            let encoded = ::hex::encode(&serialized);
            let decoded = BLS12381PrivateKey::from_encoded_string(&encoded);
            prop_assert_eq!(Some(keypair.private_key), decoded.ok());
        }
        {
            let serialized = lcs::to_bytes(&keypair.public_key).unwrap();
            let encoded = ::hex::encode(&serialized);
            let decoded = BLS12381PublicKey::from_encoded_string(&encoded);
            prop_assert_eq!(Some(keypair.public_key), decoded.ok());
        }
    }

    #[test]
    fn test_keys_serde(keypair in uniform_keypair_strategy::<BLS12381PrivateKey, BLS12381PublicKey>()) {
        {
            let serialized: &[u8] = &lcs::to_bytes(&keypair.private_key).unwrap();
            let deserialized = BLS12381PrivateKey::try_from(serialized);
            prop_assert_eq!(Some(keypair.private_key), deserialized.ok());
        }
        {
            let serialized: &[u8] = &lcs::to_bytes(&keypair.public_key).unwrap();
            let deserialized = BLS12381PublicKey::try_from(serialized);
            prop_assert_eq!(Some(keypair.public_key), deserialized.ok());
        }
    }

    #[test]
    fn test_keys_custom_serialisation(
        keypair in uniform_keypair_strategy::<BLS12381PrivateKey, BLS12381PublicKey>()
    ) {
        {
            let serialized: &[u8] = &(keypair.private_key.to_bytes());
            prop_assert_eq!(serialized.len(), BLS12381_PRIVATE_KEY_LENGTH);
            let deserialized = BLS12381PrivateKey::try_from(serialized);
            prop_assert_eq!(Some(keypair.private_key), deserialized.ok());
        }
        {
            let serialized: &[u8] = &(keypair.public_key.to_bytes());
            prop_assert_eq!(serialized.len(), BLS12381_PUBLIC_KEY_LENGTH);
            let deserialized = BLS12381PublicKey::try_from(serialized);
            prop_assert_eq!(Some(keypair.public_key), deserialized.ok());
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn test_signature_serde(
        hash in any::<HashValue>(),
        keypair in uniform_keypair_strategy::<BLS12381PrivateKey, BLS12381PublicKey>()
    ) {
        let signature = keypair.private_key.sign_message(&hash);
        let serialized = lcs::to_bytes(&signature).unwrap();
        prop_assert_eq!(serialized.len(), BLS12381_SIGNATURE_LENGTH);
        let deserialized = lcs::from_bytes(&serialized).unwrap();
        prop_assert!(keypair.public_key.verify_signature(&hash, &deserialized).is_ok());
    }

    #[test]
    fn test_sign_and_verify(
        hash in any::<HashValue>(),
        keypair in uniform_keypair_strategy::<BLS12381PrivateKey, BLS12381PublicKey>()
    ) {
        let signature = keypair.private_key.sign_message(&hash);
        let serialized = lcs::to_bytes(&signature).unwrap();
        let deserialized = lcs::from_bytes(&serialized).unwrap();
        prop_assert!(keypair.public_key.verify_signature(&hash, &deserialized).is_ok());
    }

    #[test]
    fn test_errors(
        hash in any::<HashValue>(),
        keypair in uniform_keypair_strategy::<BLS12381PrivateKey, BLS12381PublicKey>()
    ){
        {
            let serialized = keypair.public_key.to_bytes();
            let truncated = &serialized[..31];
            let key_error = BLS12381PublicKey::try_from(truncated);
            prop_assert_eq!(key_error, Err(CryptoMaterialError::WrongLengthError));
        }
        {
            let serialized = keypair.private_key.to_bytes();
            let truncated = &serialized[..31];
            let key_error = BLS12381PrivateKey::try_from(truncated);
            prop_assert_eq!(key_error, Err(CryptoMaterialError::WrongLengthError));
        }
        {
            let signature = keypair.private_key.sign_message(&hash);
            let truncated = &signature.to_bytes()[..31];
            let sig_error = BLS12381Signature::try_from(truncated);
            // errors for short signatures turn up as validation errors
            prop_assert_eq!(sig_error, Err(CryptoMaterialError::ValidationError));
            let mut extended: Vec<u8> = signature.to_bytes().to_vec();
            extended.extend_from_slice(&signature.to_bytes()[..]);
            let sig_error = BLS12381Signature::try_from(&extended[..]);
            // errors for short signatures turn up as validation errors
            prop_assert_eq!(sig_error, Err(CryptoMaterialError::WrongLengthError));
        }
    }

    #[test]
    fn test_user_accessible_string_encodings(
        keypair in uniform_keypair_strategy::<BLS12381PrivateKey, BLS12381PublicKey>()
    ){
        {
            let encoded = keypair.public_key.to_encoded_string().unwrap();
            let decoded = BLS12381PublicKey::from_encoded_string(&encoded)?;
            prop_assert_eq!(decoded, keypair.public_key);
        }{
            let encoded = keypair.private_key.to_encoded_string().unwrap();
            let decoded = BLS12381PrivateKey::from_encoded_string(&encoded)?;
            prop_assert_eq!(decoded, keypair.private_key);
        }
    }
}
