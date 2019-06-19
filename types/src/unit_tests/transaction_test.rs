// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_address::AccountAddress,
    transaction::{Program, RawTransaction, SignedTransaction},
};
use crypto::{
    signing::{generate_keypair, Signature},
    utils::keypair_strategy,
};
use proptest::prelude::*;
use proto_conv::{FromProto, IntoProto};

#[test]
fn test_signed_transaction_from_proto_invalid_signature() {
    let keypair = generate_keypair();
    assert!(SignedTransaction::from_proto(
        SignedTransaction::craft_signed_transaction_for_client(
            RawTransaction::new(
                AccountAddress::random(),
                0,
                Program::new(vec![], vec![], vec![]),
                0,
                0,
                std::time::Duration::new(0, 0),
            ),
            keypair.1,
            Signature::from_compact(&[0; 64]).unwrap(),
        )
        .into_proto(),
    )
    .is_err());
}

proptest! {
    #[test]
    fn test_sig(raw_txn in any::<RawTransaction>(), (sk1, pk1) in keypair_strategy()) {
        let signed_txn = raw_txn.sign(&sk1, pk1).unwrap();
        assert!(signed_txn.verify_signature().is_ok());
    }
}
