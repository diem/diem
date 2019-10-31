// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::proof::{
    AccountStateProof, AccumulatorConsistencyProof, EventProof, SparseMerkleProof,
    TestAccumulatorProof, TestAccumulatorRangeProof, TransactionListProof, TransactionProof,
};
use libra_prost_ext::test_helpers::assert_protobuf_encode_decode;
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_accumulator_protobuf_conversion_roundtrip(proof in any::<TestAccumulatorProof>()) {
        assert_protobuf_encode_decode::<crate::proto::types::AccumulatorProof, TestAccumulatorProof>(&proof);
    }

    #[test]
    fn test_sparse_merkle_protobuf_conversion_roundtrip(proof in any::<SparseMerkleProof>()) {
        assert_protobuf_encode_decode::<crate::proto::types::SparseMerkleProof, SparseMerkleProof>(&proof);
    }

    #[test]
    fn test_accumulator_consistency_protobuf_conversion_roundtrip(
        proof in any::<AccumulatorConsistencyProof>(),
    ) {
        assert_protobuf_encode_decode::<crate::proto::types::AccumulatorConsistencyProof, AccumulatorConsistencyProof>(&proof);
    }

    #[test]
    fn test_accumulator_range_protobuf_conversion_roundtrip(
        proof in any::<TestAccumulatorRangeProof>(),
    ) {
        assert_protobuf_encode_decode::<crate::proto::types::AccumulatorRangeProof, TestAccumulatorRangeProof>(&proof);
    }

    #[test]
    fn test_transaction_proof_protobuf_conversion_roundtrip(proof in any::<TransactionProof>()) {
        assert_protobuf_encode_decode::<crate::proto::types::TransactionProof, TransactionProof>(&proof);
    }

    #[test]
    fn test_account_state_proof_protobuf_conversion_roundtrip(proof in any::<AccountStateProof>()) {
        assert_protobuf_encode_decode::<crate::proto::types::AccountStateProof, AccountStateProof>(&proof);
    }

    #[test]
    fn test_event_proof_protobuf_conversion_roundtrip(proof in any::<EventProof>()) {
        assert_protobuf_encode_decode::<crate::proto::types::EventProof, EventProof>(&proof);
    }

    #[test]
    fn test_transaction_list_proof_protobuf_conversion_roundtrip(proof in any::<TransactionListProof>()) {
        assert_protobuf_encode_decode::<crate::proto::types::TransactionListProof, TransactionListProof>(&proof);
    }
}
