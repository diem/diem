// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Wrapper around the boogie program. Allows to call boogie and analyze the output.

use crate::cli::{abort_on_error, abort_with_error};
use crate::code_writer::CodeWriter;
use crate::driver::PSEUDO_PRELUDE_MODULE;
use crate::env::{GlobalEnv, ModuleIndex};
use codespan::{ByteIndex, ByteSpan, CodeMap, ColumnIndex, FileName, LineIndex};
use codespan_reporting::termcolor::{ColorChoice, StandardStream};
use codespan_reporting::{Diagnostic, Label, Severity};
use ir_to_bytecode_syntax::ast::Loc;
use itertools::Itertools;
use log::{debug, error, info};
use regex::Regex;
use std::fs;
use std::process::Command;

/// Represents the boogie wrapper.
pub struct BoogieWrapper<'env> {
    pub env: &'env GlobalEnv,
    pub writer: &'env CodeWriter,
}

/// Output of a boogie run.
pub struct BoogieOutput {
    /// All errors which could be parsed from the output.
    pub errors: Vec<BoogieError>,

    /// Full output as a string.
    pub all_output: String,
}

/// Line/Column pair position reported by boogie.
pub type Position = (LineIndex, ColumnIndex);

/// Kind of boogie error.
#[derive(PartialEq)]
pub enum BoogieErrorKind {
    Compilation,
    Verification,
}

/// A boogie error.
pub struct BoogieError {
    pub kind: BoogieErrorKind,
    pub position: Position,
    pub message: String,
    pub execution_trace: Vec<(Position, String)>,
}

impl<'env> BoogieWrapper<'env> {
    /// Calls boogie on the given file. On success, returns a struct representing the analyzed
    /// output of boogie.
    pub fn call_boogie(&self, boogie_file: &str) -> Option<BoogieOutput> {
        let args = self.env.options.get_boogie_command(boogie_file);
        info!("calling boogie");
        debug!("command line: {}", args.iter().join(" "));
        let output = abort_on_error(
            Command::new(&args[0]).args(&args[1..]).output(),
            "error executing boogie",
        );
        if !output.status.success() {
            error!("boogie exited with status {:?}", output.status.code());
            None
        } else {
            let out = String::from_utf8_lossy(&output.stdout).to_string();
            let mut errors = self.extract_verification_errors(&out);
            errors.extend(self.extract_compilation_errors(&out));
            Some(BoogieOutput {
                errors,
                all_output: out,
            })
        }
    }

    /// Calls boogie and analyzes output. Errors which can be associated with source modules
    /// will be added to the environment. Any other errors will be reported directly. Returns true
    /// if verification succeeded.
    pub fn call_boogie_and_verify_output(&self, boogie_file: &str) -> bool {
        match self.call_boogie(boogie_file) {
            None => {
                abort_with_error("exiting") // we already reported boogie error
            }
            Some(BoogieOutput { errors, all_output }) => {
                let boogie_log_file = self.env.options.get_boogie_log_file(boogie_file);
                info!("writing boogie log to {}", boogie_log_file);
                abort_on_error(
                    fs::write(&boogie_log_file, &all_output),
                    "cannot write boogie log file",
                );

                let mut diag_on_target = vec![];
                for error in &errors {
                    self.add_error(error, &mut diag_on_target);
                }
                if !diag_on_target.is_empty() {
                    // Report diagnostic which could not be associated with a module directly
                    // on the boogie source.
                    self.writer.process_result(|boogie_code| {
                        let mut codemap = CodeMap::new();
                        codemap.add_filemap(
                            FileName::real(&self.env.options.output_path),
                            boogie_code.to_string(),
                        );
                        for diag in &diag_on_target {
                            let writer = StandardStream::stderr(ColorChoice::Auto);
                            codespan_reporting::emit(writer, &codemap, diag)
                                .expect("emitting diagnostic failed")
                        }
                    });
                }
                errors.is_empty()
            }
        }
    }

    /// Helper to add a boogie error as a codespan Diagnostic.
    fn add_error(&self, error: &BoogieError, diags_on_target: &mut Vec<Diagnostic>) {
        let (index, location) = self.get_locations(error.position);
        let on_source = error.kind == BoogieErrorKind::Verification
            && location.is_some()
            && location.unwrap().0 != PSEUDO_PRELUDE_MODULE;
        let label = Label::new_primary(if on_source {
            // Location is on original source.
            location.unwrap().1
        } else {
            // Location is on generated boogie.
            ByteSpan::new(index, index)
        });
        let mut add_diag = |diag| {
            if on_source {
                self.env.get_module(location.unwrap().0).add_diag(diag);
            } else {
                diags_on_target.push(diag);
            }
        };
        let severity = if on_source {
            Severity::Error
        } else {
            Severity::Bug
        };
        add_diag(Diagnostic::new(severity, &error.message).with_label(label));
        if error.kind == BoogieErrorKind::Verification && !error.execution_trace.is_empty() {
            let mut trace = error
                .execution_trace
                .iter()
                .filter_map(|(pos, _)| {
                    let (_, trace_location) = self.get_locations(*pos);
                    if self.env.options.minimize_execution_trace
                        && (trace_location.is_none()
                            || trace_location.unwrap().0 == PSEUDO_PRELUDE_MODULE)
                    {
                        // trace entry has no source location or is from prelude, skip
                        return None;
                    }
                    let (file, act_pos) = if on_source
                        && trace_location.is_some()
                        && trace_location.unwrap().0 != PSEUDO_PRELUDE_MODULE
                    {
                        // Calculate line/column source position from (ModuleIndex, Loc).
                        if let Some((module_idx, loc)) = trace_location {
                            self.env.get_module(module_idx).get_position(loc)
                        } else {
                            unreachable!()
                        }
                    } else {
                        // Use boogie target position
                        (self.env.options.output_path.clone(), *pos)
                    };
                    Some(format!(
                        "    at {}:{}:{}",
                        file,
                        (act_pos.0).0 + 1,
                        (act_pos.1).0 + 1,
                    ))
                })
                .collect_vec();
            if self.env.options.minimize_execution_trace {
                // Removes consecutive entries which point to the same line.
                trace.dedup();
            }
            add_diag(Diagnostic::new(
                Severity::Help,
                &format!("Execution trace for previous error:\n{}", trace.join("\n")),
            ))
        }
    }

    /// Gets the target byte index and source location (if available) from a target line/column
    /// position.
    fn get_locations(&self, pos: Position) -> (ByteIndex, Option<(ModuleIndex, Loc)>) {
        let index = self.writer.get_output_byte_index(pos.0, pos.1).unwrap();
        (index, self.writer.get_source_location(index))
    }

    /// Extracts verification errors from Boogie output.
    ///
    /// This parses the output for errors of the form:
    ///
    /// ```ignore
    /// (0,0): Error BP5003: A postcondition might not hold on this return path.
    /// output.bpl(2964,1): Related location: This is the postcondition that might not hold.
    /// Execution trace:
    ///    output.bpl(3068,5): anon0
    ///    output.bpl(2960,23): inline$LibraAccount_pay_from_sender_with_metadata$0$Entry
    ///    output.bpl(2989,5): inline$LibraAccount_pay_from_sender_with_metadata$0$anon0
    ///    ...
    /// ```
    fn extract_verification_errors(&self, out: &str) -> Vec<BoogieError> {
        let verification_diag_start = Regex::new(r"(?m)^.*Error BP\d+:(?P<msg>.*)$").unwrap();
        let verification_diag_cond =
            Regex::new(r"(?m)^.+\((?P<line>\d+),(?P<col>\d+)\): Related.*$").unwrap();
        let verification_diag_trace = Regex::new(r"(?m)^Execution trace:$").unwrap();
        let verification_diag_trace_entry =
            Regex::new(r"(?m)^    .*\((?P<line>\d+),(?P<col>\d+)\): (?P<msg>.*)$").unwrap();
        let mut errors = vec![];
        let mut at: usize = 0;
        while let Some(cap) = verification_diag_start.captures(&out[at..]) {
            let msg = cap.name("msg").unwrap().as_str();
            at += cap.get(0).unwrap().end();
            if let Some(cap) = verification_diag_cond.captures(&out[at..]) {
                let line = cap.name("line").unwrap().as_str();
                let col = cap.name("col").unwrap().as_str();
                let mut trace = vec![];
                at += cap.get(0).unwrap().end();
                if let Some(m) = verification_diag_trace.find(&out[at..]) {
                    at += m.end();
                    while let Some(cap) = verification_diag_trace_entry.captures(&out[at..]) {
                        let line = cap.name("line").unwrap().as_str();
                        let col = cap.name("col").unwrap().as_str();
                        let msg = cap.name("msg").unwrap().as_str();
                        trace.push((make_position(line, col), msg.to_string()));
                        at += cap.get(0).unwrap().end();
                        if !out[at..].starts_with("\n  ") && !out[at..].starts_with("\r\n  ") {
                            // Don't read further if this line does not start with an indent,
                            // as all trace entries do. Otherwise we would match the trace entry
                            // for the next error.
                            break;
                        }
                    }
                }
                errors.push(BoogieError {
                    kind: BoogieErrorKind::Verification,
                    position: make_position(line, col),
                    message: msg.to_string(),
                    execution_trace: trace,
                });
            }
        }
        errors
    }

    /// Extracts compilation errors. This captures any kind of errors different than the
    /// verification errors (as seen so far).
    fn extract_compilation_errors(&self, out: &str) -> Vec<BoogieError> {
        let diag_re =
            Regex::new(r"(?m)^.*\((?P<line>\d+),(?P<col>\d+)\).*(Error:|error:).*$").unwrap();
        diag_re
            .captures_iter(&out)
            .map(|cap| {
                let line = cap.name("line").unwrap().as_str();
                let col = cap.name("col").unwrap().as_str();
                let msg = cap.get(0).unwrap().as_str();
                BoogieError {
                    kind: BoogieErrorKind::Compilation,
                    position: make_position(line, col),
                    message: msg.to_string(),
                    execution_trace: vec![],
                }
            })
            .collect_vec()
    }
}

/// Creates a position (line/column pair) from strings which are known to consist only of digits.
fn make_position(line_str: &str, col_str: &str) -> Position {
    // This will crash on overflow.
    let mut line = line_str.parse::<u32>().unwrap();
    let mut col = col_str.parse::<u32>().unwrap();
    if line > 0 {
        line -= 1;
    }
    if col > 0 {
        col -= 1;
    }
    (LineIndex(line), ColumnIndex(col))
}
