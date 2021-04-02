// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::cli::Options;
use abigen::Abigen;
use anyhow::anyhow;
use boogie_backend::{boogie_wrapper::BoogieWrapper, bytecode_translator::BoogieTranslator};
use bytecode::{
    data_invariant_instrumentation::DataInvariantInstrumentationProcessor,
    debug_instrumentation::DebugInstrumenter,
    function_target_pipeline::{FunctionTargetPipeline, FunctionTargetsHolder},
    global_invariant_instrumentation::GlobalInvariantInstrumentationProcessor,
    global_invariant_instrumentation_v2::GlobalInvariantInstrumentationProcessorV2,
    read_write_set_analysis::{self, ReadWriteSetProcessor},
    spec_instrumentation::SpecInstrumentationProcessor,
};
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream, WriteColor};
use docgen::Docgen;
use errmapgen::ErrmapGen;
use itertools::Itertools;
#[allow(unused_imports)]
use log::{debug, info, warn};
use move_lang::find_move_filenames;
use move_model::{
    code_writer::CodeWriter, model::GlobalEnv, run_model_builder, run_spec_instrumenter,
};
use once_cell::sync::Lazy;
use regex::Regex;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    time::Instant,
};

pub mod cli;
mod pipelines;

// =================================================================================================
// Entry Point

pub fn run_move_prover<W: WriteColor>(
    error_writer: &mut W,
    options: Options,
) -> anyhow::Result<()> {
    let now = Instant::now();
    let target_sources = find_move_filenames(&options.move_sources, true)?;
    let all_sources = collect_all_sources(
        &target_sources,
        &find_move_filenames(&options.move_deps, true)?,
        options.inv_v2,
    )?;
    let other_sources = remove_sources(&target_sources, all_sources);
    if options.run_spec_instrument {
        let env = run_spec_instrumenter(
            target_sources,
            other_sources,
            &options.output_path,
            options.prover.dump_bytecode,
        )?;
        if env.has_errors() {
            env.report_errors(error_writer);
            return Err(anyhow!("exiting with instrumentation errors"));
        }
        if options.prover.report_warnings && env.has_warnings() {
            env.report_warnings(error_writer);
        }
        return Ok(());
    }
    debug!("parsing and checking sources");
    let mut env: GlobalEnv = run_model_builder(target_sources, other_sources)?;
    if env.has_errors() {
        env.report_errors(error_writer);
        return Err(anyhow!("exiting with checking errors"));
    }
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

    let targets = create_and_process_bytecode(&options, &env);
    if env.has_errors() {
        env.report_errors(error_writer);
        return Err(anyhow!("exiting with transformation errors"));
    }

    if env.has_errors() {
        env.report_errors(error_writer);
        return Err(anyhow!("exiting with modifies checking errors"));
    }
    // Analyze and find out the set of modules/functions to be translated and/or verified.
    if env.has_errors() {
        env.report_errors(error_writer);
        return Err(anyhow!("exiting with analysis errors"));
    }
    let writer = CodeWriter::new(env.internal_loc());
    boogie_backend::add_prelude(&options.backend, &writer)?;
    let mut translator = BoogieTranslator::new(&env, &options.backend, &targets, &writer);
    translator.translate();
    if env.has_errors() {
        env.report_errors(error_writer);
        return Err(anyhow!("exiting with boogie generation errors"));
    }
    let output_existed = std::path::Path::new(&options.output_path).exists();
    debug!("writing boogie to `{}`", &options.output_path);
    writer.process_result(|result| fs::write(&options.output_path, result))?;
    let translator_elapsed = now.elapsed();
    if !options.prover.generate_only {
        let boogie_file_id =
            writer.process_result(|result| env.add_source(&options.output_path, result, false));
        let boogie = BoogieWrapper {
            env: &env,
            targets: &targets,
            writer: &writer,
            options: &options.backend,
            boogie_file_id,
        };
        boogie.call_boogie_and_verify_output(options.backend.bench_repeat, &options.output_path)?;
        let boogie_elapsed = now.elapsed();
        if options.backend.bench_repeat <= 1 {
            info!(
                "{:.3}s translator, {:.3}s solver",
                translator_elapsed.as_secs_f64(),
                (boogie_elapsed - translator_elapsed).as_secs_f64()
            );
        } else {
            info!(
                "{:.3}s translator, {:.3}s solver (average over {} solver runs)",
                translator_elapsed.as_secs_f64(),
                (boogie_elapsed - translator_elapsed).as_secs_f64()
                    / (options.backend.bench_repeat as f64),
                options.backend.bench_repeat
            );
        }
        if env.has_errors() {
            env.report_errors(error_writer);
            return Err(anyhow!("exiting with boogie verification errors"));
        }
    }
    if !output_existed && !options.backend.keep_artifacts {
        std::fs::remove_file(&options.output_path).unwrap_or_default();
    }

    Ok(())
}

pub fn run_move_prover_errors_to_stderr(options: Options) -> anyhow::Result<()> {
    let mut error_writer = StandardStream::stderr(ColorChoice::Auto);
    run_move_prover(&mut error_writer, options)
}

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
    pipeline.run(env, &mut targets, None, /* dump_cfg */ false);
    read_write_set_analysis::get_read_write_set(env, &targets);
    println!("generated for {:?}", options.move_sources);

    let end = now.elapsed();
    info!("{:.3}s analyzing", (end - start).as_secs_f64());
}

/// Create bytecode and process it.
fn create_and_process_bytecode(options: &Options, env: &GlobalEnv) -> FunctionTargetsHolder {
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
    let dump_file = if options.prover.dump_bytecode {
        Some(
            options
                .move_sources
                .get(0)
                .cloned()
                .unwrap_or_else(|| "bytecode".to_string())
                .replace(".move", ""),
        )
    } else {
        None
    };
    pipeline.run(env, &mut targets, dump_file, options.prover.dump_cfg);

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
    res
}

/// Remove the target Move files from the list of files.
fn remove_sources(sources: &[String], all_files: Vec<String>) -> Vec<String> {
    let canonical_sources = sources
        .iter()
        .map(|s| canonicalize(s))
        .collect::<BTreeSet<_>>();
    all_files
        .into_iter()
        .filter(|d| !canonical_sources.contains(&canonicalize(d)))
        .collect_vec()
}

/// Collect all the relevant Move sources among sources represented by `input deps`
/// parameter. The resulting vector of sources includes target sources, dependencies
/// of target sources, (recursive)friends of targets and dependencies, and
/// dependencies of friends.
fn collect_all_sources(
    target_sources: &[String],
    input_deps: &[String],
    use_inv_v2: bool,
) -> anyhow::Result<Vec<String>> {
    let mut all_sources = target_sources.to_vec();
    static DEP_REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"(?m)use\s*0x[0-9abcdefABCDEF]+::\s*(\w+)").unwrap());
    static NEW_FRIEND_REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"(?m)friend\s*0x[0-9abcdefABCDEF]+::\s*(\w+)").unwrap());
    static FRIEND_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"(?m)pragma\s*friend\s*=\s*0x[0-9abcdefABCDEF]+::\s*(\w+)").unwrap()
    });

    let target_deps = calculate_deps(&all_sources, input_deps, &DEP_REGEX)?;
    all_sources.extend(target_deps);

    let friend_sources = calculate_deps(
        &all_sources,
        input_deps,
        if use_inv_v2 {
            &NEW_FRIEND_REGEX
        } else {
            &FRIEND_REGEX
        },
    )?;

    all_sources.extend(friend_sources);

    let friend_deps = calculate_deps(&all_sources, input_deps, &DEP_REGEX)?;
    all_sources.extend(friend_deps);

    Ok(all_sources)
}

/// Calculates transitive dependencies of the given Move sources. This function
/// is also used to calculate transitive friends depending on the regex provided
/// for extracting matches.
fn calculate_deps(
    sources: &[String],
    input_deps: &[String],
    regex: &Regex,
) -> anyhow::Result<Vec<String>> {
    let file_map = calculate_file_map(input_deps)?;
    let mut deps = vec![];
    let mut visited = BTreeSet::new();
    for src in sources.iter() {
        calculate_deps_recursively(Path::new(src), &file_map, &mut visited, &mut deps, regex)?;
    }
    // Remove input sources from deps. They can end here because our dep analysis is an
    // over-approximation and for example cannot distinguish between references inside
    // and outside comments.
    let mut deps = remove_sources(sources, deps);
    // Sort deps by simple file name. Sorting is important because different orders
    // caused by platform dependent ways how `calculate_deps_recursively` may return values, can
    // cause different behavior of the SMT solver (butterfly effect). By using the simple file
    // name we abstract from places where the sources live in the file system. Since Move has
    // no namespaces and file names can be expected to be unique matching module/script names,
    // this should work in most cases.
    deps.sort_by(|a, b| {
        let fa = PathBuf::from(a);
        let fb = PathBuf::from(b);
        Ord::cmp(fa.file_name().unwrap(), fb.file_name().unwrap())
    });
    Ok(deps)
}

fn canonicalize(s: &str) -> String {
    match fs::canonicalize(s) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(_) => s.to_string(),
    }
}

/// Recursively calculate dependencies.
fn calculate_deps_recursively(
    path: &Path,
    file_map: &BTreeMap<String, PathBuf>,
    visited: &mut BTreeSet<String>,
    deps: &mut Vec<String>,
    regex: &Regex,
) -> anyhow::Result<()> {
    if !visited.insert(path.to_string_lossy().to_string()) {
        return Ok(());
    }
    debug!("including `{}`", path.display());
    for dep in extract_matches(path, regex)? {
        if let Some(dep_path) = file_map.get(&dep) {
            let dep_str = dep_path.to_string_lossy().to_string();
            if !deps.contains(&dep_str) {
                deps.push(dep_str);
                calculate_deps_recursively(dep_path.as_path(), file_map, visited, deps, regex)?;
            }
        }
    }
    Ok(())
}

/// Calculate a map of module names to files which define those modules.
fn calculate_file_map(deps: &[String]) -> anyhow::Result<BTreeMap<String, PathBuf>> {
    static REX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?m)module\s+(\w+)\s*\{").unwrap());
    let mut module_to_file = BTreeMap::new();
    for dep in deps {
        let dep_path = PathBuf::from(dep);
        for module in extract_matches(dep_path.as_path(), &*REX)? {
            module_to_file.insert(module, dep_path.clone());
        }
    }
    Ok(module_to_file)
}

/// Extracts matches out of some text file. `rex` must be a regular expression with one anonymous
/// group.
fn extract_matches(path: &Path, rex: &Regex) -> anyhow::Result<Vec<String>> {
    let mut content = String::new();
    let mut file = File::open(path)?;
    file.read_to_string(&mut content)?;
    let mut at = 0;
    let mut res = vec![];
    while let Some(cap) = rex.captures(&content[at..]) {
        res.push(cap.get(1).unwrap().as_str().to_string());
        at += cap.get(0).unwrap().end();
    }
    Ok(res)
}
