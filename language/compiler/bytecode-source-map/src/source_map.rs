// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{format_err, Result};
use libra_types::account_address::AccountAddress;
use move_core_types::identifier::Identifier;
use move_ir_types::ast::{ModuleName, QualifiedModuleIdent};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, ops::Bound};
use vm::{
    access::*,
    file_format::{
        AddressPoolIndex, CodeOffset, CompiledModule, CompiledScript, FieldDefinitionIndex,
        FunctionDefinition, FunctionDefinitionIndex, IdentifierIndex, StructDefinition,
        StructDefinitionIndex, TableIndex,
    },
    internals::ModuleIndex,
};

//***************************************************************************
// Source location mapping
//***************************************************************************

pub type SourceMap<Location> = Vec<ModuleSourceMap<Location>>;
pub type SourceName<Location> = (String, Location);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StructSourceMap<Location: Clone + Eq> {
    /// The source declaration location of the struct
    pub decl_location: Location,

    /// Important: type parameters need to be added in the order of their declaration
    pub type_parameters: Vec<SourceName<Location>>,

    /// Note that fields to a struct source map need to be added in the order of the fields in the
    /// struct definition.
    pub fields: Vec<Location>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FunctionSourceMap<Location: Clone + Eq> {
    /// The source location for the definition of this entire function. Note that in certain
    /// instances this will have no valid source location e.g. the "main" function for modules that
    /// are treated as programs are synthesized and therefore have no valid source location.
    pub decl_location: Location,

    /// Note that type parameters need to be added in the order of their declaration
    pub type_parameters: Vec<SourceName<Location>>,

    /// The index into the vector is the locals index. The corresponding `(Identifier, Location)` tuple
    /// is the name and location of the local.
    pub locals: Vec<SourceName<Location>>,

    /// The source location map for the function body.
    pub code_map: BTreeMap<CodeOffset, Location>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModuleSourceMap<Location: Clone + Eq> {
    /// The name <address.module_name> for module that this source map is for
    pub module_name: (AccountAddress, Identifier),

    // A mapping of StructDefinitionIndex to source map for each struct/resource
    struct_map: BTreeMap<TableIndex, StructSourceMap<Location>>,

    // A mapping of FunctionDefinitionIndex to the soure map for that function.
    function_map: BTreeMap<TableIndex, FunctionSourceMap<Location>>,
}

pub fn remap_locations_source_name<Location: Clone + Eq, Other: Clone + Eq>(
    (id, loc): SourceName<Location>,
    f: &mut impl FnMut(Location) -> Other,
) -> SourceName<Other> {
    (id, f(loc))
}

pub fn remap_locations_source_map<Location: Clone + Eq, Other: Clone + Eq>(
    map: SourceMap<Location>,
    f: &mut impl FnMut(Location) -> Other,
) -> SourceMap<Other> {
    map.into_iter().map(|m| m.remap_locations(f)).collect()
}

impl<Location: Clone + Eq> StructSourceMap<Location> {
    pub fn new(decl_location: Location) -> Self {
        Self {
            decl_location,
            type_parameters: Vec::new(),
            fields: Vec::new(),
        }
    }

    pub fn add_type_parameter(&mut self, type_name: SourceName<Location>) {
        self.type_parameters.push(type_name)
    }

    pub fn get_type_parameter_name(
        &self,
        type_parameter_idx: usize,
    ) -> Option<SourceName<Location>> {
        self.type_parameters.get(type_parameter_idx).cloned()
    }

    pub fn add_field_location(&mut self, field_loc: Location) {
        self.fields.push(field_loc)
    }

    pub fn get_field_location(&self, field_index: FieldDefinitionIndex) -> Option<Location> {
        self.fields.get(field_index.into_index()).cloned()
    }

    pub fn dummy_struct_map(
        &mut self,
        module: &CompiledModule,
        struct_def: &StructDefinition,
        default_loc: Location,
    ) -> Result<()> {
        let struct_handle = module.struct_handle_at(struct_def.struct_handle);

        // Add dummy locations for the fields
        match struct_def.declared_field_count() {
            Err(_) => (),
            Ok(count) => (0..count).for_each(|_| self.fields.push(default_loc.clone())),
        }

        for i in 0..struct_handle.type_formals.len() {
            let name = format!("Ty{}", i);
            self.add_type_parameter((name, default_loc.clone()))
        }
        Ok(())
    }

    pub fn remap_locations<Other: Clone + Eq>(
        self,
        f: &mut impl FnMut(Location) -> Other,
    ) -> StructSourceMap<Other> {
        let StructSourceMap {
            decl_location,
            type_parameters,
            fields,
        } = self;
        let decl_location = f(decl_location);
        let type_parameters = type_parameters
            .into_iter()
            .map(|n| remap_locations_source_name(n, f))
            .collect();
        let fields = fields.into_iter().map(|loc| f(loc)).collect();
        StructSourceMap {
            decl_location,
            type_parameters,
            fields,
        }
    }
}

impl<Location: Clone + Eq> FunctionSourceMap<Location> {
    pub fn new(decl_location: Location) -> Self {
        Self {
            decl_location,
            type_parameters: Vec::new(),
            locals: Vec::new(),
            code_map: BTreeMap::new(),
        }
    }

    pub fn add_type_parameter(&mut self, type_name: SourceName<Location>) {
        self.type_parameters.push(type_name)
    }

    pub fn get_type_parameter_name(
        &self,
        type_parameter_idx: usize,
    ) -> Option<SourceName<Location>> {
        self.type_parameters.get(type_parameter_idx).cloned()
    }

    /// A single source-level instruction may possibly map to a number of bytecode instructions. In
    /// order to not store a location for each instruction, we instead use a BTreeMap to represent
    /// a segment map (holding the left-hand-sides of each segment).  Thus, an instruction
    /// sequence is always marked from its starting point. To determine what part of the source
    /// code corresponds to a given `CodeOffset` we query to find the element that is the largest
    /// number less than or equal to the query. This will give us the location for that bytecode
    /// range.
    pub fn add_code_mapping(&mut self, start_offset: CodeOffset, location: Location) {
        let possible_segment = self.get_code_location(start_offset);
        match possible_segment.map(|other_location| other_location != location) {
            Some(true) | None => {
                self.code_map.insert(start_offset, location);
            }
            _ => (),
        };
    }

    // Not that it is important that locations be added in order.
    pub fn add_local_mapping(&mut self, name: SourceName<Location>) {
        self.locals.push(name);
    }

    /// Recall that we are using a segment tree. We therefore lookup the location for the code
    /// offset by performing a range query for the largest number less than or equal to the code
    /// offset passed in.
    pub fn get_code_location(&self, code_offset: CodeOffset) -> Option<Location> {
        self.code_map
            .range((Bound::Unbounded, Bound::Included(&code_offset)))
            .next_back()
            .map(|(_, vl)| vl.clone())
    }

    pub fn get_local_name(&self, local_index: u64) -> Option<SourceName<Location>> {
        self.locals.get(local_index as usize).cloned()
    }

    pub fn dummy_function_map(
        &mut self,
        module: &CompiledModule,
        function_def: &FunctionDefinition,
        default_loc: Location,
    ) -> Result<()> {
        let function_handle = module.function_handle_at(function_def.function);
        let function_signature = module.function_signature_at(function_handle.signature);
        let function_code = &function_def.code;

        // Generate names for each type parameter
        for i in 0..function_signature.type_formals.len() {
            let name = format!("Ty{}", i);
            self.add_type_parameter((name, default_loc.clone()))
        }

        if !function_def.is_native() {
            let locals = module.locals_signature_at(function_code.locals);
            for i in 0..locals.0.len() {
                let name = format!("loc{}", i);
                self.add_local_mapping((name, default_loc.clone()))
            }
        }

        // We just need to insert the code map at the 0'th index since we represent this with a
        // segment map
        self.add_code_mapping(0, default_loc);

        Ok(())
    }

    pub fn remap_locations<Other: Clone + Eq>(
        self,
        f: &mut impl FnMut(Location) -> Other,
    ) -> FunctionSourceMap<Other> {
        let FunctionSourceMap {
            decl_location,
            type_parameters,
            locals,
            code_map,
        } = self;
        let decl_location = f(decl_location);
        let type_parameters = type_parameters
            .into_iter()
            .map(|n| remap_locations_source_name(n, f))
            .collect();
        let locals = locals
            .into_iter()
            .map(|n| remap_locations_source_name(n, f))
            .collect();
        let code_map = code_map.into_iter().map(|(i, loc)| (i, f(loc))).collect();
        FunctionSourceMap {
            decl_location,
            type_parameters,
            locals,
            code_map,
        }
    }
}

impl<Location: Clone + Eq> ModuleSourceMap<Location> {
    pub fn new(module_name: QualifiedModuleIdent) -> Self {
        let ident = Identifier::new(module_name.name.into_inner()).unwrap();
        Self {
            module_name: (module_name.address, ident),
            struct_map: BTreeMap::new(),
            function_map: BTreeMap::new(),
        }
    }

    pub fn add_top_level_function_mapping(
        &mut self,
        fdef_idx: FunctionDefinitionIndex,
        location: Location,
    ) -> Result<()> {
        self.function_map.insert(fdef_idx.0, FunctionSourceMap::new(location)).map_or(Ok(()), |_| { Err(format_err!(
                    "Multiple functions at same function definition index encountered when constructing source map"
                )) })
    }

    pub fn add_function_type_parameter_mapping(
        &mut self,
        fdef_idx: FunctionDefinitionIndex,
        name: SourceName<Location>,
    ) -> Result<()> {
        let func_entry = self.function_map.get_mut(&fdef_idx.0).ok_or_else(|| {
            format_err!("Tried to add function type parameter mapping to undefined function index")
        })?;
        func_entry.add_type_parameter(name);
        Ok(())
    }

    pub fn get_function_type_parameter_name(
        &self,
        fdef_idx: FunctionDefinitionIndex,
        type_parameter_idx: usize,
    ) -> Result<SourceName<Location>> {
        self.function_map
            .get(&fdef_idx.0)
            .and_then(|function_source_map| {
                function_source_map.get_type_parameter_name(type_parameter_idx)
            })
            .ok_or_else(|| format_err!("Unable to get function type parameter name"))
    }

    pub fn add_code_mapping(
        &mut self,
        fdef_idx: FunctionDefinitionIndex,
        start_offset: CodeOffset,
        location: Location,
    ) -> Result<()> {
        let func_entry = self
            .function_map
            .get_mut(&fdef_idx.0)
            .ok_or_else(|| format_err!("Tried to add code mapping to undefined function index"))?;
        func_entry.add_code_mapping(start_offset, location);
        Ok(())
    }

    /// Given a function definition and a code offset within that function definition, this returns
    /// the location in the source code associated with the instruction at that offset.
    pub fn get_code_location(
        &self,
        fdef_idx: FunctionDefinitionIndex,
        offset: CodeOffset,
    ) -> Result<Location> {
        self.function_map
            .get(&fdef_idx.0)
            .and_then(|function_source_map| function_source_map.get_code_location(offset))
            .ok_or_else(|| format_err!("Tried to get code location from undefined function index"))
    }

    pub fn add_local_mapping(
        &mut self,
        fdef_idx: FunctionDefinitionIndex,
        name: SourceName<Location>,
    ) -> Result<()> {
        let func_entry = self
            .function_map
            .get_mut(&fdef_idx.0)
            .ok_or_else(|| format_err!("Tried to add local mapping to undefined function index"))?;
        func_entry.add_local_mapping(name);
        Ok(())
    }

    pub fn get_local_name(
        &self,
        fdef_idx: FunctionDefinitionIndex,
        index: u64,
    ) -> Result<SourceName<Location>> {
        self.function_map
            .get(&fdef_idx.0)
            .and_then(|function_source_map| function_source_map.get_local_name(index))
            .ok_or_else(|| format_err!("Tried to get local name at undefined function index"))
    }

    pub fn add_top_level_struct_mapping(
        &mut self,
        struct_def_idx: StructDefinitionIndex,
        location: Location,
    ) -> Result<()> {
        self.struct_map.insert(struct_def_idx.0, StructSourceMap::new(location)).map_or(Ok(()), |_| { Err(format_err!(
                "Multiple structs at same struct definition index encountered when constructing source map"
                )) })
    }

    pub fn add_struct_field_mapping(
        &mut self,
        struct_def_idx: StructDefinitionIndex,
        location: Location,
    ) -> Result<()> {
        let struct_entry = self
            .struct_map
            .get_mut(&struct_def_idx.0)
            .ok_or_else(|| format_err!("Tried to add file mapping to undefined struct index"))?;
        struct_entry.add_field_location(location);
        Ok(())
    }

    pub fn get_struct_field_name(
        &self,
        struct_def_idx: StructDefinitionIndex,
        field_idx: FieldDefinitionIndex,
    ) -> Option<Location> {
        self.struct_map
            .get(&struct_def_idx.0)
            .and_then(|struct_source_map| struct_source_map.get_field_location(field_idx))
    }

    pub fn add_struct_type_parameter_mapping(
        &mut self,
        struct_def_idx: StructDefinitionIndex,
        name: SourceName<Location>,
    ) -> Result<()> {
        let struct_entry = self.struct_map.get_mut(&struct_def_idx.0).ok_or_else(|| {
            format_err!("Tried to add struct type parameter mapping to undefined struct index")
        })?;
        struct_entry.add_type_parameter(name);
        Ok(())
    }

    pub fn get_struct_type_parameter_name(
        &self,
        struct_def_idx: StructDefinitionIndex,
        type_parameter_idx: usize,
    ) -> Result<SourceName<Location>> {
        self.struct_map
            .get(&struct_def_idx.0)
            .and_then(|struct_source_map| {
                struct_source_map.get_type_parameter_name(type_parameter_idx)
            })
            .ok_or_else(|| format_err!("Unable to get struct type parameter name"))
    }

    pub fn get_function_source_map(
        &self,
        fdef_idx: FunctionDefinitionIndex,
    ) -> Result<&FunctionSourceMap<Location>> {
        self.function_map
            .get(&fdef_idx.0)
            .ok_or_else(|| format_err!("Unable to get function source map"))
    }

    pub fn get_struct_source_map(
        &self,
        struct_def_idx: StructDefinitionIndex,
    ) -> Result<&StructSourceMap<Location>> {
        self.struct_map
            .get(&struct_def_idx.0)
            .ok_or_else(|| format_err!("Unable to get struct source map"))
    }

    /// Create a 'dummy' source map for a compiled module. This is useful for e.g. disassembling
    /// with generated or real names depending upon if the source map is available or not.
    pub fn dummy_from_module(module: &CompiledModule, default_loc: Location) -> Result<Self> {
        let module_name =
            ModuleName::new(module.identifier_at(IdentifierIndex::new(0)).to_string());
        let module_ident =
            QualifiedModuleIdent::new(module_name, *module.address_at(AddressPoolIndex::new(0)));

        let mut empty_source_map = Self::new(module_ident);

        for (function_idx, function_def) in module.function_defs().iter().enumerate() {
            empty_source_map.add_top_level_function_mapping(
                FunctionDefinitionIndex(function_idx as TableIndex),
                default_loc.clone(),
            )?;
            empty_source_map
                .function_map
                .get_mut(&(function_idx as TableIndex))
                .ok_or_else(|| format_err!("Unable to get function map while generating dummy"))?
                .dummy_function_map(&module, &function_def, default_loc.clone())?;
        }

        for (struct_idx, struct_def) in module.struct_defs().iter().enumerate() {
            empty_source_map.add_top_level_struct_mapping(
                StructDefinitionIndex(struct_idx as TableIndex),
                default_loc.clone(),
            )?;
            empty_source_map
                .struct_map
                .get_mut(&(struct_idx as TableIndex))
                .ok_or_else(|| format_err!("Unable to get struct map while generating dummy"))?
                .dummy_struct_map(&module, &struct_def, default_loc.clone())?;
        }

        Ok(empty_source_map)
    }

    pub fn dummy_from_script(script: &CompiledScript, default_loc: Location) -> Result<Self> {
        Self::dummy_from_module(&script.clone().into_module(), default_loc)
    }

    pub fn remap_locations<Other: Clone + Eq>(
        self,
        f: &mut impl FnMut(Location) -> Other,
    ) -> ModuleSourceMap<Other> {
        let ModuleSourceMap {
            module_name,
            struct_map,
            function_map,
        } = self;
        let struct_map = struct_map
            .into_iter()
            .map(|(n, m)| (n, m.remap_locations(f)))
            .collect();
        let function_map = function_map
            .into_iter()
            .map(|(n, m)| (n, m.remap_locations(f)))
            .collect();
        ModuleSourceMap {
            module_name,
            struct_map,
            function_map,
        }
    }
}
