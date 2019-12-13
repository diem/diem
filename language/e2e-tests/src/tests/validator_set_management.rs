// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::{Account, AccountData},
    common_transactions::{add_validator_txn, register_validator_txn},
    executor::FakeExecutor,
};
use libra_types::{
    transaction::TransactionStatus,
    vm_error::{StatusCode, VMStatus},
};

#[test]
fn validator_add() {
    let mut executor = FakeExecutor::from_genesis_file();
    let genesis_account = Account::new_association();
    let new_validator = AccountData::new(1_000_000, 0);

    // create a FakeExecutor with a genesis from file
    executor.add_account_data(&new_validator);

    let txn = register_validator_txn(
        new_validator.account(),
        vec![],
        vec![],
        vec![],
        vec![],
        vec![],
        vec![],
        0,
    );
    executor.execute_and_apply(txn);
    let txn = add_validator_txn(&genesis_account, new_validator.account(), 1);

    let output = executor.execute_transaction(txn);
    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED))
    );
}
