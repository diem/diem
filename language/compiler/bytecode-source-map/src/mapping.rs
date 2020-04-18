// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{marking::MarkedSourceMapping, source_map::SourceMap};
use vm::file_format::{CompiledModule, CompiledScript};

/// An object that associates source code with compiled bytecode and source map.
#[derive(Debug)]
pub struct SourceMapping<Location: Clone + Eq> {
    // The resulting bytecode from compiling the source map
    pub bytecode: CompiledModule,

    // The source map for the bytecode made w.r.t. to the `source_code`
    pub source_map: SourceMap<Location>,

    // The source code for the bytecode. This is not required for disassembly, but it is required
    // for being able to print out corresponding source code for marked functions and structs.
    // Unused for now, this will be used when we start printing function/struct markings
    pub source_code: Option<(String, String)>,

    // Function and struct markings. These are used to lift up annotations/messages on the bytecode
    // into the disassembled program and/or IR source code.
    pub marks: Option<MarkedSourceMapping>,
}

impl<Location: Clone + Eq> SourceMapping<Location> {
    pub fn new(source_map: SourceMap<Location>, bytecode: CompiledModule) -> Self {
        Self {
            source_map,
            bytecode,
            source_code: None,
            marks: None,
        }
    }

    pub fn new_from_script(source_map: SourceMap<Location>, bytecode: CompiledScript) -> Self {
        Self::new(source_map, bytecode.into_module().1)
    }

    pub fn with_marks(&mut self, marks: MarkedSourceMapping) {
        self.marks = Some(marks);
    }

    pub fn with_source_code(&mut self, source_code: (String, String)) {
        self.source_code = Some(source_code);
    }
}
