// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_vm::file_format as F;
use std::{collections::HashMap, convert::TryInto};

// Switches the jumps from being block-indexed to being instruction-indexed

pub fn code(mut blocks: Vec<Vec<F::Bytecode>>) -> Vec<F::Bytecode> {
    let mut offset = 0;
    let mut label_to_offset = HashMap::new();
    for (lbl, block) in blocks.iter().enumerate() {
        label_to_offset.insert(lbl as u16, offset.try_into().unwrap());
        offset += block.len();
    }

    super::remap_offsets(&mut blocks, &label_to_offset);

    blocks.into_iter().flatten().collect()
}
