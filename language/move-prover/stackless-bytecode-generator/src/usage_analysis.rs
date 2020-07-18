// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    function_target::{FunctionTarget, FunctionTargetData},
    function_target_pipeline::{FunctionTargetProcessor, FunctionTargetsHolder},
    stackless_bytecode::{Bytecode, Operation},
};
use spec_lang::env::{FunId, FunctionEnv, GlobalEnv, QualifiedId, StructId};
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
};

/// A data structure to retrieve transitive enriched usage data. This internally
/// uses the result of the usage processor, which must be installed in
/// function target pipeline for this to work.
#[derive(Default)]
pub struct TransitiveUsage {
    cache: RefCell<BTreeMap<QualifiedId<FunId>, TransitiveFunctionUsage>>,
}

#[derive(Default)]
struct TransitiveFunctionUsage {
    called_functions: BTreeSet<QualifiedId<FunId>>,
    used_memory: BTreeSet<QualifiedId<StructId>>,
}

impl TransitiveUsage {
    pub fn get_called_functions(
        &self,
        env: &GlobalEnv,
        targets: &FunctionTargetsHolder,
        fun: QualifiedId<FunId>,
    ) -> BTreeSet<QualifiedId<FunId>> {
        if !self.cache.borrow().contains_key(&fun) {
            Self::populate_cache(
                &mut BTreeSet::new(),
                &mut *self.cache.borrow_mut(),
                env,
                targets,
                fun,
            );
        }
        self.cache
            .borrow()
            .get(&fun)
            .unwrap()
            .called_functions
            .clone()
    }

    pub fn get_used_memory(
        &self,
        env: &GlobalEnv,
        targets: &FunctionTargetsHolder,
        fun: QualifiedId<FunId>,
    ) -> BTreeSet<QualifiedId<StructId>> {
        if !self.cache.borrow().contains_key(&fun) {
            Self::populate_cache(
                &mut BTreeSet::new(),
                &mut *self.cache.borrow_mut(),
                env,
                targets,
                fun,
            );
        }
        self.cache.borrow().get(&fun).unwrap().used_memory.clone()
    }

    fn populate_cache(
        visited: &mut BTreeSet<QualifiedId<FunId>>,
        cache: &mut BTreeMap<QualifiedId<FunId>, TransitiveFunctionUsage>,
        env: &GlobalEnv,
        targets: &FunctionTargetsHolder,
        fun: QualifiedId<FunId>,
    ) {
        if !visited.insert(fun) {
            return;
        }
        let func_env = env.get_module(fun.module_id).into_function(fun.id);
        let func_target = targets.get_target(&func_env);
        let mut usage = TransitiveFunctionUsage::default();
        if let Some(annotation) = func_target.get_annotations().get::<UsageAnnotation>() {
            // Account for direct usage of this function.
            usage
                .called_functions
                .extend(annotation.called_functions.iter());
            usage.used_memory.extend(annotation.used_memory.iter());
            // Recursively visit all called functions and add their usage.
            for called in &annotation.called_functions {
                Self::populate_cache(visited, cache, env, targets, *called);
                let called_usage = cache.get(&called).unwrap();
                usage
                    .called_functions
                    .extend(called_usage.called_functions.iter());
                usage.used_memory.extend(called_usage.used_memory.iter());
            }
        }
        cache.insert(fun, usage);
    }
}

/// The annotation for usage of functions. This is computed by the function target processor.
#[derive(Default)]
struct UsageAnnotation {
    // The functions which are directly called by this function.
    called_functions: BTreeSet<QualifiedId<FunId>>,
    // The memory which is directly accessed by this function.
    used_memory: BTreeSet<QualifiedId<StructId>>,
}

pub struct UsageProcessor();

impl UsageProcessor {
    pub fn new() -> Box<Self> {
        Box::new(UsageProcessor())
    }
}

impl FunctionTargetProcessor for UsageProcessor {
    fn process(
        &self,
        _targets: &mut FunctionTargetsHolder,
        func_env: &FunctionEnv<'_>,
        mut data: FunctionTargetData,
    ) -> FunctionTargetData {
        let mut annotation = UsageAnnotation::default();

        if !func_env.is_native() {
            Self::analyze(&mut annotation, FunctionTarget::new(func_env, &data));
        }
        data.annotations.set(annotation);
        data
    }
}

impl UsageProcessor {
    fn analyze(annotation: &mut UsageAnnotation, func_target: FunctionTarget<'_>) {
        use Bytecode::*;
        use Operation::*;
        for code in func_target.get_bytecode() {
            if let Call(_, _, oper, _) = code {
                match oper {
                    Function(mid, fid, _) => {
                        annotation.called_functions.insert(mid.qualified(*fid));
                    }
                    MoveTo(mid, sid, _)
                    | MoveFrom(mid, sid, _)
                    | Exists(mid, sid, _)
                    | BorrowGlobal(mid, sid, _)
                    | GetField(mid, sid, ..)
                    | GetGlobal(mid, sid, _) => {
                        annotation.used_memory.insert(mid.qualified(*sid));
                    }
                    _ => {}
                }
            }
        }
    }
}
