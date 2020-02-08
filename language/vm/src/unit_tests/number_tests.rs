// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::file_format_common::*;
use byteorder::{LittleEndian, ReadBytesExt};
use proptest::prelude::*;
use std::io::Cursor;

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

fn test_u16(value: u16, expected_bytes: usize) {
    let mut buf = BinaryData::new();
    write_u16_as_uleb128(&mut buf, value).expect("serialization should work");
    assert_eq!(buf.len(), expected_bytes);
    let buf = buf.into_inner();
    check_vector(&buf);
    let mut cursor = Cursor::new(&buf[..]);
    let val = read_uleb128_as_u16(&mut cursor).expect("deserialization should work");
    assert_eq!(value, val);
}

fn test_u32(value: u32, expected_bytes: usize) {
    let mut buf = BinaryData::new();
    write_u32_as_uleb128(&mut buf, value).expect("serialization should work");
    assert_eq!(buf.len(), expected_bytes);
    let buf = buf.into_inner();
    check_vector(&buf);
    let mut cursor = Cursor::new(&buf[..]);
    let val = read_uleb128_as_u32(&mut cursor).expect("deserialization should work");
    assert_eq!(value, val);
}

#[test]
fn lab128_u16_test() {
    test_u16(0, 1);
    test_u16(16, 1);
    test_u16(2u16.pow(7) - 1, 1);
    test_u16(2u16.pow(7), 2);
    test_u16(2u16.pow(7) + 1, 2);
    test_u16(2u16.pow(14) - 1, 2);
    test_u16(2u16.pow(14), 3);
    test_u16(2u16.pow(14) + 1, 3);
    test_u16(u16::max_value() - 2, 3);
    test_u16(u16::max_value() - 1, 3);
    test_u16(u16::max_value(), 3);
}

#[test]
fn lab128_u32_test() {
    test_u32(0, 1);
    test_u32(16, 1);
    test_u32(2u32.pow(7) - 1, 1);
    test_u32(2u32.pow(7), 2);
    test_u32(2u32.pow(7) + 1, 2);
    test_u32(2u32.pow(14) - 1, 2);
    test_u32(2u32.pow(14), 3);
    test_u32(2u32.pow(14) + 1, 3);
    test_u32(2u32.pow(21) - 1, 3);
    test_u32(2u32.pow(21), 4);
    test_u32(2u32.pow(21) + 1, 4);
    test_u32(2u32.pow(28) - 1, 4);
    test_u32(2u32.pow(28), 5);
    test_u32(2u32.pow(28) + 1, 5);
    test_u32(u32::max_value() - 2, 5);
    test_u32(u32::max_value() - 1, 5);
    test_u32(u32::max_value(), 5);
}

#[test]
fn lab128_malformed_test() {
    assert!(read_uleb128_as_u16(&mut Cursor::new(&[])).is_err());
    assert!(read_uleb128_as_u16(&mut Cursor::new(&[0x80])).is_err());
    assert!(read_uleb128_as_u16(&mut Cursor::new(&[0x80, 0x80])).is_err());
    assert!(read_uleb128_as_u16(&mut Cursor::new(&[0x80, 0x80, 0x80, 0x80])).is_err());
    assert!(read_uleb128_as_u16(&mut Cursor::new(&[0x80, 0x80, 0x80, 0x2])).is_err());

    assert!(read_uleb128_as_u32(&mut Cursor::new(&[])).is_err());
    assert!(read_uleb128_as_u32(&mut Cursor::new(&[0x80])).is_err());
    assert!(read_uleb128_as_u32(&mut Cursor::new(&[0x80, 0x80])).is_err());
    assert!(read_uleb128_as_u32(&mut Cursor::new(&[0x80, 0x80, 0x80, 0x80])).is_err());
    assert!(read_uleb128_as_u32(&mut Cursor::new(&[0x80, 0x80, 0x80, 0x80, 0x80, 0x2])).is_err());
}

proptest! {
    #[test]
    fn u16_uleb128_roundtrip(input in any::<u16>()) {
        let mut serialized = BinaryData::new();
        write_u16_as_uleb128(&mut serialized, input).expect("serialization should work");
        let serialized = serialized.into_inner();
        let mut cursor = Cursor::new(&serialized[..]);
        let output = read_uleb128_as_u16(&mut cursor).expect("deserialization should work");
        prop_assert_eq!(input, output);
    }

    #[test]
    fn u32_uleb128_roundtrip(input in any::<u32>()) {
        let mut serialized = BinaryData::new();
        write_u32_as_uleb128(&mut serialized, input).expect("serialization should work");
        let serialized = serialized.into_inner();
        let mut cursor = Cursor::new(&serialized[..]);
        let output = read_uleb128_as_u32(&mut cursor).expect("deserialization should work");
        prop_assert_eq!(input, output);
    }

    #[test]
    fn u16_roundtrip(input in any::<u16>()) {
        let mut serialized = BinaryData::new();
        write_u16(&mut serialized, input).expect("serialization should work");
        let serialized = serialized.into_inner();
        let mut cursor = Cursor::new(&serialized[..]);
        let output = cursor.read_u16::<LittleEndian>().expect("deserialization should work");
        prop_assert_eq!(input, output);
    }

    #[test]
    fn u32_roundtrip(input in any::<u32>()) {
        let mut serialized = BinaryData::new();
        write_u32(&mut serialized, input).expect("serialization should work");
        let serialized = serialized.into_inner();
        let mut cursor = Cursor::new(&serialized[..]);
        let output = cursor.read_u32::<LittleEndian>().expect("deserialization should work");
        prop_assert_eq!(input, output);
    }

    #[test]
    fn u64_roundtrip(input in any::<u64>()) {
        let mut serialized = BinaryData::new();
        write_u64(&mut serialized, input).expect("serialization should work");
        let serialized = serialized.into_inner();
        let mut cursor = Cursor::new(&serialized[..]);
        let output = cursor.read_u64::<LittleEndian>().expect("deserialization should work");
        prop_assert_eq!(input, output);
    }
}
