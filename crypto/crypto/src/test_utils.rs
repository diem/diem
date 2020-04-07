// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Internal module containing convenience utility functions mainly for testing

use crate::traits::Uniform;
use serde::{Deserialize, Serialize};

/// A deterministic seed for PRNGs related to keys
pub const TEST_SEED: [u8; 32] = [0u8; 32];

/// A keypair consisting of a private and public key
#[cfg_attr(feature = "cloneable-private-keys", derive(Clone))]
#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub struct KeyPair<S, P>
where
    for<'a> P: From<&'a S>,
{
    /// the private key component
    pub private_key: S,
    /// the public key component
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
    fn generate<R>(rng: &mut R) -> Self
    where
        R: ::rand::SeedableRng + ::rand::RngCore + ::rand::CryptoRng,
    {
        let private_key = S::generate(rng);
        private_key.into()
    }
}

/// A pair consisting of a private and public key
impl<S, P> Uniform for (S, P)
where
    S: Uniform,
    for<'a> P: From<&'a S>,
{
    fn generate<R>(rng: &mut R) -> Self
    where
        R: ::rand::SeedableRng + ::rand::RngCore + ::rand::CryptoRng,
    {
        let private_key = S::generate(rng);
        let public_key = (&private_key).into();
        (private_key, public_key)
    }
}

impl<Priv, Pub> std::fmt::Debug for KeyPair<Priv, Pub>
where
    Priv: Serialize,
    Pub: Serialize + for<'a> From<&'a Priv>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut v = lcs::to_bytes(&self.private_key).unwrap();
        v.extend(&lcs::to_bytes(&self.public_key).unwrap());
        write!(f, "{}", hex::encode(&v[..]))
    }
}

#[cfg(any(test, feature = "fuzzing"))]
use proptest::prelude::*;
#[cfg(any(test, feature = "fuzzing"))]
use rand::{rngs::StdRng, SeedableRng};

/// Produces a uniformly random keypair from a seed
#[cfg(any(test, feature = "fuzzing"))]
pub fn uniform_keypair_strategy<Priv, Pub>() -> impl Strategy<Value = KeyPair<Priv, Pub>>
where
    Pub: Serialize + for<'a> From<&'a Priv>,
    Priv: Serialize + Uniform,
{
    // The no_shrink is because keypairs should be fixed -- shrinking would cause a different
    // keypair to be generated, which appears to not be very useful.
    any::<[u8; 32]>()
        .prop_map(|seed| {
            let mut rng = StdRng::from_seed(seed);
            KeyPair::<Priv, Pub>::generate(&mut rng)
        })
        .no_shrink()
}
