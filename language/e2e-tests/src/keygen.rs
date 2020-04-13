// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    TPrivateKeyUniform,
};
use rand::{
    rngs::{OsRng, StdRng},
    Rng, SeedableRng,
};

/// Ed25519 key generator.
pub struct KeyGen(StdRng);

impl KeyGen {
    /// Constructs a key generator with a specific seed.
    pub fn from_seed(seed: [u8; 32]) -> Self {
        Self(StdRng::from_seed(seed))
    }

    /// Constructs a key generator with a random seed.
    /// The random seed itself is generated using the OS rng.
    pub fn from_os_rng() -> Self {
        let mut seed_rng = OsRng::new().expect("can't access OsRng");
        let seed: [u8; 32] = seed_rng.gen();
        Self::from_seed(seed)
    }

    /// Generate an Ed25519 key pair.
    pub fn generate_keypair(&mut self) -> (Ed25519PrivateKey, Ed25519PublicKey) {
        let private_key = Ed25519PrivateKey::generate(&mut self.0);
        let public_key = private_key.public_key();
        (private_key, public_key)
    }
}
