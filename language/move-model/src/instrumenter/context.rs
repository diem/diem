// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bytecode_source_map::source_map::SourceMap;
use move_ir_types::location::Loc;
use move_lang::{
    compiled_unit::FunctionInfo, parser::ast::FunctionName, shared::unique_map::UniqueMap,
};

/// A context structure capturing the additional information available in the CompiledUnit
/// that may have lost during the construction of the GlobalEnv.
pub struct CompilationContext {
    pub source_map: SourceMap<Loc>,
    pub function_infos: UniqueMap<FunctionName, FunctionInfo>,
}
