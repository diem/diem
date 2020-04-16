// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::{fs::File, io::BufReader, path::PathBuf};

use crate::{noise::NoiseConfig, test_utils::TEST_SEED, x25519};

use rand::SeedableRng;
use serde::*;
use serde_json;
use x25519_dalek;

fn vec_to_private_key(bytes: &[u8]) -> x25519_dalek::StaticSecret {
    if bytes.len() != x25519::PRIVATE_KEY_SIZE {
        panic!("X25519 private key is of incorrect length");
    }
    let mut array = [0u8; x25519::PRIVATE_KEY_SIZE];
    array.copy_from_slice(bytes);
    array.into()
}

#[test]
fn simple_handshake() {
    // setup peers
    let mut rng = ::rand::rngs::StdRng::from_seed(TEST_SEED);
    let initiator_static = x25519_dalek::StaticSecret::new(&mut rng);
    let initiator_public = x25519_dalek::PublicKey::from(&initiator_static);
    let responder_static = x25519_dalek::StaticSecret::new(&mut rng);
    let responder_public = x25519_dalek::PublicKey::from(&responder_static);
    let initiator = NoiseConfig::new(initiator_static);
    let responder = NoiseConfig::new(responder_static);

    // initiator sends first message
    let prologue = b"prologue";
    let (initiator_state, first_message) = initiator
        .initiate_connection(&mut rng, prologue, &responder_public, Some(b"payload1"))
        .unwrap();

    // responder parses the first message and responds
    let (second_message, remote_static, received_payload, mut responder_session) = responder
        .respond_to_client_and_finalize(&mut rng, prologue, &first_message, Some(b"payload2"))
        .unwrap();
    assert_eq!(remote_static.as_bytes(), initiator_public.as_bytes());
    assert_eq!(received_payload, b"payload1");

    // initiator parses the response
    let (received_payload, mut initiator_session) = initiator
        .finalize_connection(initiator_state, &second_message)
        .unwrap();
    assert_eq!(received_payload, b"payload2");

    // session usage
    let mut message_sent = "payload".to_string().into_bytes();
    for i in 0..10 {
        message_sent.push(i);
        let received_message = if i % 2 == 0 {
            let encrypted_message = initiator_session
                .write_message(&message_sent)
                .expect("session should not be closed");
            responder_session
                .read_message(&encrypted_message)
                .expect("session should not be closed")
        } else {
            let encrypted_message = responder_session
                .write_message(&message_sent)
                .expect("session should not be closed");
            initiator_session
                .read_message(&encrypted_message)
                .expect("session should not be closed")
        };
        assert_eq!(received_message, message_sent);
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
    let initiator_private =
        vec_to_private_key(&hex::decode(test_vector.init_static.as_ref().unwrap()).unwrap());
    let initiator_public = x25519_dalek::PublicKey::from(&initiator_private);
    let responder_private =
        vec_to_private_key(&hex::decode(&test_vector.resp_static.as_ref().unwrap()).unwrap());
    let responder_public = x25519_dalek::PublicKey::from(&responder_private);

    let initiator = NoiseConfig::new(initiator_private);
    let responder = NoiseConfig::new(responder_private);

    // assert public keys
    let init_remote_static = hex::decode(test_vector.init_remote_static.as_ref().unwrap()).unwrap();
    assert_eq!(responder_public.as_bytes(), init_remote_static.as_slice());

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
    let (initiator_state, ciphertext) = initiator
        .initiate_connection(&mut rng, &prologue, &responder_public, Some(&payload1))
        .unwrap();
    assert_eq!(ciphertext, expected_ciphertext);

    // second handshake message
    let message = messages.next().unwrap();
    let payload2 = hex::decode(&message.payload).unwrap();
    let expected_ciphertext = hex::decode(&message.ciphertext).unwrap();
    // responder part
    let resp_ephemeral = hex::decode(test_vector.resp_ephemeral.as_ref().unwrap()).unwrap();
    let mut rng = EphemeralRng {
        ephemeral: resp_ephemeral,
    };
    let (ciphertext, remote_static, received_payload, mut responder_session) = responder
        .respond_to_client_and_finalize(&mut rng, &prologue, &ciphertext, Some(&payload2))
        .unwrap();
    assert_eq!(ciphertext, expected_ciphertext);
    assert_eq!(remote_static.as_bytes(), initiator_public.as_bytes());
    assert_eq!(payload1, received_payload);
    // initiator part
    let (received_payload, mut initiator_session) = initiator
        .finalize_connection(initiator_state, &ciphertext)
        .unwrap();
    assert_eq!(payload2, received_payload);

    // post-handshake messages
    let mut client_turn = true;
    for message in messages {
        // decode
        let payload = hex::decode(&message.payload).unwrap();
        let expected_ciphertext = hex::decode(&message.ciphertext).unwrap();

        // initiator and responder takes turn to send messages
        let (ciphertext, received_payload) = if client_turn {
            let ciphertext = initiator_session
                .write_message(&payload)
                .expect("session should not be closed");
            let received_payload = responder_session
                .read_message(&ciphertext)
                .expect("session should not be closed");
            (ciphertext, received_payload)
        } else {
            let ciphertext = responder_session
                .write_message(&payload)
                .expect("session should not be closed");
            let received_payload = initiator_session
                .read_message(&ciphertext)
                .expect("session should not be closed");
            (ciphertext, received_payload)
        };

        assert_eq!(ciphertext, expected_ciphertext);
        assert_eq!(payload, received_payload);

        // swap sender
        client_turn = !client_turn;
    }
}
