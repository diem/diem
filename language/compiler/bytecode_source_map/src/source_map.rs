// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use codespan::Span;
use ir_to_bytecode_syntax::ast::{Loc, QualifiedModuleIdent, Var_};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::ops::Bound;
use types::account_address::AccountAddress;
use types::identifier::Identifier;
use vm::file_format::{
    CodeOffset, FieldDefinitionIndex, FunctionDefinitionIndex, StructDefinitionIndex, TableIndex,
};
use vm::internals::ModuleIndex;

//***************************************************************************
// Source location mapping
//***************************************************************************

pub type SourceMap = Vec<ModuleSourceMap>;
pub type SourceName = (Identifier, Loc);

#[derive(Debug, Serialize, Deserialize)]
pub struct StructSourceMap {
    // NB type parameters need to be added in the order of their declaration
    type_parameters: Vec<SourceName>,

    // NB that fields to a struct source map need to be added in the order of the fields.
    fields: Vec<SourceName>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FunctionSourceMap {
    // NB type parameters need to be added in the order of their declaration
    type_parameters: Vec<SourceName>,

    // The index into the vector is the locals index. The corresponding `(Identifier, Loc)` tuple
    // is the name and location of the local.
    locals: Vec<SourceName>,

    // The source location map for the function body.
    code_map: BTreeMap<CodeOffset, Loc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModuleSourceMap {
    // The name <address.module_name> for module that this source map is for
    module_name: (AccountAddress, Identifier),

    // A mapping of StructDefinitionIndex to source map for each struct/resource
    struct_map: HashMap<TableIndex, StructSourceMap>,

    // A mapping of FunctionDefinitionIndex to the soure map for that function.
    function_map: HashMap<TableIndex, FunctionSourceMap>,
}

impl StructSourceMap {
    pub fn new() -> Self {
        Self {
            type_parameters: Vec::new(),
            fields: Vec::new(),
        }
    }

    pub fn add_type_parameter(&mut self, type_name: SourceName) {
        self.type_parameters.push(type_name)
    }

    pub fn get_type_parameter_name(&self, type_parameter_idx: usize) -> Option<SourceName> {
        self.type_parameters.get(type_parameter_idx).cloned()
    }

    pub fn add_field(&mut self, field_name: SourceName) {
        self.fields.push(field_name)
    }

    pub fn get_field_name(&self, field_index: FieldDefinitionIndex) -> Option<SourceName> {
        self.fields.get(field_index.into_index()).cloned()
    }
}

impl FunctionSourceMap {
    pub fn new() -> Self {
        Self {
            type_parameters: Vec::new(),
            locals: Vec::new(),
            code_map: BTreeMap::new(),
        }
    }

    pub fn add_type_parameter(&mut self, type_name: SourceName) {
        self.type_parameters.push(type_name)
    }

    pub fn get_type_parameter_name(&self, type_parameter_idx: usize) -> Option<SourceName> {
        self.type_parameters.get(type_parameter_idx).cloned()
    }

    /// A single source-level instruction may possibly map to a number of bytecode instructions. In
    /// order to not store a location for each instruction, we instead use a BTreeMap to represent
    /// a segement map (holding the left-hand-sands of the segments).  Thus, an instruction
    /// sequence is always marked from the starting point. To determine what part of the source
    /// code corresponds to a given `CodeOffset` we query to find the element that is the largest
    /// number less than or equal to the query. This will give us the location for that bytcode
    /// range.
    pub fn add_code_mapping(&mut self, start_offset: CodeOffset, location: Loc) {
        self.code_map.insert(start_offset, location);
    }

    // NB that locations must be added in order.
    pub fn add_local_mapping(&mut self, name: Var_) {
        let loc = name.span;
        let name = Identifier::from(name.value.name());
        self.locals.push((name, loc));
    }

    /// Recall that we are using a segment tree. We therefore lookup the location for the code
    /// offset by performing a range query for the largest number less than or equal to the code
    /// offset passed in.
    pub fn get_code_location(&self, code_offset: CodeOffset) -> Option<Loc> {
        self.code_map
            .range((Bound::Unbounded, Bound::Included(&code_offset)))
            .next_back()
            .and_then(|(_, vl)| Some(Span::from(vl.clone())))
    }

    pub fn get_local_name(&self, local_index: u64) -> Option<SourceName> {
        self.locals.get(local_index as usize).cloned()
    }
}

impl ModuleSourceMap {
    pub fn new(module_name: QualifiedModuleIdent) -> Self {
        Self {
            module_name: (module_name.address, module_name.name.into_inner()),
            struct_map: HashMap::new(),
            function_map: HashMap::new(),
        }
    }

    pub fn add_function_type_parameter_mapping(
        &mut self,
        fdef_idx: FunctionDefinitionIndex,
        name: SourceName,
    ) {
        let func_entry = self
            .function_map
            .entry(fdef_idx.0)
            .or_insert_with(|| FunctionSourceMap::new());
        func_entry.add_type_parameter(name);
    }

    pub fn get_function_type_parameter_name(
        &self,
        fdef_idx: FunctionDefinitionIndex,
        type_parameter_idx: usize,
    ) -> Option<SourceName> {
        self.function_map
            .get(&fdef_idx.0)
            .and_then(|function_source_map| {
                function_source_map.get_type_parameter_name(type_parameter_idx)
            })
    }

    pub fn add_code_mapping(
        &mut self,
        function_definition_index: FunctionDefinitionIndex,
        start_offset: CodeOffset,
        location: Loc,
    ) {
        let func_entry = self
            .function_map
            .entry(function_definition_index.0)
            .or_insert_with(|| FunctionSourceMap::new());
        func_entry.add_code_mapping(start_offset, location);
    }

    /// Given a function definition and a code offset within that function definition, this returns
    /// the location in the source code associated with the instruction at that offset.
    pub fn get_code_location(
        &self,
        function_definition_index: &FunctionDefinitionIndex,
        offset: CodeOffset,
    ) -> Option<Loc> {
        self.function_map
            .get(&function_definition_index.0)
            .and_then(|function_source_map| function_source_map.get_code_location(offset))
    }

    pub fn add_local_mapping(&mut self, fdef_idx: FunctionDefinitionIndex, name: Var_) {
        let func_entry = self
            .function_map
            .entry(fdef_idx.0)
            .or_insert_with(|| FunctionSourceMap::new());
        func_entry.add_local_mapping(name);
    }

    pub fn get_local_name(
        &self,
        fdef_idx: FunctionDefinitionIndex,
        index: u64,
    ) -> Option<SourceName> {
        self.function_map
            .get(&fdef_idx.0)
            .and_then(|function_source_map| function_source_map.get_local_name(index))
    }

    pub fn add_struct_field_mapping(
        &mut self,
        struct_def_idx: StructDefinitionIndex,
        name: SourceName,
    ) {
        let struct_entry = self
            .struct_map
            .entry(struct_def_idx.0)
            .or_insert_with(|| StructSourceMap::new());
        struct_entry.add_field(name);
    }

    pub fn get_struct_field_name(
        &self,
        struct_def_idx: StructDefinitionIndex,
        field_idx: FieldDefinitionIndex,
    ) -> Option<SourceName> {
        self.struct_map
            .get(&struct_def_idx.0)
            .and_then(|struct_source_map| struct_source_map.get_field_name(field_idx))
    }

    pub fn add_struct_type_parameter_mapping(
        &mut self,
        fdef_idx: StructDefinitionIndex,
        name: SourceName,
    ) {
        let struct_entry = self
            .struct_map
            .entry(fdef_idx.0)
            .or_insert_with(|| StructSourceMap::new());
        struct_entry.add_type_parameter(name);
    }

    pub fn get_struct_type_parameter_name(
        &self,
        fdef_idx: StructDefinitionIndex,
        type_parameter_idx: usize,
    ) -> Option<SourceName> {
        self.struct_map
            .get(&fdef_idx.0)
            .and_then(|struct_source_map| {
                struct_source_map.get_type_parameter_name(type_parameter_idx)
            })
    }
}
