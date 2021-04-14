// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Analysis which computes information needed in backends for monomorphization. This
//! computes the distinct type instantiations in the model for structs and inlined functions.

use crate::{
    function_target::FunctionTarget,
    function_target_pipeline::{FunctionTargetProcessor, FunctionTargetsHolder, FunctionVariant},
    stackless_bytecode::{Bytecode, Operation},
    verification_analysis,
};
use itertools::Itertools;
use move_model::{
    model::{FunId, GlobalEnv, QualifiedId, StructEnv, StructId},
    ty::{Type, TypeDisplayContext},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    rc::Rc,
};

/// The environment extension computed by this analysis.
#[derive(Clone, Default, Debug, PartialEq, PartialOrd, Eq)]
pub struct MonoInfo {
    pub structs: BTreeMap<QualifiedId<StructId>, BTreeSet<Vec<Type>>>,
    pub funs: BTreeMap<QualifiedId<FunId>, BTreeSet<Vec<Type>>>,
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

impl FunctionTargetProcessor for MonoAnalysisProcessor {
    fn run(&self, env: &GlobalEnv, targets: &mut FunctionTargetsHolder) {
        let mut analyzer = Analyzer {
            env,
            targets,
            info: MonoInfo::default(),
            todo_targets: vec![],
            done_targets: BTreeSet::new(),
            inst_opt: None,
        };
        analyzer.analyze();
        let Analyzer { info, .. } = analyzer;
        env.set_extension(info);
    }

    fn is_single_run(&self) -> bool {
        true
    }

    fn name(&self) -> String {
        "mono_analysis".to_owned()
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

struct Analyzer<'a> {
    env: &'a GlobalEnv,
    targets: &'a FunctionTargetsHolder,
    info: MonoInfo,
    todo_targets: Vec<(QualifiedId<FunId>, Vec<Type>)>,
    done_targets: BTreeSet<(QualifiedId<FunId>, Vec<Type>)>,
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
                        self.analyze_target(target);
                    }
                }
            }
        }
        // Now incrementally work included targets until they are done, while self.inst_opt
        // contains the specific instantiation.
        while !self.todo_targets.is_empty() {
            let (fun, inst) = self.todo_targets.pop().unwrap();
            self.inst_opt = Some(inst);
            self.analyze_target(
                self.targets
                    .get_target(&self.env.get_function(fun), &FunctionVariant::Baseline),
            );
            let inst = std::mem::take(&mut self.inst_opt).unwrap();
            if !inst.is_empty() {
                // Insert it into final analysis result if not trivial
                self.info.funs.entry(fun).or_default().insert(inst.clone());
            }
            self.done_targets.insert((fun, inst));
        }
    }

    fn analyze_target(&mut self, target: FunctionTarget<'_>) {
        // Analyze function locals and return value types.
        for idx in 0..target.get_local_count() {
            self.add_type(target.get_local_type(idx));
        }
        for ty in target.get_return_types().iter() {
            self.add_type(ty);
        }
        // Analyze code.
        if !target.func_env.is_native_or_intrinsic() {
            for bc in target.get_bytecode() {
                self.analyze_bytecode(&target, bc);
            }
        }
    }

    fn analyze_bytecode(&mut self, target: &FunctionTarget<'_>, bc: &Bytecode) {
        use Bytecode::*;
        use Operation::*;
        // We only need to analyze function calls, not `pack` or other instructions
        // because the types those are using are reflected in locals which are analyzed
        // elsewhere.
        if let Call(_, _, Function(mid, fid, targs), ..) = bc {
            if !target.is_opaque() {
                // This call needs to be inlined, with targs instantiated by self.inst_opt.
                // Schedule for later processing.
                let actuals = if let Some(inst) = &self.inst_opt {
                    targs.iter().map(|ty| ty.instantiate(inst)).collect_vec()
                } else {
                    targs.to_owned()
                };
                let fun = mid.qualified(*fid);
                // Only if this call has not been processed yet, queue it for future processing.
                if !self.done_targets.contains(&(fun, actuals.clone())) {
                    self.todo_targets.push((mid.qualified(*fid), actuals));
                }
            }
        }
    }

    // Type Analysis
    // =============

    fn add_type(&mut self, ty: &Type) {
        if let Some(inst) = &self.inst_opt {
            let ty = ty.instantiate(inst);
            self.add_type_continue(&ty)
        } else {
            self.add_type_continue(ty)
        }
    }

    fn add_type_continue(&mut self, ty: &Type) {
        match ty {
            Type::Primitive(_) => {}
            Type::Tuple(tys) => self.add_types(tys),
            Type::Vector(et) => self.add_type(&*et),
            Type::Struct(mid, sid, targs) => {
                self.add_struct(self.env.get_module(*mid).into_struct(*sid), targs)
            }
            Type::Reference(_, rt) => self.add_type(&*rt),
            Type::Fun(args, res) => {
                self.add_types(args);
                self.add_type(&*res);
            }
            Type::TypeDomain(rd) => self.add_type(&*rd),
            Type::ResourceDomain(mid, sid, Some(targs)) => {
                self.add_struct(self.env.get_module(*mid).into_struct(*sid), targs)
            }
            _ => {}
        }
    }

    fn add_types<'b, T: IntoIterator<Item = &'b Type>>(&mut self, tys: T) {
        for ty in tys {
            self.add_type(ty);
        }
    }

    fn add_struct(&mut self, struct_: StructEnv<'_>, targs: &[Type]) {
        if !targs.is_empty() {
            self.info
                .structs
                .entry(struct_.get_qualified_id())
                .or_default()
                .insert(targs.to_owned());
            self.add_types(targs);
        }
    }
}
