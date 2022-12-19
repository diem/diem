// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! A monomorphization processor for the elimination of generic types found in
//! expressions, i.e., rewriting a generic expression with type instantiations.
//!
//! This mono processor is a *local monomorphization* processor, it specializes
//! a generic proposition with types found in the enclosing function only. This
//! complements the *global monomorphization* processor in `mono_analysis.rs`
//! which, if schedule to run after this local mono pass, instantiate axioms
//! (and currently existential type quantifiers also) with information obtained
//! in the whole `GlobalEnv`.
//!
//! Local monomorphization can only be performed when two conditions hold:
//!
//! - all generic proposition are in the form of top-level universal type
//!   quantifiers, i.e., the proposition `P` must be in the form of
//!   `forall t1: type, t2: type, ... : expr<t1, t2, ...>` and there is no
//!   operation over `P` (e.g., `!P`, `P ==> Q`, etc. are not allowed).
//!   This is guaranteed by the generic invariant syntax.
//!
//! - the global invariant instrumentation pass places a global invariant
//!   `I` into all relevant instantiations of a function `F`. For example,
//!   if `I<T>` talks about a memory `S<T>` that is also going to be touched
//!   by `F<T>`, then `I<T>` must be instrumented in the generic version of
//!   `F` as well as in every instantiation of `F` (e.g., `F<bool>`,
//!   `F<u64>`, etc).
//!
//! If both conditions hold, we can run local monomorphization, i.e., assert
//! properties that are only relevant to the enclosing function only and
//! safely assume that the properties hold for all other types that are
//! not touched by the function.

use crate::{
    function_data_builder::FunctionDataBuilder,
    function_target::{FunctionData, FunctionTarget},
    function_target_pipeline::{FunctionTargetProcessor, FunctionTargetsHolder},
    stackless_bytecode::Bytecode,
    usage_analysis,
};

use move_model::{
    ast::{Exp, ExpData, MemoryLabel, Operation},
    exp_generator::ExpGenerator,
    model::{FunctionEnv, GlobalEnv, NodeId, QualifiedInstId, StructId},
    symbol::Symbol,
    ty::{Type, TypeUnificationAdapter, Variance},
};

use itertools::Itertools;
use std::collections::{BTreeMap, BTreeSet};

/// A context struct that holds/accumulates information during the monomorphization process.
struct MonoRewriter {
    // Collect all memory resource types that are used by this function.
    memory_inst_usage: BTreeSet<QualifiedInstId<StructId>>,
    // A map from memory label accessed from within the body of the quantifier
    // which needs to be specialized to the given instances in SaveMem instructions.
    mem_inst_by_label: BTreeMap<MemoryLabel, BTreeSet<QualifiedInstId<StructId>>>,
}

impl MonoRewriter {
    pub fn new(target: &FunctionTarget) -> Self {
        // the usage analysis result inherits from the generic function, therefore, we need to
        // instantiate the types if we are processing a verification variant which represents an
        // instantiated version of the generic function.
        let target_inst = target.data.get_type_instantiation(target.func_env);
        let memory_inst_usage = usage_analysis::get_used_memory_inst(target)
            .iter()
            .map(|mem| mem.instantiate_ref(&target_inst))
            .collect();
        Self {
            memory_inst_usage,
            mem_inst_by_label: BTreeMap::new(),
        }
    }

    pub fn run(&mut self, builder: &mut FunctionDataBuilder) {
        let code = std::mem::take(&mut builder.data.code);
        for bc in code {
            if let Bytecode::Prop(id, kind, exp) = bc {
                let exp = self.rewrite_generic_types(builder, exp);
                builder.emit(Bytecode::Prop(id, kind, exp));
            } else {
                builder.emit(bc);
            }
        }

        // rewrite the SaveMem bytecode
        let code = std::mem::take(&mut builder.data.code);
        for bc in code {
            match bc {
                Bytecode::SaveMem(id, label, mem) => {
                    if self.mem_inst_by_label.contains_key(&label) {
                        for inst in self.mem_inst_by_label.get(&label).unwrap() {
                            builder.emit(Bytecode::SaveMem(id, label, inst.to_owned()));
                        }
                    } else if !mem
                        .inst
                        .iter()
                        .any(|ty| ty.contains(&|t| matches!(t, Type::TypeLocal(_))))
                    {
                        // Only retain the SaveMem if it does not contain type locals.
                        // Such SaveMem's can result from zero expansions during quantifier
                        // elimination, and they are dead.
                        builder.emit(Bytecode::SaveMem(id, label, mem));
                    }
                }
                _ => builder.emit(bc),
            }
        }
    }

    fn rewrite_generic_types(&mut self, builder: &mut FunctionDataBuilder, exp: Exp) -> Exp {
        let env = builder.global_env();
        ExpData::rewrite(exp, &mut |e| {
            if e.is_generic(env) {
                // collect type generics
                let prop_insts = self.analyze_instantiation(env, &builder.data.type_args, &e);

                // expand the expressions with instantiations of type generics
                let mut expanded = vec![];
                for inst in &prop_insts {
                    let new_exp = self.eliminate_generic_types(env, &e, inst);
                    expanded.push(new_exp);
                }

                // compose the resulting list of expansions into a conjunction or disjunction.
                builder.set_loc(env.get_node_loc(e.node_id()));
                let combined_exp = builder
                    .mk_join_bool(Operation::And, expanded.into_iter())
                    .unwrap_or_else(|| builder.mk_bool_const(true));

                // marks that the expression IS re-written and the rewriter SHOULD NOT
                // descend into the sub-expressions.
                return Ok(combined_exp);
            }

            // marks that the expression IS NOT re-written and the rewriter SHOULD descend into the
            // sub-expressions for further processing.
            Err(e)
        })
    }

    // collect potential instantiations for this generic expression
    fn analyze_instantiation(
        &mut self,
        env: &GlobalEnv,
        inst: &BTreeMap<u16, Type>,
        exp: &Exp,
    ) -> Vec<BTreeMap<Symbol, Type>> {
        // holds possible instantiations per type local
        let mut prop_insts = BTreeMap::new();

        let exp_mems: BTreeSet<_> = exp
            .used_memory(env)
            .into_iter()
            .map(|(mem, _)| mem)
            .collect();

        for exp_mem in &exp_mems {
            for fun_mem in &self.memory_inst_usage {
                if exp_mem.module_id != fun_mem.module_id || exp_mem.id != fun_mem.id {
                    continue;
                }
                let adapter = TypeUnificationAdapter::new_vec(
                    &fun_mem.inst,
                    &exp_mem.inst,
                    /* treat_type_param_as_var */ false,
                    /* treat_type_local_as_var */ true,
                );
                let rel = adapter.unify(Variance::Allow, /* shallow_subst */ false);
                match rel {
                    None => continue,
                    Some((_, subst_rhs)) => {
                        for (k, v) in subst_rhs {
                            match k {
                                Type::TypeLocal(local_idx) => {
                                    let v_with_inst = match v {
                                        Type::TypeParameter(param_idx) => inst
                                            .get(&param_idx)
                                            .cloned()
                                            .unwrap_or(Type::TypeParameter(param_idx)),
                                        _ => v,
                                    };
                                    prop_insts
                                        .entry(local_idx)
                                        .or_insert_with(BTreeSet::new)
                                        .insert(v_with_inst);
                                }
                                _ => panic!("Only TypeLocal is expected in the substitution"),
                            }
                        }
                    }
                }
            }
        }

        // get cartesian product of all per-local instantiations
        let ty_locals: Vec<_> = prop_insts.keys().cloned().collect();
        let mut all_insts = vec![];
        for one_inst in prop_insts
            .values()
            .map(|tys| tys.iter())
            .multi_cartesian_product()
        {
            let map_view: BTreeMap<_, _> = ty_locals
                .iter()
                .zip(one_inst.into_iter())
                .map(|(s, t)| (*s, t.clone()))
                .collect();
            all_insts.push(map_view);
        }
        all_insts
    }

    // instantiate the generic expression with an instantiation
    fn eliminate_generic_types(
        &mut self,
        env: &GlobalEnv,
        exp: &Exp,
        inst: &BTreeMap<Symbol, Type>,
    ) -> Exp {
        // Create the effective proposition with type generics eliminated.
        let new_prop = exp.clone();
        let mut node_rewriter = |id: NodeId| {
            let node_ty = env.get_node_type(id);
            let mut new_node_ty = node_ty.clone();
            for (name, ty) in inst {
                new_node_ty = new_node_ty.replace_type_local(*name, ty.clone());
            }
            let node_inst = env.get_node_instantiation_opt(id);
            let new_node_inst = node_inst.clone().map(|i| {
                i.iter()
                    .map(|t| {
                        let mut new_t = t.clone();
                        for (name, ty) in inst {
                            new_t = new_t.replace_type_local(*name, ty.clone());
                        }
                        new_t
                    })
                    .collect_vec()
            });
            if node_ty != new_node_ty || node_inst != new_node_inst {
                let loc = env.get_node_loc(id);
                let new_id = env.new_node(loc, new_node_ty);
                if let Some(inst) = new_node_inst {
                    env.set_node_instantiation(new_id, inst);
                }
                Some(new_id)
            } else {
                None
            }
        };
        let inst_prop = ExpData::rewrite_node_id(new_prop, &mut node_rewriter);

        // Collect memory used by the expanded body. We need to rewrite SaveMem
        // instructions to point to the instantiated memory.
        for (qid, label_opt) in inst_prop.used_memory(env) {
            if let Some(label) = label_opt {
                self.mem_inst_by_label.entry(label).or_default().insert(qid);
            }
        }

        inst_prop
    }
}

/// This is the monomorphization processor that works on a function level.
///
/// It eliminates potential quantifiers over types by substituting those types with instantiations
/// that are found within the function being processed.
pub struct LocalMonoProcessor {}

impl LocalMonoProcessor {
    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }
}

impl FunctionTargetProcessor for LocalMonoProcessor {
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
        if !data.variant.is_verified() {
            // Only need to instrument if this is a verification variant
            return data;
        }

        // actual monomorphization logic encapsulated in the MonoAnalyzer
        let mut builder = FunctionDataBuilder::new(fun_env, data);

        // rewrite
        let target = builder.get_target();
        let mut rewriter = MonoRewriter::new(&target);
        rewriter.run(&mut builder);

        // done with the monomorphization transformation
        builder.data
    }

    fn name(&self) -> String {
        "prop_monomorphization".to_owned()
    }
}
