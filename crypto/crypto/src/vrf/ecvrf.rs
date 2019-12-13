// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module implements an instantiation of a verifiable random function known as
//! [ECVRF-ED25519-SHA512-TAI](https://tools.ietf.org/html/draft-irtf-cfrg-vrf-04).
//!
//! # Examples
//!
//! ```
//! use libra_crypto::{traits::Uniform, vrf::ecvrf::*};
//! use rand::{rngs::StdRng, SeedableRng};
//!
//! let message = b"Test message";
//! let mut rng: StdRng = SeedableRng::from_seed([0; 32]);
//! let private_key = VRFPrivateKey::generate_for_testing(&mut rng);
//! let public_key: VRFPublicKey = (&private_key).into();
//! ```
//! **Note**: The above example generates a private key using a private function intended only for
//! testing purposes. Production code should find an alternate means for secure key generation.
//!
//! Produce a proof for a message from a `VRFPrivateKey`, and verify the proof and message
//! using a `VRFPublicKey`:
//!
//! ```
//! # use libra_crypto::{traits::Uniform, vrf::ecvrf::*};
//! # use rand::{rngs::StdRng, SeedableRng};
//! # let message = b"Test message";
//! # let mut rng: StdRng = SeedableRng::from_seed([0; 32]);
//! # let private_key = VRFPrivateKey::generate_for_testing(&mut rng);
//! # let public_key: VRFPublicKey = (&private_key).into();
//! let proof = private_key.prove(message);
//! assert!(public_key.verify(&proof, message).is_ok());
//! ```
//!
//! Produce a pseudorandom output from a `Proof`:
//!
//! ```
//! # use libra_crypto::{traits::Uniform, vrf::ecvrf::*};
//! # use rand::{rngs::StdRng, SeedableRng};
//! # let message = b"Test message";
//! # let mut rng: StdRng = SeedableRng::from_seed([0; 32]);
//! # let private_key = VRFPrivateKey::generate_for_testing(&mut rng);
//! # let public_key: VRFPublicKey = (&private_key).into();
//! # let proof = private_key.prove(message);
//! let output: Output = (&proof).into();
//! ```

use crate::traits::*;
use anyhow::{bail, Result};
use core::convert::TryFrom;
use curve25519_dalek::{
    constants::ED25519_BASEPOINT_POINT,
    edwards::{CompressedEdwardsY, EdwardsPoint},
    scalar::Scalar as ed25519_Scalar,
};
use ed25519_dalek::{
    self, Digest, PublicKey as ed25519_PublicKey, SecretKey as ed25519_PrivateKey, Sha512,
};
use libra_crypto_derive::Deref;
use serde::{Deserialize, Serialize};

const SUITE: u8 = 0x03;
const ONE: u8 = 0x01;
const TWO: u8 = 0x02;
const THREE: u8 = 0x03;

/// The number of bytes of [`Output`]
pub const OUTPUT_LENGTH: usize = 64;
/// The number of bytes of [`Proof`]
pub const PROOF_LENGTH: usize = 80;

/// An ECVRF private key
#[derive(Serialize, Deserialize, Deref, Debug)]
pub struct VRFPrivateKey(ed25519_PrivateKey);

/// An ECVRF public key
#[derive(Serialize, Deserialize, Debug, Deref, PartialEq, Eq)]
pub struct VRFPublicKey(ed25519_PublicKey);

/// A longer private key which is slightly optimized for proof generation.
///
/// This is similar in structure to ed25519_dalek::ExpandedSecretKey. It can be produced from
/// a VRFPrivateKey.
pub struct VRFExpandedPrivateKey {
    pub(super) key: ed25519_Scalar,
    pub(super) nonce: [u8; 32],
}

impl VRFPrivateKey {
    /// Produces a proof for an input (using the private key)
    pub fn prove(&self, alpha: &[u8]) -> Proof {
        VRFExpandedPrivateKey::from(self).prove(&VRFPublicKey((&self.0).into()), alpha)
    }
}

impl VRFExpandedPrivateKey {
    /// Produces a proof for an input (using the expanded private key)
    pub fn prove(&self, pk: &VRFPublicKey, alpha: &[u8]) -> Proof {
        let h_point = pk.hash_to_curve(alpha);
        let k_scalar =
            ed25519_Scalar::from_bytes_mod_order_wide(&nonce_generation_bytes(self.nonce, h_point));
        let gamma = h_point * self.key;
        let c_scalar = hash_points(&[
            h_point,
            gamma,
            ED25519_BASEPOINT_POINT * k_scalar,
            h_point * k_scalar,
        ]);

        Proof {
            gamma,
            c: c_scalar,
            s: k_scalar + c_scalar * self.key,
        }
    }
}

impl Uniform for VRFPrivateKey {
    fn generate_for_testing<R>(rng: &mut R) -> Self
    where
        R: ::rand::SeedableRng + ::rand::RngCore + ::rand::CryptoRng,
    {
        VRFPrivateKey(ed25519_PrivateKey::generate(rng))
    }
}

impl TryFrom<&[u8]> for VRFPrivateKey {
    type Error = CryptoMaterialError;

    fn try_from(bytes: &[u8]) -> std::result::Result<VRFPrivateKey, CryptoMaterialError> {
        Ok(VRFPrivateKey(
            ed25519_PrivateKey::from_bytes(bytes).unwrap(),
        ))
    }
}

impl TryFrom<&[u8]> for VRFPublicKey {
    type Error = CryptoMaterialError;

    fn try_from(bytes: &[u8]) -> std::result::Result<VRFPublicKey, CryptoMaterialError> {
        if bytes.len() != ed25519_dalek::PUBLIC_KEY_LENGTH {
            return Err(CryptoMaterialError::WrongLengthError);
        }

        let mut bits: [u8; 32] = [0u8; 32];
        bits.copy_from_slice(&bytes[..32]);

        let compressed = curve25519_dalek::edwards::CompressedEdwardsY(bits);
        let point = compressed
            .decompress()
            .ok_or(CryptoMaterialError::DeserializationError)?;

        // Check if the point lies on a small subgroup. This is required
        // when using curves with a small cofactor (in ed25519, cofactor = 8).
        if point.is_small_order() {
            return Err(CryptoMaterialError::SmallSubgroupError);
        }

        Ok(VRFPublicKey(ed25519_PublicKey::from_bytes(bytes).unwrap()))
    }
}

impl VRFPublicKey {
    /// Given a [`Proof`] and an input, returns whether or not the proof is valid for the input
    /// and public key
    pub fn verify(&self, proof: &Proof, alpha: &[u8]) -> Result<()> {
        let h_point = self.hash_to_curve(alpha);
        let pk_point = CompressedEdwardsY::from_slice(self.as_bytes())
            .decompress()
            .unwrap();
        let cprime = hash_points(&[
            h_point,
            proof.gamma,
            ED25519_BASEPOINT_POINT * proof.s - pk_point * proof.c,
            h_point * proof.s - proof.gamma * proof.c,
        ]);

        if proof.c == cprime {
            Ok(())
        } else {
            bail!("The proof failed to verify for this public key")
        }
    }

    pub(super) fn hash_to_curve(&self, alpha: &[u8]) -> EdwardsPoint {
        let mut result = [0u8; 32];
        let mut counter = 0;
        let mut wrapped_point: Option<EdwardsPoint> = None;

        while wrapped_point.is_none() {
            result.copy_from_slice(
                &Sha512::new()
                    .chain(&[SUITE, ONE])
                    .chain(self.as_bytes())
                    .chain(&alpha)
                    .chain(&[counter])
                    .result()[..32],
            );
            wrapped_point = CompressedEdwardsY::from_slice(&result).decompress();
            counter += 1;
        }

        wrapped_point.unwrap().mul_by_cofactor()
    }
}

impl<'a> From<&'a VRFPrivateKey> for VRFPublicKey {
    fn from(private_key: &'a VRFPrivateKey) -> Self {
        let secret: &ed25519_PrivateKey = private_key;
        let public: ed25519_PublicKey = secret.into();
        VRFPublicKey(public)
    }
}

impl<'a> From<&'a VRFPrivateKey> for VRFExpandedPrivateKey {
    fn from(private_key: &'a VRFPrivateKey) -> Self {
        let mut h: Sha512 = Sha512::default();
        let mut hash: [u8; 64] = [0u8; 64];
        let mut lower: [u8; 32] = [0u8; 32];
        let mut upper: [u8; 32] = [0u8; 32];

        h.input(private_key.to_bytes());
        hash.copy_from_slice(h.result().as_slice());

        lower.copy_from_slice(&hash[00..32]);
        upper.copy_from_slice(&hash[32..64]);

        lower[0] &= 248;
        lower[31] &= 63;
        lower[31] |= 64;

        VRFExpandedPrivateKey {
            key: ed25519_Scalar::from_bits(lower),
            nonce: upper,
        }
    }
}

/// A VRF proof that can be used to validate an input with a public key
pub struct Proof {
    gamma: EdwardsPoint,
    c: ed25519_Scalar,
    s: ed25519_Scalar,
}

impl Proof {
    /// Produces a new Proof struct from its fields
    pub fn new(gamma: EdwardsPoint, c: ed25519_Scalar, s: ed25519_Scalar) -> Proof {
        Proof { gamma, c, s }
    }

    /// Converts a Proof into bytes
    pub fn to_bytes(&self) -> [u8; PROOF_LENGTH] {
        let mut ret = [0u8; PROOF_LENGTH];
        ret[..32].copy_from_slice(&self.gamma.compress().to_bytes()[..]);
        ret[32..48].copy_from_slice(&self.c.to_bytes()[..16]);
        ret[48..].copy_from_slice(&self.s.to_bytes()[..]);
        ret
    }
}

impl TryFrom<&[u8]> for Proof {
    type Error = CryptoMaterialError;

    fn try_from(bytes: &[u8]) -> std::result::Result<Proof, CryptoMaterialError> {
        let mut c_buf = [0u8; 32];
        c_buf[..16].copy_from_slice(&bytes[32..48]);
        let mut s_buf = [0u8; 32];
        s_buf.copy_from_slice(&bytes[48..]);
        Ok(Proof {
            gamma: CompressedEdwardsY::from_slice(&bytes[..32])
                .decompress()
                .unwrap(),
            c: ed25519_Scalar::from_bits(c_buf),
            s: ed25519_Scalar::from_bits(s_buf),
        })
    }
}

/// The ECVRF output produced from the proof
pub struct Output([u8; OUTPUT_LENGTH]);

impl Output {
    /// Converts an Output into bytes
    #[inline]
    pub fn to_bytes(&self) -> [u8; OUTPUT_LENGTH] {
        self.0
    }
}

impl<'a> From<&'a Proof> for Output {
    fn from(proof: &'a Proof) -> Output {
        let mut output = [0u8; OUTPUT_LENGTH];
        output.copy_from_slice(
            &Sha512::new()
                .chain(&[SUITE, THREE])
                .chain(&proof.gamma.mul_by_cofactor().compress().to_bytes()[..])
                .result()[..],
        );
        Output(output)
    }
}

pub(super) fn nonce_generation_bytes(nonce: [u8; 32], h_point: EdwardsPoint) -> [u8; 64] {
    let mut k_buf = [0u8; 64];
    k_buf.copy_from_slice(
        &Sha512::new()
            .chain(nonce)
            .chain(h_point.compress().as_bytes())
            .result()[..],
    );
    k_buf
}

pub(super) fn hash_points(points: &[EdwardsPoint]) -> ed25519_Scalar {
    let mut result = [0u8; 32];
    let mut hash = Sha512::new().chain(&[SUITE, TWO]);
    for point in points.iter() {
        hash = hash.chain(point.compress().to_bytes());
    }
    result[..16].copy_from_slice(&hash.result()[..16]);
    ed25519_Scalar::from_bits(result)
}
