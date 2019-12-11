// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! An implementation of HKDF, the HMAC-based Extract-and-Expand Key Derivation Function for the
//! Libra project based on [RFC 5869](https://tools.ietf.org/html/rfc5869).
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
//! for generating keys in Libra, especially when a True Random Generator is not available.
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
//! use libra_crypto::hkdf::Hkdf;
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
    generic_array::{self, ArrayLength, GenericArray},
    BlockInput, FixedOutput, Input, Reset,
};
use generic_array::typenum::Unsigned;
use hmac::{Hmac, Mac};
use std::marker::PhantomData;
use thiserror::Error;

/// Structure representing the HKDF, capable of HKDF-Extract and HKDF-Expand operations, as defined
/// in RFC 5869.
#[derive(Clone, Debug)]
pub struct Hkdf<D>
where
    D: Input + BlockInput + FixedOutput + Reset + Default + Clone,
    D::OutputSize: ArrayLength<u8>,
{
    _marker: PhantomData<D>,
}

impl<D> Hkdf<D>
where
    D: Input + BlockInput + FixedOutput + Reset + Default + Clone,
    D::BlockSize: ArrayLength<u8> + Clone,
    D::OutputSize: ArrayLength<u8>,
{
    /// Minimum acceptable output length for the underlying hash function is 32 bytes.
    const D_MINIMUM_SIZE: usize = 32;

    /// The RFC5869 HKDF-Extract operation.
    pub fn extract(salt: Option<&[u8]>, ikm: &[u8]) -> Result<Vec<u8>, HkdfError> {
        let d_output_size = D::OutputSize::to_usize();
        if d_output_size < Hkdf::<D>::D_MINIMUM_SIZE {
            return Err(HkdfError::NotSupportedHashFunctionError);
        }

        let mut hmac = match salt {
            Some(s) => Hmac::<D>::new_varkey(s).map_err(|_| HkdfError::MACKeyError)?,
            None => Hmac::<D>::new(&Default::default()),
        };

        hmac.input(ikm);

        Ok(hmac.result().code().to_vec())
    }

    /// The RFC5869 HKDF-Expand operation.
    pub fn expand(prk: &[u8], info: Option<&[u8]>, length: usize) -> Result<Vec<u8>, HkdfError> {
        let hmac_output_bytes = D::OutputSize::to_usize();
        if prk.len() < hmac_output_bytes {
            return Err(HkdfError::WrongPseudorandomKeyError);
        }
        // According to RFC5869, MAX_OUTPUT_LENGTH <= 255 * HashLen.
        // We specifically exclude zero size as well.
        if length == 0 || length > hmac_output_bytes * 255 {
            return Err(HkdfError::InvalidOutputLengthError);
        }
        let mut okm = vec![0u8; length];
        let mut prev: Option<GenericArray<u8, <D as digest::FixedOutput>::OutputSize>> = None;
        let mut hmac = Hmac::<D>::new_varkey(prk).map_err(|_| HkdfError::MACKeyError)?;

        for (blocknum, okm_block) in okm.chunks_mut(hmac_output_bytes).enumerate() {
            if let Some(ref prev) = prev {
                hmac.input(prev)
            }
            if let Some(_info) = info {
                hmac.input(_info);
            }
            hmac.input(&[blocknum as u8 + 1]);

            let output = hmac.result_reset().code();
            okm_block.copy_from_slice(&output[..okm_block.len()]);

            prev = Some(output);
        }

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
    /// Hash function is not supported because its output is less than 32 bits.
    #[error(
        "HKDF error - the hash function is not supported because \
         its output is less than 32 bits"
    )]
    NotSupportedHashFunctionError,
    /// PRK on HKDF-Expand should not be less than the underlying hash output bits.
    #[error(
        "HKDF expand error - the pseudorandom key input ('prk' in RFC 5869) \
         is less than the underlying hash output bits"
    )]
    WrongPseudorandomKeyError,
    /// HMAC key related error; unlikely to happen because every key size is accepted in HMAC.
    #[error("HMAC key error")]
    MACKeyError,
}
