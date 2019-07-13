// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Internal module containing convenience utility functions mainly for testing

use crate::traits::{SeedableCryptoRng, Uniform};
use bincode::serialize;
use serde::Serialize;

/// A seed number for testing
pub const TEST_SEED: [u8; 32] = [0; 32];

/// A labeled tuple consisting of a private and public key
#[cfg_attr(test, derive(Clone))]
pub struct KeyPair<S, P> {
    /// the private (signing) key component
    pub private_key: S,
    /// the corresponding public (verifying) key component
    pub public_key: P,
}

impl<S, P> From<S> for KeyPair<S, P>
where
    for<'a> P: From<&'a S>,
{
    fn from(private_key: S) -> Self {
        KeyPair {
            public_key: (&private_key).into(),
            private_key,
        }
    }
}

impl<S, P> Uniform for KeyPair<S, P>
where
    S: Uniform,
    for<'a> P: From<&'a S>,
{
    fn generate_for_testing<R>(rng: &mut R) -> Self
    where
        R: SeedableCryptoRng,
    {
        let private_key = S::generate_for_testing(rng);
        private_key.into()
    }
}

impl<Priv, Pub> std::fmt::Debug for KeyPair<Priv, Pub>
where
    Priv: Serialize,
    Pub: Serialize + for<'a> From<&'a Priv>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut v = serialize(&self.private_key).unwrap();
        v.extend(&serialize(&self.public_key).unwrap());
        write!(f, "{}", hex::encode(&v[..]))
    }
}
