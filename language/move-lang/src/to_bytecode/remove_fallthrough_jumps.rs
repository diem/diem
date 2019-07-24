// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_vm::file_format as F;
use std::{collections::HashMap, convert::TryInto};

// Removes any "fall through jumps", i.e. this a is a jump directly to the next instruction.
// Iterates to find a fixpoint as it might create empty blocks which could create more jumps to
// clean up

pub fn code(blocks: &mut Vec<Vec<F::Bytecode>>) {
    let mut changed = true;
    while changed {
        let fall_through_removed = remove_fall_through(blocks);
        let block_removed = remove_empty_blocks(blocks);
        changed = fall_through_removed || block_removed;
    }
}

fn remove_fall_through(blocks: &mut Vec<Vec<F::Bytecode>>) -> bool {
    use F::Bytecode as B;
    let mut changed = false;
    for (label, block) in blocks.iter_mut().enumerate() {
        let next_block: u16 = (label + 1).try_into().unwrap();
        let remove_last = match block.last().unwrap() {
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

fn remove_empty_blocks(blocks: &mut Vec<Vec<F::Bytecode>>) -> bool {
    let mut label_map = HashMap::new();
    let mut num_removed = 0;

    let mut removed = false;
    let old_blocks = std::mem::replace(blocks, vec![]);
    for (label, block) in old_blocks.into_iter().enumerate() {
        let lbl = label as u16;
        label_map.insert(lbl, lbl - num_removed);

        if block.is_empty() {
            num_removed += 1;
            removed = true;
        } else {
            blocks.push(block)
        }
    }

    if removed {
        super::remap_offsets(blocks, &label_map);
    }

    removed
}
