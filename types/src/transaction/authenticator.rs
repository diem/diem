// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_address::AccountAddress,
    transaction::{RawTransaction, RawTransactionWithData},
};
use anyhow::{ensure, Error, Result};
use diem_crypto::{
    ed25519::{Ed25519PublicKey, Ed25519Signature},
    hash::CryptoHash,
    multi_ed25519::{MultiEd25519PublicKey, MultiEd25519Signature},
    traits::Signature,
    validatable::Validatable,
    CryptoMaterialError, HashValue, ValidCryptoMaterial, ValidCryptoMaterialStringExt,
};
use diem_crypto_derive::{CryptoHasher, DeserializeKey, SerializeKey};
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use rand::{rngs::OsRng, Rng};
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, fmt, str::FromStr};
use thiserror::Error;

/// Maximum number of signatures supported in `TransactionAuthenticator`,
/// across all `AccountAuthenticator`s included.
pub const MAX_NUM_OF_SIGS: usize = 32;

/// An error enum for issues related to transaction or account authentication.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
#[error("{:?}", self)]
pub enum AuthenticationError {
    /// The number of signatures exceeds the maximum supported.
    MaxSignaturesExceeded,
}

/// Each transaction submitted to the Diem blockchain contains a `TransactionAuthenticator`. During
/// transaction execution, the executor will check if every `AccountAuthenticator`'s signature on
/// the transaction hash is well-formed and whether the sha3 hash of the
/// `AccountAuthenticator`'s `AuthenticationKeyPreimage` matches the `AuthenticationKey` stored
/// under the participating signer's account address.

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum TransactionAuthenticator {
    /// Single signature
    Ed25519 {
        public_key: Validatable<Ed25519PublicKey>,
        signature: Ed25519Signature,
    },
    /// K-of-N multisignature
    MultiEd25519 {
        public_key: MultiEd25519PublicKey,
        signature: MultiEd25519Signature,
    },
    /// Multi-agent transaction.
    MultiAgent {
        sender: AccountAuthenticator,
        secondary_signer_addresses: Vec<AccountAddress>,
        secondary_signers: Vec<AccountAuthenticator>,
    },
}

impl TransactionAuthenticator {
    /// Create a single-signature ed25519 authenticator
    pub fn ed25519(public_key: Ed25519PublicKey, signature: Ed25519Signature) -> Self {
        Self::Ed25519 {
            public_key: Validatable::new_valid(public_key),
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

    /// Create a multi-agent authenticator
    pub fn multi_agent(
        sender: AccountAuthenticator,
        secondary_signer_addresses: Vec<AccountAddress>,
        secondary_signers: Vec<AccountAuthenticator>,
    ) -> Self {
        Self::MultiAgent {
            sender,
            secondary_signer_addresses,
            secondary_signers,
        }
    }

    /// Return Ok if all AccountAuthenticator's public keys match their signatures, Err otherwise
    pub fn verify(&self, raw_txn: &RawTransaction) -> Result<()> {
        let num_sigs: usize = self.sender().number_of_signatures()
            + self
                .secondary_signers()
                .iter()
                .map(|auth| auth.number_of_signatures())
                .sum::<usize>();
        if num_sigs > MAX_NUM_OF_SIGS {
            return Err(Error::new(AuthenticationError::MaxSignaturesExceeded));
        }
        match self {
            Self::Ed25519 {
                public_key,
                signature,
            } => signature.verify(raw_txn, public_key.validate()?),
            Self::MultiEd25519 {
                public_key,
                signature,
            } => signature.verify(raw_txn, public_key),
            Self::MultiAgent {
                sender,
                secondary_signer_addresses,
                secondary_signers,
            } => {
                let message = RawTransactionWithData::new_multi_agent(
                    raw_txn.clone(),
                    secondary_signer_addresses.clone(),
                );
                sender.verify(&message)?;
                for signer in secondary_signers {
                    signer.verify(&message)?;
                }
                Ok(())
            }
        }
    }

    pub fn sender(&self) -> AccountAuthenticator {
        match self {
            Self::Ed25519 {
                public_key,
                signature,
            } => AccountAuthenticator::Ed25519 {
                public_key: public_key.clone(),
                signature: signature.clone(),
            },
            Self::MultiEd25519 {
                public_key,
                signature,
            } => AccountAuthenticator::multi_ed25519(public_key.clone(), signature.clone()),
            Self::MultiAgent { sender, .. } => sender.clone(),
        }
    }

    pub fn secondary_signer_addreses(&self) -> Vec<AccountAddress> {
        match self {
            Self::Ed25519 { .. }
            | Self::MultiEd25519 {
                public_key: _,
                signature: _,
            } => vec![],
            Self::MultiAgent {
                sender: _,
                secondary_signer_addresses,
                ..
            } => secondary_signer_addresses.to_vec(),
        }
    }

    pub fn secondary_signers(&self) -> Vec<AccountAuthenticator> {
        match self {
            Self::Ed25519 { .. }
            | Self::MultiEd25519 {
                public_key: _,
                signature: _,
            } => vec![],
            Self::MultiAgent {
                sender: _,
                secondary_signer_addresses: _,
                secondary_signers,
            } => secondary_signers.to_vec(),
        }
    }
}

impl fmt::Display for TransactionAuthenticator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ed25519 {
                public_key: _,
                signature: _,
            } => {
                write!(
                    f,
                    "TransactionAuthenticator[scheme: Ed25519, sender: {}]",
                    self.sender()
                )
            }
            Self::MultiEd25519 {
                public_key: _,
                signature: _,
            } => {
                write!(
                    f,
                    "TransactionAuthenticator[scheme: MultiEd25519, sender: {}]",
                    self.sender()
                )
            }
            Self::MultiAgent {
                sender,
                secondary_signer_addresses,
                secondary_signers,
            } => {
                let mut sec_addrs: String = "".to_string();
                for sec_addr in secondary_signer_addresses {
                    sec_addrs = format!("{}\n\t\t\t{:#?},", sec_addrs, sec_addr);
                }
                let mut sec_signers: String = "".to_string();
                for sec_signer in secondary_signers {
                    sec_signers = format!("{}\n\t\t\t{:#?},", sec_signers, sec_signer);
                }
                write!(
                    f,
                    "TransactionAuthenticator[\n\
                        \tscheme: MultiAgent, \n\
                        \tsender: {}\n\
                        \tsecondary signer addresses: {}\n\
                        \tsecondary signers: {}]",
                    sender, sec_addrs, sec_signers,
                )
            }
        }
    }
}

/// An `AccountAuthenticator` is an an abstraction of a signature scheme. It must know:
/// (1) How to check its signature against a message and public key
/// (2) How to convert its public key into an `AuthenticationKeyPreimage` structured as
/// (public_key | signaure_scheme_id).
/// Each on-chain `DiemAccount` must store an `AuthenticationKey` (computed via a sha3 hash of an
/// `AuthenticationKeyPreimage`).

// TODO: in the future, can tie these to the AccountAuthenticator enum directly with https://github.com/rust-lang/rust/issues/60553
#[derive(Debug)]
#[repr(u8)]
pub enum Scheme {
    Ed25519 = 0,
    MultiEd25519 = 1,
    // ... add more schemes here
}

impl fmt::Display for Scheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let display = match self {
            Scheme::Ed25519 => "Ed25519",
            Scheme::MultiEd25519 => "MultiEd25519",
        };
        write!(f, "Scheme::{}", display)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum AccountAuthenticator {
    /// Single signature
    Ed25519 {
        public_key: Validatable<Ed25519PublicKey>,
        signature: Ed25519Signature,
    },
    /// K-of-N multisignature
    MultiEd25519 {
        public_key: MultiEd25519PublicKey,
        signature: MultiEd25519Signature,
    },
    // ... add more schemes here
}

impl AccountAuthenticator {
    /// Unique identifier for the signature scheme
    pub fn scheme(&self) -> Scheme {
        match self {
            Self::Ed25519 { .. } => Scheme::Ed25519,
            Self::MultiEd25519 { .. } => Scheme::MultiEd25519,
        }
    }

    /// Create a single-signature ed25519 authenticator
    pub fn ed25519(public_key: Ed25519PublicKey, signature: Ed25519Signature) -> Self {
        Self::Ed25519 {
            public_key: Validatable::new_valid(public_key),
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
    pub fn verify<T: Serialize + CryptoHash>(&self, message: &T) -> Result<()> {
        match self {
            Self::Ed25519 {
                public_key,
                signature,
            } => signature.verify(message, public_key.validate()?),
            Self::MultiEd25519 {
                public_key,
                signature,
            } => signature.verify(message, public_key),
        }
    }

    /// Return the raw bytes of `self.public_key`
    pub fn public_key_bytes(&self) -> Vec<u8> {
        match self {
            Self::Ed25519 { public_key, .. } => public_key.unvalidated().to_bytes().to_vec(),
            Self::MultiEd25519 { public_key, .. } => public_key.to_bytes().to_vec(),
        }
    }

    /// Return the raw bytes of `self.signature`
    pub fn signature_bytes(&self) -> Vec<u8> {
        match self {
            Self::Ed25519 { signature, .. } => signature.to_bytes().to_vec(),
            Self::MultiEd25519 { signature, .. } => signature.to_bytes().to_vec(),
        }
    }

    /// Return an authentication key preimage derived from `self`'s public key and scheme id
    pub fn authentication_key_preimage(&self) -> AuthenticationKeyPreimage {
        AuthenticationKeyPreimage::new(self.public_key_bytes(), self.scheme())
    }

    /// Return an authentication key derived from `self`'s public key and scheme id
    pub fn authentication_key(&self) -> AuthenticationKey {
        AuthenticationKey::from_preimage(&self.authentication_key_preimage())
    }

    /// Return the number of signatures included in this account authenticator.
    pub fn number_of_signatures(&self) -> usize {
        match self {
            Self::Ed25519 { .. } => 1,
            Self::MultiEd25519 { signature, .. } => signature.signatures().len(),
        }
    }
}

/// A struct that represents an account authentication key. An account's address is the last 16
/// bytes of authentication key used to create it
#[derive(
    Clone,
    Copy,
    CryptoHasher,
    Debug,
    DeserializeKey,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
    SerializeKey,
)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct AuthenticationKey([u8; AuthenticationKey::LENGTH]);

impl AuthenticationKey {
    /// Create an authentication key from `bytes`
    pub const fn new(bytes: [u8; Self::LENGTH]) -> Self {
        Self(bytes)
    }

    /// Return an authentication key that is impossible (in expectation) to sign for--useful for
    /// intentionally relinquishing control of an account.
    pub const fn zero() -> Self {
        Self([0; 32])
    }

    /// The number of bytes in an authentication key.
    pub const LENGTH: usize = 32;

    /// Create an authentication key from a preimage by taking its sha3 hash
    pub fn from_preimage(preimage: &AuthenticationKeyPreimage) -> AuthenticationKey {
        AuthenticationKey::new(*HashValue::sha3_256_of(&preimage.0).as_ref())
    }

    /// Create an authentication key from an Ed25519 public key
    pub fn ed25519(public_key: &Ed25519PublicKey) -> AuthenticationKey {
        Self::from_preimage(&AuthenticationKeyPreimage::ed25519(public_key))
    }

    /// Create an authentication key from a MultiEd25519 public key
    pub fn multi_ed25519(public_key: &MultiEd25519PublicKey) -> Self {
        Self::from_preimage(&AuthenticationKeyPreimage::multi_ed25519(public_key))
    }

    /// Return an address derived from the last `AccountAddress::LENGTH` bytes of this
    /// authentication key.
    pub fn derived_address(&self) -> AccountAddress {
        // keep only last 16 bytes
        let mut array = [0u8; AccountAddress::LENGTH];
        array.copy_from_slice(&self.0[Self::LENGTH - AccountAddress::LENGTH..]);
        AccountAddress::new(array)
    }

    /// Return the first AccountAddress::LENGTH bytes of this authentication key
    pub fn prefix(&self) -> [u8; AccountAddress::LENGTH] {
        let mut array = [0u8; AccountAddress::LENGTH];
        array.copy_from_slice(&self.0[..AccountAddress::LENGTH]);
        array
    }

    /// Construct a vector from this authentication key
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    /// Create a random authentication key. For testing only
    pub fn random() -> Self {
        let mut rng = OsRng;
        let buf: [u8; Self::LENGTH] = rng.gen();
        AuthenticationKey::new(buf)
    }
}

impl ValidCryptoMaterial for AuthenticationKey {
    fn to_bytes(&self) -> Vec<u8> {
        self.to_vec()
    }
}

/// A value that can be hashed to produce an authentication key
pub struct AuthenticationKeyPreimage(Vec<u8>);

impl AuthenticationKeyPreimage {
    /// Return bytes for (public_key | scheme_id)
    fn new(mut public_key_bytes: Vec<u8>, scheme: Scheme) -> Self {
        public_key_bytes.push(scheme as u8);
        Self(public_key_bytes)
    }

    /// Construct a preimage from an Ed25519 public key
    pub fn ed25519(public_key: &Ed25519PublicKey) -> AuthenticationKeyPreimage {
        Self::new(public_key.to_bytes().to_vec(), Scheme::Ed25519)
    }

    /// Construct a preimage from a MultiEd25519 public key
    pub fn multi_ed25519(public_key: &MultiEd25519PublicKey) -> AuthenticationKeyPreimage {
        Self::new(public_key.to_bytes(), Scheme::MultiEd25519)
    }

    /// Construct a vector from this authentication key
    pub fn into_vec(self) -> Vec<u8> {
        self.0
    }
}

impl fmt::Display for AccountAuthenticator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AccountAuthenticator[scheme id: {:?}, public key: {}, signature: {}]",
            self.scheme(),
            hex::encode(&self.public_key_bytes()),
            hex::encode(&self.signature_bytes())
        )
    }
}

impl TryFrom<&[u8]> for AuthenticationKey {
    type Error = CryptoMaterialError;

    fn try_from(bytes: &[u8]) -> std::result::Result<AuthenticationKey, CryptoMaterialError> {
        if bytes.len() != Self::LENGTH {
            return Err(CryptoMaterialError::WrongLengthError);
        }
        let mut addr = [0u8; Self::LENGTH];
        addr.copy_from_slice(bytes);
        Ok(AuthenticationKey(addr))
    }
}

impl TryFrom<Vec<u8>> for AuthenticationKey {
    type Error = CryptoMaterialError;

    fn try_from(bytes: Vec<u8>) -> std::result::Result<AuthenticationKey, CryptoMaterialError> {
        AuthenticationKey::try_from(&bytes[..])
    }
}

impl FromStr for AuthenticationKey {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        ensure!(
            !s.is_empty(),
            "authentication key string should not be empty.",
        );
        let bytes_out = ::hex::decode(s)?;
        let key = AuthenticationKey::try_from(bytes_out.as_slice())?;
        Ok(key)
    }
}

impl AsRef<[u8]> for AuthenticationKey {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::LowerHex for AuthenticationKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(&self.0))
    }
}

impl fmt::Display for AuthenticationKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        // Forward to the LowerHex impl with a "0x" prepended (the # flag).
        write!(f, "{:#x}", self)
    }
}

#[cfg(test)]
mod tests {
    use crate::transaction::authenticator::AuthenticationKey;
    use std::str::FromStr;

    #[test]
    fn test_from_str_should_not_panic_by_given_empty_string() {
        assert!(AuthenticationKey::from_str("").is_err());
    }
}
