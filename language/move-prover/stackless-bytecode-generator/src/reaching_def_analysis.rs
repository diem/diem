// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    dataflow_analysis::{AbstractDomain, DataflowAnalysis, JoinResult, TransferFunctions},
    function_target::{FunctionTarget, FunctionTargetData},
    function_target_pipeline::{FunctionTargetProcessor, FunctionTargetsHolder},
    stackless_bytecode::{Bytecode, Constant, TempIndex},
    stackless_control_flow_graph::{BlockId, StacklessControlFlowGraph},
};
use itertools::Itertools;
use spec_lang::env::FunctionEnv;
use std::collections::{BTreeMap, BTreeSet};
use vm::file_format::CodeOffset;

/// The definitions we are capturing. Currently we only capture constants
/// and aliases (assignment).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Def {
    Const(Constant),
    Alias(TempIndex),
}

/// The annotation for reaching definitions. For each code position, we have a map of local
/// indices to the set of definitions reaching the code position.
#[derive(Default)]
pub struct ReachingDefAnnotation(BTreeMap<CodeOffset, BTreeMap<TempIndex, BTreeSet<Def>>>);

pub struct ReachingDefProcessor();

impl ReachingDefProcessor {
    pub fn new() -> Box<Self> {
        Box::new(ReachingDefProcessor())
    }
}

impl FunctionTargetProcessor for ReachingDefProcessor {
    fn process(
        &self,
        _targets: &mut FunctionTargetsHolder,
        func_env: &FunctionEnv<'_>,
        mut data: FunctionTargetData,
    ) -> FunctionTargetData {
        let defs = if func_env.is_native() {
            BTreeMap::new()
        } else {
            let cfg = StacklessControlFlowGraph::new_forward(&data.code);
            let mut analyzer = ReachingDefAnalysis {
                _target: FunctionTarget::new(func_env, &data),
            };
            let mut block_state_map = analyzer.analyze_function(
                ReachingDefState {
                    map: BTreeMap::new(),
                },
                &data.code,
                &cfg,
            );
            // Convert the block state map into a map for each code offset. We achieve this by
            // replaying execution of the instructions of each individual block, based on the pre
            // state. Because the analyzes converged to a fixpoint, the result is sound.
            let mut res = BTreeMap::new();
            for block_id in cfg.blocks() {
                let mut state = block_state_map.remove(&block_id).expect("basic block").pre;
                for code_offset in cfg.instr_indexes(block_id) {
                    res.insert(code_offset, state.map.clone());
                    state = analyzer.execute(state, &data.code[code_offset as usize], code_offset);
                }
            }
            res
        };
        data.annotations.set(ReachingDefAnnotation(defs));
        data
    }
}

struct ReachingDefAnalysis<'a> {
    _target: FunctionTarget<'a>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct ReachingDefState {
    map: BTreeMap<TempIndex, BTreeSet<Def>>,
}

impl<'a> ReachingDefAnalysis<'a> {
    fn execute(
        &mut self,
        pre: ReachingDefState,
        instr: &Bytecode,
        _idx: CodeOffset,
    ) -> ReachingDefState {
        use Bytecode::*;
        let mut post = pre;
        match instr {
            Assign(_, dst, src, _) => {
                post.def_alias(*dst, *src);
            }
            Load(_, dst, cons) => {
                post.def_const(*dst, cons.clone());
            }
            Call(_, dsts, ..) => {
                for dst in dsts {
                    post.kill(*dst);
                }
            }
            _ => {}
        }
        post
    }
}

impl<'a> TransferFunctions for ReachingDefAnalysis<'a> {
    type State = ReachingDefState;
    type AnalysisError = ();

    fn execute_block(
        &mut self,
        block_id: BlockId,
        pre_state: Self::State,
        instrs: &[Bytecode],
        cfg: &StacklessControlFlowGraph,
    ) -> Result<Self::State, Self::AnalysisError> {
        let mut state = pre_state;
        for offset in cfg.instr_indexes(block_id) {
            let instr = &instrs[offset as usize];
            state = self.execute(state, instr, offset);
        }
        Ok(state)
    }
}

impl<'a> DataflowAnalysis for ReachingDefAnalysis<'a> {}

impl AbstractDomain for ReachingDefState {
    fn join(&mut self, other: &Self) -> JoinResult {
        let mut result = JoinResult::Unchanged;
        for (idx, other_defs) in &other.map {
            if !self.map.contains_key(idx) {
                self.map.insert(*idx, other_defs.clone());
                result = JoinResult::Changed;
            } else {
                let defs = self.map.get_mut(idx).unwrap();
                for d in other_defs {
                    if defs.insert(d.clone()) {
                        result = JoinResult::Changed;
                    }
                }
            }
        }
        result
    }
}

impl ReachingDefState {
    fn def_alias(&mut self, dest: TempIndex, src: TempIndex) {
        let set = self.map.entry(dest).or_insert_with(BTreeSet::new);
        set.clear();
        set.insert(Def::Alias(src));
    }

    fn def_const(&mut self, dest: TempIndex, cons: Constant) {
        let set = self.map.entry(dest).or_insert_with(BTreeSet::new);
        set.clear();
        set.insert(Def::Const(cons));
    }

    fn kill(&mut self, dest: TempIndex) {
        self.map.remove(&dest);
    }
}

// =================================================================================================
// Formatting

/// Format a reaching definition annotation.
pub fn format_reaching_def_annotation(
    target: &FunctionTarget<'_>,
    code_offset: CodeOffset,
) -> Option<String> {
    if let Some(ReachingDefAnnotation(map)) =
        target.get_annotations().get::<ReachingDefAnnotation>()
    {
        if let Some(map_at) = map.get(&code_offset) {
            let mut res = map_at
                .iter()
                .map(|(idx, defs)| {
                    let name = target.get_local_name(*idx);
                    format!(
                        "{} -> {{{}}}",
                        name.display(target.symbol_pool()),
                        defs.iter()
                            .map(|def| {
                                match def {
                                    Def::Alias(a) => format!(
                                        "{}",
                                        target.get_local_name(*a).display(target.symbol_pool())
                                    ),
                                    Def::Const(c) => format!("{}", c),
                                }
                            })
                            .join(", ")
                    )
                })
                .join(", ");
            res.insert_str(0, "reach: ");
            return Some(res);
        }
    }
    None
}
