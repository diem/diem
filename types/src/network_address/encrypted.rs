// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_address::AccountAddress,
    network_address::{NetworkAddress, ParseError},
};
use aes_gcm::{
    aead::{generic_array::GenericArray, AeadInPlace, NewAead},
    Aes256Gcm,
};
use diem_crypto::{compat::Sha3_256, hkdf::Hkdf};
#[cfg(any(test, feature = "fuzzing"))]
use proptest::prelude::*;
use serde::{Deserialize, Serialize};
use std::mem;

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
/// `"7ffda2ae982a2ebfab2a4da62f76fe33592c85e02445b875f02ded51a520ba2a"` which is
/// also equal to the hash value `SHA3-256(b"DIEM_ENCRYPTED_NETWORK_ADDRESS_SALT")`.
///
/// ```
/// use diem_types::network_address::encrypted::HKDF_SALT;
/// use diem_crypto::hash::HashValue;
///
/// let derived_salt = HashValue::sha3_256_of(b"DIEM_ENCRYPTED_NETWORK_ADDRESS_SALT");
/// assert_eq!(HKDF_SALT.as_ref(), derived_salt.as_ref());
/// ```
pub const HKDF_SALT: [u8; 32] = [
    0x7f, 0xfd, 0xa2, 0xae, 0x98, 0x2a, 0x2e, 0xbf, 0xab, 0x2a, 0x4d, 0xa6, 0x2f, 0x76, 0xfe, 0x33,
    0x59, 0x2c, 0x85, 0xe0, 0x24, 0x45, 0xb8, 0x75, 0xf0, 0x2d, 0xed, 0x51, 0xa5, 0x20, 0xba, 0x2a,
];

/// An encrypted [`NetworkAddress`].
///
/// ### Threat Model
///
/// Encrypting the on-chain network addresses is purely a defense-in-depth
/// mitigation to minimize attack surface and reduce DDoS attacks on the validators
/// by restricting the visibility of their public-facing network addresses only
/// to other validators.
///
/// These encrypted network addresses are intended to be stored on-chain under
/// each validator's advertised network addresses in their [`ValidatorConfig`]s.
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
/// where `HKDF-SHA3-256::extract_and_expand` is
/// [HKDF extract-and-expand](https://tools.ietf.org/html/rfc5869) with SHA3-256,
/// [`HKDF_SALT`] is a constant salt for application separation, `shared_val_netaddr_key` is the
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
/// [`NetworkAddress`], and `enc_addr` is the encrypted network address
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
/// [`ValidatorConfig`]: https://github.com/diem/diem/blob/main/language/diem-framework/modules/doc/ValidatorConfig.md
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct EncNetworkAddress {
    key_version: KeyVersion,
    seq_num: u64,
    #[serde(with = "serde_bytes")]
    enc_addr: Vec<u8>,
}

///////////////////////
// EncNetworkAddress //
///////////////////////

impl EncNetworkAddress {
    /// ### Panics
    ///
    /// encrypt will panic if `addr` length > 64 GiB.
    pub fn encrypt(
        addr: NetworkAddress,
        shared_val_netaddr_key: &Key,
        key_version: KeyVersion,
        account: &AccountAddress,
        seq_num: u64,
        addr_idx: u32,
    ) -> Result<Self, ParseError> {
        // unpack the NetworkAddress into its base Vec<u8>
        let mut addr_vec: Vec<u8> = bcs::to_bytes(&addr)?;

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

        Ok(Self {
            key_version,
            seq_num,
            enc_addr: addr_vec,
        })
    }

    pub fn decrypt(
        self,
        shared_val_netaddr_key: &Key,
        account: &AccountAddress,
        addr_idx: u32,
    ) -> Result<NetworkAddress, ParseError> {
        let key_version = self.key_version;
        let seq_num = self.seq_num;
        let mut enc_addr = self.enc_addr;

        // ciphertext is too small to even contain the authentication tag, so it
        // must be invalid.
        if enc_addr.len() < AES_GCM_TAG_LEN {
            return Err(ParseError::DecryptError);
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
        // using the "additional data"
        let ad_buf = key_version.to_be_bytes();
        let ad_slice = &ad_buf[..];

        // split buffer into separate ciphertext and authentication tag slices
        let auth_tag_offset = enc_addr.len() - AES_GCM_TAG_LEN;
        let (enc_addr_slice, auth_tag_slice) = enc_addr.split_at_mut(auth_tag_offset);
        let auth_tag_slice = GenericArray::from_slice(auth_tag_slice);

        aead.decrypt_in_place_detached(nonce_slice, ad_slice, enc_addr_slice, auth_tag_slice)
            .map_err(|_| ParseError::DecryptError)?;

        // remove the auth tag suffix, leaving just the decrypted network address
        enc_addr.truncate(auth_tag_offset);

        bcs::from_bytes(&enc_addr).map_err(|e| e.into())
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

        any::<NetworkAddress>()
            .prop_map(move |addr| {
                EncNetworkAddress::encrypt(
                    addr,
                    &shared_val_netaddr_key,
                    key_version,
                    &account,
                    seq_num,
                    addr_idx,
                )
                .unwrap()
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
        let enc_addr = addr
            .clone()
            .encrypt(
                &shared_val_netaddr_key,
                key_version,
                &account,
                seq_num,
                addr_idx,
            )
            .unwrap();

        // we expect decrypting a properly encrypted address to work
        let dec_addr = enc_addr
            .clone()
            .decrypt(&shared_val_netaddr_key, &account, addr_idx)
            .unwrap();
        assert_eq!(addr, dec_addr);

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
            addr in any::<NetworkAddress>(),
        ) {
            let shared_val_netaddr_key = TEST_SHARED_VAL_NETADDR_KEY;
            let key_version = TEST_SHARED_VAL_NETADDR_KEY_VERSION;
            let account = AccountAddress::ZERO;
            let seq_num = 0;
            let addr_idx = 0;
            let enc_addr = addr.clone().encrypt(&shared_val_netaddr_key, key_version, &account, seq_num, addr_idx);
            let dec_addr = enc_addr.unwrap().decrypt(&shared_val_netaddr_key, &account, addr_idx);
            assert_eq!(addr, dec_addr.unwrap());
        }

        #[test]
        fn encrypt_decrypt_roundtrip_all_parameters(
            shared_val_netaddr_key in any::<Key>(),
            key_version in any::<KeyVersion>(),
            account in any::<[u8; AccountAddress::LENGTH]>(),
            seq_num in any::<u64>(),
            addr_idx in any::<u32>(),
            addr in any::<NetworkAddress>(),
        ) {
            let account = AccountAddress::new(account);
            let enc_addr = addr.clone().encrypt(&shared_val_netaddr_key, key_version, &account, seq_num, addr_idx);
            let dec_addr = enc_addr.unwrap().decrypt(&shared_val_netaddr_key, &account, addr_idx);
            assert_eq!(addr, dec_addr.unwrap());
        }
    }
}
