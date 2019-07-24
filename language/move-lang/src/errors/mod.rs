// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::shared::Loc;
use codespan::{ByteSpan, CodeMap, FileMap, FileName, Span};
use codespan_reporting::{
    emit,
    termcolor::{ColorChoice, StandardStream},
    Diagnostic, Label,
};
use std::collections::{HashMap, HashSet};

pub type Errors = Vec<Error>;
pub type Error = Vec<(Loc, String)>;
pub type ErrorSlice = [(Loc, String)];
pub type HashableError = Vec<(&'static str, usize, usize, String)>;

pub type Files = HashMap<&'static str, FileMap>;

pub fn report_errors(files: Files, errors: Errors) -> ! {
    assert!(!errors.is_empty());
    let mut codemap = CodeMap::new();
    let mut file_mapping = HashMap::new();
    let mut current_end = 1;
    for (fname, filemap) in files.into_iter() {
        file_mapping.insert(fname, current_end);
        let added_fmap = codemap.add_filemap(FileName::real(fname), filemap.src().to_string());
        current_end = added_fmap.span().end().to_usize() + 1;
    }
    render_errors(&codemap, file_mapping, errors);
    std::process::exit(1)
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

fn render_errors(codemap: &CodeMap, file_mapping: HashMap<&'static str, usize>, errors: Errors) {
    let mut seen: HashSet<HashableError> = HashSet::new();
    for error in errors.into_iter() {
        let hashable_error = hashable_error(&error);
        if seen.contains(&hashable_error) {
            continue;
        }
        seen.insert(hashable_error);
        let writer = StandardStream::stderr(ColorChoice::Auto);
        let err = render_error(&file_mapping, error);
        emit(writer, &codemap, &err).unwrap()
    }
}

fn convert_loc(file_mapping: &HashMap<&'static str, usize>, loc: Loc) -> ByteSpan {
    let fname = loc.file();
    let offset = *file_mapping.get(fname).unwrap();
    let begin_index = (loc.span().start().to_usize() + offset) as u32;
    let end_index = (loc.span().end().to_usize() + offset) as u32;
    Span::new(begin_index.into(), end_index.into())
}

fn render_error(file_mapping: &HashMap<&'static str, usize>, error: Error) -> Diagnostic {
    let mut diag = Diagnostic::new_error(error[0].1.clone());
    for err in error {
        let label = Label::new_primary(convert_loc(file_mapping, err.0)).with_message(err.1);
        diag = diag.with_label(label);
    }
    diag
}
