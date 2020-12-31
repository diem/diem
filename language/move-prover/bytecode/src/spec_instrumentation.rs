// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Transformation injection specifications into the bytecode.

use crate::{
    function_data_builder::FunctionDataBuilder,
    function_target::FunctionData,
    function_target_pipeline::{FunctionTargetProcessor, FunctionTargetsHolder},
    stackless_bytecode::{Bytecode, PropKind},
};
use itertools::Itertools;
use move_model::{
    ast,
    ast::{ConditionKind, Exp, LocalVarDecl, MemoryLabel},
    model::{FunctionEnv, QualifiedId, SpecVarId, StructId},
    ty::NUM_TYPE,
};
use std::collections::BTreeMap;

pub struct SpecInstrumenter();

impl SpecInstrumenter {
    pub fn new() -> Box<Self> {
        Box::new(Self())
    }
}

impl FunctionTargetProcessor for SpecInstrumenter {
    fn process(
        &self,
        _targets: &mut FunctionTargetsHolder,
        fun_env: &FunctionEnv<'_>,
        data: FunctionData,
    ) -> FunctionData {
        if fun_env.is_native() {
            // Nothing to do
            return data;
        }

        let builder = FunctionDataBuilder::new(fun_env, data);
        let mut ctrans = ConditionTranslator::new(builder);

        // Translate specification of this function.
        ctrans.translate_spec();
        let ConditionTranslator {
            mut builder,
            saved_memory,
            saved_spec_vars,
            pre,
            post,
        } = ctrans;

        // Extract and clear current code
        let old_code = std::mem::take(&mut builder.data.code);

        // Inject preconditions.
        use Bytecode::*;
        use PropKind::*;
        builder.set_loc(builder.fun_env.get_loc()); // reset to function level
        for exp in pre {
            builder.emit_with(move |attr_id| Prop(attr_id, Assume, exp))
        }

        // Inject state save instructions.
        for (mem, label) in saved_memory {
            builder.emit_with(|attr_id| SaveMem(attr_id, label, mem));
        }
        for (spec_var, label) in saved_spec_vars {
            builder.emit_with(|attr_id| SaveSpecVar(attr_id, label, spec_var));
        }

        // Copy old_code, adding post conditions before returns
        for bc in old_code {
            if matches!(bc, Ret(..)) {
                for exp in &post {
                    builder.emit_with(move |attr_id| Prop(attr_id, Assert, exp.clone()))
                }
            }
            builder.emit(bc)
        }

        builder.data
    }

    fn name(&self) -> String {
        "spec_instrumenter".to_string()
    }
}

/// A helper which reduces specification conditions to assume/assert statements.
struct ConditionTranslator<'a> {
    builder: FunctionDataBuilder<'a>,
    saved_memory: BTreeMap<QualifiedId<StructId>, MemoryLabel>,
    saved_spec_vars: BTreeMap<QualifiedId<SpecVarId>, MemoryLabel>,
    pre: Vec<Exp>,
    post: Vec<Exp>,
}

impl<'a> ConditionTranslator<'a> {
    fn new(builder: FunctionDataBuilder<'a>) -> Self {
        ConditionTranslator {
            builder,
            saved_memory: Default::default(),
            saved_spec_vars: Default::default(),
            pre: vec![],
            post: vec![],
        }
    }

    fn translate_spec(&mut self) {
        use ast::Operation::*;
        let fun_env = self.builder.fun_env;
        let spec = fun_env.get_spec();

        let aborts_if = spec.filter_kind(ConditionKind::AbortsIf).collect_vec();
        let requires = spec.filter_kind(ConditionKind::Requires).collect_vec();
        let ensures = spec.filter_kind(ConditionKind::Ensures).collect_vec();

        // Add requires to pre-condition.
        for cond in requires {
            self.pre.push(cond.exp.clone());
        }

        // Add aborts_if to post-condition.
        // 1st generate `<cond> ==> abort_flag [ && abort_code == <code> ]`
        for cond in &aborts_if {
            self.builder.set_loc(cond.loc.clone());
            let mut concl = self.builder.mk_bool_call(AbortFlag, vec![]);
            if !cond.additional_exps.is_empty() {
                let abort_code = self.builder.mk_call(&NUM_TYPE, AbortCode, vec![]);
                let abort_code_eq = self
                    .builder
                    .mk_bool_call(Eq, vec![abort_code, cond.additional_exps[0].clone()]);
                concl = self.builder.mk_bool_call(And, vec![concl, abort_code_eq]);
            }
            let abort_cond = self.eliminate_old(&cond.exp, true);
            self.post
                .push(self.builder.mk_bool_call(Implies, vec![abort_cond, concl]));
        }
        // 2nd generate `abort_flag ==> <cond1> || .. || <condN>`
        self.builder.set_loc(self.builder.fun_env.get_loc());
        if aborts_if.is_empty() {
            self.post.push(
                self.builder
                    .mk_bool_call(Not, vec![self.builder.mk_bool_call(AbortFlag, vec![])]),
            );
        } else {
            let mut concl = self.eliminate_old(&aborts_if[0].exp, true);
            for cond in aborts_if.iter().skip(1) {
                let abort_cond = self.eliminate_old(&cond.exp, true);
                concl = self.builder.mk_bool_call(Or, vec![concl, abort_cond]);
            }
            self.post.push(self.builder.mk_bool_call(
                Implies,
                vec![self.builder.mk_bool_call(AbortFlag, vec![]), concl],
            ));
        }

        // Add ensures to post-condition.
        for cond in ensures {
            let exp = self.eliminate_old(&cond.exp, false);
            self.post.push(exp);
        }
    }

    fn eliminate_old(&mut self, exp: &Exp, in_old: bool) -> Exp {
        use ast::Operation::*;
        use Exp::*;
        match exp {
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
            Call(_, Old, args) => self.eliminate_old(&args[0], true),
            Call(node_id, oper, args) => {
                Call(*node_id, oper.clone(), self.eliminate_old_vec(args, in_old))
            }
            Invoke(node_id, target, args) => {
                let target = self.eliminate_old(target, in_old);
                Invoke(
                    *node_id,
                    Box::new(target),
                    self.eliminate_old_vec(args, in_old),
                )
            }
            Lambda(node_id, decls, body) => {
                let decls = self.eliminate_old_decls(decls, in_old);
                Lambda(*node_id, decls, Box::new(self.eliminate_old(body, in_old)))
            }
            Block(node_id, decls, body) => {
                let decls = self.eliminate_old_decls(decls, in_old);
                Block(*node_id, decls, Box::new(self.eliminate_old(body, in_old)))
            }
            IfElse(node_id, cond, if_true, if_false) => IfElse(
                *node_id,
                Box::new(self.eliminate_old(cond, in_old)),
                Box::new(self.eliminate_old(if_true, in_old)),
                Box::new(self.eliminate_old(if_false, in_old)),
            ),
            _ => exp.clone(),
        }
    }

    fn eliminate_old_vec(&mut self, exps: &[Exp], in_old: bool) -> Vec<Exp> {
        exps.iter()
            .map(|e| self.eliminate_old(e, in_old))
            .collect_vec()
    }

    fn eliminate_old_decls(&mut self, decls: &[LocalVarDecl], in_old: bool) -> Vec<LocalVarDecl> {
        decls
            .iter()
            .map(|LocalVarDecl { id, name, binding }| LocalVarDecl {
                id: *id,
                name: *name,
                binding: binding.as_ref().map(|e| self.eliminate_old(e, in_old)),
            })
            .collect_vec()
    }

    fn save_spec_var(&mut self, qid: QualifiedId<SpecVarId>) -> MemoryLabel {
        let builder = &mut self.builder;
        *self
            .saved_spec_vars
            .entry(qid)
            .or_insert_with(|| builder.global_env().new_global_id())
    }

    fn save_memory(&mut self, qid: QualifiedId<StructId>) -> MemoryLabel {
        let builder = &mut self.builder;
        *self
            .saved_memory
            .entry(qid)
            .or_insert_with(|| builder.global_env().new_global_id())
    }
}
