// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod absint;
pub mod ast;
mod borrows;
pub(crate) mod cfg;
mod constant_fold;
mod eliminate_locals;
mod inline_blocks;
mod liveness;
mod locals;
mod remove_no_ops;
mod simplify_jumps;
pub(crate) mod translate;

use crate::{
    expansion::ast::{AbilitySet, ModuleIdent},
    hlir::ast::*,
    parser::ast::{StructName, Var},
    shared::{unique_map::UniqueMap, CompilationEnv},
};
use cfg::*;
use move_ir_types::location::*;
use std::collections::{BTreeMap, BTreeSet};

pub fn refine_inference_and_verify(
    compilation_env: &mut CompilationEnv,
    struct_declared_abilities: &UniqueMap<ModuleIdent, UniqueMap<StructName, AbilitySet>>,
    signature: &FunctionSignature,
    acquires: &BTreeMap<StructName, Loc>,
    locals: &UniqueMap<Var, SingleType>,
    cfg: &mut BlockCFG,
    infinite_loop_starts: &BTreeSet<Label>,
) {
    liveness::last_usage(compilation_env, locals, cfg, infinite_loop_starts);
    let locals_states = locals::verify(
        compilation_env,
        struct_declared_abilities,
        signature,
        acquires,
        locals,
        cfg,
    );

    liveness::release_dead_refs(&locals_states, locals, cfg, infinite_loop_starts);
    borrows::verify(compilation_env, signature, acquires, locals, cfg);
}

pub fn optimize(
    signature: &FunctionSignature,
    _locals: &UniqueMap<Var, SingleType>,
    cfg: &mut BlockCFG,
) {
    loop {
        let mut changed = false;
        changed |= eliminate_locals::optimize(signature, cfg);
        changed |= constant_fold::optimize(cfg);
        changed |= simplify_jumps::optimize(cfg);
        changed |= inline_blocks::optimize(cfg);

        if !changed {
            break;
        }
    }
}
