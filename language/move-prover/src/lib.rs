// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::cli::Options;
use abigen::Abigen;
use anyhow::anyhow;
use boogie_backend::{
    add_prelude, boogie_wrapper::BoogieWrapper, bytecode_translator::BoogieTranslator,
};
use bytecode::{
    data_invariant_instrumentation::DataInvariantInstrumentationProcessor,
    debug_instrumentation::DebugInstrumenter,
    function_target_pipeline::{FunctionTargetPipeline, FunctionTargetsHolder},
    global_invariant_instrumentation::GlobalInvariantInstrumentationProcessor,
    global_invariant_instrumentation_v2::GlobalInvariantInstrumentationProcessorV2,
    mono_analysis::MonoAnalysisProcessor,
    read_write_set_analysis::{self, ReadWriteSetProcessor},
    spec_instrumentation::SpecInstrumentationProcessor,
};
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream, WriteColor};
use docgen::Docgen;
use errmapgen::ErrmapGen;
#[allow(unused_imports)]
use log::{debug, info, warn};
use move_model::{code_writer::CodeWriter, model::GlobalEnv, run_model_builder};
use std::{fs, path::PathBuf, time::Instant};

pub mod cli;
mod pipelines;

// =================================================================================================
// Prover API

pub fn run_move_prover_errors_to_stderr(options: Options) -> anyhow::Result<()> {
    let mut error_writer = StandardStream::stderr(ColorChoice::Auto);
    run_move_prover(&mut error_writer, options)
}

pub fn run_move_prover<W: WriteColor>(
    error_writer: &mut W,
    options: Options,
) -> anyhow::Result<()> {
    let now = Instant::now();

    // Run the model builder.
    let env = run_model_builder(&options.move_sources, &options.move_deps)?;
    let build_duration = now.elapsed();
    check_errors(&env, error_writer, "exiting with model building errors")?;
    if options.prover.report_warnings && env.has_warnings() {
        env.report_warnings(error_writer);
    }

    // Add the prover options as an extension to the environment, so they can be accessed
    // from there.
    env.set_extension(options.prover.clone());

    // Until this point, prover and docgen have same code. Here we part ways.
    if options.run_docgen {
        return run_docgen(&env, &options, error_writer, now);
    }
    // Same for ABI generator.
    if options.run_abigen {
        return run_abigen(&env, &options, now);
    }
    // Same for the error map generator
    if options.run_errmapgen {
        return Ok(run_errmapgen(&env, &options, now));
    }
    // Same for read/write set analysis
    if options.run_read_write_set {
        return Ok(run_read_write_set(&env, &options, now));
    }

    // Check correct backend versions.
    options.backend.check_tool_versions()?;

    // Create and process bytecode
    let now = Instant::now();
    let targets = create_and_process_bytecode(&options, &env);
    let trafo_duration = now.elapsed();
    check_errors(
        &env,
        error_writer,
        "exiting with bytecode transformation errors",
    )?;

    // Generate boogie code
    let now = Instant::now();
    let code_writer = generate_boogie(&env, &options, &targets)?;
    let gen_duration = now.elapsed();
    check_errors(&env, error_writer, "exiting with boogie generation errors")?;

    // Verify boogie code.
    let now = Instant::now();
    verify_boogie(&env, &options, &targets, code_writer)?;
    let verify_duration = now.elapsed();

    // Report durations.
    info!(
        "{:.3}s build, {:.3}s trafo, {:.3}s gen, {:.3}s verify",
        build_duration.as_secs_f64(),
        trafo_duration.as_secs_f64(),
        gen_duration.as_secs_f64(),
        verify_duration.as_secs_f64()
    );
    check_errors(
        &env,
        error_writer,
        "exiting with boogie verification errors",
    )
}

pub fn check_errors<W: WriteColor>(
    env: &GlobalEnv,
    error_writer: &mut W,
    msg: &'static str,
) -> anyhow::Result<()> {
    if env.has_errors() {
        env.report_errors(error_writer);
        Err(anyhow!(msg))
    } else {
        Ok(())
    }
}

pub fn generate_boogie(
    env: &GlobalEnv,
    options: &Options,
    targets: &FunctionTargetsHolder,
) -> anyhow::Result<CodeWriter> {
    let writer = CodeWriter::new(env.internal_loc());
    if options.boogie_exp {
        // Use the experimental boogie backend.
        boogie_backend_exp::add_prelude(&options.backend, &writer)?;
        let mut translator = boogie_backend_exp::bytecode_translator::BoogieTranslator::new(
            &env,
            &options.backend,
            &targets,
            &writer,
        );
        translator.translate();
    } else {
        add_prelude(&options.backend, &writer)?;
        let mut translator = BoogieTranslator::new(&env, &options.backend, &targets, &writer);
        translator.translate();
    }
    Ok(writer)
}

pub fn verify_boogie(
    env: &GlobalEnv,
    options: &Options,
    targets: &FunctionTargetsHolder,
    writer: CodeWriter,
) -> anyhow::Result<()> {
    let output_existed = std::path::Path::new(&options.output_path).exists();
    debug!("writing boogie to `{}`", &options.output_path);
    writer.process_result(|result| fs::write(&options.output_path, result))?;
    if options.boogie_exp {
        let boogie = boogie_backend_exp::boogie_wrapper::BoogieWrapper {
            env: &env,
            targets: &targets,
            writer: &writer,
            options: &options.backend,
        };
        boogie.call_boogie_and_verify_output(options.backend.bench_repeat, &options.output_path)?;
    } else {
        let boogie = BoogieWrapper {
            env: &env,
            targets: &targets,
            writer: &writer,
            options: &options.backend,
        };
        boogie.call_boogie_and_verify_output(options.backend.bench_repeat, &options.output_path)?;
    }
    if !output_existed && !options.backend.keep_artifacts {
        std::fs::remove_file(&options.output_path).unwrap_or_default();
    }
    Ok(())
}

/// Create bytecode and process it.
pub fn create_and_process_bytecode(options: &Options, env: &GlobalEnv) -> FunctionTargetsHolder {
    let mut targets = FunctionTargetsHolder::default();

    // Add function targets for all functions in the environment.
    for module_env in env.get_modules() {
        if options.prover.dump_bytecode {
            let output_file = options
                .move_sources
                .get(0)
                .cloned()
                .unwrap_or_else(|| "bytecode".to_string())
                .replace(".move", ".mv.disas");
            fs::write(&output_file, &module_env.disassemble())
                .expect("dumping disassembled module");
        }
        for func_env in module_env.get_functions() {
            targets.add_target(&func_env)
        }
    }

    // Create processing pipeline and run it.
    let pipeline = create_bytecode_processing_pipeline(options);
    if options.prover.dump_bytecode {
        let dump_file = options
            .move_sources
            .get(0)
            .cloned()
            .unwrap_or_else(|| "bytecode".to_string())
            .replace(".move", "");
        pipeline.run_with_dump(env, &mut targets, &dump_file, options.prover.dump_cfg)
    } else {
        pipeline.run(env, &mut targets);
    }
    targets
}

/// Function to create the transformation pipeline.
fn create_bytecode_processing_pipeline(options: &Options) -> FunctionTargetPipeline {
    let mut res = FunctionTargetPipeline::default();
    // Add processors in order they are executed.

    res.add_processor(DebugInstrumenter::new());
    pipelines::pipelines(options)
        .into_iter()
        .for_each(|processor| res.add_processor(processor));
    res.add_processor(SpecInstrumentationProcessor::new());
    res.add_processor(DataInvariantInstrumentationProcessor::new());
    if options.inv_v2 {
        // *** convert to v2 version ***
        res.add_processor(GlobalInvariantInstrumentationProcessorV2::new());
    } else {
        res.add_processor(GlobalInvariantInstrumentationProcessor::new());
    }
    res.add_processor(MonoAnalysisProcessor::new());
    res
}

// Tools using the Move prover top-level driver
// ============================================

// TODO: make those tools independent. Need to first address the todo to
// move the model builder into the move-model crate.

fn run_docgen<W: WriteColor>(
    env: &GlobalEnv,
    options: &Options,
    error_writer: &mut W,
    now: Instant,
) -> anyhow::Result<()> {
    let generator = Docgen::new(env, &options.docgen);
    let checking_elapsed = now.elapsed();
    info!("generating documentation");
    for (file, content) in generator.gen() {
        let path = PathBuf::from(&file);
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(path.as_path(), content)?;
    }
    let generating_elapsed = now.elapsed();
    info!(
        "{:.3}s checking, {:.3}s generating",
        checking_elapsed.as_secs_f64(),
        (generating_elapsed - checking_elapsed).as_secs_f64()
    );
    if env.has_errors() {
        env.report_errors(error_writer);
        Err(anyhow!("exiting with documentation generation errors"))
    } else {
        Ok(())
    }
}

fn run_abigen(env: &GlobalEnv, options: &Options, now: Instant) -> anyhow::Result<()> {
    let mut generator = Abigen::new(env, &options.abigen);
    let checking_elapsed = now.elapsed();
    info!("generating ABI files");
    generator.gen();
    for (file, content) in generator.into_result() {
        let path = PathBuf::from(&file);
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(path.as_path(), content)?;
    }
    let generating_elapsed = now.elapsed();
    info!(
        "{:.3}s checking, {:.3}s generating",
        checking_elapsed.as_secs_f64(),
        (generating_elapsed - checking_elapsed).as_secs_f64()
    );
    Ok(())
}

fn run_errmapgen(env: &GlobalEnv, options: &Options, now: Instant) {
    let mut generator = ErrmapGen::new(env, &options.errmapgen);
    let checking_elapsed = now.elapsed();
    info!("generating error map");
    generator.gen();
    generator.save_result();
    let generating_elapsed = now.elapsed();
    info!(
        "{:.3}s checking, {:.3}s generating",
        checking_elapsed.as_secs_f64(),
        (generating_elapsed - checking_elapsed).as_secs_f64()
    );
}

fn run_read_write_set(env: &GlobalEnv, options: &Options, now: Instant) {
    let mut targets = FunctionTargetsHolder::default();

    for module_env in env.get_modules() {
        for func_env in module_env.get_functions() {
            targets.add_target(&func_env)
        }
    }
    let mut pipeline = FunctionTargetPipeline::default();
    pipeline.add_processor(ReadWriteSetProcessor::new());

    let start = now.elapsed();
    info!("generating read/write set");
    pipeline.run(env, &mut targets);
    read_write_set_analysis::get_read_write_set(env, &targets);
    println!("generated for {:?}", options.move_sources);

    let end = now.elapsed();
    info!("{:.3}s analyzing", (end - start).as_secs_f64());
}
