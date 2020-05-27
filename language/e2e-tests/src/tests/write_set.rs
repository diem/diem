// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::{self, Account, AccountData},
    common_transactions::peer_to_peer_txn,
    executor::FakeExecutor,
};
use libra_types::{
    access_path::AccessPath,
    account_config::{lbr_type_tag, CORE_CODE_ADDRESS, LBR_NAME},
    contract_event::ContractEvent,
    on_chain_config::new_epoch_event_key,
    transaction::{ChangeSet, TransactionPayload, TransactionStatus},
    vm_error::{StatusCode, VMStatus},
    write_set::{WriteOp, WriteSetMut},
};
use move_core_types::{
    identifier::Identifier,
    language_storage::{ResourceKey, StructTag},
};
use transaction_builder::encode_mint_lbr_to_address_script;

#[test]
fn invalid_write_set_sender() {
    // create a FakeExecutor with a genesis from file
    let mut executor = FakeExecutor::from_genesis_file();
    executor.new_block();

    // (1) Create a WriteSet that adds an account on a new address
    let sender_account = AccountData::new(1000, 10);
    executor.add_account_data(&sender_account);

    let new_account_data = AccountData::new(1000, 10);
    let write_set = new_account_data.to_writeset();

    let writeset_txn = sender_account.account().create_signed_txn_impl(
        *sender_account.address(),
        TransactionPayload::WriteSet(ChangeSet::new(write_set, vec![])),
        0,
        100_000,
        1,
        LBR_NAME.to_owned(),
    );

    let output = executor.execute_transaction(writeset_txn);
    assert_eq!(
        output.status().vm_status().major_status,
        StatusCode::ABORTED
    );
    assert_eq!(output.status().vm_status().sub_status, Some(33));
}

#[test]
fn verify_and_execute_writeset() {
    // create a FakeExecutor with a genesis from file
    let mut executor = FakeExecutor::from_genesis_file();
    let genesis_account = Account::new_association();
    executor.new_block();

    let old_association_balance = executor
        .read_balance_resource(&genesis_account, account::lbr_currency_code())
        .expect("sender balance must exist");

    // (1) Create a WriteSet that adds an account on a new address
    let new_account_data = AccountData::new(1000, 10);
    let write_set = new_account_data.to_writeset();

    let writeset_txn = genesis_account.create_signed_txn_impl(
        *genesis_account.address(),
        TransactionPayload::WriteSet(ChangeSet::new(write_set.clone(), vec![])),
        1,
        100_000,
        1,
        LBR_NAME.to_owned(),
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

    let updated_association_account = executor
        .read_account_resource(&genesis_account)
        .expect("sender must exist");
    let updated_association_balance = executor
        .read_balance_resource(&genesis_account, account::lbr_currency_code())
        .expect("sender balance must exist");
    let updated_sender = executor
        .read_account_resource(new_account_data.account())
        .expect("sender must exist");
    let updated_sender_balance = executor
        .read_balance_resource(new_account_data.account(), account::lbr_currency_code())
        .expect("sender balance must exist");

    assert_eq!(
        old_association_balance.coin(),
        updated_association_balance.coin()
    );
    assert_eq!(2, updated_association_account.sequence_number());
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
        LBR_NAME.to_owned(),
    );
    let output = executor.execute_transaction(writeset_txn.clone());
    let status = output.status();
    assert!(status.is_discarded());
    assert_eq!(status.vm_status().major_status, StatusCode::ABORTED);
    assert_eq!(status.vm_status().sub_status, Some(11));
    let err = executor.verify_transaction(writeset_txn).status().unwrap();
    assert_eq!(err.major_status, StatusCode::ABORTED);
    assert_eq!(err.sub_status, Some(11));
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
        1,
        100_000,
        1,
        LBR_NAME.to_owned(),
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
        1,
        100_000,
        1,
        LBR_NAME.to_owned(),
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
    let path = AccessPath::resource_access_path(&key);

    let write_set = WriteSetMut::new(vec![(path, WriteOp::Value(vec![]))])
        .freeze()
        .unwrap();
    let writeset_txn = genesis_account.create_signed_txn_impl(
        *genesis_account.address(),
        TransactionPayload::WriteSet(ChangeSet::new(write_set, vec![])),
        1,
        100_000,
        0,
        LBR_NAME.to_owned(),
    );

    let output = executor.execute_transaction(writeset_txn);
    assert_eq!(
        output.status(),
        &TransactionStatus::Discard(VMStatus::new(StatusCode::INVALID_WRITE_SET))
    );

    // (4) The WriteSet attempts to change association AccountResource, will be dropped.
    let key = ResourceKey::new(
        *genesis_account.address(),
        StructTag {
            address: CORE_CODE_ADDRESS,
            module: Identifier::new("LibraAccount").unwrap(),
            name: Identifier::new("T").unwrap(),
            type_params: vec![],
        },
    );
    let path = AccessPath::resource_access_path(&key);

    let write_set = WriteSetMut::new(vec![(path, WriteOp::Value(vec![]))])
        .freeze()
        .unwrap();
    let writeset_txn = genesis_account.create_signed_txn_impl(
        *genesis_account.address(),
        TransactionPayload::WriteSet(ChangeSet::new(write_set, vec![])),
        1,
        100_000,
        0,
        LBR_NAME.to_owned(),
    );

    let output = executor.execute_transaction(writeset_txn);
    assert_eq!(
        output.status(),
        &TransactionStatus::Discard(VMStatus::new(StatusCode::INVALID_WRITE_SET))
    );
}

#[test]
fn transfer_and_execute_writeset() {
    // create a FakeExecutor with a genesis from file
    let mut executor = FakeExecutor::from_genesis_file();
    let genesis_account = Account::new_association();
    executor.new_block();

    let receiver = AccountData::new(100_000, 10);
    executor.add_account_data(&receiver);

    // (1) Association mint some coin
    let mint_amount = 1_000_000;

    executor.execute_and_apply(genesis_account.signed_script_txn(
        encode_mint_lbr_to_address_script(genesis_account.address(), vec![], mint_amount),
        1,
    ));

    // (2) Create a WriteSet that adds an account on a new address
    let new_account_data = AccountData::new(1000, 10);
    let write_set = new_account_data.to_writeset();

    let writeset_txn = genesis_account.create_signed_txn_impl(
        *genesis_account.address(),
        TransactionPayload::WriteSet(ChangeSet::new(write_set, vec![])),
        2,
        100_000,
        1,
        LBR_NAME.to_owned(),
    );

    let output = executor.execute_transaction(writeset_txn.clone());
    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED))
    );
    assert!(executor.verify_transaction(writeset_txn).status().is_none());

    executor.apply_write_set(output.write_set());

    let updated_association_account = executor
        .read_account_resource(&genesis_account)
        .expect("sender must exist");
    let updated_sender = executor
        .read_account_resource(new_account_data.account())
        .expect("sender must exist");
    let updated_sender_balance = executor
        .read_balance_resource(new_account_data.account(), account::lbr_currency_code())
        .expect("sender balance must exist");

    assert_eq!(3, updated_association_account.sequence_number());
    assert_eq!(1000, updated_sender_balance.coin());
    assert_eq!(10, updated_sender.sequence_number());

    // (3) Create another transfer
    let transfer_amount = 1_000;
    let txn = peer_to_peer_txn(&genesis_account, receiver.account(), 3, transfer_amount);

    // execute transaction
    let output = executor.execute_transaction(txn);
    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED))
    );

    executor.apply_write_set(output.write_set());
}
