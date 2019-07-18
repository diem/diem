// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module implements the Verifying/Signing API for signatures on the
//! [BLS12-381 curve](https://tools.ietf.org/id/draft-yonezawa-pairing-friendly-curves-00.html).
//!
//! # Example
//!
//! ```
//! use crypto::hash::{CryptoHasher, TestOnlyHasher};
//! use nextgen_crypto::{
//!     bls12381::*,
//!     traits::{Signature, SigningKey, Uniform},
//! };
//! use rand::{rngs::StdRng, SeedableRng};
//!
//! let mut hasher = TestOnlyHasher::default();
//! hasher.write("Test message".as_bytes());
//! let hashed_message = hasher.finish();
//!
//! let mut rng: StdRng = SeedableRng::from_seed([0; 32]);
//! let private_key = BLS12381PrivateKey::generate_for_testing(&mut rng);
//! let public_key: BLS12381PublicKey = (&private_key).into();
//! let signature = private_key.sign_message(&hashed_message);
//! assert!(signature.verify(&hashed_message, &public_key).is_ok());
//! ```
//! **Note**: The above example generates a private key using a private function intended only for
//! testing purposes. Production code should find an alternate means for secure key generation.
//!
//! This module is not currently used, but could be included in the future for improved
//! performance in consensus.

use crate::traits::*;
use bincode::{deserialize, serialize};
use core::convert::TryFrom;
use crypto::hash::HashValue;
use crypto_derive::{SilentDebug, SilentDisplay};
use derive_deref::Deref;
use failure::prelude::*;
use pairing::{
    bls12_381::{Fr, FrRepr},
    PrimeField,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use threshold_crypto;

// type alias for this unwieldy type
type ThresholdBLSPrivateKey =
    threshold_crypto::serde_impl::SerdeSecret<threshold_crypto::SecretKey>;

/// A BLS12-381 private key
#[derive(Serialize, Deserialize, Deref, SilentDisplay, SilentDebug)]
pub struct BLS12381PrivateKey(ThresholdBLSPrivateKey);

/// A BLS12-381 public key
#[derive(Clone, Hash, Serialize, Deserialize, Deref, Debug, PartialEq, Eq)]
pub struct BLS12381PublicKey(threshold_crypto::PublicKey);

/// A BLS12-381 signature
#[derive(Clone, Hash, Serialize, Deserialize, Deref, Debug, PartialEq, Eq)]
pub struct BLS12381Signature(threshold_crypto::Signature);

impl BLS12381PublicKey {
    /// Serializes a BLS12381PublicKey
    pub fn to_bytes(&self) -> [u8; threshold_crypto::PK_SIZE] {
        self.0.to_bytes()
    }
}

impl BLS12381Signature {
    /// Serializes a BLS12381Signature
    pub fn to_bytes(&self) -> [u8; threshold_crypto::SIG_SIZE] {
        self.0.to_bytes()
    }
}

impl BLS12381PrivateKey {
    #[allow(dead_code)]
    /// Deserialize a [`BLS12381PrivateKey`]. This method DOES NOT check for key validity.
    fn from_bytes_unchecked(
        mut fr_repr: [u64; 4usize],
    ) -> std::result::Result<BLS12381PrivateKey, CryptoMaterialError> {
        // let mut fr_repr: [u64; 4usize] = rng.gen();
        // Since field modulus is 381-bit prime, drop the 3 highest-order bits
        fr_repr[3] &= 0x1FFF_FFFF_FFFF_FFFF;
        let mut fr = Fr::from_repr(FrRepr(fr_repr)).unwrap();
        Ok(BLS12381PrivateKey(
            threshold_crypto::serde_impl::SerdeSecret(threshold_crypto::SecretKey::from_mut(
                &mut fr,
            )),
        ))
    }
}

///////////////////////
// PrivateKey Traits //
///////////////////////

impl PrivateKey for BLS12381PrivateKey {
    type PublicKeyMaterial = BLS12381PublicKey;
}

impl SigningKey for BLS12381PrivateKey {
    type VerifyingKeyMaterial = BLS12381PublicKey;
    type SignatureMaterial = BLS12381Signature;

    fn sign_message(&self, message: &HashValue) -> BLS12381Signature {
        let secret_key: &ThresholdBLSPrivateKey = self;
        let sig = secret_key.sign(message.as_ref());
        BLS12381Signature(sig)
    }
}

impl Uniform for BLS12381PrivateKey {
    fn generate_for_testing<R>(rng: &mut R) -> Self
    where
        R: ::rand::SeedableRng + ::rand::RngCore + ::rand::CryptoRng,
    {
        let mut fr_repr: [u64; 4usize] = rng.gen();
        // Since field modulus is 381-bit prime, drop the 3 highest-order bits
        fr_repr[3] &= 0x1FFF_FFFF_FFFF_FFFF;
        let mut fr = Fr::from_repr(FrRepr(fr_repr)).unwrap();
        BLS12381PrivateKey(threshold_crypto::serde_impl::SerdeSecret(
            threshold_crypto::SecretKey::from_mut(&mut fr),
        ))
    }
}

impl std::cmp::PartialEq<Self> for BLS12381PrivateKey {
    fn eq(&self, other: &Self) -> bool {
        serialize(self).unwrap() == serialize(other).unwrap()
    }
}

impl std::cmp::Eq for BLS12381PrivateKey {}

impl TryFrom<&[u8]> for BLS12381PrivateKey {
    type Error = CryptoMaterialError;

    fn try_from(bytes: &[u8]) -> std::result::Result<BLS12381PrivateKey, CryptoMaterialError> {
        // first we deserialize raw bytes, which may or may not work
        let key_res = deserialize::<BLS12381PrivateKey>(bytes);
        // Note that the underlying implementation of SerdeSecret checks for validation during
        // deserialization. Also, we don't need to check for validity of the derived PublicKey, as a
        // correct SerdeSecret (checked during deserialization that is in field) will always produce
        // a valid PublicKey.
        key_res.or(Err(CryptoMaterialError::DeserializationError))
    }
}
impl ValidKey for BLS12381PrivateKey {
    // TODO(ladi): implement!
    fn to_bytes(&self) -> Vec<u8> {
        unimplemented!("ask ladi!")
    }
}

impl Genesis for BLS12381PrivateKey {
    fn genesis() -> Self {
        let mut buf = [0u8; threshold_crypto::PK_SIZE];
        buf[threshold_crypto::PK_SIZE - 1] = 1;
        Self::try_from(buf.as_ref()).unwrap()
    }
}

//////////////////////
// PublicKey Traits //
//////////////////////
impl From<&BLS12381PrivateKey> for BLS12381PublicKey {
    fn from(secret_key: &BLS12381PrivateKey) -> Self {
        let secret: &ThresholdBLSPrivateKey = secret_key;
        let public: threshold_crypto::PublicKey = secret.public_key();
        BLS12381PublicKey(public)
    }
}

// We deduce PublicKey from this
// we get the ability to do `pubkey.validate(msg, signature)`
impl PublicKey for BLS12381PublicKey {
    type PrivateKeyMaterial = BLS12381PrivateKey;
    fn length() -> usize {
        threshold_crypto::PK_SIZE
    }
}

impl VerifyingKey for BLS12381PublicKey {
    type SigningKeyMaterial = BLS12381PrivateKey;
    type SignatureMaterial = BLS12381Signature;
}
impl std::fmt::Display for BLS12381PublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(&self.to_bytes()[..]))
    }
}

impl TryFrom<&[u8]> for BLS12381PublicKey {
    type Error = CryptoMaterialError;

    fn try_from(bytes: &[u8]) -> std::result::Result<BLS12381PublicKey, CryptoMaterialError> {
        // first we deserialize raw bytes, which may or may not work
        let key_res = deserialize::<BLS12381PublicKey>(bytes);
        // TODO: call some validation! For now we just put in a
        // very conspicuous useless lambda
        key_res.or(Err(CryptoMaterialError::DeserializationError))
    }
}

impl ValidKey for BLS12381PublicKey {
    fn to_bytes(&self) -> Vec<u8> {
        self.0.to_bytes().to_vec()
    }
}

//////////////////////
// Signature Traits //
//////////////////////

impl Signature for BLS12381Signature {
    type VerifyingKeyMaterial = BLS12381PublicKey;
    type SigningKeyMaterial = BLS12381PrivateKey;

    /// Checks that `signature` is valid for `message` using `public_key`.
    fn verify(&self, message: &HashValue, public_key: &BLS12381PublicKey) -> Result<()> {
        self.verify_arbitrary_msg(message.as_ref(), public_key)
    }

    /// Checks that `signature` is valid for an arbitrary &[u8] `message` using `public_key`.
    fn verify_arbitrary_msg(&self, message: &[u8], public_key: &BLS12381PublicKey) -> Result<()> {
        if public_key.verify(self, message.as_ref()) {
            Ok(())
        } else {
            bail!("The provided signature is not valid on this PublicKey and Message")
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.0.to_bytes().to_vec()
    }
}

impl TryFrom<&[u8]> for BLS12381Signature {
    type Error = CryptoMaterialError;

    fn try_from(bytes: &[u8]) -> std::result::Result<BLS12381Signature, CryptoMaterialError> {
        let l = bytes.len();
        if l > threshold_crypto::SIG_SIZE {
            return Err(CryptoMaterialError::WrongLengthError);
        }
        let mut tmp = [0u8; threshold_crypto::SIG_SIZE];
        tmp[..l].copy_from_slice(&bytes[..l]);
        let sig = threshold_crypto::Signature::from_bytes(&tmp)
            .map_err(|_err| CryptoMaterialError::ValidationError)?;
        Ok(BLS12381Signature(sig))
    }
}
