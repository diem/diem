// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use anyhow::Result;
use bytecode_source_map::source_map::SourceMap;
use codespan::{FileId, Files};

use crate::{
    bytecode_viewer::{BytecodeInfo, BytecodeViewer},
    interfaces::{RightScreen, SourceContext},
};
use move_binary_format::file_format::CompiledModule;
use move_ir_types::location::Loc;
use std::{cmp, fs, path::Path};

const CONTEXT_SIZE: usize = 1000;

#[derive(Debug, Clone)]
pub struct ModuleViewer {
    file_id: FileId,
    source_code: Files<String>,
    source_map: SourceMap<Loc>,
    module: CompiledModule,
}

impl ModuleViewer {
    pub fn new(module: CompiledModule, source_map: SourceMap<Loc>, source_location: &Path) -> Self {
        let mut source_code = Files::new();
        let file_contents = fs::read_to_string(source_location).unwrap();
        let file_id = source_code.add(source_location.as_os_str().to_os_string(), file_contents);

        Self {
            file_id,
            source_code,
            source_map,
            module,
        }
    }
}

impl RightScreen<BytecodeViewer> for ModuleViewer {
    fn source_for_code_location(&self, bytecode_info: &BytecodeInfo) -> Result<SourceContext> {
        let span = self
            .source_map
            .get_code_location(bytecode_info.function_index, bytecode_info.code_offset)?
            .span();

        let span_start = span.start().to_usize();
        let span_end = span.end().to_usize();
        let source = self.source_code.source(self.file_id);
        let source_span_end = self.source_code.source_span(self.file_id).end().to_usize();

        // Determine the context around the span that we want to show.
        // Show either from the start of the file, or back `CONTEXT_SIZE` number of characters.
        let context_start = span_start.saturating_sub(CONTEXT_SIZE);
        // Show either to the end of the file, or `CONTEXT_SIZE` number of characters after the
        // span end.
        let context_end = cmp::min(span_end.checked_add(CONTEXT_SIZE).unwrap(), source_span_end);

        // Create the contexts
        let left = &source[context_start..span_start];
        let highlight = self
            .source_code
            .source_slice(self.file_id, span)
            .unwrap()
            .to_string();
        let remainder = &source[span_end..context_end];

        Ok(SourceContext {
            left: left.to_string(),
            highlight,
            remainder: remainder.to_string(),
        })
    }

    fn backing_string(&self) -> String {
        self.source_code.source(self.file_id).to_string()
    }
}
