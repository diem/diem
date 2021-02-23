// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::cli::Options;
use bytecode::{
    borrow_analysis::BorrowAnalysisProcessor, borrow_analysis_v2,
    clean_and_optimize::CleanAndOptimizeProcessor, eliminate_imm_refs::EliminateImmRefsProcessor,
    function_target_pipeline::FunctionTargetProcessor, livevar_analysis::LiveVarAnalysisProcessor,
    loop_analysis::LoopAnalysisProcessor, memory_instrumentation::MemoryInstrumentationProcessor,
    memory_instrumentation_v2, mut_ref_instrumentation::MutRefInstrumenter,
    reaching_def_analysis::ReachingDefProcessor, usage_analysis::UsageProcessor,
    verification_analysis::VerificationAnalysisProcessor,
};

/// Allows client to decide between one of two pipelines for ease of benchmarking
pub fn pipelines(options: &Options) -> Vec<Box<dyn FunctionTargetProcessor>> {
    if !options.experimental_pipeline {
        vec![
            EliminateImmRefsProcessor::new(),
            MutRefInstrumenter::new(),
            ReachingDefProcessor::new(),
            LiveVarAnalysisProcessor::new(),
            BorrowAnalysisProcessor::new(),
            MemoryInstrumentationProcessor::new(),
            CleanAndOptimizeProcessor::new(),
            UsageProcessor::new(),
            VerificationAnalysisProcessor::new(),
            LoopAnalysisProcessor::new(),
        ]
    } else {
        // Enter your pipeline here
        vec![
            EliminateImmRefsProcessor::new(),
            MutRefInstrumenter::new(),
            ReachingDefProcessor::new(),
            LiveVarAnalysisProcessor::new(),
            borrow_analysis_v2::BorrowAnalysisProcessor::new(),
            memory_instrumentation_v2::MemoryInstrumentationProcessor::new(),
            CleanAndOptimizeProcessor::new(),
            UsageProcessor::new(),
            VerificationAnalysisProcessor::new(),
            LoopAnalysisProcessor::new(),
        ]
    }
}
