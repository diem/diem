// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines a data structure made to contain a cryptographic
//! signature, in the sense of an implementation of
//! libra_crypto::traits::Signature. The container is an opaque NewType that
//! intentionally does not allow access to the inner impl.
//!
//! It also proxies the four methods used in consensus on structures that
//! expect to receive one such implementation directly.
//!
//! The goal of this structure is two-fold:
//! - help make sure that any consensus data that uses cryptographic material is defined generically
//!   in terms of libra_crypto::traits, rather than accessing the implementation details of a
//!   particular scheme (a.k.a. encapsulation),
//! - contribute to a single-location place for instantiations of these polymorphic types
//!   (`crate::chained_bft::consensus_types`)
//! - help the consensus crate avoid generic parameters within the code, by avoiding propagating
//!   generic signature type parameters throughout the crate's code (as prototyped in https://github.com/libra/libra/pull/522),
//!   as generic types are seen as hindering readability.

use crate::{
    account_address::AccountAddress,
    ledger_info::LedgerInfoWithSignatures as RawLedgerInfoWithSignatures,
    validator_change::ValidatorChangeEventWithProof as RawValidatorChangeEventWithProof,
    validator_signer::ValidatorSigner as RawValidatorSigner,
    validator_verifier::{ValidatorVerifier as RawValidatorVerifier, VerifyError},
};
use libra_crypto::{hash::HashValue, traits::Signature as RawSignature};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct SignatureWrapper<Sig: RawSignature>(Sig);

impl<Sig: RawSignature> SignatureWrapper<Sig> {
    pub fn verify(
        &self,
        validator_verifier: &RawValidatorVerifier<Sig::VerifyingKeyMaterial>,
        author: AccountAddress,
        message: HashValue,
    ) -> std::result::Result<(), VerifyError> {
        validator_verifier.verify_signature(author, message, &self.0)
    }

    pub fn try_from(bytes: &[u8]) -> Result<Self, libra_crypto::traits::CryptoMaterialError> {
        Sig::try_from(bytes).map(SignatureWrapper)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.to_bytes()
    }

    pub fn add_to_li(
        self,
        author: AccountAddress,
        li_with_sig: &mut RawLedgerInfoWithSignatures<Sig>,
    ) {
        li_with_sig.add_signature(author, self.0)
    }
}

impl<Sig: RawSignature> From<Sig> for SignatureWrapper<Sig> {
    fn from(s: Sig) -> Self {
        SignatureWrapper(s)
    }
}

// This sets the types containing cryptographic materials used in the
// consensus crate. It is intended as a one-stop shop for changing the
// signing scheme of the consensus. If consensus uses a type that
// contains cryptographic material, its polymorphic parameter should be
// instantiated here and re-exported.
//
// Imports specific to any signing scheme (e.g. ed25519::*) should not be
// present anywhere in the consensus crate. Use of raw cryptographic
// types that do not go through the instantiated polymorphic structures
// below is banned.

use libra_crypto::ed25519::*;

// used in chained_bft::consensus_types::block_test
#[cfg(any(test, feature = "testing"))]
pub type SecretKey = Ed25519PrivateKey;

pub type Signature = SignatureWrapper<Ed25519Signature>;
pub type LedgerInfoWithSignatures = RawLedgerInfoWithSignatures<Ed25519Signature>;
pub type ValidatorVerifier = RawValidatorVerifier<Ed25519PublicKey>;
pub type ValidatorSigner = RawValidatorSigner<Ed25519PrivateKey>;
pub type ValidatorChangeEventWithProof = RawValidatorChangeEventWithProof<Ed25519Signature>;
