// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! A monomorphization processor for elimination of type quantification (`forall t: type:: P`)

use crate::{
    function_data_builder::FunctionDataBuilder,
    function_target::{FunctionData, FunctionTarget},
    function_target_pipeline::{FunctionTargetProcessor, FunctionTargetsHolder},
    mono_analysis::MonoAnalysisProcessor,
    stackless_bytecode::Bytecode,
    usage_analysis,
};

use move_model::{
    ast::{Exp, ExpData, LocalVarDecl, MemoryLabel, Operation, QuantKind},
    exp_generator::ExpGenerator,
    model::{FunctionEnv, GlobalEnv, NodeId, QualifiedInstId, StructId, TypeParameter},
    symbol::Symbol,
    ty::{PrimitiveType, Type, TypeUnification},
};

use itertools::Itertools;
use std::collections::{BTreeMap, BTreeSet};

/// A context struct that holds/accumulates information during the monomorphization process.
struct MonoRewriter {
    type_parameters: Vec<TypeParameter>,
    memory_inst_usage: BTreeSet<QualifiedInstId<StructId>>,
    // A map from memory label accessed from within the body of the quantifier
    // which needs to be specialized to the given instances in SaveMem instructions.
    mem_inst_by_label: BTreeMap<MemoryLabel, BTreeSet<QualifiedInstId<StructId>>>,
    quants_eliminated: bool,
}

impl MonoRewriter {
    pub fn new(target: &FunctionTarget) -> Self {
        let memory_inst_usage = usage_analysis::get_used_memory_inst(target)
            .iter()
            .cloned()
            .collect();

        Self {
            type_parameters: target.get_type_parameters(),
            memory_inst_usage,
            mem_inst_by_label: BTreeMap::new(),
            quants_eliminated: false,
        }
    }

    pub fn run(&mut self, builder: &mut FunctionDataBuilder) {
        let code = std::mem::take(&mut builder.data.code);
        for bc in code {
            if let Bytecode::Prop(id, kind, exp) = bc {
                let exp = self.rewrite_type_quant(builder, exp);
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

    fn rewrite_type_quant(&mut self, builder: &mut FunctionDataBuilder, exp: Exp) -> Exp {
        let env = builder.global_env();

        ExpData::rewrite(exp, &mut |e| {
            if let ExpData::Quant(node_id, kind, ranges, triggers, condition, body) = e.as_ref() {
                let mut type_vars = BTreeSet::new();
                for (var, range) in ranges {
                    let ty = env.get_node_type(range.node_id());
                    if let Type::TypeDomain(bt) = ty.skip_reference() {
                        if matches!(bt.as_ref(), Type::Primitive(PrimitiveType::TypeValue)) {
                            type_vars.insert(var.name);
                        }
                    }
                }
                // skip mono if there is no type qualification in this expression.
                if type_vars.is_empty() {
                    return Err(e);
                }

                if !triggers.is_empty() {
                    env.error(
                        &env.get_node_loc(*node_id),
                        "Cannot have triggers with type value ranges",
                    );
                    return Err(e);
                }
                if kind.is_choice() {
                    env.error(
                        &env.get_node_loc(*node_id),
                        "Type quantification cannot be used with a choice operator",
                    );
                    return Err(e);
                }
                // TODO (mengxu) I am not confident that we could eliminate the `exists` quantifier,
                // i.e, `exists t: type: P<t>`...

                // eliminate the type quantifiers
                let prop_insts = self.analyze_instantiation(env, condition.as_ref(), body);

                let mut expanded = vec![];
                for inst in &prop_insts {
                    let new_exp = self.eliminate_type_quantifier(
                        env,
                        *node_id,
                        ranges,
                        kind,
                        condition.as_ref(),
                        body,
                        inst,
                    );
                    expanded.push(new_exp);
                }

                // Compose the resulting list of expansions into a conjunction or disjunction.
                builder.set_loc(env.get_node_loc(*node_id));
                let combined_exp = match kind {
                    QuantKind::Forall => builder
                        .mk_join_bool(Operation::And, expanded.into_iter())
                        .unwrap_or_else(|| builder.mk_bool_const(true)),
                    QuantKind::Exists => builder
                        .mk_join_bool(Operation::Or, expanded.into_iter())
                        .unwrap_or_else(|| builder.mk_bool_const(false)),
                    _ => unreachable!(),
                };

                self.quants_eliminated = true;
                // marks that the expression is re-written and the rewriter should NOT
                // descend into the sub-expressions.
                return Ok(combined_exp);
            }
            // marks that the expression is NOT re-written and the rewriter should descend into the
            // sub-expressions for further processing.
            Err(e)
        })
    }

    // collect potential instantiations for this quantified expression
    fn analyze_instantiation(
        &mut self,
        env: &GlobalEnv,
        cond: Option<&Exp>,
        body: &Exp,
    ) -> Vec<BTreeMap<Symbol, Type>> {
        // holds possible instantiations per type local
        let mut prop_insts = BTreeMap::new();

        let exp_mems: BTreeSet<_> = cond
            .map(|e| e.used_memory(env))
            .unwrap_or_else(BTreeSet::new)
            .into_iter()
            .chain(body.used_memory(env))
            .map(|(mem, _)| mem)
            .collect();

        for exp_mem in &exp_mems {
            for fun_mem in &self.memory_inst_usage {
                if exp_mem.module_id != fun_mem.module_id || exp_mem.id != fun_mem.id {
                    continue;
                }
                let rel = TypeUnification::unify_vec(
                    &fun_mem.inst,
                    &exp_mem.inst,
                    env,
                    &self.type_parameters,
                    /* match_num_and_int*/ true,
                );
                match rel {
                    None => continue,
                    Some(unifier) => {
                        let (_, subst_rhs) = unifier.decompose();
                        for (k, v) in subst_rhs {
                            match k {
                                Type::TypeLocal(local_idx) => {
                                    prop_insts
                                        .entry(local_idx)
                                        .or_insert_with(BTreeSet::new)
                                        .insert(v);
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

    // collect potential instantiations for this quantified expression
    fn eliminate_type_quantifier(
        &mut self,
        env: &GlobalEnv,
        node_id: NodeId,
        ranges: &[(LocalVarDecl, Exp)],
        kind: &QuantKind,
        cond: Option<&Exp>,
        body: &Exp,
        inst: &BTreeMap<Symbol, Type>,
    ) -> Exp {
        // Collect remaining range variables
        let new_ranges: Vec<_> = ranges
            .iter()
            .filter_map(|(v, e)| {
                if inst.contains_key(&v.name) {
                    None
                } else {
                    Some((v.clone(), e.clone()))
                }
            })
            .collect();

        // Create the effective proposition of the eliminated quantifier.
        let new_prop = if new_ranges.is_empty() {
            match cond {
                Some(c) => match kind {
                    QuantKind::Forall => {
                        ExpData::Call(node_id, Operation::Implies, vec![c.clone(), body.clone()])
                            .into_exp()
                    }
                    QuantKind::Exists => {
                        ExpData::Call(node_id, Operation::And, vec![c.clone(), body.clone()])
                            .into_exp()
                    }
                    _ => unreachable!(),
                },
                _ => body.clone(),
            }
        } else {
            ExpData::Quant(
                node_id,
                *kind,
                new_ranges,
                vec![],
                cond.cloned(),
                body.clone(),
            )
            .into_exp()
        };

        // Instantiate the new proposition
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
        inst_prop.visit(&mut |e| match e {
            ExpData::Call(id, Operation::Global(Some(label)), _)
            | ExpData::Call(id, Operation::Exists(Some(label)), _) => {
                let mut node_inst = env.get_node_instantiation(*id);
                let qid = match node_inst.pop().unwrap() {
                    Type::Struct(mid, sid, struct_inst) => mid.qualified_inst(sid, struct_inst),
                    t => panic!("expected `Type::Struct`, found: `{:?}`", t),
                };
                self.mem_inst_by_label
                    .entry(*label)
                    .or_default()
                    .insert(qid);
            }
            ExpData::Call(id, Operation::Function(mid, fid, Some(labels)), _) => {
                let node_inst = env.get_node_instantiation(*id);
                let module_env = env.get_module(*mid);
                let fun = module_env.get_spec_fun(*fid);
                for (i, mem) in fun.used_memory.iter().enumerate() {
                    let qid = mem.clone().instantiate(&node_inst);
                    self.mem_inst_by_label
                        .entry(labels[i])
                        .or_default()
                        .insert(qid);
                }
            }
            _ => {}
        });

        inst_prop
    }
}

/// This is the monomorphization processor that works on a function level.
///
/// It eliminates potential quantifiers over types by substituting those types with instantiations
/// that are found within the function being processed.
pub struct MonoProcessorV2 {}

impl MonoProcessorV2 {
    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }
}

impl FunctionTargetProcessor for MonoProcessorV2 {
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

    fn finalize(&self, env: &GlobalEnv, targets: &mut FunctionTargetsHolder) {
        // TODO(mengxu) type quantifier elimination on axioms

        // fill in the MonoInfo
        let analyzer = MonoAnalysisProcessor::new();
        analyzer.analyze(env, None, targets);
    }

    fn name(&self) -> String {
        "mono_analysis".to_owned()
    }
}
