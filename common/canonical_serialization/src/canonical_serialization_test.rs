// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Required to allow Arbitrary
#![allow(clippy::unit_arg)]

//https://rust-lang.github.io/rust-clippy/master/index.html#blacklisted_name
//disable it in test so that we can use variable names such as 'foo' and 'bar'
#![allow(clippy::blacklisted_name)]
#![allow(clippy::many_single_char_names)]

use crate::*;
use byteorder::WriteBytesExt;
use failure::Result;
use proptest::prelude::*;
use proptest_derive::Arbitrary;
use std::collections::BTreeMap;

// Do not change this test vector. It is used to verify correctness of the serializer.
const TEST_VECTOR: &str = "ffffffffffffffff060000006463584d4237640000000000000009000000000102\
                           03040506070805050505050505050505050505050505050505050505050505050505\
                           05050505630000000103000000010000000103000000161543030000000038150300\
                           0000160a05040000001415596903000000c9175a";

#[derive(Arbitrary, Clone, Debug, Eq, PartialEq)]
pub struct Addr(pub [u8; 32]);

impl Addr {
    fn new(bytes: [u8; 32]) -> Self {
        Addr(bytes)
    }
}

impl CanonicalDeserialize for Addr {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self> {
        let mut data_slice: [u8; 32] = [0; 32];
        let data_decoded = deserializer.decode_bytes_with_len(32)?;
        data_slice.copy_from_slice(data_decoded.as_slice());
        Ok(Addr::new(data_slice))
    }
}

impl CanonicalSerialize for Addr {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer.encode_raw_bytes(&self.0)?;
        Ok(())
    }
}

#[derive(Arbitrary, Clone, Debug, Eq, PartialEq)]
struct Bar {
    a: u64,
    b: Vec<u8>,
    c: Addr,
    d: u32,
}

impl CanonicalDeserialize for Bar {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self> {
        Ok(Bar {
            a: deserializer.decode_u64()?,
            b: deserializer.decode_variable_length_bytes()?,
            c: deserializer.decode_struct()?,
            d: deserializer.decode_u32()?,
        })
    }
}

impl CanonicalSerialize for Bar {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer
            .encode_u64(self.a)?
            .encode_variable_length_bytes(&self.b)?
            .encode_struct(&self.c)?
            .encode_u32(self.d)?;
        Ok(())
    }
}

#[derive(Arbitrary, Clone, Debug, Eq, PartialEq)]
struct Foo {
    a: u64,
    b: Vec<u8>,
    c: Bar,
    d: bool,
    e: BTreeMap<Vec<u8>, Vec<u8>>,
}

impl CanonicalDeserialize for Foo {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self> {
        Ok(Foo {
            a: deserializer.decode_u64()?,
            b: deserializer.decode_variable_length_bytes()?,
            c: deserializer.decode_struct()?,
            d: deserializer.decode_bool()?,
            e: deserializer.decode_btreemap()?,
        })
    }
}

impl CanonicalSerialize for Foo {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer
            .encode_u64(self.a)?
            .encode_variable_length_bytes(&self.b)?
            .encode_struct(&self.c)?
            .encode_bool(self.d)?
            .encode_btreemap(&self.e)?;
        Ok(())
    }
}

proptest! {
    #[test]
    fn serializer_bar(value in any::<Bar>()) {
        test_helper::assert_canonical_encode_decode(&value);
    }
    #[test]
    fn serializer_bool(value in any::<bool>()) {
        test_helper::assert_canonical_encode_decode(&value);
    }

    #[test]
    fn serialize_btreemap(value in any::<BTreeMap<Vec<u8>, Vec<u8>>>()) {
        test_helper::assert_canonical_encode_decode(&value);
    }

    #[test]
    fn serialize_byte_array(value in any::<Vec<u8>>()) {
        test_helper::assert_canonical_encode_decode(&value);
    }

    #[test]
    fn serializer_foo(value in any::<Foo>()) {
        test_helper::assert_canonical_encode_decode(&value);
    }

    #[test]
    fn serialize_i8(value in any::<i8>()) {
        test_helper::assert_canonical_encode_decode(&value);
    }

    #[test]
    fn serialize_i16(value in any::<i16>()) {
        test_helper::assert_canonical_encode_decode(&value);
    }

    #[test]
    fn serialize_i32(value in any::<i32>()) {
        test_helper::assert_canonical_encode_decode(&value);
    }

    #[test]
    fn serialize_i64(value in any::<i64>()) {
        test_helper::assert_canonical_encode_decode(&value);
    }

    #[test]
    fn serialize_string(value in any::<String>()) {
        test_helper::assert_canonical_encode_decode(&value);
    }

    #[test]
    fn serialize_tuple2(value in any::<(i16, String)>()) {
        test_helper::assert_canonical_encode_decode(&value);
    }

    #[test]
    fn serialize_tuple3(value in any::<(bool, u32, String)>()) {
        test_helper::assert_canonical_encode_decode(&value);
    }

    #[test]
    fn serialize_u8(value in any::<u8>()) {
        test_helper::assert_canonical_encode_decode(&value);
    }

    #[test]
    fn serialize_u16(value in any::<u16>()) {
        test_helper::assert_canonical_encode_decode(&value);
    }

    #[test]
    fn serialize_u32(value in any::<u32>()) {
        test_helper::assert_canonical_encode_decode(&value);
    }

    #[test]
    fn serialize_u64(value in any::<u64>()) {
        test_helper::assert_canonical_encode_decode(&value);
    }
}

#[test]
fn test_serialization_correctness_using_known_vector() {
    let bar = Bar {
        a: 100,
        b: vec![0, 1, 2, 3, 4, 5, 6, 7, 8],
        c: Addr::new([5u8; 32]),
        d: 99,
    };

    let mut map = BTreeMap::new();
    map.insert(vec![0, 56, 21], vec![22, 10, 5]);
    map.insert(vec![1], vec![22, 21, 67]);
    map.insert(vec![20, 21, 89, 105], vec![201, 23, 90]);

    let foo = Foo {
        a: u64::max_value(),
        b: vec![100, 99, 88, 77, 66, 55],
        c: bar,
        d: true,
        e: map,
    };

    let mut serializer = SimpleSerializer::<Vec<u8>>::new();
    foo.serialize(&mut serializer).unwrap();
    let serialized_bytes = serializer.get_output();

    // make sure we serialize into exact same bytes as before
    assert_eq!(TEST_VECTOR, hex::encode(serialized_bytes));

    // make sure we can deserialize the test vector into expected struct
    let test_vector_bytes = hex::decode(TEST_VECTOR).unwrap();
    let deserialized_foo: Foo = SimpleDeserializer::deserialize(&test_vector_bytes).unwrap();
    assert_eq!(foo, deserialized_foo);
}

#[test]
fn test_btreemap_lexicographic_order() {
    let mut map = BTreeMap::new();
    let value = vec![54, 20, 21, 200];
    let key1 = vec![0]; // after serialization: [1, 0]
    let key2 = vec![0, 6]; // after serialization: [2, 0, 6]
    let key3 = vec![1]; // after serialization: [1, 1]
    let key4 = vec![2]; // after serialization: [1, 2]
    map.insert(key1.clone(), value.clone());
    map.insert(key2.clone(), value.clone());
    map.insert(key3.clone(), value.clone());
    map.insert(key4.clone(), value.clone());

    let serialized_bytes = SimpleSerializer::<Vec<u8>>::serialize(&map).unwrap();

    let mut deserializer = SimpleDeserializer::new(&serialized_bytes);

    // ensure the order was encoded in lexicographic order
    assert_eq!(deserializer.decode_u32().unwrap(), 4);
    assert_eq!(deserializer.decode_variable_length_bytes().unwrap(), key1);
    assert_eq!(deserializer.decode_variable_length_bytes().unwrap(), value);
    assert_eq!(deserializer.decode_variable_length_bytes().unwrap(), key3);
    assert_eq!(deserializer.decode_variable_length_bytes().unwrap(), value);
    assert_eq!(deserializer.decode_variable_length_bytes().unwrap(), key4);
    assert_eq!(deserializer.decode_variable_length_bytes().unwrap(), value);
    assert_eq!(deserializer.decode_variable_length_bytes().unwrap(), key2);
    assert_eq!(deserializer.decode_variable_length_bytes().unwrap(), value);
}

#[test]
fn test_serialization_optional() {
    let bar1: Option<u32> = Some(42);
    test_helper::assert_canonical_encode_decode(&bar1);

    let bar2: Option<u32> = None;
    test_helper::assert_canonical_encode_decode(&bar2);
}

#[test]
fn test_serialization_failure_cases() {
    // a vec longer than representable range should result in failure
    let bar = Bar {
        a: 100,
        b: vec![0; i32::max_value() as usize + 1],
        c: Addr::new([0u8; 32]),
        d: 222,
    };

    let mut serializer = SimpleSerializer::<Vec<u8>>::new();
    assert!(bar.serialize(&mut serializer).is_err());
}

#[test]
fn test_deserialization_failure_cases() {
    // invalid length prefix should fail on all decoding methods
    let bytes_len_2 = vec![0; 2];
    let mut deserializer = SimpleDeserializer::new(&bytes_len_2);
    assert!(deserializer.clone().decode_u64().is_err());
    assert!(deserializer.clone().decode_bytes_with_len(32).is_err());
    assert!(deserializer.clone().decode_variable_length_bytes().is_err());
    assert!(deserializer.clone().decode_struct::<Foo>().is_err());
    assert!(Foo::deserialize(&mut deserializer.clone()).is_err());

    // a length prefix longer than maximum allowed should fail
    let mut long_bytes = Vec::new();
    long_bytes
        .write_u32::<Endianness>(ARRAY_MAX_LENGTH as u32 + 1)
        .unwrap();
    deserializer = SimpleDeserializer::new(&long_bytes);
    assert!(deserializer.clone().decode_variable_length_bytes().is_err());

    // vec not long enough should fail
    let mut bytes_len_10 = Vec::new();
    bytes_len_10.write_u32::<Endianness>(32).unwrap();
    deserializer = SimpleDeserializer::new(&bytes_len_10);
    assert!(deserializer.clone().decode_variable_length_bytes().is_err());
    assert!(deserializer.clone().decode_bytes_with_len(32).is_err());

    // malformed struct should fail
    let mut some_bytes = Vec::new();
    some_bytes.write_u64::<Endianness>(10).unwrap();
    some_bytes.write_u32::<Endianness>(50).unwrap();
    deserializer = SimpleDeserializer::new(&some_bytes);
    assert!(deserializer.clone().decode_struct::<Foo>().is_err());

    // malformed encoded bytes with length prefix larger than real
    let mut evil_bytes = Vec::new();
    evil_bytes.write_u32::<Endianness>(500).unwrap();
    evil_bytes.resize_with(4 + 499, Default::default);
    deserializer = SimpleDeserializer::new(&evil_bytes);
    assert!(deserializer.clone().decode_variable_length_bytes().is_err());

    // malformed encoded bool with value not 0 or 1
    let mut bool_bytes = Vec::new();
    bool_bytes.write_u8(2).unwrap();
    deserializer = SimpleDeserializer::new(&bool_bytes);
    assert!(deserializer.clone().decode_bool().is_err());
}
