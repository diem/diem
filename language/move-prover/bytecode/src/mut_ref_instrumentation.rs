// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    function_data_builder::FunctionDataBuilder,
    function_target::FunctionData,
    function_target_pipeline::{FunctionTargetProcessor, FunctionTargetsHolder},
    stackless_bytecode::{Bytecode, Operation},
};

use crate::stackless_bytecode::AssignKind;
use itertools::Itertools;
use move_model::ast::TempIndex;
pub use move_model::{
    model::{FunctionEnv, Loc},
    ty::Type,
};

pub struct MutRefInstrumenter {}

impl MutRefInstrumenter {
    pub fn new() -> Box<Self> {
        Box::new(MutRefInstrumenter {})
    }
}

impl FunctionTargetProcessor for MutRefInstrumenter {
    fn process(
        &self,
        _targets: &mut FunctionTargetsHolder,
        fun_env: &FunctionEnv<'_>,
        data: FunctionData,
    ) -> FunctionData {
        if fun_env.is_native() {
            return data;
        }

        let mut builder = FunctionDataBuilder::new(fun_env, data);

        // Compute &mut parameters.
        let param_count = builder.get_target().get_parameter_count();
        let mut_ref_params = (0..param_count)
            .filter(|idx| is_mut_ref(&builder, *idx))
            .collect_vec();

        // Transform bytecode.
        for bc in std::mem::take(&mut builder.data.code) {
            use Bytecode::*;
            use Operation::*;
            match bc {
                Assign(attr_id, dest, src, AssignKind::Move)
                    if src < param_count && is_mut_ref(&builder, src) =>
                {
                    // Do not allow a move of a &mut parameter. This would make it hard
                    // to return the right version of it at return. Instead turn this into
                    // a copy, which will ensure any value updated will be written back
                    // to the original parameter before returned (via borrow semantics).
                    builder.emit(Assign(attr_id, dest, src, AssignKind::Copy))
                }
                Ret(attr_id, rets) => {
                    // Emit traces for &mut params at exit.
                    builder.set_loc_from_attr(attr_id);
                    for added in &mut_ref_params {
                        builder.emit_with(|id| {
                            Call(id, vec![], TraceLocal(*added), vec![*added], None)
                        });
                    }
                    builder.emit(Ret(attr_id, rets));
                }
                _ => builder.emit(bc),
            }
        }

        builder.data
    }

    fn name(&self) -> String {
        "mut_ref_instrumentation".to_string()
    }
}

fn is_mut_ref(builder: &FunctionDataBuilder<'_>, idx: TempIndex) -> bool {
    builder
        .get_target()
        .get_local_type(idx)
        .is_mutable_reference()
}
