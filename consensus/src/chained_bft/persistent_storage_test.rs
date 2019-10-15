// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chained_bft::{
        persistent_storage::RecoveryData,
        test_utils::{
            build_chain,
            build_simple_tree,
            placeholder_ledger_info_for_consensus_block_id,
        },
    },
};
use consensus_types::{
    quorum_cert::QuorumCert,
    block::{
        Block,
        ExecutedBlock,
    },
};
use std::sync::Arc;
use crypto::HashValue;

/// Testing strategy for `RecoveryData::find_root()`
///
/// Partition the inputs as follows:
///  - blocks:          non-branched chain: Genesis--> A1--> A2--> A3--> A4---> ...
///
///                     simple tree:              ╭--> A1--> A2--> A3
///                                         Genesis--> B1--> B2
///                                                     ╰--> C1
///  - quorum_certs:    non-branched chain, simple tree (matches the `blocks` input above)
///  - storage_ledger:  disconnect,
///                     strict ancestor of the root,
///                     root (i.e. LI(S).block.round == LI(C).block.round)
///
/// Partition the outputs as follows:
///  - validity:        root found, root not found
///  - LI(S) and LI(C) relationship:    LI(S) exist && LI(S) is an ancestor of LI(C)
///                                     LI(S) exist && LI(S) is NOT an ancestor of LI(C)
///                                     LI(S) does not exist

/// Partially obtain parameters for `RecoveryData::find_root()`.
/// The parameter of `storage_ledger` will be manually defined, and will control the output of `find_root()`.
fn get_find_root_params(
    executed_blocks: Vec<Arc<ExecutedBlock<Vec<usize>>>>
) -> (Vec<Block<Vec<usize>>>, Vec<QuorumCert>) {
    let blocks: Vec<Block<Vec<usize>>> =
        executed_blocks
            .iter()
            .map(|executed_block|
                executed_block.block().clone()
            )
            .collect();
    let quorum_certs: Vec<QuorumCert> =
        executed_blocks
            .iter()
            .map(|executed_block|
                executed_block.block().quorum_cert().clone()
            )
            .collect();
    (blocks, quorum_certs)
}

/// covers
///  - blocks: non-branched chain
///  - quorum_certs: non-branched chain
///  - storage_ledger: disconnect
///  - validity: root found
///  - LI(S) and LI(C) relationship: LI(S) does not exist
///
/// ==> LI(C) is the root block
#[test]
fn test_chain_restartability_root_from_consensusdb() {
    let (executed_blocks, _) = build_chain();
    let (blocks, quorum_certs) = get_find_root_params(executed_blocks);

    // We arbitrarily trim the blocks, while keeping the following invariant.
    // The root of the chain of blocks must have a parent, a grandparent, a child, and a grandchild.
    const B_IDX_START: usize = 2;
    const B_IDX_END: usize = 6;
    assert!(1 < B_IDX_START && B_IDX_START < blocks.len() - 2);

    // Define ledger info for storage.
    let li_storage = &blocks[0];

    // Obtain parameters for `RecoveryData::find_root()`.
    let mut blocks_advanced = blocks[B_IDX_START..B_IDX_END].iter().cloned().collect();
    let mut qc_advanced = quorum_certs[B_IDX_START..B_IDX_END].iter().cloned().collect();
    let storage_ledger = placeholder_ledger_info_for_consensus_block_id(li_storage.id());

    // Subset of blocks is inputted to `find_root()` so that LI(S) is disconnected to LI(C).
    let (root_block, root_quorum_cert, root_ledger_info) =
        RecoveryData::find_root(&mut blocks_advanced, &mut qc_advanced, &storage_ledger).unwrap();

    // Verify that LI(C) is the root.
    let li_consensus = &blocks[B_IDX_END - 3];
    let li_consensus_child = &blocks[B_IDX_END - 2];
    let li_consensus_grandchild = &blocks[B_IDX_END - 1];
    assert_eq!(*li_consensus, root_block);
    assert_eq!(*li_consensus_child.quorum_cert(), root_quorum_cert);
    assert_eq!(*li_consensus_grandchild.quorum_cert(), root_ledger_info);
}


/// covers
///  - blocks: non-branched chain
///  - quorum_certs: non-branched chain
///  - storage_ledger: strict ancestor of the root
///  - validity: root found
///  - LI(S) and LI(C) relationship: LI(S) exist && LI(S) is an ancestor of LI(C)
///
/// ==> LI(S) is the root block
#[test]
fn test_chain_restartability_root_from_storage() {
    let (executed_blocks, _) = build_chain();
    let (blocks, mut quorum_certs) = get_find_root_params(executed_blocks);

    const STORAGE_LI_IDX: usize = 2;
    assert!(STORAGE_LI_IDX < blocks.len() - 4, "LI(S) must be an ancestor of LI(C)");

    // Define ledger info for storage.
    let li_storage = &blocks[STORAGE_LI_IDX];
    let li_storage_child = &blocks[STORAGE_LI_IDX + 1];
    let li_storage_grandchild = &blocks[STORAGE_LI_IDX + 2];

    // Obtain parameters for `RecoveryData::find_root()`.
    // `blocks` and `quorum_certs` are not trimmed, because LI(S) must be an ancestor of LI(C).
    let storage_ledger = placeholder_ledger_info_for_consensus_block_id(li_storage.id());

    // Subset of blocks is inputted to `find_root()` so that LI(S) is disconnected to LI(C).
    let (root_block, root_quorum_cert, root_ledger_info) =
        RecoveryData::find_root(&mut blocks.clone(), &mut quorum_certs, &storage_ledger).unwrap();

    // Verify that LI(S) is the root.
    assert_eq!(*li_storage, root_block);
    assert_eq!(*li_storage_child.quorum_cert(), root_quorum_cert);
    assert_eq!(*li_storage_grandchild.quorum_cert(), root_ledger_info);
}


/// covers
///  - blocks: non-branched chain
///  - quorum_certs: non-branched chain
///  - storage_ledger: root
///  - validity: root found
///  - LI(S) and LI(C) relationship: LI(S) exist && LI(S) is an ancestor of LI(C)
///                                  (note: a block is considered to be an ancestor of itself)
///
/// ==> LI(S) is the root block
#[test]
fn test_chain_restartability_same_root() {
    let (executed_blocks, _) = build_chain();
    let (blocks, mut quorum_certs) = get_find_root_params(executed_blocks);

    const STORAGE_LI_IDX: usize = 4;
    assert_eq!(STORAGE_LI_IDX, blocks.len() - 4, "LI(S) and LI(C) must be the same");

    // Define ledger info for storage.
    let li_storage = &blocks[STORAGE_LI_IDX];
    let li_storage_child = &blocks[STORAGE_LI_IDX + 1];
    let li_storage_grandchild = &blocks[STORAGE_LI_IDX + 2];

    // Obtain parameters for `RecoveryData::find_root()`.
    // `blocks` and `quorum_certs` are not trimmed, because LI(S) must be an ancestor of LI(C).
    let storage_ledger = placeholder_ledger_info_for_consensus_block_id(li_storage.id());

    // Subset of blocks is inputted to `find_root()` so that LI(S) is disconnected to LI(C).
    let (root_block, root_quorum_cert, root_ledger_info) =
        RecoveryData::find_root(&mut blocks.clone(), &mut quorum_certs, &storage_ledger).unwrap();

    // Verify that LI(S) is the root.
    assert_eq!(*li_storage, root_block);
    assert_eq!(*li_storage_child.quorum_cert(), root_quorum_cert);
    assert_eq!(*li_storage_grandchild.quorum_cert(), root_ledger_info);
}

/// covers
///  - blocks: simple tree
///  - quorum_certs: simple tree
///  - storage_ledger: disconnect
///  - validity: root found
///  - LI(S) and LI(C) relationship: LI(S) does not exist
///
/// ==> LI(C) is the root block
#[test]
fn test_tree_restartability_root_from_consensusdb() {
    let (executed_blocks, _) = build_simple_tree();
    let (mut blocks, mut quorum_certs) = get_find_root_params(executed_blocks.clone());

    // Use a random hash value as the consensus block id.
    // (i.e. LI(S) does not exist.)
    let storage_ledger = placeholder_ledger_info_for_consensus_block_id(HashValue::random());

    // Subset of blocks is inputted to `find_root()` so that LI(S) is disconnected to LI(C).
    let (root_block, root_quorum_cert, root_ledger_info) =
        RecoveryData::find_root(&mut blocks, &mut quorum_certs, &storage_ledger).unwrap();

    // Unpack the longest valid chain in the tree. See `build_simple_tree()` diagram.
    let a1 = executed_blocks[1].block();
    let a2 = executed_blocks[2].block();
    let a3 = executed_blocks[3].block();

    // Verify that LI(C), or a1, is the root.
    assert_eq!(*a1, root_block);
    assert_eq!(*a2.quorum_cert(), root_quorum_cert);
    assert_eq!(*a3.quorum_cert(), root_ledger_info);
}

/// covers
///  - blocks: simple tree
///  - quorum_certs: simple tree
///  - storage_ledger: root
///  - validity: root found
///  - LI(S) and LI(C) relationship: LI(S) exist && LI(S) is an ancestor of LI(C)
///                                  (note: a block is considered to be an ancestor of itself)
///
/// ==> LI(S) is the root block
#[test]
fn test_tree_restartability_same_root() {
    let (executed_blocks, _) = build_simple_tree();
    let (mut blocks, mut quorum_certs) = get_find_root_params(executed_blocks.clone());

    // Construct the storage ledger using the id of a1.
    let storage_ledger = placeholder_ledger_info_for_consensus_block_id(executed_blocks[1].block().id());

    // Subset of blocks is inputted to `find_root()` so that LI(S) is disconnected to LI(C).
    let (root_block, root_quorum_cert, root_ledger_info) =
        RecoveryData::find_root(&mut blocks, &mut quorum_certs, &storage_ledger).unwrap();

    // Unpack the longest valid chain in the tree. See `build_simple_tree()` diagram.
    let a1 = executed_blocks[1].block();
    let a2 = executed_blocks[2].block();
    let a3 = executed_blocks[3].block();

    // Verify that LI(S), or a1, is the root. (Note: LI(S) == LI(C))
    assert_eq!(*a1, root_block);
    assert_eq!(*a2.quorum_cert(), root_quorum_cert);
    assert_eq!(*a3.quorum_cert(), root_ledger_info);
}


/// covers
///  - blocks: non-branched chain
///  - quorum_certs: non-branched chain
///  - storage_ledger: disconnect
///  - validity: root not found
///  - LI(S) and LI(C) relationship: N/A
///
/// ==> Error
#[test]
#[should_panic]
fn test_not_restartable_too_short() {
    let (executed_blocks, _) = build_chain();
    let (blocks, quorum_certs) = get_find_root_params(executed_blocks);

    // Set a random storage ledger parameter for `RecoveryData::find_root()`.
    // `find_root()` will panic regardless of the storage ledger.
    let storage_ledger = placeholder_ledger_info_for_consensus_block_id(HashValue::random());

    let mut blocks_shortened = blocks[..2].iter().cloned().collect();
    let mut qc_shortened = quorum_certs[..2].iter().cloned().collect();

    // Subset of blocks is inputted to `find_root()` so that LI(S) is disconnected to LI(C).
    let (root_block, _, _) =
        RecoveryData::find_root(&mut blocks_shortened, &mut qc_shortened, &storage_ledger).unwrap();

    // `find_root()` returns an Error.
    let a1 = &blocks[1];
    assert_eq!(*a1, root_block);
}
