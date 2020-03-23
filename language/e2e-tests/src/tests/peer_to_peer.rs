// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account::{Account, AccountData},
    common_transactions::peer_to_peer_txn,
    executor::FakeExecutor,
    gas_costs, transaction_status_eq,
};
use libra_types::{
    account_config::{lbr_type_tag, ReceivedPaymentEvent, SentPaymentEvent},
    on_chain_config::VMPublishingOption,
    transaction::{SignedTransaction, TransactionOutput, TransactionPayload, TransactionStatus},
    vm_error::{StatusCode, VMStatus},
};
use std::{convert::TryFrom, time::Instant};

#[test]
fn single_peer_to_peer_with_event() {
    ::libra_logger::Logger::new().environment_only(true).init();
    // create a FakeExecutor with a genesis from file
    let mut executor = FakeExecutor::from_genesis_file();
    // create and publish a sender with 1_000_000 coins and a receiver with 100_000 coins
    let sender = AccountData::new(1_000_000, 10);
    let receiver = AccountData::new(100_000, 10);
    executor.add_account_data(&sender);
    executor.add_account_data(&receiver);

    let transfer_amount = 1_000;
    let txn = peer_to_peer_txn(sender.account(), receiver.account(), 10, transfer_amount);

    // execute transaction
    let output = executor.execute_transaction(txn);
    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED))
    );

    executor.apply_write_set(output.write_set());

    // check that numbers in stored DB are correct
    let gas = output.gas_used();
    let sender_balance = 1_000_000 - transfer_amount - gas;
    let receiver_balance = 100_000 + transfer_amount;
    let (updated_sender, updated_sender_balance) = executor
        .read_account_info(sender.account())
        .expect("sender must exist");
    let (updated_receiver, updated_receiver_balance) = executor
        .read_account_info(receiver.account())
        .expect("receiver must exist");
    assert_eq!(receiver_balance, updated_receiver_balance.coin());
    assert_eq!(sender_balance, updated_sender_balance.coin());
    assert_eq!(11, updated_sender.sequence_number());
    assert_eq!(0, updated_sender.received_events().count(),);
    assert_eq!(1, updated_sender.sent_events().count());
    assert_eq!(1, updated_receiver.received_events().count());
    assert_eq!(0, updated_receiver.sent_events().count());

    let rec_ev_path = receiver.received_events_key().to_vec();
    let sent_ev_path = sender.sent_events_key().to_vec();
    for event in output.events() {
        assert!(
            rec_ev_path.as_slice() == event.key().as_bytes()
                || sent_ev_path.as_slice() == event.key().as_bytes()
        );
    }
}

#[test]
fn single_peer_to_peer_with_padding() {
    ::libra_logger::Logger::new().environment_only(true).init();
    // create a FakeExecutor with a genesis from file
    let mut executor = FakeExecutor::from_genesis_with_options(VMPublishingOption::CustomScripts);

    // create and publish a sender with 1_000_000 coins and a receiver with 100_000 coins
    let sender = AccountData::new(1_000_000, 10);
    let receiver = AccountData::new(100_000, 10);
    executor.add_account_data(&sender);
    executor.add_account_data(&receiver);

    let transfer_amount = 1_000;
    let txn = sender.account().create_signed_txn_impl(
        *sender.address(),
        TransactionPayload::Script(transaction_builder::encode_transfer_script_with_padding(
            receiver.address(),
            transfer_amount,
            1000,
        )),
        10,
        gas_costs::TXN_RESERVED, // this is a default for gas
        1,
        lbr_type_tag(),
    );
    let unpadded_txn = peer_to_peer_txn(sender.account(), receiver.account(), 10, transfer_amount);
    assert!(txn.raw_txn_bytes_len() > unpadded_txn.raw_txn_bytes_len());
    // execute transaction
    let output = executor.execute_transaction(txn);
    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED))
    );

    executor.apply_write_set(output.write_set());

    // check that numbers in stored DB are correct
    let gas = output.gas_used();
    let sender_balance = 1_000_000 - transfer_amount - gas;
    let receiver_balance = 100_000 + transfer_amount;
    let (updated_sender, updated_sender_balance) = executor
        .read_account_info(sender.account())
        .expect("sender must exist");
    let updated_receiver_balance = executor
        .read_balance_resource(receiver.account())
        .expect("receiver balance must exist");
    assert_eq!(receiver_balance, updated_receiver_balance.coin());
    assert_eq!(sender_balance, updated_sender_balance.coin());
    assert_eq!(11, updated_sender.sequence_number());
}

#[test]
fn few_peer_to_peer_with_event() {
    // create a FakeExecutor with a genesis from file
    let mut executor = FakeExecutor::from_genesis_file();
    // create and publish a sender with 1_000_000 coins and a receiver with 100_000 coins
    let sender = AccountData::new(1_000_000, 10);
    let receiver = AccountData::new(100_000, 10);
    executor.add_account_data(&sender);
    executor.add_account_data(&receiver);

    let transfer_amount = 1_000;

    // execute transaction
    let txns: Vec<SignedTransaction> = vec![
        peer_to_peer_txn(sender.account(), receiver.account(), 10, transfer_amount),
        peer_to_peer_txn(sender.account(), receiver.account(), 11, transfer_amount),
        peer_to_peer_txn(sender.account(), receiver.account(), 12, transfer_amount),
        peer_to_peer_txn(sender.account(), receiver.account(), 13, transfer_amount),
    ];
    let output = executor.execute_block(txns).unwrap();
    for (idx, txn_output) in output.iter().enumerate() {
        assert_eq!(
            txn_output.status(),
            &TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED))
        );

        // check events
        for event in txn_output.events() {
            if let Ok(payload) = SentPaymentEvent::try_from(event) {
                assert_eq!(transfer_amount, payload.amount());
                assert_eq!(receiver.address(), &payload.receiver());
            } else if let Ok(payload) = ReceivedPaymentEvent::try_from(event) {
                assert_eq!(transfer_amount, payload.amount());
                assert_eq!(sender.address(), &payload.sender());
            } else {
                panic!("Unexpected Event Type")
            }
        }

        let original_sender_balance = executor
            .read_balance_resource(sender.account())
            .expect("sender balance must exist");
        let original_receiver_balance = executor
            .read_balance_resource(receiver.account())
            .expect("receiver balcne must exist");
        executor.apply_write_set(txn_output.write_set());

        // check that numbers in stored DB are correct
        let gas = txn_output.gas_used();
        let sender_balance = original_sender_balance.coin() - transfer_amount - gas;
        let receiver_balance = original_receiver_balance.coin() + transfer_amount;
        let (updated_sender, updated_sender_balance) = executor
            .read_account_info(sender.account())
            .expect("sender must exist");
        let (updated_receiver, updated_receiver_balance) = executor
            .read_account_info(receiver.account())
            .expect("receiver must exist");
        assert_eq!(receiver_balance, updated_receiver_balance.coin());
        assert_eq!(sender_balance, updated_sender_balance.coin());
        assert_eq!(11 + idx as u64, updated_sender.sequence_number());
        assert_eq!(0, updated_sender.received_events().count());
        assert_eq!(idx as u64 + 1, updated_sender.sent_events().count());
        assert_eq!(idx as u64 + 1, updated_receiver.received_events().count());
        assert_eq!(0, updated_receiver.sent_events().count());
    }
}

/// Test that a zero-amount transaction fails, per policy.
#[test]
fn zero_amount_peer_to_peer() {
    let mut executor = FakeExecutor::from_genesis_file();
    let sequence_number = 10;
    let sender = AccountData::new(1_000_000, sequence_number);
    let receiver = AccountData::new(100_000, sequence_number);
    executor.add_account_data(&sender);
    executor.add_account_data(&receiver);

    let transfer_amount = 0;
    let txn = peer_to_peer_txn(
        sender.account(),
        receiver.account(),
        sequence_number,
        transfer_amount,
    );

    let output = &executor.execute_transaction(txn);
    // Error code 7 means that the transaction was a zero-amount one.
    assert!(transaction_status_eq(
        &output.status(),
        &TransactionStatus::Keep(VMStatus::new(StatusCode::ABORTED).with_sub_status(7))
    ));
}

#[test]
fn peer_to_peer_create_account() {
    // create a FakeExecutor with a genesis from file
    let mut executor = FakeExecutor::from_genesis_file();
    // create and publish a sender with 1_000_000 coins
    let sender = AccountData::new(1_000_000, 10);
    executor.add_account_data(&sender);
    let new_account = Account::new();

    // define the arguments to the peer to peer transaction
    let transfer_amount = 1_000;
    let txn = peer_to_peer_txn(sender.account(), &new_account, 10, transfer_amount);

    // execute transaction
    let output = executor.execute_transaction(txn);
    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED))
    );
    executor.apply_write_set(output.write_set());

    // check that numbers in stored DB are correct
    let gas = output.gas_used();
    let sender_balance = 1_000_000 - transfer_amount - gas;
    let receiver_balance = transfer_amount;
    let (updated_sender, updated_sender_balance) = executor
        .read_account_info(sender.account())
        .expect("sender must exist");
    let updated_receiver_balance = executor
        .read_balance_resource(&new_account)
        .expect("receiver must exist");
    assert_eq!(receiver_balance, updated_receiver_balance.coin());
    assert_eq!(sender_balance, updated_sender_balance.coin());
    assert_eq!(11, updated_sender.sequence_number());
}

// Holder for transaction data; arguments to transactions.
struct TxnInfo {
    pub sender: Account,
    pub receiver: Account,
    pub transfer_amount: u64,
}

impl TxnInfo {
    fn new(sender: &Account, receiver: &Account, transfer_amount: u64) -> Self {
        TxnInfo {
            sender: sender.clone(),
            receiver: receiver.clone(),
            transfer_amount,
        }
    }
}

// Create a cyclic transfer around a slice of Accounts.
// Each Account makes a transfer for the same amount to the next LibraAccount.
fn create_cyclic_transfers(
    executor: &FakeExecutor,
    accounts: &[Account],
    transfer_amount: u64,
) -> (Vec<TxnInfo>, Vec<SignedTransaction>) {
    let mut txns: Vec<SignedTransaction> = Vec::new();
    let mut txns_info: Vec<TxnInfo> = Vec::new();
    // loop through all transactions and let each transfer the same amount to the next one
    let count = accounts.len();
    for i in 0..count {
        let sender = &accounts[i];
        let sender_resource = executor
            .read_account_resource(&sender)
            .expect("sender must exist");
        let seq_num = sender_resource.sequence_number();
        let receiver = &accounts[(i + 1) % count];

        let txn = peer_to_peer_txn(sender, receiver, seq_num, transfer_amount);
        txns.push(txn);
        txns_info.push(TxnInfo::new(sender, receiver, transfer_amount));
    }
    (txns_info, txns)
}

// Create a one to many transfer around a slice of Accounts.
// The first account is the payer and all others are receivers.
fn create_one_to_many_transfers(
    executor: &FakeExecutor,
    accounts: &[Account],
    transfer_amount: u64,
) -> (Vec<TxnInfo>, Vec<SignedTransaction>) {
    let mut txns: Vec<SignedTransaction> = Vec::new();
    let mut txns_info: Vec<TxnInfo> = Vec::new();
    // grab account 0 as a sender
    let sender = &accounts[0];
    let sender_resource = executor
        .read_account_resource(&sender)
        .expect("sender must exist");
    let seq_num = sender_resource.sequence_number();
    // loop through all transactions and let each transfer the same amount to the next one
    let count = accounts.len();
    for (i, receiver) in accounts.iter().enumerate().take(count).skip(1) {
        // let receiver = &accounts[i];

        let txn = peer_to_peer_txn(sender, receiver, seq_num + i as u64 - 1, transfer_amount);
        txns.push(txn);
        txns_info.push(TxnInfo::new(sender, receiver, transfer_amount));
    }
    (txns_info, txns)
}

// Create a many to one transfer around a slice of Accounts.
// The first account is the receiver and all others are payers.
fn create_many_to_one_transfers(
    executor: &FakeExecutor,
    accounts: &[Account],
    transfer_amount: u64,
) -> (Vec<TxnInfo>, Vec<SignedTransaction>) {
    let mut txns: Vec<SignedTransaction> = Vec::new();
    let mut txns_info: Vec<TxnInfo> = Vec::new();
    // grab account 0 as a sender
    let receiver = &accounts[0];
    // loop through all transactions and let each transfer the same amount to the next one
    let count = accounts.len();
    for sender in accounts.iter().take(count).skip(1) {
        //let sender = &accounts[i];
        let sender_resource = executor
            .read_account_resource(sender)
            .expect("sender must exist");
        let seq_num = sender_resource.sequence_number();

        let txn = peer_to_peer_txn(sender, receiver, seq_num, transfer_amount);
        txns.push(txn);
        txns_info.push(TxnInfo::new(sender, receiver, transfer_amount));
    }
    (txns_info, txns)
}

// Verify a transfer output.
// Checks that sender and receiver in a peer to peer transaction are in proper
// state after a successful transfer.
// The transaction arguments are provided in txn_args.
// Apply the WriteSet to the data store.
fn check_and_apply_transfer_output(
    executor: &mut FakeExecutor,
    txn_args: &[TxnInfo],
    output: &[TransactionOutput],
) {
    let count = output.len();
    for i in 0..count {
        let txn_info = &txn_args[i];
        let sender = &txn_info.sender;
        let receiver = &txn_info.receiver;
        let transfer_amount = txn_info.transfer_amount;
        let (sender_resource, sender_balance) = executor
            .read_account_info(&sender)
            .expect("sender must exist");
        let sender_initial_balance = sender_balance.coin();
        let sender_seq_num = sender_resource.sequence_number();
        let receiver_initial_balance = executor
            .read_balance_resource(&receiver)
            .expect("receiver must exist")
            .coin();

        // apply single transaction to DB
        let txn_output = &output[i];
        executor.apply_write_set(txn_output.write_set());

        // check that numbers stored in DB are correct
        let gas = txn_output.gas_used();
        let sender_balance = sender_initial_balance - transfer_amount - gas;
        let receiver_balance = receiver_initial_balance + transfer_amount;
        let (updated_sender, updated_sender_balance) = executor
            .read_account_info(&sender)
            .expect("sender must exist");
        let updated_receiver_balance = executor
            .read_balance_resource(&receiver)
            .expect("receiver balance must exist");
        assert_eq!(receiver_balance, updated_receiver_balance.coin());
        assert_eq!(sender_balance, updated_sender_balance.coin());
        assert_eq!(sender_seq_num + 1, updated_sender.sequence_number());
    }
}

// simple utility to print all account to visually inspect account data
fn print_accounts(executor: &FakeExecutor, accounts: &[Account]) {
    for account in accounts {
        let account_resource = executor
            .read_account_resource(&account)
            .expect("sender must exist");
        println!("{:?}", account_resource);
    }
}

#[test]
fn cycle_peer_to_peer() {
    // create a FakeExecutor with a genesis from file
    let mut executor = FakeExecutor::from_genesis_file();
    // create and publish accounts with 2_000_000 coins
    let account_size = 100usize;
    let initial_balance = 2_000_000u64;
    let initial_seq_num = 10u64;
    let accounts = executor.create_accounts(account_size, initial_balance, initial_seq_num);

    // set up the transactions
    let transfer_amount = 1_000;
    let (txns_info, txns) = create_cyclic_transfers(&executor, &accounts, transfer_amount);

    // execute transaction
    let mut execution_time = 0u128;
    let now = Instant::now();
    let output = executor.execute_block(txns).unwrap();
    execution_time += now.elapsed().as_nanos();
    println!("EXECUTION TIME: {}", execution_time);
    for txn_output in &output {
        assert_eq!(
            txn_output.status(),
            &TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED))
        );
    }
    assert_eq!(accounts.len(), output.len());

    check_and_apply_transfer_output(&mut executor, &txns_info, &output);
    print_accounts(&executor, &accounts);
}

#[test]
fn cycle_peer_to_peer_multi_block() {
    // create a FakeExecutor with a genesis from file
    let mut executor = FakeExecutor::from_genesis_file();
    // create and publish accounts with 1_000_000 coins
    let account_size = 100usize;
    let initial_balance = 1_000_000u64;
    let initial_seq_num = 10u64;
    let accounts = executor.create_accounts(account_size, initial_balance, initial_seq_num);

    // set up the transactions
    let transfer_amount = 1_000;
    let block_count = 5u64;
    let cycle = account_size / (block_count as usize);
    let mut range_left = 0usize;
    let mut execution_time = 0u128;
    for _i in 0..block_count {
        range_left = if range_left + cycle >= account_size {
            account_size - cycle
        } else {
            range_left
        };
        let (txns_info, txns) = create_cyclic_transfers(
            &executor,
            &accounts[range_left..range_left + cycle],
            transfer_amount,
        );

        // execute transaction
        let now = Instant::now();
        let output = executor.execute_block(txns).unwrap();
        execution_time += now.elapsed().as_nanos();
        for txn_output in &output {
            assert_eq!(
                txn_output.status(),
                &TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED))
            );
        }
        assert_eq!(cycle, output.len());
        check_and_apply_transfer_output(&mut executor, &txns_info, &output);
        range_left = (range_left + cycle) % account_size;
    }
    println!("EXECUTION TIME: {}", execution_time);
    print_accounts(&executor, &accounts);
}

#[test]
fn one_to_many_peer_to_peer() {
    // create a FakeExecutor with a genesis from file
    let mut executor = FakeExecutor::from_genesis_file();
    // create and publish accounts with 4_000_000 coins
    let account_size = 100usize;
    let initial_balance = 4_000_000u64;
    let initial_seq_num = 10u64;
    let accounts = executor.create_accounts(account_size, initial_balance, initial_seq_num);

    // set up the transactions
    let transfer_amount = 1_000;
    let block_count = 2u64;
    let cycle = account_size / (block_count as usize);
    let mut range_left = 0usize;
    let mut execution_time = 0u128;
    for _i in 0..block_count {
        range_left = if range_left + cycle >= account_size {
            account_size - cycle
        } else {
            range_left
        };
        let (txns_info, txns) = create_one_to_many_transfers(
            &executor,
            &accounts[range_left..range_left + cycle],
            transfer_amount,
        );

        // execute transaction
        let now = Instant::now();
        let output = executor.execute_block(txns).unwrap();
        execution_time += now.elapsed().as_nanos();
        for txn_output in &output {
            assert_eq!(
                txn_output.status(),
                &TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED))
            );
        }
        assert_eq!(cycle - 1, output.len());
        check_and_apply_transfer_output(&mut executor, &txns_info, &output);
        range_left = (range_left + cycle) % account_size;
    }
    println!("EXECUTION TIME: {}", execution_time);
    print_accounts(&executor, &accounts);
}

#[test]
fn many_to_one_peer_to_peer() {
    // create a FakeExecutor with a genesis from file
    let mut executor = FakeExecutor::from_genesis_file();
    // create and publish accounts with 1_000_000 coins
    let account_size = 100usize;
    let initial_balance = 1_000_000u64;
    let initial_seq_num = 10u64;
    let accounts = executor.create_accounts(account_size, initial_balance, initial_seq_num);

    // set up the transactions
    let transfer_amount = 1_000;
    let block_count = 2u64;
    let cycle = account_size / (block_count as usize);
    let mut range_left = 0usize;
    let mut execution_time = 0u128;
    for _i in 0..block_count {
        range_left = if range_left + cycle >= account_size {
            account_size - cycle
        } else {
            range_left
        };
        let (txns_info, txns) = create_many_to_one_transfers(
            &executor,
            &accounts[range_left..range_left + cycle],
            transfer_amount,
        );

        // execute transaction
        let now = Instant::now();
        let output = executor.execute_block(txns).unwrap();
        execution_time += now.elapsed().as_nanos();
        for txn_output in &output {
            assert_eq!(
                txn_output.status(),
                &TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED))
            );
        }
        assert_eq!(cycle - 1, output.len());
        check_and_apply_transfer_output(&mut executor, &txns_info, &output);
        range_left = (range_left + cycle) % account_size;
    }
    println!("EXECUTION TIME: {}", execution_time);
    print_accounts(&executor, &accounts);
}
