// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! # Libra Canonical Serialization (LCS)
//!
//! LCS defines a deterministic means for translating a message or data structure into bytes
//! irrespective of platform, architecture, or programming language.
//!
//! ## Background
//!
//! In Libra, participants pass around messages or data structures that often times need to be
//! signed by a prover and verified by one or more verifiers.  Serialization in this context refers
//! to the process of converting a message into a byte array. Many serialization approaches support
//! loose standards such that two implementations can produce two different byte streams that would
//! represent the same, identical message.  While for many applications, non-deterministic
//! serialization causes no issues, it does so for applications using serialization for
//! cryptographic purposes. For example, given a signature and a message, a verifier may not unable
//! to produce the same serialized byte array constructed by the prover when the prover signed the
//! message resulting in a non-verifiable message. In other words, to ensure message verifiability
//! when using non-deterministic serialization, participants must either retain the original
//! serialized bytes or risk losing the ability to verify messages. This creates a burden requiring
//! participants to maintain both a copy of the serialized bytes and the deserialized message often
//! leading to confusion about safety and correctness. While there exist a handful of existing
//! deterministic serialization formats, there is no obvious choice.  To address this, we propose
//! Libra Canonical Serialization that defines a deterministic means for translating a message into
//! bytes and back again.
//!
//! ## Specification
//!
//! LCS supports the following data types:
//!
//! * Booleans
//! * Signed 8-bit, 16-bit, 32-bit, 64-bit, and 128-bit integers
//! * Unsigned 8-bit, 16-bit, 32-bit, 64-bit, and 128-bit integers
//! * Option
//! * Fixed and variable length sequences
//! * UTF-8 Encoded Strings
//! * Tuples
//! * Structures
//! * Externally tagged enumerations
//! * Maps
//!
//! ## General structure
//!
//! LCS is not a self-describing format and as such, in order to deserialize a message you must
//! know the message type and layout ahead of time.
//!
//! Unless specified, all numbers are stored in little endian, two's complement format.
//!
//! ### Booleans and Integers
//!
//! |Type                       |Original data          |Hex representation |Serialized format  |
//! |---                        |---                    |---                |---                |
//! |Boolean                    |True / False           |0x01 / 0x00        |[01] / [00]        |
//! |8-bit signed integer       |-1                     |0xFF               |[FF]               |
//! |8-bit unsigned integer     |1                      |0x01               |[01]               |
//! |16-bit signed integer      |-4660                  |0xEDCC             |[CCED]             |
//! |16-bit unsigned integer    |4660                   |0x1234             |[3412]             |
//! |32-bit signed integer      |-305419896             |0xEDCBA988         |[88A9CBED]         |
//! |32-bit unsigned integer    |305419896              |0x12345678         |[78563412]         |
//! |64-bit signed integer      |-1311768467750121216   |0xEDCBA98754321100 |[0011325487A9CBED] |
//! |64-bit unsigned integer    |1311768467750121216    |0x12345678ABCDEF00 |[00EFCDAB78563412] |
//!
//! ### Optional Data
//!
//! Optional or nullable data either exists in its full representation or does not. LCS represents
//! this as a single byte representing the presence `0x01` or absence `0x00` of data. If the data
//! is present then the serialized form of that data follows. For example:
//!
//! ```rust
//! # use libra_canonical_serialization::{Result, to_bytes};
//! # fn main() -> Result<()> {
//! let some_data: Option<u8> = Some(8);
//! assert_eq!(to_bytes(&some_data)?, vec![1, 8]);
//!
//! let no_data: Option<u8> = None;
//! assert_eq!(to_bytes(&no_data)?, vec![0]);
//! # Ok(())}
//! ```
//!
//! ### Fixed and Variable Length Sequences
//!
//! Sequences can be made of up of any LCS supported types (even complex structures) but all
//! elements in the sequence must be of the same type. If the length of a sequence is fixed and
//! well known then LCS represents this as just the concatenation of the serialized form of each
//! individual element in the sequence. If the length of the sequence can be variable, then the
//! serialized sequence is length prefixed with a 32-bit unsigned integer indicating the number of
//! elements in the sequence. All variable length sequences must be `MAX_SEQUENCE_LENGTH` elements
//! long or less.
//!
//! ```rust
//! # use libra_canonical_serialization::{Result, to_bytes};
//! # fn main() -> Result<()> {
//! let fixed: [u16; 3] = [1, 2, 3];
//! assert_eq!(to_bytes(&fixed)?, vec![1, 0, 2, 0, 3, 0]);
//!
//! let variable: Vec<u16> = vec![1, 2];
//! assert_eq!(to_bytes(&variable)?, vec![2, 0, 0, 0, 1, 0, 2, 0]);
//! # Ok(())}
//! ```
//!
//! ### Strings
//!
//! Only valid UTF-8 Strings are supported. LCS serializes such strings as a variable length byte
//! sequence, i.e. length prefixed with a 32-bit unsigned integer followed by the byte
//! representation of the string.
//!
//! ```rust
//! # use libra_canonical_serialization::{Result, to_bytes};
//! # fn main() -> Result<()> {
//! // Note that this string has 10 characters but has a byte length of 24
//! let utf8_str = "çå∞≠¢õß∂ƒ∫";
//! let expecting = vec![
//!     24, 0, 0, 0, 0xc3, 0xa7, 0xc3, 0xa5, 0xe2, 0x88, 0x9e, 0xe2, 0x89, 0xa0, 0xc2,
//!     0xa2, 0xc3, 0xb5, 0xc3, 0x9f, 0xe2, 0x88, 0x82, 0xc6, 0x92, 0xe2, 0x88, 0xab,
//! ];
//! assert_eq!(to_bytes(&utf8_str)?, expecting);
//! # Ok(())}
//! ```
//!
//! ### Tuples
//!
//! Tuples are typed composition of objects: `(Type0, Type1)`
//!
//! Tuples are considered a fixed length sequence where each element in the sequence can be a
//! different type supported by LCS. Each element of a tuple is serialized in the order it is
//! defined within the tuple, i.e. [tuple.0, tuple.2].
//!
//! ```rust
//! # use libra_canonical_serialization::{Result, to_bytes};
//! # fn main() -> Result<()> {
//! let tuple = (-1i8, "libra");
//! let expecting = vec![0xFF, 5, 0, 0, 0, b'l', b'i', b'b', b'r', b'a'];
//! assert_eq!(to_bytes(&tuple)?, expecting);
//! # Ok(())}
//! ```
//!
//!
//! ### Structures
//!
//! Structures are fixed length sequences consisting of fields with potentially different types.
//! Each field within a struct is serialized in the order specified by the canonical structure
//! definition. Structs can exist within other structs and as such, LCS recurses into each struct
//! ans serializes them in order. There are no labels in the serialized format, the struct ordering
//! defines the organization within the serialization stream.
//!
//! ```rust
//! # use libra_canonical_serialization::{Result, to_bytes};
//! # use serde::Serialize;
//! # fn main() -> Result<()> {
//! # #[derive(Serialize)]
//! struct MyStruct {
//!     boolean: bool,
//!     bytes: Vec<u8>,
//!     label: String,
//! }
//!
//! # #[derive(Serialize)]
//! struct Wrapper {
//!     inner: MyStruct,
//!     name: String,
//! }
//!
//! let s = MyStruct {
//!     boolean: true,
//!     bytes: vec![0xC0, 0xDE],
//!     label: "a".to_owned(),
//! };
//! let s_bytes = to_bytes(&s)?;
//! let mut expecting = vec![1, 2, 0, 0, 0, 0xC0, 0xDE, 1, 0, 0, 0, b'a'];
//! assert_eq!(s_bytes, expecting);
//!
//! let w = Wrapper {
//!     inner: s,
//!     name: "b".to_owned(),
//! };
//! let w_bytes = to_bytes(&w)?;
//! assert!(w_bytes.starts_with(&s_bytes));
//!
//! expecting.append(&mut vec![1, 0, 0, 0, b'b']);
//! assert_eq!(w_bytes, expecting);
//! # Ok(())}
//! ```
//!
//! ### Externally Tagged Enumerations
//!
//! An enumeration is typically represented as a type that can take one of potentially many
//! different variants. In LCS, each variant is mapped to a variant index, an unsigned 32-bit
//! integer, followed by optionally serialized data if the type has an associated value. An
//! associated type can be any LCS supported type. The variant index is determined based on the
//! ordering of the variants in the canonical enum definition, where the first variant has an index
//! of `0`, the second an index of `1`, etc.
//!
//! ```rust
//! # use libra_canonical_serialization::{Result, to_bytes};
//! # use serde::Serialize;
//! # fn main() -> Result<()> {
//! # #[derive(Serialize)]
//! enum E {
//!     Variant0(u16),
//!     Variant1(u8),
//!     Variant2(String),
//! }
//!
//! let v0 = E::Variant0(8000);
//! let v1 = E::Variant1(255);
//! let v2 = E::Variant2("e".to_owned());
//!
//! assert_eq!(to_bytes(&v0)?, vec![0, 0, 0, 0, 0x40, 0x1F]);
//! assert_eq!(to_bytes(&v1)?, vec![1, 0, 0, 0, 0xFF]);
//! assert_eq!(to_bytes(&v2)?, vec![2, 0, 0, 0, 1, 0, 0, 0, b'e']);
//! # Ok(())}
//! ```
//!
//! If you need to serialize a C-style enum, you should use a primitive integer type.
//!
//! ### Maps (Key / Value Stores)
//!
//! Maps can be considered a variable length array of length two tuples where Key points to Value is
//! represented as (Key, Value). Hence they should be serialized first with the length of number of
//! entries followed by each entry in lexicographical order as defined by the byte representation of
//! the LCS serialized key.
//!
//! ```rust
//! # use libra_canonical_serialization::{Result, to_bytes};
//! # use std::collections::HashMap;
//! # fn main() -> Result<()> {
//! let mut map = HashMap::new();
//! map.insert(b'e', b'f');
//! map.insert(b'a', b'b');
//! map.insert(b'c', b'd');
//!
//! let expecting = vec![(b'a', b'b'), (b'c', b'd'), (b'e', b'f')];
//!
//! assert_eq!(to_bytes(&map)?, to_bytes(&expecting)?);
//! # Ok(())}
//! ```
//!
//! ## Backwards compatibility
//!
//! Complex types dependent upon the specification in which they are used. LCS does not provide
//! direct provisions for versioning or backwards / forwards compatibility. A change in an objects
//! structure could prevent historical clients from understanding new clients and vice-versa.

mod de;
mod error;
mod ser;

/// Variable length sequences in LCS are limited to max length of 2^31
pub const MAX_SEQUENCE_LENGTH: usize = 1 << 31;

pub use de::{from_bytes, from_bytes_seed};
pub use error::{Error, Result};
pub use ser::to_bytes;
