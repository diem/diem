// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::source_map::ModuleSourceMap;
use vm::file_format::{CompiledModule, CompiledScript};

// An object that associates source code with compiled bytecode and source map.
struct SourceMapping<T> {
    source_code: String,
    source_map: ModuleSourceMap,
    bytecode: T,
}

pub type MappedModule = SourceMapping<CompiledModule>;
pub type MappedScript = SourceMapping<CompiledScript>;
// TODO: Add source mapping for compiled programs
//pub type MappedProgram = SourceMaping<CompiledProgram>;

impl<T> SourceMapping<T> {
    pub fn new(source_code: String, source_map: ModuleSourceMap, bytecode: T) -> Self {
        Self {
            source_code,
            source_map,
            bytecode,
        }
    }
}
