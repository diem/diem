// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Wrapper around the boogie program. Allows to call boogie and analyze the output.

use crate::cli::{abort_on_error, abort_with_error};
use crate::code_writer::CodeWriter;
use crate::driver::PSEUDO_PRELUDE_MODULE;
use crate::env::{FunctionEnv, GlobalEnv, GlobalType, ModuleEnv, ModuleIndex};
use codespan::{
    ByteIndex, ByteOffset, ByteSpan, CodeMap, ColumnIndex, FileName, LineIndex, RawIndex,
};
use codespan_reporting::termcolor::{ColorChoice, StandardStream};
use codespan_reporting::{Diagnostic, Label, Severity};
use ir_to_bytecode_syntax::ast::Loc;
use itertools::Itertools;
use libra_types::identifier::IdentStr;
use log::warn;
use log::{debug, error, info};
use num::BigInt;
use pretty::RcDoc;
use regex::{Captures, Regex};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::option::Option::None;
use std::process::Command;
use vm::file_format::{FunctionDefinitionIndex, StructDefinitionIndex};

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
    pub model: Option<Model>,
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
        let on_source = error.kind == BoogieErrorKind::Verification
            && self.to_proper_source_location(location).is_some();

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
        if error.kind == BoogieErrorKind::Verification && !error.execution_trace.is_empty() {
            // Get debug variables from model.
            let mut vars_shown = BTreeSet::new();
            let trace = error
                .execution_trace
                .iter()
                .filter_map(|(pos, msg)| {
                    // If trace entry has no source location or is from prelude, skip
                    let trace_location =
                        self.to_proper_source_location(self.get_locations(*pos).1)?;
                    Some((pos, trace_location, msg))
                })
                .dedup_by(|(_, location1, _), (_, location2, msg2)| {
                    // Removes consecutive entries which point to the same location if trace
                    // minimization is active.
                    self.env.options.minimize_execution_trace
                        && location1 == location2
                        // Do not dedup the return trace entry. It may have the same location as
                        // the entry trace info, but we need to display it.
                        && !msg2.ends_with("$Return")
                })
                .map(|(pos, (module_idx, loc), msg)| {
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
                                &mut vars_shown,
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
        vars_shown: &mut BTreeSet<(ByteIndex, DebugVar)>,
        boogie_msg: &str,
    ) -> PrettyDoc {
        let mut has_vars = false;
        if let Some(func_env) = module_env.get_enclosing_function(loc) {
            let func_name = func_env.get_name().to_string();
            let mut func_display = func_name;
            if boogie_msg.ends_with("$Entry") {
                func_display = format!("{} (entry)", func_display);
                // Narrow location to the beginning of the function.
                loc = Loc::new(loc.start(), loc.start() + ByteOffset(1));
            } else if boogie_msg.ends_with("$Return") {
                func_display = format!("{} (exit)", func_display);
                // Narrow location to the end of the function.
                if loc.end().0 > 0 {
                    loc = Loc::new(loc.end() - ByteOffset(1), loc.end());
                }
            }
            // DEBUG
            // func_display = format!("{} ({})", func_display, loc);
            let model_info = if let Some(model) = model_opt {
                let displayed_vars = model
                    .relevant_vars(&func_env, loc, vars_shown)
                    .iter()
                    .map(|(var, val)| {
                        let ty = self.get_type_of_local_or_return(&func_env, var.var_idx);
                        var.pretty(self.env, &func_env, model, &ty, val)
                    })
                    .collect_vec();
                has_vars = !displayed_vars.is_empty();
                PrettyDoc::intersperse(
                    displayed_vars,
                    PrettyDoc::text(",").append(PrettyDoc::hardline()),
                )
            } else {
                PrettyDoc::nil()
            };
            if has_vars {
                PrettyDoc::text(func_display)
                    .append(PrettyDoc::hardline())
                    .append(model_info)
            } else {
                PrettyDoc::text(func_display)
            }
        } else {
            PrettyDoc::text("<unknown function>")
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
        let verification_diag_start = Regex::new(r"(?m)^.*Error BP\d+:(?P<msg>.*)$").unwrap();
        let verification_diag_cond =
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
                        model,
                    });
                }
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

/// Represents a boogie model.
#[derive(Debug)]
pub struct Model {
    vars: BTreeMap<ModelValue, ModelValue>,
    debug_vars: BTreeMap<ModuleIndex, BTreeMap<ByteIndex, Vec<(DebugVar, ModelValue)>>>,
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
                // Compute the debug variables and arrange them in a map convient for
                // lookup at a given code location.
                let mut debug_vars: BTreeMap<
                    ModuleIndex,
                    BTreeMap<ByteIndex, Vec<(DebugVar, ModelValue)>>,
                > = BTreeMap::new();
                for (var, code_index, v) in vars.iter().filter_map(|(k, v)| {
                    if let ModelValue::Literal(s) = k {
                        if let Some((var, code_index)) = DebugVar::parse(env, s) {
                            return Some((var, code_index, v));
                        }
                    }
                    None
                }) {
                    let elem = (var.clone(), v.clone());
                    if let Some(module_map) = debug_vars.get_mut(&var.module_idx) {
                        if let Some(vec) = module_map.get_mut(&code_index) {
                            vec.push(elem);
                        } else {
                            module_map.insert(code_index, vec![elem]);
                        }
                    } else {
                        let mut module_map = BTreeMap::new();
                        module_map.insert(code_index, vec![elem]);
                        debug_vars.insert(var.module_idx, module_map);
                    }
                }
                let model = Model { vars, debug_vars };
                // DEBUG
                /*
                for module_map in model.debug_vars.values() {
                    for (code_index, vars) in module_map {
                        info!("{} ->", code_index);
                        for (var, val) in vars {
                            info!("  {} = {:?}", var.var_name, val);
                        }
                    }
                }
                */
                // END DEBUG
                Ok(model)
            })
    }

    /// Computes the relevant debug variables to show at the given location `loc`.
    ///
    /// As locations reported via boogie execution traces aren't actually pointing to the
    /// place where a variable is updated (rather they often point to branch points), we apply
    /// the following heuristic: all variables are returned which have been updated from
    /// function start until `loc.end()` and which have *not* yet been shown before. We
    /// track this via the `vars_shown` set which consists of pairs of code byte index and the
    /// variable as it has been updated at this point. This heuristic works reliable for the
    /// cases seen so far, but may need further refinement.
    fn relevant_vars(
        &self,
        func_env: &FunctionEnv<'_>,
        loc: Loc,
        vars_shown: &mut BTreeSet<(ByteIndex, DebugVar)>,
    ) -> Vec<(DebugVar, ModelValue)> {
        self.debug_vars
            .get(&func_env.module_env.get_module_idx())
            .map(|vars| {
                let mut res = vec![];
                for (code_index, vars_at) in vars.range(func_env.get_loc().start()..loc.end()) {
                    for (var, val) in vars_at {
                        if vars_shown.insert((*code_index, var.clone())) {
                            res.push((var.clone(), val.clone()))
                        }
                    }
                }
                res
            })
            .unwrap_or_else(|| vec![])
        // DEBUG
        /*
        let vars = ...
        info!(
            "{} -> {:?}",
            loc,
            vars.iter().map(|(v, _)| &v.var_name).collect_vec()
        );
        vars
        */
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
                        // DEBUG .or_else(|| value_array_map.get(&ModelValue::literal("else")))
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
        let struct_env = module_env.get_struct(&struct_idx);
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

/// Represents a debug variable.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct DebugVar {
    module_idx: ModuleIndex,
    func_idx: FunctionDefinitionIndex,
    var_idx: usize,
    var_name: String,
    instance_id: usize,
}

impl DebugVar {
    /// Parses a model variable name into the components of a debug variable.
    ///
    /// Variables have the form
    /// `inline$<ignored>$<instance>$debug#<module>#<func>#<idx>#<name>#<pos>@<version>`.
    /// `instance` above appears to be for distinguishing inlined variables from the same
    /// function. `version` appears to be the number Boogie appends for update history
    /// (i.e. x@1, x@2, ...). As all our debug variables are never updated, we are only
    /// interested in version @0.
    fn parse(env: &GlobalEnv, s: &str) -> Option<(DebugVar, ByteIndex)> {
        fn get_cap_usize(cap: &Captures, name: &str) -> usize {
            cap.name(name).unwrap().as_str().parse::<usize>().unwrap()
        }
        let re = Regex::new(r"inline\$[^$]+\$(?P<instance>[0-9]+)\$debug#").unwrap();
        re.captures(s).and_then(|ref cap| {
            let instance_id = get_cap_usize(cap, "instance");
            let re = Regex::new(
                r"(?P<mod>[^#]+)#(?P<func>[^#]+)#(?P<idx>[0-9]+)#(?P<var>[^#]+)#(?P<pos>[^@]+)@0",
            )
            .unwrap();

            re.captures(&s[cap.get(0).unwrap().end()..])
                .and_then(|ref cap| {
                    let module_env = env.find_module_by_name(cap.name("mod").unwrap().as_str())?;
                    let func_env = module_env
                        .find_function(&IdentStr::new(cap.name("func").unwrap().as_str()).ok()?)?;
                    let idx = get_cap_usize(cap, "idx");
                    let var = cap.name("var").unwrap().as_str();
                    let pos = get_cap_usize(cap, "pos");
                    (Some((
                        DebugVar {
                            module_idx: module_env.get_module_idx(),
                            func_idx: func_env.get_def_idx(),
                            var_idx: idx,
                            var_name: var.to_string(),
                            instance_id,
                        },
                        ByteIndex(pos as RawIndex),
                    )))
                })
        })
    }

    /// Pretty prints a debug variable.
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
            self.var_name.clone()
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

/// Represents parser for a boogie model.
struct ModelParser<'s> {
    input: &'s str,
    at: usize,
}

/// Represents error resulting from model parsing.
struct ModelParseError(String);

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
