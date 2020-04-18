// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    dataflow_analysis::{
        AbstractDomain, DataflowAnalysis, JoinResult, StateMap, TransferFunctions,
    },
    function_target::{FunctionTarget, FunctionTargetData},
    function_target_pipeline::{FunctionTargetProcessor, FunctionTargetsHolder},
    stackless_bytecode::{BranchCond, Bytecode, Operation, TempIndex},
    stackless_control_flow_graph::{BlockId, StacklessControlFlowGraph},
};
use itertools::Itertools;
use spec_lang::env::FunctionEnv;
use std::collections::{BTreeMap, BTreeSet};
use vm::file_format::CodeOffset;

/// The annotation for live variable analysis. For each code position, we have a set of local
/// variable indices that are live just before the code offset, i.e. these variables are used
/// before being overwritten.
#[derive(Default)]
pub struct LiveVarAnnotation(BTreeMap<CodeOffset, BTreeSet<TempIndex>>);

pub struct LiveVarAnalysisProcessor();

impl LiveVarAnalysisProcessor {
    pub fn new() -> Box<Self> {
        Box::new(LiveVarAnalysisProcessor())
    }
}

impl FunctionTargetProcessor for LiveVarAnalysisProcessor {
    fn process(
        &self,
        _targets: &mut FunctionTargetsHolder,
        func_env: &FunctionEnv<'_>,
        mut data: FunctionTargetData,
    ) -> FunctionTargetData {
        let offset_to_live_refs = if func_env.is_native() {
            // Native functions have no byte code.
            LiveVarAnnotation(BTreeMap::new())
        } else {
            let cfg = StacklessControlFlowGraph::new_backward(&data.code);
            LiveVarAnnotation(LiveVarAnalysis::analyze(&cfg, &data.code))
        };
        // Annotate function target with computed life variable data.
        data.annotations
            .set::<LiveVarAnnotation>(offset_to_live_refs);
        data
    }
}

struct LiveVarAnalysis();

#[derive(Debug, Clone, Eq, PartialEq)]
struct LiveVarState {
    livevars: BTreeSet<TempIndex>,
}

impl LiveVarState {
    fn remove(&mut self, vars: &[TempIndex]) -> bool {
        let mut removed = false;
        for v in vars {
            if self.livevars.remove(v) {
                removed = true;
            }
        }
        removed
    }

    fn insert(&mut self, vars: Vec<TempIndex>) {
        for v in vars {
            self.livevars.insert(v);
        }
    }

    fn reset(&mut self) {
        self.livevars = BTreeSet::new();
    }
}

impl LiveVarAnalysis {
    fn analyze(
        cfg: &StacklessControlFlowGraph,
        instrs: &[Bytecode],
    ) -> BTreeMap<CodeOffset, BTreeSet<TempIndex>> {
        let initial_state = LiveVarState {
            livevars: BTreeSet::new(),
        };
        let mut analyzer = LiveVarAnalysis {};
        let state_map = analyzer.analyze_function(initial_state, &instrs, cfg);
        analyzer.post_process(cfg, instrs, state_map)
    }

    fn post_process(
        &mut self,
        cfg: &StacklessControlFlowGraph,
        instrs: &[Bytecode],
        state_map: StateMap<LiveVarState>,
    ) -> BTreeMap<CodeOffset, BTreeSet<TempIndex>> {
        let mut result = BTreeMap::new();
        for (block_id, block_state) in state_map {
            let mut state = block_state.pre;
            for offset in cfg.instr_indexes(block_id).rev() {
                let instr = &instrs[offset as usize];
                state = self.execute(state, instr, offset);
                result.insert(offset, state.livevars.clone());
            }
        }
        result
    }

    fn execute(&mut self, pre: LiveVarState, instr: &Bytecode, _idx: CodeOffset) -> LiveVarState {
        use Bytecode::*;
        let mut post = pre;
        match instr {
            Assign(_, dst, src, _) => {
                if post.remove(&[*dst]) {
                    post.insert(vec![*src]);
                }
            }
            Load(_, dst, _) => {
                post.remove(&[*dst]);
            }
            Call(_, dsts, op, srcs) => {
                use Operation::*;
                let removed = match op {
                    Abort => {
                        post.reset();
                        true
                    }
                    _ => post.remove(dsts),
                };
                if removed {
                    post.insert(srcs.clone());
                }
            }
            Ret(_, srcs) => {
                post.insert(srcs.clone());
            }
            Branch(_, _, cond) => {
                use BranchCond::*;
                match cond {
                    True(src) | False(src) => post.insert(vec![*src]),
                    Always => {}
                }
            }
            _ => {}
        }
        post
    }
}

impl TransferFunctions for LiveVarAnalysis {
    type State = LiveVarState;

    fn execute_block(
        &mut self,
        block_id: BlockId,
        pre_state: Self::State,
        instrs: &[Bytecode],
        cfg: &StacklessControlFlowGraph,
    ) -> Self::State {
        let mut state = pre_state;
        for offset in cfg.instr_indexes(block_id).rev() {
            let instr = &instrs[offset as usize];
            state = self.execute(state, instr, offset);
        }
        state
    }
}

impl DataflowAnalysis for LiveVarAnalysis {}

impl AbstractDomain for LiveVarState {
    fn join(&mut self, other: &Self) -> JoinResult {
        let mut result = JoinResult::Unchanged;
        for livevar in &other.livevars {
            if !self.livevars.contains(livevar) {
                self.livevars.insert(*livevar);
                result = JoinResult::Changed;
            }
        }
        result
    }
}

// =================================================================================================
// Formatting

/// Format a live variable annotation.
pub fn format_livevar_annotation(
    target: &FunctionTarget<'_>,
    code_offset: CodeOffset,
) -> Option<String> {
    if let Some(LiveVarAnnotation(map)) = target.get_annotations().get::<LiveVarAnnotation>() {
        if let Some(map_at) = map.get(&code_offset) {
            let mut res = map_at
                .iter()
                .map(|idx| {
                    let name = target.get_local_name(*idx);
                    format!("{}", name.display(target.symbol_pool()),)
                })
                .join(", ");
            res.insert_str(0, "live vars: ");
            return Some(res);
        }
    }
    None
}
