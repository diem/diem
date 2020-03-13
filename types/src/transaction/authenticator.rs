// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_address::AuthenticationKey;
use libra_crypto::{
    ed25519::{Ed25519PublicKey, Ed25519Signature},
    multi_ed25519::{MultiEd25519PublicKey, MultiEd25519Signature},
    traits::Signature,
    HashValue,
};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, fmt};

// TODO: in the future, can tie these to the enum with https://github.com/rust-lang/rust/issues/60553
const ED25519_SCHEME: u8 = 0;
const MULTI_ED25519_SCHEME: u8 = 1;

pub struct AuthenticationKeyPreimage(pub Vec<u8>);

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum TransactionAuthenticator {
    /// Single signature
    Ed25519 {
        public_key: Ed25519PublicKey,
        signature: Ed25519Signature,
    },
    /// K-of-N multisignature
    MultiEd25519 {
        public_key: MultiEd25519PublicKey,
        signature: MultiEd25519Signature,
    },
    // ... add more schemes here
}

impl TransactionAuthenticator {
    /// Unique identifier for the signature scheme
    pub fn scheme_id(&self) -> u8 {
        match self {
            Self::Ed25519 { .. } => ED25519_SCHEME,
            Self::MultiEd25519 { .. } => MULTI_ED25519_SCHEME,
        }
    }

    /// Create a single-signature ed25519 authenticator
    pub fn ed25519(public_key: Ed25519PublicKey, signature: Ed25519Signature) -> Self {
        Self::Ed25519 {
            public_key,
            signature,
        }
    }

    /// Create a multisignature ed25519 authenticator
    pub fn multi_ed25519(
        public_key: MultiEd25519PublicKey,
        signature: MultiEd25519Signature,
    ) -> Self {
        Self::MultiEd25519 {
            public_key,
            signature,
        }
    }

    /// Return Ok if the authenticator's public key matches its signature, Err otherwise
    pub fn verify_signature(&self, message: &HashValue) -> Result<()> {
        match self {
            Self::Ed25519 {
                public_key,
                signature,
            } => signature.verify(message, public_key),
            Self::MultiEd25519 {
                public_key,
                signature,
            } => signature.verify(message, public_key),
        }
    }

    pub fn public_key_bytes(&self) -> Vec<u8> {
        match self {
            Self::Ed25519 { public_key, .. } => public_key.to_bytes().to_vec(),
            Self::MultiEd25519 { public_key, .. } => public_key.to_bytes().to_vec(),
        }
    }

    pub fn signature_bytes(&self) -> Vec<u8> {
        match self {
            Self::Ed25519 { signature, .. } => signature.to_bytes().to_vec(),
            Self::MultiEd25519 { signature, .. } => signature.to_bytes().to_vec(),
        }
    }

    /// Return bytes for (self.public_key | self.scheme_id). To create an account authentication
    /// key, sha3 these bytes
    // TODO: move AuthenticationKey to this file and make this private
    pub fn compute_authentication_key_preimage(
        mut public_key_bytes: Vec<u8>,
        scheme_id: u8,
    ) -> AuthenticationKeyPreimage {
        public_key_bytes.push(scheme_id);
        AuthenticationKeyPreimage(public_key_bytes)
    }

    // TODO: move AuthenticationKey to this file to avoid try_from
    fn preimage_to_authentication_key(preimage: &AuthenticationKeyPreimage) -> AuthenticationKey {
        AuthenticationKey::try_from(HashValue::from_sha3_256(&preimage.0).to_vec()).unwrap()
    }

    pub fn authentication_key_preimage(&self) -> AuthenticationKeyPreimage {
        Self::compute_authentication_key_preimage(
            self.public_key_bytes().to_vec(),
            self.scheme_id(),
        )
    }

    /// Return bytes for (self.public_key | self.scheme_id)
    pub fn ed25519_authentication_key_preimage(
        public_key: &Ed25519PublicKey,
    ) -> AuthenticationKeyPreimage {
        Self::compute_authentication_key_preimage(public_key.to_bytes().to_vec(), ED25519_SCHEME)
    }

    /// Return bytes for (self.public_key | self.scheme_id)
    pub fn multi_ed25519_authentication_key_preimage(
        public_key: &MultiEd25519PublicKey,
    ) -> AuthenticationKeyPreimage {
        Self::compute_authentication_key_preimage(public_key.to_bytes(), MULTI_ED25519_SCHEME)
    }

    /// Return an authentication derived from this public key
    pub fn ed25519_authentication_key(public_key: &Ed25519PublicKey) -> AuthenticationKey {
        Self::preimage_to_authentication_key(&Self::compute_authentication_key_preimage(
            public_key.to_bytes().to_vec(),
            ED25519_SCHEME,
        ))
    }

    /// Return an authentication derived from this public key
    pub fn multi_ed25519_authentication_key(
        public_key: &MultiEd25519PublicKey,
    ) -> AuthenticationKey {
        Self::preimage_to_authentication_key(&Self::compute_authentication_key_preimage(
            public_key.to_bytes(),
            MULTI_ED25519_SCHEME,
        ))
    }
}

impl fmt::Display for TransactionAuthenticator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "TransactionAuthenticator[scheme id: {}, public key: {}, signature: {}]",
            self.scheme_id(),
            hex::encode(&self.public_key_bytes()),
            hex::encode(&self.signature_bytes())
        )
    }
}
