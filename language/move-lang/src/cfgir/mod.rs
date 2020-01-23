// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod absint;
pub mod ast;
mod borrows;
pub mod cfg;
mod liveness;
mod locals;
pub mod translate;

use crate::shared::unique_map::UniqueMap;
use crate::{errors::Errors, parser::ast::Var};
use ast::*;
use cfg::*;
use std::collections::BTreeSet;

/// This is a placeholder for "optimization passes" that "fix" operations so the behave as expected
/// The two major passes here are:
/// - Last inferred copy becomes an inferred move
/// - References are "released"/popped after their last usage
///   - Might prove be a bit tricky to get exactly right as it might happen only at the statement
///     level instead of the expression level
pub fn refine_and_verify(
    errors: &mut Errors,
    signature: &FunctionSignature,
    locals: &UniqueMap<Var, SingleType>,
    cfg: &mut BlockCFG,
    infinite_loop_starts: &BTreeSet<Label>,
) {
    liveness::refine_and_verify(errors, signature, locals, cfg, infinite_loop_starts);
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
