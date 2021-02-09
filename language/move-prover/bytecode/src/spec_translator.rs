// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module supports translations of specifications as found in the move-model to
//! expressions which can be used in assumes/asserts in bytecode.

use std::collections::{BTreeMap, BTreeSet};

use itertools::Itertools;

use move_model::{
    ast,
    ast::{Condition, ConditionKind, Exp, GlobalInvariant, LocalVarDecl, MemoryLabel, TempIndex},
    model::{FunctionEnv, Loc, NodeId, QualifiedId, SpecVarId, StructId},
    pragmas::{ABORTS_IF_IS_STRICT_PRAGMA, CONDITION_ABSTRACT_PROP, CONDITION_CONCRETE_PROP},
    symbol::Symbol,
    ty::Type,
};

use crate::function_data_builder::FunctionDataBuilder;
use move_model::{
    model::GlobalId,
    pragmas::{CONDITION_EXPORT_PROP, CONDITION_INJECTED_PROP},
    ty::PrimitiveType,
};

/// A helper which reduces specification conditions to assume/assert statements.
pub struct SpecTranslator<'a, 'b> {
    /// The builder for the function we are currently translating. Note this is not
    /// necessarily the same as the function for which we translate specs.
    pub builder: &'b mut FunctionDataBuilder<'a>,
    /// The function for which we translate specifications.
    pub fun_env: &'b FunctionEnv<'a>,
    /// The type instantiation of the function.
    type_args: &'b [Type],
    /// An optional substitution for parameters of the above function.
    param_substitution: Option<&'b [TempIndex]>,
    /// A substitution for return vales.
    ret_locals: &'b [TempIndex],
    /// A set of locals which are declared by outer block, lambda, or quant expressions.
    shadowed: Vec<BTreeSet<Symbol>>,
    /// The translated spec.
    result: TranslatedSpec,
    /// Whether we are translating in a post-state (ensures)
    at_post: bool,
}

/// Represents a translated spec.
#[derive(Default)]
pub struct TranslatedSpec {
    pub saved_memory: BTreeMap<QualifiedId<StructId>, MemoryLabel>,
    pub saved_spec_vars: BTreeMap<QualifiedId<SpecVarId>, MemoryLabel>,
    pub pre: Vec<(Loc, Exp)>,
    pub post: Vec<(Loc, Exp)>,
    pub aborts: Vec<(Loc, Exp, Option<Exp>)>,
    pub aborts_with: Vec<(Loc, Vec<Exp>)>,
    pub emits: Vec<(Loc, Exp, Exp, Option<Exp>)>,
    pub modifies: Vec<(Loc, Exp)>,
    pub invariants: Vec<(Loc, GlobalId, Exp)>,
}

impl TranslatedSpec {
    /// Creates a boolean expression which describes the overall abort condition. This is
    /// a disjunction of the individual abort conditions.
    pub fn aborts_condition(&self, builder: &FunctionDataBuilder<'_>) -> Option<Exp> {
        builder.mk_join_bool(
            ast::Operation::Or,
            self.aborts.iter().map(|(_, e, _)| e.clone()),
        )
    }

    /// Creates a boolean expression which describes the overall condition which constraints
    /// the abort code.
    ///
    /// Let (P1, C1)..(Pj, Cj) be aborts_if with a code, Pk..Pl aborts_if without a code, and the
    /// Cm..Cn standalone aborts codes from an aborts_with:
    ///
    ///  ```notrust
    ///   P1 && abort_code == C1 || .. || Pj && abort_code == Cj
    ///       || Pk || .. || Pl
    ///       || abort_code == Cm || .. || abort_code == Cn
    /// ```
    ///
    /// This characterizes the allowed value of the code. In the presence of aborts_if with code,
    /// whenever the aborts condition is true, the code must also be the specified ones. Notice
    /// that still allows any other member of the disjunction to make the overall condition true.
    /// Specifically, if someone specifies `aborts_if P with C1; aborts_with C2`, then even if
    /// P is true, C2 is allowed as an abort code.
    pub fn aborts_code_condition(
        &self,
        builder: &FunctionDataBuilder<'_>,
        actual_code: &Exp,
    ) -> Option<Exp> {
        let eq_code = |e: &Exp| builder.mk_eq(e.clone(), actual_code.clone());
        builder.mk_join_bool(
            ast::Operation::Or,
            self.aborts
                .iter()
                .map(|(_, exp, code)| {
                    builder
                        .mk_join_opt_bool(
                            ast::Operation::And,
                            Some(exp.clone()),
                            code.as_ref().map(|c| eq_code(c)),
                        )
                        .unwrap()
                })
                .chain(
                    self.aborts_with
                        .iter()
                        .map(|(_, codes)| codes.iter())
                        .flatten()
                        .map(|c| eq_code(c)),
                ),
        )
    }

    /// Returns true if there are any specs about the abort code.
    pub fn has_aborts_code_specs(&self) -> bool {
        !self.aborts_with.is_empty() || self.aborts.iter().any(|(_, _, c)| c.is_some())
    }

    /// Return an iterator of effective pre conditions.
    pub fn pre_conditions(
        &self,
        _builder: &FunctionDataBuilder<'_>,
    ) -> impl Iterator<Item = (Loc, Exp)> + '_ {
        self.pre.iter().cloned()
    }

    /// Returns a sequence of EventCheck expressions which verify the `emits` clauses of a
    /// function spec. While logically we could generate a single EventCheck, for better
    /// error reporting we construct incrementally multiple EventCheck expressions with some
    /// redundancy for each individual `emits, so we the see the exact failure at the right
    /// emit condition.
    pub fn emits_conditions(&self, builder: &FunctionDataBuilder<'_>) -> Vec<(Loc, Exp)> {
        let es_ty = Type::Primitive(PrimitiveType::EventStore);
        let mut result = vec![];
        for i in 0..self.emits.len() {
            let loc = self.emits[i].0.clone();
            let es = self.build_event_store(
                builder,
                builder.mk_call(&es_ty, ast::Operation::EmptyEventStore, vec![]),
                &self.emits[0..i + 1],
            );
            result.push((
                loc,
                builder.mk_bool_call(ast::Operation::CheckEventStore, vec![es]),
            ));
        }
        result
    }

    fn build_event_store(
        &self,
        builder: &FunctionDataBuilder<'_>,
        es: Exp,
        emits: &[(Loc, Exp, Exp, Option<Exp>)],
    ) -> Exp {
        if emits.is_empty() {
            es
        } else {
            let (_, event, handle, cond) = &emits[0];
            let mut args = vec![es, event.clone(), handle.clone()];
            if let Some(c) = cond {
                args.push(c.clone())
            }
            let es_ty = Type::Primitive(PrimitiveType::EventStore);
            let extend_exp = builder.mk_call(&es_ty, ast::Operation::ExtendEventStore, args);
            self.build_event_store(builder, extend_exp, &emits[1..])
        }
    }
}

impl<'a, 'b> SpecTranslator<'a, 'b> {
    /// Translates the specification of function `fun_env`. This can happen for a call of the
    /// function or for its definition (parameter `for_call`). This will process all the
    /// conditions found in the spec block of the function, dealing with references to `old(..)`,
    /// and creating respective memory/spec var saves. If `for_call` is true, abort conditions
    /// will be translated for the current state, otherwise they will be treated as in an `old`.
    /// and creating respective memory/spec var saves. It also allows to provide type arguments
    /// with which the specifications are instantiated, as well as a substitution for temporaries.
    /// The later two parameters are used to instantiate a function specification for a given
    /// call context.
    pub fn translate_fun_spec(
        for_call: bool,
        builder: &'b mut FunctionDataBuilder<'a>,
        fun_env: &'b FunctionEnv<'a>,
        type_args: &[Type],
        param_substitution: Option<&'b [TempIndex]>,
        ret_locals: &'b [TempIndex],
    ) -> TranslatedSpec {
        let mut translator = SpecTranslator {
            builder,
            fun_env,
            type_args,
            param_substitution,
            ret_locals,
            shadowed: Default::default(),
            result: Default::default(),
            at_post: false,
        };
        translator.translate_spec(for_call);
        translator.result
    }

    /// Translates a set of invariants. If there are any references to `old(...)` they
    /// will be rewritten and respective memory/spec var saves will be generated.
    pub fn translate_invariants(
        builder: &'b mut FunctionDataBuilder<'a>,
        invariants: impl Iterator<Item = &'b GlobalInvariant>,
    ) -> TranslatedSpec {
        let fun_env = builder.fun_env;
        let mut translator = SpecTranslator {
            builder,
            fun_env,
            type_args: &[],
            param_substitution: Default::default(),
            ret_locals: Default::default(),
            shadowed: Default::default(),
            result: Default::default(),
            at_post: false,
        };
        for inv in invariants {
            let exp = translator.translate_exp(&inv.cond, false);
            translator
                .result
                .invariants
                .push((inv.loc.clone(), inv.id, exp));
        }
        translator.result
    }

    fn translate_spec(&mut self, for_call: bool) {
        let fun_env = self.fun_env;
        let spec = fun_env.get_spec();

        // A function which determines whether a condition is applicable in the context, which
        // is `for_call` for the function being called, and `!for_call` if its verified.
        // If a condition has the `[abstract]` property, it will only be included for calls,
        // and if it has the `[concrete]` property only for verification. Also, conditions
        // which are injected from a schema are only included on call site if they are also
        // exported.
        let is_applicable = |cond: &&Condition| {
            let env = fun_env.module_env.env;
            let abstract_ = env
                .is_property_true(&cond.properties, CONDITION_ABSTRACT_PROP)
                .unwrap_or(false);
            let concrete = env
                .is_property_true(&cond.properties, CONDITION_CONCRETE_PROP)
                .unwrap_or(false);
            let injected = env
                .is_property_true(&cond.properties, CONDITION_INJECTED_PROP)
                .unwrap_or(false);
            let exported = env
                .is_property_true(&cond.properties, CONDITION_EXPORT_PROP)
                .unwrap_or(false);
            if for_call {
                (!injected || exported) && (abstract_ || !concrete)
            } else {
                concrete || !abstract_
            }
        };
        for cond in spec
            .filter_kind(ConditionKind::Requires)
            .filter(is_applicable)
        {
            self.at_post = false;
            let exp = self.translate_exp(&cond.exp, false);
            self.result.pre.push((cond.loc.clone(), exp));
        }

        let translate_aborts_in_old = !for_call;
        self.at_post = translate_aborts_in_old;
        for cond in spec
            .filter_kind(ConditionKind::AbortsIf)
            .filter(is_applicable)
        {
            let code_opt = if cond.additional_exps.is_empty() {
                None
            } else {
                Some(self.translate_exp(&cond.additional_exps[0], translate_aborts_in_old))
            };
            let exp = self.translate_exp(&cond.exp, translate_aborts_in_old);
            self.result.aborts.push((cond.loc.clone(), exp, code_opt));
        }

        self.at_post = translate_aborts_in_old;
        for cond in spec
            .filter_kind(ConditionKind::AbortsWith)
            .filter(is_applicable)
        {
            let codes = cond
                .all_exps()
                .map(|e| self.translate_exp(e, translate_aborts_in_old))
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

        self.at_post = true;
        for cond in spec
            .filter_kind(ConditionKind::Ensures)
            .filter(is_applicable)
        {
            let exp = self.translate_exp(&cond.exp, false);
            self.result.post.push((cond.loc.clone(), exp));
        }

        self.at_post = false;
        for cond in spec
            .filter_kind(ConditionKind::Modifies)
            .filter(is_applicable)
        {
            let exp = self.translate_exp(&cond.exp, false);
            self.result.modifies.push((cond.loc.clone(), exp));
        }

        self.at_post = true;
        for cond in spec.filter_kind(ConditionKind::Emits).filter(is_applicable) {
            let event_exp = self.translate_exp(&cond.exp, false);
            let handle_exp = self.translate_exp(&cond.additional_exps[0], false);
            let cond_exp = if cond.additional_exps.len() > 1 {
                Some(self.translate_exp(&cond.additional_exps[1], false))
            } else {
                None
            };
            self.result
                .emits
                .push((cond.loc.clone(), event_exp, handle_exp, cond_exp));
        }
    }

    fn translate_exp(&mut self, exp: &Exp, in_old: bool) -> Exp {
        use move_model::ast::{Exp::*, Operation::*};
        match exp {
            Temporary(node_id, idx) => {
                // Compute the effective name of parameter.
                let mut_ret_opt = self.get_ret_proxy(*idx);
                let effective_idx = match (in_old || !self.at_post, mut_ret_opt) {
                    (false, Some(mut_ret_idx)) => {
                        // We access a &mut outside of old context. Map it to the according return
                        // parameter. Notice the result of ret_locals[idx] needs to be interpreted
                        // in the builders function env, because we are substituting locals of the
                        // built function for parameters used by the function spec of this function.
                        self.ret_locals[mut_ret_idx]
                    }
                    _ => {
                        // We either access a regular parameter, or a &mut in old context, which is
                        // treated like a regular parameter.
                        if let Some(map) = self.param_substitution {
                            map[*idx]
                        } else {
                            *idx
                        }
                    }
                };
                // The type of this temporary might be different than the node's type w.r.t.
                // references. Create a new node id with the effective type.
                let effective_type = self
                    .builder
                    .data
                    .local_types
                    .get(effective_idx)
                    .expect("type expected");
                let loc = self.builder.global_env().get_node_loc(*node_id);
                let node_id = self
                    .builder
                    .global_env()
                    .new_node(loc, effective_type.clone());
                Temporary(self.instantiate(node_id), effective_idx)
            }
            SpecVar(node_id, mid, vid, None) if in_old => SpecVar(
                self.instantiate(*node_id),
                *mid,
                *vid,
                Some(self.save_spec_var(mid.qualified(*vid))),
            ),
            Call(node_id, Global(None), args) if in_old => {
                let args = self.translate_exp_vec(args, in_old);
                Call(
                    self.instantiate(*node_id),
                    Global(Some(
                        self.save_memory(self.builder.get_memory_of_node(*node_id)),
                    )),
                    args,
                )
            }
            Call(node_id, Exists(None), args) if in_old => {
                let args = self.translate_exp_vec(args, in_old);
                Call(
                    self.instantiate(*node_id),
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
                    self.instantiate(*node_id),
                    Function(*mid, *fid, Some(labels)),
                    self.translate_exp_vec(args, in_old),
                )
            }
            Call(_, Old, args) => self.translate_exp(&args[0], true),
            Call(node_id, Result(n), _) => {
                self.builder.set_loc_from_node(*node_id);
                self.builder.mk_temporary(self.ret_locals[*n])
            }
            Call(node_id, oper, args) => Call(
                self.instantiate(*node_id),
                oper.clone(),
                self.translate_exp_vec(args, in_old),
            ),
            Invoke(node_id, target, args) => {
                let target = self.translate_exp(target, in_old);
                Invoke(
                    self.instantiate(*node_id),
                    Box::new(target),
                    self.translate_exp_vec(args, in_old),
                )
            }
            Lambda(node_id, decls, body) => {
                let decls = self.translate_exp_decls(decls, in_old);
                self.shadowed.push(decls.iter().map(|d| d.name).collect());
                let res = Lambda(
                    self.instantiate(*node_id),
                    decls,
                    Box::new(self.translate_exp(body, in_old)),
                );
                self.shadowed.pop();
                res
            }
            Block(node_id, decls, body) => {
                let decls = self.translate_exp_decls(decls, in_old);
                self.shadowed.push(decls.iter().map(|d| d.name).collect());
                let res = Block(
                    self.instantiate(*node_id),
                    decls,
                    Box::new(self.translate_exp(body, in_old)),
                );
                self.shadowed.pop();
                res
            }
            Quant(node_id, kind, decls, triggers, where_opt, body) => {
                let decls = self.translate_exp_quant_decls(decls, in_old);
                self.shadowed
                    .push(decls.iter().map(|(d, _)| d.name).collect());
                let triggers = triggers
                    .iter()
                    .map(|tr| tr.iter().map(|e| self.translate_exp(e, in_old)).collect())
                    .collect();
                let where_opt = where_opt
                    .as_ref()
                    .map(|e| Box::new(self.translate_exp(e, in_old)));
                let body = Box::new(self.translate_exp(body, in_old));
                let res = Quant(
                    self.instantiate(*node_id),
                    *kind,
                    decls,
                    triggers,
                    where_opt,
                    body,
                );
                self.shadowed.pop();
                res
            }
            IfElse(node_id, cond, if_true, if_false) => IfElse(
                self.instantiate(*node_id),
                Box::new(self.translate_exp(cond, in_old)),
                Box::new(self.translate_exp(if_true, in_old)),
                Box::new(self.translate_exp(if_false, in_old)),
            ),
            _ => exp.clone(),
        }
    }

    /// Instantiate this expression node's type information with the provided type arguments.
    /// This returns a new node id for the instantiation, if one is needed, otherwise the given
    /// one.
    fn instantiate(&self, node_id: NodeId) -> NodeId {
        if self.type_args.is_empty() {
            node_id
        } else {
            let env = self.builder.global_env();
            let ty = env.get_node_type(node_id).instantiate(self.type_args);
            let inst = env
                .get_node_instantiation(node_id)
                .iter()
                .map(|t| t.instantiate(self.type_args))
                .collect_vec();
            let loc = env.get_node_loc(node_id);
            let node_id = env.new_node(loc, ty);
            env.set_node_instantiation(node_id, inst);
            node_id
        }
    }

    /// If the parameter is a &mut, return the proxy return parameter which was introduced by
    /// memory instrumentation for it.
    fn get_ret_proxy(&self, idx: TempIndex) -> Option<usize> {
        if self.fun_env.get_local_type(idx).is_mutable_reference() {
            let mut_ref_pos = (0..idx)
                .filter(|i| self.fun_env.get_local_type(*i).is_mutable_reference())
                .count();
            Some(usize::checked_add(self.fun_env.get_return_count(), mut_ref_pos).unwrap())
        } else {
            None
        }
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
