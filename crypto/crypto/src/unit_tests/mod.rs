// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod cross_test;
mod ed25519_test;
mod hkdf_test;
mod multi_ed25519_test;
mod slip0010_test;
mod x25519_test;

use crate::{test_utils::KeyPair, traits::Uniform};
use proptest::prelude::*;
use rand::{rngs::StdRng, SeedableRng};
use serde::Serialize;

/// Produces a uniformly random keypair from a seed
pub(super) fn uniform_keypair_strategy<Priv, Pub>() -> impl Strategy<Value = KeyPair<Priv, Pub>>
where
    Pub: Serialize + for<'a> From<&'a Priv>,
    Priv: Serialize + Uniform,
{
    // The no_shrink is because keypairs should be fixed -- shrinking would cause a different
    // keypair to be generated, which appears to not be very useful.
    any::<[u8; 32]>()
        .prop_map(|seed| {
            let mut rng = StdRng::from_seed(seed);
            KeyPair::<Priv, Pub>::generate_for_testing(&mut rng)
        })
        .no_shrink()
}
