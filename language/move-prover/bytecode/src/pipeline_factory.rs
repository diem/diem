// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    borrow_analysis::BorrowAnalysisProcessor,
    clean_and_optimize::CleanAndOptimizeProcessor,
    data_invariant_instrumentation::DataInvariantInstrumentationProcessor,
    debug_instrumentation::DebugInstrumenter,
    eliminate_imm_refs::EliminateImmRefsProcessor,
    function_target_pipeline::{FunctionTargetPipeline, FunctionTargetProcessor},
    global_invariant_analysis::GlobalInvariantAnalysisProcessor,
    global_invariant_instrumentation::GlobalInvariantInstrumentationProcessor,
    inconsistency_check::InconsistencyCheckInstrumenter,
    livevar_analysis::LiveVarAnalysisProcessor,
    loop_analysis::LoopAnalysisProcessor,
    memory_instrumentation::MemoryInstrumentationProcessor,
    mono_analysis::MonoAnalysisProcessor,
    mut_ref_instrumentation::MutRefInstrumenter,
    mutation_tester::MutationTester,
    options::ProverOptions,
    reaching_def_analysis::ReachingDefProcessor,
    spec_instrumentation::SpecInstrumentationProcessor,
    usage_analysis::UsageProcessor,
    verification_analysis::VerificationAnalysisProcessor,
};

pub fn default_pipeline_with_options(options: &ProverOptions) -> FunctionTargetPipeline {
    // NOTE: the order of these processors is import!
    let mut processors: Vec<Box<dyn FunctionTargetProcessor>> = vec![
        DebugInstrumenter::new(),
        // transformation and analysis
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
        // spec instrumentation
        SpecInstrumentationProcessor::new(),
        DataInvariantInstrumentationProcessor::new(),
        GlobalInvariantAnalysisProcessor::new(),
        GlobalInvariantInstrumentationProcessor::new(),
    ];

    if options.mutation {
        // pass which may do nothing
        processors.push(MutationTester::new());
    }

    // run monomorphization when the backend does not have mono support.else
    //
    // TODO: not running the MonoAnalysisProcessor does not break the
    // transformation pipeline but will output a bpl file that is uncompilable
    // in Boogie. Solving this issue will require some deep understanding of
    // the mono backend on both sides, which we defer to a later stage.
    if !options.boogie_poly {
        processors.push(MonoAnalysisProcessor::new());
    }

    // inconsistency check instrumentation should be the last one in the pipeline
    if options.check_inconsistency {
        processors.push(InconsistencyCheckInstrumenter::new());
    }

    let mut res = FunctionTargetPipeline::default();
    for p in processors {
        res.add_processor(p);
    }
    res
}

pub fn default_pipeline() -> FunctionTargetPipeline {
    default_pipeline_with_options(&ProverOptions::default())
}

pub fn experimental_pipeline() -> FunctionTargetPipeline {
    // Enter your pipeline here
    let processors: Vec<Box<dyn FunctionTargetProcessor>> = vec![
        DebugInstrumenter::new(),
        // transformation and analysis
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
        // spec instrumentation
        SpecInstrumentationProcessor::new(),
        DataInvariantInstrumentationProcessor::new(),
        GlobalInvariantAnalysisProcessor::new(),
        GlobalInvariantInstrumentationProcessor::new(),
        // optimization
        MonoAnalysisProcessor::new(),
    ];

    let mut res = FunctionTargetPipeline::default();
    for p in processors {
        res.add_processor(p);
    }
    res
}
