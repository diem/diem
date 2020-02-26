// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_address::AccountAddress,
    test_helpers::assert_canonical_encode_decode,
    transaction::{
        RawTransaction, Script, SignedTransaction, Transaction, TransactionInfo,
        TransactionListWithProof, TransactionPayload, TransactionToCommit, TransactionWithProof,
    },
};
use libra_crypto::ed25519::*;
use libra_prost_ext::test_helpers::assert_protobuf_encode_decode;
use proptest::prelude::*;
use std::convert::TryFrom;

#[test]
fn test_invalid_signature() {
    let keypair = compat::generate_keypair(None);
    let proto_txn: crate::proto::types::SignedTransaction = SignedTransaction::new(
        RawTransaction::new_script(
            AccountAddress::random(),
            0,
            Script::new(vec![], vec![]),
            0,
            0,
            std::time::Duration::new(0, 0),
        ),
        keypair.1,
        Ed25519Signature::try_from(&[1u8; 64][..]).unwrap(),
    )
    .into();
    let txn = SignedTransaction::try_from(proto_txn)
        .expect("initial conversion from_proto should succeed");
    txn.check_signature()
        .expect_err("signature checking should fail");
}

proptest! {
    #[test]
    fn test_sign_raw_transaction(raw_txn in any::<RawTransaction>(), (sk1, pk1) in compat::keypair_strategy()) {
        let txn = raw_txn.sign(&sk1, pk1).unwrap();
        let signed_txn = txn.into_inner();
        assert!(signed_txn.check_signature().is_ok());
    }

    #[test]
    fn transaction_payload_lcs_roundtrip(txn_payload in any::<TransactionPayload>()) {
        assert_canonical_encode_decode(txn_payload);
    }

    #[test]
    fn raw_transaction_lcs_roundtrip(raw_txn in any::<RawTransaction>()) {
        assert_canonical_encode_decode(raw_txn);
    }

    #[test]
    fn signed_transaction_lcs_roundtrip(signed_txn in any::<SignedTransaction>()) {
        assert_canonical_encode_decode(signed_txn);
    }

    #[test]
    fn signed_transaction_proto_roundtrip(signed_txn in any::<SignedTransaction>()) {
        assert_protobuf_encode_decode::<crate::proto::types::SignedTransaction, SignedTransaction>(&signed_txn);
    }

    #[test]
    fn transaction_info_lcs_roundtrip(txn_info in any::<TransactionInfo>()) {
        assert_canonical_encode_decode(txn_info);
    }

    #[test]
    fn transaction_info_proto_roundtrip(txn_info in any::<TransactionInfo>()) {
        assert_protobuf_encode_decode::<crate::proto::types::TransactionInfo, TransactionInfo>(&txn_info);
    }

    #[test]
    fn transaction_to_commit_proto_roundtrip(txn_to_commit in any::<TransactionToCommit>()) {
        assert_protobuf_encode_decode::<crate::proto::types::TransactionToCommit, TransactionToCommit>(&txn_to_commit);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn transaction_list_with_proof_lcs_roundtrip(txn_list in any::<TransactionListWithProof>()) {
        assert_canonical_encode_decode(txn_list);
    }

    #[test]
    fn transaction_list_with_proof_proto_roundtrip(txn_list in any::<TransactionListWithProof>()) {
        assert_protobuf_encode_decode::<crate::proto::types::TransactionListWithProof, TransactionListWithProof>(&txn_list);
    }

    #[test]
    fn transaction_lcs_roundtrip(txn in any::<Transaction>()) {
        assert_canonical_encode_decode(txn);
    }

    #[test]
    fn transaction_proto_roundtrip(txn in any::<Transaction>()) {
        assert_protobuf_encode_decode::<crate::proto::types::Transaction, Transaction>(&txn);
    }

    #[test]
    fn transaction_with_proof_lcs_roundtrip(txn_with_proof in any::<TransactionWithProof>()) {
        assert_canonical_encode_decode(txn_with_proof);
    }

    #[test]
    fn transaction_with_proof_proto_roundtrip(txn_with_proof in any::<TransactionWithProof>()) {
        assert_protobuf_encode_decode::<crate::proto::types::TransactionWithProof, TransactionWithProof>(&txn_with_proof);
    }
}
