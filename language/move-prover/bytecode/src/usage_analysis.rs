// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    compositional_analysis::{CompositionalAnalysis, SummaryCache},
    dataflow_analysis::{
        AbstractDomain, DataflowAnalysis, JoinResult, SetDomain, TransferFunctions,
    },
    function_target::{FunctionData, FunctionTarget},
    function_target_pipeline::{FunctionTargetProcessor, FunctionTargetsHolder},
    stackless_bytecode::{Bytecode, Operation},
};
use move_model::model::{FunctionEnv, QualifiedId, StructId};
use std::collections::BTreeSet;
use vm::file_format::CodeOffset;

pub fn get_used_memory<'env>(
    target: &'env FunctionTarget,
) -> &'env BTreeSet<QualifiedId<StructId>> {
    &target
        .get_annotations()
        .get::<UsageState>()
        .expect("Invariant violation: target not analyzed")
        .used_memory
}

pub fn get_modified_memory<'env>(
    target: &'env FunctionTarget,
) -> &'env BTreeSet<QualifiedId<StructId>> {
    &target
        .get_annotations()
        .get::<UsageState>()
        .expect("Invariant violation: target not analyzed")
        .modified_memory
}

pub fn get_directly_modified_memory<'env>(
    target: &'env FunctionTarget,
) -> &'env BTreeSet<QualifiedId<StructId>> {
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
    used_memory: SetDomain<QualifiedId<StructId>>,
    // The memory which is directly and transitively modified by this function.
    modified_memory: SetDomain<QualifiedId<StructId>>,
    directly_modified_memory: SetDomain<QualifiedId<StructId>>,
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
    fn to_summary(&self, state: UsageState) -> UsageState {
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
        data: FunctionData,
    ) -> FunctionData {
        let mut initial_state = UsageState::default();
        let func_target = FunctionTarget::new(func_env, &data);
        func_target.get_modify_targets().keys().for_each(|target| {
            initial_state.modified_memory.insert(*target);
        });

        let cache = SummaryCache::new(targets, func_env.module_env.env);
        let analysis = MemoryUsageAnalysis { cache };
        analysis.summarize(func_env, initial_state, data)
    }

    fn name(&self) -> String {
        "usage_analysis".to_string()
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
                Function(mid, fid, _) => {
                    if let Some(summary) = self.cache.get::<UsageState>(mid.qualified(*fid)) {
                        state.modified_memory.extend(&summary.modified_memory.0);
                        state.used_memory.extend(&summary.used_memory.0);
                    }
                }
                MoveTo(mid, sid, _) | MoveFrom(mid, sid, _) | BorrowGlobal(mid, sid, _) => {
                    state.modified_memory.insert(mid.qualified(*sid));
                    state.directly_modified_memory.insert(mid.qualified(*sid));
                    state.used_memory.insert(mid.qualified(*sid));
                }
                Exists(mid, sid, _) | GetGlobal(mid, sid, _) => {
                    state.used_memory.insert(mid.qualified(*sid));
                }
                _ => {}
            }
        }
    }
}
