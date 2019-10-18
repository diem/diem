// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chained_bft::test_utils::{placeholder_ledger_info, TestPayload};
use crate::chained_bft::{
    persistent_storage::RecoveryData,
    test_utils::{build_chain, build_simple_tree},
};
use consensus_types::{
    block::{Block, ExecutedBlock},
    quorum_cert::QuorumCert,
};
use libra_crypto::HashValue;
use std::sync::Arc;

/// Partially obtain parameters for `RecoveryData::find_root()`.
/// The parameter of `storage_ledger` will be manually defined, and will control the output of `find_root()`.
fn get_find_root_params(
    executed_blocks: Vec<Arc<ExecutedBlock<TestPayload>>>,
) -> (Vec<Block<TestPayload>>, Vec<QuorumCert>) {
    executed_blocks
        .iter()
        .map(|eb| (eb.block().clone(), eb.block().quorum_cert().clone()))
        .unzip()
}

/// Verify that the found root matches the expected root.
fn is_correct_root(
    blocks: &[Block<TestPayload>],
    committed_b_idx: usize,
    found_root: (Block<TestPayload>, QuorumCert, QuorumCert),
) -> bool {
    let root = &blocks[committed_b_idx];
    let qc = blocks[committed_b_idx + 1].quorum_cert();
    let li = blocks[committed_b_idx + 3].quorum_cert();
    *root == found_root.0 && *qc == found_root.1 && *li == found_root.2
}

/// Verify that the found root is the genesis.
fn is_genesis_root(found_root: (Block<TestPayload>, QuorumCert, QuorumCert)) -> bool {
    let genesis = Block::make_genesis_block();
    genesis == found_root.0
        && QuorumCert::certificate_for_genesis() == found_root.1
        && QuorumCert::certificate_for_genesis() == found_root.2
}

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
///                     ancestor of the root,
///                     root (i.e. LI(S).block.round == LI(C).block.round),
///                     special case (i.e. genesis)
///
/// Partition the outputs as follows:
///  - validity:        root found, root not found
///  - LI(S) and LI(C) relationship:    LI(S) exist && LI(S) is an ancestor of LI(C)
///                                     LI(S) exist && LI(S) is NOT an ancestor of LI(C)
///                                     LI(S) does not exist

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
    let executed_blocks = build_chain();
    let (blocks, quorum_certs) = get_find_root_params(executed_blocks);

    // Define ledger info for storage.
    let li_storage = &blocks[0];
    let mut storage_ledger = placeholder_ledger_info();
    storage_ledger.set_consensus_block_id(li_storage.id());

    // The root of the chain of blocks must have a parent, a grandparent, a child, and a grandchild.
    for b_idx_start in 2..blocks.len() - 3 {
        for b_idx_end in b_idx_start + 3..blocks.len() {
            // Arbitrarily trim the blocks, while keeping the following invariant.
            // Trimming the blocks disconnects LI(S) from LI(C).
            // Obtain parameters for `RecoveryData::find_root()`.
            let mut blocks_advanced = blocks[b_idx_start..=b_idx_end].to_vec();
            let mut qc_advanced = quorum_certs[b_idx_start..=b_idx_end].to_vec();

            // Subset of blocks is fed into `find_root()` so that LI(S) is disconnected from LI(C).
            let found_root =
                RecoveryData::find_root(&mut blocks_advanced, &mut qc_advanced, &storage_ledger)
                    .unwrap();

            // Verify that LI(C) is the root.
            assert!(is_correct_root(&blocks, b_idx_end - 3, found_root));
        }
    }
}

/// covers
///  - blocks: non-branched chain
///  - quorum_certs: non-branched chain
///  - storage_ledger: ancestor of the root
///  - validity: root found
///  - LI(S) and LI(C) relationship: LI(S) exist && LI(S) is an ancestor of LI(C)
///
/// ==> LI(S) is the root block
#[test]
fn test_chain_restartability_root_from_storage() {
    let executed_blocks = build_chain();
    let (blocks, quorum_certs) = get_find_root_params(executed_blocks);

    for committed_b_idx in 2..blocks.len() - 4 {
        // Define ledger info for storage.
        let li_storage = &blocks[committed_b_idx];
        let mut storage_ledger = placeholder_ledger_info();
        storage_ledger.set_consensus_block_id(li_storage.id());

        for b_idx_end in committed_b_idx + 4..blocks.len() {
            let mut blocks_advanced = blocks[committed_b_idx..=b_idx_end].to_vec();
            let mut qc_advanced = quorum_certs[committed_b_idx..=b_idx_end].to_vec();

            // Subset of blocks is fed into `find_root()` so that LI(S) is disconnected from LI(C).
            let found_root =
                RecoveryData::find_root(&mut blocks_advanced, &mut qc_advanced, &storage_ledger)
                    .unwrap();

            // Verify that LI(S) is the root.
            assert!(is_correct_root(&blocks, committed_b_idx, found_root));
        }
    }
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
    let executed_blocks = build_chain();
    let (blocks, quorum_certs) = get_find_root_params(executed_blocks);

    for committed_b_idx in 2..blocks.len() - 4 {
        // Define ledger info for storage.
        let li_storage = &blocks[committed_b_idx];
        let mut storage_ledger = placeholder_ledger_info();
        storage_ledger.set_consensus_block_id(li_storage.id());
        let b_idx_end = committed_b_idx + 3;

        let mut blocks_advanced = blocks[committed_b_idx..=b_idx_end].to_vec();
        let mut qc_advanced = quorum_certs[committed_b_idx..=b_idx_end].to_vec();

        // Subset of blocks is fed into `find_root()` so that LI(S) is disconnected from LI(C).
        let found_root =
            RecoveryData::find_root(&mut blocks_advanced, &mut qc_advanced, &storage_ledger)
                .unwrap();

        // Verify that LI(S) is the root.
        assert!(is_correct_root(&blocks, committed_b_idx, found_root));
    }
}

/// covers
///  - blocks: simple tree
///  - quorum_certs: simple tree
///  - storage_ledger: special case (i.e. genesis)
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
    let mut storage_ledger = placeholder_ledger_info();
    storage_ledger.set_consensus_block_id(HashValue::random());

    // Verify that LI(C) is the root.
    // If B0 <- [C0 <- B1] <- [C1 <- B2] <- [C2 <- B3], then `find_root` returns (B0, C0, C0).
    let found_root =
        RecoveryData::find_root(&mut blocks, &mut quorum_certs, &storage_ledger).unwrap();
    assert!(is_genesis_root(found_root));
}

/// covers
///  - blocks: simple tree
///  - quorum_certs: simple tree
///  - storage_ledger: root + special case (i.e. genesis)
///  - validity: root found
///  - LI(S) and LI(C) relationship: LI(S) exist && LI(S) is an ancestor of LI(C)
///                                  (note: a block is considered to be an ancestor of itself)
///
/// ==> LI(S) is the root block
#[test]
fn test_tree_restartability_same_root() {
    let (executed_blocks, _) = build_simple_tree();
    let (mut blocks, mut quorum_certs) = get_find_root_params(executed_blocks.clone());

    // Construct the storage ledger such that LI(S) is the ancestor of LI(C).
    // In this case LI(S) and LI(C) are equal.
    let mut storage_ledger = placeholder_ledger_info();
    storage_ledger.set_consensus_block_id(executed_blocks[0].block().id());

    // Verify that LI(S) is the root.
    // If B0 <- [C0 <- B1] <- [C1 <- B2] <- [C2 <- B3], then `find_root` returns (B0, C0, C0).
    let found_root =
        RecoveryData::find_root(&mut blocks, &mut quorum_certs, &storage_ledger).unwrap();
    assert!(is_genesis_root(found_root));
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
fn test_not_restartable_too_short() {
    let executed_blocks = build_chain();
    let (blocks, quorum_certs) = get_find_root_params(executed_blocks);

    // Set a random storage ledger parameter for `RecoveryData::find_root()`.
    // `find_root()` will panic regardless of the storage ledger.
    let mut storage_ledger = placeholder_ledger_info();
    storage_ledger.set_consensus_block_id(HashValue::random());

    let mut blocks_shortened = blocks[5..].to_vec();
    let mut qc_shortened = quorum_certs[5..].to_vec();

    // Subset of blocks is inputted to `find_root()` so that LI(S) is disconnected from LI(C).
    // `find_root()` returns an Error.
    assert!(
        RecoveryData::find_root(&mut blocks_shortened, &mut qc_shortened, &storage_ledger).is_err()
    );
}
