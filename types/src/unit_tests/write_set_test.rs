// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::write_set::WriteSet;
use canonical_serialization::{
    CanonicalDeserializer, CanonicalSerializer, SimpleDeserializer, SimpleSerializer,
};
use proptest::prelude::*;

proptest! {
    #[test]
    fn write_set_roundtrip_canonical_serialization(write_set in any::<WriteSet>()) {
        let mut serializer = SimpleSerializer::<Vec<u8>>::new();
        serializer.encode_struct(&write_set).unwrap();
        let serialized_bytes = serializer.get_output();

        let mut deserializer = SimpleDeserializer::new(&serialized_bytes);
        let output: WriteSet = deserializer.decode_struct().unwrap();
        assert_eq!(write_set, output);
    }
}
