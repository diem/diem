// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod absint;
pub mod ast;
mod borrows;
pub(crate) mod cfg;
mod constant_fold;
mod eliminate_locals;
mod liveness;
mod locals;
mod remove_no_ops;
pub(crate) mod translate;

use crate::{
    errors::Errors,
    hlir::ast::*,
    parser::ast::{StructName, Var},
    shared::unique_map::UniqueMap,
};
use cfg::*;
use move_ir_types::location::*;
use std::collections::{BTreeMap, BTreeSet};

pub fn refine_inference_and_verify(
    errors: &mut Errors,
    signature: &FunctionSignature,
    acquires: &BTreeMap<StructName, Loc>,
    locals: &UniqueMap<Var, SingleType>,
    cfg: &mut BlockCFG,
    infinite_loop_starts: &BTreeSet<Label>,
) {
    remove_no_ops::optimize(cfg);

    liveness::last_usage(errors, locals, cfg, infinite_loop_starts);
    let locals_states = locals::verify(errors, signature, acquires, locals, cfg);

    liveness::release_dead_refs(&locals_states, locals, cfg, infinite_loop_starts);
    borrows::verify(errors, signature, acquires, locals, cfg);
}

pub fn optimize(
    _signature: &FunctionSignature,
    _locals: &UniqueMap<Var, SingleType>,
    cfg: &mut BlockCFG,
) {
    loop {
        let mut changed = false;
        changed = eliminate_locals::optimize(cfg) || changed;
        changed = constant_fold::optimize(cfg) || changed;

        if !changed {
            break;
        }
    }
}
