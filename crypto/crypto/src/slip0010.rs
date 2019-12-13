// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module provides an API for SLIP-0010 and for the Ed25519 curve based on
//! [SLIP-0010 : Universal private key derivation from master private key](https://github.com/satoshilabs/slips/blob/master/slip-0010.md).
//!
//! SLIP-0010 describes how to derive a master private/public key for various curves and how a
//! [BIP-0032](https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki) like derivation is used to hierarchically generate keys from an original seed.
//!
//! Note that SLIP-0010 only supports private parent key → private child key hardened key generation
//! for Ed25519. This key derivation protocol should be preferred to HKDF if compatibility with
//! hardware wallets and the BIP32 protocol is required.
use byteorder::{BigEndian, ByteOrder};
use ed25519_dalek::{PublicKey, SecretKey};
use hmac::{Hmac, Mac};
use sha2::Sha512;
use thiserror::Error;

/// Extended private key that includes additional child_path and chain-code.
pub struct ExtendedPrivKey {
    /// Full key path.
    key_path: String,
    /// Private key.
    private_key: SecretKey,
    /// Chain code.
    chain_code: [u8; 32],
}

impl ExtendedPrivKey {
    /// Construct a new ExtendedPrivKey.
    pub fn new(
        key_path: String,
        private_key: SecretKey,
        chain_code: [u8; 32],
    ) -> Result<Self, Slip0010Error> {
        if Slip0010::is_valid_path(&key_path) {
            Ok(Self {
                key_path,
                private_key,
                chain_code,
            })
        } else {
            Err(Slip0010Error::KeyPathError)
        }
    }

    /// Get public key.
    pub fn get_public(&self) -> PublicKey {
        (&self.private_key).into()
    }

    /// Get key path.
    pub fn get_key_path(&self) -> &str {
        &self.key_path
    }

    /// Get private key.
    pub fn get_private(&self) -> &SecretKey {
        &self.private_key
    }

    /// Get chain code.
    pub fn get_chain_code(&self) -> &[u8; 32] {
        &self.chain_code
    }

    /// Derive a child key from this key and a child number.
    fn child_key(&self, child_number: u32) -> Result<Self, Slip0010Error> {
        let hardened_child_number = if child_number < Slip0010::HARDENED_START {
            child_number + Slip0010::HARDENED_START
        } else {
            child_number
        };

        let mut hmac =
            Hmac::<Sha512>::new_varkey(&self.chain_code).map_err(|_| Slip0010Error::MACKeyError)?;

        let mut be_n = [0u8; 4];
        BigEndian::write_u32(&mut be_n, hardened_child_number);

        hmac.input(&[0u8]);
        hmac.input(self.private_key.as_bytes());
        hmac.input(&be_n);

        let hmac_output = hmac.result_reset().code();

        let mut chain_code_bits: [u8; 32] = [0u8; 32];
        chain_code_bits.copy_from_slice(&hmac_output[32..]);

        let mut new_child_path = self.key_path.to_owned();
        new_child_path.push_str("/");
        new_child_path.push_str(&*child_number.to_string());

        let secret_key =
            SecretKey::from_bytes(&hmac_output[..32]).map_err(|_| Slip0010Error::SecretKeyError)?;

        ExtendedPrivKey::new(new_child_path, secret_key, chain_code_bits)
    }
}

/// SLIP-0010 structure.
pub struct Slip0010 {}

impl Slip0010 {
    /// Curve = "ed25519 seed" for the ed25519 curve.
    const ED25519_CURVE: &'static [u8] = b"ed25519 seed";
    /// Hardened keys start from 2^31 = 2147483648.
    const HARDENED_START: u32 = 2_147_483_648;

    /// Generate master key from seed.
    pub fn generate_master(seed: &[u8]) -> Result<ExtendedPrivKey, Slip0010Error> {
        let mut hmac = Hmac::<Sha512>::new_varkey(&Slip0010::ED25519_CURVE)
            .map_err(|_| Slip0010Error::MACKeyError)?;
        hmac.input(seed);
        let hmac_output = hmac.result_reset().code();

        let mut chain_code_bits: [u8; 32] = [0u8; 32];
        chain_code_bits.copy_from_slice(&hmac_output[32..]);

        let secret_key =
            SecretKey::from_bytes(&hmac_output[..32]).map_err(|_| Slip0010Error::SecretKeyError)?;

        ExtendedPrivKey::new("m".to_string(), secret_key, chain_code_bits)
    }

    /// Generate a child private key.
    pub fn derive_child_key(
        parent_key: ExtendedPrivKey,
        child_number: u32,
    ) -> Result<ExtendedPrivKey, Slip0010Error> {
        parent_key.child_key(child_number)
    }

    /// Match a valid path of the form "m/A/B.."; each sub-path after m is smaller than 2147483648.
    pub fn is_valid_path(path: &str) -> bool {
        let mut segments = path.split('/');
        if segments.next() != Some("m") {
            return false;
        }
        segments.all(|s| {
            if !s.starts_with('+') {
                if let Ok(num) = s.parse::<u32>() {
                    num < 2_147_483_648
                } else {
                    false
                }
            } else {
                false
            }
        })
    }

    /// Derive a key from a path and a seed.
    pub fn derive_from_path(path: &str, seed: &[u8]) -> Result<ExtendedPrivKey, Slip0010Error> {
        if !Slip0010::is_valid_path(path) {
            return Err(Slip0010Error::KeyPathError);
        }

        let mut key = Slip0010::generate_master(seed)?;

        let segments: Vec<&str> = path.split('/').collect();
        let segments = segments
            .iter()
            .skip(1)
            .map(|s| s.replace("'", ""))
            // We first check if the path is valid, so this will never fail.
            .map(|s| s.parse::<u32>().unwrap())
            .collect::<Vec<_>>();

        for segment in segments {
            key = Slip0010::derive_child_key(key, segment)?;
        }

        Ok(key)
    }
}

/// An error type for SLIP-0010 key derivation issues.
///
/// This enum reflects there are various causes of SLIP-0010 failures, including:
/// a) invalid key-path.
/// b) secret_key generation errors.
/// c) hmac related errors.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum Slip0010Error {
    /// Invalid key path.
    #[error("SLIP-0010 invalid key path")]
    KeyPathError,
    /// Any error related to key derivation.
    #[error("SLIP-0010 - cannot generate key")]
    SecretKeyError,
    /// HMAC key related error; unlikely to happen because every key size is accepted in HMAC.
    #[error("SLIP-0010 - HMAC key error")]
    MACKeyError,
}
