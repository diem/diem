// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::{
    mock_genesis::{db_with_mock_genesis, GENESIS_INFO},
    test_helper::arb_blocks_to_commit,
};
use crypto::hash::CryptoHash;
use proptest::prelude::*;
use rusty_fork::{rusty_fork_id, rusty_fork_test, rusty_fork_test_name};
use std::collections::HashMap;
use types::{contract_event::ContractEvent, ledger_info::LedgerInfo};

fn test_save_blocks_impl(
    input: Vec<(Vec<TransactionToCommit>, LedgerInfoWithSignatures)>,
) -> Result<()> {
    let tmp_dir = tempfile::tempdir()?;
    let db = db_with_mock_genesis(&tmp_dir)?;

    let num_batches = input.len();
    let mut cur_ver = 0;
    for (batch_idx, (txns_to_commit, ledger_info_with_sigs)) in input.iter().enumerate() {
        db.save_transactions(
            &txns_to_commit,
            cur_ver + 1, /* first_version */
            &Some(ledger_info_with_sigs.clone()),
        )?;

        assert_eq!(
            db.ledger_store.get_latest_ledger_info()?,
            *ledger_info_with_sigs
        );
        verify_committed_transactions(
            &db,
            &txns_to_commit,
            cur_ver,
            ledger_info_with_sigs,
            batch_idx + 1 == num_batches, /* is_latest */
        )?;

        cur_ver += txns_to_commit.len() as u64;
    }

    let first_batch = input.first().unwrap().0.clone();
    let first_batch_ledger_info = input.first().unwrap().1.clone();
    let latest_ledger_info = input.last().unwrap().1.clone();
    // Verify an old batch with the latest LedgerInfo.
    verify_committed_transactions(
        &db,
        &first_batch,
        0,
        &latest_ledger_info,
        false, /* is_latest */
    )?;
    // Verify an old batch with an old LedgerInfo.
    verify_committed_transactions(
        &db,
        &first_batch,
        0,
        &first_batch_ledger_info,
        true, /* is_latest */
    )?;

    Ok(())
}

fn test_sync_transactions_impl(
    input: Vec<(Vec<TransactionToCommit>, LedgerInfoWithSignatures)>,
) -> Result<()> {
    let tmp_dir = tempfile::tempdir()?;
    let db = db_with_mock_genesis(&tmp_dir)?;

    let num_batches = input.len();
    let mut cur_ver = 0;
    for (batch_idx, (txns_to_commit, ledger_info_with_sigs)) in input.into_iter().enumerate() {
        // if batch has more than 2 transactions, save them in two batches
        let batch1_len = txns_to_commit.len() / 2;
        if batch1_len > 0 {
            db.save_transactions(
                &txns_to_commit[0..batch1_len],
                cur_ver + 1, /* first_version */
                &None,
            )?;
        }
        db.save_transactions(
            &txns_to_commit[batch1_len..],
            cur_ver + batch1_len as u64 + 1, /* first_version */
            &Some(ledger_info_with_sigs.clone()),
        )?;

        verify_committed_transactions(
            &db,
            &txns_to_commit,
            cur_ver,
            &ledger_info_with_sigs,
            batch_idx + 1 == num_batches, /* is_latest */
        )?;
        cur_ver += txns_to_commit.len() as u64;
    }

    Ok(())
}

fn get_events_by_access_path(
    db: &LibraDB,
    ledger_info: &LedgerInfo,
    access_path: &AccessPath,
    first_seq_num: u64,
    last_seq_num: u64,
    ascending: bool,
    is_latest: bool,
) -> Result<Vec<ContractEvent>> {
    const LIMIT: u64 = 3;

    let mut cursor = if ascending {
        first_seq_num
    } else if is_latest {
        // Test the ability to get the latest.
        u64::max_value()
    } else {
        last_seq_num
    };

    let mut ret = Vec::new();
    loop {
        let (events_with_proof, proof_of_latest_event) = db.get_events_by_event_access_path(
            access_path,
            cursor,
            ascending,
            LIMIT,
            ledger_info.version(),
        )?;

        let num_events = events_with_proof.len() as u64;
        if ascending && num_events < LIMIT || !ascending && cursor == u64::max_value() {
            let proof_of_latest_event = proof_of_latest_event.unwrap();
            proof_of_latest_event.verify(
                ledger_info,
                ledger_info.version(),
                access_path.address,
            )?;
        // TODO: decode and see event seq_num once things get more real.
        } else {
            assert!(proof_of_latest_event.is_none());
        }

        if cursor == u64::max_value() {
            cursor = last_seq_num;
        }
        let expected_seq_nums: Vec<_> = if ascending {
            (cursor..cursor + num_events).collect()
        } else {
            (cursor + 1 - num_events..=cursor).rev().collect()
        };

        let events: Vec<_> = itertools::zip_eq(events_with_proof, expected_seq_nums)
            .map(|(e, seq_num)| {
                e.verify(
                    ledger_info,
                    access_path,
                    seq_num,
                    e.transaction_version,
                    e.event_index,
                )
                .unwrap();
                e.event
            })
            .collect();

        let num_results = events.len() as u64;
        if num_results == 0 {
            break;
        }
        assert_eq!(events.first().unwrap().sequence_number(), cursor);

        if ascending {
            if cursor + num_results > last_seq_num {
                ret.extend(
                    events
                        .into_iter()
                        .take((last_seq_num - cursor + 1) as usize),
                );
                break;
            } else {
                ret.extend(events.into_iter());
                cursor += num_results;
            }
        } else {
            // descending
            if first_seq_num + num_results > cursor {
                ret.extend(
                    events
                        .into_iter()
                        .take((cursor - first_seq_num + 1) as usize),
                );
                break;
            } else {
                ret.extend(events.into_iter());
                cursor -= num_results;
            }
        }
    }

    if !ascending {
        ret.reverse();
    }

    Ok(ret)
}

fn verify_events_by_access_path(
    db: &LibraDB,
    events: &[ContractEvent],
    ledger_info: &LedgerInfo,
    is_latest: bool,
) -> Result<()> {
    let mut events_by_access_path = HashMap::new();
    events.iter().for_each(|e| {
        let list = events_by_access_path
            .entry(e.access_path().clone())
            .or_insert_with(Vec::new);
        list.push(e.clone())
    });

    events_by_access_path
        .into_iter()
        .map(|(access_path, events)| {
            let first_seq = events
                .first()
                .expect("Shouldn't be empty")
                .sequence_number();
            let last_seq = events.last().expect("Shouldn't be empty").sequence_number();

            let traversed = get_events_by_access_path(
                db,
                ledger_info,
                &access_path,
                first_seq,
                last_seq,
                /* ascending = */ true,
                is_latest,
            )?;
            assert_eq!(events, traversed);

            let rev_traversed = get_events_by_access_path(
                db,
                ledger_info,
                &access_path,
                first_seq,
                last_seq,
                /* ascending = */ false,
                is_latest,
            )?;
            assert_eq!(events, rev_traversed);
            Ok(())
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(())
}

fn verify_committed_transactions(
    db: &LibraDB,
    txns_to_commit: &[TransactionToCommit],
    first_version: Version,
    ledger_info_with_sigs: &LedgerInfoWithSignatures,
    is_latest: bool,
) -> Result<()> {
    let ledger_info = ledger_info_with_sigs.ledger_info();
    let ledger_version = ledger_info.version();

    let mut cur_ver = first_version;
    for txn_to_commit in txns_to_commit {
        cur_ver += 1;

        let txn_info = db.ledger_store.get_transaction_info(cur_ver)?;

        // Verify transaction hash.
        assert_eq!(
            txn_info.signed_transaction_hash(),
            txn_to_commit.signed_txn().hash()
        );
        let txn_list_with_proof =
            db.get_transactions(cur_ver, 1, ledger_version, true /* fetch_events */)?;
        txn_list_with_proof.verify(ledger_info, Some(cur_ver))?;

        // Fetch and verify account states.
        for (addr, expected_blob) in txn_to_commit.account_states() {
            let account_state_with_proof =
                db.get_account_state_with_proof(*addr, cur_ver, ledger_version)?;
            assert_eq!(account_state_with_proof.blob, Some(expected_blob.clone()));
            account_state_with_proof.verify(ledger_info, cur_ver, *addr)?;
        }
    }

    // Fetch and verify events.
    // TODO: verify events are saved to correct transaction version.
    verify_events_by_access_path(
        db,
        &txns_to_commit
            .iter()
            .flat_map(|t| t.events().to_vec())
            .collect::<Vec<_>>(),
        ledger_info,
        is_latest,
    )?;

    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn test_save_blocks(input in arb_blocks_to_commit()) {
        test_save_blocks_impl(input).unwrap();
    }

    #[test]
    fn test_sync_transactions(input in arb_blocks_to_commit()) {
        test_sync_transactions_impl(input).unwrap();
    }
}

#[test]
fn test_bootstrap() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let db = LibraDB::new(&tmp_dir);

    let genesis_txn_info = GENESIS_INFO.0.clone();
    let genesis_ledger_info_with_sigs = GENESIS_INFO.1.clone();
    let genesis_txn = GENESIS_INFO.2.clone();

    db.save_transactions(
        &[genesis_txn],
        0, /* first_version */
        &Some(genesis_ledger_info_with_sigs.clone()),
    )
    .unwrap();

    assert_eq!(db.get_latest_version().unwrap(), 0);
    assert_eq!(
        db.ledger_store.get_latest_ledger_info().unwrap(),
        genesis_ledger_info_with_sigs
    );
    assert_eq!(
        db.ledger_store.get_transaction_info(0).unwrap(),
        genesis_txn_info
    );
}

rusty_fork_test! {
#[test]
fn test_committed_txns_counter() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let db = LibraDB::new(&tmp_dir);

    let genesis_ledger_info_with_sigs = GENESIS_INFO.1.clone();
    let genesis_txn = GENESIS_INFO.2.clone();

    db.save_transactions(&[genesis_txn],
                         0 /* first_version */,
                         &Some(genesis_ledger_info_with_sigs.clone()))
        .unwrap();
    assert_eq!(OP_COUNTER.counter("committed_txns").get(), 1);
}
}

#[test]
fn test_bootstrapping_already_bootstrapped_db() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let db = db_with_mock_genesis(&tmp_dir).unwrap();
    let ledger_info = db.ledger_store.get_latest_ledger_info().unwrap();

    let genesis_ledger_info_with_sigs = GENESIS_INFO.1.clone();
    let genesis_txn = GENESIS_INFO.2.clone();
    assert!(db
        .save_transactions(&[genesis_txn], 0, &Some(genesis_ledger_info_with_sigs))
        .is_ok());
    assert_eq!(
        ledger_info,
        db.ledger_store.get_latest_ledger_info().unwrap()
    );
}

#[test]
fn test_get_first_seq_num_and_limit() {
    assert!(get_first_seq_num_and_limit(true, 0, 0).is_err());

    // ascending
    assert_eq!(get_first_seq_num_and_limit(true, 0, 4).unwrap(), (0, 4));
    assert_eq!(get_first_seq_num_and_limit(true, 0, 1).unwrap(), (0, 1));

    // descending
    assert_eq!(get_first_seq_num_and_limit(false, 2, 1).unwrap(), (2, 1));
    assert_eq!(get_first_seq_num_and_limit(false, 2, 2).unwrap(), (1, 2));
    assert_eq!(get_first_seq_num_and_limit(false, 2, 3).unwrap(), (0, 3));
    assert_eq!(get_first_seq_num_and_limit(false, 2, 4).unwrap(), (0, 3));
}
