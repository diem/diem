// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::values::{prop::layout_and_value_strategy, Value};
use move_core_types::value::{MoveTypeLayout, MoveValue};
use proptest::prelude::*;
use std::convert::TryInto;

proptest! {
    #[test]
    fn serializer_round_trip((layout, value) in layout_and_value_strategy()) {
        let blob = value.simple_serialize(&layout).expect("must serialize");
        let value_deserialized = Value::simple_deserialize(&blob, &layout).expect("must deserialize");
        assert!(value.equals(&value_deserialized).unwrap());

        let ty = (&layout).try_into().unwrap();
        let move_value = value.as_move_value(&layout);

        let blob2 = move_value.simple_serialize().expect("must serialize");
        assert_eq!(blob, blob2);

        let move_value_deserialized = MoveValue::simple_deserialize(&blob2, &ty).expect("must deserialize.");
        assert_eq!(move_value, move_value_deserialized);
    }
}
