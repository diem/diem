// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[macro_use]
extern crate criterion;

use criterion::Criterion;

use libra_crypto_derive::{CryptoHasher, LCSCryptoHash};
use rand::{prelude::ThreadRng, thread_rng};
use serde::{Deserialize, Serialize};

#[derive(Debug, CryptoHasher, LCSCryptoHash, Serialize, Deserialize)]
pub struct TestLibraCrypto(pub String);

use libra_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey, Ed25519Signature},
    traits::{Signature, SigningKey, Uniform},
};

fn verify(c: &mut Criterion) {
    let mut csprng: ThreadRng = thread_rng();
    let priv_key = Ed25519PrivateKey::generate(&mut csprng);
    let pub_key: Ed25519PublicKey = (&priv_key).into();
    let msg = TestLibraCrypto("".to_string());
    let sig: Ed25519Signature = priv_key.sign(&msg);

    c.bench_function("Ed25519 signature verification", move |b| {
        b.iter(|| sig.verify(&msg, &pub_key))
    });
}

criterion_group!(ed25519_benches, verify);
criterion_main!(ed25519_benches);
