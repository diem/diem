// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::values::{prop::layout_and_value_strategy, Value};
use proptest::prelude::*;

proptest! {
    #[test]
    fn serializer_round_trip((layout, value) in layout_and_value_strategy()) {
        let blob = value.simple_serialize(&layout).expect("must serialize");
        let value_deserialized = Value::simple_deserialize(&blob, &layout).expect("must deserialize");
        assert!(value.equals(&value_deserialized).unwrap());
    }
}
