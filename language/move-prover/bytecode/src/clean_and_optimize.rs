// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Final phase of cleanup and optimization.

use crate::{
    dataflow_analysis::{DataflowAnalysis, TransferFunctions},
    function_target::{FunctionData, FunctionTarget},
    function_target_pipeline::{FunctionTargetProcessor, FunctionTargetsHolder},
    options::ProverOptions,
    stackless_bytecode::{BorrowNode, Bytecode, Operation},
    stackless_control_flow_graph::StacklessControlFlowGraph,
};
use move_binary_format::file_format::CodeOffset;
use move_model::{
    model::FunctionEnv,
    native::{EVENT_EMIT_EVENT, VECTOR_BORROW_MUT},
};

use crate::dataflow_domains::{AbstractDomain, JoinResult};
use std::collections::BTreeSet;

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
        mut data: FunctionData,
    ) -> FunctionData {
        if func_env.is_native() {
            // Nothing to do
            return data;
        }

        // Run optimizer
        let options = ProverOptions::get(func_env.module_env.env);
        let instrs = std::mem::take(&mut data.code);
        let new_instrs = Optimizer {
            options: &*options,
            target: &FunctionTarget::new(func_env, &data),
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
#[derive(Debug, Clone, Default, Eq, PartialEq, PartialOrd)]
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
    options: &'a ProverOptions,
    target: &'a FunctionTarget<'a>,
}

impl<'a> TransferFunctions for Optimizer<'a> {
    type State = AnalysisState;
    const BACKWARD: bool = false;

    fn execute(&self, state: &mut AnalysisState, instr: &Bytecode, _offset: CodeOffset) {
        use BorrowNode::*;
        use Bytecode::*;
        use Operation::*;
        if let Call(_, _, oper, srcs, _) = instr {
            match oper {
                WriteRef => {
                    state.unwritten.insert(Reference(srcs[0]));
                }
                WriteBack(Reference(dest), ..) => {
                    if state.unwritten.contains(&Reference(srcs[0])) {
                        state.unwritten.insert(Reference(*dest));
                    }
                }
                Function(mid, fid, _) => {
                    let callee_env = &self
                        .target
                        .global_env()
                        .get_function_qid(mid.qualified(*fid));
                    let has_effect = if !self.options.for_interpretation
                        && callee_env.is_native_or_intrinsic()
                    {
                        // Exploit knowledge about builtin functions
                        let pool = callee_env.symbol_pool();
                        !matches!(
                            format!(
                                "{}::{}",
                                callee_env.module_env.get_name().display_full(pool),
                                callee_env.get_name().display(pool)
                            )
                            .as_str(),
                            VECTOR_BORROW_MUT | EVENT_EMIT_EVENT
                        )
                    } else {
                        true
                    };

                    // Mark &mut parameters to functions as unwritten.
                    if has_effect {
                        for src in srcs {
                            if self.target.get_local_type(*src).is_mutable_reference() {
                                state.unwritten.insert(Reference(*src));
                            }
                        }
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

            let is_unwritten = |code_offset: CodeOffset, node: &BorrowNode| {
                if let Some(unwritten) = data.get(&code_offset).map(|d| &d.unwritten) {
                    unwritten.contains(node)
                } else {
                    // No data for this node, so assume it is unwritten.
                    true
                }
            };
            if !new_instrs.is_empty() {
                // Perform peephole optimization
                match (&new_instrs[new_instrs.len() - 1], &instr) {
                    (Call(_, _, UnpackRef, srcs1, _), Call(_, _, PackRef, srcs2, _))
                        if srcs1[0] == srcs2[0] =>
                    {
                        // skip this redundant unpack/pack pair.
                        new_instrs.pop();
                        continue;
                    }
                    (Call(_, dests, IsParent(..), srcs, _), Branch(_, _, _, tmp))
                        if dests[0] == *tmp
                            && !is_unwritten(code_offset as CodeOffset, &Reference(srcs[0])) =>
                    {
                        // skip this obsolete IsParent check
                        new_instrs.pop();
                        continue;
                    }
                    _ => {}
                }
            }
            // Remove unnecessary WriteBack
            match &instr {
                Call(_, _, WriteBack(..), srcs, _)
                    if !is_unwritten(code_offset as CodeOffset, &Reference(srcs[0])) =>
                {
                    // skip this obsolete WriteBack
                    continue;
                }
                _ => {}
            }
            new_instrs.push(instr);
        }
        new_instrs
    }
}
