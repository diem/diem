// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{mapping::SourceMapping, source_map::SourceMap};
use anyhow::{format_err, Result};
use codespan::{FileId, Files, Span};
use codespan_reporting::{
    diagnostic::{Diagnostic, Label},
    term::{
        emit,
        termcolor::{ColorChoice, StandardStream},
        Config,
    },
};
use move_ir_types::location::Loc;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{collections::HashMap, fs::File, io::Read, path::Path};

pub type Error = (Loc, String);
pub type Errors = Vec<Error>;

pub fn source_map_from_file<Location>(file_path: &Path) -> Result<SourceMap<Location>>
where
    Location: Clone + Eq + Default + DeserializeOwned,
{
    let mut bytes = Vec::new();
    File::open(file_path)
        .ok()
        .and_then(|mut file| file.read_to_end(&mut bytes).ok())
        .ok_or_else(|| format_err!("Error while reading in source map information"))?;
    bcs::from_bytes::<SourceMap<Location>>(&bytes)
        .map_err(|_| format_err!("Error deserializing into source map"))
}

pub fn render_errors(source_mapper: &SourceMapping<Loc>, errors: Errors) -> Result<()> {
    if let Some((source_file_name, source_string)) = &source_mapper.source_code {
        let mut codemap = Files::new();
        let id = codemap.add(source_file_name, source_string.to_string());
        for err in errors {
            let diagnostic = create_diagnostic(id, err);
            let writer = &mut StandardStream::stderr(ColorChoice::Auto);
            emit(writer, &Config::default(), &codemap, &diagnostic).unwrap();
        }
        Ok(())
    } else {
        Err(format_err!(
            "Unable to render errors since source file information is not available"
        ))
    }
}

pub fn create_diagnostic(id: FileId, (loc, msg): Error) -> Diagnostic {
    Diagnostic::new_error("", Label::new(id, loc.span(), msg))
}

//***************************************************************************
// Deserialization helper
//***************************************************************************

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct OwnedLoc {
    file: String,
    span: Span,
}

pub fn remap_owned_loc_to_loc(m: SourceMap<OwnedLoc>) -> SourceMap<Loc> {
    let mut table: HashMap<String, &'static str> = HashMap::new();
    let mut f = |owned| {
        let OwnedLoc { file, span } = owned;
        let file = *table
            .entry(file.clone())
            .or_insert_with(|| Box::leak(Box::new(file)));
        Loc::new(file, span)
    };
    m.remap_locations(&mut f)
}
