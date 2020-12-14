// Copyright (c) The Diem Core Contributors
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
        R: ::rand::RngCore + ::rand::CryptoRng,
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
        R: ::rand::RngCore + ::rand::CryptoRng,
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
        let mut v = bcs::to_bytes(&self.private_key).unwrap();
        v.extend(&bcs::to_bytes(&self.public_key).unwrap());
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

/// This struct provides a means of testing signing and verification through
/// BCS serialization and domain separation
#[cfg(any(test, feature = "fuzzing"))]
#[derive(Debug, Serialize, Deserialize)]
pub struct TestDiemCrypto(pub String);

// the following block is macro expanded from derive(CryptoHasher, BCSCryptoHash)

/// Cryptographic hasher for an BCS-serializable #item
#[cfg(any(test, feature = "fuzzing"))]
pub struct TestDiemCryptoHasher(crate::hash::DefaultHasher);
#[cfg(any(test, feature = "fuzzing"))]
impl ::core::clone::Clone for TestDiemCryptoHasher {
    #[inline]
    fn clone(&self) -> TestDiemCryptoHasher {
        match *self {
            TestDiemCryptoHasher(ref __self_0_0) => {
                TestDiemCryptoHasher(::core::clone::Clone::clone(&(*__self_0_0)))
            }
        }
    }
}
#[cfg(any(test, feature = "fuzzing"))]
static TEST_DIEM_CRYPTO_SEED: crate::_once_cell::sync::OnceCell<[u8; 32]> =
    crate::_once_cell::sync::OnceCell::new();
#[cfg(any(test, feature = "fuzzing"))]
impl TestDiemCryptoHasher {
    fn new() -> Self {
        let name = crate::_serde_name::trace_name::<TestDiemCrypto>()
            .expect("The `CryptoHasher` macro only applies to structs and enums");
        TestDiemCryptoHasher(crate::hash::DefaultHasher::new(&name.as_bytes()))
    }
}
#[cfg(any(test, feature = "fuzzing"))]
static TEST_DIEM_CRYPTO_HASHER: crate::_once_cell::sync::Lazy<TestDiemCryptoHasher> =
    crate::_once_cell::sync::Lazy::new(TestDiemCryptoHasher::new);
#[cfg(any(test, feature = "fuzzing"))]
impl std::default::Default for TestDiemCryptoHasher {
    fn default() -> Self {
        TEST_DIEM_CRYPTO_HASHER.clone()
    }
}
#[cfg(any(test, feature = "fuzzing"))]
impl crate::hash::CryptoHasher for TestDiemCryptoHasher {
    fn seed() -> &'static [u8; 32] {
        TEST_DIEM_CRYPTO_SEED.get_or_init(|| {
            let name = crate::_serde_name::trace_name::<TestDiemCrypto>()
                .expect("The `CryptoHasher` macro only applies to structs and enums.")
                .as_bytes();
            crate::hash::DefaultHasher::prefixed_hash(&name)
        })
    }
    fn update(&mut self, bytes: &[u8]) {
        self.0.update(bytes);
    }
    fn finish(self) -> crate::hash::HashValue {
        self.0.finish()
    }
}
#[cfg(any(test, feature = "fuzzing"))]
impl std::io::Write for TestDiemCryptoHasher {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.update(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
#[cfg(any(test, feature = "fuzzing"))]
impl crate::hash::CryptoHash for TestDiemCrypto {
    type Hasher = TestDiemCryptoHasher;
    fn hash(&self) -> crate::hash::HashValue {
        use crate::hash::CryptoHasher;
        let mut state = Self::Hasher::default();
        bcs::serialize_into(&mut state, &self)
            .expect("BCS serialization of TestDiemCrypto should not fail");
        state.finish()
    }
}

/// Produces a random TestDiemCrypto signable / verifiable struct.
#[cfg(any(test, feature = "fuzzing"))]
pub fn random_serializable_struct() -> impl Strategy<Value = TestDiemCrypto> {
    (String::arbitrary()).prop_map(TestDiemCrypto).no_shrink()
}
