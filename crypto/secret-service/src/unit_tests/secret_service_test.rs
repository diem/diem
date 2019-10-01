// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    proto::KeyType,
    secret_service_server::{KeyID, SecretServiceServer},
};
use crypto::{
    hash::HashValue,
    traits::{Signature, ValidKey},
};

/////////////////////////////////////////////////////////////////////////////////////
// These tests check interoperability of key_generation,                           //
// key_retrieval and signing for crate::secret_service_server::SecretServiceServer //
/////////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_generate_and_retrieve_keys() {
    let mut ss_service = SecretServiceServer::new();
    let keys = [KeyType::Ed25519, KeyType::Bls12381];

    for key_type in keys.iter() {
        /* no placeholder key id exists */
        assert!(
            ss_service
                .get_public_key_inner(&KeyID(HashValue::zero()))
                .is_none(),
            "Empty SecretService returns a key"
        );

        let keyid1 = ss_service.generate_key_inner(*key_type).unwrap();

        /* no placeholder key id exists */
        assert!(
            ss_service
                .get_public_key_inner(&KeyID(HashValue::zero()))
                .is_none(),
            "SecretService returns a key on incorrect keyid"
        );

        /* key id that was put exists */
        let public_key1 = ss_service.get_public_key_inner(&keyid1);
        assert!(
            ss_service.get_public_key_inner(&keyid1).is_some(),
            "SecretService does not return a key"
        );
        let public_key1 = public_key1.unwrap();

        /* serialized and deserialized key id that was put exists */
        let key_id_from_wire = KeyID(HashValue::from_slice(&keyid1.0.to_vec()).unwrap());
        assert!(
            ss_service.get_public_key_inner(&key_id_from_wire).is_some(),
            "KeyId serialization into byte array is broken"
        );

        let keyid2 = ss_service.generate_key_inner(*key_type).unwrap();
        let public_key2 = ss_service.get_public_key_inner(&keyid2);
        assert!(
            ss_service.get_public_key_inner(&keyid2).is_some(),
            "SecretService does not return a key"
        );
        let public_key2 = public_key2.unwrap();

        assert_ne!(
            public_key1.to_bytes(),
            public_key2.to_bytes(),
            "SecretService returns same keys on different key ids"
        );

        /* check same keys received when invoked again */
        let public_key10 = ss_service.get_public_key_inner(&keyid1);
        assert!(
            ss_service.get_public_key_inner(&keyid1).is_some(),
            "SecretService does not return a key"
        );
        let public_key10 = public_key10.unwrap();
        let public_key20 = ss_service.get_public_key_inner(&keyid2);
        assert!(
            ss_service.get_public_key_inner(&keyid2).is_some(),
            "SecretService does not return a key"
        );
        let public_key20 = public_key20.unwrap();
        assert_eq!(
            public_key1.to_bytes(),
            public_key10.to_bytes(),
            "SecretService keys don't match"
        );
        assert_eq!(
            public_key2.to_bytes(),
            public_key20.to_bytes(),
            "SecretService keys don't match"
        );
    }
}

#[test]
fn test_ed25519_sign() {
    let mut ss_service = SecretServiceServer::new();

    let keys = [KeyType::Ed25519, KeyType::Bls12381];

    for key_type in keys.iter() {
        let keyid1 = ss_service.generate_key_inner(*key_type).unwrap();
        let public_key1 = ss_service.get_public_key_inner(&keyid1);
        assert!(
            ss_service.get_public_key_inner(&keyid1).is_some(),
            "SecretService does not return a key"
        );
        let public_key1 = public_key1.unwrap();

        let keyid2 = ss_service.generate_key_inner(*key_type).unwrap();
        let public_key2 = ss_service.get_public_key_inner(&keyid2);
        assert!(
            ss_service.get_public_key_inner(&keyid2).is_some(),
            "SecretService does not return a key"
        );
        let public_key2 = public_key2.unwrap();

        /* signature obtained verifies */
        // let message_hash1 = b"hello".digest(STANDARD_DIGESTER.get());
        let message_hash1 = HashValue::random();
        let signature11 = ss_service.sign_inner(&keyid1, &message_hash1);
        assert!(
            signature11.is_some(),
            "SecretService does not return a signature"
        );
        let signature11 = signature11.unwrap();
        assert!(
            signature11.verify(&message_hash1, &public_key1).is_ok(),
            "Correct signature does not verify"
        );

        // let message_hash2 = b"world".digest(STANDARD_DIGESTER.get());
        let message_hash2 = HashValue::random();
        assert!(
            signature11.verify(&message_hash2, &public_key1).is_err(),
            "Incorrect signature verifies"
        );

        let signature22 = ss_service.sign_inner(&keyid2, &message_hash2);
        assert!(
            signature22.is_some(),
            "SecretService does not return a signature"
        );
        let signature22 = signature22.unwrap();
        assert!(
            signature22.verify(&message_hash2, &public_key2).is_ok(),
            "Correct signature does not verify"
        );
        assert!(
            signature22.verify(&message_hash1, &public_key2).is_err(),
            "Incorrect signature verifies"
        );
        assert!(
            signature22.verify(&message_hash1, &public_key1).is_err(),
            "Incorrect signature verifies"
        );
        assert!(
            signature11.verify(&message_hash2, &public_key2).is_err(),
            "Incorrect signature verifies"
        );
    }
}
