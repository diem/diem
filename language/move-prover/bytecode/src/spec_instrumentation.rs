// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Transformation injection specifications into the bytecode.

use crate::{
    function_data_builder::FunctionDataBuilder,
    function_target::FunctionData,
    function_target_pipeline::{FunctionTargetProcessor, FunctionTargetsHolder, FunctionVariant},
    livevar_analysis::LiveVarAnalysisProcessor,
    options::{ProverOptions, PROVER_DEFAULT_OPTIONS},
    reaching_def_analysis::ReachingDefProcessor,
    stackless_bytecode::{AssignKind, AttrId, Bytecode, Label, Operation, PropKind, TempIndex},
};
use itertools::Itertools;
use move_model::{
    ast,
    ast::{ConditionKind, Exp, LocalVarDecl, MemoryLabel},
    model::{ConditionTag, FunId, FunctionEnv, Loc, ModuleId, QualifiedId, SpecVarId, StructId},
    pragmas::{ABORTS_IF_IS_PARTIAL_PRAGMA, ABORTS_IF_IS_STRICT_PRAGMA},
    symbol::Symbol,
    ty::NUM_TYPE,
};
use std::collections::{BTreeMap, BTreeSet};

const REQUIRES_FAILS_MESSAGE: &str = "precondition does not hold at this call";
const ENSURES_FAILS_MESSAGE: &str = "post-condition does not hold";
const ABORTS_IF_FAILS_MESSAGE: &str = "function does not abort under this condition";
const ABORT_NOT_COVERED: &str = "abort not covered by any of the `aborts_if` clauses";
const WRONG_ABORTS_CODE: &str = "function aborts under this condition but with the wrong code";
const ABORTS_CODE_NOT_COVERED: &str =
    "abort code not covered by any of the `aborts_if` or `aborts_with` clauses";

pub struct SpecInstrumenter {}

impl SpecInstrumenter {
    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }
}

impl FunctionTargetProcessor for SpecInstrumenter {
    fn process(
        &self,
        targets: &mut FunctionTargetsHolder,
        fun_env: &FunctionEnv<'_>,
        mut data: FunctionData,
    ) -> FunctionData {
        if fun_env.is_native() || fun_env.is_intrinsic() {
            // Nothing to do
            return data;
        }

        // If verification is enabled for this function, create a new variant for verification
        // purposes.
        let options = fun_env
            .module_env
            .env
            .get_extension::<ProverOptions>()
            .unwrap_or_else(|| &*PROVER_DEFAULT_OPTIONS);
        if fun_env.should_verify(options.verify_scope) {
            // Create a clone of the function data, moving annotations
            // out of this data and into the clone.
            // TODO(refactoring): we cannot clone annotations because they use the Any type.
            //   In any case, function variants should be revisited once refactoring is
            //   finished and all the old boilerplate is removed.
            let annotations = std::mem::take(&mut data.annotations);
            let mut verification_data = data.clone();
            verification_data.annotations = annotations;
            verification_data = Instrumenter::run(
                targets,
                fun_env,
                FunctionVariant::Verification,
                verification_data,
            );
            targets.insert_target_data(
                &fun_env.get_qualified_id(),
                FunctionVariant::Verification,
                verification_data,
            );
        }

        // Instrument baseline variant.
        Instrumenter::run(targets, fun_env, FunctionVariant::Baseline, data)
    }

    fn name(&self) -> String {
        "spec_instrumenter".to_string()
    }
}

struct Instrumenter<'a> {
    variant: FunctionVariant,
    builder: FunctionDataBuilder<'a>,
    spec: TranslatedSpec,
    ret_locals: Vec<TempIndex>,
    ret_label: Label,
    can_return: bool,
    abort_local: TempIndex,
    abort_label: Label,
    can_abort: bool,
}

impl<'a> Instrumenter<'a> {
    fn run(
        targets: &mut FunctionTargetsHolder,
        fun_env: &FunctionEnv<'a>,
        variant: FunctionVariant,
        data: FunctionData,
    ) -> FunctionData {
        let mut builder = FunctionDataBuilder::new(fun_env, data);

        // Create label and locals for unified return exit point. We translate each `Ret(t..)`
        // instruction into `Assign(r.., t..); Jump(RetLab)`.
        let ret_locals = builder
            .data
            .return_types
            .clone()
            .into_iter()
            .map(|ty| builder.new_temp(ty))
            .collect_vec();
        let ret_label = builder.new_label();

        // Similarly create label and local for unified abort exit point. We translate `Abort(c)`
        // into `Assign(r, c); Jump(AbortLabel)`, as well as `Call(..)` into `Call(..);
        // OnAbort(AbortLabel, r)`. The `OnAbort` is a new instruction: if the last
        // call aborted, it stores the abort code in `r` and jumps to the label.
        let abort_local = builder.new_temp(NUM_TYPE.clone());
        let abort_label = builder.new_label();

        // Translate the specification. This deals with elimination of `old(..)` expressions,
        // as well as replaces `result_n` references with `ret_locals`.
        let spec = SpecTranslator::translate(&mut builder, fun_env, None, &ret_locals);

        // Create and run the instrumenter.
        let mut instrumenter = Instrumenter {
            variant,
            builder,
            spec,
            ret_locals,
            ret_label,
            can_return: false,
            abort_local,
            abort_label,
            can_abort: false,
        };
        instrumenter.instrument();

        // Run copy propagation (reaching definitions) and then assignment
        // elimination (live vars). This cleans up some redundancy created by
        // the instrumentation scheme.
        let mut data = instrumenter.builder.data;
        let reach_def = ReachingDefProcessor::new_no_preserve_user_locals();
        let live_vars = LiveVarAnalysisProcessor::new_no_annotate();
        data = reach_def.process(targets, fun_env, data);
        live_vars.process(targets, fun_env, data)
    }

    fn instrument(&mut self) {
        use Bytecode::*;
        use PropKind::*;

        // Extract and clear current code
        let old_code = std::mem::take(&mut self.builder.data.code);

        // Inject preconditions as assumes. This is done for all self.variant values.
        self.builder
            .set_loc(self.builder.fun_env.get_loc().at_start()); // reset to function level
        for (loc, exp) in &self.spec.pre {
            self.builder.set_loc(loc.clone());
            self.builder
                .emit_with(move |attr_id| Prop(attr_id, Assume, exp.clone()))
        }

        if self.variant == FunctionVariant::Verification {
            // For the verification variant, we generate post-conditions. Inject any state
            // save instructions needed for this.
            for (mem, label) in &self.spec.saved_memory {
                self.builder
                    .emit_with(|attr_id| SaveMem(attr_id, *label, *mem));
            }
            for (spec_var, label) in &self.spec.saved_spec_vars {
                self.builder
                    .emit_with(|attr_id| SaveSpecVar(attr_id, *label, *spec_var));
            }
        }

        // Instrument and generate new code
        for bc in old_code {
            self.instrument_bytecode(bc.clone());
        }

        // Generate return and abort blocks
        if self.can_return {
            self.generate_return_block();
        }
        if self.can_abort {
            self.generate_abort_block();
        }
    }

    fn instrument_bytecode(&mut self, bc: Bytecode) {
        use Bytecode::*;
        use Operation::*;
        match bc {
            Ret(id, results) => {
                self.builder.set_loc_from_attr(id);
                for (i, r) in self.ret_locals.clone().into_iter().enumerate() {
                    self.builder
                        .emit_with(|id| Assign(id, r, results[i], AssignKind::Move));
                }
                let ret_label = self.ret_label;
                self.builder.emit_with(|id| Jump(id, ret_label));
                self.can_return = true;
            }
            Abort(id, code) => {
                self.builder.set_loc_from_attr(id);
                let abort_local = self.abort_local;
                let abort_label = self.abort_label;
                self.builder
                    .emit_with(|id| Assign(id, abort_local, code, AssignKind::Move));
                self.builder.emit_with(|id| Jump(id, abort_label));
                self.can_abort = true;
            }
            Call(id, dests, Function(mid, sid, _), srcs) if self.is_opaque_fun(mid, sid) => {
                self.generate_opaque_call(id, dests, mid, sid, srcs);
            }
            Call(id, dests, oper, srcs) if oper.can_abort() => {
                self.builder.emit(Call(id, dests, oper, srcs));
                self.builder.set_loc_from_attr(id);
                let abort_label = self.abort_label;
                let abort_local = self.abort_local;
                self.builder
                    .emit_with(|id| OnAbort(id, abort_label, abort_local));
                self.can_abort = true;
            }
            _ => self.builder.emit(bc),
        }
    }

    fn is_opaque_fun(&self, mid: ModuleId, fid: FunId) -> bool {
        let fun_env = self.builder.global_env().get_module(mid).into_function(fid);
        fun_env.is_opaque()
    }

    fn generate_opaque_call(
        &mut self,
        id: AttrId,
        dests: Vec<TempIndex>,
        mid: ModuleId,
        fid: FunId,
        srcs: Vec<TempIndex>,
    ) {
        use Bytecode::*;
        use PropKind::*;

        let fun_env = self.builder.global_env().get_module(mid).into_function(fid);
        let mut spec = SpecTranslator::translate(&mut self.builder, &fun_env, Some(&srcs), &dests);

        self.builder.set_loc_from_attr(id);

        // Emit pre conditions as assertions.
        for (loc, cond) in std::mem::take(&mut spec.pre) {
            self.builder
                .set_loc_and_vc_info(loc, ConditionTag::Requires, REQUIRES_FAILS_MESSAGE);
            self.builder.emit_with(|id| Prop(id, Assert, cond));
        }

        // Emit all necessary state saves
        for (mem, label) in std::mem::take(&mut spec.saved_memory) {
            self.builder.emit_with(|id| SaveMem(id, label, mem));
        }
        for (var, label) in std::mem::take(&mut spec.saved_spec_vars) {
            self.builder.emit_with(|id| SaveSpecVar(id, label, var));
        }

        // Emit modifies
        for (_, modifies) in std::mem::take(&mut spec.modifies) {
            self.builder.emit_with(|id| Prop(id, Modifies, modifies));
        }

        // Translate the abort condition. We generate:
        //
        //   assume <temp> == <abort_cond>
        //   if <temp> goto abort_label
        //
        if let Some(abort_cond_temp) = self.generate_abort_opaque_cond(&spec) {
            let abort_label = self.abort_label;
            let no_abort_label = self.builder.new_label();
            self.builder
                .emit_with(|id| Branch(id, abort_label, no_abort_label, abort_cond_temp));
            self.builder.emit_with(|id| Label(id, no_abort_label));
            self.can_abort = true;
        }

        // Emit post conditions as assumptions.
        for (_, cond) in std::mem::take(&mut spec.post) {
            self.builder.emit_with(|id| Prop(id, Assume, cond));
        }
    }

    fn generate_abort_block(&mut self) {
        use Bytecode::*;
        // Set the location to the function and emit label.
        let fun_loc = self.builder.fun_env.get_loc().at_end();
        self.builder.set_loc(fun_loc);
        let abort_label = self.abort_label;
        self.builder.emit_with(|id| Label(id, abort_label));

        if self.variant == FunctionVariant::Verification {
            self.generate_abort_verify();
        }

        // Emit abort
        let abort_local = self.abort_local;
        self.builder.emit_with(|id| Abort(id, abort_local));
    }

    /// Generates verification conditions for abort block. Note that some of the complexity
    /// of this stems from that we aim to get as much detailed as possible diagnosis of
    /// verification failures, by splitting this into multiple asserts.
    ///
    /// Let (P1, C1)..(Pj, Cj) be aborts_if with a code, Pk..Pl aborts_if without a code, and the
    /// Cm..Cn standalone aborts codes from an aborts_with. We generate:
    ///
    ///  ```notrust
    ///   let P_with_code = P1 || .. || Pj
    ///   let P_without_code = Pk || .. || Pl
    ///   assert P_with_code || P_without_code    [if not partial]
    ///   assert P1 ==> abort_code == C1
    ///   ..
    ///   assert Pj ==> abort_code == Cj
    ///   assert !P_with_code ==> abort_code == Cm || .. || abort_code == Cn
    /// ```
    ///
    /// Each of the asserts has its own related ConditionInfo for failure reporting.
    fn generate_abort_verify(&mut self) {
        use Bytecode::*;
        use PropKind::*;

        let is_partial = self
            .builder
            .fun_env
            .is_pragma_true(ABORTS_IF_IS_PARTIAL_PRAGMA, || false);

        let actual_code = self.builder.mk_local(self.abort_local);
        let mut p_with_code = self.spec.aborts_if_with_code_disjunction(&self.builder);
        let p_without_code = self.spec.aborts_if_without_code_disjunction(&self.builder);
        let p_implies_code = self
            .spec
            .aborts_if_with_code_implication(&self.builder, &actual_code);
        let aborts_with_codes = self
            .spec
            .aborts_with_code_disjunction(&self.builder, &actual_code);

        if aborts_with_codes.is_some() && p_with_code.is_some() && !is_partial {
            // We need the p_with_code expression two times: first for asserting
            // the aborts condition, and second for asserting the aborts_with codes.
            // Save it into a temporary.
            p_with_code = p_with_code.map(|e| self.builder.emit_let(e).1);
        }

        // If !is_partial, assert the abort condition.
        if !is_partial {
            let p_overall = self.builder.mk_join_opt_bool(
                ast::Operation::Or,
                p_with_code.clone(),
                p_without_code,
            );
            if let Some(p) = p_overall {
                // TODO(wrwg): we need a location for the spec block of this function.
                //   The conditions don't give us a good indication because via
                //   schemas, they can come from anywhere. For now we use the
                //   function location.
                let loc = self.builder.fun_env.get_loc();
                self.builder
                    .set_loc_and_vc_info(loc, ConditionTag::Ensures, ABORT_NOT_COVERED);
                self.builder.emit_with(move |id| Prop(id, Assert, p));
            }
        }

        // Next assert implications for aborts_if with code.
        for (loc, implies) in p_implies_code {
            self.builder
                .set_loc_and_vc_info(loc, ConditionTag::Ensures, WRONG_ABORTS_CODE);
            self.builder.emit_with(move |id| Prop(id, Assert, implies));
        }

        // Finally emit aborts_with.
        if aborts_with_codes.is_some() {
            let not_p_with_code = p_with_code.map(|e| self.builder.mk_not(e));
            let aborts_with_cond = self
                .builder
                .mk_join_opt_bool(ast::Operation::Implies, not_p_with_code, aborts_with_codes)
                .unwrap();
            self.builder.set_loc_and_vc_info(
                self.builder.fun_env.get_loc().at_start(),
                ConditionTag::Ensures,
                ABORTS_CODE_NOT_COVERED,
            );
            self.builder
                .emit_with(move |id| Prop(id, Assert, aborts_with_cond));
        }
    }

    /// Generates an abort condition for assumption in opaque calls. In contrast
    /// to `generate_abort_verify`, this function does not need to consider failure
    /// reporting. We generate (see `generate_abort_if` for definitions):
    ///
    ///  ```notrust
    ///   let P_with_code = P1 || .. || Pj
    ///   let P_without_code = Pk || .. || Pl
    ///   let result = (P_with_code || P_without_code)
    ///             && (P1 ==> abort_code == C1)
    ///             ..
    ///             && (Pj ==> abort_code == Cj)
    ///             && (!P_with_code ==> abort_code == Cm || .. || abort_code == Cn)
    /// ```
    ///
    /// `result` is stored in a temporary which is returned and can be used to branch
    /// under this condition.
    fn generate_abort_opaque_cond(&mut self, spec: &TranslatedSpec) -> Option<TempIndex> {
        // TODO(refactoring): we expect that opaque functions are `aborts_if_is_partial`. Need
        //   to check whether we check this in the frontend.
        let actual_code = self.builder.mk_local(self.abort_local);
        let mut p_with_code = spec.aborts_if_with_code_disjunction(&self.builder);
        let p_without_code = spec.aborts_if_without_code_disjunction(&self.builder);
        let p_implies_code = spec.aborts_if_with_code_implication(&self.builder, &actual_code);
        let abort_with_codes = spec.aborts_with_code_disjunction(&self.builder, &actual_code);

        if abort_with_codes.is_some() && p_with_code.is_some() {
            // We need the p_with_code expression two times: first for the
            // aborts condition, and second for the aborts_with codes.
            // Save it into a temporary.
            p_with_code = p_with_code.map(|e| self.builder.emit_let(e).1);
        }

        let mut p_overall =
            self.builder
                .mk_join_opt_bool(ast::Operation::Or, p_with_code.clone(), p_without_code);

        // Next add implications for aborts_if with code.
        for (_, implies) in p_implies_code {
            p_overall =
                self.builder
                    .mk_join_opt_bool(ast::Operation::And, p_overall, Some(implies));
        }

        // Finally add aborts_with.
        if abort_with_codes.is_some() {
            let not_p_with_code = p_with_code.map(|e| self.builder.mk_not(e));
            let aborts_with_cond = self.builder.mk_join_opt_bool(
                ast::Operation::Implies,
                not_p_with_code,
                abort_with_codes,
            );
            p_overall =
                self.builder
                    .mk_join_opt_bool(ast::Operation::And, p_overall, aborts_with_cond);
        }

        if let Some(p) = p_overall {
            Some(self.builder.emit_let(p).0)
        } else {
            None
        }
    }

    fn generate_return_block(&mut self) {
        use Bytecode::*;
        use PropKind::*;

        // Set the location to the function and emit label.
        self.builder
            .set_loc(self.builder.fun_env.get_loc().at_end());
        let ret_label = self.ret_label;
        self.builder.emit_with(|id| Label(id, ret_label));

        if self.variant == FunctionVariant::Verification {
            // Emit the negation of all aborts conditions.
            for (loc, abort_cond, _) in &self.spec.aborts {
                self.builder.set_loc_and_vc_info(
                    loc.clone(),
                    ConditionTag::Ensures,
                    ABORTS_IF_FAILS_MESSAGE,
                );
                let exp = self.builder.mk_not(abort_cond.clone());
                self.builder.emit_with(|id| Prop(id, Assert, exp))
            }

            // Emit all post-conditions which must hold as we do not abort.
            for (loc, cond) in &self.spec.post {
                self.builder.set_loc_and_vc_info(
                    loc.clone(),
                    ConditionTag::Ensures,
                    ENSURES_FAILS_MESSAGE,
                );
                self.builder
                    .emit_with(move |id| Prop(id, Assert, cond.clone()))
            }
        }

        // Emit return
        let ret_locals = self.ret_locals.clone();
        self.builder.emit_with(move |id| Ret(id, ret_locals))
    }
}

/// A helper which reduces specification conditions to assume/assert statements.
struct SpecTranslator<'a, 'b> {
    /// The builder for the function we are currently translating. Note this is not
    /// necessarily the same as the function for which we translate specs.
    builder: &'b mut FunctionDataBuilder<'a>,
    /// The function for which we translate specifications.
    fun_env: &'b FunctionEnv<'a>,
    /// An optional substitution for parameters of the above function.
    param_locals: Option<&'b [TempIndex]>,
    /// A substitution for return vales.
    ret_locals: &'b [TempIndex],
    /// A set of locals which are declared by outer block, lambda, or quant expressions.
    shadowed: Vec<BTreeSet<Symbol>>,
    /// The translated spec.
    result: TranslatedSpec,
}

/// Represents a translated spec.
#[derive(Default)]
struct TranslatedSpec {
    saved_memory: BTreeMap<QualifiedId<StructId>, MemoryLabel>,
    saved_spec_vars: BTreeMap<QualifiedId<SpecVarId>, MemoryLabel>,
    pre: Vec<(Loc, Exp)>,
    post: Vec<(Loc, Exp)>,
    aborts: Vec<(Loc, Exp, Option<Exp>)>,
    aborts_with: Vec<(Loc, Vec<Exp>)>,
    modifies: Vec<(Loc, Exp)>,
}

impl TranslatedSpec {
    /// Creates a disjunction of all abort conditions which come with a code.
    fn aborts_if_with_code_disjunction(&self, builder: &FunctionDataBuilder<'_>) -> Option<Exp> {
        builder.mk_join_bool(
            ast::Operation::Or,
            self.aborts
                .iter()
                .filter_map(|(_, e, c)| if c.is_some() { Some(e.clone()) } else { None }),
        )
    }

    /// Creates a disjunction of all abort conditions which come without a code.
    fn aborts_if_without_code_disjunction(&self, builder: &FunctionDataBuilder<'_>) -> Option<Exp> {
        builder.mk_join_bool(
            ast::Operation::Or,
            self.aborts
                .iter()
                .filter_map(|(_, e, c)| if c.is_none() { Some(e.clone()) } else { None }),
        )
    }

    /// Creates a list of implications that if an aborts condition with a code holds, the
    /// abort code must have the expected value.
    fn aborts_if_with_code_implication(
        &self,
        builder: &FunctionDataBuilder<'_>,
        actual_code: &Exp,
    ) -> Vec<(Loc, Exp)> {
        self.aborts
            .iter()
            .filter_map(|(_, e, c)| {
                if let Some(expected_code) = c {
                    let loc = builder.global_env().get_node_loc(expected_code.node_id());
                    let matches = builder.mk_eq(actual_code.clone(), expected_code.clone());
                    Some((loc, builder.mk_implies(e.clone(), matches)))
                } else {
                    None
                }
            })
            .collect_vec()
    }

    /// Creates a disjunction that the actual aborts code must be one of the codes specified
    /// with aborts_with.
    fn aborts_with_code_disjunction(
        &self,
        builder: &FunctionDataBuilder<'_>,
        actual_code: &Exp,
    ) -> Option<Exp> {
        let codes = self
            .aborts_with
            .iter()
            .map(|(_, v)| v.iter())
            .flatten()
            .cloned();
        let equalities =
            codes.map(|expected_code| builder.mk_eq(actual_code.clone(), expected_code));
        builder.mk_join_bool(ast::Operation::Or, equalities)
    }
}

impl<'a, 'b> SpecTranslator<'a, 'b> {
    fn translate(
        builder: &'b mut FunctionDataBuilder<'a>,
        fun_env: &'b FunctionEnv<'a>,
        param_locals: Option<&'b [TempIndex]>,
        ret_locals: &'b [TempIndex],
    ) -> TranslatedSpec {
        let mut translator = SpecTranslator {
            builder,
            fun_env,
            param_locals,
            ret_locals,
            shadowed: Default::default(),
            result: Default::default(),
        };
        translator.translate_spec();
        translator.result
    }

    fn translate_spec(&mut self) {
        let fun_env = self.fun_env;
        let spec = fun_env.get_spec();

        for cond in spec.filter_kind(ConditionKind::Requires) {
            self.result.pre.push((cond.loc.clone(), cond.exp.clone()));
        }

        for cond in spec.filter_kind(ConditionKind::AbortsIf) {
            let code_opt = if cond.additional_exps.is_empty() {
                None
            } else {
                Some(self.translate_exp(&cond.additional_exps[0], true))
            };
            let exp = self.translate_exp(&cond.exp, true);
            self.result.aborts.push((cond.loc.clone(), exp, code_opt));
        }

        for cond in spec.filter_kind(ConditionKind::AbortsWith) {
            let codes = cond
                .all_exps()
                .map(|e| self.translate_exp(e, true))
                .collect_vec();
            self.result.aborts_with.push((cond.loc.clone(), codes));
        }

        // If there are no aborts_if and aborts_with, and the pragma `aborts_if_is_strict` is set,
        // add an implicit aborts_if false.
        if self.result.aborts.is_empty()
            && self.result.aborts_with.is_empty()
            && self
                .fun_env
                .is_pragma_true(ABORTS_IF_IS_STRICT_PRAGMA, || false)
        {
            self.result.aborts.push((
                self.fun_env.get_loc().at_end(),
                self.builder.mk_bool_const(false),
                None,
            ));
        }

        for cond in spec.filter_kind(ConditionKind::Ensures) {
            let exp = self.translate_exp(&cond.exp, false);
            self.result.post.push((cond.loc.clone(), exp));
        }

        for cond in spec.filter_kind(ConditionKind::Modifies) {
            let exp = self.translate_exp(&cond.exp, false);
            self.result.modifies.push((cond.loc.clone(), exp));
        }
    }

    fn translate_exp(&mut self, exp: &Exp, in_old: bool) -> Exp {
        use ast::Operation::*;
        use Exp::*;
        match exp {
            LocalVar(node_id, name) if !self.is_parameter(name) => {
                // Compute the effective name of parameter.
                let (idx, mut_ret_opt) = self.get_param_index_and_ret_proxy(*name);
                let effective_name = match (in_old, mut_ret_opt) {
                    (false, Some(mut_ret_idx)) => {
                        // We access a &mut outside of old context. Map it to the according return
                        // parameter. Notice the result of ret_locals[idx] needs to be interpreted
                        // in the builders function env, because we are substituting locals of the
                        // built function for parameters used by the function spec of this function.
                        self.builder
                            .fun_env
                            .get_local_name(self.ret_locals[mut_ret_idx])
                    }
                    _ => {
                        // We either access a regular parameter, or a &mut in old context, which is
                        // treated like a regular parameter.
                        if let Some(map) = self.param_locals {
                            // If there is parameter mapping, apply it.
                            self.builder.fun_env.get_local_name(map[idx])
                        } else {
                            *name
                        }
                    }
                };
                LocalVar(*node_id, effective_name)
            }
            SpecVar(node_id, mid, vid, None) if in_old => SpecVar(
                *node_id,
                *mid,
                *vid,
                Some(self.save_spec_var(mid.qualified(*vid))),
            ),
            Call(node_id, Global(None), args) if in_old => {
                let args = self.translate_exp_vec(args, in_old);
                Call(
                    *node_id,
                    Global(Some(
                        self.save_memory(self.builder.get_memory_of_node(*node_id)),
                    )),
                    args,
                )
            }
            Call(node_id, Exists(None), args) if in_old => {
                let args = self.translate_exp_vec(args, in_old);
                Call(
                    *node_id,
                    Exists(Some(
                        self.save_memory(self.builder.get_memory_of_node(*node_id)),
                    )),
                    args,
                )
            }
            Call(node_id, Function(mid, fid, None), args) if in_old => {
                let (used_memory, used_spec_vars) = {
                    let module_env = self.builder.global_env().get_module(*mid);
                    let decl = module_env.get_spec_fun(*fid);
                    // Unfortunately, the below clones are necessary, as we cannot borrow decl
                    // and at the same time mutate self later.
                    (decl.used_memory.clone(), decl.used_spec_vars.clone())
                };
                let mut labels = vec![];
                for mem in used_memory {
                    labels.push(self.save_memory(mem));
                }
                for var in used_spec_vars {
                    labels.push(self.save_spec_var(var));
                }
                Call(
                    *node_id,
                    Function(*mid, *fid, Some(labels)),
                    self.translate_exp_vec(args, in_old),
                )
            }
            Call(_, Old, args) => self.translate_exp(&args[0], true),
            Call(node_id, Result(n), _) => {
                self.builder.set_loc_from_node(*node_id);
                self.builder.mk_local(self.ret_locals[*n])
            }
            Call(node_id, oper, args) => {
                Call(*node_id, oper.clone(), self.translate_exp_vec(args, in_old))
            }
            Invoke(node_id, target, args) => {
                let target = self.translate_exp(target, in_old);
                Invoke(
                    *node_id,
                    Box::new(target),
                    self.translate_exp_vec(args, in_old),
                )
            }
            Lambda(node_id, decls, body) => {
                let decls = self.translate_exp_decls(decls, in_old);
                self.shadowed.push(decls.iter().map(|d| d.name).collect());
                let res = Lambda(*node_id, decls, Box::new(self.translate_exp(body, in_old)));
                self.shadowed.pop();
                res
            }
            Block(node_id, decls, body) => {
                let decls = self.translate_exp_decls(decls, in_old);
                self.shadowed.push(decls.iter().map(|d| d.name).collect());
                let res = Block(*node_id, decls, Box::new(self.translate_exp(body, in_old)));
                self.shadowed.pop();
                res
            }
            Quant(node_id, kind, decls, where_opt, body) => {
                let decls = self.translate_exp_quant_decls(decls, in_old);
                self.shadowed
                    .push(decls.iter().map(|(d, _)| d.name).collect());
                let where_opt = where_opt
                    .as_ref()
                    .map(|e| Box::new(self.translate_exp(e, in_old)));
                let body = Box::new(self.translate_exp(body, in_old));
                let res = Quant(*node_id, *kind, decls, where_opt, body);
                self.shadowed.pop();
                res
            }
            IfElse(node_id, cond, if_true, if_false) => IfElse(
                *node_id,
                Box::new(self.translate_exp(cond, in_old)),
                Box::new(self.translate_exp(if_true, in_old)),
                Box::new(self.translate_exp(if_false, in_old)),
            ),
            _ => exp.clone(),
        }
    }

    /// Returns true if the local name is shadowed, i.e. declared in a block, lambda, or quant
    /// instead of being a function parameter.
    fn is_parameter(&self, name: &Symbol) -> bool {
        self.shadowed.iter().any(|s| s.contains(name))
    }

    /// Get index of a parameter without having access to FunctionData (we do not have this access
    /// because we also need to generate specs for called functions, not just the currently
    /// processed one). Also, if the local is a &mut, return the proxy return parameter which
    /// was introduced by memory instrumentation for it.
    /// TODO(wrwg): to be removed after refactoring and once we switch to indices for locals,
    ///   as well as made memory instrumentation explicit in rewritten specs.
    fn get_param_index_and_ret_proxy(&self, name: Symbol) -> (TempIndex, Option<usize>) {
        let mut mut_ref_count = 0;
        for i in 0..self.fun_env.get_parameter_count() {
            let is_mut_ref = self.fun_env.get_local_type(i).is_mutable_reference();
            if name == self.fun_env.get_local_name(i) {
                if is_mut_ref {
                    return (i, Some(self.fun_env.get_return_count() + mut_ref_count));
                } else {
                    return (i, None);
                }
            }
            if is_mut_ref {
                mut_ref_count += 1;
            }
        }
        panic!(
            "cannot determine index of parameter `{}` in `{}::{}`",
            name.display(self.fun_env.symbol_pool()),
            self.fun_env
                .module_env
                .get_name()
                .display(self.fun_env.symbol_pool()),
            self.fun_env.get_name().display(self.fun_env.symbol_pool()),
        )
    }

    fn translate_exp_vec(&mut self, exps: &[Exp], in_old: bool) -> Vec<Exp> {
        exps.iter()
            .map(|e| self.translate_exp(e, in_old))
            .collect_vec()
    }

    fn translate_exp_decls(&mut self, decls: &[LocalVarDecl], in_old: bool) -> Vec<LocalVarDecl> {
        decls
            .iter()
            .map(|LocalVarDecl { id, name, binding }| LocalVarDecl {
                id: *id,
                name: *name,
                binding: binding.as_ref().map(|e| self.translate_exp(e, in_old)),
            })
            .collect_vec()
    }

    fn translate_exp_quant_decls(
        &mut self,
        decls: &[(LocalVarDecl, Exp)],
        in_old: bool,
    ) -> Vec<(LocalVarDecl, Exp)> {
        decls
            .iter()
            .map(|(LocalVarDecl { id, name, .. }, exp)| {
                (
                    LocalVarDecl {
                        id: *id,
                        name: *name,
                        binding: None,
                    },
                    self.translate_exp(exp, in_old),
                )
            })
            .collect_vec()
    }

    fn save_spec_var(&mut self, qid: QualifiedId<SpecVarId>) -> MemoryLabel {
        let builder = &mut self.builder;
        *self
            .result
            .saved_spec_vars
            .entry(qid)
            .or_insert_with(|| builder.global_env().new_global_id())
    }

    fn save_memory(&mut self, qid: QualifiedId<StructId>) -> MemoryLabel {
        let builder = &mut self.builder;
        *self
            .result
            .saved_memory
            .entry(qid)
            .or_insert_with(|| builder.global_env().new_global_id())
    }
}
