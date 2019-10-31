// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::collections::{BTreeMap, HashMap};
use vm::file_format::{
    CodeOffset, FieldDefinitionIndex, FunctionDefinitionIndex, StructDefinitionIndex, TableIndex,
};

/// A data structure used to track any markings or extra information that is desired to be exposed
/// in the disassembled function definition. Every marking can have multiple messages associated with it.
#[derive(Debug, Default)]
pub struct FunctionMarking {
    // Code offset markings
    pub code_offsets: BTreeMap<CodeOffset, Vec<String>>,

    // Type parameters markings
    pub type_param_offsets: BTreeMap<usize, Vec<String>>,
}

/// A data structure used to track any markings or extra information that is desired to be exposed
/// in the disassembled struct definition. Every marking can have multiple messages associated with it.
#[derive(Debug, Default)]
pub struct StructMarking {
    // Field markings
    pub fields: BTreeMap<FieldDefinitionIndex, Vec<String>>,

    // Type parameter markings
    pub type_param_offsets: BTreeMap<usize, Vec<String>>,
}

/// A data structure that contains markings for both functions and structs. This will be used for
/// printing out error messages and the like.
#[derive(Debug, Default)]
pub struct MarkedSourceMapping {
    // Any function markings
    function_marks: HashMap<TableIndex, FunctionMarking>,

    // Any struct marking
    struct_marks: HashMap<TableIndex, StructMarking>,
}

impl FunctionMarking {
    pub fn new() -> Self {
        Self {
            code_offsets: BTreeMap::new(),
            type_param_offsets: BTreeMap::new(),
        }
    }

    pub fn code_offset(&mut self, code_offset: CodeOffset, message: String) {
        self.code_offsets
            .entry(code_offset)
            .or_insert_with(Vec::new)
            .push(message)
    }

    pub fn type_param(&mut self, type_param_index: usize, message: String) {
        self.type_param_offsets
            .entry(type_param_index)
            .or_insert_with(Vec::new)
            .push(message)
    }
}

impl StructMarking {
    pub fn new() -> Self {
        Self {
            fields: BTreeMap::new(),
            type_param_offsets: BTreeMap::new(),
        }
    }

    pub fn field(&mut self, field_index: FieldDefinitionIndex, message: String) {
        self.fields
            .entry(field_index)
            .or_insert_with(Vec::new)
            .push(message)
    }

    pub fn type_param(&mut self, type_param_index: usize, message: String) {
        self.type_param_offsets
            .entry(type_param_index)
            .or_insert_with(Vec::new)
            .push(message)
    }
}

impl MarkedSourceMapping {
    pub fn new() -> Self {
        Self {
            function_marks: HashMap::new(),
            struct_marks: HashMap::new(),
        }
    }

    pub fn mark_code_offset(
        &mut self,
        function_definition_index: FunctionDefinitionIndex,
        code_offset: CodeOffset,
        message: String,
    ) {
        self.function_marks
            .entry(function_definition_index.0)
            .or_insert_with(FunctionMarking::new)
            .code_offset(code_offset, message)
    }

    pub fn mark_function_type_param(
        &mut self,
        function_definition_index: FunctionDefinitionIndex,
        type_param_offset: usize,
        message: String,
    ) {
        self.function_marks
            .entry(function_definition_index.0)
            .or_insert_with(FunctionMarking::new)
            .type_param(type_param_offset, message)
    }

    pub fn mark_struct_field(
        &mut self,
        struct_definition_index: StructDefinitionIndex,
        field_def_index: FieldDefinitionIndex,
        message: String,
    ) {
        self.struct_marks
            .entry(struct_definition_index.0)
            .or_insert_with(StructMarking::new)
            .field(field_def_index, message)
    }

    pub fn mark_struct_type_param(
        &mut self,
        struct_definition_index: StructDefinitionIndex,
        type_param_offset: usize,
        message: String,
    ) {
        self.struct_marks
            .entry(struct_definition_index.0)
            .or_insert_with(StructMarking::new)
            .type_param(type_param_offset, message)
    }
}
