// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use bytecode::{
    borrow_analysis::BorrowAnalysisProcessor, clean_and_optimize::CleanAndOptimizeProcessor,
    eliminate_imm_refs::EliminateImmRefsProcessor, eliminate_mut_refs::EliminateMutRefsProcessor,
    function_target_pipeline::FunctionTargetProcessor, livevar_analysis::LiveVarAnalysisProcessor,
    memory_instrumentation::MemoryInstrumentationProcessor,
    reaching_def_analysis::ReachingDefProcessor, usage_analysis::UsageProcessor,
    verification_analysis::VerificationAnalysisProcessor,
};

/// Allows client to decide between one of two pipelines for ease of benchmarking
pub fn pipelines(experimental_pipeline: bool) -> Vec<Box<dyn FunctionTargetProcessor + 'static>> {
    println!("option is {}", experimental_pipeline);
    let vec: Vec<Box<dyn FunctionTargetProcessor + 'static>> = if experimental_pipeline == false {
        vec![
            EliminateImmRefsProcessor::new(),
            EliminateMutRefsProcessor::new(),
            ReachingDefProcessor::new(),
            LiveVarAnalysisProcessor::new(),
            BorrowAnalysisProcessor::new(),
            MemoryInstrumentationProcessor::new(),
            CleanAndOptimizeProcessor::new(),
            UsageProcessor::new(),
            VerificationAnalysisProcessor::new(),
        ]
    }
    // Enter your pipeline here
    else {
        panic!("No experimental pipeline set");
    };
    vec
}
