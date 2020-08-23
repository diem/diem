// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    function_target::{FunctionTarget, FunctionTargetData},
    function_target_pipeline::{FunctionTargetProcessor, FunctionTargetsHolder},
    stackless_bytecode::{Bytecode, Operation},
};
use spec_lang::env::{FunctionEnv, QualifiedId, StructId};
use std::collections::BTreeSet;

pub fn get_used_memory<'env>(
    target: &'env FunctionTarget,
) -> &'env BTreeSet<QualifiedId<StructId>> {
    &target
        .get_annotations()
        .get::<UsageAnnotation>()
        .expect("Invariant violation: target not analyzed")
        .used_memory
}

pub fn get_modified_memory<'env>(
    target: &'env FunctionTarget,
) -> &'env BTreeSet<QualifiedId<StructId>> {
    &target
        .get_annotations()
        .get::<UsageAnnotation>()
        .expect("Invariant violation: target not analyzed")
        .modified_memory
}

/// The annotation for usage of functions. This is computed by the function target processor.
#[derive(Default)]
struct UsageAnnotation {
    // The memory which is directly and transitively accessed by this function.
    used_memory: BTreeSet<QualifiedId<StructId>>,
    // The memory which is directly and transitiviely modfied by this function.
    modified_memory: BTreeSet<QualifiedId<StructId>>,
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
        targets: &mut FunctionTargetsHolder,
        func_env: &FunctionEnv<'_>,
        mut data: FunctionTargetData,
    ) -> FunctionTargetData {
        let mut annotation = UsageAnnotation::default();

        let func_target = FunctionTarget::new(func_env, &data);
        func_target.get_modify_targets().keys().for_each(|target| {
            annotation.modified_memory.insert(*target);
        });
        if !func_env.is_native() {
            self.analyze(&mut annotation, func_target, targets);
        }
        data.annotations.set(annotation);
        data
    }
}

impl UsageProcessor {
    fn analyze(
        &self,
        annotation: &mut UsageAnnotation,
        func_target: FunctionTarget<'_>,
        targets: &FunctionTargetsHolder,
    ) {
        use Bytecode::*;
        use Operation::*;

        for code in func_target.get_bytecode() {
            if let Call(_, _, oper, _) = code {
                match oper {
                    Function(mid, fid, _) => {
                        let func_env = func_target
                            .get_global_env()
                            .get_function(mid.qualified(*fid));
                        if !func_env.is_native() {
                            let func_target = targets.get_target(&func_env);
                            let summary = func_target
                                .get_annotations()
                                .get::<UsageAnnotation>()
                                .unwrap_or_else(|| {
                                    panic!(
                                        "Failed to look up summary for {:?} in module {:?}",
                                        func_target.func_env.get_identifier(),
                                        func_target.func_env.module_env.get_identifier()
                                    )
                                });

                            annotation.modified_memory.extend(&summary.modified_memory);
                            annotation.used_memory.extend(&summary.used_memory);
                        }
                    }
                    MoveTo(mid, sid, _) | MoveFrom(mid, sid, _) | BorrowGlobal(mid, sid, _) => {
                        annotation.modified_memory.insert(mid.qualified(*sid));
                        annotation.used_memory.insert(mid.qualified(*sid));
                    }
                    Exists(mid, sid, _) | GetField(mid, sid, ..) | GetGlobal(mid, sid, _) => {
                        annotation.used_memory.insert(mid.qualified(*sid));
                    }
                    _ => {}
                }
            }
        }
    }
}
