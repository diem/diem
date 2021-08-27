// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Analysis which computes information needed in backends for monomorphization. This
//! computes the distinct type instantiations in the model for structs and inlined functions.
//! It also eliminates type quantification (`forall coin_type: type:: P`).

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    rc::Rc,
};

use itertools::Itertools;

use move_model::{
    ast,
    ast::{Condition, ConditionKind, ExpData},
    model::{
        FunId, FunctionEnv, GlobalEnv, ModuleId, QualifiedId, QualifiedInstId, SpecFunId,
        SpecVarId, StructEnv, StructId,
    },
    ty::{Type, TypeDisplayContext},
};

use crate::{
    function_target::{FunctionData, FunctionTarget},
    function_target_pipeline::{FunctionTargetProcessor, FunctionTargetsHolder, FunctionVariant},
    stackless_bytecode::{Bytecode, Operation},
    verification_analysis,
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
    pub axioms: Vec<Condition>,
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

/// This processor computes monomorphization information for backends.
impl FunctionTargetProcessor for MonoAnalysisProcessor {
    fn process(
        &self,
        _targets: &mut FunctionTargetsHolder,
        _fun_env: &FunctionEnv<'_>,
        data: FunctionData,
    ) -> FunctionData {
        // Nothing to do
        data
    }

    fn name(&self) -> String {
        "mono_analysis".to_owned()
    }

    fn initialize(&self, env: &GlobalEnv, targets: &mut FunctionTargetsHolder) {
        self.analyze(env, None, targets);
    }

    fn finalize(&self, env: &GlobalEnv, _targets: &mut FunctionTargetsHolder) {
        // TODO: specialize axioms based on functions they are using. For now,
        //   we can't deal with generic axioms.
        let mut axioms = vec![];
        for module_env in env.get_modules() {
            for cond in &module_env.get_spec().conditions {
                if let ConditionKind::Axiom(params) = &cond.kind {
                    if params.is_empty() {
                        axioms.push(cond.clone());
                    } else {
                        env.error(&cond.loc, "generic axioms not yet supported")
                    }
                }
            }
        }
        if !axioms.is_empty() {
            env.update_extension(move |info: &mut MonoInfo| {
                info.axioms = axioms;
            });
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
        for (fid, insts) in &info.spec_funs {
            let module_env = env.get_module(fid.module_id);
            let decl = module_env.get_spec_fun(fid.id);
            let mname = module_env.get_full_name_str();
            let fname = decl.name.display(env.symbol_pool());
            writeln!(f, "spec fun {}::{} = {{", mname, fname)?;
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
    fn analyze<'a>(
        &self,
        env: &'a GlobalEnv,
        rewritten_axioms: Option<&[Condition]>,
        targets: &'a FunctionTargetsHolder,
    ) {
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
        if let Some(axioms) = rewritten_axioms {
            // Analyze newly rewritten axioms.
            for axiom in axioms {
                analyzer.analyze_exp(&axiom.exp)
            }
        } else {
            // Analyze axioms found in modules.
            for module_env in env.get_modules() {
                for axiom in module_env.get_spec().filter_kind_axiom() {
                    analyzer.analyze_exp(&axiom.exp)
                }
            }
        }
        analyzer.analyze_funs();
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
    fn analyze_funs(&mut self) {
        // Analyze top-level, verified functions. Any functions they call will be queued
        // in self.todo_targets for later analysis. During this phase, self.inst_opt is None.
        for module in self.env.get_modules() {
            for fun in module.get_functions() {
                for (_, target) in self.targets.get_targets(&fun) {
                    let is_verified = verification_analysis::get_info(&target).verified;
                    if is_verified {
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
        // then the spec functions; the later can never add new regular functions.
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
            SaveMem(_, _, mem) => {
                let mem = self.instantiate_mem(mem.to_owned());
                let struct_env = self.env.get_struct_qid(mem.to_qualified_id());
                self.add_struct(struct_env, &mem.inst);
            }
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

    fn instantiate_mem(&self, mem: QualifiedInstId<StructId>) -> QualifiedInstId<StructId> {
        if let Some(inst) = &self.inst_opt {
            mem.instantiate(inst)
        } else {
            mem
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

    fn analyze_exp(&mut self, exp: &ExpData) {
        exp.visit(&mut |e| {
            let node_id = e.node_id();
            self.add_type_root(&self.env.get_node_type(node_id));
            for ref ty in self.env.get_node_instantiation(node_id) {
                self.add_type_root(ty);
            }
            match e {
                ExpData::Call(node_id, ast::Operation::Function(mid, fid, _), _) => {
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
                ExpData::SpecVar(node_id, mid, sid, _) => {
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
