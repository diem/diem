// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::mapping::SourceMapping;
use crate::source_map::{ModuleSourceMap, SourceMap};
use anyhow::{format_err, Result};
use codespan::{CodeMap, FileName};
use codespan_reporting::{
    emit,
    termcolor::{ColorChoice, StandardStream},
    Diagnostic, Label,
};
use ir_to_bytecode_syntax::ast::Loc;
use serde::de::DeserializeOwned;
use std::fs::File;
use std::path::Path;

pub type Error = (Loc, String);
pub type Errors = Vec<Error>;

pub fn module_source_map_from_file<Location>(file_path: &Path) -> Result<ModuleSourceMap<Location>>
where
    Location: Clone + Eq + Default + DeserializeOwned,
{
    File::open(file_path)
        .ok()
        .and_then(|file| serde_json::from_reader(file).ok())
        .ok_or_else(|| format_err!("Error while reading in source map information"))
}

pub fn source_map_from_file<Location>(file_path: &Path) -> Result<SourceMap<Location>>
where
    Location: Clone + Eq + Default + DeserializeOwned,
{
    File::open(file_path)
        .ok()
        .and_then(|file| serde_json::from_reader(file).ok())
        .ok_or_else(|| format_err!("Error while reading in source map information"))
}

pub fn render_errors(source_mapper: &SourceMapping<Loc>, errors: Errors) -> Result<()> {
    if let Some((source_file_name, source_string)) = &source_mapper.source_code {
        let mut codemap = CodeMap::new();
        codemap.add_filemap(FileName::real(source_file_name), source_string.to_string());
        for err in errors {
            let diagnostic = create_diagnostic(err);
            let writer = StandardStream::stderr(ColorChoice::Auto);
            emit(writer, &codemap, &diagnostic).unwrap();
        }
        Ok(())
    } else {
        Err(format_err!(
            "Unable to render errors since source file information is not available"
        ))
    }
}

pub fn create_diagnostic(error: Error) -> Diagnostic {
    let label = Label::new_primary(error.0);
    Diagnostic::new_error(error.1).with_label(label)
}
