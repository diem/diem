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
    function_target_pipeline::{FunctionTargetPipeline, FunctionTargetsHolder},
    pipeline_factory,
    read_write_set_analysis::{self, ReadWriteSetProcessor},
};
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream, WriteColor};
use docgen::Docgen;
use errmapgen::ErrmapGen;
#[allow(unused_imports)]
use log::{debug, info, warn};
use move_model::{
    code_writer::CodeWriter, model::GlobalEnv, parse_addresses_from_options,
    run_model_builder_with_options,
};
use std::{
    fs,
    path::{Path, PathBuf},
    time::Instant,
};

pub mod cli;

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
    let env = run_model_builder_with_options(
        &options.move_sources,
        &options.move_deps,
        options.model_builder.clone(),
        parse_addresses_from_options(options.move_named_address_values.clone())?,
    )?;
    run_move_prover_with_model(&env, error_writer, options, Some(now))
}

pub fn run_move_prover_with_model<W: WriteColor>(
    env: &GlobalEnv,
    error_writer: &mut W,
    options: Options,
    timer: Option<Instant>,
) -> anyhow::Result<()> {
    let now = timer.unwrap_or_else(Instant::now);

    let build_duration = now.elapsed();
    check_errors(
        env,
        &options,
        error_writer,
        "exiting with model building errors",
    )?;
    env.report_diag(error_writer, options.prover.report_severity);

    // Add the prover options as an extension to the environment, so they can be accessed
    // from there.
    env.set_extension(options.prover.clone());

    // Until this point, prover and docgen have same code. Here we part ways.
    if options.run_docgen {
        return run_docgen(env, &options, error_writer, now);
    }
    // Same for ABI generator.
    if options.run_abigen {
        return run_abigen(env, &options, now);
    }
    // Same for the error map generator
    if options.run_errmapgen {
        return {
            run_errmapgen(env, &options, now);
            Ok(())
        };
    }
    // Same for read/write set analysis
    if options.run_read_write_set {
        return {
            run_read_write_set(env, &options, now);
            Ok(())
        };
    }

    // Check correct backend versions.
    options.backend.check_tool_versions()?;

    // Create and process bytecode
    let now = Instant::now();
    let targets = create_and_process_bytecode(&options, env);
    let trafo_duration = now.elapsed();
    check_errors(
        env,
        &options,
        error_writer,
        "exiting with bytecode transformation errors",
    )?;

    // Generate boogie code
    let now = Instant::now();
    let code_writer = generate_boogie(env, &options, &targets)?;
    let gen_duration = now.elapsed();
    check_errors(
        env,
        &options,
        error_writer,
        "exiting with boogie generation errors",
    )?;

    // Verify boogie code.
    let now = Instant::now();
    verify_boogie(env, &options, &targets, code_writer)?;
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
        env,
        &options,
        error_writer,
        "exiting with boogie verification errors",
    )
}

pub fn check_errors<W: WriteColor>(
    env: &GlobalEnv,
    options: &Options,
    error_writer: &mut W,
    msg: &'static str,
) -> anyhow::Result<()> {
    env.report_diag(error_writer, options.prover.report_severity);
    if env.has_errors() {
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
    add_prelude(env, &options.backend, &writer)?;
    let mut translator = BoogieTranslator::new(env, &options.backend, targets, &writer);
    translator.translate();
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
    let boogie = BoogieWrapper {
        env,
        targets,
        writer: &writer,
        options: &options.backend,
    };
    boogie.call_boogie_and_verify_output(&options.output_path)?;
    if !output_existed && !options.backend.keep_artifacts {
        std::fs::remove_file(&options.output_path).unwrap_or_default();
    }
    Ok(())
}

/// Create bytecode and process it.
pub fn create_and_process_bytecode(options: &Options, env: &GlobalEnv) -> FunctionTargetsHolder {
    let mut targets = FunctionTargetsHolder::default();
    let output_dir = Path::new(&options.output_path)
        .parent()
        .expect("expect the parent directory of the output path to exist");
    let output_prefix = options.move_sources.get(0).map_or("bytecode", |s| {
        Path::new(s).file_name().unwrap().to_str().unwrap()
    });

    // Add function targets for all functions in the environment.
    for module_env in env.get_modules() {
        if options.prover.dump_bytecode {
            let dump_file = output_dir.join(format!("{}.mv.disas", output_prefix));
            fs::write(&dump_file, &module_env.disassemble()).expect("dumping disassembled module");
        }
        for func_env in module_env.get_functions() {
            targets.add_target(&func_env)
        }
    }

    // Create processing pipeline and run it.
    let pipeline = if options.experimental_pipeline {
        pipeline_factory::experimental_pipeline()
    } else {
        pipeline_factory::default_pipeline_with_options(&options.prover)
    };

    if options.prover.dump_bytecode {
        let dump_file_base = output_dir
            .join(output_prefix)
            .into_os_string()
            .into_string()
            .unwrap();
        pipeline.run_with_dump(env, &mut targets, &dump_file_base, options.prover.dump_cfg)
    } else {
        pipeline.run(env, &mut targets);
    }

    targets
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
        env.report_diag(error_writer, options.prover.report_severity);
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
