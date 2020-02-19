// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Wrapper around the boogie program. Allows to call boogie and analyze the output.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::option::Option::None;
use std::process::Command;

use codespan::{
    ByteIndex, ByteOffset, ByteSpan, CodeMap, ColumnIndex, FileName, LineIndex, RawIndex,
};
use codespan_reporting::termcolor::{ColorChoice, StandardStream};
use codespan_reporting::{Diagnostic, Label, Severity};
use itertools::Itertools;
use log::warn;
use log::{debug, error, info};
use num::BigInt;
use pretty::RcDoc;
use regex::Regex;

use move_ir_types::ast::Loc;
use vm::file_format::{FunctionDefinitionIndex, StructDefinitionIndex};

use crate::cli::{abort_on_error, abort_with_error};
use crate::code_writer::CodeWriter;
use crate::driver::PSEUDO_PRELUDE_MODULE;
use crate::env::{FunctionEnv, GlobalEnv, GlobalType, ModuleEnv, ModuleIndex};

/// A type alias for the way how we use crate `pretty`'s document type. `pretty` is a
/// Wadler-style pretty printer. Our simple usage doesn't require any lifetime management.
type PrettyDoc = RcDoc<'static, ()>;

// -----------------------------------------------
// # Boogie Wrapper

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
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BoogieErrorKind {
    Compilation,
    Postcondition,
    Assertion,
}

impl BoogieErrorKind {
    fn is_from_verification(self) -> bool {
        self == BoogieErrorKind::Assertion || self == BoogieErrorKind::Postcondition
    }
}

/// A boogie error.
pub struct BoogieError {
    pub kind: BoogieErrorKind,
    pub position: Position,
    pub message: String,
    pub execution_trace: Vec<(Position, TraceKind, String)>,
    pub model: Option<Model>,
}

/// The type of a trace.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TraceKind {
    Regular,
    EnterFunction,
    ExitFunction,
    Aborted,
    UpdateInvariant,
    Pack,
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

        // Determine whether we report this error on the source
        let on_source =
            error.kind.is_from_verification() && self.to_proper_source_location(location).is_some();

        // Create label (position information) for main error.
        let label = Label::new_primary(if on_source {
            // Location is on original source.
            location.unwrap().1
        } else {
            // Location is on generated boogie.
            ByteSpan::new(index, index)
        });

        // Helper to add a diag either to source module or to target
        let mut add_diag = |diag| {
            if on_source {
                self.env.get_module(location.unwrap().0).add_diag(diag);
            } else {
                diags_on_target.push(diag);
            }
        };

        // Add error
        add_diag(
            Diagnostic::new(
                if on_source {
                    Severity::Error
                } else {
                    Severity::Bug
                },
                &error.message,
            )
            .with_label(label),
        );

        // Now add trace diagnostics.
        if error.kind.is_from_verification() && !error.execution_trace.is_empty() {
            // Add trace information.
            let mut locals_shown = BTreeSet::new();
            let mut aborted = false;
            let cleaned_trace = error
                .execution_trace
                .iter()
                .filter_map(|(pos, kind, msg)| {
                    // If trace entry has no source location or is from prelude, skip
                    let trace_location =
                        self.to_proper_source_location(self.get_locations(*pos).1)?;

                    // If trace info is not about function code, skip.
                    let module_emv = self.env.get_module(trace_location.0);
                    if module_emv
                        .get_enclosing_function(trace_location.1)
                        .is_some()
                    {
                        Some((pos, trace_location, kind, msg))
                    } else {
                        // TODO: we may want to extend to track struct generated code.
                        None
                    }
                })
                .dedup_by(|(_, location1, _, _), (_, location2, kind2, _)| {
                    // Removes consecutive entries which point to the same location if trace
                    // minimization is active.
                    self.env.options.minimize_execution_trace
                        && location1 == location2
                        // Only dedup regular trace infos.
                        && *kind2 == &TraceKind::Regular
                })
                // Cut the trace after abort -- the remaining only proapagtes the abort up
                // the call stack and isn't useful.
                .filter_map(|(pos, (module_idx, loc), mut kind, msg)| {
                    if aborted {
                        return None;
                    }
                    let module_env = self.env.get_module(module_idx);
                    aborted = if let Some(model) = &error.model {
                        model
                            .tracked_aborts
                            .get(&(module_env.get_module_idx(), loc.start()))
                            .is_some()
                    } else {
                        false
                    };
                    if aborted {
                        kind = &TraceKind::Aborted
                    }
                    Some((pos, (module_idx, loc), kind, msg))
                })
                .collect_vec();

            let n = cleaned_trace.len();
            let trace = cleaned_trace
                .into_iter()
                .enumerate()
                .map(|(i, (pos, (module_idx, loc), kind, msg))| {
                    if on_source {
                        let module_env = self.env.get_module(module_idx);
                        let (source_file, source_pos) = module_env.get_position(loc);
                        self.render_trace_info(
                            source_file,
                            source_pos,
                            self.pretty_trace_info(
                                &module_env,
                                loc,
                                error.model.as_ref(),
                                &mut locals_shown,
                                *kind,
                                i == n - 1,
                                msg,
                            ),
                        )
                    } else {
                        self.render_trace_info(
                            self.env.options.output_path.clone(),
                            *pos,
                            PrettyDoc::text(format!("<boogie>: {}", msg)),
                        )
                    }
                })
                .collect_vec();
            add_diag(Diagnostic::new(
                Severity::Help,
                &format!("Execution trace for previous error:\n{}", trace.join("\n")),
            ))
        }
    }

    /// Transform a source location into a proper one, filtering out pseudo module locations.
    fn to_proper_source_location(
        &self,
        location: Option<(ModuleIndex, Loc)>,
    ) -> Option<(ModuleIndex, Loc)> {
        location.filter(|(module_idx, _)| *module_idx != PSEUDO_PRELUDE_MODULE)
    }

    /// Renders trace information.
    fn render_trace_info(
        &self,
        file: String,
        pos: (LineIndex, ColumnIndex),
        info: PrettyDoc,
    ) -> String {
        let mut lines = vec![];
        PrettyDoc::text(format!(
            "    at {}:{}:{}: ",
            file,
            (pos.0).0 + 1,
            (pos.1).0 + 1
        ))
        .append(info.nest(8))
        .render(70, &mut lines)
        .unwrap();
        String::from_utf8_lossy(&lines).to_string()
    }

    /// Gets the code byte index and source location (if available) from a target line/column
    /// position.
    fn get_locations(&self, pos: Position) -> (ByteIndex, Option<(ModuleIndex, Loc)>) {
        let index = self.writer.get_output_byte_index(pos.0, pos.1).unwrap();
        (index, self.writer.get_source_location(index))
    }

    /// Pretty print trace information, injecting function name and variable bindings if available.
    fn pretty_trace_info(
        &'env self,
        module_env: &ModuleEnv<'env>,
        mut loc: Loc,
        model_opt: Option<&'env Model>,
        locals_shown: &mut BTreeSet<(CodeIndex, LocalDescriptor)>,
        kind: TraceKind,
        is_last: bool,
        _boogie_msg: &str,
    ) -> PrettyDoc {
        let mut has_info = false;
        let kind_tag = match kind {
            TraceKind::EnterFunction => {
                // Narrow location to the beginning of the function.
                // Do not narrow location if this is the last trace entry --
                // otherwise we wont show variables at this point.
                loc = Loc::new(loc.start(), loc.start() + ByteOffset(1));
                " (entry)"
            }
            TraceKind::ExitFunction => {
                // Narrow location to the end of the function.
                if loc.end().0 > 0 {
                    loc = Loc::new(loc.end() - ByteOffset(1), loc.end());
                }
                " (exit)"
            }
            TraceKind::Aborted => " (ABORTED)",
            TraceKind::UpdateInvariant => " (invariant)",
            TraceKind::Pack => " (pack)",
            _ => "",
        };
        if let Some(func_env) = module_env.get_enclosing_function(loc) {
            let func_name = func_env.get_name().to_string();
            let func_display = format!("{}{}", func_name, kind_tag);
            if is_last {
                // If this is the last trace entry, set the location to the end of the function.
                // This ensures that all variables which haven't yet will be printed
                let mut end = func_env.get_loc().end().to_usize();
                if end > 0 {
                    end -= 1;
                }
                loc = Loc::new(ByteIndex(end as u32), ByteIndex((end + 1) as u32));
            }
            let model_info = if let Some(model) = model_opt {
                let displayed_info = model
                    .relevant_tracked_locals(&func_env, loc, locals_shown)
                    .iter()
                    .map(|(var, val)| {
                        let ty = self.get_type_of_local_or_return(&func_env, var.var_idx);
                        var.pretty(self.env, &func_env, model, &ty, val)
                    })
                    .collect_vec();
                has_info = !displayed_info.is_empty();
                PrettyDoc::intersperse(
                    displayed_info,
                    PrettyDoc::text(",").append(PrettyDoc::hardline()),
                )
            } else {
                PrettyDoc::nil()
            };
            if has_info {
                PrettyDoc::text(func_display)
                    // DEBUG
                    // .append(PrettyDoc::text(format!(" ({}) ", loc)))
                    .append(PrettyDoc::hardline())
                    .append(model_info)
            } else {
                PrettyDoc::text(func_display)
            }
        } else {
            PrettyDoc::text(format!("<unknown function>{}", kind_tag))
        }
    }

    /// Returns the type of either a local or a return parameter.
    fn get_type_of_local_or_return(&self, func_env: &FunctionEnv<'_>, idx: usize) -> GlobalType {
        let n = func_env.get_local_count();
        if idx < n {
            func_env.get_local_type(idx)
        } else {
            func_env.get_return_types()[idx - n].clone()
        }
    }

    /// Extracts verification errors from Boogie output.
    ///
    /// This parses the output for errors of the form:
    ///
    /// ```ignore
    /// *** MODEL
    /// ...
    /// *** END MODEL
    /// (0,0): Error BP5003: A postcondition might not hold on this return path.
    /// output.bpl(2964,1): Related location: This is the postcondition that might not hold.
    /// Execution trace:
    ///    output.bpl(3068,5): anon0
    ///    output.bpl(2960,23): inline$LibraAccount_pay_from_sender_with_metadata$0$Entry
    ///    output.bpl(2989,5): inline$LibraAccount_pay_from_sender_with_metadata$0$anon0
    ///    ...
    /// ```
    fn extract_verification_errors(&self, out: &str) -> Vec<BoogieError> {
        let model_region =
            Regex::new(r"(?m)^\*\*\* MODEL$(?P<mod>(?s:.)*?^\*\*\* END_MODEL$)").unwrap();
        let verification_diag_start =
            Regex::new(r"(?m)^.*\((?P<line>\d+),(?P<col>\d+)\): Error BP\d+:(?P<msg>.*)$").unwrap();
        let verification_diag_related =
            Regex::new(r"(?m)^.+\((?P<line>\d+),(?P<col>\d+)\): Related.*$").unwrap();
        let verification_diag_trace = Regex::new(r"(?m)^Execution trace:$").unwrap();
        let verification_diag_trace_entry =
            Regex::new(r"(?m)^    .*\((?P<line>\d+),(?P<col>\d+)\): (?P<msg>.*)$").unwrap();
        let mut errors = vec![];
        let mut at: usize = 0;
        loop {
            let model = model_region.captures(&out[at..]).and_then(|cap| {
                at += cap.get(0).unwrap().end();
                match Model::parse(self.env, cap.name("mod").unwrap().as_str()) {
                    Ok(model) => Some(model),
                    Err(parse_error) => {
                        warn!("failed to parse boogie model: {}", parse_error.0);
                        None
                    }
                }
            });

            if let Some(cap) = verification_diag_start.captures(&out[at..]) {
                let msg = cap.name("msg").unwrap().as_str();
                at += cap.get(0).unwrap().end();
                // Check whether there is a `Related` message which points to the pre/post condition.
                // If so, this has the real position.
                let (line, col) = if let Some(cap1) = verification_diag_related.captures(&out[at..])
                {
                    at += cap1.get(0).unwrap().end();
                    let line = cap1.name("line").unwrap().as_str();
                    let col = cap1.name("col").unwrap().as_str();
                    (line, col)
                } else {
                    let line = cap.name("line").unwrap().as_str();
                    let col = cap.name("col").unwrap().as_str();
                    (line, col)
                };
                let mut trace = vec![];
                if let Some(m) = verification_diag_trace.find(&out[at..]) {
                    at += m.end();
                    while let Some(cap) = verification_diag_trace_entry.captures(&out[at..]) {
                        let line = cap.name("line").unwrap().as_str();
                        let col = cap.name("col").unwrap().as_str();
                        let msg = cap.name("msg").unwrap().as_str();
                        let trace_kind = if msg.ends_with("$Entry") {
                            TraceKind::EnterFunction
                        } else if msg.ends_with("$Return") {
                            TraceKind::ExitFunction
                        } else if msg.contains("_update_inv$") {
                            TraceKind::UpdateInvariant
                        } else if msg.contains("$Pack_") {
                            TraceKind::Pack
                        } else {
                            TraceKind::Regular
                        };
                        trace.push((make_position(line, col), trace_kind, msg.to_string()));
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
                    kind: if msg.contains("assertion might not hold") {
                        BoogieErrorKind::Assertion
                    } else {
                        BoogieErrorKind::Postcondition
                    },
                    position: make_position(line, col),
                    message: msg.to_string(),
                    execution_trace: trace,
                    model,
                });
            } else {
                break;
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
                    model: None,
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

// -----------------------------------------------
// # Boogie Model Analysis

/// A type to uniquely describe a code location -- consisting of a module index and the byte index
/// into that modules code.
type CodeIndex = (ModuleIndex, ByteIndex);

/// Represents a boogie model.
#[derive(Debug)]
pub struct Model {
    vars: BTreeMap<ModelValue, ModelValue>,
    tracked_locals: BTreeMap<CodeIndex, Vec<(LocalDescriptor, ModelValue)>>,
    tracked_aborts: BTreeMap<CodeIndex, AbortDescriptor>,
}

impl Model {
    /// Parses the given string into a model. The string is expected to end with MODULE_END_MARKER.
    fn parse(env: &GlobalEnv, input: &str) -> Result<Model, ModelParseError> {
        let mut model_parser = ModelParser { input, at: 0 };
        model_parser
            .parse_map()
            .and_then(|m| {
                model_parser.expect(MODEL_END_MARKER)?;
                Ok(m)
            })
            .and_then(|m| match m {
                ModelValue::Map(vars) => Ok(vars),
                _ => Err(ModelParseError("expected ModelValue::Map".to_string())),
            })
            .and_then(|vars| {
                // Extract the tracked locals.
                let mut tracked_locals: BTreeMap<CodeIndex, Vec<(LocalDescriptor, ModelValue)>> =
                    BTreeMap::new();
                let track_local_map = vars
                    .get(&ModelValue::literal("$DebugTrackLocal"))
                    .and_then(|x| x.extract_map())
                    .ok_or_else(Self::invalid_track_info)?;
                for k in track_local_map.keys() {
                    if k == &ModelValue::literal("else") {
                        continue;
                    }
                    let (var, code_idx, value) = Self::extract_debug_var(env, k)?;
                    let idx = (var.module_idx, code_idx);
                    if let Some(vec) = tracked_locals.get_mut(&idx) {
                        vec.push((var, value));
                    } else {
                        tracked_locals.insert(idx, vec![(var, value)]);
                    }
                }

                // Extract the tracked aborts.
                let mut tracked_aborts = BTreeMap::new();
                let track_abort_map = vars
                    .get(&ModelValue::literal("$DebugTrackAbort"))
                    .and_then(|x| x.extract_map())
                    .ok_or_else(Self::invalid_track_info)?;
                for k in track_abort_map.keys() {
                    if k == &ModelValue::literal("else") {
                        continue;
                    }
                    let (mark, code_idx) = Self::extract_abort_marker(env, k)?;
                    tracked_aborts.insert((mark.module_idx, code_idx), mark);
                }
                // DEBUG
                // for ((_, byte_idx), values) in &tracked_locals {
                //     info!("{} -> ", byte_idx);
                //     for (var, val) in values {
                //         info!("  {} -> {:?}", var.var_idx, val);
                //     }
                // }
                // END DEBUG
                let model = Model {
                    vars,
                    tracked_locals,
                    tracked_aborts,
                };
                Ok(model)
            })
    }

    /// Extract and validate a tracked local from $DebugTrackLocal map.
    fn extract_debug_var(
        env: &GlobalEnv,
        map_entry: &ModelValue,
    ) -> Result<(LocalDescriptor, ByteIndex, ModelValue), ModelParseError> {
        if let ModelValue::List(args) = map_entry {
            if args.len() != 5 {
                return Err(Self::invalid_track_info());
            }
            let module_idx = args[0]
                .extract_number()
                .ok_or_else(Self::invalid_track_info)
                .and_then(Self::index_range_check(env.get_module_count()))?;
            let module_env = env.get_module(module_idx);
            let func_idx = args[1]
                .extract_number()
                .ok_or_else(Self::invalid_track_info)
                .and_then(Self::index_range_check(module_env.get_function_count()))?;
            let func_env = module_env.get_function(FunctionDefinitionIndex(func_idx as u16));
            let var_idx = args[2]
                .extract_number()
                .ok_or_else(Self::invalid_track_info)
                .and_then(Self::index_range_check(
                    func_env.get_local_count() + func_env.get_return_count(),
                ))?;

            let code_idx = args[3]
                .extract_number()
                .ok_or_else(Self::invalid_track_info)?;
            Ok((
                LocalDescriptor {
                    module_idx,
                    func_idx: FunctionDefinitionIndex(func_idx as u16),
                    var_idx,
                },
                ByteIndex(code_idx as RawIndex),
                (&args[4]).clone(),
            ))
        } else {
            Err(Self::invalid_track_info())
        }
    }

    /// Extract and validate a tracked abort from $DebugTrackAbort map.
    fn extract_abort_marker(
        env: &GlobalEnv,
        map_entry: &ModelValue,
    ) -> Result<(AbortDescriptor, ByteIndex), ModelParseError> {
        if let ModelValue::List(args) = map_entry {
            if args.len() != 3 {
                return Err(Self::invalid_track_info());
            }
            let module_idx = args[0]
                .extract_number()
                .ok_or_else(Self::invalid_track_info)
                .and_then(Self::index_range_check(env.get_module_count()))?;
            let module_env = env.get_module(module_idx);
            let func_idx = args[1]
                .extract_number()
                .ok_or_else(Self::invalid_track_info)
                .and_then(Self::index_range_check(module_env.get_function_count()))?;
            let code_idx = args[2]
                .extract_number()
                .ok_or_else(Self::invalid_track_info)?;
            Ok((
                AbortDescriptor {
                    module_idx,
                    func_idx: FunctionDefinitionIndex(func_idx as u16),
                },
                ByteIndex(code_idx as RawIndex),
            ))
        } else {
            Err(Self::invalid_track_info())
        }
    }

    fn invalid_track_info() -> ModelParseError {
        ModelParseError::new("invalid debug track info")
    }

    fn index_range_check(max: usize) -> impl FnOnce(usize) -> Result<usize, ModelParseError> {
        move |idx: usize| -> Result<usize, ModelParseError> {
            if idx < max {
                Ok(idx)
            } else {
                Err(ModelParseError::new(&format!(
                    "invalid debug track info: index out of range (upper bound {}, got {})",
                    max, idx
                )))
            }
        }
    }

    /// Computes the relevant tracked locals to show at the given location `loc`.
    ///
    /// As locations reported via boogie execution traces aren't actually pointing to the
    /// place where a variable is updated (rather they often point to branch points), we apply
    /// the following heuristic: all variables are returned which have been updated from
    /// function start until `loc.end()` and which have *not* yet been shown before. We
    /// track this via the `locals_shown` set which consists of pairs of code byte index and the
    /// variable as it has been updated at this point. This heuristic works reliable for the
    /// cases seen so far, but may need further refinement.
    fn relevant_tracked_locals(
        &self,
        func_env: &FunctionEnv<'_>,
        loc: Loc,
        locals_shown: &mut BTreeSet<(CodeIndex, LocalDescriptor)>,
    ) -> Vec<(LocalDescriptor, ModelValue)> {
        let module_idx = func_env.module_env.get_module_idx();
        let range = (module_idx, func_env.get_loc().start())..(module_idx, loc.end());
        self.tracked_locals
            .range(range)
            .flat_map(|(idx, vars)| {
                let mut res = vec![];
                for (var, val) in vars {
                    if locals_shown.insert((*idx, var.clone())) {
                        res.push((var.clone(), val.clone()))
                    }
                }
                res
            })
            .collect_vec()
    }
}

/// Represents a model value.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ModelValue {
    Literal(String),
    List(Vec<ModelValue>),
    Map(BTreeMap<ModelValue, ModelValue>),
}

impl ModelValue {
    /// Makes a literal from a str.
    fn literal(s: &str) -> ModelValue {
        ModelValue::Literal(s.to_string())
    }

    // Makes an error value.
    fn error() -> ModelValue {
        ModelValue::List(vec![ModelValue::literal("Error")])
    }

    /// Extracts a vector from `(Vector value_array)`. This follows indirections in the model
    /// to extract the actual values.
    fn extract_vector(&self, model: &Model) -> Option<Vec<ModelValue>> {
        let args = self.extract_list("Vector")?;
        if args.len() != 1 {
            return None;
        }
        args[0].extract_value_array(model)
    }

    /// Extracts a value array from `(ValueArray map_key size)`. This follows indirections in the
    /// model. We find the value array map at `Select_[$int]Value`.
    fn extract_value_array(&self, model: &Model) -> Option<Vec<ModelValue>> {
        let args = self.extract_list("ValueArray")?;
        if args.len() != 2 {
            return None;
        }
        let size = (&args[1]).extract_number()?;
        let map_key = &args[0];
        let value_array_map = model
            .vars
            .get(&ModelValue::literal("Select_[$int]Value"))?
            .extract_map()?;

        Some(
            (0..size)
                .map(|i| {
                    let key = ModelValue::List(vec![
                        map_key.clone(),
                        ModelValue::literal(&i.to_string()),
                    ]);
                    value_array_map
                        .get(&key)
                        .cloned()
                        .unwrap_or_else(ModelValue::error)
                })
                .collect_vec(),
        )
    }

    fn extract_map(&self) -> Option<&BTreeMap<ModelValue, ModelValue>> {
        if let ModelValue::Map(map) = self {
            Some(map)
        } else {
            None
        }
    }

    /// Extract the arguments of a list of the form `(<ctor> element...)`.
    fn extract_list(&self, ctor: &str) -> Option<&[ModelValue]> {
        if let ModelValue::List(elems) = self {
            if !elems.is_empty() && elems[0] == ModelValue::literal(ctor) {
                return Some(&elems[1..]);
            }
        }
        None
    }

    /// Extract a number from a literal.
    fn extract_number(&self) -> Option<usize> {
        if let Ok(n) = self.extract_literal()?.parse::<usize>() {
            Some(n)
        } else {
            None
        }
    }

    /// Extract the value of a primitive.
    fn extract_primitive(&self, ctor: &str) -> Option<&String> {
        let args = self.extract_list(ctor)?;
        if args.len() != 1 {
            return None;
        }
        (&args[0]).extract_literal()
    }

    /// Extract a literal.
    fn extract_literal(&self) -> Option<&String> {
        if let ModelValue::Literal(s) = self {
            Some(s)
        } else {
            None
        }
    }

    /// Pretty prints the given model value which has given type. If printing fails, falls
    /// back to print the debug value.
    pub fn pretty_or_raw<'env>(
        &self,
        env: &'env GlobalEnv,
        model: &Model,
        ty: &GlobalType,
    ) -> PrettyDoc {
        self.pretty(env, model, ty).unwrap_or_else(|| {
            // Print the raw debug value.
            PrettyDoc::text(format!("<? {:?}>", self))
        })
    }

    /// Pretty prints the given model value which has given type.
    pub fn pretty(&self, env: &GlobalEnv, model: &Model, ty: &GlobalType) -> Option<PrettyDoc> {
        if self.extract_list("Error").is_some() {
            // This is an undefined value
            return Some(PrettyDoc::text("<undef>"));
        }
        match ty {
            GlobalType::U8 => Some(PrettyDoc::text(format!(
                "{}u8",
                self.extract_primitive("Integer")?
            ))),
            GlobalType::U64 => Some(PrettyDoc::text(
                self.extract_primitive("Integer")?.to_string(),
            )),
            GlobalType::U128 => Some(PrettyDoc::text(format!(
                "{}u128",
                self.extract_primitive("Integer")?.to_string()
            ))),
            GlobalType::Bool => Some(PrettyDoc::text(
                self.extract_primitive("Boolean")?.to_string(),
            )),
            GlobalType::Address => {
                let addr = BigInt::parse_bytes(
                    &self.extract_primitive("Address")?.clone().into_bytes(),
                    10,
                )?;
                Some(PrettyDoc::text(format!("0x{}", &addr.to_str_radix(16))))
            }
            GlobalType::ByteArray => {
                // Right now byte arrays are abstract in boogie, so we can only distinguish
                // instances.
                let repr = self.extract_primitive("ByteArray")?;
                let display = if repr.starts_with("T@ByteArray!val!") {
                    &repr["T@ByteArray!val!".len()..]
                } else {
                    repr
                };
                Some(PrettyDoc::text(format!("<bytearray {}>", display)))
            }
            GlobalType::Struct(module_idx, struct_idx, params) => {
                self.pretty_struct_or_vector(env, model, *module_idx, *struct_idx, &params)
            }
            GlobalType::Reference(bt) | GlobalType::MutableReference(bt) => {
                Some(PrettyDoc::text("&").append(self.pretty(env, model, &*bt)?))
            }
            GlobalType::Subrange => {
                // TODO (DD) *** Not sure how to do this. Need to extract subrange from model?
                Some(PrettyDoc::text("Unimplemented subrange lb..ub"))
            }
            GlobalType::TypeParameter(_) => {
                // The value of a generic cannot be easily displayed because we do not know the
                // actual type unless we parse it out from the model (via the type value parameter)
                // and convert into a GlobalType. However, since the value is parametric and cannot
                // effect the verification outcome, we may not have much need for seeing it.
                Some(PrettyDoc::text("<generic>"))
            }
        }
    }

    /// Pretty prints a struct or vector.
    pub fn pretty_struct_or_vector(
        &self,
        env: &GlobalEnv,
        model: &Model,
        module_idx: ModuleIndex,
        struct_idx: StructDefinitionIndex,
        params: &[GlobalType],
    ) -> Option<PrettyDoc> {
        let error = ModelValue::error();
        let module_env = env.get_module(module_idx);
        let struct_env = module_env.get_struct(struct_idx);
        let values = self.extract_vector(model)?;
        let entries = if struct_env.is_vector() {
            values
                .iter()
                .map(|v| v.pretty_or_raw(env, model, &params[0]))
                .collect_vec()
        } else {
            struct_env
                .get_fields()
                .enumerate()
                .map(|(i, f)| {
                    let ty = f.get_type().instantiate(params);
                    let v = if i < values.len() { &values[i] } else { &error };
                    let vp = v.pretty_or_raw(env, model, &ty);
                    PrettyDoc::text(f.get_name().as_str().to_string())
                        .append(PrettyDoc::text(" ="))
                        .append(PrettyDoc::line().append(vp).nest(2).group())
                })
                .collect_vec()
        };
        Some(
            PrettyDoc::text(format!(
                "{}.{}",
                struct_env.module_env.get_id().name(),
                struct_env.get_name()
            ))
            .append(PrettyDoc::text("{"))
            .append(
                PrettyDoc::line_()
                    .append(PrettyDoc::intersperse(
                        entries,
                        PrettyDoc::text(",").append(PrettyDoc::line()),
                    ))
                    .nest(2)
                    .group(),
            )
            .append(PrettyDoc::text("}")),
        )
    }
}

/// Represents a descriptor for a tracked local.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct LocalDescriptor {
    module_idx: ModuleIndex,
    func_idx: FunctionDefinitionIndex,
    var_idx: usize,
}

impl LocalDescriptor {
    /// Pretty prints a tracked locals.
    fn pretty(
        &self,
        env: &GlobalEnv,
        func_env: &FunctionEnv<'_>,
        model: &Model,
        ty: &GlobalType,
        resolved_value: &ModelValue,
    ) -> PrettyDoc {
        let n = func_env.get_local_count();
        let var_name = if self.var_idx >= n {
            if func_env.get_return_count() > 1 {
                format!("RET({})", self.var_idx - n)
            } else {
                "RET".to_string()
            }
        } else {
            func_env.get_local_name(self.var_idx)
        };
        PrettyDoc::text(var_name)
            .append(PrettyDoc::space())
            .append(PrettyDoc::text("="))
            .append(
                PrettyDoc::line()
                    .append(resolved_value.pretty_or_raw(env, model, ty))
                    .nest(2)
                    .group(),
            )
    }
}

/// Represents an abort descriptor.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct AbortDescriptor {
    module_idx: ModuleIndex,
    func_idx: FunctionDefinitionIndex,
}

/// Represents parser for a boogie model.
struct ModelParser<'s> {
    input: &'s str,
    at: usize,
}

/// Represents error resulting from model parsing.
struct ModelParseError(String);

impl ModelParseError {
    fn new(s: &str) -> Self {
        ModelParseError(s.to_string())
    }
}

const MODEL_END_MARKER: &str = "*** END_MODEL";

impl<'s> ModelParser<'s> {
    fn skip_space(&mut self) {
        while self.input[self.at..].starts_with(|ch| [' ', '\r', '\n', '\t'].contains(&ch)) {
            self.at += 1;
        }
    }

    fn looking_at(&mut self, s: &str) -> bool {
        self.skip_space();
        self.input[self.at..].starts_with(s)
    }

    fn looking_at_then_consume(&mut self, s: &str) -> bool {
        if self.looking_at(s) {
            self.at += s.len();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, s: &str) -> Result<(), ModelParseError> {
        self.skip_space();
        if self.input[self.at..].starts_with(s) {
            self.at += s.len();
            Ok(())
        } else {
            let end = std::cmp::min(self.at + 80, self.input.len());
            Err(ModelParseError(format!(
                "expected `{}` (at `{}...`)",
                s,
                &self.input[self.at..end]
            )))
        }
    }

    fn parse_map(&mut self) -> Result<ModelValue, ModelParseError> {
        let mut map = BTreeMap::new();
        while !self.looking_at("}") && !self.looking_at(MODEL_END_MARKER) {
            let key = self.parse_key()?;
            self.expect("->")?;
            let value = if self.looking_at_then_consume("{") {
                let value = self.parse_map()?;
                self.expect("}")?;
                value
            } else {
                self.parse_value()?
            };
            map.insert(key, value);
        }
        Ok(ModelValue::Map(map))
    }

    fn parse_key(&mut self) -> Result<ModelValue, ModelParseError> {
        let mut comps = vec![];
        while !self.looking_at("->") {
            let value = self.parse_value()?;
            comps.push(value);
        }
        if comps.is_empty() {
            Err(ModelParseError(
                "expected at least one component of a key".to_string(),
            ))
        } else if comps.len() == 1 {
            Ok(comps.pop().unwrap())
        } else {
            Ok(ModelValue::List(comps))
        }
    }

    fn parse_value(&mut self) -> Result<ModelValue, ModelParseError> {
        if self.looking_at_then_consume("(") {
            let mut comps = vec![];
            while !self.looking_at_then_consume(")") {
                let value = self.parse_value()?;
                comps.push(value);
            }
            Ok(ModelValue::List(comps))
        } else {
            // We do not know the exact lexis, so take everything until next space or ).
            self.skip_space();
            let start = self.at;
            while self.at < self.input.len()
                && !self.input[self.at..]
                    .starts_with(|ch| [')', ' ', '\r', '\n', '\t'].contains(&ch))
            {
                self.at += 1;
            }
            Ok(ModelValue::Literal(self.input[start..self.at].to_string()))
        }
    }
}
