// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_address::{AccountAddress, ADDRESS_LENGTH};
use hex::FromHex;
use libra_crypto::{hash::CryptoHash, HashValue};
use proptest::prelude::*;
use std::convert::{AsRef, TryFrom};

#[test]
fn test_address_bytes() {
    let hex = Vec::from_hex("ca843279e3427144cead5e4d5999a3d0ccf92b8e124793820837625638742903")
        .expect("You must provide a valid Hex format");

    assert_eq!(
        hex.len(),
        ADDRESS_LENGTH as usize,
        "Address {:?} is not {}-bytes long. Addresses must be {} bytes",
        hex,
        ADDRESS_LENGTH,
        ADDRESS_LENGTH,
    );
    let address = AccountAddress::try_from(&hex[..]).unwrap_or_else(|_| {
        panic!(
            "The address {:?} is of invalid length. Addresses must be 32-bytes long",
            &hex
        )
    });

    assert_eq!(address.as_ref().to_vec(), hex);
}

#[test]
fn test_address() {
    let hex = Vec::from_hex("ca843279e3427144cead5e4d5999a3d0ccf92b8e124793820837625638742903")
        .expect("You must provide a valid Hex format");

    assert_eq!(
        hex.len(),
        ADDRESS_LENGTH as usize,
        "Address {:?} is not {}-bytes long. Addresses must be {} bytes",
        hex,
        ADDRESS_LENGTH,
        ADDRESS_LENGTH,
    );

    let address: AccountAddress = AccountAddress::try_from(&hex[..]).unwrap_or_else(|_| {
        panic!(
            "The address {:?} is of invalid length. Addresses must be 32-bytes long",
            &hex
        )
    });

    let hash_vec =
        &Vec::from_hex("84a1bb90a6130da458abde12cc8ea21f29c6e0bcda007491fff1852561b830a7")
            .expect("You must provide a valid Hex format");

    let mut hash = [0u8; 32];
    let bytes = &hash_vec[..32];
    hash.copy_from_slice(&bytes);

    assert_eq!(address.hash(), HashValue::new(hash));
    assert_eq!(address.as_ref().to_vec(), hex);
}

#[test]
fn test_ref() {
    let address = AccountAddress::new([1u8; 32]);
    let _: &[u8] = address.as_ref();
}

#[test]
fn test_bech32() {
    let address = AccountAddress::try_from(
        &Vec::from_hex("269bdde7f42c25476707821eb44d5ce3c6c9e50a774f43ddebc5494a42870aa6")
            .expect("You must provide a valid Hex format")[..],
    )
    .expect("Address is not a valid hex format");
    let bech32 = address.to_bech32_literal();
    assert_eq!(
        bech32.to_string(),
        "lb1y6damel59sj5wec8sg0tgn2uu0rvneg2wa858h0tc4y55s58p2nqjyd2lr".to_string()
    );
    let bech32_address = AccountAddress::from_bech32_literal(&bech32)
        .expect("The provided input string is not a valid bech32 format");
    assert_eq!(
        address.as_ref().to_vec(),
        bech32_address.as_ref().to_vec(),
        "The two addresses do not match",
    );
}

#[test]
fn test_address_from_proto_invalid_length() {
    let bytes = vec![1; 123];
    assert!(AccountAddress::try_from(&bytes[..]).is_err());
}

proptest! {
    #[test]
    fn test_address_string_roundtrip(addr in any::<AccountAddress>()) {
        let s = String::from(&addr);
        let addr2 = AccountAddress::try_from(s).expect("roundtrip to string should work");
        prop_assert_eq!(addr, addr2);
    }

    #[test]
    fn test_address_bech32_roundtrip(addr in any::<AccountAddress>()) {
        let b = addr.to_bech32_literal();
        let addr2 = AccountAddress::from_bech32_literal(&b).expect("Address::from_bech32_literal should work");
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
