// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use language_e2e_tests::{
    account::{self, Account, AccountData},
    common_transactions::rotate_key_txn,
    executor::FakeExecutor,
};
use libra_crypto::{ed25519::Ed25519PrivateKey, PrivateKey, Uniform};
use libra_types::{
    access_path::AccessPath,
    account_config::{lbr_type_tag, CORE_CODE_ADDRESS},
    chain_id::{ChainId, NamedChain},
    contract_event::ContractEvent,
    on_chain_config::new_epoch_event_key,
    transaction::{
        authenticator::AuthenticationKey, ChangeSet, TransactionStatus, WriteSetPayload,
    },
    vm_status::{KeptVMStatus, StatusCode},
    write_set::{WriteOp, WriteSet, WriteSetMut},
};
use move_core_types::{
    identifier::Identifier,
    language_storage::{ResourceKey, StructTag},
    move_resource::MoveResource,
};

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

    let writeset_txn = sender_account
        .account()
        .transaction()
        .write_set(WriteSetPayload::Direct(ChangeSet::new(write_set, vec![])))
        .sequence_number(0)
        .sign();
    let output = executor.execute_transaction(writeset_txn);
    assert_eq!(
        output.status(),
        &TransactionStatus::Discard(StatusCode::REJECTED_WRITE_SET),
    );
}

#[test]
fn invalid_write_set_signer() {
    // create a FakeExecutor with a genesis from file
    let mut executor = FakeExecutor::from_genesis_file();
    let genesis_account = Account::new_libra_root();
    executor.new_block();

    // (1) Create a WriteSet that adds an account on a new address
    let new_account_data = AccountData::new(0, 10);
    let write_set = new_account_data.to_writeset();

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
    let output = executor.execute_transaction(writeset_txn.clone());
    assert_eq!(
        output.status(),
        &TransactionStatus::Discard(StatusCode::REJECTED_WRITE_SET)
    );
    assert_eq!(
        executor.verify_transaction(writeset_txn).status(),
        Some(StatusCode::REJECTED_WRITE_SET)
    );
}

#[test]
fn verify_and_execute_writeset() {
    // create a FakeExecutor with a genesis from file
    let mut executor = FakeExecutor::from_genesis_file();
    let genesis_account = Account::new_libra_root();
    executor.new_block();

    // (1) Create a WriteSet that adds an account on a new address
    let new_account_data = AccountData::new(0, 10);
    let write_set = new_account_data.to_writeset();

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

    let updated_libra_root_account = executor
        .read_account_resource(&genesis_account)
        .expect("sender must exist");
    let updated_sender = executor
        .read_account_resource(new_account_data.account())
        .expect("sender must exist");
    let updated_sender_balance = executor
        .read_balance_resource(new_account_data.account(), account::lbr_currency_code())
        .expect("sender balance must exist");

    assert_eq!(2, updated_libra_root_account.sequence_number());
    assert_eq!(0, updated_sender_balance.coin());
    assert_eq!(10, updated_sender.sequence_number());

    // (2) Cannot reapply the same writeset.
    let output = executor.execute_transaction(writeset_txn.clone());
    assert_eq!(
        output.status(),
        &TransactionStatus::Discard(StatusCode::REJECTED_WRITE_SET)
    );
    assert_eq!(
        executor.verify_transaction(writeset_txn).status().unwrap(),
        StatusCode::REJECTED_WRITE_SET,
    );

    // (3) Cannot apply the writeset with future sequence number.
    let writeset_txn = genesis_account
        .transaction()
        .write_set(WriteSetPayload::Direct(ChangeSet::new(write_set, vec![])))
        .sequence_number(10)
        .sign();
    let output = executor.execute_transaction(writeset_txn.clone());
    let status = output.status();
    assert!(status.is_discarded());
    assert_eq!(
        output.status(),
        &TransactionStatus::Discard(StatusCode::REJECTED_WRITE_SET)
    );
    let err = executor.verify_transaction(writeset_txn).status().unwrap();
    assert_eq!(err, StatusCode::REJECTED_WRITE_SET);
}

#[test]
fn bad_writesets() {
    // create a FakeExecutor with a genesis from file
    let mut executor = FakeExecutor::from_genesis_file();
    let genesis_account = Account::new_libra_root();
    executor.new_block();

    // Create a WriteSet that adds an account on a new address
    let new_account_data = AccountData::new(1000, 10);
    let write_set = new_account_data.to_writeset();

    // (1) This WriteSet is signed by an arbitrary account rather than the libra root account. Should be
    // rejected.
    let writeset_txn = new_account_data
        .account()
        .transaction()
        .write_set(WriteSetPayload::Direct(ChangeSet::new(
            write_set,
            vec![],
        )))
        .sequence_number(1)
        .sign();
    let output = executor.execute_transaction(writeset_txn);
    assert_eq!(
        output.status(),
        &TransactionStatus::Discard(StatusCode::REJECTED_WRITE_SET)
    );

    // (2) The WriteSet contains a conflicting reconfiguration event from that emitted by the
    // writeset epilogue, will be dropped.
    let event = ContractEvent::new(new_epoch_event_key(), 1, lbr_type_tag(), vec![]);
    let writeset_txn = genesis_account
        .transaction()
        .write_set(WriteSetPayload::Direct(ChangeSet::new(
            WriteSet::default(),
            vec![event],
        )))
        .sequence_number(1)
        .sign();
    let output = executor.execute_transaction(writeset_txn);
    assert_eq!(
        output.status(),
        &TransactionStatus::Discard(StatusCode::INVALID_WRITE_SET)
    );

    // (3) The WriteSet attempts to change LibraWriteSetManager, will be dropped.
    let key = ResourceKey::new(
        *genesis_account.address(),
        StructTag {
            address: CORE_CODE_ADDRESS,
            module: Identifier::new("LibraAccount").unwrap(),
            name: Identifier::new("LibraWriteSetManager").unwrap(),
            type_params: vec![],
        },
    );
    let path = AccessPath::resource_access_path(&key);

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

    // (4) The WriteSet attempts to change libra root AccountResource, will be dropped.
    let key = ResourceKey::new(
        *genesis_account.address(),
        StructTag {
            address: CORE_CODE_ADDRESS,
            module: Identifier::new("LibraAccount").unwrap(),
            name: Identifier::new("LibraAccount").unwrap(),
            type_params: vec![],
        },
    );
    let path = AccessPath::resource_access_path(&key);

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

    // (5) The WriteSet has bad ChainId
    let writeset_txn = genesis_account
        .transaction()
        .write_set(WriteSetPayload::Direct(ChangeSet::new(
            WriteSet::default(),
            vec![],
        )))
        .sequence_number(1)
        .chain_id(ChainId::new(NamedChain::DEVNET.id()))
        .sign();
    let output = executor.execute_transaction(writeset_txn);
    assert_eq!(
        output.status(),
        &TransactionStatus::Discard(StatusCode::REJECTED_WRITE_SET)
    );

    // (6) The WriteSet has expired
    let writeset_txn = genesis_account
        .transaction()
        .write_set(WriteSetPayload::Direct(ChangeSet::new(
            WriteSet::default(),
            vec![],
        )))
        .sequence_number(1)
        .ttl(0)
        .sign();
    let output = executor.execute_transaction(writeset_txn);
    assert_eq!(
        output.status(),
        &TransactionStatus::Discard(StatusCode::REJECTED_WRITE_SET)
    );

    // (7) The WriteSet attempts to write the Configuration to the same value as emitted from
    // writeset epilogue, should be kept
    let key = ResourceKey::new(
        *genesis_account.address(),
        libra_types::on_chain_config::ConfigurationResource::struct_tag(),
    );
    let path = AccessPath::resource_access_path(&key);

    let configuration_value =
        lcs::to_bytes(&libra_types::on_chain_config::ConfigurationResource::new_for_test(2, 1, 2))
            .unwrap();
    let write_set = WriteSetMut::new(vec![(path, WriteOp::Value(configuration_value))])
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
        &TransactionStatus::Keep(KeptVMStatus::Executed),
    );

    // (8) The WriteSet contains the same reconfiguration event as that emitted from writeset
    // epilogue, will be kept.
    let event = ContractEvent::new(new_epoch_event_key(), 2, lbr_type_tag(), vec![2]);
    let writeset_txn = genesis_account
        .transaction()
        .write_set(WriteSetPayload::Direct(ChangeSet::new(
            WriteSet::default(),
            vec![event],
        )))
        .sequence_number(1)
        .sign();
    let output = executor.execute_transaction(writeset_txn);
    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(KeptVMStatus::Executed),
    );
}

#[test]
fn transfer_and_execute_writeset() {
    // create a FakeExecutor with a genesis from file
    let mut executor = FakeExecutor::from_genesis_file();
    let genesis_account = Account::new_libra_root();
    let blessed_account = Account::new_blessed_tc();
    executor.new_block();

    let receiver = AccountData::new(100_000, 10);
    executor.add_account_data(&receiver);

    // (1) Association mint some coin
    let privkey = Ed25519PrivateKey::generate_for_testing();
    let pubkey = privkey.public_key();
    let new_key_hash = AuthenticationKey::ed25519(&pubkey).to_vec();

    executor.execute_and_apply(rotate_key_txn(&blessed_account, new_key_hash, 0));

    // (2) Create a WriteSet that adds an account on a new address
    let new_account_data = AccountData::new(0, 10);
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

    let updated_libra_root_account = executor
        .read_account_resource(&genesis_account)
        .expect("sender must exist");
    let updated_sender = executor
        .read_account_resource(new_account_data.account())
        .expect("sender must exist");
    let updated_sender_balance = executor
        .read_balance_resource(new_account_data.account(), account::lbr_currency_code())
        .expect("sender balance must exist");

    assert_eq!(2, updated_libra_root_account.sequence_number());
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
