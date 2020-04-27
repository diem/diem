// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_address::AccountAddress;
use hex::FromHex;
use libra_crypto::{hash::CryptoHash, HashValue};
use proptest::prelude::*;
use std::convert::{AsRef, TryFrom};

#[test]
fn test_address_bytes() {
    let hex = Vec::from_hex("ca843279e3427144cead5e4d5999a3d0")
        .expect("You must provide a valid Hex format");

    assert_eq!(
        hex.len(),
        AccountAddress::LENGTH as usize,
        "Address {:?} is not {}-bytes long. Addresses must be {} bytes",
        hex,
        AccountAddress::LENGTH,
        AccountAddress::LENGTH,
    );
    let address = AccountAddress::try_from(&hex[..]).unwrap_or_else(|_| {
        panic!(
            "The address {:?} is of invalid length. Addresses must be 16-bytes long",
            &hex
        )
    });

    assert_eq!(address.as_ref().to_vec(), hex);
}

#[test]
fn test_address() {
    let hex = Vec::from_hex("ca843279e3427144cead5e4d5999a3d0")
        .expect("You must provide a valid Hex format");

    assert_eq!(
        hex.len(),
        AccountAddress::LENGTH as usize,
        "Address {:?} is not {}-bytes long. Addresses must be {} bytes",
        hex,
        AccountAddress::LENGTH,
        AccountAddress::LENGTH,
    );

    let address: AccountAddress = AccountAddress::try_from(&hex[..]).unwrap_or_else(|_| {
        panic!(
            "The address {:?} is of invalid length. Addresses must be 16-bytes long",
            &hex
        )
    });

    let hash_vec =
        &Vec::from_hex("2f06a55a39cbe1bdfbb3a1ca7a770674af51849d48a9ce5a9beb6f82bde14e6f")
            .expect("You must provide a valid Hex format");

    let mut hash = [0u8; 32];
    let bytes = &hash_vec[..32];
    hash.copy_from_slice(&bytes);

    assert_eq!(address.hash(), HashValue::new(hash));
    assert_eq!(address.as_ref().to_vec(), hex);
}

#[test]
fn test_ref() {
    let address = AccountAddress::new([1u8; AccountAddress::LENGTH]);
    let _: &[u8] = address.as_ref();
}

#[test]
fn test_address_from_proto_invalid_length() {
    let bytes = vec![1; 123];
    assert!(AccountAddress::try_from(&bytes[..]).is_err());
}

#[test]
fn test_deserialize_from_json_value() {
    let address = AccountAddress::random();
    let json_value = serde_json::to_value(address).expect("serde_json::to_value fail.");
    let address2: AccountAddress =
        serde_json::from_value(json_value).expect("serde_json::from_value fail.");
    assert_eq!(address, address2)
}

proptest! {
    #[test]
    fn test_address_string_roundtrip(addr in any::<AccountAddress>()) {
        let s = String::from(&addr);
        let addr2 = AccountAddress::try_from(s).expect("roundtrip to string should work");
        prop_assert_eq!(addr, addr2);
    }

    #[test]
    fn test_address_protobuf_roundtrip(addr in any::<AccountAddress>()) {
        let bytes = addr.to_vec();
        prop_assert_eq!(bytes.clone(), addr.as_ref());
        let addr2 = AccountAddress::try_from(&bytes[..]).unwrap();
        prop_assert_eq!(addr, addr2);
    }
}
