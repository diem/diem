// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module implements the Verifying/Signing API for signatures on the
//! [BLS12-381 curve](https://tools.ietf.org/id/draft-yonezawa-pairing-friendly-curves-00.html).
//!
//! # Example
//!
//! ```
//! use libra_crypto::hash::{CryptoHasher, TestOnlyHasher};
//! use libra_crypto::{
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
//! testing purposes. Production code should generate the key according to the spec [draft-irtf-cfrg-bls-signature-00](https://tools.ietf.org/id/draft-irtf-cfrg-bls-signature-00.html#keygen).
//!
//! This module is not currently used, but could be included in the future for improved
//! performance in consensus.

use crate::{traits::*, HashValue};
use anyhow::{bail, Result};
use core::convert::TryFrom;
use libra_crypto_derive::{Deref, SilentDebug, SilentDisplay};
use pairing::{
    bls12_381::{Fr, FrRepr},
    PrimeField,
};
use rand::Rng;
use serde::{Deserialize, Serialize};

/// The length of the BLS12381PrivateKey.
pub const BLS12381_PRIVATE_KEY_LENGTH: usize = 32;
/// The length of the BLS12381PublicKey.
pub const BLS12381_PUBLIC_KEY_LENGTH: usize = threshold_crypto::PK_SIZE;
/// The length of the BLS12381Signature.
pub const BLS12381_SIGNATURE_LENGTH: usize = threshold_crypto::SIG_SIZE;

// type alias for this unwieldy type.
type ThresholdBLSPrivateKey =
    threshold_crypto::serde_impl::SerdeSecret<threshold_crypto::SecretKey>;

/// A BLS12-381 private key.
#[derive(Serialize, Deserialize, Deref, SilentDisplay, SilentDebug)]
pub struct BLS12381PrivateKey(ThresholdBLSPrivateKey);

#[cfg(feature = "assert-private-keys-not-cloneable")]
static_assertions::assert_not_impl_any!(BLS12381PrivateKey: Clone);

/// A BLS12-381 public key.
#[derive(Clone, Hash, Serialize, Deserialize, Deref, Debug, PartialEq, Eq)]
pub struct BLS12381PublicKey(threshold_crypto::PublicKey);

/// A BLS12-381 signature.
#[derive(Clone, Hash, Serialize, Deserialize, Deref, Debug, PartialEq, Eq)]
pub struct BLS12381Signature(threshold_crypto::Signature);

impl BLS12381PublicKey {
    /// Serializes a BLS12381PublicKey.
    pub fn to_bytes(&self) -> [u8; BLS12381_PUBLIC_KEY_LENGTH] {
        self.0.to_bytes()
    }
}

impl BLS12381Signature {
    /// Serializes a BLS12381Signature.
    pub fn to_bytes(&self) -> [u8; BLS12381_SIGNATURE_LENGTH] {
        self.0.to_bytes()
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

/// r = 52435875175126190479447740508185965837690552500527637822603658699938581184513
/// For BLS12-381 curve parameters see [draft-yonezawa-pairing-friendly-curves-02](https://tools.ietf.org/html/draft-yonezawa-pairing-friendly-curves-02#section-4.2.2).
const MODULUS_R: FrRepr = FrRepr([
    0xffff_ffff_0000_0001,
    0x53bd_a402_fffe_5bfe,
    0x3339_d808_09a1_d805,
    0x73ed_a753_299d_7d48,
]);

impl Uniform for BLS12381PrivateKey {
    /// This method generates a private signing key for BLS.
    /// It samples a random 255 bits number and checks whether this number is smaller than the
    /// modulu r, if so, it uses the number to compose a secret key, otherwise it retries.
    /// This method should be used for testing purposes only, production code should generate
    /// the key according to the spec [draft-irtf-cfrg-bls-signature-00](https://tools.ietf.org/id/draft-irtf-cfrg-bls-signature-00.html#keygen).
    fn generate_for_testing<R>(rng: &mut R) -> Self
    where
        R: ::rand::SeedableRng + ::rand::RngCore + ::rand::CryptoRng,
    {
        let mut fr;
        loop {
            let mut fr_repr: [u64; 4usize] = rng.gen();
            fr_repr[3] &= 0x7FFF_FFFF_FFFF_FFFF;

            // stop sampling if the resulting number is less than the modulus r
            if FrRepr(fr_repr) < MODULUS_R {
                fr = Fr::from_repr(FrRepr(fr_repr)).unwrap();
                break;
            }
        }
        BLS12381PrivateKey(threshold_crypto::serde_impl::SerdeSecret(
            threshold_crypto::SecretKey::from_mut(&mut fr),
        ))
    }
}

impl std::cmp::PartialEq<Self> for BLS12381PrivateKey {
    fn eq(&self, other: &Self) -> bool {
        lcs::to_bytes(self).unwrap() == lcs::to_bytes(other).unwrap()
    }
}

impl std::cmp::Eq for BLS12381PrivateKey {}

impl TryFrom<&[u8]> for BLS12381PrivateKey {
    type Error = CryptoMaterialError;

    fn try_from(bytes: &[u8]) -> std::result::Result<BLS12381PrivateKey, CryptoMaterialError> {
        if bytes.len() != 32 {
            return Err(CryptoMaterialError::WrongLengthError);
        }
        // First we deserialize raw bytes, which may or may not work.
        let key_res = lcs::from_bytes(bytes);
        // Note that the underlying implementation of SerdeSecret checks for validation during
        // deserialization. Also, we don't need to check for validity of the derived PublicKey, as a
        // correct SerdeSecret (checked during deserialization that is in field) will always produce
        // a valid PublicKey.
        key_res.or(Err(CryptoMaterialError::DeserializationError))
    }
}
impl ValidKey for BLS12381PrivateKey {
    fn to_bytes(&self) -> Vec<u8> {
        // This is expected to succeed as we just return the bytes of an existing key.
        lcs::to_bytes(&self.0).unwrap()
    }
}

impl Genesis for BLS12381PrivateKey {
    fn genesis() -> Self {
        let mut buf = [0u8; BLS12381_PRIVATE_KEY_LENGTH];
        buf[BLS12381_PRIVATE_KEY_LENGTH - 1] = 1;
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
        if bytes.len() != BLS12381_PUBLIC_KEY_LENGTH {
            return Err(CryptoMaterialError::WrongLengthError);
        }
        let mut byte_array = [0u8; BLS12381_PUBLIC_KEY_LENGTH];
        byte_array.copy_from_slice(&bytes[..BLS12381_PUBLIC_KEY_LENGTH]);
        // Note that the underlying implementation of threshold_crypto::PublicKey::from_bytes checks
        // for validation during deserialization.
        let pub_key = threshold_crypto::PublicKey::from_bytes(byte_array);
        match pub_key {
            Ok(key) => Ok(BLS12381PublicKey(key)),
            Err(_) => Err(CryptoMaterialError::DeserializationError),
        }
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
        if l > BLS12381_SIGNATURE_LENGTH {
            return Err(CryptoMaterialError::WrongLengthError);
        }
        let mut tmp = [0u8; BLS12381_SIGNATURE_LENGTH];
        tmp[..l].copy_from_slice(&bytes[..l]);
        let sig = threshold_crypto::Signature::from_bytes(&tmp)
            .map_err(|_err| CryptoMaterialError::ValidationError)?;
        Ok(BLS12381Signature(sig))
    }
}
