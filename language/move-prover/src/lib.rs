// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::{
    boogie_wrapper::BoogieWrapper,
    bytecode_translator::BoogieTranslator,
    cli::{Options, INLINE_PRELUDE},
    prelude_template_helpers::StratificationHelper,
};
use abigen::Abigen;
use anyhow::anyhow;
use bytecode::{
    borrow_analysis::BorrowAnalysisProcessor,
    clean_and_optimize::CleanAndOptimizeProcessor,
    eliminate_imm_refs::EliminateImmRefsProcessor,
    eliminate_mut_refs::EliminateMutRefsProcessor,
    function_target_pipeline::{FunctionTargetPipeline, FunctionTargetsHolder},
    livevar_analysis::LiveVarAnalysisProcessor,
    memory_instrumentation::MemoryInstrumentationProcessor,
    reaching_def_analysis::ReachingDefProcessor,
    stackless_bytecode::{Bytecode, Operation},
    test_instrumenter::TestInstrumenter,
    usage_analysis::{self, UsageProcessor},
};
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream, WriteColor};
use docgen::Docgen;
use errmapgen::ErrmapGen;
use handlebars::Handlebars;
use itertools::Itertools;
#[allow(unused_imports)]
use log::{debug, info, warn};
use move_lang::find_move_filenames;
use once_cell::sync::Lazy;
use regex::Regex;
use spec_lang::{code_writer::CodeWriter, emit, emitln, env::GlobalEnv, run_spec_lang_compiler};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
    time::Instant,
};

mod boogie_helpers;
mod boogie_wrapper;
mod bytecode_translator;
pub mod cli;
mod prelude_template_helpers;
mod prover_task_runner;
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
    let sources = find_move_filenames(&options.move_sources, true)?;
    let deps = calculate_deps(&sources, &find_move_filenames(&options.move_deps, true)?)?;
    let address = Some(options.account_address.as_ref());
    debug!("parsing and checking sources");
    let mut env: GlobalEnv = run_spec_lang_compiler(sources, deps, address)?;
    if env.has_errors() {
        env.report_errors(error_writer);
        return Err(anyhow!("exiting with checking errors"));
    }
    if options.prover.report_warnings && env.has_warnings() {
        env.report_warnings(error_writer);
    }

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
        return run_errmapgen(&env, &options, now);
    }

    let targets = create_and_process_bytecode(&options, &env);
    if env.has_errors() {
        env.report_errors(error_writer);
        return Err(anyhow!("exiting with transformation errors"));
    }

    if options.run_packed_types_gen {
        return run_packed_types_gen(&options, &env, &targets, now);
    }
    check_modifies(&env, &targets);
    if env.has_errors() {
        env.report_errors(error_writer);
        return Err(anyhow!("exiting with modifies checking errors"));
    }

    let writer = CodeWriter::new(env.internal_loc());
    add_prelude(&options, &writer)?;
    let mut translator = BoogieTranslator::new(&env, &options, &targets, &writer);
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

fn run_errmapgen(env: &GlobalEnv, options: &Options, now: Instant) -> anyhow::Result<()> {
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
    Ok(())
}

fn run_packed_types_gen(
    options: &Options,
    env: &GlobalEnv,
    targets: &FunctionTargetsHolder,
    now: Instant,
) -> anyhow::Result<()> {
    let checking_elapsed = now.elapsed();
    info!("generating packed_types");
    let packed_types = usage_analysis::get_packed_types(&env, &targets);
    info!("found {:?} packed types", packed_types.len());
    let mut access_path_type_map = BTreeMap::new();
    for ty in packed_types {
        access_path_type_map.insert(ty.access_vector(), ty);
    }
    let flattened_map = access_path_type_map
        .into_iter()
        .map(|(k, v)| (hex::encode(&k), v))
        .collect::<Vec<_>>();
    let types_json = serde_json::to_string_pretty(&flattened_map)?;
    let mut types_file = File::create(options.output_path.clone())?;
    types_file.write_all(&types_json.as_bytes())?;
    types_file.write_all(b"\n")?;

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

/// Check modifies annotations
fn check_modifies(env: &GlobalEnv, targets: &FunctionTargetsHolder) {
    use Bytecode::*;
    use Operation::*;

    for module_env in env.get_modules() {
        for func_env in module_env.get_functions() {
            let caller_func_target = targets.get_target(&func_env);
            for code in caller_func_target.get_bytecode() {
                if let Call(_, _, oper, _) = code {
                    if let Function(mid, fid, _) = oper {
                        let callee = mid.qualified(*fid);
                        let callee_func_env = env.get_function(callee);
                        let callee_func_target = targets.get_target(&callee_func_env);
                        let callee_modified_memory =
                            usage_analysis::get_modified_memory(&callee_func_target);
                        caller_func_target.get_modify_targets().keys().for_each(|target| {
                                if callee_modified_memory.contains(target) && callee_func_target.get_modify_targets_for_type(target).is_none() {
                                    let loc = caller_func_target.get_bytecode_loc(code.get_attr_id());
                                    env.error(&loc, &format!(
                                                        "caller `{}` specifies modify targets for `{}::{}` but callee does not",
                                                        env.symbol_pool().string(caller_func_target.get_name()),
                                                        env.get_module(target.module_id).get_name().display(env.symbol_pool()),
                                                        env.symbol_pool().string(target.id.symbol())));
                                }
                            });
                    }
                }
            }
        }
    }
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
    pipeline.run(env, &mut targets, dump_file);

    targets
}

/// Function to create the transformation pipeline.
fn create_bytecode_processing_pipeline(options: &Options) -> FunctionTargetPipeline {
    let mut res = FunctionTargetPipeline::default();

    // Add processors in order they are executed.
    res.add_processor(EliminateImmRefsProcessor::new());
    res.add_processor(EliminateMutRefsProcessor::new());
    res.add_processor(ReachingDefProcessor::new());
    res.add_processor(LiveVarAnalysisProcessor::new());
    res.add_processor(BorrowAnalysisProcessor::new());
    res.add_processor(MemoryInstrumentationProcessor::new());
    res.add_processor(CleanAndOptimizeProcessor::new());
    res.add_processor(TestInstrumenter::new(options.prover.verify_scope));
    res.add_processor(UsageProcessor::new());

    res
}

/// Calculates transitive dependencies of the given Move sources.
fn calculate_deps(sources: &[String], input_deps: &[String]) -> anyhow::Result<Vec<String>> {
    let file_map = calculate_file_map(input_deps)?;
    let mut deps = vec![];
    let mut visited = BTreeSet::new();
    for src in sources.iter() {
        calculate_deps_recursively(Path::new(src), &file_map, &mut visited, &mut deps)?;
    }
    // Remove input sources from deps. They can end here because our dep analysis is an
    // over-approximation and for example cannot distinguish between references inside
    // and outside comments.
    let canonical_sources = sources
        .iter()
        .map(|s| canonicalize(s))
        .collect::<BTreeSet<_>>();
    let mut deps = deps
        .into_iter()
        .filter(|d| !canonical_sources.contains(&canonicalize(d)))
        .collect_vec();
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
) -> anyhow::Result<()> {
    static REX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"(?m)use\s*0x[0-9abcdefABCDEF]+::\s*(\w+)").unwrap());
    if !visited.insert(path.to_string_lossy().to_string()) {
        return Ok(());
    }
    debug!("including `{}`", path.display());
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
