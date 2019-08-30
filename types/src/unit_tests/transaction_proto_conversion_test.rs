// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::transaction::*;
use proptest::prelude::*;
use proto_conv::test_helper::assert_protobuf_encode_decode;

proptest! {
    #[test]
    fn test_signed_txn(signed_txn in any::<SignedTransaction>()) {
        assert_protobuf_encode_decode(&signed_txn);
    }

    #[test]
    fn test_signed_txn_with_proof(signed_txn_with_proof in any::<SignedTransactionWithProof>()) {
        assert_protobuf_encode_decode(&signed_txn_with_proof);
    }

    #[test]
    fn test_transaction_info(txn_info in any::<TransactionInfo>()) {
        assert_protobuf_encode_decode(&txn_info);
    }

    #[test]
    fn test_transaction_to_commit(txn_to_commit in any::<TransactionToCommit>()) {
        assert_protobuf_encode_decode(&txn_to_commit);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn test_transaction_list_with_proof(txn_list in any::<TransactionListWithProof>()) {
        assert_protobuf_encode_decode(&txn_list);
    }
}
