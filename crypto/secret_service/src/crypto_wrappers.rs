// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module contains wrappers around nextgen crypto API to support crypto agility
//! and to make the secret service agnostic to the details of the particular signing algorithms.

use core::convert::TryFrom;
use crypto::{
    bls12381::{BLS12381PrivateKey, BLS12381PublicKey, BLS12381Signature},
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey, Ed25519Signature},
    hash::HashValue,
    traits::*,
};
use crypto_derive::SilentDebug;
use failure::prelude::*;
use serde::{Deserialize, Serialize};
use std::ops::Deref;

/// KeyID value is a handler to the secret key and a simple wrapper around the hash value.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct KeyID(pub HashValue);

impl Deref for KeyID {
    type Target = HashValue;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

///////////////////////////////////////////////////////////////////
// Declarations pulled from crypto/src/unit_tests/cross_test.rs  //
///////////////////////////////////////////////////////////////////
/// Generic public key enum
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GenericPublicKey {
    /// Ed25519 public key
    Ed(Ed25519PublicKey),
    /// BLS12-381 public key
    BLS(BLS12381PublicKey),
}

/// Generic private key enum
#[derive(Serialize, Deserialize, SilentDebug)]
pub enum GenericPrivateKey {
    /// Ed25519 private key
    Ed(Ed25519PrivateKey),
    /// BLS12-381 private key
    BLS(BLS12381PrivateKey),
}

/// Generic signature enum
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum GenericSignature {
    /// Ed25519 signature
    Ed(Ed25519Signature),
    /// BLS12-381 signature
    BLS(BLS12381Signature),
}

impl GenericSignature {
    /// Convert the signature to bytes for serialization
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            GenericSignature::Ed(sig) => sig.to_bytes().to_vec(),
            GenericSignature::BLS(sig) => sig.to_bytes().to_vec(),
        }
    }
}

impl From<&GenericPrivateKey> for GenericPublicKey {
    fn from(secret_key: &GenericPrivateKey) -> Self {
        match secret_key {
            GenericPrivateKey::Ed(pk) => GenericPublicKey::Ed(pk.into()),
            GenericPrivateKey::BLS(pk) => GenericPublicKey::BLS(pk.into()),
        }
    }
}

impl TryFrom<&[u8]> for GenericPrivateKey {
    type Error = CryptoMaterialError;
    fn try_from(bytes: &[u8]) -> std::result::Result<GenericPrivateKey, CryptoMaterialError> {
        Ed25519PrivateKey::try_from(bytes)
            .and_then(|ed_priv_key| Ok(GenericPrivateKey::Ed(ed_priv_key)))
            .or_else(|_err| {
                BLS12381PrivateKey::try_from(bytes)
                    .and_then(|bls_priv_key| Ok(GenericPrivateKey::BLS(bls_priv_key)))
            })
    }
}

impl ValidKey for GenericPrivateKey {
    fn to_bytes(&self) -> Vec<u8> {
        match self {
            GenericPrivateKey::BLS(privkey) => privkey.to_bytes().to_vec(),
            GenericPrivateKey::Ed(privkey) => privkey.to_bytes().to_vec(),
        }
    }
}

impl PublicKey for GenericPublicKey {
    type PrivateKeyMaterial = GenericPrivateKey;
}

impl TryFrom<&[u8]> for GenericPublicKey {
    type Error = CryptoMaterialError;
    fn try_from(bytes: &[u8]) -> std::result::Result<GenericPublicKey, CryptoMaterialError> {
        Ed25519PublicKey::try_from(bytes)
            .and_then(|ed_priv_key| Ok(GenericPublicKey::Ed(ed_priv_key)))
            .or_else(|_err| {
                BLS12381PublicKey::try_from(bytes)
                    .and_then(|bls_priv_key| Ok(GenericPublicKey::BLS(bls_priv_key)))
            })
    }
}

impl ValidKey for GenericPublicKey {
    fn to_bytes(&self) -> Vec<u8> {
        match self {
            GenericPublicKey::BLS(pubkey) => pubkey.to_bytes().to_vec(),
            GenericPublicKey::Ed(pubkey) => pubkey.to_bytes().to_vec(),
        }
    }
}

impl PrivateKey for GenericPrivateKey {
    type PublicKeyMaterial = GenericPublicKey;
}

impl SigningKey for GenericPrivateKey {
    type VerifyingKeyMaterial = GenericPublicKey;
    type SignatureMaterial = GenericSignature;

    fn sign_message(&self, message: &HashValue) -> GenericSignature {
        match self {
            GenericPrivateKey::Ed(ed_priv) => GenericSignature::Ed(ed_priv.sign_message(message)),
            GenericPrivateKey::BLS(bls_priv) => {
                GenericSignature::BLS(bls_priv.sign_message(message))
            }
        }
    }
}

impl Signature for GenericSignature {
    type VerifyingKeyMaterial = GenericPublicKey;
    type SigningKeyMaterial = GenericPrivateKey;

    fn verify(&self, message: &HashValue, public_key: &GenericPublicKey) -> Result<()> {
        self.verify_arbitrary_msg(message.as_ref(), public_key)
    }

    fn verify_arbitrary_msg(&self, message: &[u8], public_key: &GenericPublicKey) -> Result<()> {
        match (self, public_key) {
            (GenericSignature::Ed(ed_sig), GenericPublicKey::Ed(ed_pub)) => {
                ed_sig.verify_arbitrary_msg(message, ed_pub)
            }
            (GenericSignature::BLS(bls_sig), GenericPublicKey::BLS(bls_pub)) => {
                bls_sig.verify_arbitrary_msg(message, bls_pub)
            }
            _ => bail!(
                "provided the wrong alternative in {:?}!",
                (self, public_key)
            ),
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        match self {
            GenericSignature::Ed(sig) => sig.to_bytes().to_vec(),
            GenericSignature::BLS(sig) => sig.to_bytes().to_vec(),
        }
    }
}

impl TryFrom<&[u8]> for GenericSignature {
    type Error = CryptoMaterialError;
    fn try_from(bytes: &[u8]) -> std::result::Result<GenericSignature, CryptoMaterialError> {
        Ed25519Signature::try_from(bytes)
            .and_then(|ed_sig| Ok(GenericSignature::Ed(ed_sig)))
            .or_else(|_err| {
                BLS12381Signature::try_from(bytes)
                    .and_then(|bls_sig| Ok(GenericSignature::BLS(bls_sig)))
            })
    }
}

impl VerifyingKey for GenericPublicKey {
    type SigningKeyMaterial = GenericPrivateKey;
    type SignatureMaterial = GenericSignature;
}
