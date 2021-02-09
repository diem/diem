// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Transformation which injects data invariants into the bytecode.
//!
//! This transformation adds the data invariant to all occurrences of the `WellFormed(x)` call
//! which has been introduced by the spec instrumenter, by essentially transforming `WellFormed(x)`
//! into `WellFormed(x) && <data invariant>(x)`. The `WellFormed` expressions are maintained in the
//! output for processing by the backend, in case type assumptions needed to be added by the backend
//! (which depends on the compilation scheme). It also handles PackRef/PackRefDeep
//! instructions introduced by memory instrumentation, as well as the Pack instructions.

use crate::{
    function_data_builder::FunctionDataBuilder,
    function_target::FunctionData,
    function_target_pipeline::{FunctionTargetProcessor, FunctionTargetsHolder, FunctionVariant},
    options::{ProverOptions, PROVER_DEFAULT_OPTIONS},
    stackless_bytecode::{Bytecode, Operation, PropKind},
};

use move_model::{
    ast,
    ast::{ConditionKind, Exp, QuantKind, TempIndex},
    exp_rewriter::ExpRewriter,
    model::{ConditionTag, FunctionEnv, Loc, StructEnv},
    ty::Type,
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
        targets: &mut FunctionTargetsHolder,
        fun_env: &FunctionEnv<'_>,
        data: FunctionData,
    ) -> FunctionData {
        if fun_env.is_native() || fun_env.is_intrinsic() {
            // Nothing to do.
            return data;
        }
        let options = fun_env
            .module_env
            .env
            .get_extension::<ProverOptions>()
            .unwrap_or_else(|| &*PROVER_DEFAULT_OPTIONS);
        Instrumenter::run(options, targets, fun_env, data)
    }

    fn name(&self) -> String {
        "data_invariant_instrumenter".to_string()
    }
}

struct Instrumenter<'a> {
    _options: &'a ProverOptions,
    _targets: &'a mut FunctionTargetsHolder,
    builder: FunctionDataBuilder<'a>,
    for_verification: bool,
}

impl<'a> Instrumenter<'a> {
    fn run(
        options: &'a ProverOptions,
        targets: &'a mut FunctionTargetsHolder,
        fun_env: &FunctionEnv<'a>,
        data: FunctionData,
    ) -> FunctionData {
        // Function is instrumented for verification if this is the verification variant,
        // or if it is function with a friend which is verified in the friends context.
        let for_verification =
            data.variant == FunctionVariant::Verification || fun_env.has_friend();
        let builder = FunctionDataBuilder::new(fun_env, data);
        let mut instrumenter = Instrumenter {
            _options: options,
            _targets: targets,
            builder,
            for_verification,
        };
        instrumenter.instrument();
        instrumenter.builder.data
    }

    fn instrument(&mut self) {
        // Extract and clear current code
        let old_code = std::mem::take(&mut self.builder.data.code);

        // Instrument and generate new code
        for bc in old_code {
            self.instrument_bytecode(bc.clone());
        }
    }

    fn instrument_bytecode(&mut self, bc: Bytecode) {
        use Bytecode::*;
        use Operation::*;
        match bc {
            // Remove Unpack, we currently don't need it.
            Call(_, _, UnpackRef, ..) | Call(_, _, UnpackRefDeep, ..) => {}

            // Instructions which lead to asserting data invariants.
            Call(id, dests, Pack(mid, sid, targs), srcs, aa) if self.for_verification => {
                let struct_temp = dests[0];
                self.builder
                    .emit(Call(id, dests, Pack(mid, sid, targs), srcs, aa));
                // Emit a shallow assert of the data invariant.
                self.emit_data_invariant_for_temp(false, PropKind::Assert, struct_temp);
            }
            Call(_, _, PackRef, srcs, _) if self.for_verification => {
                // Emit a shallow assert of the data invariant.
                self.emit_data_invariant_for_temp(false, PropKind::Assert, srcs[0]);
            }
            Call(_, _, PackRefDeep, srcs, _) if self.for_verification => {
                // Emit a deep assert of the data invariant.
                self.emit_data_invariant_for_temp(true, PropKind::Assert, srcs[0]);
            }

            // Augment WellFormed calls in assumptions. Currently those cannot appear in assertions.
            // We leave the old WellFormed check for the backend to process any type related
            // assumptions.
            Prop(id, PropKind::Assume, exp) => {
                let mut rewriter = |e: Exp| {
                    if let Exp::Call(_, ast::Operation::WellFormed, args) = &e {
                        let inv = self.builder.mk_join_bool(
                            ast::Operation::And,
                            self.translate_invariant(true, args[0].clone())
                                .into_iter()
                                .map(|(_, e)| e),
                        );
                        let e = self
                            .builder
                            .mk_join_opt_bool(ast::Operation::And, Some(e), inv)
                            .unwrap();
                        (true, e)
                    } else {
                        (false, e)
                    }
                };
                let exp = exp.rewrite(&mut rewriter);
                self.builder.emit(Prop(id, PropKind::Assume, exp));
            }
            _ => self.builder.emit(bc),
        }
    }

    /// Emits a data invariant, shallow or deep, assume or assert, for the value in temporary.
    fn emit_data_invariant_for_temp(&mut self, deep: bool, kind: PropKind, temp: TempIndex) {
        let temp_exp = self.builder.mk_temporary(temp);
        for (loc, inv) in self.translate_invariant(deep, temp_exp) {
            self.builder.set_next_debug_comment(format!(
                "data invariant {}",
                loc.display(self.builder.global_env())
            ));
            if kind == PropKind::Assert {
                self.builder.set_loc_and_vc_info(
                    loc,
                    ConditionTag::Requires,
                    INVARIANT_FAILS_MESSAGE,
                );
            }
            self.builder.emit_with(|id| Bytecode::Prop(id, kind, inv));
        }
    }

    fn translate_invariant(&self, deep: bool, value: Exp) -> Vec<(Loc, Exp)> {
        let ty = self.builder.global_env().get_node_type(value.node_id());
        match ty.skip_reference() {
            Type::Struct(mid, sid, targs) => {
                let struct_env = self.builder.global_env().get_module(*mid).into_struct(*sid);
                self.translate_invariant_for_struct(deep, value, struct_env, targs)
            }
            Type::Vector(ety) => {
                // When dealing with a vector, we cannot maintain individual locations for
                // invariants. Instead we choose just one as a representative.
                // TODO(refactoring): we should use the spec block position instead.
                let mut loc = self.builder.global_env().unknown_loc();
                let quant = self.builder.mk_vector_quant_opt(
                    QuantKind::Forall,
                    value,
                    &*ety,
                    &mut |elem| {
                        let invs = self.translate_invariant(deep, elem);
                        if !invs.is_empty() {
                            loc = invs[0].0.clone();
                        }
                        self.builder
                            .mk_join_bool(ast::Operation::And, invs.into_iter().map(|(_, e)| e))
                    },
                );
                if let Some(e) = quant {
                    vec![(loc, e)]
                } else {
                    vec![]
                }
            }
            _ => vec![],
        }
    }

    fn translate_invariant_for_struct(
        &self,
        deep: bool,
        value: Exp,
        struct_env: StructEnv<'_>,
        targs: &[Type],
    ) -> Vec<(Loc, Exp)> {
        use ast::Operation::*;
        use Exp::*;

        // First generate a conjunction for all invariants on this struct.
        let mut result = vec![];
        for cond in struct_env.get_spec().filter_kind(ConditionKind::Invariant) {
            // Rewrite the invariant expression, inserting `value` for the struct target.
            // By convention, selection from the target is represented as a `Select` operation with
            // an empty argument list. It is guaranteed that this uniquely identifies the
            // target, as any other `Select` will have exactly one argument.
            let exp = cond.exp.clone().rewrite(&mut |e| match e {
                Call(id, oper @ Select(..), args) if args.is_empty() => {
                    (true, Call(id, oper, vec![value.clone()]))
                }
                _ => (false, e),
            });
            // Also need to instantiate any generics in the expression with targs.
            let mut replacer = |_, _| None;
            let mut rewriter =
                ExpRewriter::new(self.builder.global_env(), &mut replacer).set_type_args(targs);
            let exp = rewriter.rewrite(&exp);
            result.push((cond.loc.clone(), exp));
        }

        // If this is deep, recurse over all fields.
        if deep {
            for field_env in struct_env.get_fields() {
                let field_exp = self
                    .builder
                    .mk_field_select(&field_env, targs, value.clone());
                result.extend(self.translate_invariant(deep, field_exp));
            }
        }

        result
    }
}
