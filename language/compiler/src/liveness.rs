// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bytecode_verifier::{
    absint::{AbstractInterpreter, SetDomain, TransferFunctions},
    control_flow_graph::BackwardControlFlowGraph,
    VerifiedModule,
};
use vm::{
    access::ModuleAccess,
    file_format::{Bytecode, CompiledModule, FunctionDefinition, LocalIndex},
    views::FunctionDefinitionView,
};

#[derive(Debug)]
pub enum LivenessError {
    /// Emitted when code writes to a local, but never reads the result
    DeadStore { local: LocalIndex, index: usize },
    /// Emitted when code copies from a local, but could have safely moved instead
    RedundantCopy { local: LocalIndex, index: usize },
}

pub struct LivenessAnalysis {
    errors: Vec<LivenessError>,
}

impl TransferFunctions for LivenessAnalysis {
    type State = SetDomain<LocalIndex>;
    type AnalysisError = ();

    fn execute(
        &mut self,
        pre: &mut Self::State,
        instr: &Bytecode,
        index: usize,
        _last_index: usize,
    ) -> Result<(), Self::AnalysisError> {
        println!("analyzing instr {:?}", instr);
        match instr {
            Bytecode::MoveLoc(idx) | Bytecode::BorrowLoc(idx) => {
                pre.0.insert(*idx);
                Ok(())
            }
            Bytecode::CopyLoc(idx) => {
                if pre.0.insert(*idx) {
                    self.errors
                        .push(LivenessError::RedundantCopy { local: *idx, index })
                }
                Ok(())
            }
            Bytecode::StLoc(idx) => {
                if !pre.0.remove(idx) {
                    self.errors
                        .push(LivenessError::DeadStore { local: *idx, index })
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

impl AbstractInterpreter for LivenessAnalysis {}

impl LivenessAnalysis {

    pub fn analyze(module: &VerifiedModule) -> Vec<LivenessError> {
        println!("Running liveness analysis on {:?}", module.name());
        Self::analyze_module(module.as_module())
    }

    fn analyze_module(module: &CompiledModule) -> Vec<LivenessError> {
        println!("Analyzing module");
        module
            .function_defs()
            .iter()
            .enumerate()
            .map(move |(_idx, function_definition)| {
                Self::analyze_function(function_definition, module)
            })
            .flatten()
            .collect()
    }

    fn analyze_function(
        function_definition: &FunctionDefinition,
        module: &CompiledModule,
    ) -> Vec<LivenessError> {
        println!("Running liveness on function");
        let code = &function_definition.code.code;
        let cfg = BackwardControlFlowGraph::new(code);
        let function_definition_view = FunctionDefinitionView::new(module, function_definition);

        let mut analyzer = LivenessAnalysis { errors: Vec::new() };
        let initial_state = SetDomain::<LocalIndex>::empty();
        analyzer.analyze_function(initial_state, &function_definition_view, &cfg);
        // TODO: grab inv_map an look for unused parameters
        analyzer.errors
    }
}
