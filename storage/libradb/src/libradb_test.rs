// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::test_helper::arb_blocks_to_commit;
use libra_crypto::hash::CryptoHash;
use libra_tools::tempdir::TempPath;
use libra_types::{
    account_config::get_account_resource_or_default, contract_event::ContractEvent,
    ledger_info::LedgerInfo,
};
use proptest::prelude::*;
use std::collections::HashMap;

fn verify_epochs(db: &LibraDB, ledger_infos_with_sigs: &[LedgerInfoWithSignatures]) {
    let (_, latest_li, actual_epoch_change_lis, _) =
        db.update_to_latest_ledger(0, Vec::new()).unwrap();
    let expected_epoch_change_lis: Vec<_> = ledger_infos_with_sigs
        .iter()
        .filter(|info| {
            info.ledger_info().next_validator_set().is_some()
                && info.ledger_info().epoch() < latest_li.ledger_info().epoch()
        })
        .cloned()
        .collect();
    assert_eq!(
        actual_epoch_change_lis.ledger_info_with_sigs,
        expected_epoch_change_lis,
    );
}

fn test_save_blocks_impl(input: Vec<(Vec<TransactionToCommit>, LedgerInfoWithSignatures)>) {
    let tmp_dir = TempPath::new();
    let db = LibraDB::new(&tmp_dir);

    let num_batches = input.len();
    let mut cur_ver = 0;
    for (batch_idx, (txns_to_commit, ledger_info_with_sigs)) in input.iter().enumerate() {
        db.save_transactions(
            &txns_to_commit,
            cur_ver, /* first_version */
            &Some(ledger_info_with_sigs.clone()),
        )
        .unwrap();

        assert_eq!(
            db.ledger_store.get_latest_ledger_info().unwrap(),
            *ledger_info_with_sigs
        );
        verify_committed_transactions(
            &db,
            &txns_to_commit,
            cur_ver,
            ledger_info_with_sigs,
            batch_idx + 1 == num_batches, /* is_latest */
        );

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
    );
    // Verify an old batch with an old LedgerInfo.
    verify_committed_transactions(
        &db,
        &first_batch,
        0,
        &first_batch_ledger_info,
        true, /* is_latest */
    );
    let (_, ledger_infos_with_sigs): (Vec<_>, Vec<_>) = input.iter().cloned().unzip();
    verify_epochs(&db, &ledger_infos_with_sigs);
}

fn test_sync_transactions_impl(input: Vec<(Vec<TransactionToCommit>, LedgerInfoWithSignatures)>) {
    let tmp_dir = TempPath::new();
    let db = LibraDB::new(&tmp_dir);

    let num_batches = input.len();
    let mut cur_ver = 0;
    for (batch_idx, (txns_to_commit, ledger_info_with_sigs)) in input.into_iter().enumerate() {
        // if batch has more than 2 transactions, save them in two batches
        let batch1_len = txns_to_commit.len() / 2;
        if batch1_len > 0 {
            db.save_transactions(
                &txns_to_commit[0..batch1_len],
                cur_ver, /* first_version */
                &None,
            )
            .unwrap();
        }
        db.save_transactions(
            &txns_to_commit[batch1_len..],
            cur_ver + batch1_len as u64, /* first_version */
            &Some(ledger_info_with_sigs.clone()),
        )
        .unwrap();

        verify_committed_transactions(
            &db,
            &txns_to_commit,
            cur_ver,
            &ledger_info_with_sigs,
            batch_idx + 1 == num_batches, /* is_latest */
        );
        cur_ver += txns_to_commit.len() as u64;
    }
}

fn get_events_by_query_path(
    db: &LibraDB,
    ledger_info: &LedgerInfo,
    query_path: &AccessPath,
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
        let (events_with_proof, proof_of_latest_event) = db.get_events_by_query_path(
            query_path,
            cursor,
            ascending,
            LIMIT,
            ledger_info.version(),
        )?;

        let account_resource = get_account_resource_or_default(&proof_of_latest_event.blob)?;
        let expected_event_key = account_resource
            .get_event_handle_by_query_path(&query_path.path)?
            .key();

        let num_events = events_with_proof.len() as u64;
        proof_of_latest_event.verify(ledger_info, ledger_info.version(), query_path.address)?;

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
                    &expected_event_key,
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

fn verify_events_by_query_path(
    db: &LibraDB,
    events: Vec<(AccessPath, Vec<ContractEvent>)>,
    ledger_info: &LedgerInfo,
    is_latest: bool,
) {
    events
        .into_iter()
        .map(|(access_path, events)| {
            let first_seq = events
                .first()
                .expect("Shouldn't be empty")
                .sequence_number();
            let last_seq = events.last().expect("Shouldn't be empty").sequence_number();

            let traversed = get_events_by_query_path(
                db,
                ledger_info,
                &access_path,
                first_seq,
                last_seq,
                /* ascending = */ true,
                is_latest,
            )
            .unwrap();
            assert_eq!(events, traversed);

            let rev_traversed = get_events_by_query_path(
                db,
                ledger_info,
                &access_path,
                first_seq,
                last_seq,
                /* ascending = */ false,
                is_latest,
            )
            .unwrap();
            assert_eq!(events, rev_traversed);
            Ok(())
        })
        .collect::<Result<Vec<_>>>()
        .unwrap();
}

fn group_events_by_query_path(
    txns_to_commit: &[TransactionToCommit],
) -> Vec<(AccessPath, Vec<ContractEvent>)> {
    let mut event_key_to_query_path = HashMap::new();
    for txn in txns_to_commit {
        for (address, account_blob) in txn.account_states().iter() {
            let account_btree = account_blob
                .try_into()
                .expect("The stored account blob can't be parsed as BTreeMap");
            let account =
                AccountResource::make_from(&account_btree).expect("AccountResource is not found");
            event_key_to_query_path.insert(
                account.sent_events().key().clone(),
                AccessPath::new_for_sent_event(*address),
            );
            event_key_to_query_path.insert(
                account.received_events().key().clone(),
                AccessPath::new_for_received_event(*address),
            );
        }
    }
    let mut query_path_to_events: HashMap<AccessPath, Vec<ContractEvent>> = HashMap::new();
    for txn in txns_to_commit {
        for event in txn.events() {
            let query_path = event_key_to_query_path
                .get(event.key())
                .expect("Unknown Event Key")
                .clone();
            query_path_to_events
                .entry(query_path)
                .or_default()
                .push(event.clone());
        }
    }
    query_path_to_events.into_iter().collect()
}

fn verify_committed_transactions(
    db: &LibraDB,
    txns_to_commit: &[TransactionToCommit],
    first_version: Version,
    ledger_info_with_sigs: &LedgerInfoWithSignatures,
    is_latest: bool,
) {
    let ledger_info = ledger_info_with_sigs.ledger_info();
    let ledger_version = ledger_info.version();

    let mut cur_ver = first_version;
    for txn_to_commit in txns_to_commit {
        let txn_info = db.ledger_store.get_transaction_info(cur_ver).unwrap();

        // Verify transaction hash.
        assert_eq!(
            txn_info.transaction_hash(),
            txn_to_commit.transaction().hash()
        );

        // Fetch and verify transaction itself.
        let txn = txn_to_commit.transaction().as_signed_user_txn().unwrap();
        let txn_with_proof = db
            .get_transaction_with_proof(cur_ver, ledger_version, true)
            .unwrap();
        txn_with_proof
            .verify_user_txn(ledger_info, cur_ver, txn.sender(), txn.sequence_number())
            .unwrap();

        let txn_with_proof = db
            .get_txn_by_account(txn.sender(), txn.sequence_number(), ledger_version, true)
            .unwrap()
            .expect("Should exist.");
        txn_with_proof
            .verify_user_txn(ledger_info, cur_ver, txn.sender(), txn.sequence_number())
            .unwrap();

        let txn_list_with_proof = db
            .get_transactions(cur_ver, 1, ledger_version, true /* fetch_events */)
            .unwrap();
        txn_list_with_proof
            .verify(ledger_info, Some(cur_ver))
            .unwrap();

        // Fetch and verify account states.
        for (addr, expected_blob) in txn_to_commit.account_states() {
            let account_state_with_proof = db
                .get_account_state_with_proof(*addr, cur_ver, ledger_version)
                .unwrap();
            assert_eq!(account_state_with_proof.blob, Some(expected_blob.clone()));
            account_state_with_proof
                .verify(ledger_info, cur_ver, *addr)
                .unwrap();
        }

        cur_ver += 1;
    }

    // Fetch and verify events.
    // TODO: verify events are saved to correct transaction version.
    verify_events_by_query_path(
        db,
        group_events_by_query_path(txns_to_commit),
        ledger_info,
        is_latest,
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn test_save_blocks(input in arb_blocks_to_commit()) {
        test_save_blocks_impl(input);
    }

    #[test]
    fn test_sync_transactions(input in arb_blocks_to_commit()) {
        test_sync_transactions_impl(input);
    }
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

#[test]
fn test_too_many_requested() {
    let tmp_dir = TempPath::new();
    let db = LibraDB::new(&tmp_dir);

    assert!(db
        .update_to_latest_ledger(
            0,
            vec![
                RequestItem::GetTransactions {
                    start_version: 0,
                    limit: 100,
                    fetch_events: false,
                };
                101
            ]
        )
        .is_err());
    assert!(db.get_transactions(0, 1001 /* limit */, 0, true).is_err());
    assert!(db
        .get_events_by_query_path(
            &AccessPath::new_for_sent_event(AccountAddress::random()),
            0,
            true,
            1001, /* limit */
            0
        )
        .is_err());
}
