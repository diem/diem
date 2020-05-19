// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_prost_test_helpers::assert_protobuf_encode_decode;
use libra_types::validator_info::ValidatorInfo;
use proptest::prelude::*;

proptest! {
#![proptest_config(ProptestConfig::with_cases(20))]

#[test]
fn test_validator_info_protobuf_conversion(set in any::<ValidatorInfo>()) {
    assert_protobuf_encode_decode::<crate::proto::types::ValidatorInfo, ValidatorInfo>(&set);
}}
