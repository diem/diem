// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, format_err, Result};
use bytecode_source_map::{
    mapping::SourceMapping,
    source_map::{FunctionSourceMap, SourceName},
};
use bytecode_verifier::control_flow_graph::{ControlFlowGraph, VMControlFlowGraph};
use colored::*;
use move_core_types::identifier::IdentStr;
use move_coverage::coverage_map::{CoverageMap, FunctionCoverage};
use vm::{
    access::ModuleAccess,
    file_format::{
        Bytecode, FieldHandleIndex, FunctionDefinition, FunctionDefinitionIndex, Kind, Signature,
        SignatureIndex, SignatureToken, StructDefinition, StructDefinitionIndex,
        StructFieldInformation, TableIndex, TypeSignature,
    },
};

/// Holds the various options that we support while disassembling code.
#[derive(Debug, Default)]
pub struct DisassemblerOptions {
    /// Only print public functions
    pub only_public: bool,

    /// Print the bytecode for the instructions within the function.
    pub print_code: bool,

    /// Print the basic blocks of the bytecode.
    pub print_basic_blocks: bool,

    /// Print the locals inside each function body.
    pub print_locals: bool,
}

impl DisassemblerOptions {
    pub fn new() -> Self {
        Self {
            only_public: false,
            print_code: false,
            print_basic_blocks: false,
            print_locals: false,
        }
    }
}

pub struct Disassembler<Location: Clone + Eq> {
    source_mapper: SourceMapping<Location>,
    // The various options that we can set for disassembly.
    options: DisassemblerOptions,
    // Optional coverage map for use in displaying code coverage
    coverage_map: Option<CoverageMap>,
}

impl<Location: Clone + Eq> Disassembler<Location> {
    pub fn new(source_mapper: SourceMapping<Location>, options: DisassemblerOptions) -> Self {
        Self {
            source_mapper,
            options,
            coverage_map: None,
        }
    }

    pub fn add_coverage_map(&mut self, coverage_map: CoverageMap) {
        self.coverage_map = Some(coverage_map);
    }

    //***************************************************************************
    // Helpers
    //***************************************************************************

    fn get_function_def(
        &self,
        function_definition_index: FunctionDefinitionIndex,
    ) -> Result<&FunctionDefinition> {
        if function_definition_index.0 as usize >= self.source_mapper.bytecode.function_defs().len()
        {
            bail!("Invalid function definition index supplied when marking function")
        }
        Ok(self
            .source_mapper
            .bytecode
            .function_def_at(function_definition_index))
    }

    fn get_struct_def(
        &self,
        struct_definition_index: StructDefinitionIndex,
    ) -> Result<&StructDefinition> {
        if struct_definition_index.0 as usize >= self.source_mapper.bytecode.struct_defs().len() {
            bail!("Invalid struct definition index supplied when marking struct")
        }
        Ok(self
            .source_mapper
            .bytecode
            .struct_def_at(struct_definition_index))
    }

    //***************************************************************************
    // Code Coverage Helpers
    //***************************************************************************

    fn get_function_coverage(&self, function_name: &IdentStr) -> Option<&FunctionCoverage> {
        self.source_mapper
            .source_map
            .module_name_opt
            .as_ref()
            .and_then(|module| {
                self.coverage_map.as_ref().and_then(|coverage_map| {
                    coverage_map
                        .module_maps
                        .get(&module)
                        .and_then(|module_map| module_map.get_function_coverage(function_name))
                })
            })
    }

    fn is_function_called(&self, function_name: &IdentStr) -> bool {
        self.get_function_coverage(function_name).is_some()
    }

    fn format_function_coverage(&self, name: &IdentStr, function_body: String) -> String {
        if self.is_function_called(name) {
            function_body.green()
        } else {
            function_body.red()
        }
        .to_string()
    }

    fn format_with_instruction_coverage(
        pc: usize,
        function_coverage_map: Option<&FunctionCoverage>,
        instruction: String,
    ) -> String {
        let coverage = function_coverage_map.and_then(|map| map.get(&(pc as u64)));
        match coverage {
            Some(coverage) => format!("[{}]\t{}: {}", coverage, pc, instruction).green(),
            None => format!("\t{}: {}", pc, instruction).red(),
        }
        .to_string()
    }

    //***************************************************************************
    // Formatting Helpers
    //***************************************************************************

    fn name_for_field(&self, field_idx: FieldHandleIndex) -> Result<String> {
        let field_handle = self.source_mapper.bytecode.field_handle_at(field_idx);
        let struct_def = self
            .source_mapper
            .bytecode
            .struct_def_at(field_handle.owner);
        let field_def = match &struct_def.field_information {
            StructFieldInformation::Native => {
                return Err(format_err!("Attempt to access field on a native struct"));
            }
            StructFieldInformation::Declared(fields) => fields
                .get(field_handle.field as usize)
                .ok_or_else(|| format_err!("Bad field index"))?,
        };
        let field_name = self
            .source_mapper
            .bytecode
            .identifier_at(field_def.name)
            .to_string();
        let struct_handle = self
            .source_mapper
            .bytecode
            .struct_handle_at(struct_def.struct_handle);
        let struct_name = self
            .source_mapper
            .bytecode
            .identifier_at(struct_handle.name)
            .to_string();
        Ok(format!("{}.{}", struct_name, field_name))
    }

    fn type_for_field(&self, field_idx: FieldHandleIndex) -> Result<String> {
        let field_handle = self.source_mapper.bytecode.field_handle_at(field_idx);
        let struct_def = self
            .source_mapper
            .bytecode
            .struct_def_at(field_handle.owner);
        let field_def = match &struct_def.field_information {
            StructFieldInformation::Native => {
                return Err(format_err!("Attempt to access field on a native struct"));
            }
            StructFieldInformation::Declared(fields) => fields
                .get(field_handle.field as usize)
                .ok_or_else(|| format_err!("Bad field index"))?,
        };
        let struct_source_info = self
            .source_mapper
            .source_map
            .get_struct_source_map(field_handle.owner)?;
        let field_type_sig = field_def.signature.0.clone();
        let ty = self.disassemble_sig_tok(field_type_sig, &struct_source_info.type_parameters)?;
        Ok(ty)
    }

    fn struct_type_info(
        &self,
        struct_idx: StructDefinitionIndex,
        signature: &Signature,
    ) -> Result<(String, String)> {
        let struct_definition = self.get_struct_def(struct_idx)?;
        let struct_source_map = self
            .source_mapper
            .source_map
            .get_struct_source_map(struct_idx)?;
        let type_arguments = signature
            .0
            .iter()
            .map(|sig_tok| {
                Ok(self.disassemble_sig_tok(sig_tok.clone(), &struct_source_map.type_parameters)?)
            })
            .collect::<Result<Vec<String>>>()?;

        let struct_handle = self
            .source_mapper
            .bytecode
            .struct_handle_at(struct_definition.struct_handle);
        let name = self
            .source_mapper
            .bytecode
            .identifier_at(struct_handle.name)
            .to_string();
        Ok((name, Self::format_type_params(&type_arguments)))
    }

    fn name_for_parameter_or_local(
        &self,
        local_idx: usize,
        function_source_map: &FunctionSourceMap<Location>,
    ) -> Result<String> {
        let name = function_source_map
                .get_parameter_or_local_name(local_idx as u64)
                .ok_or_else(|| {
                    format_err!(
                        "Unable to get local name at index {} while disassembling location-based instruction", local_idx
                    )
                })?
                .0;
        Ok(name)
    }

    fn type_for_parameter_or_local(
        &self,
        idx: usize,
        parameters: &Signature,
        locals: &Signature,
        function_source_map: &FunctionSourceMap<Location>,
    ) -> Result<String> {
        let sig_tok = if idx < parameters.len() {
            &parameters.0[idx]
        } else if idx < parameters.len() + locals.len() {
            &locals.0[idx - parameters.len()]
        } else {
            bail!("Unable to get type for parameter or local at index {}", idx)
        };
        self.disassemble_sig_tok(sig_tok.clone(), &function_source_map.type_parameters)
    }

    fn type_for_local(
        &self,
        local_idx: usize,
        locals: &Signature,
        function_source_map: &FunctionSourceMap<Location>,
    ) -> Result<String> {
        let sig_tok = locals
            .0
            .get(local_idx as usize)
            .ok_or_else(|| format_err!("Unable to get type for local at index {}", local_idx))?;
        self.disassemble_sig_tok(sig_tok.clone(), &function_source_map.type_parameters)
    }

    fn format_type_params(ty_params: &[String]) -> String {
        if ty_params.is_empty() {
            "".to_string()
        } else {
            format!("<{}>", ty_params.join(", "))
        }
    }

    fn format_ret_type(ty_rets: &[String]) -> String {
        if ty_rets.is_empty() {
            "".to_string()
        } else {
            format!(": {}", ty_rets.join(" * "))
        }
    }

    fn format_function_body(locals: Vec<String>, bytecode: Vec<String>) -> String {
        if locals.is_empty() && bytecode.is_empty() {
            "".to_string()
        } else {
            let body_iter: Vec<String> = locals
                .into_iter()
                .enumerate()
                .map(|(local_idx, local)| format!("L{}:\t{}", local_idx, local))
                .chain(bytecode.into_iter())
                .collect();
            format!(" {{\n{}\n}}", body_iter.join("\n"))
        }
    }

    //***************************************************************************
    // Disassemblers
    //***************************************************************************

    // These need to be in the context of a function or a struct definition since type parameters
    // can refer to function/struct type parameters.
    fn disassemble_sig_tok(
        &self,
        sig_tok: SignatureToken,
        type_param_context: &[SourceName<Location>],
    ) -> Result<String> {
        Ok(match sig_tok {
            SignatureToken::Bool => "bool".to_string(),
            SignatureToken::U8 => "u8".to_string(),
            SignatureToken::U64 => "u64".to_string(),
            SignatureToken::U128 => "u128".to_string(),
            SignatureToken::Address => "address".to_string(),
            SignatureToken::Struct(struct_handle_idx) => self
                .source_mapper
                .bytecode
                .identifier_at(
                    self.source_mapper
                        .bytecode
                        .struct_handle_at(struct_handle_idx)
                        .name,
                )
                .to_string(),
            SignatureToken::StructInstantiation(struct_handle_idx, instantiation) => {
                let instantiation = instantiation
                    .into_iter()
                    .map(|tok| self.disassemble_sig_tok(tok, type_param_context))
                    .collect::<Result<Vec<_>>>()?;
                let formatted_instantiation = Self::format_type_params(&instantiation);
                let name = self
                    .source_mapper
                    .bytecode
                    .identifier_at(
                        self.source_mapper
                            .bytecode
                            .struct_handle_at(struct_handle_idx)
                            .name,
                    )
                    .to_string();
                format!("{}{}", name, formatted_instantiation)
            }
            SignatureToken::Vector(sig_tok) => format!(
                "vector<{}>",
                self.disassemble_sig_tok(*sig_tok, type_param_context)?
            ),
            SignatureToken::Reference(sig_tok) => format!(
                "&{}",
                self.disassemble_sig_tok(*sig_tok, type_param_context)?
            ),
            SignatureToken::MutableReference(sig_tok) => format!(
                "&mut {}",
                self.disassemble_sig_tok(*sig_tok, type_param_context)?
            ),
            SignatureToken::TypeParameter(ty_param_index) => type_param_context
                .get(ty_param_index as usize)
                .ok_or_else(|| {
                    format_err!(
                        "Type parameter index {} out of bounds while disassembling type signature",
                        ty_param_index
                    )
                })?
                .0
                .to_string(),
        })
    }

    fn disassemble_instruction(
        &self,
        parameters: &Signature,
        instruction: &Bytecode,
        locals_sigs: &Signature,
        function_source_map: &FunctionSourceMap<Location>,
        default_location: &Location,
    ) -> Result<String> {
        match instruction {
            Bytecode::LdConst(idx) => {
                let constant = self.source_mapper.bytecode.constant_at(*idx);
                Ok(format!(
                    "LdAddr[{}]({:?}: {:?})",
                    idx, &constant.type_, &constant.data
                ))
            }
            Bytecode::CopyLoc(local_idx) => {
                let name =
                    self.name_for_parameter_or_local(usize::from(*local_idx), function_source_map)?;
                let ty = self.type_for_parameter_or_local(
                    usize::from(*local_idx),
                    parameters,
                    locals_sigs,
                    function_source_map,
                )?;
                Ok(format!("CopyLoc[{}]({}: {})", local_idx, name, ty))
            }
            Bytecode::MoveLoc(local_idx) => {
                let name =
                    self.name_for_parameter_or_local(usize::from(*local_idx), function_source_map)?;
                let ty = self.type_for_parameter_or_local(
                    usize::from(*local_idx),
                    parameters,
                    locals_sigs,
                    function_source_map,
                )?;
                Ok(format!("MoveLoc[{}]({}: {})", local_idx, name, ty))
            }
            Bytecode::StLoc(local_idx) => {
                let name =
                    self.name_for_parameter_or_local(usize::from(*local_idx), function_source_map)?;
                let ty = self.type_for_parameter_or_local(
                    usize::from(*local_idx),
                    parameters,
                    locals_sigs,
                    function_source_map,
                )?;
                Ok(format!("StLoc[{}]({}: {})", local_idx, name, ty))
            }
            Bytecode::MutBorrowLoc(local_idx) => {
                let name =
                    self.name_for_parameter_or_local(usize::from(*local_idx), function_source_map)?;
                let ty = self.type_for_parameter_or_local(
                    usize::from(*local_idx),
                    parameters,
                    locals_sigs,
                    function_source_map,
                )?;
                Ok(format!("MutBorrowLoc[{}]({}: {})", local_idx, name, ty))
            }
            Bytecode::ImmBorrowLoc(local_idx) => {
                let name =
                    self.name_for_parameter_or_local(usize::from(*local_idx), function_source_map)?;
                let ty = self.type_for_parameter_or_local(
                    usize::from(*local_idx),
                    parameters,
                    locals_sigs,
                    function_source_map,
                )?;
                Ok(format!("ImmBorrowLoc[{}]({}: {})", local_idx, name, ty))
            }
            Bytecode::MutBorrowField(field_idx) => {
                let name = self.name_for_field(*field_idx)?;
                let ty = self.type_for_field(*field_idx)?;
                Ok(format!("MutBorrowField[{}]({}: {})", field_idx, name, ty))
            }
            Bytecode::MutBorrowFieldGeneric(field_idx) => {
                let field_inst = self
                    .source_mapper
                    .bytecode
                    .field_instantiation_at(*field_idx);
                let name = self.name_for_field(field_inst.handle)?;
                let ty = self.type_for_field(field_inst.handle)?;
                Ok(format!(
                    "MutBorrowFieldGeneric[{}]({}: {})",
                    field_idx, name, ty
                ))
            }
            Bytecode::ImmBorrowField(field_idx) => {
                let name = self.name_for_field(*field_idx)?;
                let ty = self.type_for_field(*field_idx)?;
                Ok(format!("ImmBorrowField[{}]({}: {})", field_idx, name, ty))
            }
            Bytecode::ImmBorrowFieldGeneric(field_idx) => {
                let field_inst = self
                    .source_mapper
                    .bytecode
                    .field_instantiation_at(*field_idx);
                let name = self.name_for_field(field_inst.handle)?;
                let ty = self.type_for_field(field_inst.handle)?;
                Ok(format!(
                    "ImmBorrowFieldGeneric[{}]({}: {})",
                    field_idx, name, ty
                ))
            }
            Bytecode::Pack(struct_idx) => {
                let (name, ty_params) = self.struct_type_info(*struct_idx, &Signature(vec![]))?;
                Ok(format!("Pack[{}]({}{})", struct_idx, name, ty_params))
            }
            Bytecode::PackGeneric(struct_idx) => {
                let struct_inst = self
                    .source_mapper
                    .bytecode
                    .struct_instantiation_at(*struct_idx);
                let type_params = self
                    .source_mapper
                    .bytecode
                    .signature_at(struct_inst.type_parameters);
                let (name, ty_params) = self.struct_type_info(struct_inst.def, type_params)?;
                Ok(format!(
                    "PackGeneric[{}]({}{})",
                    struct_idx, name, ty_params
                ))
            }
            Bytecode::Unpack(struct_idx) => {
                let (name, ty_params) = self.struct_type_info(*struct_idx, &Signature(vec![]))?;
                Ok(format!("Unpack[{}]({}{})", struct_idx, name, ty_params))
            }
            Bytecode::UnpackGeneric(struct_idx) => {
                let struct_inst = self
                    .source_mapper
                    .bytecode
                    .struct_instantiation_at(*struct_idx);
                let type_params = self
                    .source_mapper
                    .bytecode
                    .signature_at(struct_inst.type_parameters);
                let (name, ty_params) = self.struct_type_info(struct_inst.def, type_params)?;
                Ok(format!(
                    "UnpackGeneric[{}]({}{})",
                    struct_idx, name, ty_params
                ))
            }
            Bytecode::Exists(struct_idx) => {
                let (name, ty_params) = self.struct_type_info(*struct_idx, &Signature(vec![]))?;
                Ok(format!("Exists[{}]({}{})", struct_idx, name, ty_params))
            }
            Bytecode::ExistsGeneric(struct_idx) => {
                let struct_inst = self
                    .source_mapper
                    .bytecode
                    .struct_instantiation_at(*struct_idx);
                let type_params = self
                    .source_mapper
                    .bytecode
                    .signature_at(struct_inst.type_parameters);
                let (name, ty_params) = self.struct_type_info(struct_inst.def, type_params)?;
                Ok(format!(
                    "ExistsGeneric[{}]({}{})",
                    struct_idx, name, ty_params
                ))
            }
            Bytecode::MutBorrowGlobal(struct_idx) => {
                let (name, ty_params) = self.struct_type_info(*struct_idx, &Signature(vec![]))?;
                Ok(format!(
                    "MutBorrowGlobal[{}]({}{})",
                    struct_idx, name, ty_params
                ))
            }
            Bytecode::MutBorrowGlobalGeneric(struct_idx) => {
                let struct_inst = self
                    .source_mapper
                    .bytecode
                    .struct_instantiation_at(*struct_idx);
                let type_params = self
                    .source_mapper
                    .bytecode
                    .signature_at(struct_inst.type_parameters);
                let (name, ty_params) = self.struct_type_info(struct_inst.def, type_params)?;
                Ok(format!(
                    "MutBorrowGlobalGeneric[{}]({}{})",
                    struct_idx, name, ty_params
                ))
            }
            Bytecode::ImmBorrowGlobal(struct_idx) => {
                let (name, ty_params) = self.struct_type_info(*struct_idx, &Signature(vec![]))?;
                Ok(format!(
                    "ImmBorrowGlobal[{}]({}{})",
                    struct_idx, name, ty_params
                ))
            }
            Bytecode::ImmBorrowGlobalGeneric(struct_idx) => {
                let struct_inst = self
                    .source_mapper
                    .bytecode
                    .struct_instantiation_at(*struct_idx);
                let type_params = self
                    .source_mapper
                    .bytecode
                    .signature_at(struct_inst.type_parameters);
                let (name, ty_params) = self.struct_type_info(struct_inst.def, type_params)?;
                Ok(format!(
                    "ImmBorrowGlobalGeneric[{}]({}{})",
                    struct_idx, name, ty_params
                ))
            }
            Bytecode::MoveFrom(struct_idx) => {
                let (name, ty_params) = self.struct_type_info(*struct_idx, &Signature(vec![]))?;
                Ok(format!("MoveFrom[{}]({}{})", struct_idx, name, ty_params))
            }
            Bytecode::MoveFromGeneric(struct_idx) => {
                let struct_inst = self
                    .source_mapper
                    .bytecode
                    .struct_instantiation_at(*struct_idx);
                let type_params = self
                    .source_mapper
                    .bytecode
                    .signature_at(struct_inst.type_parameters);
                let (name, ty_params) = self.struct_type_info(struct_inst.def, type_params)?;
                Ok(format!(
                    "MoveFromGeneric[{}]({}{})",
                    struct_idx, name, ty_params
                ))
            }
            Bytecode::MoveToSender(struct_idx) => {
                let (name, ty_params) = self.struct_type_info(*struct_idx, &Signature(vec![]))?;
                Ok(format!(
                    "MoveToSender[{}]({}{})",
                    struct_idx, name, ty_params
                ))
            }
            Bytecode::MoveToSenderGeneric(struct_idx) => {
                let struct_inst = self
                    .source_mapper
                    .bytecode
                    .struct_instantiation_at(*struct_idx);
                let type_params = self
                    .source_mapper
                    .bytecode
                    .signature_at(struct_inst.type_parameters);
                let (name, ty_params) = self.struct_type_info(struct_inst.def, type_params)?;
                Ok(format!(
                    "MoveToSenderGeneric[{}]({}{})",
                    struct_idx, name, ty_params
                ))
            }
            Bytecode::Call(method_idx) => {
                let function_handle = self.source_mapper.bytecode.function_handle_at(*method_idx);
                let fcall_name = self
                    .source_mapper
                    .bytecode
                    .identifier_at(function_handle.name)
                    .to_string();
                let type_arguments = self
                    .source_mapper
                    .bytecode
                    .signature_at(function_handle.parameters)
                    .0
                    .iter()
                    .map(|sig_tok| Ok(self.disassemble_sig_tok(sig_tok.clone(), &[])?))
                    .collect::<Result<Vec<String>>>()?
                    .join(", ");
                let type_rets = self
                    .source_mapper
                    .bytecode
                    .signature_at(function_handle.return_)
                    .0
                    .iter()
                    .map(|sig_tok| Ok(self.disassemble_sig_tok(sig_tok.clone(), &[])?))
                    .collect::<Result<Vec<String>>>()?;
                Ok(format!(
                    "Call[{}]({}({}){})",
                    method_idx,
                    fcall_name,
                    type_arguments,
                    Self::format_ret_type(&type_rets)
                ))
            }
            Bytecode::CallGeneric(method_idx) => {
                let func_inst = self
                    .source_mapper
                    .bytecode
                    .function_instantiation_at(*method_idx);
                let function_handle = self
                    .source_mapper
                    .bytecode
                    .function_handle_at(func_inst.handle);
                let fcall_name = self
                    .source_mapper
                    .bytecode
                    .identifier_at(function_handle.name)
                    .to_string();
                let ty_params = self
                    .source_mapper
                    .bytecode
                    .signature_at(func_inst.type_parameters)
                    .0
                    .iter()
                    .map(|sig_tok| {
                        Ok((
                            self.disassemble_sig_tok(
                                sig_tok.clone(),
                                &function_source_map.type_parameters,
                            )?,
                            default_location.clone(),
                        ))
                    })
                    .collect::<Result<Vec<_>>>()?;
                let type_arguments = self
                    .source_mapper
                    .bytecode
                    .signature_at(function_handle.parameters)
                    .0
                    .iter()
                    .map(|sig_tok| Ok(self.disassemble_sig_tok(sig_tok.clone(), &ty_params)?))
                    .collect::<Result<Vec<String>>>()?
                    .join(", ");
                let type_rets = self
                    .source_mapper
                    .bytecode
                    .signature_at(function_handle.return_)
                    .0
                    .iter()
                    .map(|sig_tok| Ok(self.disassemble_sig_tok(sig_tok.clone(), &ty_params)?))
                    .collect::<Result<Vec<String>>>()?;
                Ok(format!(
                    "Call[{}]({}{}({}){})",
                    method_idx,
                    fcall_name,
                    Self::format_type_params(
                        &ty_params.into_iter().map(|(s, _)| s).collect::<Vec<_>>()
                    ),
                    type_arguments,
                    Self::format_ret_type(&type_rets)
                ))
            }
            // All other instructions are OK to be printed using the standard debug print.
            x => Ok(format!("{:#?}", x)),
        }
    }

    fn disassemble_bytecode(
        &self,
        function_definition_index: FunctionDefinitionIndex,
    ) -> Result<Vec<String>> {
        if !self.options.print_code {
            return Ok(vec!["".to_string()]);
        }

        let function_def = self.get_function_def(function_definition_index)?;
        let function_handle = self
            .source_mapper
            .bytecode
            .function_handle_at(function_def.function);
        let code = match &function_def.code {
            Some(code) => code,
            None => return Ok(vec!["".to_string()]),
        };

        let parameters = self
            .source_mapper
            .bytecode
            .signature_at(function_handle.parameters);
        let locals_sigs = self.source_mapper.bytecode.signature_at(code.locals);
        let function_source_map = self
            .source_mapper
            .source_map
            .get_function_source_map(function_definition_index)?;

        let function_name = self
            .source_mapper
            .bytecode
            .identifier_at(function_handle.name);
        let function_code_coverage_map = self.get_function_coverage(function_name);

        let decl_location = &function_source_map.decl_location;
        let instrs: Vec<String> = code
            .code
            .iter()
            .map(|instruction| {
                self.disassemble_instruction(
                    parameters,
                    instruction,
                    locals_sigs,
                    function_source_map,
                    &decl_location,
                )
            })
            .collect::<Result<Vec<String>>>()?;

        let mut instrs: Vec<String> = instrs
            .into_iter()
            .enumerate()
            .map(|(instr_index, dis_instr)| {
                Self::format_with_instruction_coverage(
                    instr_index,
                    function_code_coverage_map,
                    dis_instr,
                )
            })
            .collect();

        if self.options.print_basic_blocks {
            let cfg = VMControlFlowGraph::new(&code.code);
            for (block_number, block_id) in cfg.blocks().iter().enumerate() {
                instrs.insert(
                    *block_id as usize + block_number,
                    format!("B{}:", block_number),
                );
            }
        }

        Ok(instrs)
    }

    fn disassemble_type_formals(
        source_map_ty_params: &[SourceName<Location>],
        kinds: &[Kind],
    ) -> String {
        let ty_params: Vec<String> = source_map_ty_params
            .iter()
            .zip(kinds.iter())
            .map(|((name, _), kind)| format!("{}: {:#?}", name.as_str(), kind))
            .collect();
        Self::format_type_params(&ty_params)
    }

    fn disassemble_locals(
        &self,
        function_source_map: &FunctionSourceMap<Location>,
        locals_idx: SignatureIndex,
        parameter_len: usize,
    ) -> Result<Vec<String>> {
        if !self.options.print_locals {
            return Ok(vec![]);
        }

        let signature = self.source_mapper.bytecode.signature_at(locals_idx);
        let locals_names_tys = function_source_map
            .locals
            .iter()
            .skip(parameter_len)
            .enumerate()
            .map(|(local_idx, (name, _))| {
                let ty =
                    self.type_for_local(parameter_len + local_idx, signature, function_source_map)?;
                Ok(format!("{}: {}", name.to_string(), ty))
            })
            .collect::<Result<Vec<String>>>()?;
        Ok(locals_names_tys)
    }

    pub fn disassemble_function_def(
        &self,
        function_definition_index: FunctionDefinitionIndex,
    ) -> Result<String> {
        let function_definition = self.get_function_def(function_definition_index)?;
        let function_handle = self
            .source_mapper
            .bytecode
            .function_handle_at(function_definition.function);

        let function_source_map = self
            .source_mapper
            .source_map
            .get_function_source_map(function_definition_index)?;

        if self.options.only_public && !function_definition.is_public() {
            return Ok("".to_string());
        }

        let visibility_modifier = if function_definition.is_native() {
            "native "
        } else if function_definition.is_public() {
            "public "
        } else {
            ""
        };

        let ty_params = Self::disassemble_type_formals(
            &function_source_map.type_parameters,
            &function_handle.type_parameters,
        );
        let name = self
            .source_mapper
            .bytecode
            .identifier_at(function_handle.name);
        let return_ = self
            .source_mapper
            .bytecode
            .signature_at(function_handle.return_);
        let ret_type: Vec<String> = return_
            .0
            .iter()
            .cloned()
            .map(|sig_token| {
                let sig_tok_str =
                    self.disassemble_sig_tok(sig_token, &function_source_map.type_parameters)?;
                Ok(sig_tok_str)
            })
            .collect::<Result<Vec<String>>>()?;
        let parameters_sig = &self
            .source_mapper
            .bytecode
            .signature_at(function_handle.parameters);
        let parameters = parameters_sig
            .0
            .iter()
            .zip(function_source_map.locals.iter())
            .map(|(tok, (name, _))| {
                Ok(format!(
                    "{}: {}",
                    name,
                    self.disassemble_sig_tok(tok.clone(), &function_source_map.type_parameters)?
                ))
            })
            .collect::<Result<Vec<_>>>()?;

        let body = match &function_definition.code {
            Some(code) => {
                let locals =
                    self.disassemble_locals(function_source_map, code.locals, parameters.len())?;
                let bytecode = self.disassemble_bytecode(function_definition_index)?;
                Self::format_function_body(locals, bytecode)
            }
            None => "".to_string(),
        };
        Ok(self.format_function_coverage(
            name,
            format!(
                "{visibility_modifier}{name}{ty_params}({params}){ret_type}{body}",
                visibility_modifier = visibility_modifier,
                name = name,
                ty_params = ty_params,
                params = &parameters.join(", "),
                ret_type = Self::format_ret_type(&ret_type),
                body = body,
            ),
        ))
    }

    // The struct defs will filter out the structs that we print to only be the ones that are
    // defined in the module in question.
    pub fn disassemble_struct_def(&self, struct_def_idx: StructDefinitionIndex) -> Result<String> {
        let struct_definition = self.get_struct_def(struct_def_idx)?;
        let struct_handle = self
            .source_mapper
            .bytecode
            .struct_handle_at(struct_definition.struct_handle);
        let struct_source_map = self
            .source_mapper
            .source_map
            .get_struct_source_map(struct_def_idx)?;

        let field_info: Option<Vec<(&IdentStr, &TypeSignature)>> =
            match &struct_definition.field_information {
                StructFieldInformation::Native => None,
                StructFieldInformation::Declared(fields) => Some(
                    fields
                        .iter()
                        .map(|field_definition| {
                            let type_sig = &field_definition.signature;
                            let field_name = self
                                .source_mapper
                                .bytecode
                                .identifier_at(field_definition.name);
                            (field_name, type_sig)
                        })
                        .collect(),
                ),
            };

        let native = if field_info.is_none() { "native " } else { "" };

        let nominal_name = if struct_handle.is_nominal_resource {
            "resource"
        } else {
            "struct"
        };

        let name = self
            .source_mapper
            .bytecode
            .identifier_at(struct_handle.name)
            .to_string();

        let ty_params = Self::disassemble_type_formals(
            &struct_source_map.type_parameters,
            &struct_handle.type_parameters,
        );
        let mut fields = match field_info {
            None => vec![],
            Some(field_info) => field_info
                .iter()
                .map(|(name, ty)| {
                    let ty_str =
                        self.disassemble_sig_tok(ty.0.clone(), &struct_source_map.type_parameters)?;
                    Ok(format!("{}: {}", name.to_string(), ty_str))
                })
                .collect::<Result<Vec<String>>>()?,
        };

        if let Some(first_elem) = fields.first_mut() {
            first_elem.insert_str(0, "{\n\t");
        }

        if let Some(last_elem) = fields.last_mut() {
            last_elem.push_str("\n}");
        }

        Ok(format!(
            "{native}{nominal_name} {name}{ty_params} {fields}",
            native = native,
            nominal_name = nominal_name,
            name = name,
            ty_params = ty_params,
            fields = &fields.join(",\n\t"),
        ))
    }

    pub fn disassemble(&self) -> Result<String> {
        let name_opt = self.source_mapper.source_map.module_name_opt.as_ref();
        let name = name_opt.map(|(addr, n)| format!("{}.{}", addr.short_str(), n.to_string()));
        let header = match name {
            Some(s) => format!("module {}", s),
            None => "script".to_owned(),
        };

        let struct_defs: Vec<String> = (0..self.source_mapper.bytecode.struct_defs().len())
            .map(|i| self.disassemble_struct_def(StructDefinitionIndex(i as TableIndex)))
            .collect::<Result<Vec<String>>>()?;

        let function_defs: Vec<String> = (0..self.source_mapper.bytecode.function_defs().len())
            .map(|i| self.disassemble_function_def(FunctionDefinitionIndex(i as TableIndex)))
            .collect::<Result<Vec<String>>>()?;

        Ok(format!(
            "{header} {{\n{struct_defs}\n\n{function_defs}\n}}",
            header = header,
            struct_defs = &struct_defs.join("\n"),
            function_defs = &function_defs.join("\n")
        ))
    }
}
