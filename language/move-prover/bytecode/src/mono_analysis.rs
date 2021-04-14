// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Analysis which computes information needed in backends for monomorphization. This
//! computes the distinct type instantiations in the model for structs and inlined functions.
//! It also eliminates type quantification (`forall coin_type: type:: P`).

use crate::{
    function_data_builder::FunctionDataBuilder,
    function_target::{FunctionData, FunctionTarget},
    function_target_pipeline::{FunctionTargetProcessor, FunctionTargetsHolder, FunctionVariant},
    mono_analysis,
    mut_ref_instrumentation::FunctionEnv,
    stackless_bytecode::{Bytecode, Bytecode::SaveMem, Operation},
    verification_analysis,
};
use itertools::Itertools;
use move_model::{
    ast,
    ast::{Exp, LocalVarDecl, MemoryLabel, QuantKind},
    exp_generator::ExpGenerator,
    model::{
        FunId, GlobalEnv, ModuleId, NodeId, QualifiedId, QualifiedInstId, SpecFunId, SpecVarId,
        StructEnv, StructId,
    },
    symbol::Symbol,
    ty::{PrimitiveType, Type, TypeDisplayContext},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    rc::Rc,
};

/// The environment extension computed by this analysis.
#[derive(Clone, Default, Debug)]
pub struct MonoInfo {
    pub structs: BTreeMap<QualifiedId<StructId>, BTreeSet<Vec<Type>>>,
    pub funs: BTreeMap<QualifiedId<FunId>, BTreeSet<Vec<Type>>>,
    pub spec_funs: BTreeMap<QualifiedId<SpecFunId>, BTreeSet<Vec<Type>>>,
    pub spec_vars: BTreeMap<QualifiedId<SpecVarId>, BTreeSet<Vec<Type>>>,
    pub type_params: BTreeSet<u16>,
    pub vec_inst: BTreeSet<Type>,
    pub native_inst: BTreeMap<ModuleId, BTreeSet<Vec<Type>>>,
}

/// Get the information computed by this analysis.
pub fn get_info(env: &GlobalEnv) -> Rc<MonoInfo> {
    env.get_extension::<MonoInfo>()
        .unwrap_or_else(|| Rc::new(MonoInfo::default()))
}

pub struct MonoAnalysisProcessor();

impl MonoAnalysisProcessor {
    pub fn new() -> Box<Self> {
        Box::new(Self())
    }
}

/// A marker stored as an environment extension to tell the finalize method that
/// the analysis needs to be re-run because new instances have been introduced
/// by type quantifier elimination.
struct RunAnalysisAgain();

/// This processor does the following.
/// 1. During `initialize`, it performs an analysis of all instantiations in
///    the program, setting the `MonoInfo` data in the environment.
/// 2. It then processes each function, eliminating potential quantifiers over
///    types by substituting those types with instantiations found in the program.
/// 3. If (2) has added new types, it runs the mono analysis a second time.
impl FunctionTargetProcessor for MonoAnalysisProcessor {
    fn process(
        &self,
        _targets: &mut FunctionTargetsHolder,
        fun_env: &FunctionEnv<'_>,
        mut data: FunctionData,
    ) -> FunctionData {
        // Eliminate a quantification over type variable `name` by collecting all instantiations
        // of constructors using `name` inside of the quantors `body`, and then expanding
        // the body into a conjunction or disjunction. For example, given the expression
        // `forall t: type :: P[TypeCtor<u64, t>]`, and the program context containing
        // instantiations `TypeCtor<u64, A>` and `TypeCtor<address, B>`, we will generate
        // `P[TypeCtor<u64, A>] && P[TypeCtor<address, B>]`.
        let code = std::mem::take(&mut data.code);
        let mut rewriter = TypeQuantRewriter {
            builder: FunctionDataBuilder::new(fun_env, data),
            instantiated_memory: Default::default(),
            quants_eliminated: false,
        };
        for bc in code {
            if let Bytecode::Prop(id, kind, exp) = bc {
                let exp = rewriter.rewrite_type_quant(exp);
                rewriter.builder.emit(Bytecode::Prop(id, kind, exp));
            } else {
                rewriter.builder.emit(bc);
            }
        }
        if rewriter.quants_eliminated {
            // We need to go a second time over the code to expand `SaveMem` instructions.
            // Where we previous had `SaveMem<R<type_var>>` we need to create one SaveMem
            // for every instance.
            let code = std::mem::take(&mut rewriter.builder.data.code);
            for bc in code {
                match bc {
                    Bytecode::SaveMem(id, label, mem) => {
                        if rewriter.instantiated_memory.contains_key(&label) {
                            for inst in rewriter.instantiated_memory.get(&label).unwrap() {
                                rewriter.builder.emit(Bytecode::SaveMem(
                                    id,
                                    label,
                                    inst.to_owned(),
                                ));
                            }
                        } else if !mem
                            .inst
                            .iter()
                            .any(|ty| ty.contains(&|t| matches!(t, Type::TypeLocal(_))))
                        {
                            // Only retain the SaveMem if it does not contain type locals.
                            // Such SaveMem's can result from zero expansions during quantifier
                            // elimination, and they are dead.
                            rewriter.builder.emit(SaveMem(id, label, mem));
                        }
                    }
                    _ => rewriter.builder.emit(bc),
                }
            }
            rewriter
                .builder
                .global_env()
                .set_extension(RunAnalysisAgain());
        }
        rewriter.builder.data
    }

    fn name(&self) -> String {
        "mono_analysis".to_owned()
    }

    fn initialize(&self, env: &GlobalEnv, targets: &mut FunctionTargetsHolder) {
        self.analyze(env, targets);
    }

    fn finalize(&self, env: &GlobalEnv, targets: &mut FunctionTargetsHolder) {
        if env.has_extension::<RunAnalysisAgain>() {
            self.analyze(env, targets);
        }
    }

    fn dump_result(
        &self,
        f: &mut fmt::Formatter,
        env: &GlobalEnv,
        _targets: &FunctionTargetsHolder,
    ) -> fmt::Result {
        writeln!(f, "\n\n==== mono-analysis result ====\n")?;
        let info = env
            .get_extension::<MonoInfo>()
            .expect("monomorphization analysis not run");
        let tctx = TypeDisplayContext::WithEnv {
            env,
            type_param_names: None,
        };
        let display_inst = |tys: &[Type]| {
            tys.iter()
                .map(|ty| ty.display(&tctx).to_string())
                .join(", ")
        };
        for (sid, insts) in &info.structs {
            let sname = env.get_struct(*sid).get_full_name_str();
            writeln!(f, "struct {} = {{", sname)?;
            for inst in insts {
                writeln!(f, "  <{}>", display_inst(inst))?;
            }
            writeln!(f, "}}")?;
        }
        for (fid, insts) in &info.funs {
            let fname = env.get_function(*fid).get_full_name_str();
            writeln!(f, "fun {} = {{", fname)?;
            for inst in insts {
                writeln!(f, "  <{}>", display_inst(inst))?;
            }
            writeln!(f, "}}")?;
        }

        Ok(())
    }
}

// Instantiation Analysis
// ======================

impl MonoAnalysisProcessor {
    fn analyze(&self, env: &GlobalEnv, targets: &FunctionTargetsHolder) {
        let mut analyzer = Analyzer {
            env,
            targets,
            info: MonoInfo::default(),
            todo_funs: vec![],
            done_funs: BTreeSet::new(),
            todo_spec_funs: vec![],
            done_spec_funs: BTreeSet::new(),
            done_types: BTreeSet::new(),
            inst_opt: None,
        };
        analyzer.analyze();
        let Analyzer { info, .. } = analyzer;
        env.set_extension(info);
    }
}

struct Analyzer<'a> {
    env: &'a GlobalEnv,
    targets: &'a FunctionTargetsHolder,
    info: MonoInfo,
    todo_funs: Vec<(QualifiedId<FunId>, Vec<Type>)>,
    done_funs: BTreeSet<(QualifiedId<FunId>, Vec<Type>)>,
    todo_spec_funs: Vec<(QualifiedId<SpecFunId>, Vec<Type>)>,
    done_spec_funs: BTreeSet<(QualifiedId<SpecFunId>, Vec<Type>)>,
    done_types: BTreeSet<Type>,
    inst_opt: Option<Vec<Type>>,
}

impl<'a> Analyzer<'a> {
    fn analyze(&mut self) {
        // Analyze top-level, verified functions. Any functions they call will be queued in
        // self.todo_targets for later analysis. During this phase, self.inst_opt is None.
        for module in self.env.get_modules() {
            for fun in module.get_functions() {
                for (_, target) in self.targets.get_targets(&fun) {
                    let info = verification_analysis::get_info(&target);
                    if info.verified {
                        self.analyze_fun(target.clone());

                        // We also need to analyze all modify targets because they are not
                        // included in the bytecode.
                        for (_, exps) in target.get_modify_ids_and_exps() {
                            for exp in exps {
                                self.analyze_exp(exp);
                            }
                        }
                    }
                }
            }
        }
        // Now incrementally work todo lists until they are done, while self.inst_opt
        // contains the specific instantiation. We can first do regular functions,
        // the spec functions; the later can never add new regular functions.
        while !self.todo_funs.is_empty() {
            let (fun, inst) = self.todo_funs.pop().unwrap();
            self.inst_opt = Some(inst);
            self.analyze_fun(
                self.targets
                    .get_target(&self.env.get_function(fun), &FunctionVariant::Baseline),
            );
            let inst = std::mem::take(&mut self.inst_opt).unwrap();
            // Insert it into final analysis result.
            self.info.funs.entry(fun).or_default().insert(inst.clone());
            self.done_funs.insert((fun, inst));
        }
        while !self.todo_spec_funs.is_empty() {
            let (fun, inst) = self.todo_spec_funs.pop().unwrap();
            self.inst_opt = Some(inst);
            self.analyze_spec_fun(fun);
            let inst = std::mem::take(&mut self.inst_opt).unwrap();
            // Insert it into final analysis result.
            self.info
                .spec_funs
                .entry(fun)
                .or_default()
                .insert(inst.clone());
            self.done_spec_funs.insert((fun, inst));
        }
    }

    fn analyze_fun(&mut self, target: FunctionTarget<'_>) {
        // Analyze function locals and return value types.
        for idx in 0..target.get_local_count() {
            self.add_type_root(target.get_local_type(idx));
        }
        for ty in target.get_return_types().iter() {
            self.add_type_root(ty);
        }
        // Analyze code.
        if !target.func_env.is_native_or_intrinsic() {
            for bc in target.get_bytecode() {
                self.analyze_bytecode(&target, bc);
            }
        }
    }

    fn analyze_bytecode(&mut self, _target: &FunctionTarget<'_>, bc: &Bytecode) {
        use Bytecode::*;
        use Operation::*;
        // We only need to analyze function calls, not `pack` or other instructions
        // because the types those are using are reflected in locals which are analyzed
        // elsewhere.
        match bc {
            Call(_, _, Function(mid, fid, targs), ..) => {
                let callee = &self.env.get_module(*mid).into_function(*fid);
                let actuals = self.instantiate_vec(targs);
                if callee.is_native_or_intrinsic() && !actuals.is_empty() {
                    // Mark the associated module to be instantiated with the given actuals.
                    // This will instantiate all functions in the module with matching number
                    // of type parameters.
                    self.info
                        .native_inst
                        .entry(callee.module_env.get_id())
                        .or_default()
                        .insert(actuals);
                } else if !callee.is_opaque() {
                    // This call needs to be inlined, with targs instantiated by self.inst_opt.
                    // Schedule for later processing if this instance has not been processed yet.
                    let entry = (mid.qualified(*fid), actuals);
                    if !self.done_funs.contains(&entry) {
                        self.todo_funs.push(entry);
                    }
                }
            }
            Prop(_, _, exp) => self.analyze_exp(exp),
            _ => {}
        }
    }

    fn instantiate_vec(&self, targs: &[Type]) -> Vec<Type> {
        if let Some(inst) = &self.inst_opt {
            Type::instantiate_slice(targs, inst)
        } else {
            targs.to_owned()
        }
    }

    // Expression and Spec Fun Analysis
    // --------------------------------

    fn analyze_spec_fun(&mut self, fun: QualifiedId<SpecFunId>) {
        let module_env = self.env.get_module(fun.module_id);
        let decl = module_env.get_spec_fun(fun.id);
        for (_, ty) in &decl.params {
            self.add_type_root(ty)
        }
        self.add_type_root(&decl.result_type);
        if let Some(exp) = &decl.body {
            self.analyze_exp(exp)
        }
    }

    fn analyze_exp(&mut self, exp: &Exp) {
        exp.visit(&mut |e| {
            let node_id = e.node_id();
            self.add_type_root(&self.env.get_node_type(node_id));
            for ref ty in self.env.get_node_instantiation(node_id) {
                self.add_type_root(ty);
            }
            match e {
                Exp::Call(node_id, ast::Operation::Function(mid, fid, _), _) => {
                    let actuals = self.instantiate_vec(&self.env.get_node_instantiation(*node_id));
                    // Only if this call has not been processed yet, queue it for future processing.
                    let module = self.env.get_module(*mid);
                    let spec_fun = module.get_spec_fun(*fid);
                    if spec_fun.is_native && !actuals.is_empty() {
                        // Add module to native modules
                        self.info
                            .native_inst
                            .entry(module.get_id())
                            .or_default()
                            .insert(actuals);
                    } else {
                        let entry = (mid.qualified(*fid), actuals);
                        if !self.done_spec_funs.contains(&entry) {
                            self.todo_spec_funs.push(entry);
                        }
                    }
                }
                Exp::SpecVar(node_id, mid, sid, _) => {
                    let actuals = self.instantiate_vec(&self.env.get_node_instantiation(*node_id));
                    let qid = mid.qualified(*sid);
                    self.info.spec_vars.entry(qid).or_default().insert(actuals);
                }
                _ => {}
            }
        });
    }

    // Type Analysis
    // -------------

    fn add_type_root(&mut self, ty: &Type) {
        if let Some(inst) = &self.inst_opt {
            let ty = ty.instantiate(inst);
            self.add_type(&ty)
        } else {
            self.add_type(ty)
        }
    }

    fn add_type(&mut self, ty: &Type) {
        if !self.done_types.insert(ty.to_owned()) {
            return;
        }
        ty.visit(&mut |t| match t {
            Type::Vector(et) => {
                self.info.vec_inst.insert(et.as_ref().clone());
            }
            Type::Struct(mid, sid, targs) => {
                self.add_struct(self.env.get_module(*mid).into_struct(*sid), targs)
            }
            Type::TypeParameter(idx) => {
                self.info.type_params.insert(*idx);
            }
            _ => {}
        });
    }

    fn add_struct(&mut self, struct_: StructEnv<'_>, targs: &[Type]) {
        if targs
            .iter()
            .any(|ty| ty.contains(&|t| matches!(t, Type::TypeLocal(..))))
        {
            // Do not add instantiations based on type locals. They will be eliminated by
            // the backend.
            return;
        }
        if struct_.is_native_or_intrinsic() && !targs.is_empty() {
            self.info
                .native_inst
                .entry(struct_.module_env.get_id())
                .or_default()
                .insert(targs.to_owned());
        } else {
            self.info
                .structs
                .entry(struct_.get_qualified_id())
                .or_default()
                .insert(targs.to_owned());
            for field in struct_.get_fields() {
                self.add_type(&field.get_type().instantiate(targs));
            }
        }
    }
}

// Type Quantifier Elimination
// ===========================

struct TypeQuantRewriter<'e> {
    builder: FunctionDataBuilder<'e>,
    quants_eliminated: bool,
    // A map from memory label accessed from within the body of the quantifier
    // which needs to be specialized to the given instances in SaveMem instructions.
    instantiated_memory: BTreeMap<MemoryLabel, BTreeSet<QualifiedInstId<StructId>>>,
}

impl<'e> TypeQuantRewriter<'e> {
    fn rewrite_type_quant(&mut self, exp: Exp) -> Exp {
        let env = self.builder.global_env();
        exp.rewrite(&mut |e| {
            if let Exp::Quant(node_id, kind, ranges, triggers, condition, body) = &e {
                for (i, (var, range)) in ranges.iter().enumerate() {
                    let ty = env.get_node_type(range.node_id());
                    if let Type::TypeDomain(bt) = ty.skip_reference() {
                        self.quants_eliminated = true;
                        if matches!(bt.as_ref(), Type::Primitive(PrimitiveType::TypeValue)) {
                            if !triggers.is_empty() {
                                env.error(
                                    &env.get_node_loc(*node_id),
                                    "Cannot have triggers with type value ranges",
                                );
                            }
                            let mut remaining_ranges = ranges.clone();
                            remaining_ranges.remove(i);
                            return (
                                true,
                                self.eliminate_type_quant(
                                    *node_id,
                                    *kind,
                                    var.name,
                                    remaining_ranges,
                                    condition.clone(),
                                    body.clone(),
                                ),
                            );
                        }
                    }
                }
            }
            (false, e)
        })
    }

    fn eliminate_type_quant(
        &mut self,
        node_id: NodeId,
        kind: QuantKind,
        name: Symbol,
        ranges: Vec<(LocalVarDecl, Exp)>,
        condition: Option<Box<Exp>>,
        body: Box<Exp>,
    ) -> Exp {
        let env = self.builder.global_env();

        // Create the effective condition of the eliminated quantifier.
        let body = if !ranges.is_empty() {
            Exp::Quant(node_id, kind, ranges, vec![], condition, body)
        } else {
            match condition {
                Some(c) => match kind {
                    QuantKind::Forall => Exp::Call(
                        node_id,
                        ast::Operation::Implies,
                        vec![c.as_ref().clone(), *body],
                    ),
                    QuantKind::Exists => Exp::Call(
                        node_id,
                        ast::Operation::And,
                        vec![c.as_ref().clone(), *body],
                    ),
                },
                _ => *body,
            }
        };

        // Collect all instantiations in which the type name appears.
        let mono_info = mono_analysis::get_info(env);
        let mut insts = BTreeSet::new();
        body.visit(&mut |e| {
            let node_id = e.node_id();
            self.collect_type_local_insts(
                mono_info.as_ref(),
                &mut insts,
                name,
                &env.get_node_type(node_id),
            );
            for ref ty in env.get_node_instantiation(node_id) {
                self.collect_type_local_insts(mono_info.as_ref(), &mut insts, name, ty);
            }
            if let Exp::LocalVar(_, n) = e {
                if *n == name {
                    env.error(
                        &env.get_node_loc(node_id),
                        &format!(
                            "type value `{}` can only be used in type constructor",
                            name.display(env.symbol_pool())
                        ),
                    )
                }
            }
        });

        // For each of the instantiations, expand the body, with the instantiation substituted.
        let expanded = insts
            .into_iter()
            .map(|ty| {
                let mut node_rewriter = |id: NodeId| {
                    let loc = env.get_node_loc(id);
                    let new_ty = env.get_node_type(id).replace_type_local(name, ty.clone());
                    let new_inst = env
                        .get_node_instantiation(id)
                        .into_iter()
                        .map(|t| t.replace_type_local(name, ty.clone()))
                        .collect_vec();
                    let new_id = env.new_node(loc, new_ty);
                    env.set_node_instantiation(new_id, new_inst);
                    new_id
                };
                let body = body.clone().rewrite_node_id(&mut node_rewriter);
                // Collect memory used by the expanded body. We need to rewrite SaveMem
                // instructions to point to the instantiated memory.
                body.visit(&mut |e| {
                    use ast::Operation::*;
                    use Exp::*;
                    match e {
                        Call(id, Global(Some(label)), _) | Call(id, Exists(Some(label)), _) => {
                            let inst = env.get_node_instantiation(*id);
                            let ty = &inst[0];
                            let (mid, sid, inst) = ty.require_struct();
                            self.instantiated_memory
                                .entry(*label)
                                .or_default()
                                .insert(mid.qualified_inst(sid, inst.to_owned()));
                        }
                        Call(id, Function(mid, fid, Some(labels)), _) => {
                            let inst = &env.get_node_instantiation(*id);
                            let module_env = env.get_module(*mid);
                            let fun = module_env.get_spec_fun(*fid);
                            for (i, mem) in fun.used_memory.iter().enumerate() {
                                let mem = mem.to_owned().instantiate(inst);
                                self.instantiated_memory
                                    .entry(labels[i])
                                    .or_default()
                                    .insert(mem);
                            }
                        }
                        _ => {}
                    }
                });
                body
            })
            .collect_vec();

        // Compose the resulting list of expansions into a conjunction or disjunction.
        self.builder.set_loc(env.get_node_loc(node_id));
        match kind {
            QuantKind::Forall => self
                .builder
                .mk_join_bool(ast::Operation::And, expanded.into_iter())
                .unwrap_or_else(|| self.builder.mk_bool_const(true)),
            QuantKind::Exists => self
                .builder
                .mk_join_bool(ast::Operation::Or, expanded.into_iter())
                .unwrap_or_else(|| self.builder.mk_bool_const(false)),
        }
    }

    /// In the given type, collect all concrete instantiations in `mono_info` of structs or
    /// vectors which are instantiated with  the type local `name`.
    fn collect_type_local_insts(
        &self,
        mono_info: &MonoInfo,
        insts: &mut BTreeSet<Type>,
        name: Symbol,
        ty: &Type,
    ) {
        let type_local = &Type::TypeLocal(name);
        let empty = &BTreeSet::new();
        ty.visit(&mut |ty| {
            use Type::*;
            match ty {
                Struct(mid, sid, inst) => {
                    for (i, ity) in inst.iter().enumerate() {
                        if ity == type_local {
                            insts.extend(
                                mono_info
                                    .structs
                                    .get(&mid.qualified(*sid))
                                    .unwrap_or(empty)
                                    .iter()
                                    .map(|tys| tys[i].clone()),
                            );
                        }
                    }
                }
                Vector(et) => {
                    if et.as_ref() == type_local {
                        insts.extend(mono_info.vec_inst.iter().cloned())
                    }
                }
                _ => {}
            }
        })
    }
}
