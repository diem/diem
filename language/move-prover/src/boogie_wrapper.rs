// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Wrapper around the boogie program. Allows to call boogie and analyze the output.

use anyhow::anyhow;

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    option::Option::None,
    process::Command,
};

use codespan::{ByteIndex, ByteOffset, ColumnIndex, FileId, LineIndex, Location, Span};
use codespan_reporting::diagnostic::{Diagnostic, Label};
use itertools::Itertools;
use log::{debug, info, warn};
use num::BigInt;
use pretty::RcDoc;
use regex::Regex;

use spec_lang::{
    env::{FunId, GlobalEnv, Loc, ModuleId, StructId},
    ty::{PrimitiveType, Type},
};

use crate::{cli::Options, code_writer::CodeWriter};
use stackless_bytecode_generator::{
    function_target::FunctionTarget, function_target_pipeline::FunctionTargetsHolder,
};

/// A type alias for the way how we use crate `pretty`'s document type. `pretty` is a
/// Wadler-style pretty printer. Our simple usage doesn't require any lifetime management.
type PrettyDoc = RcDoc<'static, ()>;

// -----------------------------------------------
// # Boogie Wrapper

/// Represents the boogie wrapper.
pub struct BoogieWrapper<'env> {
    pub env: &'env GlobalEnv,
    pub targets: &'env FunctionTargetsHolder,
    pub writer: &'env CodeWriter,
    pub options: &'env Options,
    pub boogie_file_id: FileId,
}

/// Output of a boogie run.
pub struct BoogieOutput {
    /// All errors which could be parsed from the output.
    pub errors: Vec<BoogieError>,

    /// Full output as a string.
    pub all_output: String,
}

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
    pub position: Location,
    pub message: String,
    pub execution_trace: Vec<(Location, TraceKind, String)>,
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
    pub fn call_boogie(&self, boogie_file: &str) -> anyhow::Result<BoogieOutput> {
        let args = self.options.get_boogie_command(boogie_file);
        info!("calling boogie");
        debug!("command line: {}", args.iter().join(" "));
        let output = Command::new(&args[0]).args(&args[1..]).output()?;
        if !output.status.success() {
            Err(anyhow!(
                "boogie exited with status {:?}",
                output.status.code()
            ))
        } else {
            let out = String::from_utf8_lossy(&output.stdout).to_string();
            let mut errors = self.extract_verification_errors(&out);
            errors.extend(self.extract_compilation_errors(&out));
            Ok(BoogieOutput {
                errors,
                all_output: out,
            })
        }
    }

    /// Calls boogie and analyzes output.
    pub fn call_boogie_and_verify_output(&self, boogie_file: &str) -> anyhow::Result<()> {
        let BoogieOutput { errors, all_output } = self.call_boogie(boogie_file)?;
        let boogie_log_file = self.options.get_boogie_log_file(boogie_file);
        info!("writing boogie log to {}", boogie_log_file);
        fs::write(&boogie_log_file, &all_output)?;

        for error in &errors {
            self.add_error(error);
        }
        Ok(())
    }

    /// Helper to add a boogie error as a codespan Diagnostic.
    fn add_error(&self, error: &BoogieError) {
        let (index, loc_opt) = self.get_locations(error.position);
        // Create label (position information) for main error.
        let loc_opt = self.to_proper_source_location(loc_opt).and_then(|loc| {
            if error.kind.is_from_verification() {
                Some(loc)
            } else {
                None
            }
        });
        let on_source = loc_opt.is_some();
        let label = if let Some(loc) = &loc_opt {
            Label::primary(loc.file_id(), loc.span()).with_message("")
        } else {
            Label::primary(self.boogie_file_id, Span::new(index, index)).with_message("")
        };

        let mut diag = if on_source {
            Diagnostic::error()
        } else {
            Diagnostic::bug()
        }
        .with_message(&error.message)
        .with_labels(vec![label]);

        // Now add trace diagnostics.
        if error.kind.is_from_verification()
            && !error.execution_trace.is_empty()
            // Reporting errors on boogie source seems to have some non-determinism, so skip
            // this if stable output is required
            && (on_source || !self.options.stable_test_output)
        {
            let mut locals_shown = BTreeSet::new();
            let mut aborted = false;
            let cleaned_trace = error
                .execution_trace
                .iter()
                .filter_map(|(orig_pos, kind, msg)| {
                    // If trace entry has no source location or is from prelude, skip
                    let source_loc =
                        self.to_proper_source_location(self.get_locations(*orig_pos).1)?;

                    // If trace entry points to non-function location, skip
                    self.env.get_enclosing_function(source_loc.clone())?;

                    // Get file/location info; if not available, skip.
                    let source_pos = self.env.get_position(source_loc.clone())?;
                    Some((orig_pos, source_loc, source_pos, kind, msg))
                })
                .dedup_by(|(_, _, source_pos1, _, _), (_, _, source_pos2, kind2, _)| {
                    // Removes consecutive entries which point to the same location if trace
                    // minimization is active.
                    if !self.options.minimize_execution_trace {
                        return false;
                    }
                    if *kind2 != &TraceKind::Regular {
                        // Only dedup regular trace infos.
                        return false;
                    }
                    source_pos1.1.line == source_pos2.1.line
                })
                // Cut the trace after abort -- the remaining only propagates the abort up
                // the call stack and isn't useful.
                .filter_map(|(orig_pos, source_loc, source_pos, mut kind, msg)| {
                    if aborted {
                        return None;
                    }
                    aborted = error
                        .model
                        .as_ref()
                        .map(|model| model.tracked_aborts.get(&source_loc).is_some())
                        .unwrap_or(false);
                    if aborted {
                        kind = &TraceKind::Aborted
                    }
                    Some((orig_pos, source_loc, source_pos, kind, msg))
                })
                .collect_vec();

            let n = cleaned_trace.len();
            let trace = cleaned_trace
                .into_iter()
                .enumerate()
                .map(|(i, (orig_pos, source_loc, source_pos, kind, msg))| {
                    if on_source {
                        self.render_trace_info(
                            source_pos.0,
                            source_pos.1,
                            self.pretty_trace_info(
                                source_loc,
                                error.model.as_ref(),
                                &mut locals_shown,
                                *kind,
                                i == n - 1,
                            ),
                        )
                    } else {
                        // Fall back to report on boogie level
                        self.render_trace_info(
                            self.options.output_path.clone(),
                            *orig_pos,
                            PrettyDoc::text(format!("<boogie>: {}", msg)),
                        )
                    }
                })
                .flat_map(|entry| {
                    str::lines(entry.as_str())
                        .map(|s| s.to_string())
                        .collect_vec()
                })
                .collect_vec();
            diag = diag.with_notes(trace);
        }
        self.env.add_diag(diag);
    }

    /// Transform a source location into a proper one, filtering out internal locations.
    fn to_proper_source_location(&self, location: Option<Loc>) -> Option<Loc> {
        location.filter(|loc| loc != &self.env.internal_loc())
    }

    /// Renders trace information.
    fn render_trace_info(&self, file: String, pos: Location, info: PrettyDoc) -> String {
        let mut lines = vec![];
        PrettyDoc::text(format!(
            "    at {}:{}:{}: ",
            file,
            pos.line.0 + 1,
            pos.column.0 + 1
        ))
        .append(info.nest(8))
        .render(70, &mut lines)
        .unwrap();
        String::from_utf8_lossy(&lines).to_string()
    }

    /// Gets the code byte index and source location (if available) from a target line/column
    /// position.
    fn get_locations(&self, pos: Location) -> (ByteIndex, Option<Loc>) {
        let index = self
            .writer
            .get_output_byte_index(pos.line, pos.column)
            .unwrap_or(ByteIndex(0));
        (index, self.writer.get_source_location(index))
    }

    /// Pretty print trace information, injecting function name and variable bindings if available.
    fn pretty_trace_info(
        &'env self,
        mut loc: Loc,
        model_opt: Option<&'env Model>,
        locals_shown: &mut BTreeSet<(Loc, LocalDescriptor)>,
        kind: TraceKind,
        is_last: bool,
    ) -> PrettyDoc {
        let mut has_info = false;
        let kind_tag = match kind {
            TraceKind::EnterFunction => {
                // Narrow location to the start of the function.
                loc = loc.at_start();
                " (entry)"
            }
            TraceKind::ExitFunction => {
                // Narrow location to the end of the function.
                loc = loc.at_end();
                " (exit)"
            }
            TraceKind::Aborted => " (ABORTED)",
            TraceKind::UpdateInvariant => " (invariant)",
            TraceKind::Pack => " (pack)",
            _ => "",
        };
        if let Some(func_env) = self.env.get_enclosing_function(loc.clone()) {
            let func_target = self.targets.get_target(&func_env);
            let func_name = format!(
                "{}",
                func_target.get_name().display(func_target.symbol_pool())
            );
            let func_display = format!("{}{}", func_name, kind_tag);
            if is_last {
                // If this is the last trace entry, set the location to the end of the function.
                // This ensures that all variables which haven't yet will be printed
                loc = loc.at_end();
            }
            let model_info = if let Some(model) = model_opt {
                let displayed_info = model
                    .relevant_tracked_locals(&func_target, loc, locals_shown)
                    .iter()
                    .map(|(var, val)| {
                        let ty = self.get_type_of_local_or_return(&func_target, var.var_idx);
                        var.pretty(self, &func_target, model, &ty, val)
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
                    // .append(PrettyDoc::text(format!(" ({}) ", loc.span())))
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
    fn get_type_of_local_or_return(
        &self,
        func_target: &'env FunctionTarget<'env>,
        idx: usize,
    ) -> &'env Type {
        let n = func_target.get_local_count();
        if idx < n {
            func_target.get_local_type(idx)
        } else {
            func_target.get_return_type(idx - n)
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
            Regex::new(r"^\n.+\((?P<line>\d+),(?P<col>\d+)\): Related.*\n").unwrap();
        let verification_diag_trace = Regex::new(r"(?m)^Execution trace:$").unwrap();
        let verification_diag_trace_entry =
            Regex::new(r"(?m)^    .*\((?P<line>\d+),(?P<col>\d+)\): (?P<msg>.*)$").unwrap();
        let mut errors = vec![];
        let mut at: usize = 0;
        loop {
            let model = model_region.captures(&out[at..]).and_then(|cap| {
                at += cap.get(0).unwrap().end();
                match Model::parse(self, cap.name("mod").unwrap().as_str()) {
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
fn make_position(line_str: &str, col_str: &str) -> Location {
    // This will crash on overflow.
    let mut line = line_str.parse::<u32>().unwrap();
    let col = col_str.parse::<u32>().unwrap();
    if line > 0 {
        line -= 1;
    }
    Location::new(LineIndex(line), ColumnIndex(col))
}

// -----------------------------------------------
// # Boogie Model Analysis

/// Represents a boogie model.
#[derive(Debug)]
pub struct Model {
    vars: BTreeMap<ModelValue, ModelValue>,
    tracked_locals: BTreeMap<Loc, Vec<(LocalDescriptor, ModelValue)>>,
    tracked_aborts: BTreeMap<Loc, AbortDescriptor>,
}

impl Model {
    /// Parses the given string into a model. The string is expected to end with MODULE_END_MARKER.
    fn parse(wrapper: &BoogieWrapper<'_>, input: &str) -> Result<Model, ModelParseError> {
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
                let mut tracked_locals: BTreeMap<Loc, Vec<(LocalDescriptor, ModelValue)>> =
                    BTreeMap::new();
                let track_local_map = vars
                    .get(&ModelValue::literal("$DebugTrackLocal"))
                    .and_then(|x| x.extract_map())
                    .ok_or_else(Self::invalid_track_info)?;
                for k in track_local_map.keys() {
                    if k == &ModelValue::literal("else") {
                        continue;
                    }
                    let (var, loc, value) = Self::extract_debug_var(wrapper, k)?;
                    tracked_locals
                        .entry(loc)
                        .or_insert_with(|| vec![])
                        .push((var, value));
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
                    let (mark, loc) = Self::extract_abort_marker(wrapper, k)?;
                    tracked_aborts.insert(loc, mark);
                }
                // DEBUG
                // for (loc, values) in &tracked_locals {
                //     info!("{} -> ", loc.span());
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
        wrapper: &BoogieWrapper<'_>,
        map_entry: &ModelValue,
    ) -> Result<(LocalDescriptor, Loc, ModelValue), ModelParseError> {
        if let ModelValue::List(args) = map_entry {
            if args.len() != 4 {
                return Err(Self::invalid_track_info());
            }
            let loc = Self::extract_loc(wrapper, args)?;
            let func_env = wrapper
                .env
                .get_enclosing_function(loc.clone())
                .ok_or_else(Self::invalid_track_info)?;
            let func_target = wrapper.targets.get_target(&func_env);
            let var_idx = args[2]
                .extract_number()
                .ok_or_else(Self::invalid_track_info)
                .and_then(Self::index_range_check(
                    func_target.get_local_count() + func_target.get_return_count(),
                ))?;
            Ok((
                LocalDescriptor {
                    module_id: func_target.func_env.module_env.get_id(),
                    func_id: func_target.get_id(),
                    var_idx,
                },
                loc,
                (&args[3]).clone(),
            ))
        } else {
            Err(Self::invalid_track_info())
        }
    }

    /// Extract and validate a tracked abort from $DebugTrackAbort map.
    fn extract_abort_marker(
        wrapper: &BoogieWrapper<'_>,
        map_entry: &ModelValue,
    ) -> Result<(AbortDescriptor, Loc), ModelParseError> {
        if let ModelValue::List(args) = map_entry {
            if args.len() != 2 {
                return Err(Self::invalid_track_info());
            }
            let loc = Self::extract_loc(wrapper, args)?;
            let func_env = wrapper
                .env
                .get_enclosing_function(loc.clone())
                .ok_or_else(Self::invalid_track_info)?;
            let func_target = wrapper.targets.get_target(&func_env);
            Ok((
                AbortDescriptor {
                    module_id: func_target.func_env.module_env.get_id(),
                    func_id: func_target.get_id(),
                },
                loc,
            ))
        } else {
            Err(Self::invalid_track_info())
        }
    }

    // Extract Loc from model values.
    fn extract_loc(
        wrapper: &BoogieWrapper<'_>,
        args: &[ModelValue],
    ) -> Result<Loc, ModelParseError> {
        if args.len() < 2 {
            Err(Self::invalid_track_info())
        } else {
            let file_idx = args[0]
                .extract_number()
                .ok_or_else(Self::invalid_track_info)
                .and_then(Self::index_range_check(wrapper.env.get_file_count()))?;
            let file_id = wrapper.env.file_idx_to_id(file_idx as u16);
            let byte_index = ByteIndex(
                args[1]
                    .extract_number()
                    .ok_or_else(Self::invalid_track_info)
                    .and_then(Self::index_range_check(std::u32::MAX as usize))?
                    as u32,
            );
            Ok(Loc::new(
                file_id,
                Span::new(byte_index, byte_index + ByteOffset(1)),
            ))
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
        func_target: &FunctionTarget<'_>,
        loc: Loc,
        locals_shown: &mut BTreeSet<(Loc, LocalDescriptor)>,
    ) -> BTreeSet<(LocalDescriptor, ModelValue)> {
        let func_loc = func_target.get_loc();
        self.tracked_locals
            .range(func_loc.at_start()..loc.at_end())
            .flat_map(|(loc, vars)| {
                let mut res = vec![];
                for (var, val) in vars {
                    let var_name = func_target.get_local_name(var.var_idx);
                    if func_target.symbol_pool().string(var_name).contains("$$") {
                        // Do not show temporaries generated by the move compiler.
                        continue;
                    }
                    if locals_shown.insert((loc.clone(), var.clone())) {
                        res.push((var.clone(), val.clone()))
                    }
                }
                res
            })
            .collect()
    }
}

/// Represents a model value.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ModelValue {
    Literal(String),
    List(Vec<ModelValue>),
    Map(BTreeMap<ModelValue, ModelValue>),
}

/// Represents a spare representation of a model value vector.
#[derive(Debug)]
pub struct ModelValueVector {
    /// The size of the vector.
    size: usize,
    /// Those indices 0..size which have a value appear here.
    values: BTreeMap<usize, ModelValue>,
    /// Others in this range have assigned this value.
    default: ModelValue,
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
    fn extract_vector(&self, model: &Model) -> Option<ModelValueVector> {
        let args = self.extract_list("Vector")?;
        if args.len() != 1 {
            return None;
        }
        args[0].extract_value_array(model)
    }

    /// Extracts a value array from `(ValueArray map_key size)`. This follows indirections in the
    /// model. We find the value array map at `Select_[$int]Value`. This has e.g. the form
    ///
    /// ```model
    ///   Select_[$int]Value -> {
    //      |T@[Int]Value!val!1| 0 -> (Integer 2)
    //      |T@[Int]Value!val!1| 22 -> (Integer 2)
    //      else -> (Integer 0)
    //    }
    // ```
    fn extract_value_array(&self, model: &Model) -> Option<ModelValueVector> {
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
        let mut values = BTreeMap::new();
        let mut default = ModelValue::error();
        for (key, value) in value_array_map {
            if let ModelValue::List(elems) = key {
                if elems.len() == 2 && &elems[0] == map_key {
                    if let Some(idx) = elems[1].extract_number() {
                        values.insert(idx, value.clone());
                    }
                }
            } else if key == &ModelValue::literal("else") {
                default = value.clone();
            }
        }
        Some(ModelValueVector {
            size,
            values,
            default,
        })
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
    pub fn pretty_or_raw(&self, wrapper: &BoogieWrapper, model: &Model, ty: &Type) -> PrettyDoc {
        if wrapper.options.stable_test_output {
            return PrettyDoc::text("<redacted>");
        }
        self.pretty(wrapper, model, ty).unwrap_or_else(|| {
            // Print the raw debug value.
            PrettyDoc::text(format!("<? {:?}>", self))
        })
    }

    /// Pretty prints the given model value which has given type.
    pub fn pretty(&self, wrapper: &BoogieWrapper, model: &Model, ty: &Type) -> Option<PrettyDoc> {
        if self.extract_list("Error").is_some() {
            // This is an undefined value
            return Some(PrettyDoc::text("<undef>"));
        }
        match ty {
            Type::Primitive(PrimitiveType::U8) => Some(PrettyDoc::text(format!(
                "{}u8",
                self.extract_primitive("Integer")?
            ))),
            Type::Primitive(PrimitiveType::U64) => Some(PrettyDoc::text(
                self.extract_primitive("Integer")?.to_string(),
            )),
            Type::Primitive(PrimitiveType::U128) => Some(PrettyDoc::text(format!(
                "{}u128",
                self.extract_primitive("Integer")?.to_string()
            ))),
            Type::Primitive(PrimitiveType::Num) => Some(PrettyDoc::text(format!(
                "{}u128",
                self.extract_primitive("Integer")?.to_string()
            ))),
            Type::Primitive(PrimitiveType::Bool) => Some(PrettyDoc::text(
                self.extract_primitive("Boolean")?.to_string(),
            )),
            Type::Primitive(PrimitiveType::Address) => {
                let addr = BigInt::parse_bytes(
                    &self.extract_primitive("Address")?.clone().into_bytes(),
                    10,
                )?;
                Some(PrettyDoc::text(format!("0x{}", &addr.to_str_radix(16))))
            }
            Type::Vector(param) => self.pretty_vector(wrapper, model, param),
            Type::Struct(module_id, struct_id, params) => {
                self.pretty_struct(wrapper, model, *module_id, *struct_id, &params)
            }
            Type::Reference(_, bt) => {
                Some(PrettyDoc::text("&").append(self.pretty(wrapper, model, &*bt)?))
            }
            Type::TypeParameter(_) => {
                // The value of a generic cannot be easily displayed because we do not know the
                // actual type unless we parse it out from the model (via the type value parameter)
                // and convert into a Type. However, since the value is parametric and cannot
                // effect the verification outcome, we may not have much need for seeing it.
                Some(PrettyDoc::text("<generic>"))
            }
            _ => None,
        }
    }

    /// Pretty prints the body of a struct or vector, enclosed in braces.
    pub fn pretty_vec_or_struct_body(entries: Vec<PrettyDoc>) -> PrettyDoc {
        PrettyDoc::text("{")
            .append(
                PrettyDoc::line_()
                    .append(PrettyDoc::intersperse(
                        entries,
                        PrettyDoc::text(",").append(PrettyDoc::line()),
                    ))
                    .nest(2)
                    .group(),
            )
            .append(PrettyDoc::text("}"))
    }

    /// Pretty prints a vector.
    pub fn pretty_vector(
        &self,
        wrapper: &BoogieWrapper,
        model: &Model,
        param: &Type,
    ) -> Option<PrettyDoc> {
        let values = self.extract_vector(model)?;
        let mut entries = vec![];
        let mut next = 0;
        let mut sparse = false;
        for idx in values.values.keys().sorted() {
            let mut p = values.values.get(idx)?.pretty_or_raw(wrapper, model, param);
            if *idx > next {
                p = PrettyDoc::text(format!("{}: ", idx)).append(p);
                sparse = true;
            }
            next = *idx + 1;
            entries.push(p);
        }
        if next < values.size || sparse {
            let default = values
                .default
                .pretty(wrapper, model, param)
                .unwrap_or_else(|| PrettyDoc::text("undef"));
            entries.push(PrettyDoc::text("else: ").append(default));
        }
        Some(PrettyDoc::text("vector").append(Self::pretty_vec_or_struct_body(entries)))
    }

    /// Pretty prints a struct.
    pub fn pretty_struct(
        &self,
        wrapper: &BoogieWrapper,
        model: &Model,
        module_id: ModuleId,
        struct_id: StructId,
        params: &[Type],
    ) -> Option<PrettyDoc> {
        let module_env = wrapper.env.get_module(module_id);
        let struct_env = module_env.get_struct(struct_id);
        let values = self.extract_vector(model)?;
        let entries = struct_env
            .get_fields()
            .enumerate()
            .map(|(i, f)| {
                let ty = f.get_type().instantiate(params);
                let v = values.values.get(&i).unwrap_or(&values.default);
                let vp = v.pretty_or_raw(wrapper, model, &ty);
                PrettyDoc::text(format!(
                    "{}",
                    f.get_name().display(struct_env.symbol_pool())
                ))
                .append(PrettyDoc::text(" ="))
                .append(PrettyDoc::line().append(vp).nest(2).group())
            })
            .collect_vec();
        Some(
            PrettyDoc::text(format!(
                "{}.{}",
                struct_env
                    .module_env
                    .get_name()
                    .name()
                    .display(module_env.symbol_pool()),
                struct_env.get_name().display(module_env.symbol_pool())
            ))
            .append(Self::pretty_vec_or_struct_body(entries)),
        )
    }
}

/// Represents a descriptor for a tracked local.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct LocalDescriptor {
    module_id: ModuleId,
    func_id: FunId,
    var_idx: usize,
}

impl LocalDescriptor {
    /// Pretty prints a tracked local.
    fn pretty(
        &self,
        wrapper: &BoogieWrapper,
        func_target: &FunctionTarget<'_>,
        model: &Model,
        ty: &Type,
        resolved_value: &ModelValue,
    ) -> PrettyDoc {
        let n = func_target.get_local_count();
        let var_name = if self.var_idx >= n {
            if func_target.get_return_count() > 1 {
                format!("result_{}", self.var_idx - n)
            } else {
                "result".to_string()
            }
        } else {
            format!(
                "{}",
                func_target
                    .get_local_name(self.var_idx)
                    .display(func_target.symbol_pool())
            )
        };
        PrettyDoc::text(var_name)
            .append(PrettyDoc::space())
            .append(PrettyDoc::text("="))
            .append(
                PrettyDoc::line()
                    .append(resolved_value.pretty_or_raw(wrapper, model, ty))
                    .nest(2)
                    .group(),
            )
    }
}

/// Represents an abort descriptor.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct AbortDescriptor {
    module_id: ModuleId,
    func_id: FunId,
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
