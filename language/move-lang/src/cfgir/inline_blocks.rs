// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::cfg::{BlockCFG, CFG};
use crate::hlir::ast::{BasicBlocks, Command_, Label};
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
    inline_single_target_blocks(&single_target_labels, blocks)
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

fn inline_single_target_blocks(
    single_jump_targets: &BTreeSet<Label>,
    blocks: &mut BasicBlocks,
) -> bool {
    let labels_vec = blocks.keys().cloned().collect::<Vec<_>>();
    let mut labels = labels_vec.into_iter();
    let mut next = labels.next();

    let mut working_blocks = std::mem::replace(blocks, BasicBlocks::new());
    let finished_blocks = blocks;

    let mut changed = false;
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
                changed = true;
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
    changed
}
