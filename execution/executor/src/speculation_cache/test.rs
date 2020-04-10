// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use libra_types::{block_info::BlockInfo, on_chain_config::ValidatorSet};

fn id(index: u64) -> HashValue {
    let bytes = index.to_be_bytes();
    let mut buf = [0; HashValue::LENGTH];
    buf[HashValue::LENGTH - 8..].copy_from_slice(&bytes);
    HashValue::new(buf)
}

fn gen_block(id: HashValue) -> (HashValue, Vec<Transaction>, ProcessedVMOutput) {
    (
        id,
        vec![],
        ProcessedVMOutput::new(vec![], ExecutedTrees::new_empty(), None),
    )
}

fn gen_ledger_info(block_id: HashValue, reconfig: bool) -> LedgerInfo {
    LedgerInfo::new(
        BlockInfo::new(
            1,
            0,
            block_id,
            HashValue::zero(),
            0,
            0,
            if reconfig {
                Some(ValidatorSet::empty())
            } else {
                None
            },
        ),
        HashValue::zero(),
    )
}

fn create_cache() -> SpeculationCache {
    //    * ---> 1 ---> 2
    //    |      |
    //    |      └----> 3 ---> 4
    //    |             |
    //    |             └----> 5
    //    |
    //    └----> 6 ---> 7 ---> 8
    //           |
    //           └----> 9 ---> 10
    //                  |
    //                  └----> 11
    // *: PRE_GENESIS_BLOCK_ID
    let mut cache = SpeculationCache::new();

    cache
        .add_block(*PRE_GENESIS_BLOCK_ID, gen_block(id(1)))
        .unwrap();
    cache.add_block(id(1), gen_block(id(2))).unwrap();
    cache.add_block(id(1), gen_block(id(3))).unwrap();
    cache.add_block(id(3), gen_block(id(4))).unwrap();
    cache.add_block(id(3), gen_block(id(5))).unwrap();
    cache
        .add_block(*PRE_GENESIS_BLOCK_ID, gen_block(id(6)))
        .unwrap();
    cache.add_block(id(6), gen_block(id(7))).unwrap();
    cache.add_block(id(7), gen_block(id(8))).unwrap();
    cache.add_block(id(6), gen_block(id(9))).unwrap();
    cache.add_block(id(9), gen_block(id(10))).unwrap();
    cache.add_block(id(9), gen_block(id(11))).unwrap();
    cache
}

#[test]
fn test_branch() {
    let mut cache = create_cache();
    // put counting blocks as a separate line to avoid core dump
    // if assertion fails.
    let mut num_blocks = cache.block_map.lock().unwrap().len();
    assert_eq!(num_blocks, 11);
    cache.prune(&gen_ledger_info(id(9), false)).unwrap();
    num_blocks = cache.block_map.lock().unwrap().len();
    assert_eq!(num_blocks, 2);
    assert_eq!(cache.committed_block_id, id(9));
}

#[test]
fn test_reconfig_id_update() {
    let mut cache = create_cache();
    cache.prune(&gen_ledger_info(id(1), true)).unwrap();
    let num_blocks = cache.block_map.lock().unwrap().len();
    assert_eq!(num_blocks, 4);
    assert_ne!(cache.committed_block_id, id(1));
}

#[test]
fn test_add_duplicate_block() {
    let mut cache = create_cache();
    cache.add_block(id(1), gen_block(id(7))).unwrap();
    cache.add_block(id(1), gen_block(id(7))).unwrap();
}

#[test]
fn test_add_block_missing_parent() {
    let mut cache = create_cache();
    assert!(cache.add_block(id(99), gen_block(id(100))).is_err());
}
