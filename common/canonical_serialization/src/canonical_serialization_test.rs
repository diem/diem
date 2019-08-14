// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//https://rust-lang.github.io/rust-clippy/master/index.html#blacklisted_name
//disable it in test so that we can use variable names such as 'foo' and 'bar'
#![allow(clippy::blacklisted_name)]
#![allow(clippy::many_single_char_names)]

use super::*;
use byteorder::WriteBytesExt;
use failure::Result;
use std::u32;

// Do not change the test vectors. Please read the comment below.
const TEST_VECTOR_1: &str = "ffffffffffffffff060000006463584d4237640000000000000009000000000102\
                             03040506070805050505050505050505050505050505050505050505050505050505\
                             05050505630000000103000000010000000103000000161543030000000038150300\
                             0000160a05040000001415596903000000c9175a";

// Why do we need test vectors?
//
// 1. Sometimes it helps to catch common bugs between serialization and
// deserialization functions that would have been missed by a simple round trip test.
// For example, if there's a bug in a shared procedure that serializes and
// deserialize both calls then roundtrip might miss it.
//
// 2. It helps to catch code changes that inadvertently introduce breaking changes
// in the serialization format that is incompatible with what generated in the
// past which would be missed by roundtrip tests, or changes that are not backward
// compatible in the sense that it may fail to deserialize bytes generated in the past.

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Addr(pub [u8; 32]);

impl Addr {
    fn new(bytes: [u8; 32]) -> Self {
        Addr(bytes)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Foo {
    a: u64,
    b: Vec<u8>,
    c: Bar,
    d: bool,
    e: BTreeMap<Vec<u8>, Vec<u8>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Bar {
    a: u64,
    b: Vec<u8>,
    c: Addr,
    d: u32,
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

impl CanonicalSerialize for Bar {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer
            .encode_u64(self.a)?
            .encode_variable_length_bytes(&self.b)?
            .encode_raw_bytes(&self.c.0)?
            .encode_u32(self.d)?;
        Ok(())
    }
}

impl CanonicalDeserialize for Foo {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self> {
        let a = deserializer.decode_u64()?;
        let b = deserializer.decode_variable_length_bytes()?;
        let c: Bar = deserializer.decode_struct::<Bar>()?;
        let d: bool = deserializer.decode_bool()?;
        let e: BTreeMap<Vec<u8>, Vec<u8>> = deserializer.decode_btreemap()?;
        Ok(Foo { a, b, c, d, e })
    }
}

impl CanonicalDeserialize for Bar {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self> {
        let a = deserializer.decode_u64()?;
        let b = deserializer.decode_variable_length_bytes()?;
        let c = deserializer.decode_bytes_with_len(32)?;
        let mut cc: [u8; 32] = [0; 32];
        cc.copy_from_slice(c.as_slice());

        let d = deserializer.decode_u32()?;
        Ok(Bar {
            a,
            b,
            c: Addr::new(cc),
            d,
        })
    }
}

#[test]
fn test_btreemap_encode() {
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
    assert_eq!(deserializer.raw_bytes.read_u32::<Endianness>().unwrap(), 4);
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
fn test_serialization_roundtrip() {
    let bar = Bar {
        a: 50,
        b: vec![10u8; 100],
        c: Addr::new([3u8; 32]),
        d: 12,
    };

    let mut map = BTreeMap::new();
    map.insert(vec![0, 56, 21], vec![22, 10, 5]);
    map.insert(vec![1], vec![22, 21, 67]);
    map.insert(vec![20, 21, 89, 105], vec![201, 23, 90]);

    let foo = Foo {
        a: 1,
        b: vec![32, 41, 190, 200, 2, 5, 90, 100, 123, 234, 159, 159, 101],
        c: bar,
        d: false,
        e: map,
    };

    let mut serializer = SimpleSerializer::<Vec<u8>>::new();
    foo.serialize(&mut serializer).unwrap();
    let serialized_bytes = serializer.get_output();

    let mut deserializer = SimpleDeserializer::new(&serialized_bytes);
    let deserialized_foo = Foo::deserialize(&mut deserializer).unwrap();
    assert_eq!(foo, deserialized_foo);
    assert_eq!(
        deserializer.raw_bytes.position(),
        deserializer.raw_bytes.get_ref().len() as u64
    );
}

#[test]
fn test_serialization_optional() {
    let bar1: Option<u32> = Some(42);
    let mut serializer = SimpleSerializer::<Vec<u8>>::new();
    serializer.encode_optional(&bar1).unwrap();
    let serialized_bytes = serializer.get_output();

    let mut deserializer = SimpleDeserializer::new(&serialized_bytes);
    let de_bar1: Option<u32> = deserializer.decode_optional().unwrap();
    assert_eq!(de_bar1, bar1);

    let bar2: Option<u32> = None;
    let mut serializer2 = SimpleSerializer::<Vec<u8>>::new();
    serializer2.encode_optional(&bar2).unwrap();
    let serialized_bytes2 = serializer2.get_output();

    let mut deserializer2 = SimpleDeserializer::new(&serialized_bytes2);
    let de_bar2: Option<u32> = deserializer2.decode_optional().unwrap();
    assert_eq!(de_bar2, bar2);
}

#[test]
fn test_encode_vec() {
    let bar1 = Bar {
        a: 55,
        b: vec![10u8; 100],
        c: Addr::new([3u8; 32]),
        d: 77,
    };
    let bar2 = Bar {
        a: 123,
        b: vec![1, 5, 20],
        c: Addr::new([8u8; 32]),
        d: 127,
    };

    let mut vec = Vec::new();
    vec.push(bar1.clone());
    vec.push(bar2.clone());
    let mut serializer = SimpleSerializer::<Vec<u8>>::new();
    serializer.encode_vec(&vec).unwrap();
    let serialized_bytes = serializer.get_output();

    let de_vec: Vec<Bar> = SimpleDeserializer::deserialize(&serialized_bytes).unwrap();

    assert_eq!(2, de_vec.len());
    assert_eq!(bar1, de_vec[0]);
    assert_eq!(bar2, de_vec[1]);

    // test Vec<T> implementation
    let mut serializer = SimpleSerializer::<Vec<u8>>::new();
    serializer.encode_struct(&vec).unwrap();
    let serialized_bytes = serializer.get_output();
    let de_vec: Vec<Bar> = SimpleDeserializer::deserialize(&serialized_bytes).unwrap();

    assert_eq!(2, de_vec.len());
    assert_eq!(bar1, de_vec[0]);
    assert_eq!(bar2, de_vec[1]);
}

#[test]
fn test_vec_impl() {
    let mut vec: Vec<i32> = Vec::new();
    vec.push(std::i32::MIN);
    vec.push(std::i32::MAX);
    vec.push(100);

    let mut serializer = SimpleSerializer::<Vec<u8>>::new();
    serializer.encode_struct(&vec).unwrap();
    let serialized_bytes = serializer.get_output();
    let de_vec: Vec<i32> = SimpleDeserializer::deserialize(&serialized_bytes).unwrap();
    assert_eq!(vec, de_vec);
}

#[test]
fn test_vectors_1() {
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
    assert_eq!(TEST_VECTOR_1, hex::encode(serialized_bytes));

    // make sure we can deserialize the test vector into expected struct
    let test_vector_bytes = hex::decode(TEST_VECTOR_1).unwrap();
    let deserialized_foo: Foo = SimpleDeserializer::deserialize(&test_vector_bytes).unwrap();
    assert_eq!(foo, deserialized_foo);
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

#[test]
fn test_tuples() {
    let input: (u32, u32) = (123, 456);
    let mut serializer = SimpleSerializer::<Vec<u8>>::new();
    serializer.encode_tuple2(&input).unwrap();
    let serialized_bytes = serializer.get_output();

    let mut deserializer = SimpleDeserializer::new(&serialized_bytes);
    let output: Result<(u32, u32)> = deserializer.decode_tuple2();
    assert!(output.is_ok());
    assert_eq!(output.unwrap(), input);

    let bad_output: Result<(u32, u32)> = deserializer.decode_tuple2();
    assert!(bad_output.is_err());
}

#[test]
fn test_nested_tuples() {
    let input: Vec<(u32, u32)> = vec![(123, 456)];
    let mut serializer = SimpleSerializer::<Vec<u8>>::new();
    serializer.encode_vec(&input).unwrap();
    let serialized_bytes = serializer.get_output();

    let mut deserializer = SimpleDeserializer::new(&serialized_bytes);
    let output: Result<Vec<(u32, u32)>> = deserializer.decode_vec();
    assert!(output.is_ok());
    assert_eq!(output.unwrap(), input);

    let bad_output: Result<(u32, u32)> = deserializer.decode_tuple2();
    assert!(bad_output.is_err());
}

#[test]
fn test_strings() {
    let input: &'static str = "Hello, World!";
    let mut serializer = SimpleSerializer::<Vec<u8>>::new();
    serializer.encode_string(input).unwrap();
    let serialized_bytes = serializer.get_output();

    let mut deserializer = SimpleDeserializer::new(&serialized_bytes);
    let output: Result<String> = deserializer.decode_string();
    assert_eq!(output.unwrap(), input);
}
