// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::file_format_common::*;
use proptest::prelude::*;
use std::io::{Cursor, Read};

// verify all bytes in the vector have the high bit set except the last one
fn check_vector(buf: &[u8]) {
    let mut last_byte: bool = false;
    for byte in buf {
        assert!(!last_byte);
        if *byte & 0x80 == 0 {
            last_byte = true;
        }
        if !last_byte {
            assert!(*byte & 0x80 > 0, "{} & 0x80", *byte);
        }
    }
    assert!(last_byte);
}

fn uleb128_test_u64(value: u64, expected_bytes: usize) {
    let mut buf = BinaryData::new();
    write_u64_as_uleb128(&mut buf, value).expect("serialization should work");
    assert_eq!(buf.len(), expected_bytes);
    let buf = buf.into_inner();
    check_vector(&buf);
    let mut cursor = Cursor::new(&buf[..]);
    let val = read_uleb128_as_u64(&mut cursor).expect("deserialization should work");
    assert_eq!(value, val);
}

#[test]
fn uleb128_test() {
    uleb128_test_u64(0, 1);
    let mut n: usize = 1;
    while n * 7 < 64 {
        let exp = (n * 7) as u32;
        uleb128_test_u64(2u64.pow(exp) - 1, n);
        uleb128_test_u64(2u64.pow(exp), n + 1);
        n += 1;
    }
    uleb128_test_u64(u64::max_value() - 1, 10);
    uleb128_test_u64(u64::max_value(), 10);
}

#[test]
fn uleb128_malformed_test() {
    assert!(read_uleb128_as_u64(&mut Cursor::new(&[])).is_err());
    assert!(read_uleb128_as_u64(&mut Cursor::new(&[0x80])).is_err());
    assert!(read_uleb128_as_u64(&mut Cursor::new(&[0x80, 0x80])).is_err());
    assert!(read_uleb128_as_u64(&mut Cursor::new(&[0x80, 0x80, 0x80, 0x80])).is_err());
    assert!(read_uleb128_as_u64(&mut Cursor::new(&[
        0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x2
    ]))
    .is_err());
}

#[test]
fn uleb128_canonicity_test() {
    assert!(read_uleb128_as_u64(&mut Cursor::new(&[0x80, 0x00])).is_err());
    assert!(read_uleb128_as_u64(&mut Cursor::new(&[0x80, 0x00, 0x00])).is_err());
    assert!(read_uleb128_as_u64(&mut Cursor::new(&[0x80, 0x80, 0x80, 0x80, 0x0f])).is_ok());
    assert!(read_uleb128_as_u64(&mut Cursor::new(&[
        0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x1
    ]))
    .is_ok());
}

proptest! {
    #[test]
    fn uleb128_roundtrip(input in any::<u64>()) {
        let mut serialized = BinaryData::new();
        write_u64_as_uleb128(&mut serialized, input).expect("serialization should work");
        let serialized = serialized.into_inner();
        let mut cursor = Cursor::new(&serialized[..]);
        let output = read_uleb128_as_u64(&mut cursor).expect("deserialization should work");
        prop_assert_eq!(input, output);
    }

    #[test]
    fn u16_roundtrip(input in any::<u16>()) {
        let mut serialized = BinaryData::new();
        write_u16(&mut serialized, input).expect("serialization should work");
        let serialized = serialized.into_inner();
        let mut cursor = Cursor::new(&serialized[..]);
        let mut u16_bytes = [0; 2];
        cursor.read_exact(&mut u16_bytes).expect("deserialization should work");
        let output = u16::from_le_bytes(u16_bytes);
        prop_assert_eq!(input, output);
    }

    #[test]
    fn u32_roundtrip(input in any::<u32>()) {
        let mut serialized = BinaryData::new();
        write_u32(&mut serialized, input).expect("serialization should work");
        let serialized = serialized.into_inner();
        let mut cursor = Cursor::new(&serialized[..]);
        let mut u32_bytes = [0; 4];
        cursor.read_exact(&mut u32_bytes).expect("deserialization should work");
        let output = u32::from_le_bytes(u32_bytes);
        prop_assert_eq!(input, output);
    }

    #[test]
    fn u64_roundtrip(input in any::<u64>()) {
        let mut serialized = BinaryData::new();
        write_u64(&mut serialized, input).expect("serialization should work");
        let serialized = serialized.into_inner();
        let mut cursor = Cursor::new(&serialized[..]);
        let mut u64_bytes = [0; 8];
        cursor.read_exact(&mut u64_bytes).expect("deserialization should work");
        let output = u64::from_le_bytes(u64_bytes);
        prop_assert_eq!(input, output);
    }
}
