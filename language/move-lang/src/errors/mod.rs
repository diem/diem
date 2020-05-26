// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use codespan::{FileId, Files, Span};
use codespan_reporting::{
    diagnostic::{Diagnostic, Label},
    term::{
        emit,
        termcolor::{Buffer, ColorChoice, StandardStream, WriteColor},
        Config,
    },
};
use move_core_types::fs::FileName;
use move_ir_types::location::*;
use std::collections::{HashMap, HashSet};

//**************************************************************************************************
// Types
//**************************************************************************************************

pub type Errors = Vec<Error>;
pub type Error = Vec<(Loc, String)>;
pub type ErrorSlice = [(Loc, String)];
pub type HashableError = Vec<(FileName, usize, usize, String)>;

pub type FilesSourceText = HashMap<FileName, String>;

type FileMapping = HashMap<FileName, FileId>;

//**************************************************************************************************
// Utils
//**************************************************************************************************

pub fn check_errors(errors: Errors) -> Result<(), Errors> {
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

//**************************************************************************************************
// Reporting
//**************************************************************************************************

pub fn report_errors(files: FilesSourceText, errors: Errors) -> ! {
    let mut writer = StandardStream::stderr(ColorChoice::Auto);
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
    assert!(!errors.is_empty());
    let mut files = Files::new();
    let mut file_mapping = HashMap::new();
    for (fname, source) in sources.into_iter() {
        let id = files.add(fname, source);
        file_mapping.insert(fname, id);
    }
    render_errors(writer, &files, &file_mapping, errors);
}

fn hashable_error(error: &ErrorSlice) -> HashableError {
    error
        .iter()
        .map(|(loc, e)| {
            (
                loc.file(),
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
    mut errors: Errors,
) {
    errors.sort_by(|e1, e2| {
        let loc1: &Loc = &e1[0].0;
        let loc2: &Loc = &e2[0].0;
        loc1.cmp(loc2)
    });
    let mut seen: HashSet<HashableError> = HashSet::new();
    for error in errors.into_iter() {
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
    let fname = loc.file();
    let id = *file_mapping.get(&fname).unwrap();
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
