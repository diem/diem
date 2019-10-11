// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use config::config::NodeConfig;
use config_builder::util::get_test_config;
use crypto::{ed25519::*, hash::GENESIS_BLOCK_ID, test_utils::TEST_SEED, HashValue};
use executor::Executor;
use failure::prelude::*;
use futures::executor::block_on;
use grpc_helpers::ServerHandle;
use grpcio::EnvBuilder;
use libra_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_config::{association_address, get_account_resource_or_default},
    account_state_blob::AccountStateWithProof,
    crypto_proxies::ValidatorVerifier,
    get_with_proof::{verify_update_to_latest_ledger_response, RequestItem},
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    test_helpers::transaction_test_helpers::get_test_signed_txn,
    transaction::{
        Script, SignedTransaction, SignedTransactionWithProof, TransactionListWithProof,
    },
};
use rand::SeedableRng;
use std::{collections::BTreeMap, sync::Arc};
use storage_client::{StorageRead, StorageReadServiceClient, StorageWriteServiceClient};
use storage_service::start_storage_service;
use transaction_builder::{encode_create_account_script, encode_transfer_script};
use vm_runtime::MoveVM;

fn gen_block_id(index: u8) -> HashValue {
    HashValue::new([index; HashValue::LENGTH])
}

fn gen_ledger_info_with_sigs(
    version: u64,
    root_hash: HashValue,
    commit_block_id: HashValue,
) -> LedgerInfoWithSignatures<Ed25519Signature> {
    let ledger_info = LedgerInfo::new(
        version,
        root_hash,
        /* consensus_data_hash = */ HashValue::zero(),
        commit_block_id,
        0,
        /* timestamp = */ 0,
        None,
    );
    LedgerInfoWithSignatures::new(ledger_info, BTreeMap::new())
}

fn create_storage_service_and_executor(config: &NodeConfig) -> (ServerHandle, Executor<MoveVM>) {
    let storage_server_handle = start_storage_service(config);

    let client_env = Arc::new(EnvBuilder::new().build());
    let storage_read_client = Arc::new(StorageReadServiceClient::new(
        Arc::clone(&client_env),
        &config.storage.address,
        config.storage.port,
    ));
    let storage_write_client = Arc::new(StorageWriteServiceClient::new(
        Arc::clone(&client_env),
        &config.storage.address,
        config.storage.port,
        None,
    ));

    let executor = Executor::new(
        Arc::clone(&storage_read_client) as Arc<dyn StorageRead>,
        storage_write_client,
        config,
    );

    (storage_server_handle, executor)
}

fn get_test_signed_transaction(
    sender: AccountAddress,
    sequence_number: u64,
    private_key: Ed25519PrivateKey,
    public_key: Ed25519PublicKey,
    program: Option<Script>,
) -> SignedTransaction {
    get_test_signed_txn(sender, sequence_number, private_key, public_key, program)
}

#[test]
fn test_execution_with_storage() {
    let (config, genesis_keypair) = get_test_config();
    let (_storage_server_handle, executor) = create_storage_service_and_executor(&config);

    let storage_read_client = Arc::new(StorageReadServiceClient::new(
        Arc::new(EnvBuilder::new().build()),
        &config.storage.address,
        config.storage.port,
    ));

    let seed = [1u8; 32];
    // TEST_SEED is also used to generate a random validator set in get_test_config. Each account
    // in this random validator set gets created in genesis. If one of {account1, account2,
    // account3} already exists in genesis, the code below will fail.
    assert!(seed != TEST_SEED);
    let mut rng = ::rand::rngs::StdRng::from_seed(seed);
    let (privkey1, pubkey1) = compat::generate_keypair(&mut rng);
    let account1 = AccountAddress::from_public_key(&pubkey1);
    let (privkey2, pubkey2) = compat::generate_keypair(&mut rng);
    let account2 = AccountAddress::from_public_key(&pubkey2);
    let (_privkey3, pubkey3) = compat::generate_keypair(&mut rng);
    let account3 = AccountAddress::from_public_key(&pubkey3);
    let genesis_account = association_address();

    // Create account1 with 2M coins.
    let txn1 = get_test_signed_transaction(
        genesis_account,
        /* sequence_number = */ 1,
        genesis_keypair.private_key.clone(),
        genesis_keypair.public_key.clone(),
        Some(encode_create_account_script(&account1, 2_000_000)),
    );

    // Create account2 with 200k coins.
    let txn2 = get_test_signed_transaction(
        genesis_account,
        /* sequence_number = */ 2,
        genesis_keypair.private_key.clone(),
        genesis_keypair.public_key.clone(),
        Some(encode_create_account_script(&account2, 200_000)),
    );

    // Create account3 with 100k coins.
    let txn3 = get_test_signed_transaction(
        genesis_account,
        /* sequence_number = */ 3,
        genesis_keypair.private_key.clone(),
        genesis_keypair.public_key.clone(),
        Some(encode_create_account_script(&account3, 100_000)),
    );

    // Transfer 20k coins from account1 to account2.
    // balance: <1.98M, 220k, 100k
    let txn4 = get_test_signed_transaction(
        account1,
        /* sequence_number = */ 0,
        privkey1.clone(),
        pubkey1.clone(),
        Some(encode_transfer_script(&account2, 20_000)),
    );

    // Transfer 10k coins from account2 to account3.
    // balance: <1.98M, <210k, 110k
    let txn5 = get_test_signed_transaction(
        account2,
        /* sequence_number = */ 0,
        privkey2.clone(),
        pubkey2.clone(),
        Some(encode_transfer_script(&account3, 10_000)),
    );

    // Transfer 70k coins from account1 to account3.
    // balance: <1.91M, <210k, 180k
    let txn6 = get_test_signed_transaction(
        account1,
        /* sequence_number = */ 1,
        privkey1.clone(),
        pubkey1.clone(),
        Some(encode_transfer_script(&account3, 70_000)),
    );

    let block1 = vec![txn1, txn2, txn3, txn4, txn5, txn6];
    let block1_id = gen_block_id(1);

    let mut block2 = vec![];
    let block2_id = gen_block_id(2);

    // Create 14 txns transferring 10k from account1 to account3 each.
    for i in 2..=15 {
        block2.push(get_test_signed_transaction(
            account1,
            /* sequence_number = */ i,
            privkey1.clone(),
            pubkey1.clone(),
            Some(encode_transfer_script(&account3, 10_000)),
        ));
    }

    let state_compute_result =
        block_on(executor.execute_block(block1.clone(), *GENESIS_BLOCK_ID, block1_id))
            .unwrap()
            .unwrap();
    let ledger_info_with_sigs =
        gen_ledger_info_with_sigs(6, state_compute_result.root_hash(), block1_id);
    block_on(executor.commit_block(ledger_info_with_sigs))
        .unwrap()
        .unwrap();

    let request_items = vec![
        RequestItem::GetAccountTransactionBySequenceNumber {
            account: genesis_account,
            sequence_number: 1,
            fetch_events: false,
        },
        RequestItem::GetAccountTransactionBySequenceNumber {
            account: genesis_account,
            sequence_number: 2,
            fetch_events: false,
        },
        RequestItem::GetAccountTransactionBySequenceNumber {
            account: genesis_account,
            sequence_number: 3,
            fetch_events: false,
        },
        RequestItem::GetAccountTransactionBySequenceNumber {
            account: genesis_account,
            sequence_number: 4,
            fetch_events: false,
        },
        RequestItem::GetAccountTransactionBySequenceNumber {
            account: account1,
            sequence_number: 0,
            fetch_events: true,
        },
        RequestItem::GetAccountTransactionBySequenceNumber {
            account: account2,
            sequence_number: 0,
            fetch_events: false,
        },
        RequestItem::GetAccountTransactionBySequenceNumber {
            account: account1,
            sequence_number: 1,
            fetch_events: false,
        },
        RequestItem::GetAccountState { address: account1 },
        RequestItem::GetAccountState { address: account2 },
        RequestItem::GetAccountState { address: account3 },
        RequestItem::GetTransactions {
            start_version: 3,
            limit: 10,
            fetch_events: false,
        },
        RequestItem::GetEventsByEventAccessPath {
            access_path: AccessPath::new_for_sent_event(account1),
            start_event_seq_num: 0,
            ascending: true,
            limit: 10,
        },
        RequestItem::GetEventsByEventAccessPath {
            access_path: AccessPath::new_for_sent_event(account2),
            start_event_seq_num: 0,
            ascending: true,
            limit: 10,
        },
        RequestItem::GetEventsByEventAccessPath {
            access_path: AccessPath::new_for_sent_event(account3),
            start_event_seq_num: 0,
            ascending: true,
            limit: 10,
        },
        RequestItem::GetEventsByEventAccessPath {
            access_path: AccessPath::new_for_received_event(account1),
            start_event_seq_num: u64::max_value(),
            ascending: false,
            limit: 10,
        },
        RequestItem::GetEventsByEventAccessPath {
            access_path: AccessPath::new_for_received_event(account2),
            start_event_seq_num: u64::max_value(),
            ascending: false,
            limit: 10,
        },
        RequestItem::GetEventsByEventAccessPath {
            access_path: AccessPath::new_for_received_event(account3),
            start_event_seq_num: u64::max_value(),
            ascending: false,
            limit: 10,
        },
    ];

    let (
        mut response_items,
        ledger_info_with_sigs,
        _validator_change_events,
        _ledger_consistency_proof,
    ) = storage_read_client
        .update_to_latest_ledger(/* client_known_version = */ 0, request_items.clone())
        .unwrap();
    verify_update_to_latest_ledger_response(
        Arc::new(ValidatorVerifier::new(BTreeMap::new())),
        0,
        &request_items,
        &response_items,
        &ledger_info_with_sigs,
    )
    .unwrap();
    response_items.reverse();

    let (t1, _) = response_items
        .pop()
        .unwrap()
        .into_get_account_txn_by_seq_num_response()
        .unwrap();
    verify_committed_txn_status(t1.as_ref(), &block1[0]).unwrap();

    let (t2, _) = response_items
        .pop()
        .unwrap()
        .into_get_account_txn_by_seq_num_response()
        .unwrap();
    verify_committed_txn_status(t2.as_ref(), &block1[1]).unwrap();

    let (t3, _) = response_items
        .pop()
        .unwrap()
        .into_get_account_txn_by_seq_num_response()
        .unwrap();
    verify_committed_txn_status(t3.as_ref(), &block1[2]).unwrap();

    let (tn, pn) = response_items
        .pop()
        .unwrap()
        .into_get_account_txn_by_seq_num_response()
        .unwrap();
    verify_uncommitted_txn_status(
        tn.as_ref(),
        pn.as_ref(),
        /* next_seq_num_of_this_account = */ 4,
    )
    .unwrap();

    let (t4, _) = response_items
        .pop()
        .unwrap()
        .into_get_account_txn_by_seq_num_response()
        .unwrap();
    verify_committed_txn_status(t4.as_ref(), &block1[3]).unwrap();
    // We requested the events to come back from this one, so verify that they did
    assert_eq!(t4.unwrap().events.unwrap().len(), 2);

    let (t5, _) = response_items
        .pop()
        .unwrap()
        .into_get_account_txn_by_seq_num_response()
        .unwrap();
    verify_committed_txn_status(t5.as_ref(), &block1[4]).unwrap();

    let (t6, _) = response_items
        .pop()
        .unwrap()
        .into_get_account_txn_by_seq_num_response()
        .unwrap();
    verify_committed_txn_status(t6.as_ref(), &block1[5]).unwrap();

    let account1_state_with_proof = response_items
        .pop()
        .unwrap()
        .into_get_account_state_response()
        .unwrap();
    verify_account_balance(&account1_state_with_proof, |x| x < 1_910_000).unwrap();

    let account2_state_with_proof = response_items
        .pop()
        .unwrap()
        .into_get_account_state_response()
        .unwrap();
    verify_account_balance(&account2_state_with_proof, |x| x < 210_000).unwrap();

    let account3_state_with_proof = response_items
        .pop()
        .unwrap()
        .into_get_account_state_response()
        .unwrap();
    verify_account_balance(&account3_state_with_proof, |x| x == 180_000).unwrap();

    let transaction_list_with_proof = response_items
        .pop()
        .unwrap()
        .into_get_transactions_response()
        .unwrap();
    verify_transactions(&transaction_list_with_proof, &block1[2..]).unwrap();

    let (account1_sent_events, _) = response_items
        .pop()
        .unwrap()
        .into_get_events_by_access_path_response()
        .unwrap();
    assert_eq!(account1_sent_events.len(), 2);

    let (account2_sent_events, _) = response_items
        .pop()
        .unwrap()
        .into_get_events_by_access_path_response()
        .unwrap();
    assert_eq!(account2_sent_events.len(), 1);

    let (account3_sent_events, _) = response_items
        .pop()
        .unwrap()
        .into_get_events_by_access_path_response()
        .unwrap();
    assert_eq!(account3_sent_events.len(), 0);

    let (account1_received_events, _) = response_items
        .pop()
        .unwrap()
        .into_get_events_by_access_path_response()
        .unwrap();
    assert_eq!(account1_received_events.len(), 1);

    let (account2_received_events, _) = response_items
        .pop()
        .unwrap()
        .into_get_events_by_access_path_response()
        .unwrap();
    assert_eq!(account2_received_events.len(), 2);

    let (account3_received_events, _) = response_items
        .pop()
        .unwrap()
        .into_get_events_by_access_path_response()
        .unwrap();
    assert_eq!(account3_received_events.len(), 3);

    // Execution the 2nd block.
    let state_compute_result =
        block_on(executor.execute_block(block2.clone(), block1_id, block2_id))
            .unwrap()
            .unwrap();
    let ledger_info_with_sigs =
        gen_ledger_info_with_sigs(20, state_compute_result.root_hash(), block2_id);
    block_on(executor.commit_block(ledger_info_with_sigs))
        .unwrap()
        .unwrap();

    let request_items = vec![
        RequestItem::GetAccountTransactionBySequenceNumber {
            account: account1,
            sequence_number: 2,
            fetch_events: false,
        },
        RequestItem::GetAccountTransactionBySequenceNumber {
            account: account1,
            sequence_number: 15,
            fetch_events: false,
        },
        RequestItem::GetAccountState { address: account1 },
        RequestItem::GetAccountState { address: account3 },
        RequestItem::GetTransactions {
            start_version: 7,
            limit: 14,
            fetch_events: false,
        },
        RequestItem::GetEventsByEventAccessPath {
            access_path: AccessPath::new_for_sent_event(account1),
            start_event_seq_num: 0,
            ascending: true,
            limit: 10,
        },
        RequestItem::GetEventsByEventAccessPath {
            access_path: AccessPath::new_for_sent_event(account1),
            start_event_seq_num: 10,
            ascending: true,
            limit: 10,
        },
        RequestItem::GetEventsByEventAccessPath {
            access_path: AccessPath::new_for_received_event(account3),
            start_event_seq_num: u64::max_value(),
            ascending: false,
            limit: 10,
        },
        RequestItem::GetEventsByEventAccessPath {
            access_path: AccessPath::new_for_received_event(account3),
            start_event_seq_num: 6,
            ascending: false,
            limit: 10,
        },
    ];
    let (
        mut response_items,
        ledger_info_with_sigs,
        _validator_change_events,
        _ledger_consistency_proof,
    ) = storage_read_client
        .update_to_latest_ledger(/* client_known_version = */ 0, request_items.clone())
        .unwrap();
    verify_update_to_latest_ledger_response(
        Arc::new(ValidatorVerifier::new(BTreeMap::new())),
        0,
        &request_items,
        &response_items,
        &ledger_info_with_sigs,
    )
    .unwrap();
    response_items.reverse();

    let (t7, _) = response_items
        .pop()
        .unwrap()
        .into_get_account_txn_by_seq_num_response()
        .unwrap();
    verify_committed_txn_status(t7.as_ref(), &block2[0]).unwrap();

    let (t20, _) = response_items
        .pop()
        .unwrap()
        .into_get_account_txn_by_seq_num_response()
        .unwrap();
    verify_committed_txn_status(t20.as_ref(), &block2[13]).unwrap();

    let account1_state_with_proof = response_items
        .pop()
        .unwrap()
        .into_get_account_state_response()
        .unwrap();
    verify_account_balance(&account1_state_with_proof, |x| x < 1_770_000).unwrap();

    let account3_state_with_proof = response_items
        .pop()
        .unwrap()
        .into_get_account_state_response()
        .unwrap();
    verify_account_balance(&account3_state_with_proof, |x| x == 320_000).unwrap();

    let transaction_list_with_proof = response_items
        .pop()
        .unwrap()
        .into_get_transactions_response()
        .unwrap();
    verify_transactions(&transaction_list_with_proof, &block2[..]).unwrap();

    let (account1_sent_events_batch1, _) = response_items
        .pop()
        .unwrap()
        .into_get_events_by_access_path_response()
        .unwrap();
    assert_eq!(account1_sent_events_batch1.len(), 10);

    let (account1_sent_events_batch2, _) = response_items
        .pop()
        .unwrap()
        .into_get_events_by_access_path_response()
        .unwrap();
    assert_eq!(account1_sent_events_batch2.len(), 6);

    let (account3_received_events_batch1, _) = response_items
        .pop()
        .unwrap()
        .into_get_events_by_access_path_response()
        .unwrap();
    assert_eq!(account3_received_events_batch1.len(), 10);
    assert_eq!(
        account3_received_events_batch1[0].event.sequence_number(),
        16
    );

    let (account3_received_events_batch2, _) = response_items
        .pop()
        .unwrap()
        .into_get_events_by_access_path_response()
        .unwrap();
    assert_eq!(account3_received_events_batch2.len(), 7);
    assert_eq!(
        account3_received_events_batch2[0].event.sequence_number(),
        6
    );
}

fn verify_account_balance<F>(account_state_with_proof: &AccountStateWithProof, f: F) -> Result<()>
where
    F: Fn(u64) -> bool,
{
    let balance = get_account_resource_or_default(&account_state_with_proof.blob)?.balance();
    ensure!(
        f(balance),
        "balance {} doesn't satisfy the condition passed in",
        balance
    );
    Ok(())
}

fn verify_transactions(
    txn_list_with_proof: &TransactionListWithProof,
    expected_txns: &[SignedTransaction],
) -> Result<()> {
    let txns = txn_list_with_proof
        .transaction_and_infos
        .iter()
        .map(|(txn, _)| txn)
        .cloned()
        .collect::<Vec<_>>();
    ensure!(
        expected_txns == &txns[..],
        "expected txns {:?} doesn't equal to returned txns {:?}",
        expected_txns,
        txns
    );
    Ok(())
}

fn verify_committed_txn_status(
    signed_txn_with_proof: Option<&SignedTransactionWithProof>,
    expected_txn: &SignedTransaction,
) -> Result<()> {
    let signed_txn = &signed_txn_with_proof
        .ok_or_else(|| format_err!("Transaction is not committed."))?
        .signed_transaction;

    ensure!(
        expected_txn == signed_txn,
        "The two transactions do not match. Expected txn: {:?}, returned txn: {:?}",
        expected_txn,
        signed_txn,
    );

    Ok(())
}

fn verify_uncommitted_txn_status(
    signed_txn_with_proof: Option<&SignedTransactionWithProof>,
    proof_of_current_sequence_number: Option<&AccountStateWithProof>,
    expected_seq_num: u64,
) -> Result<()> {
    ensure!(
        signed_txn_with_proof.is_none(),
        "Transaction is unexpectedly committed."
    );

    let proof_of_current_sequence_number = proof_of_current_sequence_number.ok_or_else(|| {
        format_err!(
        "proof_of_current_sequence_number should be provided when transaction is not committed."
    )
    })?;
    let seq_num_in_account =
        get_account_resource_or_default(&proof_of_current_sequence_number.blob)?.sequence_number();

    ensure!(
        expected_seq_num == seq_num_in_account,
        "expected_seq_num {} doesn't match that in account state \
         in TransactionStatus::Uncommmitted {}",
        expected_seq_num,
        seq_num_in_account,
    );
    Ok(())
}
