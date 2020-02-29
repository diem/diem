// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_ir_types::ast as IR;
use std::collections::HashMap;

// Removes any "fall through jumps", i.e. this a is a jump directly to the next instruction.
// Iterates to find a fixpoint as it might create empty blocks which could create more jumps to
// clean up

pub fn code(blocks: &mut IR::BytecodeBlocks) {
    let mut changed = true;
    while changed {
        let fall_through_removed = remove_fall_through(blocks);
        let block_removed = remove_empty_blocks(blocks);
        changed = fall_through_removed || block_removed;
    }
}

fn remove_fall_through(blocks: &mut IR::BytecodeBlocks) -> bool {
    use IR::Bytecode_ as B;
    let mut changed = false;
    for idx in 0..(blocks.len() - 1) {
        let next_block = blocks.get(idx + 1).unwrap().0.clone();
        let (_, block) = blocks.get_mut(idx).unwrap();
        let remove_last = match &block.last().unwrap().value {
            B::Branch(lbl) if lbl == &next_block => true,
            _ => false,
        };
        if remove_last {
            changed = true;
            block.pop();
        }
    }
    changed
}

fn remove_empty_blocks(blocks: &mut IR::BytecodeBlocks) -> bool {
    let mut label_map = HashMap::new();
    let mut cur_label = None;
    let mut removed = false;
    let old_blocks = std::mem::replace(blocks, vec![]);
    for (label, block) in old_blocks.into_iter().rev() {
        if block.is_empty() {
            removed = true;
        } else {
            cur_label = Some(label.clone());
            blocks.push((label.clone(), block))
        }
        label_map.insert(label, cur_label.clone().unwrap());
    }
    blocks.reverse();

    if removed {
        super::remap_labels(blocks, &label_map);
    }

    removed
}
