// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[macro_use]
mod context;
mod labels_to_offsets;
mod remove_fallthrough_jumps;
pub mod translate;

use move_vm::file_format as F;
use std::collections::HashMap;

fn remap_offsets(blocks: &mut Vec<Vec<F::Bytecode>>, map: &HashMap<u16, u16>) {
    use F::Bytecode as B;
    for block in blocks {
        for mut instr in block {
            match &mut instr {
                B::Branch(lbl) | B::BrTrue(lbl) | B::BrFalse(lbl) => {
                    *lbl = map[lbl];
                }
                _ => (),
            }
        }
    }
}
