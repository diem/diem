// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    core_mempool::{CoreMempool, TimelineState, TtlCache},
    tests::common::{
        add_signed_txn, add_txn, add_txns_to_mempool, exist_in_metrics_cache, setup_mempool,
        TestTransaction,
    },
};
use libra_config::config::NodeConfig;
use libra_types::transaction::SignedTransaction;
use std::{
    collections::HashSet,
    time::{Duration, SystemTime},
};

#[test]
fn test_transaction_ordering() {
    let (mut mempool, mut consensus) = setup_mempool();

    // default ordering: gas price
    let mut transactions = add_txns_to_mempool(
        &mut mempool,
        vec![TestTransaction::new(0, 0, 3), TestTransaction::new(1, 0, 5)],
    );
    assert_eq!(
        consensus.get_block(&mut mempool, 1),
        vec!(transactions[1].clone())
    );
    assert_eq!(
        consensus.get_block(&mut mempool, 1),
        vec!(transactions[0].clone())
    );

    // second level ordering: expiration time
    let (mut mempool, mut consensus) = setup_mempool();
    transactions = add_txns_to_mempool(
        &mut mempool,
        vec![TestTransaction::new(0, 0, 1), TestTransaction::new(1, 0, 1)],
    );
    for transaction in &transactions {
        assert_eq!(
            consensus.get_block(&mut mempool, 1),
            vec![transaction.clone()]
        );
    }

    // last level: for same account it should be by sequence number
    let (mut mempool, mut consensus) = setup_mempool();
    transactions = add_txns_to_mempool(
        &mut mempool,
        vec![
            TestTransaction::new(1, 0, 7),
            TestTransaction::new(1, 1, 5),
            TestTransaction::new(1, 2, 1),
            TestTransaction::new(1, 3, 6),
        ],
    );
    for transaction in &transactions {
        assert_eq!(
            consensus.get_block(&mut mempool, 1),
            vec![transaction.clone()]
        );
    }
}

#[test]
fn test_ordering_of_governance_transactions() {
    let (mut pool, mut consensus) = setup_mempool();

    let txn1 = TestTransaction::new(0, 0, 100);
    let txn2 = TestTransaction::new(1, 0, 200);
    let mut gov_txn1 = TestTransaction::new(2, 0, 2);
    let mut gov_txn2 = TestTransaction::new(3, 0, 1);
    gov_txn1.is_governance_txn = true;
    gov_txn2.is_governance_txn = true;

    let _ = add_txns_to_mempool(
        &mut pool,
        vec![
            txn1.clone(),
            txn2.clone(),
            gov_txn1.clone(),
            gov_txn2.clone(),
        ],
    );

    assert_eq!(
        consensus.get_block(&mut pool, 1),
        vec!(gov_txn1.make_signed_transaction())
    );
    assert_eq!(
        consensus.get_block(&mut pool, 1),
        vec!(gov_txn2.make_signed_transaction())
    );
    assert_eq!(
        consensus.get_block(&mut pool, 1),
        vec!(txn2.make_signed_transaction())
    );
    assert_eq!(
        consensus.get_block(&mut pool, 1),
        vec!(txn1.make_signed_transaction())
    );
}

#[test]
fn test_metric_cache_add_local_txns() {
    let (mut mempool, _) = setup_mempool();
    let txns = add_txns_to_mempool(
        &mut mempool,
        vec![TestTransaction::new(0, 0, 1), TestTransaction::new(1, 0, 2)],
    );
    // Check txns' timestamps exist in metrics_cache.
    assert_eq!(exist_in_metrics_cache(&mempool, &txns[0]), true);
    assert_eq!(exist_in_metrics_cache(&mempool, &txns[1]), true);
}

#[test]
fn test_update_transaction_in_mempool() {
    let (mut mempool, mut consensus) = setup_mempool();
    let txns = add_txns_to_mempool(
        &mut mempool,
        vec![TestTransaction::new(0, 0, 1), TestTransaction::new(1, 0, 2)],
    );
    let fixed_txns = add_txns_to_mempool(&mut mempool, vec![TestTransaction::new(0, 0, 5)]);

    // check that first transactions pops up first
    assert_eq!(
        consensus.get_block(&mut mempool, 1),
        vec![fixed_txns[0].clone()]
    );
    assert_eq!(consensus.get_block(&mut mempool, 1), vec![txns[1].clone()]);
}

#[test]
fn test_update_invalid_transaction_in_mempool() {
    let (mut mempool, mut consensus) = setup_mempool();
    let txns = add_txns_to_mempool(
        &mut mempool,
        vec![TestTransaction::new(0, 0, 1), TestTransaction::new(1, 0, 2)],
    );
    let updated_txn = TestTransaction::make_signed_transaction_with_max_gas_amount(
        &TestTransaction::new(0, 0, 5),
        200,
    );
    let _added_tnx = add_signed_txn(&mut mempool, updated_txn);

    // since both gas price and mas gas amount were updated, the ordering should not have changed.
    // the second transaction with gas price 2 should come first
    assert_eq!(consensus.get_block(&mut mempool, 1), vec![txns[1].clone()]);
    let next_tnx = consensus.get_block(&mut mempool, 1);
    assert_eq!(next_tnx, vec![txns[0].clone()]);
    assert_eq!(next_tnx[0].gas_unit_price(), 1);
}

#[test]
fn test_remove_transaction() {
    let (mut pool, mut consensus) = setup_mempool();

    // test normal flow
    let txns = add_txns_to_mempool(
        &mut pool,
        vec![TestTransaction::new(0, 0, 1), TestTransaction::new(0, 1, 2)],
    );
    for txn in txns {
        pool.remove_transaction(&txn.sender(), txn.sequence_number(), false);
    }
    let new_txns = add_txns_to_mempool(
        &mut pool,
        vec![TestTransaction::new(1, 0, 3), TestTransaction::new(1, 1, 4)],
    );
    // should return only txns from new_txns
    assert_eq!(consensus.get_block(&mut pool, 1), vec!(new_txns[0].clone()));
    assert_eq!(consensus.get_block(&mut pool, 1), vec!(new_txns[1].clone()));
}

#[test]
fn test_system_ttl() {
    // created mempool with system_transaction_timeout = 0
    // All transactions are supposed to be evicted on next gc run
    let mut config = NodeConfig::random();
    config.mempool.system_transaction_timeout_secs = 0;
    let mut mempool = CoreMempool::new(&config);

    add_txn(&mut mempool, TestTransaction::new(0, 0, 10)).unwrap();

    // reset system ttl timeout
    mempool.system_transaction_timeout = Duration::from_secs(10);
    // add new transaction. Should be valid for 10 seconds
    let transaction = TestTransaction::new(1, 0, 1);
    add_txn(&mut mempool, transaction.clone()).unwrap();

    // gc routine should clear transaction from first insert but keep last one
    mempool.gc();
    let batch = mempool.get_block(1, HashSet::new());
    assert_eq!(vec![transaction.make_signed_transaction()], batch);
}

#[test]
fn test_commit_callback() {
    // consensus commit callback should unlock txns in parking lot
    let mut pool = setup_mempool().0;
    // insert transaction with sequence number 6 to pool(while last known executed transaction is 0)
    let txns = add_txns_to_mempool(&mut pool, vec![TestTransaction::new(1, 6, 1)]);

    // check that pool is empty
    assert!(pool.get_block(1, HashSet::new()).is_empty());
    // transaction 5 got back from consensus
    pool.remove_transaction(&TestTransaction::get_address(1), 5, false);
    // verify that we can execute transaction 6
    assert_eq!(pool.get_block(1, HashSet::new())[0], txns[0]);
}

#[test]
fn test_sequence_number_cache() {
    // checks potential race where StateDB is lagging
    let mut pool = setup_mempool().0;
    // callback from consensus should set current sequence number for account
    pool.remove_transaction(&TestTransaction::get_address(1), 5, false);

    // try to add transaction with sequence number 6 to pool(while last known executed transaction
    // for AC is 0)
    add_txns_to_mempool(&mut pool, vec![TestTransaction::new(1, 6, 1)]);
    // verify that we can execute transaction 6
    assert_eq!(pool.get_block(1, HashSet::new()).len(), 1);
}

#[test]
fn test_reset_sequence_number_on_failure() {
    let mut pool = setup_mempool().0;
    // add two transactions for account
    add_txns_to_mempool(
        &mut pool,
        vec![TestTransaction::new(1, 0, 1), TestTransaction::new(1, 1, 1)],
    );

    // notify mempool about failure in arbitrary order
    pool.remove_transaction(&TestTransaction::get_address(1), 0, true);
    pool.remove_transaction(&TestTransaction::get_address(1), 1, true);

    // verify that new transaction for this account can be added
    assert!(add_txn(&mut pool, TestTransaction::new(1, 0, 1)).is_ok());
}

#[test]
fn test_timeline() {
    let mut pool = setup_mempool().0;
    add_txns_to_mempool(
        &mut pool,
        vec![
            TestTransaction::new(1, 0, 1),
            TestTransaction::new(1, 1, 1),
            TestTransaction::new(1, 3, 1),
            TestTransaction::new(1, 5, 1),
        ],
    );
    let view = |txns: Vec<SignedTransaction>| -> Vec<u64> {
        txns.iter()
            .map(SignedTransaction::sequence_number)
            .collect()
    };
    let (timeline, _) = pool.read_timeline(0, 10);
    assert_eq!(view(timeline), vec![0, 1]);

    // add txn 2 to unblock txn3
    add_txns_to_mempool(&mut pool, vec![TestTransaction::new(1, 2, 1)]);
    let (timeline, _) = pool.read_timeline(0, 10);
    assert_eq!(view(timeline), vec![0, 1, 2, 3]);

    // try different start read position
    let (timeline, _) = pool.read_timeline(2, 10);
    assert_eq!(view(timeline), vec![2, 3]);

    // simulate callback from consensus to unblock txn 5
    pool.remove_transaction(&TestTransaction::get_address(1), 4, false);
    let (timeline, _) = pool.read_timeline(0, 10);
    assert_eq!(view(timeline), vec![5]);
}

#[test]
fn test_capacity() {
    let mut config = NodeConfig::random();
    config.mempool.capacity = 1;
    config.mempool.system_transaction_timeout_secs = 0;
    let mut pool = CoreMempool::new(&config);

    // error on exceeding limit
    add_txn(&mut pool, TestTransaction::new(1, 0, 1)).unwrap();
    assert!(add_txn(&mut pool, TestTransaction::new(1, 1, 1)).is_err());

    // commit transaction and free space
    pool.remove_transaction(&TestTransaction::get_address(1), 0, false);
    assert!(add_txn(&mut pool, TestTransaction::new(1, 1, 1)).is_ok());

    // fill it up and check that GC routine will clear space
    assert!(add_txn(&mut pool, TestTransaction::new(1, 2, 1)).is_err());
    pool.gc();
    assert!(add_txn(&mut pool, TestTransaction::new(1, 2, 1)).is_ok());
}

#[test]
fn test_parking_lot_eviction() {
    let mut config = NodeConfig::random();
    config.mempool.capacity = 5;
    let mut pool = CoreMempool::new(&config);
    // add transactions with following sequence numbers to Mempool
    for seq in &[0, 1, 2, 9, 10] {
        add_txn(&mut pool, TestTransaction::new(1, *seq, 1)).unwrap();
    }
    // Mempool is full. Insert few txns for other account
    for seq in &[0, 1] {
        add_txn(&mut pool, TestTransaction::new(0, *seq, 1)).unwrap();
    }
    // Make sure that we have correct txns in Mempool
    let mut txns: Vec<_> = pool
        .get_block(5, HashSet::new())
        .iter()
        .map(SignedTransaction::sequence_number)
        .collect();
    txns.sort();
    assert_eq!(txns, vec![0, 0, 1, 1, 2]);

    // Make sure we can't insert any new transactions, cause parking lot supposed to be empty by now
    assert!(add_txn(&mut pool, TestTransaction::new(0, 2, 1)).is_err());
}

#[test]
fn test_gc_ready_transaction() {
    let mut pool = setup_mempool().0;
    add_txn(&mut pool, TestTransaction::new(1, 0, 1)).unwrap();

    // insert in the middle transaction that's going to be expired
    let txn = TestTransaction::new(1, 1, 1)
        .make_signed_transaction_with_expiration_time(Duration::from_secs(0));
    pool.add_txn(txn, 0, 1, 0, TimelineState::NotReady, false);

    // insert few transactions after it
    // They supposed to be ready because there's sequential path from 0 to them
    add_txn(&mut pool, TestTransaction::new(1, 2, 1)).unwrap();
    add_txn(&mut pool, TestTransaction::new(1, 3, 1)).unwrap();

    // check that all txns are ready
    let (timeline, _) = pool.read_timeline(0, 10);
    assert_eq!(timeline.len(), 4);

    // gc expired transaction
    pool.gc_by_expiration_time(Duration::from_secs(1));

    // make sure txns 2 and 3 became not ready and we can't read them from any API
    let block = pool.get_block(10, HashSet::new());
    assert_eq!(block.len(), 1);
    assert_eq!(block[0].sequence_number(), 0);

    let (timeline, _) = pool.read_timeline(0, 10);
    assert_eq!(timeline.len(), 1);
    assert_eq!(timeline[0].sequence_number(), 0);
}

#[test]
fn test_clean_stuck_transactions() {
    let mut pool = setup_mempool().0;
    for seq in 0..5 {
        add_txn(&mut pool, TestTransaction::new(0, seq, 1)).unwrap();
    }
    let db_sequence_number = 10;
    let txn = TestTransaction::new(0, db_sequence_number, 1).make_signed_transaction();
    pool.add_txn(
        txn,
        0,
        1,
        db_sequence_number,
        TimelineState::NotReady,
        false,
    );
    let block = pool.get_block(10, HashSet::new());
    assert_eq!(block.len(), 1);
    assert_eq!(block[0].sequence_number(), 10);
}

#[test]
fn test_ttl_cache() {
    let mut cache = TtlCache::new(2, Duration::from_secs(1));
    // test basic insertion
    cache.insert(1, 1);
    cache.insert(1, 2);
    cache.insert(2, 2);
    cache.insert(1, 3);
    assert_eq!(cache.get(&1), Some(&3));
    assert_eq!(cache.get(&2), Some(&2));
    assert_eq!(cache.size(), 2);
    // test reaching max capacity
    cache.insert(3, 3);
    assert_eq!(cache.size(), 2);
    assert_eq!(cache.get(&1), Some(&3));
    assert_eq!(cache.get(&3), Some(&3));
    assert_eq!(cache.get(&2), None);
    // test ttl functionality
    cache.gc(SystemTime::now()
        .checked_add(Duration::from_secs(10))
        .unwrap());
    assert_eq!(cache.size(), 0);
}
