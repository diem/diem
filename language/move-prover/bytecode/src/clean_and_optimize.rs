// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Final phase of cleanup and optimization.

use crate::{
    dataflow_analysis::{AbstractDomain, DataflowAnalysis, JoinResult, TransferFunctions},
    function_target::{FunctionTarget, FunctionTargetData},
    function_target_pipeline::{FunctionTargetProcessor, FunctionTargetsHolder},
    stackless_bytecode::{BorrowNode, Bytecode, Operation},
    stackless_control_flow_graph::StacklessControlFlowGraph,
};
use spec_lang::env::FunctionEnv;
use std::collections::BTreeSet;
use vm::file_format::CodeOffset;

pub struct CleanAndOptimizeProcessor();

impl CleanAndOptimizeProcessor {
    pub fn new() -> Box<Self> {
        Box::new(Self())
    }
}

impl FunctionTargetProcessor for CleanAndOptimizeProcessor {
    fn process(
        &self,
        _targets: &mut FunctionTargetsHolder,
        func_env: &FunctionEnv<'_>,
        mut data: FunctionTargetData,
    ) -> FunctionTargetData {
        if func_env.is_native() {
            // Nothing to do
            return data;
        }

        // Run optimizer
        let instrs = std::mem::take(&mut data.code);
        let new_instrs = Optimizer {
            _target: &FunctionTarget::new(func_env, &data),
        }
        .run(instrs);
        data.code = new_instrs;
        data
    }

    fn name(&self) -> String {
        "clean_and_optimize".to_string()
    }
}

// Analysis
// ========

/// A data flow analysis state used for optimization analysis. Currently it tracks the nodes
/// which have been updated but not yet written back.
#[derive(Clone, Default)]
struct AnalysisState {
    unwritten: BTreeSet<BorrowNode>,
}

impl AbstractDomain for AnalysisState {
    fn join(&mut self, other: &Self) -> JoinResult {
        let n = self.unwritten.len();
        self.unwritten.extend(other.unwritten.iter().cloned());
        if self.unwritten.len() == n {
            JoinResult::Unchanged
        } else {
            JoinResult::Changed
        }
    }
}

struct Optimizer<'a> {
    _target: &'a FunctionTarget<'a>,
}

impl<'a> TransferFunctions for Optimizer<'a> {
    type State = AnalysisState;
    const BACKWARD: bool = false;

    fn execute(&self, state: &mut AnalysisState, instr: &Bytecode, _offset: CodeOffset) {
        use BorrowNode::*;
        use Bytecode::*;
        use Operation::*;
        if let Call(_, _, oper, srcs) = instr {
            match oper {
                WriteRef => {
                    state.unwritten.insert(Reference(srcs[0]));
                }
                WriteBack(Reference(dest)) => {
                    if state.unwritten.contains(&Reference(srcs[0])) {
                        state.unwritten.insert(Reference(*dest));
                    }
                }
                _ => {}
            }
        }
    }
}

impl<'a> DataflowAnalysis for Optimizer<'a> {}

// Transformation
// ==============

impl<'a> Optimizer<'a> {
    fn run(&mut self, instrs: Vec<Bytecode>) -> Vec<Bytecode> {
        // Rum Analysis
        let cfg = StacklessControlFlowGraph::new_forward(&instrs);
        let state = self.analyze_function(AnalysisState::default(), &instrs, &cfg);
        let data = self.state_per_instruction(state, &instrs, &cfg, |before, _| before.clone());

        // Transform code.
        let mut new_instrs = vec![];
        for (code_offset, instr) in instrs.into_iter().enumerate() {
            use BorrowNode::*;
            use Bytecode::*;
            use Operation::*;
            if !new_instrs.is_empty() {
                // Perform peephole optimization
                match (&new_instrs[new_instrs.len() - 1], &instr) {
                    (Call(_, _, UnpackRef, srcs1), Call(_, _, PackRef, srcs2))
                        if srcs1[0] == srcs2[0] =>
                    {
                        // skip this redundant unpack/pack pair.
                        new_instrs.pop();
                        continue;
                    }
                    _ => {}
                }
            }
            // Remove unnecessary WriteBack
            if let Call(_, _, WriteBack(_), srcs) = &instr {
                if let Some(unwritten) =
                    data.get(&(code_offset as CodeOffset)).map(|d| &d.unwritten)
                {
                    if !unwritten.contains(&Reference(srcs[0])) {
                        continue;
                    }
                }
            }
            new_instrs.push(instr);
        }
        new_instrs
    }
}
