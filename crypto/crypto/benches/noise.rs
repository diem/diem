// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Don't forget to run this benchmark with AES-NI enable.
//! You can do this by building with the following flags:
//! `RUSTFLAGS="-Ctarget-cpu=sandybridge -Ctarget-feature=+aes,+sse2,+sse4.1,+ssse3"`.
//!

#[macro_use]
extern crate criterion;

use libra_crypto::{noise::NoiseConfig, test_utils::TEST_SEED};

use criterion::{Benchmark, Criterion, Throughput};
use rand::SeedableRng;
use x25519_dalek as x25519;

const MSG_SIZE: usize = 4096;

fn benchmarks(c: &mut Criterion) {
    // bench the builder
    c.bench(
        "builder",
        Benchmark::new("skeleton", |b| {
            // setup keys first
            let mut rng = ::rand::rngs::StdRng::from_seed(TEST_SEED);
            let initiator_static = x25519::StaticSecret::new(&mut rng);
            b.iter(|| NoiseConfig::new(initiator_static.clone()))
        })
        .throughput(Throughput::Elements(1)),
    );

    // bench the handshake
    c.bench(
        "handshake",
        Benchmark::new("xx", |b| {
            // setup keys first
            let mut rng = ::rand::rngs::StdRng::from_seed(TEST_SEED);
            let initiator_static = x25519::StaticSecret::new(&mut rng);
            let responder_static = x25519_dalek::StaticSecret::new(&mut rng);
            let responder_public = x25519_dalek::PublicKey::from(&responder_static);

            b.iter(|| {
                let initiator = NoiseConfig::new(initiator_static.clone());
                let responder = NoiseConfig::new(responder_static.clone());

                let (initiator_state, first_message) = initiator
                    .initiate_connection(&mut rng, b"prologue", &responder_public, None)
                    .unwrap();
                let (second_message, _, _, _) = responder
                    .respond_to_client_and_finalize(&mut rng, b"prologue", &first_message, None)
                    .unwrap();
                let _ = initiator
                    .finalize_connection(initiator_state, &second_message)
                    .unwrap();
            })
        })
        .throughput(Throughput::Elements(1)),
    );

    c.bench(
        "transport",
        Benchmark::new("AES-GCM throughput", |b| {
            let buffer_msg = [0u8; MSG_SIZE * 2];

            // setup keys first
            let mut rng = ::rand::rngs::StdRng::from_seed(TEST_SEED);
            let initiator_static = x25519::StaticSecret::new(&mut rng);
            let responder_static = x25519_dalek::StaticSecret::new(&mut rng);
            let responder_public = x25519_dalek::PublicKey::from(&responder_static);

            // handshake first
            let initiator = NoiseConfig::new(initiator_static);
            let responder = NoiseConfig::new(responder_static);
            let (initiator_state, first_message) = initiator
                .initiate_connection(&mut rng, b"prologue", &responder_public, None)
                .unwrap();
            let (second_message, _, _, mut responder_session) = responder
                .respond_to_client_and_finalize(&mut rng, b"prologue", &first_message, None)
                .unwrap();
            let (_, mut initiator_session) = initiator
                .finalize_connection(initiator_state, &second_message)
                .unwrap();

            // bench throughput post-handshake
            b.iter(move || {
                let ciphertext = initiator_session
                    .write_message(&buffer_msg[..MSG_SIZE])
                    .expect("session should not be closed");

                let _plaintext = responder_session
                    .read_message(&ciphertext)
                    .expect("session should not be closed");
            })
        })
        .throughput(Throughput::Bytes(MSG_SIZE as u64 * 2)),
    );
}

criterion_group!(benches, benchmarks);
criterion_main!(benches);
