// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::{Account, AccountData},
    executor::FakeExecutor,
};
use libra_types::{
    access_path::{AccessPath, Accesses},
    account_config::{lbr_type_tag, CORE_CODE_ADDRESS, LBR_NAME},
    contract_event::ContractEvent,
    language_storage::{ResourceKey, StructTag},
    on_chain_config::new_epoch_event_key,
    transaction::{ChangeSet, TransactionPayload, TransactionStatus},
    vm_error::{StatusCode, VMStatus},
    write_set::{WriteOp, WriteSetMut},
};
use move_core_types::identifier::Identifier;

#[test]
fn verify_and_execute_writeset() {
    // create a FakeExecutor with a genesis from file
    let mut executor = FakeExecutor::from_genesis_file();
    let genesis_account = Account::new_association();
    executor.new_block();

    // (1) Create a WriteSet that adds an account on a new address
    let new_account_data = AccountData::new(1000, 10);
    let write_set = new_account_data.to_writeset();

    let writeset_txn = genesis_account.create_signed_txn_impl(
        *genesis_account.address(),
        TransactionPayload::WriteSet(ChangeSet::new(write_set.clone(), vec![])),
        0,
        100_000,
        1,
        LBR_NAME.to_string(),
    );

    let output = executor.execute_transaction(writeset_txn.clone());
    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED))
    );
    assert!(executor
        .verify_transaction(writeset_txn.clone())
        .status()
        .is_none());

    executor.apply_write_set(output.write_set());

    let (updated_sender, updated_sender_balance) = executor
        .read_account_info(new_account_data.account())
        .expect("sender balance must exist");

    assert_eq!(1000, updated_sender_balance.coin());
    assert_eq!(10, updated_sender.sequence_number());

    // (2) Cannot reapply the same writeset.
    let output = executor.execute_transaction(writeset_txn.clone());
    assert_eq!(
        output.status(),
        &TransactionStatus::Discard(VMStatus::new(StatusCode::SEQUENCE_NUMBER_TOO_OLD))
    );
    assert_eq!(
        executor.verify_transaction(writeset_txn).status().unwrap(),
        VMStatus::new(StatusCode::SEQUENCE_NUMBER_TOO_OLD)
    );

    // (3) Cannot apply the writeset with future sequence number.
    let writeset_txn = genesis_account.create_signed_txn_impl(
        *genesis_account.address(),
        TransactionPayload::WriteSet(ChangeSet::new(write_set, vec![])),
        10,
        100_000,
        1,
        LBR_NAME.to_string(),
    );
    let output = executor.execute_transaction(writeset_txn.clone());
    let expected_err = VMStatus::new(StatusCode::ABORTED).with_sub_status(11);
    assert_eq!(
        output.status(),
        &TransactionStatus::Discard(expected_err.clone())
    );
    assert_eq!(
        executor.verify_transaction(writeset_txn).status().unwrap(),
        expected_err
    );
}

#[test]
fn bad_writesets() {
    // create a FakeExecutor with a genesis from file
    let mut executor = FakeExecutor::from_genesis_file();
    let genesis_account = Account::new_association();
    executor.new_block();

    // Create a WriteSet that adds an account on a new address
    let new_account_data = AccountData::new(1000, 10);
    let write_set = new_account_data.to_writeset();

    // (1) This WriteSet is signed by an arbitrary account rather than association account. Should be
    // rejected.
    let writeset_txn = new_account_data.account().create_signed_txn_impl(
        *genesis_account.address(),
        TransactionPayload::WriteSet(ChangeSet::new(write_set.clone(), vec![])),
        0,
        100_000,
        1,
        LBR_NAME.to_string(),
    );

    let output = executor.execute_transaction(writeset_txn);
    assert_eq!(
        output.status(),
        &TransactionStatus::Discard(VMStatus::new(StatusCode::INVALID_AUTH_KEY))
    );

    // (2) The WriteSet contains a reconfiguration event, will be dropped.
    let event = ContractEvent::new(new_epoch_event_key(), 0, lbr_type_tag(), vec![]);
    let writeset_txn = genesis_account.create_signed_txn_impl(
        *genesis_account.address(),
        TransactionPayload::WriteSet(ChangeSet::new(write_set, vec![event])),
        0,
        100_000,
        1,
        LBR_NAME.to_string(),
    );

    let output = executor.execute_transaction(writeset_txn);
    assert_eq!(
        output.status(),
        &TransactionStatus::Discard(VMStatus::new(StatusCode::INVALID_WRITE_SET))
    );

    // (3) The WriteSet attempts to change LibraWriteSetManager, will be dropped.
    let key = ResourceKey::new(
        *genesis_account.address(),
        StructTag {
            address: CORE_CODE_ADDRESS,
            module: Identifier::new("LibraWriteSetManager").unwrap(),
            name: Identifier::new("T").unwrap(),
            type_params: vec![],
        },
    );
    let path = AccessPath::resource_access_path(&key, &Accesses::empty());

    let write_set = WriteSetMut::new(vec![(path, WriteOp::Value(vec![]))])
        .freeze()
        .unwrap();
    let writeset_txn = genesis_account.create_signed_txn_impl(
        *genesis_account.address(),
        TransactionPayload::WriteSet(ChangeSet::new(write_set, vec![])),
        0,
        100_000,
        0,
        LBR_NAME.to_string(),
    );

    let output = executor.execute_transaction(writeset_txn);
    assert_eq!(
        output.status(),
        &TransactionStatus::Discard(VMStatus::new(StatusCode::INVALID_WRITE_SET))
    );
}
