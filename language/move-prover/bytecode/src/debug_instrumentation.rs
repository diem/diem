// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Transformation which injects trace instructions which are used to visualize execution.
//!
//! This transformation should run before copy propagation and any other bytecode modifications.
//! It emits instructions of the form `trace_local[original_idx](idx)`. Initially
//! `original_idx == idx`, where the temp `idx` is a named variable from the Move
//! compiler. Later transformations may replace `idx` but `original_idx` will be preserved so
//! the user sees the value of their named variable.

use crate::{
    function_data_builder::FunctionDataBuilder,
    function_target::FunctionData,
    function_target_pipeline::{FunctionTargetProcessor, FunctionTargetsHolder},
    stackless_bytecode::{Bytecode, Operation},
};

use move_model::model::FunctionEnv;

pub struct DebugInstrumenter {}

impl DebugInstrumenter {
    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }
}

impl FunctionTargetProcessor for DebugInstrumenter {
    fn process(
        &self,
        _targets: &mut FunctionTargetsHolder,
        fun_env: &FunctionEnv<'_>,
        data: FunctionData,
    ) -> FunctionData {
        use Bytecode::*;

        if fun_env.is_native() {
            // Nothing to do
            return data;
        }

        let mut builder = FunctionDataBuilder::new(fun_env, data);
        let code = std::mem::take(&mut builder.data.code);

        // Emit trace instructions for parameters at entry.
        builder.set_loc(builder.fun_env.get_loc().at_start());
        for i in 0..builder.fun_env.get_parameter_count() {
            builder.emit_with(|id| Call(id, vec![], Operation::TraceLocal(i), vec![i]));
        }

        for bc in code {
            match &bc {
                Ret(id, locals) => {
                    // Emit trace instructions for return values.
                    builder.set_loc_from_attr(*id);
                    for (i, l) in locals.iter().enumerate() {
                        builder
                            .emit_with(|id| Call(id, vec![], Operation::TraceReturn(i), vec![*l]));
                    }
                    builder.emit(bc);
                }
                Abort(id, l) => {
                    builder.set_loc_from_attr(*id);
                    builder.emit_with(|id| Call(id, vec![], Operation::TraceAbort, vec![*l]));
                    builder.emit(bc);
                }
                _ => {
                    builder.set_loc_from_attr(bc.get_attr_id());
                    builder.emit(bc.clone());
                    // Emit trace instructions for modified values.
                    for idx in bc.modifies() {
                        // Only emit this for user declared locals, not for ones introduced
                        // by stack elimination.
                        if idx < fun_env.get_local_count() {
                            builder.emit_with(|id| {
                                Call(id, vec![], Operation::TraceLocal(idx), vec![idx])
                            });
                        }
                    }
                }
            }
        }

        builder.data
    }

    fn name(&self) -> String {
        "debug_instrumenter".to_string()
    }
}
