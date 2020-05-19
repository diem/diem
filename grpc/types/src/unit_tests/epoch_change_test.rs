// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_prost_test_helpers::assert_protobuf_encode_decode;
use libra_types::epoch_change::EpochChangeProof;
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn test_epoch_change_proof_conversion(change in any::<EpochChangeProof>()) {
        assert_protobuf_encode_decode::<crate::proto::types::EpochChangeProof, EpochChangeProof>(&change);
    }
}
