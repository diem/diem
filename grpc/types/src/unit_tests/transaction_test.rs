// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519Signature},
    PrivateKey, Uniform,
};
use libra_prost_ext::test_helpers::assert_protobuf_encode_decode;
use libra_types::{
    account_address::AccountAddress,
    account_config::LBR_NAME,
    transaction::{
        RawTransaction, Script, SignedTransaction, Transaction, TransactionInfo,
        TransactionListWithProof, TransactionToCommit, TransactionWithProof,
    },
};
use proptest::prelude::*;
use std::convert::TryFrom;

#[test]
fn test_invalid_signature() {
    let proto_txn: crate::proto::types::SignedTransaction = SignedTransaction::new(
        RawTransaction::new_script(
            AccountAddress::random(),
            0,
            Script::new(vec![], vec![], vec![]),
            0,
            0,
            LBR_NAME.to_owned(),
            std::time::Duration::new(0, 0),
        ),
        Ed25519PrivateKey::generate_for_testing().public_key(),
        Ed25519Signature::try_from(&[1u8; 64][..]).unwrap(),
    )
    .into();
    let _txn = SignedTransaction::try_from(proto_txn)
        .expect("initial conversion from_proto should succeed");
}

proptest! {


    #[test]
    fn signed_transaction_proto_roundtrip(signed_txn in any::<SignedTransaction>()) {
        assert_protobuf_encode_decode::<crate::proto::types::SignedTransaction, SignedTransaction>(&signed_txn);
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
    fn transaction_list_with_proof_proto_roundtrip(txn_list in any::<TransactionListWithProof>()) {
        assert_protobuf_encode_decode::<crate::proto::types::TransactionListWithProof, TransactionListWithProof>(&txn_list);
    }


    #[test]
    fn transaction_proto_roundtrip(txn in any::<Transaction>()) {
        assert_protobuf_encode_decode::<crate::proto::types::Transaction, Transaction>(&txn);
    }


    #[test]
    fn transaction_with_proof_proto_roundtrip(txn_with_proof in any::<TransactionWithProof>()) {
        assert_protobuf_encode_decode::<crate::proto::types::TransactionWithProof, TransactionWithProof>(&txn_with_proof);
    }
}
