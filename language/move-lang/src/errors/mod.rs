// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::command_line::COLOR_MODE_ENV_VAR;
use codespan::{FileId, Files, Span};
use codespan_reporting::{
    diagnostic::{Diagnostic, Label},
    term::{
        emit,
        termcolor::{Buffer, ColorChoice, StandardStream, WriteColor},
        Config,
    },
};
use move_command_line_common::env::read_env_var;
use move_ir_types::location::*;
use move_symbol_pool::Symbol;
use std::{
    collections::{HashMap, HashSet},
    iter::FromIterator,
};

pub mod diagnostic_codes;
pub mod new;

//**************************************************************************************************
// Types
//**************************************************************************************************

pub use new::FilesSourceText;

#[derive(PartialEq, Eq, Clone, Debug, Default)]
pub struct Errors {
    old_fmt: Vec<Error>,
    new_fmt: new::Diagnostics,
}
pub type Error = Vec<(Loc, String)>;
pub type HashableError = Vec<(String, usize, usize, String)>;

type FileMapping = HashMap<Symbol, FileId>;

//**************************************************************************************************
// Reporting
//**************************************************************************************************

pub fn report_errors(files: FilesSourceText, errors: Errors) -> ! {
    let color_choice = match read_env_var(COLOR_MODE_ENV_VAR).as_str() {
        "NONE" => ColorChoice::Never,
        "ANSI" => ColorChoice::AlwaysAnsi,
        "ALWAYS" => ColorChoice::Always,
        _ => ColorChoice::Auto,
    };
    let mut writer = StandardStream::stderr(color_choice);
    output_errors(&mut writer, files, errors);
    std::process::exit(1)
}

pub fn report_errors_to_buffer(files: FilesSourceText, errors: Errors) -> Vec<u8> {
    let mut writer = Buffer::no_color();
    output_errors(&mut writer, files, errors);
    writer.into_inner()
}

pub fn report_errors_to_color_buffer(files: FilesSourceText, errors: Errors) -> Vec<u8> {
    let mut writer = Buffer::ansi();
    output_errors(&mut writer, files, errors);
    writer.into_inner()
}

fn output_errors<W: WriteColor>(writer: &mut W, sources: FilesSourceText, errors: Errors) {
    let Errors { old_fmt, new_fmt } = errors;
    assert!(!old_fmt.is_empty() || !new_fmt.is_empty());
    if !new_fmt.is_empty() {
        new::output_diagnostics(writer, &sources, new_fmt)
    }
    if !old_fmt.is_empty() {
        let mut files = Files::new();
        let mut file_mapping = HashMap::new();
        for (fname, source) in sources {
            let id = files.add(fname.as_str(), source);
            file_mapping.insert(fname, id);
        }
        render_errors(writer, &files, &file_mapping, old_fmt);
    }
}

fn hashable_error(error: &[(Loc, String)]) -> HashableError {
    error
        .iter()
        .map(|(loc, e)| {
            (
                loc.file().to_string(),
                loc.span().start().to_usize(),
                loc.span().end().to_usize(),
                e.clone(),
            )
        })
        .collect()
}

fn render_errors<W: WriteColor>(
    writer: &mut W,
    files: &Files<String>,
    file_mapping: &FileMapping,
    mut errors: Vec<Error>,
) {
    errors.sort_by(|e1, e2| {
        let loc1: &Loc = &e1[0].0;
        let loc2: &Loc = &e2[0].0;
        loc1.cmp(loc2)
    });
    let mut seen: HashSet<HashableError> = HashSet::new();
    for error in errors {
        let hashable_error = hashable_error(&error);
        if seen.contains(&hashable_error) {
            continue;
        }
        seen.insert(hashable_error);
        let err = render_error(files, file_mapping, error);
        emit(writer, &Config::default(), &files, &err).unwrap()
    }
}

fn convert_loc(files: &Files<String>, file_mapping: &FileMapping, loc: Loc) -> (FileId, Span) {
    let id = *file_mapping.get(&loc.file()).unwrap();
    let offset = files.source_span(id).start().to_usize();
    let begin_index = (loc.span().start().to_usize() + offset) as u32;
    let end_index = (loc.span().end().to_usize() + offset) as u32;
    (id, Span::new(begin_index, end_index))
}

fn render_error(files: &Files<String>, file_mapping: &FileMapping, mut error: Error) -> Diagnostic {
    let mk_lbl = |err: (Loc, String)| -> Label {
        let (id, span) = convert_loc(files, file_mapping, err.0);
        Label::new(id, span, err.1)
    };
    let err = error.remove(0);
    // TODO message with each error msg
    let mut diag = Diagnostic::new_error("", mk_lbl(err));
    diag = diag.with_secondary_labels(error.into_iter().map(mk_lbl));
    diag
}

//**************************************************************************************************
// impls
//**************************************************************************************************

impl Errors {
    pub fn new() -> Self {
        Self {
            old_fmt: vec![],
            new_fmt: new::Diagnostics::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.old_fmt.is_empty() && self.new_fmt.is_empty()
    }

    pub fn len(&self) -> usize {
        self.old_fmt.len() + self.new_fmt.len()
    }

    pub fn from_deprecated_fmt(old_fmt: Vec<Error>) -> Self {
        Self {
            old_fmt,
            new_fmt: new::Diagnostics::new(),
        }
    }

    pub fn add_deprecated(&mut self, error: Error) {
        self.old_fmt.push(error)
    }

    pub fn add(&mut self, error: new::Diagnostic) {
        self.new_fmt.add(error)
    }

    pub fn extend(&mut self, diags: new::Diagnostics) {
        self.new_fmt.extend(diags)
    }

    pub fn extend_deprecated(&mut self, errors: Self) {
        self.old_fmt.extend(errors.old_fmt);
        self.new_fmt.extend(errors.new_fmt)
    }

    #[deprecated]
    pub fn into_vec(self) -> Vec<Vec<(Loc, String)>> {
        let Self { old_fmt, new_fmt } = self;
        let mut v = old_fmt;

        for diag in new_fmt.diagnostics {
            let new::Diagnostic {
                info: _,
                primary_label,
                secondary_labels,
            } = diag;
            let mut inner_v = vec![primary_label];
            inner_v.extend(secondary_labels);
            v.push(inner_v)
        }
        v
    }
}

impl FromIterator<new::Diagnostic> for Errors {
    fn from_iter<I: IntoIterator<Item = new::Diagnostic>>(items: I) -> Self {
        Self {
            old_fmt: vec![],
            new_fmt: new::Diagnostics::from_iter(items),
        }
    }
}

impl FromIterator<Error> for Errors {
    fn from_iter<I: IntoIterator<Item = Error>>(items: I) -> Self {
        Self {
            old_fmt: items.into_iter().collect(),
            new_fmt: new::Diagnostics::new(),
        }
    }
}

impl From<Vec<Error>> for Errors {
    fn from(v: Vec<Error>) -> Self {
        Self {
            old_fmt: v,
            new_fmt: new::Diagnostics::new(),
        }
    }
}

impl From<Vec<new::Diagnostic>> for Errors {
    fn from(v: Vec<new::Diagnostic>) -> Self {
        Self {
            old_fmt: vec![],
            new_fmt: new::Diagnostics::from(v),
        }
    }
}

impl From<new::Diagnostics> for Errors {
    fn from(new_fmt: new::Diagnostics) -> Self {
        Self {
            old_fmt: vec![],
            new_fmt,
        }
    }
}
