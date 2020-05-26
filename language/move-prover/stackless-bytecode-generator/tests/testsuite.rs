// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::anyhow;
use std::path::Path;

use codespan_reporting::term::termcolor::Buffer;

use move_core_types::fs::AFS;
use spec_lang::{env::GlobalEnv, run_spec_lang_compiler};
use stackless_bytecode_generator::{
    borrow_analysis::BorrowAnalysisProcessor,
    eliminate_imm_refs::EliminateImmRefsProcessor,
    eliminate_mut_refs::EliminateMutRefsProcessor,
    function_target_pipeline::{FunctionTargetPipeline, FunctionTargetsHolder},
    lifetime_analysis::LifetimeAnalysisProcessor,
    livevar_analysis::LiveVarAnalysisProcessor,
    packref_analysis::PackrefAnalysisProcessor,
    reaching_def_analysis::ReachingDefProcessor,
    writeback_analysis::WritebackAnalysisProcessor,
};
use test_utils::{baseline_test::verify_or_update_baseline, extract_test_directives};

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
        "livevar" => {
            let mut pipeline = FunctionTargetPipeline::default();
            pipeline.add_processor(Box::new(LiveVarAnalysisProcessor {}));
            Ok(Some(pipeline))
        }
        "borrow" => {
            let mut pipeline = FunctionTargetPipeline::default();
            pipeline.add_processor(Box::new(EliminateImmRefsProcessor {}));
            pipeline.add_processor(Box::new(LiveVarAnalysisProcessor {}));
            pipeline.add_processor(Box::new(BorrowAnalysisProcessor {}));
            Ok(Some(pipeline))
        }
        "writeback" => {
            let mut pipeline = FunctionTargetPipeline::default();
            pipeline.add_processor(Box::new(EliminateImmRefsProcessor {}));
            pipeline.add_processor(Box::new(LiveVarAnalysisProcessor {}));
            pipeline.add_processor(Box::new(BorrowAnalysisProcessor {}));
            pipeline.add_processor(Box::new(WritebackAnalysisProcessor {}));
            Ok(Some(pipeline))
        }
        "packref" => {
            let mut pipeline = FunctionTargetPipeline::default();
            pipeline.add_processor(Box::new(EliminateImmRefsProcessor {}));
            pipeline.add_processor(Box::new(LiveVarAnalysisProcessor {}));
            pipeline.add_processor(Box::new(BorrowAnalysisProcessor {}));
            pipeline.add_processor(Box::new(PackrefAnalysisProcessor {}));
            Ok(Some(pipeline))
        }
        "eliminate_mut_refs" => {
            let mut pipeline = FunctionTargetPipeline::default();
            pipeline.add_processor(Box::new(EliminateImmRefsProcessor {}));
            pipeline.add_processor(Box::new(LiveVarAnalysisProcessor {}));
            pipeline.add_processor(Box::new(BorrowAnalysisProcessor {}));
            pipeline.add_processor(Box::new(WritebackAnalysisProcessor {}));
            pipeline.add_processor(Box::new(PackrefAnalysisProcessor {}));
            pipeline.add_processor(Box::new(EliminateMutRefsProcessor {}));
            Ok(Some(pipeline))
        }
        "lifetime" => {
            let mut pipeline = FunctionTargetPipeline::default();
            pipeline.add_processor(Box::new(LifetimeAnalysisProcessor {}));
            Ok(Some(pipeline))
        }
        "reaching_def" => {
            let mut pipeline = FunctionTargetPipeline::default();
            pipeline.add_processor(Box::new(ReachingDefProcessor()));
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
    let fs = AFS::new();
    let env: GlobalEnv = run_spec_lang_compiler(sources, vec![], Some("0x2345467"), &fs)?;
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
        text += &print_targets(&env, "initial translation from Move", &targets);

        // Run pipeline if any
        if let Some(pipeline) = pipeline_opt {
            pipeline.run(&env, &mut targets);
            text += &print_targets(&env, &format!("after pipeline `{}`", dir_name), &targets);
        }

        text
    };
    let baseline_path = path.with_extension("exp");
    verify_or_update_baseline(baseline_path.as_path(), &out)?;
    Ok(())
}

fn print_targets(env: &GlobalEnv, header: &str, targets: &FunctionTargetsHolder) -> String {
    let mut text = String::new();
    text.push_str(&format!("============ {} ================\n", header));
    for module_env in env.get_modules() {
        for func_env in module_env.get_functions() {
            let target = targets.get_target(&func_env);
            target.register_annotation_formatters_for_test();
            text += &format!("\n{}\n", target);
        }
    }
    text
}

datatest_stable::harness!(test_runner, "tests", r".*\.move");
