// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Transformation which injects data invariants into the bytecode.

use crate::{
    function_data_builder::FunctionDataBuilder,
    function_target::FunctionData,
    function_target_pipeline::{FunctionTargetProcessor, FunctionTargetsHolder, FunctionVariant},
    options::{ProverOptions, PROVER_DEFAULT_OPTIONS},
    stackless_bytecode::{Bytecode, Operation, PropKind},
};

use move_model::{
    ast,
    ast::{ConditionKind, Exp, TempIndex},
    model::{ConditionTag, FunctionEnv, StructEnv},
};

const INVARIANT_FAILS_MESSAGE: &str = "data invariant does not hold";

pub struct DataInvariantInstrumentationProcessor {}

impl DataInvariantInstrumentationProcessor {
    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }
}

impl FunctionTargetProcessor for DataInvariantInstrumentationProcessor {
    fn process(
        &self,
        _targets: &mut FunctionTargetsHolder,
        fun_env: &FunctionEnv<'_>,
        data: FunctionData,
    ) -> FunctionData {
        if fun_env.is_native() || fun_env.is_intrinsic() {
            // Nothing to do.
            return data;
        }
        if data.variant != FunctionVariant::Verification && !fun_env.has_friend() {
            // Only need to instrument if this is a verification variant, or if the
            // function has a friend and is verified in the friend's context.
            return data;
        }

        let options = fun_env
            .module_env
            .env
            .get_extension::<ProverOptions>()
            .unwrap_or_else(|| &*PROVER_DEFAULT_OPTIONS);

        Instrumenter::run(options, fun_env, data)
    }

    fn name(&self) -> String {
        "data_invariant_instrumenter".to_string()
    }
}

struct Instrumenter<'a> {
    _options: &'a ProverOptions,
    builder: FunctionDataBuilder<'a>,
}

impl<'a> Instrumenter<'a> {
    fn run(
        options: &'a ProverOptions,
        fun_env: &FunctionEnv<'a>,
        data: FunctionData,
    ) -> FunctionData {
        let builder = FunctionDataBuilder::new(fun_env, data);
        let mut instrumenter = Instrumenter {
            _options: options,
            builder,
        };
        instrumenter.instrument();
        instrumenter.builder.data
    }

    fn instrument(&mut self) {
        // Extract and clear current code
        let old_code = std::mem::take(&mut self.builder.data.code);

        // Emit entrypoint assumptions.
        self.instrument_entrypoint();

        // Instrument and generate new code
        for bc in old_code {
            self.instrument_bytecode(bc.clone());
        }
    }

    fn instrument_entrypoint(&mut self) {
        // For all parameters of struct type which are not &mut, assume their invariants.
        for param in 0..self.builder.fun_env.get_parameter_count() {
            let ty = self.builder.fun_env.get_local_type(param);
            if ty.is_mutable_reference() {
                // Invariant does not hold for mutable references.
                continue;
            }
            if ty.skip_reference().is_struct() {
                // Emit deep assume of the invariant.
                self.emit_data_invariant_for_temp(true, PropKind::Assume, param);
            }
        }
    }

    fn instrument_bytecode(&mut self, bc: Bytecode) {
        use Bytecode::*;
        use Operation::*;
        match bc {
            // Remove Unpack, we currently don't need it.
            Call(_, _, UnpackRef, ..) | Call(_, _, UnpackRefDeep, ..) => {}

            // Instructions which lead to asserting data invariants.
            Call(id, dests, Pack(mid, sid, targs), srcs, aa) => {
                let struct_temp = dests[0];
                self.builder
                    .emit(Call(id, dests, Pack(mid, sid, targs), srcs, aa));
                // Emit a shallow assert of the data invariant.
                self.emit_data_invariant_for_temp(false, PropKind::Assert, struct_temp);
            }
            Call(_, _, PackRef, srcs, _) => {
                // Emit a shallow assert of the data invariant.
                self.emit_data_invariant_for_temp(false, PropKind::Assert, srcs[0]);
            }
            Call(_, _, PackRefDeep, srcs, _) => {
                // Emit a deep assert of the data invariant.
                self.emit_data_invariant_for_temp(true, PropKind::Assert, srcs[0]);
            }

            // Instructions which lead to assuming data invariants.
            Call(id, dests, BorrowGlobal(mid, sid, targs), srcs, aa) => {
                let struct_temp = dests[0];
                self.builder
                    .emit(Call(id, dests, BorrowGlobal(mid, sid, targs), srcs, aa));
                // Emit deep assume of the data invariant.
                self.emit_data_invariant_for_temp(true, PropKind::Assume, struct_temp);
            }
            Call(id, dests, GetGlobal(mid, sid, targs), srcs, aa) => {
                let struct_temp = dests[0];
                self.builder
                    .emit(Call(id, dests, GetGlobal(mid, sid, targs), srcs, aa));
                // Emit deep assume of the data invariant.
                self.emit_data_invariant_for_temp(true, PropKind::Assume, struct_temp);
            }

            _ => self.builder.emit(bc),
        }
    }

    /// Emits a data invariant, shallow or deep, assume or assert, for the value in temporary.
    pub fn emit_data_invariant_for_temp(
        &mut self,
        deep: bool,
        kind: PropKind,
        struct_temp: TempIndex,
    ) {
        let (mid, sid, _) = self
            .builder
            .get_target()
            .get_local_type(struct_temp)
            .skip_reference()
            .require_struct();
        let struct_env = self.builder.global_env().get_module(mid).into_struct(sid);
        if deep {
            self.emit_data_invariant_deep(kind, &struct_env, &mut |_| struct_temp);
        } else {
            self.emit_data_invariant(kind, &struct_env, &mut |_| struct_temp);
        }
    }

    /// Emits a data invariant for the given struct hold in `struct_temp(_)`. Depending
    /// on `kind`, this will be an assert or assume.
    fn emit_data_invariant(
        &mut self,
        kind: PropKind,
        struct_env: &StructEnv<'_>,
        struct_temp_fun: &mut dyn FnMut(&mut FunctionDataBuilder) -> TempIndex,
    ) {
        use ast::Operation::*;
        use Exp::*;
        self.builder.set_next_debug_comment(format!(
            "data invariant for `{}`",
            struct_env.get_full_name_str()
        ));
        for cond in struct_env.get_spec().filter_kind(ConditionKind::Invariant) {
            // Rewrite the invariant expression, inserting `temp_exp` for the struct target.
            // By convention, selection from the target is represented as a `Select` operation with
            // an empty argument list. It is guaranteed that this uniquely identifies the
            // target, as any other `Select` will have exactly one argument.
            let struct_temp = struct_temp_fun(&mut self.builder);
            let temp_exp = self.builder.mk_local(struct_temp);
            let exp = cond.exp.clone().rewrite(&mut |e| match e {
                Call(id, oper @ Select(..), args) if args.is_empty() => {
                    (true, Call(id, oper, vec![temp_exp.clone()]))
                }
                _ => (false, e),
            });
            if kind == PropKind::Assert {
                self.builder.set_loc_and_vc_info(
                    cond.loc.clone(),
                    ConditionTag::Requires,
                    INVARIANT_FAILS_MESSAGE,
                );
            }
            self.builder.emit_with(|id| Bytecode::Prop(id, kind, exp));
        }
        self.builder.clear_next_debug_comment();
    }

    /// Emits a data invariant for this struct and all its transitive fields.
    fn emit_data_invariant_deep(
        &mut self,
        kind: PropKind,
        struct_env: &StructEnv<'_>,
        struct_temp_fun: &mut dyn FnMut(&mut FunctionDataBuilder) -> TempIndex,
    ) {
        self.emit_data_invariant(kind, struct_env, struct_temp_fun);
        for field_env in struct_env.get_fields() {
            let ty = field_env.get_type();
            if ty.is_struct() {
                let (mid, sid, targs) = ty.require_struct();
                let field_struct_env = self.builder.global_env().get_module(mid).into_struct(sid);
                let mut field_temp_opt: Option<TempIndex> = None;

                // Function which lazily creates a temporary which holds a selected field value.
                // This is lazy because we do not know whether we actually need it unless we have
                // descended into the field and found an invariant somewhere.
                let mut field_temp_fun = |builder: &mut FunctionDataBuilder| {
                    if let Some(temp) = field_temp_opt {
                        temp
                    } else {
                        let parent_temp = struct_temp_fun(builder);
                        let new_temp = builder.new_temp(ty.clone());
                        builder.emit_with(|id| {
                            Bytecode::Call(
                                id,
                                vec![new_temp],
                                Operation::GetField(
                                    struct_env.module_env.get_id(),
                                    struct_env.get_id(),
                                    targs.to_vec(),
                                    field_env.get_offset(),
                                ),
                                vec![parent_temp],
                                None,
                            )
                        });
                        field_temp_opt = Some(new_temp);
                        new_temp
                    }
                };
                self.emit_data_invariant_deep(kind, &field_struct_env, &mut field_temp_fun);
            }
        }
    }
}
