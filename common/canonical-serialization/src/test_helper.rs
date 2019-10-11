// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{CanonicalDeserialize, CanonicalSerialize, SimpleDeserializer, SimpleSerializer};
use std::fmt::Debug;

pub fn assert_canonical_encode_decode<T>(object: &T)
where
    T: CanonicalSerialize + CanonicalDeserialize + Debug + Eq,
{
    let serialized: Vec<u8> =
        SimpleSerializer::serialize(object).expect("Serialization should work");
    let mut deserializer = SimpleDeserializer::new(&serialized);
    let deserialized = T::deserialize(&mut deserializer).expect("Deserialization should work");
    assert_eq!(*object, deserialized);
    assert_eq!(deserializer.position(), deserializer.len() as u64);
    assert!(deserializer.is_empty());
}
