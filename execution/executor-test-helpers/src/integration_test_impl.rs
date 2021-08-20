// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    bootstrap_genesis, gen_block_id, gen_ledger_info_with_sigs, get_test_signed_transaction,
};
use anyhow::{anyhow, ensure, Result};
use diem_crypto::{ed25519::Ed25519PrivateKey, PrivateKey, Uniform};
use diem_transaction_builder::stdlib::{
    encode_create_parent_vasp_account_script, encode_peer_to_peer_with_metadata_script,
};
use diem_types::{
    account_config::{
        from_currency_code_string, testnet_dd_account_address, treasury_compliance_account_address,
        xus_tag, XUS_NAME,
    },
    account_state::AccountState,
    account_state_blob::AccountStateWithProof,
    event::EventKey,
    transaction::{
        authenticator::AuthenticationKey, Transaction, TransactionListWithProof,
        TransactionPayload, TransactionWithProof, WriteSetPayload,
    },
    trusted_state::{TrustedState, TrustedStateChange},
    waypoint::Waypoint,
};
use diem_vm::DiemVM;
use diemdb::DiemDB;
use executor::Executor;
use executor_types::BlockExecutor;
use rand::SeedableRng;
use std::{convert::TryFrom, sync::Arc};
use storage_interface::{DbReaderWriter, Order};

pub fn test_execution_with_storage_impl() -> Arc<DiemDB> {
    let (genesis, validators) = vm_genesis::test_genesis_change_set_and_validators(Some(1));
    let genesis_txn = Transaction::GenesisTransaction(WriteSetPayload::Direct(genesis));
    let genesis_key = &vm_genesis::GENESIS_KEYPAIR.0;

    let path = diem_temppath::TempPath::new();
    path.create_as_dir().unwrap();
    let (diem_db, db, executor, waypoint) = create_db_and_executor(path.path(), &genesis_txn);

    let parent_block_id = executor.committed_block_id();
    let signer = diem_types::validator_signer::ValidatorSigner::new(
        validators[0].data.address,
        validators[0].key.clone(),
    );

    // This generates accounts that do not overlap with genesis
    let seed = [3u8; 32];
    let mut rng = ::rand::rngs::StdRng::from_seed(seed);

    let privkey1 = Ed25519PrivateKey::generate(&mut rng);
    let pubkey1 = privkey1.public_key();
    let account1_auth_key = AuthenticationKey::ed25519(&pubkey1);
    let account1 = account1_auth_key.derived_address();

    let privkey2 = Ed25519PrivateKey::generate(&mut rng);
    let pubkey2 = privkey2.public_key();
    let account2_auth_key = AuthenticationKey::ed25519(&pubkey2);
    let account2 = account2_auth_key.derived_address();

    let pubkey3 = Ed25519PrivateKey::generate(&mut rng).public_key();
    let account3_auth_key = AuthenticationKey::ed25519(&pubkey3);
    let account3 = account3_auth_key.derived_address();

    let pubkey4 = Ed25519PrivateKey::generate(&mut rng).public_key();
    let account4_auth_key = AuthenticationKey::ed25519(&pubkey4); // non-existent account
    let account4 = account4_auth_key.derived_address();
    let genesis_account = testnet_dd_account_address();
    let tc_account = treasury_compliance_account_address();

    let tx1 = get_test_signed_transaction(
        tc_account,
        /* sequence_number = */ 0,
        genesis_key.clone(),
        genesis_key.public_key(),
        Some(TransactionPayload::Script(
            encode_create_parent_vasp_account_script(
                xus_tag(),
                0,
                account1,
                account1_auth_key.prefix().to_vec(),
                vec![],
                false, /* add all currencies */
            ),
        )),
    );

    let tx2 = get_test_signed_transaction(
        tc_account,
        /* sequence_number = */ 1,
        genesis_key.clone(),
        genesis_key.public_key(),
        Some(TransactionPayload::Script(
            encode_create_parent_vasp_account_script(
                xus_tag(),
                0,
                account2,
                account2_auth_key.prefix().to_vec(),
                vec![],
                false, /* add all currencies */
            ),
        )),
    );

    let tx3 = get_test_signed_transaction(
        tc_account,
        /* sequence_number = */ 2,
        genesis_key.clone(),
        genesis_key.public_key(),
        Some(TransactionPayload::Script(
            encode_create_parent_vasp_account_script(
                xus_tag(),
                0,
                account3,
                account3_auth_key.prefix().to_vec(),
                vec![],
                false, /* add all currencies */
            ),
        )),
    );

    // Create account1 with 2M coins.
    let txn1 = get_test_signed_transaction(
        genesis_account,
        /* sequence_number = */ 0,
        genesis_key.clone(),
        genesis_key.public_key(),
        Some(TransactionPayload::Script(
            encode_peer_to_peer_with_metadata_script(
                xus_tag(),
                account1,
                2_000_000,
                vec![],
                vec![],
            ),
        )),
    );

    // Create account2 with 1.2M coins.
    let txn2 = get_test_signed_transaction(
        genesis_account,
        /* sequence_number = */ 1,
        genesis_key.clone(),
        genesis_key.public_key(),
        Some(TransactionPayload::Script(
            encode_peer_to_peer_with_metadata_script(
                xus_tag(),
                account2,
                1_200_000,
                vec![],
                vec![],
            ),
        )),
    );

    // Create account3 with 1M coins.
    let txn3 = get_test_signed_transaction(
        genesis_account,
        /* sequence_number = */ 2,
        genesis_key.clone(),
        genesis_key.public_key(),
        Some(TransactionPayload::Script(
            encode_peer_to_peer_with_metadata_script(
                xus_tag(),
                account3,
                1_000_000,
                vec![],
                vec![],
            ),
        )),
    );

    // Transfer 20k coins from account1 to account2.
    // balance: <1.98M, 1.22M, 1M
    let txn4 = get_test_signed_transaction(
        account1,
        /* sequence_number = */ 0,
        privkey1.clone(),
        pubkey1.clone(),
        Some(TransactionPayload::Script(
            encode_peer_to_peer_with_metadata_script(xus_tag(), account2, 20_000, vec![], vec![]),
        )),
    );

    // Transfer 10k coins from account2 to account3.
    // balance: <1.98M, <1.21M, 1.01M
    let txn5 = get_test_signed_transaction(
        account2,
        /* sequence_number = */ 0,
        privkey2,
        pubkey2,
        Some(TransactionPayload::Script(
            encode_peer_to_peer_with_metadata_script(xus_tag(), account3, 10_000, vec![], vec![]),
        )),
    );

    // Transfer 70k coins from account1 to account3.
    // balance: <1.91M, <1.21M, 1.08M
    let txn6 = get_test_signed_transaction(
        account1,
        /* sequence_number = */ 1,
        privkey1.clone(),
        pubkey1.clone(),
        Some(TransactionPayload::Script(
            encode_peer_to_peer_with_metadata_script(xus_tag(), account3, 70_000, vec![], vec![]),
        )),
    );

    let block1 = vec![tx1, tx2, tx3, txn1, txn2, txn3, txn4, txn5, txn6];
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
            Some(TransactionPayload::Script(
                encode_peer_to_peer_with_metadata_script(
                    xus_tag(),
                    account3,
                    10_000,
                    vec![],
                    vec![],
                ),
            )),
        ));
    }

    let output1 = executor
        .execute_block((block1_id, block1.clone()), parent_block_id)
        .unwrap();
    let ledger_info_with_sigs = gen_ledger_info_with_sigs(1, &output1, block1_id, vec![&signer]);
    executor
        .commit_blocks(vec![block1_id], ledger_info_with_sigs)
        .unwrap();

    let initial_accumulator = db.reader.get_accumulator_summary(0).unwrap();
    let state_proof = db.reader.get_state_proof(0).unwrap();
    let trusted_state = TrustedState::from_epoch_waypoint(waypoint);
    let trusted_state =
        match trusted_state.verify_and_ratchet(&state_proof, Some(&initial_accumulator)) {
            Ok(TrustedStateChange::Epoch { new_state, .. }) => new_state,
            _ => panic!("unexpected state change"),
        };
    let current_version = state_proof.latest_ledger_info().version();
    assert_eq!(trusted_state.version(), 9);

    let t1 = db
        .reader
        .get_account_transaction(genesis_account, 0, false, current_version)
        .unwrap();
    verify_committed_txn_status(t1.as_ref(), &block1[3]).unwrap();

    let t2 = db
        .reader
        .get_account_transaction(genesis_account, 1, false, current_version)
        .unwrap();
    verify_committed_txn_status(t2.as_ref(), &block1[4]).unwrap();

    let t3 = db
        .reader
        .get_account_transaction(genesis_account, 2, false, current_version)
        .unwrap();
    verify_committed_txn_status(t3.as_ref(), &block1[5]).unwrap();

    let tn = db
        .reader
        .get_account_transaction(genesis_account, 3, false, current_version)
        .unwrap();
    assert!(tn.is_none());

    let t4 = db
        .reader
        .get_account_transaction(account1, 0, true, current_version)
        .unwrap();
    verify_committed_txn_status(t4.as_ref(), &block1[6]).unwrap();
    // We requested the events to come back from this one, so verify that they did
    assert_eq!(t4.unwrap().events.unwrap().len(), 2);

    let t5 = db
        .reader
        .get_account_transaction(account2, 0, false, current_version)
        .unwrap();
    verify_committed_txn_status(t5.as_ref(), &block1[7]).unwrap();

    let t6 = db
        .reader
        .get_account_transaction(account1, 1, true, current_version)
        .unwrap();
    verify_committed_txn_status(t6.as_ref(), &block1[8]).unwrap();

    let account1_state_with_proof = db
        .reader
        .get_account_state_with_proof(account1, current_version, current_version)
        .unwrap();
    verify_account_balance(&account1_state_with_proof, |x| x == 1_910_000).unwrap();

    let account2_state_with_proof = db
        .reader
        .get_account_state_with_proof(account2, current_version, current_version)
        .unwrap();
    verify_account_balance(&account2_state_with_proof, |x| x == 1_210_000).unwrap();

    let account3_state_with_proof = db
        .reader
        .get_account_state_with_proof(account3, current_version, current_version)
        .unwrap();
    verify_account_balance(&account3_state_with_proof, |x| x == 1_080_000).unwrap();

    let transaction_list_with_proof = db
        .reader
        .get_transactions(3, 10, current_version, false)
        .unwrap();
    verify_transactions(&transaction_list_with_proof, &block1[2..]).unwrap();

    let account1_sent_events = db
        .reader
        .get_events(
            &EventKey::new_from_address(&account1, 3),
            0,
            Order::Ascending,
            10,
        )
        .unwrap();
    assert_eq!(account1_sent_events.len(), 2);

    let account2_sent_events = db
        .reader
        .get_events(
            &EventKey::new_from_address(&account2, 3),
            0,
            Order::Ascending,
            10,
        )
        .unwrap();
    assert_eq!(account2_sent_events.len(), 1);

    let account3_sent_events = db
        .reader
        .get_events(
            &EventKey::new_from_address(&account3, 3),
            0,
            Order::Ascending,
            10,
        )
        .unwrap();
    assert_eq!(account3_sent_events.len(), 0);

    let account1_received_events = db
        .reader
        .get_events(
            &EventKey::new_from_address(&account1, 2),
            0,
            Order::Ascending,
            10,
        )
        .unwrap();
    assert_eq!(account1_received_events.len(), 1);

    let account2_received_events = db
        .reader
        .get_events(
            &EventKey::new_from_address(&account2, 2),
            0,
            Order::Ascending,
            10,
        )
        .unwrap();
    assert_eq!(account2_received_events.len(), 2);

    let account3_received_events = db
        .reader
        .get_events(
            &EventKey::new_from_address(&account3, 2),
            0,
            Order::Ascending,
            10,
        )
        .unwrap();
    assert_eq!(account3_received_events.len(), 3);

    let account4_state = db
        .reader
        .get_account_state_with_proof(account4, current_version, current_version)
        .unwrap();
    assert!(account4_state.blob.is_none());

    let account4_transaction = db
        .reader
        .get_account_transaction(account4, 0, true, current_version)
        .unwrap();
    assert!(account4_transaction.is_none());

    let account4_sent_events = db
        .reader
        .get_events(
            &EventKey::new_from_address(&account4, 3),
            0,
            Order::Ascending,
            10,
        )
        .unwrap();
    assert!(account4_sent_events.is_empty());

    // Execute the 2nd block.
    let output2 = executor
        .execute_block((block2_id, block2.clone()), block1_id)
        .unwrap();
    let ledger_info_with_sigs = gen_ledger_info_with_sigs(1, &output2, block2_id, vec![&signer]);
    executor
        .commit_blocks(vec![block2_id], ledger_info_with_sigs)
        .unwrap();

    let state_proof = db.reader.get_state_proof(trusted_state.version()).unwrap();
    let trusted_state_change = trusted_state
        .verify_and_ratchet(&state_proof, None)
        .unwrap();
    assert!(matches!(
        trusted_state_change,
        TrustedStateChange::Version { .. }
    ));
    let current_version = state_proof.latest_ledger_info().version();
    assert_eq!(current_version, 23);

    let t7 = db
        .reader
        .get_account_transaction(account1, 2, false, current_version)
        .unwrap();
    verify_committed_txn_status(t7.as_ref(), &block2[0]).unwrap();

    let t20 = db
        .reader
        .get_account_transaction(account1, 15, false, current_version)
        .unwrap();
    verify_committed_txn_status(t20.as_ref(), &block2[13]).unwrap();

    let account1_state_with_proof = db
        .reader
        .get_account_state_with_proof(account1, current_version, current_version)
        .unwrap();
    verify_account_balance(&account1_state_with_proof, |x| x == 1_770_000).unwrap();

    let account3_state_with_proof = db
        .reader
        .get_account_state_with_proof(account3, current_version, current_version)
        .unwrap();
    verify_account_balance(&account3_state_with_proof, |x| x == 1_220_000).unwrap();

    let transaction_list_with_proof = db
        .reader
        .get_transactions(10, 17, current_version, false)
        .unwrap();
    verify_transactions(&transaction_list_with_proof, &block2[..]).unwrap();

    let account1_sent_events_batch1 = db
        .reader
        .get_events(
            &EventKey::new_from_address(&account1, 3),
            0,
            Order::Ascending,
            10,
        )
        .unwrap();
    assert_eq!(account1_sent_events_batch1.len(), 10);

    let account1_sent_events_batch2 = db
        .reader
        .get_events(
            &EventKey::new_from_address(&account1, 3),
            10,
            Order::Ascending,
            10,
        )
        .unwrap();
    assert_eq!(account1_sent_events_batch2.len(), 6);

    let account3_received_events_batch1 = db
        .reader
        .get_events(
            &EventKey::new_from_address(&account3, 2),
            u64::max_value(),
            Order::Descending,
            10,
        )
        .unwrap();
    assert_eq!(account3_received_events_batch1.len(), 10);
    assert_eq!(account3_received_events_batch1[0].1.sequence_number(), 16);

    let account3_received_events_batch2 = db
        .reader
        .get_events(
            &EventKey::new_from_address(&account3, 2),
            6,
            Order::Descending,
            10,
        )
        .unwrap();
    assert_eq!(account3_received_events_batch2.len(), 7);
    assert_eq!(account3_received_events_batch2[0].1.sequence_number(), 6);

    diem_db
}

pub fn create_db_and_executor<P: AsRef<std::path::Path>>(
    path: P,
    genesis: &Transaction,
) -> (Arc<DiemDB>, DbReaderWriter, Executor<DiemVM>, Waypoint) {
    let (db, dbrw) = DbReaderWriter::wrap(DiemDB::new_for_test(&path));
    let waypoint = bootstrap_genesis::<DiemVM>(&dbrw, genesis).unwrap();
    let executor = Executor::<DiemVM>::new(dbrw.clone());

    (db, dbrw, executor, waypoint)
}

pub fn verify_account_balance<F>(
    account_state_with_proof: &AccountStateWithProof,
    f: F,
) -> Result<()>
where
    F: Fn(u64) -> bool,
{
    let balance = if let Some(blob) = &account_state_with_proof.blob {
        AccountState::try_from(blob)?
            .get_balance_resources()?
            .get(&from_currency_code_string(XUS_NAME).unwrap())
            .map(|b| b.coin())
            .unwrap_or(0)
    } else {
        0
    };
    ensure!(
        f(balance),
        "balance {} doesn't satisfy the condition passed in",
        balance
    );
    Ok(())
}

pub fn verify_transactions(
    txn_list_with_proof: &TransactionListWithProof,
    expected_txns: &[Transaction],
) -> Result<()> {
    let txns = &txn_list_with_proof.transactions;
    ensure!(
        *txns == expected_txns,
        "expected txns {:?} doesn't equal to returned txns {:?}",
        expected_txns,
        txns
    );
    Ok(())
}

pub fn verify_committed_txn_status(
    txn_with_proof: Option<&TransactionWithProof>,
    expected_txn: &Transaction,
) -> Result<()> {
    let txn = &txn_with_proof
        .ok_or_else(|| anyhow!("Transaction is not committed."))?
        .transaction;

    ensure!(
        expected_txn == txn,
        "The two transactions do not match. Expected txn: {:?}, returned txn: {:?}",
        expected_txn,
        txn,
    );

    Ok(())
}
