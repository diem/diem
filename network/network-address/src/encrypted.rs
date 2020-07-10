// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::RawNetworkAddress;
use aes_gcm::{
    aead::{generic_array::GenericArray, AeadInPlace, NewAead},
    Aes256Gcm,
};
use libra_crypto::{compat::Sha3_256, hkdf::Hkdf};
use move_core_types::account_address::AccountAddress;
#[cfg(any(test, feature = "fuzzing"))]
use proptest::prelude::*;
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, mem};
use thiserror::Error;

/// The length in bytes of the AES-256-GCM authentication tag.
pub const AES_GCM_TAG_LEN: usize = 16;

/// The length in bytes of the AES-256-GCM nonce.
pub const AES_GCM_NONCE_LEN: usize = 12;

/// The length in bytes of the `shared_val_netaddr_key` and per-validator
/// `derived_key`.
pub const KEY_LEN: usize = 32;

/// Convenient type alias for the `shared_val_netaddr_key` as an array.
pub type Key = [u8; KEY_LEN];
pub type KeyVersion = u32;

/// Constant key + version so we can push `EncNetworkAddress` everywhere
/// without worrying about getting the key in the right places. these will be
/// test-only soon.
// TODO(philiphayes): feature gate for testing/fuzzing only
pub const TEST_SHARED_VAL_NETADDR_KEY: Key = [0u8; KEY_LEN];
pub const TEST_SHARED_VAL_NETADDR_KEY_VERSION: KeyVersion = 0;

/// We salt the HKDF for deriving the account keys to provide application
/// separation.
///
/// Note: modifying this salt is a backwards-incompatible protocol change.
///
/// For readers, the HKDF salt is equal to the following hex string:
/// `"dfc8ffcc7f62ea4e5b9bc41ee7969b44275419ebaad1db27d2a191b6d1db6d13"` which is
/// also equal to the hash value `SHA3-256(b"LIBRA_ENCRYPTED_NETWORK_ADDRESS_SALT")`.
///
/// ```
/// use libra_network_address::encrypted::HKDF_SALT;
/// use libra_crypto::hash::HashValue;
///
/// let derived_salt = HashValue::sha3_256_of(b"LIBRA_ENCRYPTED_NETWORK_ADDRESS_SALT");
/// assert_eq!(HKDF_SALT.as_ref(), derived_salt.as_ref());
/// ```
pub const HKDF_SALT: [u8; 32] = [
    0xdf, 0xc8, 0xff, 0xcc, 0x7f, 0x62, 0xea, 0x4e, 0x5b, 0x9b, 0xc4, 0x1e, 0xe7, 0x96, 0x9b, 0x44,
    0x27, 0x54, 0x19, 0xeb, 0xaa, 0xd1, 0xdb, 0x27, 0xd2, 0xa1, 0x91, 0xb6, 0xd1, 0xdb, 0x6d, 0x13,
];

/// A serialized `EncNetworkAddress`.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct RawEncNetworkAddress(#[serde(with = "serde_bytes")] Vec<u8>);

/// An encrypted `RawNetworkAddress`.
///
/// ### Threat Model
///
/// Encrypting the on-chain network addresses is purely a defense-in-depth
/// mitigation to minimize attack surface and reduce DDoS attacks on the validators
/// by restricting the visibility of their public-facing network addresses only
/// to other validators.
///
/// These encrypted network addresses are intended to be stored on-chain under
/// each validator's advertised network addresses in their `ValidatorConfig`s.
/// All validators share the secret `shared_val_netaddr_key`, though each validator's addresses
/// are encrypted using a per-validator `derived_key`.
///
/// ### Account Key
///
/// ```txt
/// derived_key := HKDF-SHA3-256::extract_and_expand(
///     salt=HKDF_SALT,
///     ikm=shared_val_netaddr_key,
///     info=account_address,
///     output_length=32,
/// )
/// ```
///
/// where `hkdf_sha3_256_extract_and_expand` is
/// [HKDF extract-and-expand](https://tools.ietf.org/html/rfc5869) with SHA3-256,
/// `HKDF_SALT` is a constant salt for application separation, `shared_val_netaddr_key` is the
/// shared secret distributed amongst all the validators, and `account_address`
/// is the specific validator's [`AccountAddress`].
///
/// We use per-validator `derived_key`s to limit the "blast radius" of
/// nonce reuse to each validator, i.e., a validator that accidentally reuses a
/// nonce will only leak information about their network addresses or `derived_key`.
///
/// ### Encryption
///
/// A raw network address, `addr`, is then encrypted using AES-256-GCM like:
///
/// ```txt
/// enc_addr := AES-256-GCM::encrypt(
///     key=derived_key,
///     nonce=nonce,
///     ad=key_version,
///     message=addr,
/// )
/// ```
///
/// where `nonce` is a 96-bit integer as described below, `key_version` is
/// the key version as a u32 big-endian integer, `addr` is the serialized
/// `RawNetworkAddress`, and `enc_addr` is the encrypted network address
/// concatenated with the 16-byte authentication tag.
///
/// ### Nonce
///
/// ```txt
/// nonce := seq_num || addr_idx
/// ```
///
/// where `seq_num` is the `seq_num` field as a u64 big-endian integer and
/// `addr_idx` is the index of the encrypted network address in the list of
/// network addresses as a u32 big-endian integer.
///
/// ### Sequence Number
///
/// In order to reduce the probability of nonce reuse, validators should use the
/// sequence number of the rotation transaction in the `seq_num` field.
///
/// ### Key Rotation
///
/// The `EncNetworkAddress` struct contains a `key_version` field, which
/// identifies the specific `shared_val_netaddr_key` used to encrypt/decrypt the
/// `EncNetworkAddress`.
///
/// TODO(philiphayes): expand this section
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct EncNetworkAddress {
    key_version: KeyVersion,
    seq_num: u64,
    #[serde(with = "serde_bytes")]
    enc_addr: Vec<u8>,
}

#[derive(Error, Debug)]
#[error("error decrypting network address")]
pub struct DecryptError;

//////////////////////////
// RawEncNetworkAddress //
//////////////////////////

impl RawEncNetworkAddress {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
}

impl Into<Vec<u8>> for RawEncNetworkAddress {
    fn into(self) -> Vec<u8> {
        self.0
    }
}

impl AsRef<[u8]> for RawEncNetworkAddress {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl TryFrom<&EncNetworkAddress> for RawEncNetworkAddress {
    type Error = lcs::Error;

    fn try_from(value: &EncNetworkAddress) -> Result<Self, lcs::Error> {
        let bytes = lcs::to_bytes(value)?;
        Ok(RawEncNetworkAddress::new(bytes))
    }
}

#[cfg(any(test, feature = "fuzzing"))]
impl Arbitrary for RawEncNetworkAddress {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        any::<EncNetworkAddress>()
            .prop_map(|enc_addr| RawEncNetworkAddress::try_from(&enc_addr).unwrap())
            .boxed()
    }
}

///////////////////////
// EncNetworkAddress //
///////////////////////

impl EncNetworkAddress {
    /// ### Panics
    ///
    /// encrypt will panic if `addr` length > 64 GiB.
    pub fn encrypt(
        addr: RawNetworkAddress,
        shared_val_netaddr_key: &Key,
        key_version: KeyVersion,
        account: &AccountAddress,
        seq_num: u64,
        addr_idx: u32,
    ) -> Self {
        // unpack the RawNetworkAddress into its base Vec<u8>
        let mut addr_vec: Vec<u8> = addr.into();

        let derived_key = Self::derive_key(shared_val_netaddr_key, account);
        let aead = Aes256Gcm::new(GenericArray::from_slice(&derived_key));

        // nonce := seq_num || addr_idx
        //
        // concatenate seq_num and addr_idx into a 12-byte AES-GCM nonce. both
        // seq_num and addr_idx are big-endian integers.
        //
        // ex: seq_num = 0x1234, addr_idx = 0x04
        //     ==> nonce_slice == &[0, 0, 0, 0, 0, 0, 0x12, 0x34, 0, 0, 0, 0x4]
        let nonce = (((seq_num as u128) << 32) | (addr_idx as u128)).to_be_bytes();
        let nonce_slice = &nonce[mem::size_of::<u128>() - AES_GCM_NONCE_LEN..];
        let nonce_slice = GenericArray::from_slice(nonce_slice);

        // the key_version is in-the-clear, so we include it in the integrity check
        // using the "associated data"
        let ad_buf = key_version.to_be_bytes();
        let ad_slice = &ad_buf[..];

        // encrypt the raw network address in-place
        // note: this can technically panic if the serialized network address
        //       length is > 64 GiB
        let auth_tag = aead
            .encrypt_in_place_detached(nonce_slice, ad_slice, &mut addr_vec)
            .expect("addr.len() must be <= 64 GiB");

        // append the authentication tag
        addr_vec.extend_from_slice(auth_tag.as_slice());

        Self {
            key_version,
            seq_num,
            enc_addr: addr_vec,
        }
    }

    pub fn decrypt(
        self,
        shared_val_netaddr_key: &Key,
        account: &AccountAddress,
        addr_idx: u32,
    ) -> Result<RawNetworkAddress, DecryptError> {
        let key_version = self.key_version;
        let seq_num = self.seq_num;
        let mut enc_addr = self.enc_addr;

        // ciphertext is too small to even contain the authentication tag, so it
        // must be invalid.
        if enc_addr.len() < AES_GCM_TAG_LEN {
            return Err(DecryptError);
        }

        let derived_key = Self::derive_key(shared_val_netaddr_key, account);
        let aead = Aes256Gcm::new(GenericArray::from_slice(&derived_key));

        // nonce := seq_num || addr_idx
        //
        // concatenate seq_num and addr_idx into a 12-byte AES-GCM nonce. both
        // seq_num and addr_idx are big-endian integers.
        //
        // ex: seq_num = 0x1234, addr_idx = 0x04
        //     ==> nonce_slice == &[0, 0, 0, 0, 0, 0, 0x12, 0x34, 0, 0, 0, 0x4]
        let nonce = (((seq_num as u128) << 32) | (addr_idx as u128)).to_be_bytes();
        let nonce_slice = &nonce[mem::size_of::<u128>() - AES_GCM_NONCE_LEN..];
        let nonce_slice = GenericArray::from_slice(nonce_slice);

        // the key_version is in-the-clear, so we include it in the integrity check
        // using the "additonal data"
        let ad_buf = key_version.to_be_bytes();
        let ad_slice = &ad_buf[..];

        // split buffer into separate ciphertext and authentication tag slices
        let auth_tag_offset = enc_addr.len() - AES_GCM_TAG_LEN;
        let (enc_addr_slice, auth_tag_slice) = enc_addr.split_at_mut(auth_tag_offset);
        let auth_tag_slice = GenericArray::from_slice(auth_tag_slice);

        aead.decrypt_in_place_detached(nonce_slice, ad_slice, enc_addr_slice, auth_tag_slice)
            .map_err(|_| DecryptError)?;

        // remove the auth tag suffix, leaving just the decrypted network address
        enc_addr.truncate(auth_tag_offset);

        Ok(RawNetworkAddress::new(enc_addr))
    }

    /// Given the shared `shared_val_netaddr_key`, derive the per-validator
    /// `derived_key`.
    fn derive_key(shared_val_netaddr_key: &Key, account: &AccountAddress) -> Vec<u8> {
        let salt = Some(HKDF_SALT.as_ref());
        let info = Some(account.as_ref());
        Hkdf::<Sha3_256>::extract_then_expand(salt, shared_val_netaddr_key, info, KEY_LEN).expect(
            "HKDF_SHA3_256 extract_then_expand is infallible here since all inputs \
             have valid and well-defined lengths enforced by the type system",
        )
    }

    pub fn key_version(&self) -> KeyVersion {
        self.key_version
    }

    pub fn seq_num(&self) -> u64 {
        self.seq_num
    }
}

impl TryFrom<&RawEncNetworkAddress> for EncNetworkAddress {
    type Error = lcs::Error;

    fn try_from(value: &RawEncNetworkAddress) -> Result<Self, lcs::Error> {
        let enc_addr: EncNetworkAddress = lcs::from_bytes(value.as_ref())?;
        Ok(enc_addr)
    }
}

#[cfg(any(test, feature = "fuzzing"))]
impl Arbitrary for EncNetworkAddress {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        let shared_val_netaddr_key = TEST_SHARED_VAL_NETADDR_KEY;
        let key_version = TEST_SHARED_VAL_NETADDR_KEY_VERSION;
        let account = AccountAddress::ZERO;
        let seq_num = 0;
        let addr_idx = 0;

        any::<RawNetworkAddress>()
            .prop_map(move |addr| {
                EncNetworkAddress::encrypt(
                    addr,
                    &shared_val_netaddr_key,
                    key_version,
                    &account,
                    seq_num,
                    addr_idx,
                )
            })
            .boxed()
    }
}

///////////
// Tests //
///////////

#[cfg(test)]
mod test {
    use super::*;
    use crate::NetworkAddress;

    // Ensure that modifying the ciphertext or associated data causes a decryption
    // error.
    #[test]
    fn expect_decryption_failures() {
        let shared_val_netaddr_key = TEST_SHARED_VAL_NETADDR_KEY;
        let key_version = TEST_SHARED_VAL_NETADDR_KEY_VERSION;
        let account = AccountAddress::ZERO;
        let seq_num = 0x4589;
        let addr_idx = 123;
        let addr = NetworkAddress::mock();
        let raw_addr = RawNetworkAddress::try_from(&addr).unwrap();
        let enc_addr = raw_addr.clone().encrypt(
            &shared_val_netaddr_key,
            key_version,
            &account,
            seq_num,
            addr_idx,
        );

        // we expect decrypting a properly encrypted address to work
        let dec_addr = enc_addr
            .clone()
            .decrypt(&shared_val_netaddr_key, &account, addr_idx)
            .unwrap();
        assert_eq!(raw_addr, dec_addr);

        // modifying the seq_num should cause decryption failure
        let mut malicious_enc_addr = enc_addr.clone();
        malicious_enc_addr.seq_num = 1234;
        malicious_enc_addr
            .decrypt(&shared_val_netaddr_key, &account, addr_idx)
            .unwrap_err();

        // modifying the key_version should cause decryption failure
        let mut malicious_enc_addr = enc_addr.clone();
        malicious_enc_addr.key_version = 9999;
        malicious_enc_addr
            .decrypt(&shared_val_netaddr_key, &account, addr_idx)
            .unwrap_err();

        // modifying the auth_tag should cause decryption failure
        let mut malicious_enc_addr = enc_addr.clone();
        let buf = &mut malicious_enc_addr.enc_addr;
        let buf_len = buf.len();
        buf[buf_len - 1] ^= 0x55;
        malicious_enc_addr
            .decrypt(&shared_val_netaddr_key, &account, addr_idx)
            .unwrap_err();

        // modifying the enc_addr ciphertext should cause decryption failure
        let mut malicious_enc_addr = enc_addr.clone();
        malicious_enc_addr.enc_addr = vec![0x42u8; 123];
        malicious_enc_addr
            .decrypt(&shared_val_netaddr_key, &account, addr_idx)
            .unwrap_err();

        // modifying the account address should cause decryption failure
        let malicious_account = AccountAddress::new([0x33; AccountAddress::LENGTH]);
        enc_addr
            .clone()
            .decrypt(&shared_val_netaddr_key, &malicious_account, addr_idx)
            .unwrap_err();

        // modifying the shared_val_netaddr_key should cause decryption failure
        let malicious_shared_val_netaddr_key = [0x88; KEY_LEN];
        enc_addr
            .clone()
            .decrypt(&malicious_shared_val_netaddr_key, &account, addr_idx)
            .unwrap_err();

        // modifying the addr_idx should cause decryption failure
        let malicious_addr_idx = 999;
        enc_addr
            .decrypt(&shared_val_netaddr_key, &account, malicious_addr_idx)
            .unwrap_err();
    }

    proptest! {
        #[test]
        fn encrypt_decrypt_roundtrip(
            raw_addr in any::<RawNetworkAddress>(),
        ) {
            let shared_val_netaddr_key = TEST_SHARED_VAL_NETADDR_KEY;
            let key_version = TEST_SHARED_VAL_NETADDR_KEY_VERSION;
            let account = AccountAddress::ZERO;
            let seq_num = 0;
            let addr_idx = 0;
            let enc_addr = raw_addr.clone().encrypt(&shared_val_netaddr_key, key_version, &account, seq_num, addr_idx);
            let dec_addr = enc_addr.decrypt(&shared_val_netaddr_key, &account, addr_idx).unwrap();
            assert_eq!(raw_addr, dec_addr);
        }

        #[test]
        fn encrypt_decrypt_roundtrip_all_parameters(
            shared_val_netaddr_key in any::<Key>(),
            key_version in any::<KeyVersion>(),
            account in any::<[u8; AccountAddress::LENGTH]>(),
            seq_num in any::<u64>(),
            addr_idx in any::<u32>(),
            raw_addr in any::<RawNetworkAddress>(),
        ) {
            let account = AccountAddress::new(account);
            let enc_addr = raw_addr.clone().encrypt(&shared_val_netaddr_key, key_version, &account, seq_num, addr_idx);
            let dec_addr = enc_addr.decrypt(&shared_val_netaddr_key, &account, addr_idx).unwrap();
            assert_eq!(raw_addr, dec_addr);
        }

        #[test]
        fn enc_network_address_serde(enc_addr in any::<EncNetworkAddress>()) {
            let raw_enc_addr = RawEncNetworkAddress::try_from(&enc_addr).unwrap();
            let enc_addr_2 = EncNetworkAddress::try_from(&raw_enc_addr).unwrap();
            assert_eq!(enc_addr, enc_addr_2);
        }
    }
}
