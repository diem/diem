// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! An implementation of HKDF, the HMAC-based Extract-and-Expand Key Derivation Function for the
//! Diem project based on [RFC 5869](https://tools.ietf.org/html/rfc5869).
//!
//! The key derivation function (KDF) is intended to support a wide range of applications and
//! requirements, and is conservative in its use of cryptographic hash functions. In particular,
//! this implementation is compatible with hash functions that output 256 bits or more, such as
//! SHA256, SHA3-256 and SHA512.
//!
//! HKDF follows the "extract-then-expand" paradigm, where the KDF logically consists of two
//! modules: the first stage takes the input keying material (the seed) and "extracts" from it a
//! fixed-length pseudorandom key, and then the second stage "expands" this key into several
//! additional pseudorandom keys (the output of the KDF). For convenience, a function that runs both
//! steps in a single call is provided. Note that along with an initial high-entropy seed, a user
//! can optionally provide salt and app-info byte-arrays for extra security guarantees and domain
//! separation.
//!
//! # Applications
//!
//! HKDF is intended for use in a wide variety of KDF applications (see [Key derivation function](https://en.wikipedia.org/wiki/Key_derivation_function)), including:
//! a) derivation of keys from an origin high-entropy master seed. This is the recommended approach
//! for generating keys in Diem, especially when a True Random Generator is not available.
//! b) derivation of session keys from a shared Diffie-Hellman value in a key-agreement protocol.
//! c) combining entropy from multiple sources of randomness, such as entropy collected
//! from system events, user's keystrokes, /dev/urandom etc. The combined seed can then be used to
//! generate cryptographic keys for account, network and transaction signing keys among the others.
//! d) hierarchical private key derivation, similarly to Bitcoin's BIP32 protocol for easier key
//! management.
//! e) hybrid key generation that combines a master seed with a PRNG output for extra security
//! guarantees against a master seed leak or low PRNG entropy.
//!
//! # Recommendations
//!
//! **Salt**
//! HKDF can operate with and without random 'salt'. The use of salt adds to the strength of HKDF,
//! ensuring independence between different uses of the hash function, supporting
//! "source-independent" extraction, and strengthening the HKDF use. The salt value should be a
//! random string of the same length as the hash output. A shorter or less random salt value can
//! still make a contribution to the security of the output key material. Salt values should be
//! independent of the input keying material. In particular, an application needs to make sure that
//! salt values are not chosen or manipulated by an attacker.
//!
//! *Application info*
//! Key expansion accepts an optional 'info' value to which the application assigns some meaning.
//! Its objective is to bind the derived key material to application- and context-specific
//! information.  For example, 'info' may contain a protocol number, algorithm identifier,
//! child key number (similarly to [BIP32](https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki)), etc. The only technical requirement for 'info' is that
//! it be independent of the seed.
//!
//! **Which function to use: extract, expand or both?**
//! Unless absolutely sure of what they are doing, applications should use both steps — if only for
//! the sake of compatibility with the general case.
//!
//! # Example
//!
//! Run HKDF extract-then-expand so as to return 64 bytes, using 'salt', 'seed' and 'info' as
//! inputs.
//! ```
//! use diem_crypto::hkdf::Hkdf;
//! use sha2::Sha256;
//!
//! // some bytes required for this example.
//! let raw_bytes = [2u8; 10];
//! // define salt
//! let salt = Some(&raw_bytes[0..4]);
//! // define seed - in production this is recommended to be a 32 bytes or longer random seed.
//! let seed = [3u8; 32];
//! // define application info
//! let info = Some(&raw_bytes[4..10]);
//!
//! // HKDF extract-then-expand 64-bytes output
//! let derived_bytes = Hkdf::<Sha256>::extract_then_expand(salt, &seed, info, 64);
//! assert_eq!(derived_bytes.unwrap().len(), 64)
//! ```

use digest::{
    generic_array::{self, ArrayLength},
    BlockInput, FixedOutput, Reset, Update,
};

use generic_array::typenum::{IsGreaterOrEqual, True, U32};

use std::marker::PhantomData;
use thiserror::Error;

/// Hash function is not supported if its output is less than 32 bits.
type DMinimumSize = U32;

/// Seed (ikm = initial key material) is not accepted if its size is less than 16 bytes. This is a
/// precautionary measure to prevent HKDF misuse. 128 bits is the minimum accepted seed entropy
/// length in the majority of today's applications to avoid brute forcing.
/// Note that for Ed25519 keys, random seeds of at least 32 bytes are recommended.
const MINIMUM_SEED_LENGTH: usize = 16;

/// Structure representing the HKDF, capable of HKDF-Extract and HKDF-Expand operations, as defined
/// in RFC 5869.
#[derive(Clone, Debug)]
pub struct Hkdf<D>
where
    D: Update + BlockInput + FixedOutput + Reset + Default + Clone,
    D::BlockSize: ArrayLength<u8>,
    D::OutputSize: ArrayLength<u8>,
    D::OutputSize: IsGreaterOrEqual<DMinimumSize, Output = True>,
{
    _marker: PhantomData<D>,
}

impl<D> Hkdf<D>
where
    D: Update + BlockInput + FixedOutput + Reset + Default + Clone,
    D::BlockSize: ArrayLength<u8> + Clone,
    D::OutputSize: ArrayLength<u8>,
    D::OutputSize: IsGreaterOrEqual<DMinimumSize, Output = True>,
{
    /// The RFC5869 HKDF-Extract operation.
    pub fn extract(salt: Option<&[u8]>, ikm: &[u8]) -> Result<Vec<u8>, HkdfError> {
        if ikm.len() < MINIMUM_SEED_LENGTH {
            return Err(HkdfError::InvalidSeedLengthError);
        }
        Ok(Hkdf::<D>::extract_no_ikm_check(salt, ikm))
    }

    fn extract_no_ikm_check(salt: Option<&[u8]>, ikm: &[u8]) -> Vec<u8> {
        let (arr, _hkdf) = hkdf::Hkdf::<D>::extract(salt, ikm);
        arr.to_vec()
    }

    /// The RFC5869 HKDF-Expand operation.
    pub fn expand(prk: &[u8], info: Option<&[u8]>, length: usize) -> Result<Vec<u8>, HkdfError> {
        // According to RFC5869, MAX_OUTPUT_LENGTH <= 255 * HashLen — which is
        // checked below.
        // We specifically exclude a zero size length as well.
        if length == 0 {
            return Err(HkdfError::InvalidOutputLengthError);
        }

        let hkdf =
            hkdf::Hkdf::<D>::from_prk(prk).map_err(|_| HkdfError::WrongPseudorandomKeyError)?;
        let mut okm = vec![0u8; length];
        hkdf.expand(info.unwrap_or_else(|| &[]), &mut okm)
            // length > D::OutputSize::to_usize() * 255
            .map_err(|_| HkdfError::InvalidOutputLengthError)?;
        Ok(okm)
    }

    /// HKDF Extract then Expand operation as a single step.
    pub fn extract_then_expand(
        salt: Option<&[u8]>,
        ikm: &[u8],
        info: Option<&[u8]>,
        length: usize,
    ) -> Result<Vec<u8>, HkdfError> {
        let prk = Hkdf::<D>::extract(salt, ikm)?;
        Hkdf::<D>::expand(&prk, info, length)
    }

    /// CAUTION: This is not recommended because it does not take an ikm (seed) as an input and
    /// thus, it is not fully compliant with the HKDF RFC (which always expects a non-zero ikm).
    /// Please use `extract_then_expand` instead, unless you know what you are doing.
    ///
    /// This api is currently required by the Noise protocol [HKDF specs](https://noiseprotocol.org/noise.html#hash-functions).
    ///
    /// HKDF Extract then Expand operation as a single step, but without an ikm input.
    pub fn extract_then_expand_no_ikm(
        salt: Option<&[u8]>,
        info: Option<&[u8]>,
        length: usize,
    ) -> Result<Vec<u8>, HkdfError> {
        let prk = Hkdf::<D>::extract_no_ikm_check(salt, &[]);
        Hkdf::<D>::expand(&prk, info, length)
    }
}

/// An error type for HKDF key derivation issues.
///
/// This enum reflects there are various causes of HKDF failures, including:
/// a) requested HKDF output size exceeds the maximum allowed or is zero.
/// b) hash functions outputting less than 32 bits are not supported (i.e., SHA1 is not supported).
/// c) small PRK value in HKDF-Expand according to RFC 5869.
/// d) any other underlying HMAC error.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum HkdfError {
    /// HKDF expand output exceeds the maximum allowed or is zero.
    #[error("HKDF expand error - requested output size exceeds the maximum allowed or is zero")]
    InvalidOutputLengthError,
    /// PRK on HKDF-Expand should not be less than the underlying hash output bits.
    #[error(
        "HKDF expand error - the pseudorandom key input ('prk' in RFC 5869) \
         is less than the underlying hash output bits"
    )]
    WrongPseudorandomKeyError,
    /// HMAC key related error; unlikely to happen because every key size is accepted in HMAC.
    #[error("HMAC key error")]
    MACKeyError,
    /// HKDF extract input seed should not be less than the minimum accepted.
    #[error("HKDF extract error - input seed is too small")]
    InvalidSeedLengthError,
}
