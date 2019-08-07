// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::transaction::program::TransactionArgument;
use canonical_serialization::{
    CanonicalDeserializer, CanonicalSerializer, SimpleDeserializer, SimpleSerializer,
};
use proptest::prelude::*;

proptest! {
    #[test]
    fn transaction_arguments_round_trip_canonical_serialization(
        transaction_argument in any::<TransactionArgument>()
    ) {
        let mut serializer = SimpleSerializer::<Vec<u8>>::new();
        serializer.encode_struct(&transaction_argument).unwrap();
        let serialized_bytes = serializer.get_output();

        let mut deserializer = SimpleDeserializer::new(&serialized_bytes);
        let output: TransactionArgument = deserializer.decode_struct().unwrap();
        assert_eq!(transaction_argument, output);
    }
}
