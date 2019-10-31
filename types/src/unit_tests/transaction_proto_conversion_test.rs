// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::transaction::*;
use libra_prost_ext::test_helpers::assert_protobuf_encode_decode;
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_signed_txn(signed_txn in any::<SignedTransaction>()) {
        assert_protobuf_encode_decode::<crate::proto::types::SignedTransaction, SignedTransaction>(&signed_txn);
    }

    #[test]
    fn test_txn(txn in any::<Transaction>()) {
        assert_protobuf_encode_decode::<crate::proto::types::Transaction, Transaction>(&txn);
    }

    #[test]
    fn test_txn_with_proof(txn_with_proof in any::<TransactionWithProof>()) {
        assert_protobuf_encode_decode::<crate::proto::types::TransactionWithProof, TransactionWithProof>(&txn_with_proof);
    }

    #[test]
    fn test_transaction_info(txn_info in any::<TransactionInfo>()) {
        assert_protobuf_encode_decode::<crate::proto::types::TransactionInfo, TransactionInfo>(&txn_info);
    }

    #[test]
    fn test_transaction_to_commit(txn_to_commit in any::<TransactionToCommit>()) {
        assert_protobuf_encode_decode::<crate::proto::types::TransactionToCommit, TransactionToCommit>(&txn_to_commit);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn test_transaction_list_with_proof(txn_list in any::<TransactionListWithProof>()) {
        assert_protobuf_encode_decode::<crate::proto::types::TransactionListWithProof, TransactionListWithProof>(&txn_list);
    }
}
