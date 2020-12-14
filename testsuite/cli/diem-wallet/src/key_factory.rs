// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! The following is a minimalist version of a hierarchical key derivation library for the
//! DiemWallet.
//!
//! Note that the Diem Blockchain makes use of ed25519 Edwards Digital Signature Algorithm
//! (EdDSA) and therefore, BIP32 Public Key derivation is not available without falling back to
//! a non-deterministic Schnorr signature scheme. As DiemWallet is meant to be a minimalist
//! reference implementation of a simple wallet, the following does not deviate from the
//! ed25519 spec. In a future iteration of this wallet, we will also provide an implementation
//! of a Schnorr variant over curve25519 and demonstrate our proposal for BIP32-like public key
//! derivation.
//!
//! Note further that the Key Derivation Function (KDF) chosen in the derivation of Child
//! Private Keys adheres to [HKDF RFC 5869](https://tools.ietf.org/html/rfc5869).

use crate::mnemonic::Mnemonic;
use anyhow::{anyhow, Result};
use byteorder::{ByteOrder, LittleEndian};
use diem_crypto::{
    compat::Sha3_256,
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey, Ed25519Signature},
    hash::CryptoHash,
    hkdf::Hkdf,
    traits::SigningKey,
};
use diem_types::{account_address::AccountAddress, transaction::authenticator::AuthenticationKey};
use hmac::Hmac;
use mirai_annotations::*;
use pbkdf2::pbkdf2;
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, ops::AddAssign};

/// Main is a set of raw bytes that are used for child key derivation
pub struct Main([u8; 32]);
impl_array_newtype!(Main, u8, 32);
impl_array_newtype_show!(Main);
impl_array_newtype_encodable!(Main, u8, 32);

/// A child number for a derived key, used to derive a certain private key from Main
#[derive(Default, Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct ChildNumber(pub(crate) u64);
// invariant self.0 <= u64::max_value() / 2;

impl ChildNumber {
    /// Constructor from u64
    pub fn new(child_number: u64) -> Self {
        Self(child_number)
    }

    /// Bump the ChildNumber
    pub fn increment(&mut self) {
        self.add_assign(Self(1));
    }
}

impl std::ops::AddAssign for ChildNumber {
    fn add_assign(&mut self, other: Self) {
        assume!(self.0 <= u64::max_value() / 2); // invariant
        assume!(other.0 <= u64::max_value() / 2); // invariant
        *self = Self(self.0 + other.0)
    }
}

impl std::convert::AsRef<u64> for ChildNumber {
    fn as_ref(&self) -> &u64 {
        &self.0
    }
}

impl std::convert::AsMut<u64> for ChildNumber {
    fn as_mut(&mut self) -> &mut u64 {
        &mut self.0
    }
}

/// Derived private key.
pub struct ExtendedPrivKey {
    /// Child number of the key used to derive from Parent.
    _child_number: ChildNumber,
    /// Private key.
    private_key: Ed25519PrivateKey,
}

impl ExtendedPrivKey {
    /// Constructor for creating an ExtendedPrivKey from a ed25519 PrivateKey. Note that the
    /// ChildNumber are not used in this iteration of DiemWallet, but in order to
    /// enable more general Hierarchical KeyDerivation schemes, we include it for completeness.
    pub fn new(_child_number: ChildNumber, private_key: Ed25519PrivateKey) -> Self {
        Self {
            _child_number,
            private_key,
        }
    }

    /// Returns the PublicKey associated to a particular ExtendedPrivKey
    pub fn get_public(&self) -> Ed25519PublicKey {
        (&self.private_key).into()
    }

    /// Compute the account address for this account's public key
    pub fn get_address(&self) -> AccountAddress {
        diem_types::account_address::from_public_key(&self.get_public())
    }

    /// Get private key
    pub fn get_private_key(&self) -> Ed25519PrivateKey {
        self.private_key.clone()
    }

    /// Compute the authentication key for this account's public key
    pub fn get_authentication_key(&self) -> AuthenticationKey {
        AuthenticationKey::ed25519(&self.get_public())
    }

    /// Diem specific sign function that is capable of signing an arbitrary
    /// Serializable value.
    ///
    /// NOTE: In Diem, we do not sign the raw bytes of a transaction, but
    /// those raw bytes prefixed by a domain separation hash.
    /// Informally signed_bytes = sha3(domain_separator) || bcs_serialization_bytes
    ///
    /// The domain separator hash is derived automatically from a `#[derive(CryptoHasher,
    /// BCSCryptoHash)]` annotation, or can be declared manually in a process
    /// described in `diem_crypto::hash`.
    ///
    pub fn sign<T: CryptoHash + Serialize>(&self, msg: &T) -> Ed25519Signature {
        self.private_key.sign(msg)
    }
}

/// Wrapper struct from which we derive child keys
pub struct KeyFactory {
    main: Main,
}

impl KeyFactory {
    const MNEMONIC_SALT_PREFIX: &'static [u8] = b"DIEM WALLET: mnemonic salt prefix$";
    const MAIN_KEY_SALT: &'static [u8] = b"DIEM WALLET: main key salt$";
    const INFO_PREFIX: &'static [u8] = b"DIEM WALLET: derived key$";
    /// Instantiate a new KeyFactor from a Seed, where the [u8; 64] raw bytes of the Seed are used
    /// to derive both the Main and its child keys
    pub fn new(seed: &Seed) -> Result<Self> {
        let hkdf_extract = Hkdf::<Sha3_256>::extract(Some(KeyFactory::MAIN_KEY_SALT), &seed.0)?;

        Ok(Self {
            main: Main::from(&hkdf_extract[..32]),
        })
    }

    /// Getter for Main
    pub fn main(&self) -> &[u8] {
        &self.main.0[..]
    }

    /// Derive a particular PrivateKey at a certain ChildNumber
    ///
    /// Note that the function below  adheres to [HKDF RFC 5869](https://tools.ietf.org/html/rfc5869).
    pub fn private_child(&self, child: ChildNumber) -> Result<ExtendedPrivKey> {
        // application info in the HKDF context is defined as Diem derived key$child_number.
        let mut le_n = [0u8; 8];
        LittleEndian::write_u64(&mut le_n, child.0);
        let mut info = KeyFactory::INFO_PREFIX.to_vec();
        info.extend_from_slice(&le_n);

        let hkdf_expand = Hkdf::<Sha3_256>::expand(&self.main(), Some(&info), 32)?;
        let sk = Ed25519PrivateKey::try_from(hkdf_expand.as_slice()).map_err(|e| {
            anyhow!(
                "Unable to convert hkdf output into private key, met Error:{}",
                e
            )
        })?;
        Ok(ExtendedPrivKey::new(child, sk))
    }
}

/// Seed is the output of a one-way function, which accepts a Mnemonic as input
pub struct Seed([u8; 32]);

impl Seed {
    /// This constructor implements the one-way function that allows to generate a Seed from a
    /// particular Mnemonic and salt. WalletLibrary implements a fixed salt, but a user could
    /// choose a user-defined salt instead of the hardcoded one.
    pub fn new(mnemonic: &Mnemonic, salt: &str) -> Seed {
        let mut output = [0u8; 32];

        let mut msalt = KeyFactory::MNEMONIC_SALT_PREFIX.to_vec();
        msalt.extend_from_slice(salt.as_bytes());

        pbkdf2::<Hmac<Sha3_256>>(mnemonic.to_string().as_ref(), &msalt, 2048, &mut output);
        Seed(output)
    }
}

#[cfg(test)]
#[test]
fn assert_default_child_number() {
    assert_eq!(ChildNumber::default(), ChildNumber(0));
}

#[cfg(test)]
#[test]
fn test_key_derivation() {
    let data = hex::decode("7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f").unwrap();
    let mnemonic = Mnemonic::from("legal winner thank year wave sausage worth useful legal winner thank year wave sausage worth useful legal will").unwrap();
    assert_eq!(
        mnemonic.to_string(),
        Mnemonic::mnemonic(&data).unwrap().to_string()
    );
    let seed = Seed::new(&mnemonic, "DIEM");

    let key_factory = KeyFactory::new(&seed).unwrap();
    assert_eq!(
        "66ae6b767defe3ea0c646f10bf31ad3b36f822064d3923adada7676703a350c0",
        hex::encode(&key_factory.main())
    );

    // Check child_0 key derivation.
    let child_private_0 = key_factory.private_child(ChildNumber(0)).unwrap();
    assert_eq!(
        "732bc883893c716f320c01864709ca9f16f8f30342a1de42144bfcc2ddb7af10",
        hex::encode(&child_private_0.private_key.to_bytes()[..])
    );

    // Check determinism, regenerate child_0.
    let child_private_0_again = key_factory.private_child(ChildNumber(0)).unwrap();
    assert_eq!(
        hex::encode(&child_private_0.private_key.to_bytes()[..]),
        hex::encode(&child_private_0_again.private_key.to_bytes()[..])
    );

    // Check child_1 key derivation.
    let child_private_1 = key_factory.private_child(ChildNumber(1)).unwrap();
    assert_eq!(
        "f6b472bd0941e315d3c34c3ac679d610d2b9e1abe85128752d04bb0f042f3391",
        hex::encode(&child_private_1.private_key.to_bytes()[..])
    );

    let mut child_1_again = ChildNumber(0);
    child_1_again.increment();
    assert_eq!(ChildNumber(1), child_1_again);

    // Check determinism, regenerate child_1, but by incrementing ChildNumber(0).
    let child_private_1_from_increment = key_factory.private_child(child_1_again).unwrap();
    assert_eq!(
        "f6b472bd0941e315d3c34c3ac679d610d2b9e1abe85128752d04bb0f042f3391",
        hex::encode(&child_private_1_from_increment.private_key.to_bytes()[..])
    );
}
