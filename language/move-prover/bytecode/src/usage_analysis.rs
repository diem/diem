// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    compositional_analysis::{CompositionalAnalysis, SummaryCache},
    dataflow_analysis::{DataflowAnalysis, TransferFunctions},
    dataflow_domains::{AbstractDomain, JoinResult, SetDomain},
    function_target::{FunctionData, FunctionTarget},
    function_target_pipeline::{FunctionTargetProcessor, FunctionTargetsHolder, FunctionVariant},
    stackless_bytecode::{Bytecode, Operation},
};
use itertools::Itertools;
use move_binary_format::file_format::CodeOffset;
use move_model::model::{FunctionEnv, GlobalEnv, QualifiedId, QualifiedInstId, StructId};
use std::{collections::BTreeSet, fmt, fmt::Formatter, prelude::v1::Result::Ok};

// Legacy API, no representation of type instantiations.

pub fn get_used_memory(target: &FunctionTarget) -> BTreeSet<QualifiedId<StructId>> {
    get_used_memory_inst(target)
        .iter()
        .map(|id| id.to_qualified_id())
        .collect()
}

pub fn get_modified_memory(target: &FunctionTarget) -> BTreeSet<QualifiedId<StructId>> {
    get_modified_memory_inst(target)
        .iter()
        .map(|id| id.to_qualified_id())
        .collect()
}

pub fn get_directly_modified_memory(target: &FunctionTarget) -> BTreeSet<QualifiedId<StructId>> {
    get_directly_modified_memory_inst(target)
        .iter()
        .map(|id| id.to_qualified_id())
        .collect()
}

pub fn get_used_memory_inst<'env>(
    target: &'env FunctionTarget,
) -> &'env SetDomain<QualifiedInstId<StructId>> {
    &target
        .get_annotations()
        .get::<UsageState>()
        .expect("Invariant violation: target not analyzed")
        .used_memory
}

pub fn get_modified_memory_inst<'env>(
    target: &'env FunctionTarget,
) -> &'env SetDomain<QualifiedInstId<StructId>> {
    &target
        .get_annotations()
        .get::<UsageState>()
        .expect("Invariant violation: target not analyzed")
        .modified_memory
}

pub fn get_directly_modified_memory_inst<'env>(
    target: &'env FunctionTarget,
) -> &'env SetDomain<QualifiedInstId<StructId>> {
    &target
        .get_annotations()
        .get::<UsageState>()
        .expect("Invariant violation: target not analyzed")
        .directly_modified_memory
}

/// The annotation for usage of functions. This is computed by the function target processor.
#[derive(Debug, Clone, Default, Eq, PartialOrd, PartialEq)]
struct UsageState {
    // The memory which is directly and transitively accessed by this function.
    used_memory: SetDomain<QualifiedInstId<StructId>>,
    // The memory which is directly and transitively modified by this function.
    modified_memory: SetDomain<QualifiedInstId<StructId>>,
    directly_modified_memory: SetDomain<QualifiedInstId<StructId>>,
}

impl AbstractDomain for UsageState {
    // TODO: would be cool to add a derive(Join) macro for this
    fn join(&mut self, other: &Self) -> JoinResult {
        match (
            self.used_memory.join(&other.used_memory),
            self.modified_memory.join(&other.modified_memory),
            self.directly_modified_memory
                .join(&other.directly_modified_memory),
        ) {
            (JoinResult::Unchanged, JoinResult::Unchanged, JoinResult::Unchanged) => {
                JoinResult::Unchanged
            }
            _ => JoinResult::Changed,
        }
    }
}

struct MemoryUsageAnalysis<'a> {
    cache: SummaryCache<'a>,
}

impl<'a> DataflowAnalysis for MemoryUsageAnalysis<'a> {}
impl<'a> CompositionalAnalysis<UsageState> for MemoryUsageAnalysis<'a> {
    fn to_summary(&self, state: UsageState, _fun_target: &FunctionTarget) -> UsageState {
        state
    }
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
        mut data: FunctionData,
    ) -> FunctionData {
        let mut initial_state = UsageState::default();
        let func_target = FunctionTarget::new(func_env, &data);
        func_target.get_modify_ids().iter().for_each(|qid| {
            initial_state.modified_memory.insert(qid.clone());
        });

        let cache = SummaryCache::new(targets, func_env.module_env.env);
        let analysis = MemoryUsageAnalysis { cache };
        let summary = analysis.summarize(&func_target, initial_state);
        data.annotations.set(summary);
        data
    }

    fn name(&self) -> String {
        "usage_analysis".to_string()
    }

    fn dump_result(
        &self,
        f: &mut Formatter<'_>,
        env: &GlobalEnv,
        targets: &FunctionTargetsHolder,
    ) -> fmt::Result {
        writeln!(f, "\n\n********* Result of usage analysis *********\n\n")?;
        for module in env.get_modules() {
            if !module.is_target() {
                continue;
            }
            for fun in module.get_functions() {
                for (_, ref target) in targets.get_targets(&fun) {
                    writeln!(
                        f,
                        "function {} [{}] {{",
                        target.func_env.get_full_name_str(),
                        target.data.variant
                    )?;
                    writeln!(
                        f,
                        "  used = {{{}}}",
                        get_used_memory_inst(target)
                            .iter()
                            .map(|qid| env.display(qid).to_string())
                            .join(", ")
                    )?;
                    writeln!(
                        f,
                        "  modified = {{{}}}",
                        get_modified_memory_inst(target)
                            .iter()
                            .map(|qid| env.display(qid).to_string())
                            .join(", ")
                    )?;
                    writeln!(
                        f,
                        "  directly modified = {{{}}}",
                        get_directly_modified_memory_inst(target)
                            .iter()
                            .map(|qid| env.display(qid).to_string())
                            .join(", ")
                    )?;
                }
            }
        }
        writeln!(f)?;
        Ok(())
    }
}

impl<'a> TransferFunctions for MemoryUsageAnalysis<'a> {
    type State = UsageState;
    const BACKWARD: bool = false;

    fn execute(&self, state: &mut Self::State, code: &Bytecode, _offset: CodeOffset) {
        use Bytecode::*;
        use Operation::*;

        if let Call(_, _, oper, _, _) = code {
            match oper {
                Function(mid, fid, inst) => {
                    if let Some(summary) = self
                        .cache
                        .get::<UsageState>(mid.qualified(*fid), &FunctionVariant::Baseline)
                    {
                        state.modified_memory.extend(
                            summary
                                .modified_memory
                                .iter()
                                .map(|qid| qid.instantiate_ref(inst)),
                        );
                        state.used_memory.extend(
                            summary
                                .used_memory
                                .iter()
                                .map(|qid| qid.instantiate_ref(inst)),
                        );
                    }
                }
                MoveTo(mid, sid, inst)
                | MoveFrom(mid, sid, inst)
                | BorrowGlobal(mid, sid, inst) => {
                    state
                        .modified_memory
                        .insert(mid.qualified_inst(*sid, inst.to_owned()));
                    state
                        .directly_modified_memory
                        .insert(mid.qualified_inst(*sid, inst.to_owned()));
                    state
                        .used_memory
                        .insert(mid.qualified_inst(*sid, inst.to_owned()));
                }
                Exists(mid, sid, inst) | GetGlobal(mid, sid, inst) => {
                    state
                        .used_memory
                        .insert(mid.qualified_inst(*sid, inst.to_owned()));
                }
                _ => {}
            }
        }
    }
}
