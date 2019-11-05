// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::*;
use move_ir::{assert_error_type, assert_no_error};
use proptest::prelude::*;
use libra_types::{
    account_address::AccountAddress,
    transaction::{TransactionArgument, TransactionPayload},
};

#[test]
fn write_set_txn_roundtrip() {
    // Creating a new test environment is expensive so do it outside the proptest environment.
    let test_env = TestEnvironment::default();

    proptest!(|(signed_txn in SignedTransaction::genesis_strategy())| {
        let write_set = match signed_txn.payload() {
            TransactionPayload::WriteSet(write_set) => write_set.clone(),
            TransactionPayload::Program | TransactionPayload::Script(_) | TransactionPayload::Module(_) => unreachable!(
                "write set strategy should only generate write set transactions",
            ),
        };
        let output = test_env.eval_txn(signed_txn)
            .expect("write set transactions should succeed");
        prop_assert_eq!(output.write_set(), &write_set);
    });
}
