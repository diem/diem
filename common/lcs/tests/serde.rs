// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

// For some reason deriving `Arbitrary` results in clippy firing a `unit_arg` violation
#![allow(clippy::unit_arg)]

use libra_canonical_serialization::{from_bytes, to_bytes, Error, MAX_SEQUENCE_LENGTH};
use proptest::prelude::*;
use proptest_derive::Arbitrary;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{collections::BTreeMap, fmt};

fn is_same<T>(t: T)
where
    T: Serialize + DeserializeOwned + fmt::Debug + PartialEq,
{
    let bytes = to_bytes(&t).unwrap();
    let s: T = from_bytes(&bytes).unwrap();
    assert_eq!(t, s);
}

//TODO deriving `Arbitrary` is currently broken for enum types
// Once AltSysrq/proptest#163 is merged we can use `Arbitrary` again.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
enum E {
    Unit,
    Newtype(u16),
    Tuple(u16, u16),
    Struct { a: u32 },
}

#[test]
fn test_enum() {
    let u = E::Unit;
    let expected = vec![0, 0, 0, 0];
    assert_eq!(to_bytes(&u).unwrap(), expected);
    is_same(u);

    let n = E::Newtype(1);
    let expected = vec![1, 0, 0, 0, 1, 0];
    assert_eq!(to_bytes(&n).unwrap(), expected);
    is_same(n);

    let t = E::Tuple(1, 2);
    let expected = vec![2, 0, 0, 0, 1, 0, 2, 0];
    assert_eq!(to_bytes(&t).unwrap(), expected);
    is_same(t);

    let s = E::Struct { a: 1 };
    let expected = vec![3, 0, 0, 0, 1, 0, 0, 0];
    assert_eq!(to_bytes(&s).unwrap(), expected);
    is_same(s);
}

#[derive(Arbitrary, Debug, Deserialize, Serialize, PartialEq)]
struct S {
    int: u16,
    option: Option<u8>,
    seq: Vec<String>,
    boolean: bool,
}

proptest! {
    #[test]
    fn proptest_bool(v in any::<bool>()) {
        assert_eq!(to_bytes(&v)?, vec![v.into()]);
        is_same(v);
    }

    #[test]
    fn proptest_i8(v in any::<i8>()) {
        assert_eq!(to_bytes(&v)?, v.to_le_bytes());
        is_same(v);
    }

    #[test]
    fn proptest_i16(v in any::<i16>()) {
        assert_eq!(to_bytes(&v)?, v.to_le_bytes());
        is_same(v);
    }

    #[test]
    fn proptest_i32(v in any::<i32>()) {
        assert_eq!(to_bytes(&v)?, v.to_le_bytes());
        is_same(v);
    }

    #[test]
    fn proptest_i64(v in any::<i64>()) {
        assert_eq!(to_bytes(&v)?, v.to_le_bytes());
        is_same(v);
    }

    #[test]
    fn proptest_i128(v in any::<i128>()) {
        assert_eq!(to_bytes(&v)?, v.to_le_bytes());
        is_same(v);
    }

    #[test]
    fn proptest_u8(v in any::<u8>()) {
        assert_eq!(to_bytes(&v)?, v.to_le_bytes());
        is_same(v);
    }

    #[test]
    fn proptest_u16(v in any::<u16>()) {
        assert_eq!(to_bytes(&v)?, v.to_le_bytes());
        is_same(v);
    }

    #[test]
    fn proptest_u32(v in any::<u32>()) {
        assert_eq!(to_bytes(&v)?, v.to_le_bytes());
        is_same(v);
    }

    #[test]
    fn proptest_u64(v in any::<u64>()) {
        assert_eq!(to_bytes(&v)?, v.to_le_bytes());
        is_same(v);
    }

    #[test]
    fn proptest_u128(v in any::<u128>()) {
        assert_eq!(to_bytes(&v)?, v.to_le_bytes());
        is_same(v);
    }

    #[test]
    fn proptest_string(v in any::<String>()) {
        let mut expected = Vec::with_capacity(v.len() + 4);
        expected.extend_from_slice(&(v.len() as u32).to_le_bytes());
        expected.extend_from_slice(v.as_bytes());
        assert_eq!(to_bytes(&v)?, expected);

        is_same(v);
    }

    #[test]
    fn proptest_vec(v in any::<Vec<u8>>()) {
        let mut expected = Vec::with_capacity(v.len() + 4);
        expected.extend_from_slice(&(v.len() as u32).to_le_bytes());
        expected.extend_from_slice(&v);
        assert_eq!(to_bytes(&v)?, expected);

        is_same(v);
    }

    #[test]
    fn proptest_option(v in any::<Option<u8>>()) {
        let expected = v.map(|v| vec![1, v]).unwrap_or_else(|| vec![0]);
        assert_eq!(to_bytes(&v)?, expected);

        is_same(v);
    }

    #[test]
    fn proptest_btreemap(v in any::<BTreeMap<Vec<u8>, Vec<u8>>>()) {
        is_same(v);
    }

    #[test]
    fn proptest_tuple2(v in any::<(i16, String)>()) {
        is_same(v);
    }

    #[test]
    fn proptest_tuple3(v in any::<(bool, u32, String)>()) {
        is_same(v);
    }

    #[test]
    fn proptest_tuple4(v in any::<(bool, u32, Option<i64>)>()) {
        is_same(v);
    }

    #[test]
    fn proptest_tuple_strings(v in any::<(String, String, String)>()) {
        is_same(v);
    }

    #[test]
    fn proptest_lexicographic_order(v in any::<BTreeMap<Vec<u8>, Vec<u8>>>()) {
        let bytes = to_bytes(&v).unwrap();

        let v: BTreeMap<Vec<u8>, Vec<u8>> = v.into_iter().map(|(k, v)| {
            let mut k_bytes = Vec::with_capacity(k.len() + 4);
            k_bytes.extend_from_slice(&(k.len() as u32).to_le_bytes());
            k_bytes.extend(k.into_iter());
            let mut v_bytes = Vec::with_capacity(v.len() + 4);
            v_bytes.extend_from_slice(&(v.len() as u32).to_le_bytes());
            v_bytes.extend(v.into_iter());

            (k_bytes, v_bytes)
        })
        .collect();

        let mut expected = Vec::with_capacity(bytes.len());
        expected.extend_from_slice(&(v.len() as u32).to_le_bytes());
        for (key, value) in v {
            expected.extend(key.into_iter());
            expected.extend(value.into_iter());
        }

        assert_eq!(expected, bytes);
    }

    #[test]
    fn proptest_box(v in any::<Box<u32>>()) {
        is_same(v);
    }

    #[test]
    fn proptest_struct(v in any::<S>()) {
        is_same(v);
    }

    #[test]
    fn proptest_addr(v in any::<Addr>()) {
        is_same(v);
    }

    #[test]
    fn proptest_bar(v in any::<Bar>()) {
        is_same(v);
    }

    #[test]
    fn proptest_foo(v in any::<Foo>()) {
        is_same(v);
    }
}

#[test]
fn invalid_utf8() {
    let invalid_utf8 = vec![1, 0, 0, 0, 0xFF];
    assert_eq!(from_bytes::<String>(&invalid_utf8), Err(Error::Utf8));
}

#[test]
fn invalid_variant() {
    #[derive(Serialize, Deserialize, Debug)]
    enum Test {
        One,
        Two,
    };

    let invalid_variant = vec![5, 0, 0, 0];
    match from_bytes::<Test>(&invalid_variant).unwrap_err() {
        // Error message comes from serde
        Error::Custom(_) => {}
        _ => panic!(),
    }
}

#[test]
fn invalid_option() {
    let invalid_option = vec![5, 0];
    assert_eq!(
        from_bytes::<Option<u8>>(&invalid_option),
        Err(Error::ExpectedOption)
    );
}

#[test]
fn invalid_bool() {
    let invalid_bool = vec![9];
    assert_eq!(
        from_bytes::<bool>(&invalid_bool),
        Err(Error::ExpectedBoolean)
    );
}

#[test]
fn sequence_too_long() {
    let seq = vec![0; MAX_SEQUENCE_LENGTH + 1];
    match to_bytes(&seq).unwrap_err() {
        Error::ExceededMaxLen(len) => assert_eq!(len, MAX_SEQUENCE_LENGTH + 1),
        _ => panic!(),
    }
}

#[test]
fn sequence_not_long_enough() {
    let seq = vec![5, 0, 0, 0, 1, 2, 3, 4]; // Missing 5th element
    assert_eq!(from_bytes::<Vec<u8>>(&seq), Err(Error::Eof));
}

#[test]
fn leftover_bytes() {
    let seq = vec![5, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]; // 5 extra elements
    assert_eq!(from_bytes::<Vec<u8>>(&seq), Err(Error::RemainingInput));
}

#[test]
fn test_f32() {
    assert!(to_bytes(&1.0f32).is_err());
}

#[test]
fn test_f64() {
    assert!(to_bytes(&42.0f64).is_err());
}

#[test]
fn test_char() {
    assert!(to_bytes(&'a').is_err());
}

#[test]
fn zero_copy_parse() {
    #[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
    struct Foo<'a> {
        borrowed_str: &'a str,
        borrowed_bytes: &'a [u8],
    }

    let f = Foo {
        borrowed_str: "hi",
        borrowed_bytes: &[0, 1, 2, 3],
    };
    {
        let expected = vec![2, 0, 0, 0, b'h', b'i', 4, 0, 0, 0, 0, 1, 2, 3];
        let encoded = to_bytes(&f).unwrap();
        assert_eq!(expected, encoded);
        let out: Foo = from_bytes(&encoded[..]).unwrap();
        assert_eq!(out, f);
    }
}

#[test]
fn cow() {
    use std::borrow::Cow;

    let large_object = vec![1u32, 2, 3, 4, 5, 6];
    let mut large_map = BTreeMap::new();
    large_map.insert(1, 2);

    #[derive(Serialize, Deserialize, Debug)]
    enum Message<'a> {
        M1(Cow<'a, Vec<u32>>),
        M2(Cow<'a, BTreeMap<u32, u32>>),
    }

    // M1
    {
        let serialized = to_bytes(&Message::M1(Cow::Borrowed(&large_object))).unwrap();
        let deserialized: Message<'static> = from_bytes(&serialized).unwrap();

        match deserialized {
            Message::M1(b) => assert_eq!(b.into_owned(), large_object),
            _ => panic!(),
        }
    }

    // M2
    {
        let serialized = to_bytes(&Message::M2(Cow::Borrowed(&large_map))).unwrap();
        let deserialized: Message<'static> = from_bytes(&serialized).unwrap();

        match deserialized {
            Message::M2(b) => assert_eq!(b.into_owned(), large_map),
            _ => panic!(),
        }
    }
}

#[test]
fn strbox() {
    use std::borrow::Cow;

    let strx: &'static str = "hello world";
    let serialized = to_bytes(&Cow::Borrowed(strx)).unwrap();
    let deserialized: Cow<'static, String> = from_bytes(&serialized).unwrap();
    let stringx: String = deserialized.into_owned();
    assert_eq!(strx, stringx);
}

#[test]
fn slicebox() {
    use std::borrow::Cow;

    let slice = [1u32, 2, 3, 4, 5];
    let serialized = to_bytes(&Cow::Borrowed(&slice[..])).unwrap();
    let deserialized: Cow<'static, Vec<u32>> = from_bytes(&serialized).unwrap();
    {
        let sb: &[u32] = &deserialized;
        assert_eq!(slice, sb);
    }
    let vecx: Vec<u32> = deserialized.into_owned();
    assert_eq!(slice, vecx[..]);
}

#[test]
fn path_buf() {
    use std::path::{Path, PathBuf};

    let path = Path::new("foo").to_path_buf();
    let encoded = to_bytes(&path).unwrap();
    let decoded: PathBuf = from_bytes(&encoded).unwrap();
    assert!(path.to_str() == decoded.to_str());
}

#[derive(Arbitrary, Debug, Deserialize, Serialize, PartialEq)]
struct Addr([u8; 32]);

#[derive(Arbitrary, Debug, Deserialize, Serialize, PartialEq)]
struct Bar {
    a: u64,
    b: Vec<u8>,
    c: Addr,
    d: u32,
}

#[derive(Arbitrary, Debug, Deserialize, Serialize, PartialEq)]
struct Foo {
    a: u64,
    b: Vec<u8>,
    c: Bar,
    d: bool,
    e: BTreeMap<Vec<u8>, Vec<u8>>,
}

#[test]
fn serde_known_vector() {
    let b = Bar {
        a: 100,
        b: vec![0, 1, 2, 3, 4, 5, 6, 7, 8],
        c: Addr([5u8; 32]),
        d: 99,
    };

    let mut map = BTreeMap::new();
    map.insert(vec![0, 56, 21], vec![22, 10, 5]);
    map.insert(vec![1], vec![22, 21, 67]);
    map.insert(vec![20, 21, 89, 105], vec![201, 23, 90]);

    let f = Foo {
        a: u64::max_value(),
        b: vec![100, 99, 88, 77, 66, 55],
        c: b,
        d: true,
        e: map,
    };

    let bytes = to_bytes(&f).unwrap();

    let test_vector = vec![
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x06, 0x00, 0x00, 0x00, 0x64, 0x63, 0x58,
        0x4d, 0x42, 0x37, 0x64, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x09, 0x00, 0x00, 0x00,
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05,
        0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05,
        0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x63, 0x00, 0x00, 0x00,
        0x01, 0x03, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x03, 0x00, 0x00, 0x00, 0x16,
        0x15, 0x43, 0x03, 0x00, 0x00, 0x00, 0x00, 0x38, 0x15, 0x03, 0x00, 0x00, 0x00, 0x16, 0x0a,
        0x05, 0x04, 0x00, 0x00, 0x00, 0x14, 0x15, 0x59, 0x69, 0x03, 0x00, 0x00, 0x00, 0xc9, 0x17,
        0x5a,
    ];

    // make sure we serialize into exact same bytes as before
    assert_eq!(test_vector, bytes);

    // make sure we can deserialize the test vector into expected struct
    let deserialized_foo: Foo = from_bytes(&test_vector).unwrap();
    assert_eq!(f, deserialized_foo);
}
