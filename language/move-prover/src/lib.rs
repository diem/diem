// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::{
    boogie_wrapper::BoogieWrapper,
    bytecode_translator::BoogieTranslator,
    cli::{Options, INLINE_PRELUDE},
    prelude_template_helpers::StratificationHelper,
};
use anyhow::anyhow;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream, WriteColor};
use docgen::docgen::Docgen;
use handlebars::Handlebars;
#[allow(unused_imports)]
use log::{debug, info, warn};
use move_lang::find_move_filenames;
use once_cell::sync::Lazy;
use regex::Regex;
use spec_lang::{code_writer::CodeWriter, emit, emitln, env::GlobalEnv, run_spec_lang_compiler};
use stackless_bytecode_generator::{
    borrow_analysis::BorrowAnalysisProcessor,
    eliminate_imm_refs::EliminateImmRefsProcessor,
    eliminate_mut_refs::EliminateMutRefsProcessor,
    function_target_pipeline::{FunctionTargetPipeline, FunctionTargetsHolder},
    livevar_analysis::LiveVarAnalysisProcessor,
    packref_analysis::PackrefAnalysisProcessor,
    writeback_analysis::WritebackAnalysisProcessor,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    time::Instant,
};

mod boogie_helpers;
mod boogie_wrapper;
mod bytecode_translator;
pub mod cli;
mod prelude_template_helpers;
mod spec_translator;

// =================================================================================================
// Entry Point

/// Content of the default prelude.
const DEFAULT_PRELUDE: &[u8] = include_bytes!("prelude.bpl");

pub fn run_move_prover<W: WriteColor>(
    error_writer: &mut W,
    options: Options,
) -> anyhow::Result<()> {
    let now = Instant::now();
    let sources = find_move_filenames(&options.move_sources)?;
    let deps = calculate_deps(&sources, &find_move_filenames(&options.move_deps)?)?;
    let address = Some(options.account_address.as_ref());
    debug!("parsing and checking sources");
    let mut env: GlobalEnv = run_spec_lang_compiler(sources, deps, address)?;
    if env.has_errors() {
        env.report_errors(error_writer);
        return Err(anyhow!("exiting with checking errors"));
    }
    if env.has_warnings() {
        env.report_warnings(error_writer);
    }

    // Until this point, prover and docgen have same code. Here we part ways.
    if options.run_docgen {
        return run_docgen(&env, &options, now);
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
    debug!("writing boogie to {}", &options.output_path);
    writer.process_result(|result| fs::write(&options.output_path, result))?;
    let translator_elapsed = now.elapsed();
    if !options.prover.generate_only {
        let boogie_file_id =
            writer.process_result(|result| env.add_source(&options.output_path, result, false));
        let boogie = BoogieWrapper {
            env: &env,
            targets: &targets,
            writer: &writer,
            options: &options,
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
    Ok(())
}

pub fn run_move_prover_errors_to_stderr(options: Options) -> anyhow::Result<()> {
    let mut error_writer = StandardStream::stderr(ColorChoice::Auto);
    run_move_prover(&mut error_writer, options)
}

fn run_docgen(env: &GlobalEnv, options: &Options, now: Instant) -> anyhow::Result<()> {
    let mut generator = Docgen::new(env, &options.docgen);
    let checking_elapsed = now.elapsed();
    info!("generating documentation");
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

/// Adds the prelude to the generated output.
fn add_prelude(options: &Options, writer: &CodeWriter) -> anyhow::Result<()> {
    emit!(writer, "\n// ** prelude from {}\n\n", &options.prelude_path);
    let content = if options.prelude_path == INLINE_PRELUDE {
        debug!("using inline prelude");
        String::from_utf8_lossy(DEFAULT_PRELUDE).to_string()
    } else {
        debug!("using prelude at {}", &options.prelude_path);
        fs::read_to_string(&options.prelude_path)?
    };
    let mut handlebars = Handlebars::new();
    handlebars.register_helper(
        "stratified",
        Box::new(StratificationHelper::new(
            options.backend.stratification_depth,
        )),
    );
    let expanded_content = handlebars.render_template(&content, &options)?;
    emitln!(writer, &expanded_content);
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
    res.add_processor(LiveVarAnalysisProcessor::new());
    res.add_processor(BorrowAnalysisProcessor::new());
    res.add_processor(WritebackAnalysisProcessor::new());
    res.add_processor(PackrefAnalysisProcessor::new());
    res.add_processor(EliminateMutRefsProcessor::new());

    res
}

/// Calculates transitive dependencies of the given move sources.
fn calculate_deps(sources: &[String], input_deps: &[String]) -> anyhow::Result<Vec<String>> {
    let file_map = calculate_file_map(input_deps)?;
    let mut deps = vec![];
    let mut visited = BTreeSet::new();
    for src in sources.iter() {
        calculate_deps_recursively(Path::new(src), &file_map, &mut visited, &mut deps)?;
    }
    Ok(deps)
}

/// Recursively calculate dependencies.
fn calculate_deps_recursively(
    path: &Path,
    file_map: &BTreeMap<String, PathBuf>,
    visited: &mut BTreeSet<String>,
    deps: &mut Vec<String>,
) -> anyhow::Result<()> {
    static REX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"(?m)0x[0-9abcdefABCDEF]+::\s*(\w+)").unwrap());
    if !visited.insert(path.to_string_lossy().to_string()) {
        return Ok(());
    }
    for dep in extract_matches(path, &*REX)? {
        if let Some(dep_path) = file_map.get(&dep) {
            let dep_str = dep_path.to_string_lossy().to_string();
            if !deps.contains(&dep_str) {
                deps.push(dep_str);
                calculate_deps_recursively(dep_path.as_path(), file_map, visited, deps)?;
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
