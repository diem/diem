// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    multi_ed25519::{MultiEd25519PrivateKey, MultiEd25519PublicKey},
    test_utils::KeyPair,
    traits::*,
};
use diem_crypto_derive::{CryptoHasher, BCSCryptoHash};
use rand::{prelude::ThreadRng, thread_rng};
use serde::{Deserialize, Serialize};

#[derive(CryptoHasher, BCSCryptoHash, Serialize, Deserialize)]
struct TestTypedSemantics(String);

fn main() {
    let mut csprng: ThreadRng = thread_rng();
    let ed25519_keypair: KeyPair<Ed25519PrivateKey, Ed25519PublicKey> =
        KeyPair::generate(&mut csprng);

    let message = TestTypedSemantics(String::from("hello_world"));
    let signature = ed25519_keypair.private_key.sign(&message);

    let multi_ed25519_keypair: KeyPair<MultiEd25519PrivateKey, MultiEd25519PublicKey> =
        KeyPair::generate(&mut csprng);

    signature.verify(&message, &multi_ed25519_keypair.public_key);
}
