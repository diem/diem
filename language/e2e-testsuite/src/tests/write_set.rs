// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_crypto::{ed25519::Ed25519PrivateKey, PrivateKey, Uniform};
use diem_types::{
    access_path::AccessPath,
    account_config::{xus_tag, CORE_CODE_ADDRESS},
    chain_id::{ChainId, NamedChain},
    contract_event::ContractEvent,
    on_chain_config::new_epoch_event_key,
    transaction::{
        authenticator::AuthenticationKey, ChangeSet, TransactionStatus, WriteSetPayload,
    },
    vm_status::{KeptVMStatus, StatusCode},
    write_set::{WriteOp, WriteSet, WriteSetMut},
};
use language_e2e_tests::{
    account::{self, Account},
    assert_prologue_parity,
    common_transactions::rotate_key_txn,
    current_function_name,
    executor::FakeExecutor,
    transaction_status_eq,
};
use move_core_types::{
    identifier::Identifier,
    language_storage::{ResourceKey, StructTag},
};

#[test]
fn invalid_write_set_signer() {
    let mut executor = FakeExecutor::from_genesis_file();
    executor.set_golden_file(current_function_name!());
    let genesis_account = Account::new_diem_root();
    executor.new_block();

    // Create a WriteSet that adds an account on a new address.
    let new_account_data = executor.create_raw_account_data(0, 10);
    let write_set = new_account_data.to_writeset();

    // Signing the txn with a key that does not match the sender should fail.
    let writeset_txn = genesis_account
        .transaction()
        .write_set(WriteSetPayload::Direct(ChangeSet::new(write_set, vec![])))
        .sequence_number(1)
        .raw()
        .sign(
            &new_account_data.account().privkey,
            new_account_data.account().pubkey.clone(),
        )
        .unwrap()
        .into_inner();

    assert_prologue_parity!(
        executor.verify_transaction(writeset_txn.clone()).status(),
        executor.execute_transaction(writeset_txn).status(),
        StatusCode::INVALID_AUTH_KEY
    );
}

#[test]
fn verify_and_execute_writeset() {
    let mut executor = FakeExecutor::from_genesis_file();
    executor.set_golden_file(current_function_name!());
    let genesis_account = Account::new_diem_root();
    executor.new_block();

    // Create a WriteSet that adds an account on a new address.
    let new_account_data = executor.create_raw_account_data(0, 10);
    let write_set = new_account_data.to_writeset();

    // (1) Test that a correct WriteSet is executed as expected.
    let writeset_txn = genesis_account
        .transaction()
        .write_set(WriteSetPayload::Direct(ChangeSet::new(
            write_set.clone(),
            vec![],
        )))
        .sequence_number(1)
        .sign();
    let output = executor.execute_transaction(writeset_txn.clone());
    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(KeptVMStatus::Executed)
    );
    assert!(executor
        .verify_transaction(writeset_txn.clone())
        .status()
        .is_none());

    executor.apply_write_set(output.write_set());

    let updated_diem_root_account = executor
        .read_account_resource(&genesis_account)
        .expect("sender must exist");
    let updated_sender = executor
        .read_account_resource(new_account_data.account())
        .expect("sender must exist");
    let updated_sender_balance = executor
        .read_balance_resource(new_account_data.account(), account::xus_currency_code())
        .expect("sender balance must exist");

    assert_eq!(2, updated_diem_root_account.sequence_number());
    assert_eq!(0, updated_sender_balance.coin());
    assert_eq!(10, updated_sender.sequence_number());

    // (2) Cannot reapply the same writeset.
    assert_prologue_parity!(
        executor.verify_transaction(writeset_txn.clone()).status(),
        executor.execute_transaction(writeset_txn).status(),
        StatusCode::SEQUENCE_NUMBER_TOO_OLD
    );

    // (3) Cannot apply the writeset with future sequence number.
    let writeset_txn = genesis_account
        .transaction()
        .write_set(WriteSetPayload::Direct(ChangeSet::new(write_set, vec![])))
        .sequence_number(10)
        .sign();
    let output = executor.execute_transaction(writeset_txn.clone());
    assert_eq!(
        output.status(),
        &TransactionStatus::Discard(StatusCode::SEQUENCE_NUMBER_TOO_NEW)
    );
    // "Too new" sequence numbers are accepted during validation.
    assert!(executor.verify_transaction(writeset_txn).status().is_none());
}

#[test]
fn bad_writesets() {
    let mut executor = FakeExecutor::from_genesis_file();
    executor.set_golden_file(current_function_name!());
    let genesis_account = Account::new_diem_root();
    executor.new_block();

    // Create a WriteSet that adds an account on a new address
    let new_account_data = executor.create_raw_account_data(1000, 10);
    let write_set = new_account_data.to_writeset();

    // (1) A WriteSet signed by an arbitrary account, not Diem root, should be rejected.
    let writeset_txn = new_account_data
        .account()
        .transaction()
        .write_set(WriteSetPayload::Direct(ChangeSet::new(
            write_set.clone(),
            vec![],
        )))
        .sequence_number(1)
        .sign();
    assert_prologue_parity!(
        executor.verify_transaction(writeset_txn.clone()).status(),
        executor.execute_transaction(writeset_txn).status(),
        StatusCode::REJECTED_WRITE_SET
    );

    // (2) A WriteSet containing a reconfiguration event should be dropped.
    let event = ContractEvent::new(new_epoch_event_key(), 0, xus_tag(), vec![]);
    let writeset_txn = genesis_account
        .transaction()
        .write_set(WriteSetPayload::Direct(ChangeSet::new(
            write_set,
            vec![event],
        )))
        .sequence_number(1)
        .sign();
    assert_eq!(
        executor.execute_transaction(writeset_txn).status(),
        &TransactionStatus::Discard(StatusCode::INVALID_WRITE_SET)
    );

    // (3) A WriteSet attempting to change DiemWriteSetManager should be dropped.
    let key = ResourceKey::new(
        *genesis_account.address(),
        StructTag {
            address: CORE_CODE_ADDRESS,
            module: Identifier::new("DiemAccount").unwrap(),
            name: Identifier::new("DiemWriteSetManager").unwrap(),
            type_params: vec![],
        },
    );
    let path = AccessPath::resource_access_path(key);

    let write_set = WriteSetMut::new(vec![(path, WriteOp::Value(vec![]))])
        .freeze()
        .unwrap();
    let writeset_txn = genesis_account
        .transaction()
        .write_set(WriteSetPayload::Direct(ChangeSet::new(write_set, vec![])))
        .sequence_number(1)
        .sign();
    let output = executor.execute_transaction(writeset_txn);
    assert_eq!(
        output.status(),
        &TransactionStatus::Discard(StatusCode::INVALID_WRITE_SET)
    );

    // (4) A WriteSet attempting to change Diem root AccountResource should be dropped.
    let key = ResourceKey::new(
        *genesis_account.address(),
        StructTag {
            address: CORE_CODE_ADDRESS,
            module: Identifier::new("DiemAccount").unwrap(),
            name: Identifier::new("DiemAccount").unwrap(),
            type_params: vec![],
        },
    );
    let path = AccessPath::resource_access_path(key);

    let write_set = WriteSetMut::new(vec![(path, WriteOp::Value(vec![]))])
        .freeze()
        .unwrap();
    let writeset_txn = genesis_account
        .transaction()
        .write_set(WriteSetPayload::Direct(ChangeSet::new(write_set, vec![])))
        .sequence_number(1)
        .sign();
    let output = executor.execute_transaction(writeset_txn);
    assert_eq!(
        output.status(),
        &TransactionStatus::Discard(StatusCode::INVALID_WRITE_SET)
    );

    // (5) A WriteSet with a bad ChainId should be rejected.
    let writeset_txn = genesis_account
        .transaction()
        .write_set(WriteSetPayload::Direct(ChangeSet::new(
            WriteSet::default(),
            vec![],
        )))
        .sequence_number(1)
        .chain_id(ChainId::new(NamedChain::DEVNET.id()))
        .sign();
    assert_prologue_parity!(
        executor.verify_transaction(writeset_txn.clone()).status(),
        executor.execute_transaction(writeset_txn).status(),
        StatusCode::BAD_CHAIN_ID
    );

    // (6) A WriteSet that has expired should be rejected.
    let writeset_txn = genesis_account
        .transaction()
        .write_set(WriteSetPayload::Direct(ChangeSet::new(
            WriteSet::default(),
            vec![],
        )))
        .sequence_number(1)
        .ttl(0)
        .sign();
    assert_prologue_parity!(
        executor.verify_transaction(writeset_txn.clone()).status(),
        executor.execute_transaction(writeset_txn).status(),
        StatusCode::TRANSACTION_EXPIRED
    );

    // (7) The gas currency specified in the transaction must be valid
    // (even though WriteSet transactions are not charged for gas).
    let writeset_txn = genesis_account
        .transaction()
        .write_set(WriteSetPayload::Direct(ChangeSet::new(
            WriteSet::default(),
            vec![],
        )))
        .sequence_number(1)
        .gas_currency_code("Bad_ID")
        .sign();
    assert_eq!(
        executor.verify_transaction(writeset_txn).status().unwrap(),
        StatusCode::INVALID_GAS_SPECIFIER
    );

    // (8) The gas currency code must also correspond to a registered currency
    // (even though WriteSet transactions are not charged for gas).
    let writeset_txn = genesis_account
        .transaction()
        .write_set(WriteSetPayload::Direct(ChangeSet::new(
            WriteSet::default(),
            vec![],
        )))
        .sequence_number(1)
        .gas_currency_code("INVALID")
        .sign();
    assert_eq!(
        executor.verify_transaction(writeset_txn).status().unwrap(),
        StatusCode::CURRENCY_INFO_DOES_NOT_EXIST
    );
}

#[test]
fn transfer_and_execute_writeset() {
    let mut executor = FakeExecutor::from_genesis_file();
    executor.set_golden_file(current_function_name!());
    let genesis_account = Account::new_diem_root();
    let blessed_account = Account::new_blessed_tc();
    executor.new_block();

    let receiver = executor.create_raw_account_data(100_000, 10);
    executor.add_account_data(&receiver);

    // (1) Association mint some coin
    let privkey = Ed25519PrivateKey::generate_for_testing();
    let pubkey = privkey.public_key();
    let new_key_hash = AuthenticationKey::ed25519(&pubkey).to_vec();

    executor.execute_and_apply(rotate_key_txn(&blessed_account, new_key_hash, 0));

    // (2) Create a WriteSet that adds an account on a new address
    let new_account_data = executor.create_raw_account_data(0, 10);
    let write_set = new_account_data.to_writeset();

    let writeset_txn = genesis_account
        .transaction()
        .write_set(WriteSetPayload::Direct(ChangeSet::new(write_set, vec![])))
        .sequence_number(1)
        .sign();

    let output = executor.execute_transaction(writeset_txn.clone());
    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(KeptVMStatus::Executed)
    );
    assert!(executor.verify_transaction(writeset_txn).status().is_none());

    executor.apply_write_set(output.write_set());

    let updated_diem_root_account = executor
        .read_account_resource(&genesis_account)
        .expect("sender must exist");
    let updated_sender = executor
        .read_account_resource(new_account_data.account())
        .expect("sender must exist");
    let updated_sender_balance = executor
        .read_balance_resource(new_account_data.account(), account::xus_currency_code())
        .expect("sender balance must exist");

    assert_eq!(2, updated_diem_root_account.sequence_number());
    assert_eq!(0, updated_sender_balance.coin());
    assert_eq!(10, updated_sender.sequence_number());

    // (3) Rotate the accounts key
    let privkey = Ed25519PrivateKey::generate_for_testing();
    let pubkey = privkey.public_key();
    let new_key_hash = AuthenticationKey::ed25519(&pubkey).to_vec();
    let txn = rotate_key_txn(new_account_data.account(), new_key_hash, 10);

    // execute transaction
    let output = executor.execute_transaction(txn);
    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(KeptVMStatus::Executed)
    );

    executor.apply_write_set(output.write_set());
}
