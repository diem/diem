// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::boogie_wrapper::BoogieWrapper;
use crate::bytecode_translator::BoogieTranslator;
use crate::cli::{Options, INLINE_PRELUDE};
use crate::code_writer::CodeWriter;
use anyhow::anyhow;
use codespan_reporting::term::termcolor::WriteColor;
use log::info;
use spec_lang::env::GlobalEnv;
use spec_lang::run_spec_lang_compiler;
use std::fs;

// TODO: remove pub below. Having it for now to supress warnings.
#[macro_use]
pub mod code_writer;

pub mod boogie_helpers;
pub mod boogie_wrapper;
pub mod bytecode_translator;
pub mod cli;
pub mod spec_translator;

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
    let writer = CodeWriter::new(env.internal_loc());
    add_prelude(&options, &writer)?;
    let mut translator = BoogieTranslator::new(&env, &options, &writer);
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
