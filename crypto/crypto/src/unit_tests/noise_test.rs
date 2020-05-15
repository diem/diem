// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::{fs::File, io::BufReader, path::PathBuf};

use crate::{
    noise::{handshake_init_msg_len, handshake_resp_msg_len, NoiseConfig, MAX_SIZE_NOISE_MSG},
    test_utils::TEST_SEED,
    x25519, Uniform as _,
};

use crate::ed25519::{Ed25519PrivateKey, Ed25519PublicKey};
use rand::SeedableRng;
use serde::*;
use std::convert::TryFrom;

#[test]
fn convert_from_ed25519_publickey() {
    let mut rng = ::rand::rngs::StdRng::from_seed(TEST_SEED);
    let ed25519_private_key = Ed25519PrivateKey::generate(&mut rng);
    let ed25519_public_key: Ed25519PublicKey = Ed25519PublicKey::from(&ed25519_private_key);
    let x25519_public_key = x25519::PublicKey::try_from(&ed25519_public_key);
    assert!(x25519_public_key.is_ok());

    // Let's construct an x25519 private key from the ed25519 private key.
    let x25519_privatekey = x25519::PrivateKey::try_from(&ed25519_private_key).unwrap();

    // Now derive the public key from x25519_privatekey and see if it matches the public key that
    // was created from the Ed25519PublicKey.
    let x25519_publickey_2 = x25519_privatekey.public_key();
    assert_eq!(x25519_public_key.unwrap(), x25519_publickey_2)
}

#[test]
fn simple_handshake() {
    // setup peers
    let mut rng = ::rand::rngs::StdRng::from_seed(TEST_SEED);
    let initiator_private = x25519::PrivateKey::generate(&mut rng);
    let initiator_public = initiator_private.public_key();
    let responder_private = x25519::PrivateKey::generate(&mut rng);
    let responder_public = responder_private.public_key();
    let initiator = NoiseConfig::new(initiator_private);
    let responder = NoiseConfig::new(responder_private);

    // test the two APIs
    for i in 0..2 {
        // initiator sends first message
        let prologue = b"prologue";
        let payload1 = b"payload1";
        let mut first_message = vec![0u8; handshake_init_msg_len(payload1.len())];
        let initiator_state = initiator
            .initiate_connection(
                &mut rng,
                prologue,
                responder_public,
                Some(payload1),
                &mut first_message,
            )
            .unwrap();

        let payload2 = b"payload2";
        let mut second_message = vec![0u8; handshake_resp_msg_len(payload2.len())];

        // responder parses the first message and responds
        let mut responder_session = if i == 0 {
            let (received_payload, responder_session) = responder
                .respond_to_client_and_finalize(
                    &mut rng,
                    prologue,
                    &first_message,
                    Some(payload2),
                    &mut second_message,
                )
                .unwrap();
            let remote_static = responder_session.get_remote_static();
            assert_eq!(remote_static, initiator_public);
            assert_eq!(received_payload, b"payload1");
            responder_session
        } else {
            let payload2 = b"payload2";
            let (remote_static, handshake_state, received_payload) = responder
                .parse_client_init_message(prologue, &first_message)
                .unwrap();
            assert_eq!(remote_static, initiator_public);
            assert_eq!(received_payload, b"payload1");

            responder
                .respond_to_client(
                    &mut rng,
                    handshake_state,
                    Some(payload2),
                    &mut second_message,
                )
                .unwrap()
        };

        // initiator parses the response
        let (received_payload, mut initiator_session) = initiator
            .finalize_connection(initiator_state, &second_message)
            .unwrap();
        assert_eq!(received_payload, b"payload2");

        // session usage
        let mut message_sent = b"payload".to_vec();
        for i in 0..10 {
            message_sent.push(i);
            let mut message = message_sent.clone();
            let received_message = if i % 2 == 0 {
                let auth_tag = initiator_session
                    .write_message_in_place(&mut message)
                    .expect("session should not be closed");
                message.extend_from_slice(&auth_tag);
                responder_session
                    .read_message_in_place(&mut message)
                    .expect("session should not be closed")
            } else {
                let auth_tag = responder_session
                    .write_message_in_place(&mut message)
                    .expect("session should not be closed");
                message.extend_from_slice(&auth_tag);
                initiator_session
                    .read_message_in_place(&mut message)
                    .expect("session should not be closed")
            };
            assert_eq!(received_message, message_sent.as_slice());
        }
    }
}

#[test]
fn test_vectors() {
    // structures needed to deserialize test vectors
    #[derive(Serialize, Deserialize)]
    struct TestVectors {
        vectors: Vec<TestVector>,
    }
    #[derive(Serialize, Deserialize, Debug)]
    struct TestVector {
        protocol_name: String,
        init_prologue: String,
        init_static: Option<String>,
        init_ephemeral: String,
        init_remote_static: Option<String>,
        resp_static: Option<String>,
        resp_ephemeral: Option<String>,
        handshake_hash: String,
        messages: Vec<TestMessage>,
    }
    #[derive(Serialize, Deserialize, Debug)]
    struct TestMessage {
        payload: String,
        ciphertext: String,
    }

    // EphemeralRng is used to get deterministic ephemeral keys based on test vectors
    struct EphemeralRng {
        ephemeral: Vec<u8>,
    }
    impl rand::RngCore for EphemeralRng {
        fn next_u32(&mut self) -> u32 {
            unreachable!()
        }
        fn next_u64(&mut self) -> u64 {
            unreachable!()
        }
        fn fill_bytes(&mut self, dest: &mut [u8]) {
            dest.copy_from_slice(&self.ephemeral);
        }
        fn try_fill_bytes(&mut self, _dest: &mut [u8]) -> Result<(), rand::Error> {
            unreachable!()
        }
    }
    impl rand::CryptoRng for EphemeralRng {}

    // test vectors are taken from the cacophony library
    let mut test_vectors_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    test_vectors_path.push("test_vectors");
    test_vectors_path.push("noise_cacophony.txt");
    let test_vectors_path = test_vectors_path.to_str().unwrap();
    let test_vectors_file = File::open(test_vectors_path).expect("missing noise test vectors");
    let test_vectors: TestVectors =
        serde_json::from_reader(BufReader::new(test_vectors_file)).unwrap();

    // only go through Noise_IK_25519_AESGCM_SHA256 test vectors (don't exist for SHA-3)
    let test_vector = test_vectors
        .vectors
        .iter()
        .find(|vector| vector.protocol_name == "Noise_IK_25519_AESGCM_SHA256")
        .expect("test vector for Noise_IK_25519_AESGCM_SHA256 should be in cacophony test vectors");

    // initiate peers with test vector
    use crate::traits::ValidCryptoMaterialStringExt;
    let initiator_private =
        x25519::PrivateKey::from_encoded_string(&test_vector.init_static.as_ref().unwrap())
            .unwrap();
    let initiator_public = initiator_private.public_key();
    let responder_private =
        x25519::PrivateKey::from_encoded_string(&test_vector.resp_static.as_ref().unwrap())
            .unwrap();
    let responder_public = responder_private.public_key();

    let initiator = NoiseConfig::new(initiator_private);
    let responder = NoiseConfig::new(responder_private);

    // test the two APIs
    for i in 0..2 {
        // assert public keys
        let init_remote_static =
            hex::decode(test_vector.init_remote_static.as_ref().unwrap()).unwrap();
        assert_eq!(responder_public.as_slice(), init_remote_static.as_slice());

        // go through handshake test messages
        let prologue = hex::decode(&test_vector.init_prologue).unwrap();
        let mut messages = test_vector.messages.iter();

        // first handshake message
        let message = messages.next().unwrap();
        let payload1 = hex::decode(&message.payload).unwrap();
        let expected_ciphertext = hex::decode(&message.ciphertext).unwrap();
        let init_ephemeral = hex::decode(&test_vector.init_ephemeral).unwrap();
        let mut rng = EphemeralRng {
            ephemeral: init_ephemeral,
        };
        let mut first_message = vec![0u8; handshake_init_msg_len(payload1.len())];
        let initiator_state = initiator
            .initiate_connection(
                &mut rng,
                &prologue,
                responder_public,
                Some(&payload1),
                &mut first_message,
            )
            .unwrap();
        assert_eq!(first_message, expected_ciphertext);

        // second handshake message
        let message = messages.next().unwrap();
        let payload2 = hex::decode(&message.payload).unwrap();
        let expected_ciphertext = hex::decode(&message.ciphertext).unwrap();

        // responder part
        let resp_ephemeral = hex::decode(test_vector.resp_ephemeral.as_ref().unwrap()).unwrap();
        let mut rng = EphemeralRng {
            ephemeral: resp_ephemeral,
        };
        let mut second_message = vec![0u8; handshake_resp_msg_len(payload2.len())];

        let mut responder_session = if i == 0 {
            let (received_payload, responder_session) = responder
                .respond_to_client_and_finalize(
                    &mut rng,
                    &prologue,
                    &first_message,
                    Some(&payload2),
                    &mut second_message,
                )
                .unwrap();
            assert_eq!(payload1, received_payload);
            responder_session
        } else {
            let (remote_static, handshake_state, received_payload) = responder
                .parse_client_init_message(&prologue, &first_message)
                .unwrap();
            assert_eq!(remote_static, initiator_public);
            assert_eq!(payload1, received_payload);

            responder
                .respond_to_client(
                    &mut rng,
                    handshake_state,
                    Some(&payload2),
                    &mut second_message,
                )
                .unwrap()
        };

        let remote_static = responder_session.get_remote_static();
        assert_eq!(second_message, expected_ciphertext);
        assert_eq!(remote_static, initiator_public);

        // initiator part
        let (received_payload, mut initiator_session) = initiator
            .finalize_connection(initiator_state, &second_message)
            .unwrap();
        assert_eq!(payload2, received_payload);

        // post-handshake messages
        let mut client_turn = true;
        for message in messages {
            // decode
            let payload = hex::decode(&message.payload).unwrap();
            let expected_ciphertext = hex::decode(&message.ciphertext).unwrap();

            // initiator and responder takes turn to send messages
            let mut message = payload.clone();
            if client_turn {
                let auth_tag = initiator_session
                    .write_message_in_place(&mut message)
                    .expect("session should not be closed");
                message.extend_from_slice(&auth_tag);
                assert_eq!(message, expected_ciphertext);

                let received_payload = responder_session
                    .read_message_in_place(&mut message)
                    .expect("session should not be closed");
                assert_eq!(payload, received_payload);
            } else {
                let auth_tag = responder_session
                    .write_message_in_place(&mut message)
                    .expect("session should not be closed");
                message.extend_from_slice(&auth_tag);
                assert_eq!(message, expected_ciphertext);

                let received_payload = initiator_session
                    .read_message_in_place(&mut message)
                    .expect("session should not be closed");
                assert_eq!(payload, received_payload);
            }

            // swap sender
            client_turn = !client_turn;
        }
    }
}

// Negative tests
// --------------
//
// things that should fail during the handshake:
// - buffer to write is too small (should fail)
// - message received is too small (should fail)
// - message received is too big (should fail)
// - message received is larger than max noise size (should fail)
// - payload to write is larger than max noise size (should fail)
//
// things that should work during the handshake:
// - buffer to write is too big
//

#[test]
fn wrong_buffer_sizes() {
    // setup peers
    let mut rng = ::rand::rngs::StdRng::from_seed(TEST_SEED);
    let initiator_private = x25519::PrivateKey::generate(&mut rng);
    let responder_private = x25519::PrivateKey::generate(&mut rng);
    let responder_public = responder_private.public_key();
    let initiator = NoiseConfig::new(initiator_private);
    let responder = NoiseConfig::new(responder_private);

    // test the two APIs
    for i in 0..2 {
        // initiator sends first message with buffer too small (should fail)
        let payload = b"payload";
        let mut first_message_bad = vec![0u8; handshake_init_msg_len(payload.len()) - 1];
        let res = initiator.initiate_connection(
            &mut rng,
            b"",
            responder_public,
            Some(payload),
            &mut first_message_bad,
        );

        assert!(matches!(res, Err(_)));

        // try again with payload too large (should fail)
        let mut large_buffer = vec![0u8; MAX_SIZE_NOISE_MSG + 3];
        let payload_too_large = vec![1u8; MAX_SIZE_NOISE_MSG - handshake_init_msg_len(0) + 1];
        let res = initiator.initiate_connection(
            &mut rng,
            b"",
            responder_public,
            Some(&payload_too_large),
            &mut large_buffer,
        );

        assert!(matches!(res, Err(_)));

        // try again with buffer too large (should work)
        let mut first_message_good = vec![0u8; handshake_init_msg_len(payload.len()) + 1];
        let initiator_state = initiator
            .initiate_connection(
                &mut rng,
                b"",
                responder_public,
                Some(payload),
                &mut first_message_good,
            )
            .unwrap();

        // responder parses the first message and responds
        let mut second_message_small = vec![0u8; handshake_resp_msg_len(payload.len()) - 1];
        let mut second_message_large = vec![0u8; handshake_resp_msg_len(payload.len()) + 1];

        let (mut responder_session, second_message_large) = if i == 0 {
            // with buffer too small (shouldn't work)
            let res = responder.respond_to_client_and_finalize(
                &mut rng,
                b"",
                &first_message_good[..first_message_good.len() - 1],
                Some(payload),
                &mut second_message_small,
            );

            assert!(matches!(res, Err(_)));

            // with first message too large (shouldn't work)
            let res = responder.respond_to_client_and_finalize(
                &mut rng,
                b"",
                &first_message_good,
                Some(payload),
                &mut second_message_large,
            );

            assert!(matches!(res, Err(_)));

            // with incorrect prologue (should fail)
            let res = responder.respond_to_client_and_finalize(
                &mut rng,
                b"incorrect prologue",
                &first_message_good[..first_message_good.len() - 1],
                Some(payload),
                &mut second_message_large,
            );

            assert!(matches!(res, Err(_)));

            // with payload too large (should fail)
            let mut large_buffer = vec![0u8; MAX_SIZE_NOISE_MSG + 3];
            let payload_too_large = vec![1u8; MAX_SIZE_NOISE_MSG - handshake_resp_msg_len(0) + 1];
            let res = responder.respond_to_client_and_finalize(
                &mut rng,
                b"",
                &first_message_good[..first_message_good.len() - 1],
                Some(&payload_too_large),
                &mut large_buffer,
            );

            assert!(matches!(res, Err(_)));

            // with correct first message and buffer too large (should work)
            let (_, responder_session) = responder
                .respond_to_client_and_finalize(
                    &mut rng,
                    b"",
                    &first_message_good[..first_message_good.len() - 1],
                    Some(payload),
                    &mut second_message_large,
                )
                .unwrap();

            (responder_session, second_message_large)
        } else {
            // with first message too large
            let res = responder.parse_client_init_message(b"", &first_message_good);

            assert!(matches!(res, Err(_)));

            // with first message too small
            let res = responder.parse_client_init_message(
                b"",
                &first_message_good[..first_message_good.len() - 2],
            );

            assert!(matches!(res, Err(_)));

            // with wrong prologue
            let res = responder.parse_client_init_message(
                b"incorrect prologue",
                &first_message_good[..first_message_good.len() - 1],
            );

            assert!(matches!(res, Err(_)));

            // with first message of correct length
            let (_, handshake_state, _) = responder
                .parse_client_init_message(b"", &first_message_good[..first_message_good.len() - 1])
                .unwrap();

            // write to buffer to small (should fail)
            let res = responder.respond_to_client(
                &mut rng,
                handshake_state.clone(),
                Some(payload),
                &mut second_message_small,
            );

            assert!(matches!(res, Err(_)));

            // with payload too large (should fail)
            let mut large_buffer = vec![0u8; MAX_SIZE_NOISE_MSG + 3];
            let payload_too_large = vec![1u8; MAX_SIZE_NOISE_MSG - handshake_resp_msg_len(0) + 1];
            let res = responder.respond_to_client(
                &mut rng,
                handshake_state.clone(),
                Some(&payload_too_large),
                &mut large_buffer,
            );

            assert!(matches!(res, Err(_)));

            // write to buffer too big (should work)
            let responder_session = responder
                .respond_to_client(
                    &mut rng,
                    handshake_state,
                    Some(payload),
                    &mut second_message_large,
                )
                .unwrap();

            (responder_session, second_message_large)
        };

        // initiator parses the response too large (should fail)
        let res = initiator.finalize_connection(initiator_state.clone(), &second_message_large);

        assert!(matches!(res, Err(_)));

        // initiator parses the response too small (should fail)
        let res = initiator.finalize_connection(
            initiator_state.clone(),
            &second_message_large[..second_message_large.len() - 2],
        );

        assert!(matches!(res, Err(_)));

        // initiator parses response of correct size
        let (_, mut initiator_session) = initiator
            .finalize_connection(
                initiator_state.clone(),
                &second_message_large[..second_message_large.len() - 1],
            )
            .unwrap();

        // session usage
        let mut message = b"".to_vec();

        let auth_tag = initiator_session
            .write_message_in_place(&mut message)
            .expect("should work");

        // message too short to have auth tag
        let res = responder_session.read_message_in_place(&mut message);
        assert!(matches!(res, Err(_)));

        // session should be unusable now
        message.extend_from_slice(&auth_tag);
        let res = responder_session.read_message_in_place(&mut message);
        assert!(matches!(res, Err(_)));
    }
}
