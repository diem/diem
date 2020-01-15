// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! The following is a minimalist version of a hierarchical key derivation library for the
//! LibraWallet.
//!
//! Note that the Libra Blockchain makes use of ed25519 Edwards Digital Signature Algorithm
//! (EdDSA) and therefore, BIP32 Public Key derivation is not available without falling back to
//! a non-deterministic Schnorr signature scheme. As LibraWallet is meant to be a minimalist
//! reference implementation of a simple wallet, the following does not deviate from the
//! ed25519 spec. In a future iteration of this wallet, we will also provide an implementation
//! of a Schnorr variant over curve25519 and demonstrate our proposal for BIP32-like public key
//! derivation.
//!
//! Note further that the Key Derivation Function (KDF) chosen in the derivation of Child
//! Private Keys adheres to [HKDF RFC 5869](https://tools.ietf.org/html/rfc5869).

use byteorder::{ByteOrder, LittleEndian};
use hmac::Hmac;
use libra_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey, Ed25519Signature},
    hash::HashValue,
    hkdf::Hkdf,
    traits::SigningKey,
};
use libra_types::account_address::AccountAddress;
use mirai_annotations::*;
use pbkdf2::pbkdf2;
use serde::{Deserialize, Serialize};
use sha3::Sha3_256;
use std::{convert::TryFrom, ops::AddAssign};

use crate::{error::Result, mnemonic::Mnemonic};

/// Master is a set of raw bytes that are used for child key derivation
pub struct Master([u8; 32]);
impl_array_newtype!(Master, u8, 32);
impl_array_newtype_show!(Master);
impl_array_newtype_encodable!(Master, u8, 32);

/// A child number for a derived key, used to derive a certain private key from the Master
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
    /// ChildNumber are not used in this iteration of LibraWallet, but in order to
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

    /// Computes the sha3 hash of the PublicKey and attempts to construct a Libra AccountAddress
    /// from the raw bytes of the pubkey hash
    pub fn get_address(&self) -> Result<AccountAddress> {
        let public_key = self.get_public();
        let hash = *HashValue::from_sha3_256(&public_key.to_bytes()).as_ref();
        let addr = AccountAddress::try_from(&hash[..])?;
        Ok(addr)
    }

    /// Libra specific sign function that is capable of signing an arbitrary HashValue
    /// NOTE: In Libra, we do not sign the raw bytes of a transaction, instead we sign the raw
    /// bytes of the sha3 hash of the raw bytes of a transaction. It is important to note that the
    /// raw bytes of the sha3 hash will be hashed again as part of the ed25519 signature algorithm.
    /// In other words: In Libra, the message used for signature and verification is the sha3 hash
    /// of the transaction. This sha3 hash is then hashed again using SHA512 to arrive at the
    /// deterministic nonce for the EdDSA.
    pub fn sign(&self, msg: HashValue) -> Ed25519Signature {
        self.private_key.sign_message(&msg)
    }
}

/// Wrapper struct from which we derive child keys
pub struct KeyFactory {
    master: Master,
}

impl KeyFactory {
    const MNEMONIC_SALT_PREFIX: &'static [u8] = b"LIBRA WALLET: mnemonic salt prefix$";
    const MASTER_KEY_SALT: &'static [u8] = b"LIBRA WALLET: master key salt$";
    const INFO_PREFIX: &'static [u8] = b"LIBRA WALLET: derived key$";
    /// Instantiate a new KeyFactor from a Seed, where the [u8; 64] raw bytes of the Seed are used
    /// to derive both the Master
    pub fn new(seed: &Seed) -> Result<Self> {
        let hkdf_extract = Hkdf::<Sha3_256>::extract(Some(KeyFactory::MASTER_KEY_SALT), &seed.0)?;

        Ok(Self {
            master: Master::from(&hkdf_extract[..32]),
        })
    }

    /// Getter for the Master
    pub fn master(&self) -> &[u8] {
        &self.master.0[..]
    }

    /// Derive a particular PrivateKey at a certain ChildNumber
    ///
    /// Note that the function below  adheres to [HKDF RFC 5869](https://tools.ietf.org/html/rfc5869).
    pub fn private_child(&self, child: ChildNumber) -> Result<ExtendedPrivKey> {
        // application info in the HKDF context is defined as Libra derived key$child_number.
        let mut le_n = [0u8; 8];
        LittleEndian::write_u64(&mut le_n, child.0);
        let mut info = KeyFactory::INFO_PREFIX.to_vec();
        info.extend_from_slice(&le_n);

        let hkdf_expand = Hkdf::<Sha3_256>::expand(&self.master(), Some(&info), 32)?;
        let sk = Ed25519PrivateKey::try_from(hkdf_expand.as_slice())
            .expect("Unable to convert into private key");

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
    let seed = Seed::new(&mnemonic, "LIBRA");

    let key_factory = KeyFactory::new(&seed).unwrap();
    assert_eq!(
        "16274c9618ed59177ca948529c1884ba65c57984d562ec2b4e5aa1ee3e3903be",
        hex::encode(&key_factory.master())
    );

    // Check child_0 key derivation.
    let child_private_0 = key_factory.private_child(ChildNumber(0)).unwrap();
    assert_eq!(
        "358a375f36d74c30b7f3299b62d712b307725938f8cc931100fbd10a434fc8b9",
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
        "a325fe7d27b1b49f191cc03525951fec41b6ffa2d4b3007bb1d9dd353b7e56a6",
        hex::encode(&child_private_1.private_key.to_bytes()[..])
    );

    let mut child_1_again = ChildNumber(0);
    child_1_again.increment();
    assert_eq!(ChildNumber(1), child_1_again);

    // Check determinism, regenerate child_1, but by incrementing ChildNumber(0).
    let child_private_1_from_increment = key_factory.private_child(child_1_again).unwrap();
    assert_eq!(
        "a325fe7d27b1b49f191cc03525951fec41b6ffa2d4b3007bb1d9dd353b7e56a6",
        hex::encode(&child_private_1_from_increment.private_key.to_bytes()[..])
    );
}
