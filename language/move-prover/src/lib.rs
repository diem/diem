// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::{
    boogie_wrapper::BoogieWrapper,
    bytecode_translator::BoogieTranslator,
    cli::{Options, INLINE_PRELUDE},
    code_writer::CodeWriter,
};
use anyhow::anyhow;
use codespan_reporting::term::termcolor::WriteColor;
use log::info;
use spec_lang::{env::GlobalEnv, run_spec_lang_compiler};
use stackless_bytecode_generator::{
    eliminate_imm_refs::EliminateImmRefsProcessor,
    function_target_pipeline::{FunctionTargetPipeline, FunctionTargetsHolder},
    lifetime_analysis::LifetimeAnalysisProcessor,
};
use std::fs;

#[macro_use]
mod code_writer;

mod boogie_helpers;
mod boogie_wrapper;
mod bytecode_translator;
pub mod cli;
mod spec_translator;

// =================================================================================================
// Entry Point

/// Content of the default prelude.
const DEFAULT_PRELUDE: &[u8] = include_bytes!("prelude.bpl");

pub fn run_move_prover<W: WriteColor>(
    error_writer: &mut W,
    options: Options,
) -> anyhow::Result<()> {
    let sources = options.move_sources.clone();
    let address = Some(options.account_address.as_ref());
    info!("parsing and checking sources");
    let mut env: GlobalEnv = run_spec_lang_compiler(sources, vec![], address)?;
    if env.has_errors() {
        env.report_errors(error_writer);
        return Err(anyhow!("exiting with checking errors"));
    }
    let targets = create_and_process_bytecode(&options, &env);
    if env.has_errors() {
        env.report_errors(error_writer);
        return Err(anyhow!("exiting with transformation errors"));
    }
    let writer = CodeWriter::new(env.internal_loc());
    add_prelude(&options, &writer)?;
    let mut translator = BoogieTranslator::new(&env, &options, &targets, &writer);
    translator.translate();
    if env.has_errors() {
        env.report_errors(error_writer);
        return Err(anyhow!("exiting with boogie generation errors"));
    }
    info!("writing boogie to {}", &options.output_path);
    writer.process_result(|result| fs::write(&options.output_path, result))?;
    if !options.generate_only {
        let boogie_file_id =
            writer.process_result(|result| env.add_source(&options.output_path, result));
        let boogie = BoogieWrapper {
            env: &env,
            targets: &targets,
            writer: &writer,
            options: &options,
            boogie_file_id,
        };
        boogie.call_boogie_and_verify_output(&options.output_path)?;
        if env.has_errors() {
            env.report_errors(error_writer);
            return Err(anyhow!("exiting with boogie verification errors"));
        }
    }
    Ok(())
}

/// Adds the prelude to the generated output.
fn add_prelude(options: &Options, writer: &CodeWriter) -> anyhow::Result<()> {
    emit!(writer, "\n// ** prelude from {}\n\n", &options.prelude_path);
    if options.prelude_path == INLINE_PRELUDE {
        info!("using inline prelude");
        emitln!(writer, &String::from_utf8_lossy(DEFAULT_PRELUDE));
    } else {
        info!("using prelude at {}", &options.prelude_path);
        let content = fs::read_to_string(&options.prelude_path)?;
        emitln!(writer, &content);
    }
    Ok(())
}

/// Create bytecode and process it.
fn create_and_process_bytecode(options: &Options, env: &GlobalEnv) -> FunctionTargetsHolder {
    let mut targets = FunctionTargetsHolder::default();

    // Add function targets for all functions in the environment.
    for module_env in env.get_modules() {
        for func_env in module_env.get_functions() {
            targets.add_target(&func_env)
        }
    }

    // Create processing pipeline and run it.
    let pipeline = create_bytecode_processing_pipeline(options);
    pipeline.run(env, &mut targets);

    targets
}

/// Function to create the transformation pipeline.
fn create_bytecode_processing_pipeline(_options: &Options) -> FunctionTargetPipeline {
    let mut res = FunctionTargetPipeline::default();

    // Add processors in order they are executed.
    res.add_processor(EliminateImmRefsProcessor::new());

    // Must happen last as it is currently computing information based on raw code offsets.
    res.add_processor(LifetimeAnalysisProcessor::new());

    res
}
