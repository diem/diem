// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_address::AccountAddress,
    transaction::{Program, RawTransaction, SignedTransaction},
};
use nextgen_crypto::ed25519::*;
use proptest::prelude::*;
use proto_conv::{FromProto, IntoProto};
use std::convert::TryFrom;

#[test]
fn test_invalid_signature() {
    let keypair = compat::generate_keypair(None);
    let txn = SignedTransaction::from_proto(
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
            Ed25519Signature::try_from(&[1u8; 64][..]).unwrap(),
        )
        .into_proto(),
    )
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
}
