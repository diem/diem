// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::epoch_change::EpochChangeProof;
use libra_prost_ext::test_helpers::assert_protobuf_encode_decode;
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn test_epoch_change_proof_conversion(change in any::<EpochChangeProof>()) {
        assert_protobuf_encode_decode::<crate::proto::types::EpochChangeProof, EpochChangeProof>(&change);
    }
}
