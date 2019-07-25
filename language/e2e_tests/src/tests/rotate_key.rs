// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::{Account, AccountData, AccountResource},
    common_transactions::{create_account_txn, rotate_key_txn},
    executor::FakeExecutor,
};
use types::{
    account_address::AccountAddress,
    transaction::TransactionStatus,
    vm_error::{ExecutionStatus, VMStatus, VMValidationStatus},
};

#[test]
fn rotate_key() {
    let mut executor = FakeExecutor::from_genesis_file();

    // create and publish sender
    let mut sender = AccountData::new(1_000_000, 10);
    executor.add_account_data(&sender);

    let (privkey, pubkey) = crypto::signing::generate_keypair();
    let new_key_hash = AccountAddress::from(pubkey);
    let txn = rotate_key_txn(sender.account(), new_key_hash, 10);

    // execute transaction
    let output = &executor.execute_block(vec![txn])[0];
    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(VMStatus::Execution(ExecutionStatus::Executed)),
    );
    executor.apply_write_set(output.write_set());

    // Check that numbers in store are correct.
    let gas = output.gas_used();
    let balance = 1_000_000 - gas;
    let updated_sender = executor
        .read_account_resource(sender.account())
        .expect("sender must exist");
    assert_eq!(
        new_key_hash,
        AccountResource::read_auth_key(&updated_sender)
    );
    assert_eq!(balance, AccountResource::read_balance(&updated_sender));
    assert_eq!(11, AccountResource::read_sequence_number(&updated_sender));

    // Check that transactions cannot be sent with the old key any more.
    let new_account = Account::new();
    let old_key_txn = create_account_txn(sender.account(), &new_account, 11, 100_000);
    let old_key_output = &executor.execute_block(vec![old_key_txn])[0];
    assert_eq!(
        old_key_output.status(),
        &TransactionStatus::Discard(VMStatus::Validation(VMValidationStatus::InvalidAuthKey)),
    );

    // Check that transactions can be sent with the new key.
    sender.rotate_key(privkey, pubkey);
    let new_key_txn = create_account_txn(sender.account(), &new_account, 11, 100_000);
    let new_key_output = &executor.execute_block(vec![new_key_txn])[0];
    assert_eq!(
        new_key_output.status(),
        &TransactionStatus::Keep(VMStatus::Execution(ExecutionStatus::Executed)),
    );
}
