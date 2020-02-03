// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::{ast::*, cfg::BlockCFG};

pub fn optimize(cfg: &mut BlockCFG) {
    for block in cfg.blocks_mut().values_mut() {
        let old_block = std::mem::replace(block, BasicBlock::new());
        *block = old_block
            .into_iter()
            .filter(|c| !c.value.is_unit())
            .collect()
    }
}
