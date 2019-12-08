// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::test_helpers::assert_canonical_encode_decode;
use crate::{
    account_address::AccountAddress,
    transaction::{RawTransaction, Script, SignedTransaction, Transaction, TransactionPayload},
};
use libra_crypto::ed25519::*;
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
    fn test_sig(raw_txn in any::<RawTransaction>(), (sk1, pk1) in compat::keypair_strategy()) {
        let txn = raw_txn.sign(&sk1, pk1).unwrap();
        let signed_txn = txn.into_inner();
        assert!(signed_txn.check_signature().is_ok());
    }

    #[test]
    fn transaction_payload_round_trip_canonical_serialization(txn_payload in any::<TransactionPayload>()) {
        assert_canonical_encode_decode(txn_payload);
    }

    #[test]
    fn raw_transaction_round_trip_canonical_serialization(raw_txn in any::<RawTransaction>()) {
        assert_canonical_encode_decode(raw_txn);
    }

    #[test]
    fn transaction_round_trip_canonical_serialization(txn in any::<Transaction>()) {
        assert_canonical_encode_decode(txn);
    }
}
