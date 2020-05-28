// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_prost_test_helpers::assert_protobuf_encode_decode;
use libra_types::on_chain_config::ValidatorSet;
use proptest::prelude::*;

proptest! {
#![proptest_config(ProptestConfig::with_cases(20))]

#[test]
fn test_validator_set_protobuf_conversion(set in any::<ValidatorSet>()) {
    assert_protobuf_encode_decode::<crate::proto::types::ValidatorSet, ValidatorSet>(&set);
}}
