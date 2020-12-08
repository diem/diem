// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::anyhow;
use std::path::Path;

use codespan_reporting::term::termcolor::Buffer;

use bytecode::{
    borrow_analysis::BorrowAnalysisProcessor,
    clean_and_optimize::CleanAndOptimizeProcessor,
    eliminate_imm_refs::EliminateImmRefsProcessor,
    eliminate_mut_refs::EliminateMutRefsProcessor,
    function_target_pipeline::{FunctionTargetPipeline, FunctionTargetsHolder},
    livevar_analysis::LiveVarAnalysisProcessor,
    memory_instrumentation::MemoryInstrumentationProcessor,
    print_targets_for_test,
    reaching_def_analysis::ReachingDefProcessor,
};
use move_prover_test_utils::{baseline_test::verify_or_update_baseline, extract_test_directives};
use spec_lang::{env::GlobalEnv, run_spec_lang_compiler};

fn get_tested_transformation_pipeline(
    dir_name: &str,
) -> anyhow::Result<Option<FunctionTargetPipeline>> {
    match dir_name {
        "from_move" => Ok(None),
        "eliminate_imm_refs" => {
            let mut pipeline = FunctionTargetPipeline::default();
            pipeline.add_processor(Box::new(EliminateImmRefsProcessor {}));
            Ok(Some(pipeline))
        }
        "eliminate_mut_refs" => {
            let mut pipeline = FunctionTargetPipeline::default();
            pipeline.add_processor(Box::new(EliminateImmRefsProcessor {}));
            pipeline.add_processor(Box::new(EliminateMutRefsProcessor {}));
            Ok(Some(pipeline))
        }
        "reaching_def" => {
            let mut pipeline = FunctionTargetPipeline::default();
            pipeline.add_processor(Box::new(EliminateImmRefsProcessor {}));
            pipeline.add_processor(Box::new(EliminateMutRefsProcessor {}));
            pipeline.add_processor(Box::new(ReachingDefProcessor {}));
            Ok(Some(pipeline))
        }
        "livevar" => {
            let mut pipeline = FunctionTargetPipeline::default();
            pipeline.add_processor(Box::new(EliminateImmRefsProcessor {}));
            pipeline.add_processor(Box::new(EliminateMutRefsProcessor {}));
            pipeline.add_processor(Box::new(ReachingDefProcessor {}));
            pipeline.add_processor(Box::new(LiveVarAnalysisProcessor {}));
            Ok(Some(pipeline))
        }
        "borrow" => {
            let mut pipeline = FunctionTargetPipeline::default();
            pipeline.add_processor(Box::new(EliminateImmRefsProcessor {}));
            pipeline.add_processor(Box::new(EliminateMutRefsProcessor {}));
            pipeline.add_processor(Box::new(ReachingDefProcessor {}));
            pipeline.add_processor(Box::new(LiveVarAnalysisProcessor {}));
            pipeline.add_processor(Box::new(BorrowAnalysisProcessor {}));
            Ok(Some(pipeline))
        }
        "memory_instr" => {
            let mut pipeline = FunctionTargetPipeline::default();
            pipeline.add_processor(Box::new(EliminateImmRefsProcessor {}));
            pipeline.add_processor(Box::new(EliminateMutRefsProcessor {}));
            pipeline.add_processor(Box::new(ReachingDefProcessor {}));
            pipeline.add_processor(Box::new(LiveVarAnalysisProcessor {}));
            pipeline.add_processor(Box::new(BorrowAnalysisProcessor {}));
            pipeline.add_processor(Box::new(MemoryInstrumentationProcessor {}));
            Ok(Some(pipeline))
        }
        "clean_and_optimize" => {
            let mut pipeline = FunctionTargetPipeline::default();
            pipeline.add_processor(Box::new(EliminateImmRefsProcessor {}));
            pipeline.add_processor(Box::new(EliminateMutRefsProcessor {}));
            pipeline.add_processor(Box::new(ReachingDefProcessor {}));
            pipeline.add_processor(Box::new(LiveVarAnalysisProcessor {}));
            pipeline.add_processor(Box::new(BorrowAnalysisProcessor {}));
            pipeline.add_processor(Box::new(MemoryInstrumentationProcessor {}));
            pipeline.add_processor(Box::new(CleanAndOptimizeProcessor {}));
            Ok(Some(pipeline))
        }

        _ => Err(anyhow!(
            "the sub-directory `{}` has no associated pipeline to test",
            dir_name
        )),
    }
}

fn test_runner(path: &Path) -> datatest_stable::Result<()> {
    let mut sources = extract_test_directives(path, "// dep:")?;
    sources.push(path.to_string_lossy().to_string());
    let env: GlobalEnv = run_spec_lang_compiler(sources, vec![], Some("0x2345467"))?;
    let out = if env.has_errors() {
        let mut error_writer = Buffer::no_color();
        env.report_errors(&mut error_writer);
        String::from_utf8_lossy(&error_writer.into_inner()).to_string()
    } else {
        let dir_name = path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|p| p.to_str())
            .ok_or_else(|| anyhow!("bad file name"))?;
        let pipeline_opt = get_tested_transformation_pipeline(dir_name)?;

        // Initialize and print function targets
        let mut text = String::new();
        let mut targets = FunctionTargetsHolder::default();
        for module_env in env.get_modules() {
            for func_env in module_env.get_functions() {
                targets.add_target(&func_env);
            }
        }
        text += &print_targets_for_test(&env, "initial translation from Move", &targets);

        // Run pipeline if any
        if let Some(pipeline) = pipeline_opt {
            pipeline.run(&env, &mut targets, None);
            text +=
                &print_targets_for_test(&env, &format!("after pipeline `{}`", dir_name), &targets);
        }

        text
    };
    let baseline_path = path.with_extension("exp");
    verify_or_update_baseline(baseline_path.as_path(), &out)?;
    Ok(())
}

datatest_stable::harness!(test_runner, "tests", r".*\.move");
