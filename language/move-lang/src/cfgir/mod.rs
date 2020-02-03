// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod absint;
pub mod ast;
mod borrows;
pub mod cfg;
mod eliminate_locals;
mod liveness;
mod locals;
mod remove_no_ops;
pub mod translate;

use crate::shared::unique_map::UniqueMap;
use crate::{errors::Errors, parser::ast::Var};
use ast::*;
use cfg::*;
use std::collections::BTreeSet;

pub fn refine_inference_and_verify(
    errors: &mut Errors,
    signature: &FunctionSignature,
    locals: &UniqueMap<Var, SingleType>,
    cfg: &mut BlockCFG,
    infinite_loop_starts: &BTreeSet<Label>,
) {
    remove_no_ops::optimize(cfg);
    liveness::refine_inference_and_verify(errors, signature, locals, cfg, infinite_loop_starts);
}

pub fn verify(
    errors: &mut Errors,
    signature: &FunctionSignature,
    locals: &UniqueMap<Var, SingleType>,
    cfg: &BlockCFG,
) {
    locals::verify(errors, signature, locals, cfg);
    borrows::verify(errors, signature, locals, cfg)
}

pub fn optimize(
    _signature: &FunctionSignature,
    _locals: &UniqueMap<Var, SingleType>,
    cfg: &mut BlockCFG,
) {
    eliminate_locals::optimize(cfg);
}
