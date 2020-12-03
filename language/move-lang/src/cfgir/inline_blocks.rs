// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::cfg::{BlockCFG, CFG};
use crate::{
    cfgir::ast::remap_labels,
    hlir::ast::{BasicBlocks, Command_, Label},
};
use std::collections::{BTreeMap, BTreeSet};

/// returns true if anything changed
pub fn optimize(cfg: &mut BlockCFG) -> bool {
    let changed = optimize_(cfg.start_block(), cfg.blocks_mut());
    if changed {
        let dead_blocks = cfg.recompute();
        assert!(dead_blocks.is_empty())
    }
    changed
}

fn optimize_(start: Label, blocks: &mut BasicBlocks) -> bool {
    let single_target_labels = find_single_target_labels(start, &blocks);
    inline_single_target_blocks(&single_target_labels, start, blocks)
}

fn find_single_target_labels(start: Label, blocks: &BasicBlocks) -> BTreeSet<Label> {
    use Command_ as C;
    let mut counts = BTreeMap::new();
    // 'start' block starts as one as it is the entry point of the function. In some sense,
    // there is an implicit "jump" to this label to begin executing the function
    counts.insert(start, 1);
    for block in blocks.values() {
        match &block.back().unwrap().value {
            C::JumpIf {
                cond: _cond,
                if_true,
                if_false,
            } => {
                *counts.entry(*if_true).or_insert(0) += 1;
                *counts.entry(*if_false).or_insert(0) += 1
            }
            C::Jump(lbl) => *counts.entry(*lbl).or_insert(0) += 1,
            _ => (),
        }
    }
    counts
        .into_iter()
        .filter(|(_, count)| *count == 1)
        .map(|(lbl, _)| lbl)
        .collect()
}

#[allow(clippy::needless_collect)]
fn inline_single_target_blocks(
    single_jump_targets: &BTreeSet<Label>,
    start: Label,
    blocks: &mut BasicBlocks,
) -> bool {
    //cleanup of needless_collect would result in mut and non mut borrows, and compilation warning.
    let labels_vec = blocks.keys().cloned().collect::<Vec<_>>();
    let mut labels = labels_vec.into_iter();
    let mut next = labels.next();

    let mut working_blocks = std::mem::replace(blocks, BasicBlocks::new());
    let finished_blocks = blocks;

    let mut remapping = BTreeMap::new();
    while let Some(cur) = next {
        let mut block = match working_blocks.remove(&cur) {
            None => {
                next = labels.next();
                continue;
            }
            Some(b) => b,
        };

        match block.back().unwrap() {
            // Do not need to worry about infinitely unwrapping loops as loop heads will always
            // be the target of at least 2 jumps: the jump to the loop and the "continue" jump
            // This is always true as long as we start the count for the start label at 1
            sp!(_, Command_::Jump(target)) if single_jump_targets.contains(target) => {
                remapping.insert(cur, *target);
                let target_block = working_blocks.remove(target).unwrap();
                block.pop_back();
                block.extend(target_block);
                working_blocks.insert(cur, block);
            }
            _ => {
                finished_blocks.insert(cur, block);
                next = labels.next();
            }
        }
    }

    let changed = !remapping.is_empty();
    remap_to_last_target(remapping, start, finished_blocks);
    changed
}

/// In order to preserve loop invariants at the bytecode level, when a block is "inlined", that
/// block needs to be relabelled as the "inlined" block
/// Without this, blocks that were outside of loops could be inlined into the loop-body, breaking
/// invariants needed at the bytecode level.
/// For example:
/// Before:
///   A: block_a; jump B
///   B: block_b
///
///   s.t. A is the only one jumping to B
///
/// After:
///   B: block_a; block_b
fn remap_to_last_target(
    mut remapping: BTreeMap<Label, Label>,
    start: Label,
    blocks: &mut BasicBlocks,
) {
    // The start block can't be relabelled in the current CFG API.
    // But it does not need to be since it will always be the first block, thus it will not run
    // into issues in the bytecode verifier
    remapping.remove(&start);
    if remapping.is_empty() {
        return;
    }
    // populate remapping for non changed blocks
    for label in blocks.keys() {
        remapping.entry(*label).or_insert(*label);
    }
    let owned_blocks = std::mem::replace(blocks, BasicBlocks::new());
    let (_start, remapped_blocks) = remap_labels(&remapping, start, owned_blocks);
    *blocks = remapped_blocks;
}
