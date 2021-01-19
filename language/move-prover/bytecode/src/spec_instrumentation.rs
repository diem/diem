// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Transformation injection specifications into the bytecode.

use crate::{
    function_data_builder::FunctionDataBuilder,
    function_target::FunctionData,
    function_target_pipeline::{FunctionTargetProcessor, FunctionTargetsHolder, FunctionVariant},
    livevar_analysis::LiveVarAnalysisProcessor,
    reaching_def_analysis::ReachingDefProcessor,
    stackless_bytecode::{AssignKind, AttrId, Bytecode, Label, Operation, PropKind, TempIndex},
};
use itertools::Itertools;
use move_model::{
    ast,
    ast::{ConditionKind, Exp, LocalVarDecl, MemoryLabel},
    model::{
        FunId, FunctionEnv, Loc, ModuleId, QualifiedId, SpecVarId, StructId, VerificationScope,
    },
    symbol::Symbol,
    ty::{BOOL_TYPE, NUM_TYPE},
};
use std::collections::BTreeMap;

pub struct SpecInstrumenterOptions {
    pub verification_scope: VerificationScope,
}

pub struct SpecInstrumenter {
    options: SpecInstrumenterOptions,
}

impl SpecInstrumenter {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            options: SpecInstrumenterOptions {
                verification_scope: VerificationScope::All,
            },
        })
    }
}

impl FunctionTargetProcessor for SpecInstrumenter {
    fn process(
        &self,
        targets: &mut FunctionTargetsHolder,
        fun_env: &FunctionEnv<'_>,
        data: FunctionData,
    ) -> FunctionData {
        if fun_env.is_native() || fun_env.is_intrinsic() {
            // Nothing to do
            return data;
        }

        // If verification is enabled for this function, create a new variant for verification
        // purposes.
        if fun_env.should_verify(self.options.verification_scope) {
            let verification_data = Instrumenter::run(
                targets,
                fun_env,
                FunctionVariant::Verification,
                data.clone(),
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
            abort_local,
            abort_label,
            can_abort: false,
        };
        instrumenter.instrument();

        // Run copy propagation (reaching definitions) and then assignment
        // elimination (live vars). This cleans up some redundancy created by
        // the instrumentation scheme.
        let mut data = instrumenter.builder.data;
        let reach_def = ReachingDefProcessor::new_no_preserve_proxies();
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
        self.builder.set_loc(self.builder.fun_env.get_loc()); // reset to function level
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
            self.instrument_bytecode(bc)
        }

        // Generate return and abort blocks
        self.generate_return_block();
        if self.can_abort || !self.spec.aborts_with.is_empty() || !self.spec.aborts.is_empty() {
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
            self.builder.set_loc(loc);
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
        if let Some(abort_cond) = self.generate_abort_cond(&spec) {
            let abort_cond_temp = self.builder.new_temp(BOOL_TYPE.clone());
            let abort_assumption = self.builder.mk_bool_call(
                ast::Operation::Eq,
                vec![self.builder.mk_local(abort_cond_temp), abort_cond],
            );
            let abort_label = self.abort_label;
            let no_abort_label = self.builder.new_label();
            self.builder
                .emit_with(move |id| Prop(id, Assume, abort_assumption));
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
        use PropKind::*;

        // Set the location to the function and emit label.
        self.builder.set_loc(self.builder.fun_env.get_loc());
        let abort_label = self.abort_label;
        self.builder.emit_with(|id| Label(id, abort_label));

        if self.variant == FunctionVariant::Verification {
            if let Some(cond) = self.generate_abort_cond(&self.spec) {
                self.builder.emit_with(move |id| Prop(id, Assert, cond));
            }
        }

        // Emit abort
        let abort_local = self.abort_local;
        self.builder.emit_with(|id| Abort(id, abort_local));
    }

    fn generate_abort_cond(&self, spec: &TranslatedSpec) -> Option<Exp> {
        // Create a disjunction of the form:
        //
        //        P1 && code == C1
        //     || ..
        //     || Pj && code == Cj
        //     || Pk
        //     || ..
        //     || Pl
        //     || code == Cm
        //     || ..
        //     || code == Cn
        //
        // Here the P1..Pj are aborts_if with a code, the Pk..Pl aborts_if
        // without a code, and the Cm..Cn standalone aborts codes from an
        // aborts_with.
        let code_exp = self.builder.mk_local(self.abort_local);
        let mut abort_conds = spec
            .aborts
            .iter()
            .map(|(_, e, c)| {
                let mut prop = e.clone();
                if let Some(c) = c {
                    let code_eq = self
                        .builder
                        .mk_bool_call(ast::Operation::Eq, vec![c.clone(), code_exp.clone()]);
                    prop = self
                        .builder
                        .mk_bool_call(ast::Operation::And, vec![prop, code_eq])
                };
                prop
            })
            .chain(
                spec.aborts_with
                    .iter()
                    .map(|(_, codes)| {
                        codes.iter().map(|c| {
                            self.builder
                                .mk_bool_call(ast::Operation::Eq, vec![c.clone(), code_exp.clone()])
                        })
                    })
                    .flatten(),
            )
            .peekable();
        if abort_conds.peek().is_some() {
            let disjunction = self.builder.mk_join_bool(ast::Operation::Or, abort_conds);
            Some(disjunction)
        } else {
            None
        }
    }

    fn generate_return_block(&mut self) {
        use Bytecode::*;
        use PropKind::*;

        // Set the location to the function and emit label.
        self.builder.set_loc(self.builder.fun_env.get_loc());
        let ret_label = self.ret_label;
        self.builder.emit_with(|id| Label(id, ret_label));

        if self.variant == FunctionVariant::Verification {
            // Emit the negation of all aborts conditions. If we reach the return,
            // those negations must hold.
            for (loc, abort_cond, _) in &self.spec.aborts {
                self.builder.set_loc(loc.clone());
                let exp = self.builder.mk_not(abort_cond.clone());
                self.builder.emit_with(|id| Prop(id, Assert, exp))
            }

            // Emit all post-conditions which must hold as we do not abort.
            for (loc, cond) in &self.spec.post {
                self.builder.set_loc(loc.clone());
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
            LocalVar(node_id, name) => {
                // Compute the effective name of a local.
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
                let args = self.eliminate_old_vec(args, in_old);
                Call(
                    *node_id,
                    Global(Some(
                        self.save_memory(self.builder.get_memory_of_node(*node_id)),
                    )),
                    args,
                )
            }
            Call(node_id, Exists(None), args) if in_old => {
                let args = self.eliminate_old_vec(args, in_old);
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
                    self.eliminate_old_vec(args, in_old),
                )
            }
            Call(_, Old, args) => self.translate_exp(&args[0], true),
            Call(node_id, Result(n), _) => {
                self.builder.set_loc_from_node(*node_id);
                self.builder.mk_local(self.ret_locals[*n])
            }
            Call(node_id, oper, args) => {
                Call(*node_id, oper.clone(), self.eliminate_old_vec(args, in_old))
            }
            Invoke(node_id, target, args) => {
                let target = self.translate_exp(target, in_old);
                Invoke(
                    *node_id,
                    Box::new(target),
                    self.eliminate_old_vec(args, in_old),
                )
            }
            Lambda(node_id, decls, body) => {
                let decls = self.eliminate_old_decls(decls, in_old);
                Lambda(*node_id, decls, Box::new(self.translate_exp(body, in_old)))
            }
            Block(node_id, decls, body) => {
                let decls = self.eliminate_old_decls(decls, in_old);
                Block(*node_id, decls, Box::new(self.translate_exp(body, in_old)))
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
        panic!("cannot determine index of local")
    }

    fn eliminate_old_vec(&mut self, exps: &[Exp], in_old: bool) -> Vec<Exp> {
        exps.iter()
            .map(|e| self.translate_exp(e, in_old))
            .collect_vec()
    }

    fn eliminate_old_decls(&mut self, decls: &[LocalVarDecl], in_old: bool) -> Vec<LocalVarDecl> {
        decls
            .iter()
            .map(|LocalVarDecl { id, name, binding }| LocalVarDecl {
                id: *id,
                name: *name,
                binding: binding.as_ref().map(|e| self.translate_exp(e, in_old)),
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
