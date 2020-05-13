// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::{
    boogie_wrapper::BoogieWrapper,
    bytecode_translator::BoogieTranslator,
    cli::{Options, INLINE_PRELUDE},
    code_writer::CodeWriter,
    prelude_template_helpers::StratificationHelper,
};
use anyhow::anyhow;
use codespan_reporting::term::termcolor::WriteColor;
use handlebars::Handlebars;
#[allow(unused_imports)]
use log::{debug, info, warn};
use regex::Regex;
use spec_lang::{env::GlobalEnv, run_spec_lang_compiler};
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

#[macro_use]
mod code_writer;

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
    let sources = options.move_sources.clone();
    let mut deps = options.move_deps.clone();
    if !options.search_path.is_empty() {
        calculate_deps(&sources, &options.search_path, &mut deps)?;
    }
    let address = Some(options.account_address.as_ref());
    debug!("parsing and checking sources");
    let mut env: GlobalEnv = run_spec_lang_compiler(sources, deps, address)?;
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
    debug!("writing boogie to {}", &options.output_path);
    writer.process_result(|result| fs::write(&options.output_path, result))?;
    let translator_elapsed = now.elapsed();
    if !options.generate_only {
        let boogie_file_id =
            writer.process_result(|result| env.add_source(&options.output_path, result, false));
        let boogie = BoogieWrapper {
            env: &env,
            targets: &targets,
            writer: &writer,
            options: &options,
            boogie_file_id,
        };
        boogie.call_boogie_and_verify_output(options.bench_repeat, &options.output_path)?;
        let boogie_elapsed = now.elapsed();
        if options.bench_repeat <= 1 {
            info!(
                "{:.3}s translator, {:.3}s solver",
                translator_elapsed.as_secs_f64(),
                (boogie_elapsed - translator_elapsed).as_secs_f64()
            );
        } else {
            info!(
                "{:.3}s translator, {:.3}s solver (average over {} solver runs)",
                translator_elapsed.as_secs_f64(),
                (boogie_elapsed - translator_elapsed).as_secs_f64() / (options.bench_repeat as f64),
                options.bench_repeat
            );
        }

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
            options.template_context.stratification_depth,
        )),
    );
    let expanded_content = handlebars.render_template(&content, &options.template_context)?;
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
fn calculate_deps(
    sources: &[String],
    search_path: &[String],
    deps: &mut Vec<String>,
) -> anyhow::Result<()> {
    let file_map = calculate_file_map(search_path)?;
    let mut visited = BTreeSet::new();
    // Iterate both over sources and what is initial in deps, as provided via the --dep flag.
    for src in sources.iter().chain(deps.clone().iter()) {
        let path = Path::new(src);
        calculate_deps_recursively(path, &file_map, &mut visited, deps)?;
    }
    Ok(())
}

/// Recursively calculate dependencies.
fn calculate_deps_recursively(
    path: &Path,
    file_map: &BTreeMap<String, PathBuf>,
    visited: &mut BTreeSet<String>,
    deps: &mut Vec<String>,
) -> anyhow::Result<()> {
    if !visited.insert(path.to_string_lossy().to_string()) {
        return Ok(());
    }
    for dep in extract_matches(path, r"0x[0-9,a,b,c,d,e,f,A,B,C,D,E,F]+::\s*(\w+)")? {
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
fn calculate_file_map(search_path: &[String]) -> anyhow::Result<BTreeMap<String, PathBuf>> {
    let mut module_to_file = BTreeMap::new();
    for dir_str in search_path {
        let dir = Path::new(dir_str);
        for entry in dir.read_dir()? {
            let cand = entry?.path();
            if !cand.to_string_lossy().ends_with(".move") {
                continue;
            }
            for module in extract_matches(cand.as_path(), r"module\s+(\w+)\s*\{")? {
                module_to_file.insert(module, cand.clone());
            }
        }
    }
    Ok(module_to_file)
}

/// Extracts matches out of some text file. `pat` must be a regular expression with one anonymous
/// group. The list of the content of this group is returned. Use as in in
/// `extract_matches(file, "use 0x0::([a-zA-Z_]+)")`
pub fn extract_matches(path: &Path, pat: &str) -> anyhow::Result<Vec<String>> {
    let rex = Regex::new(&format!("(?m){}", pat))?;
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
