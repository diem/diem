// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::transaction_argument::*;

#[test]
fn parse_u64() {
    for s in &["0", "42", "18446744073709551615"] {
        parse_as_u64(s).unwrap();
    }
    for s in &["xx", "", "-3"] {
        parse_as_u64(s).unwrap_err();
    }
}

#[test]
fn parse_address() {
    for s in &[
        "0x0",
        "0x1",
        "0x00",
        "0x05",
        "0x100",
        "0xAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
    ] {
        parse_as_address(s).unwrap();
    }

    for s in &[
        "0x",
        "100",
        "",
        "0xG",
        "0xBBAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
    ] {
        parse_as_address(s).unwrap_err();
    }
}

#[test]
fn parse_byte_array() {
    for s in &["0", "00", "deadbeef", "aaa"] {
        parse_as_byte_array(&format!("b\"{}\"", s)).unwrap();
    }

    for s in &["", "b\"\"", "123", "b\"G\""] {
        parse_as_byte_array(s).unwrap_err();
    }
}

#[test]
fn parse_args() {
    for s in &["123", "0xf", "b\"aaa\""] {
        parse_as_transaction_argument(s).unwrap();
    }

    for s in &["garbage", ""] {
        parse_as_transaction_argument(s).unwrap_err();
    }
}
