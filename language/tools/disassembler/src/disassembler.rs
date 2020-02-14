// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, format_err, Result};
use bytecode_source_map::{
    mapping::SourceMapping,
    source_map::{FunctionSourceMap, SourceName},
};
use bytecode_verifier::control_flow_graph::{ControlFlowGraph, VMControlFlowGraph};
use libra_types::identifier::{IdentStr, Identifier};
use vm::{
    access::ModuleAccess,
    file_format::{
        Bytecode, FieldDefinitionIndex, FunctionDefinition, FunctionDefinitionIndex,
        FunctionSignature, Kind, LocalsSignature, LocalsSignatureIndex, SignatureToken,
        StructDefinition, StructDefinitionIndex, StructFieldInformation, TableIndex, TypeSignature,
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

pub struct Disassembler<Location: Clone + Eq + Default> {
    source_mapper: SourceMapping<Location>,
    // The various options that we can set for disassembly.
    options: DisassemblerOptions,
}

impl<Location: Clone + Eq + Default> Disassembler<Location> {
    pub fn new(source_mapper: SourceMapping<Location>, options: DisassemblerOptions) -> Self {
        Self {
            source_mapper,
            options,
        }
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
    // Formatting Helpers
    //***************************************************************************

    fn name_for_field(&self, field_idx: FieldDefinitionIndex) -> Result<String> {
        let field_def = self.source_mapper.bytecode.field_def_at(field_idx);
        let struct_def_idx = self
            .source_mapper
            .bytecode
            .struct_defs()
            .iter()
            .position(|struct_def| struct_def.struct_handle == field_def.struct_)
            .ok_or_else(|| format_err!("Unable to find struct definition for struct field"))?;
        let field_name = self
            .source_mapper
            .bytecode
            .identifier_at(field_def.name)
            .to_string();
        let struct_def = self
            .source_mapper
            .bytecode
            .struct_def_at(StructDefinitionIndex(struct_def_idx as TableIndex));
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

    fn type_for_field(&self, field_idx: FieldDefinitionIndex) -> Result<String> {
        let field_def = self.source_mapper.bytecode.field_def_at(field_idx);
        let struct_def_idx = self
            .source_mapper
            .bytecode
            .struct_defs()
            .iter()
            .position(|struct_def| struct_def.struct_handle == field_def.struct_)
            .ok_or_else(|| format_err!("Unable to find struct definition for struct field"))?;
        let struct_source_info = self
            .source_mapper
            .source_map
            .get_struct_source_map(StructDefinitionIndex(struct_def_idx as TableIndex))?;
        let field_type_sig = self
            .source_mapper
            .bytecode
            .type_signature_at(field_def.signature);
        let ty = self.disassemble_sig_tok(
            field_type_sig.0.clone(),
            &struct_source_info.type_parameters,
        )?;
        Ok(ty)
    }

    fn struct_type_info(
        &self,
        struct_idx: StructDefinitionIndex,
        types_idx: LocalsSignatureIndex,
    ) -> Result<(String, String)> {
        let struct_definition = self.get_struct_def(struct_idx)?;
        let struct_source_map = self
            .source_mapper
            .source_map
            .get_struct_source_map(struct_idx)?;
        let locals_signature = self.source_mapper.bytecode.locals_signature_at(types_idx);
        let type_arguments = locals_signature
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

    fn name_for_local(
        &self,
        local_idx: u64,
        function_source_map: &FunctionSourceMap<Location>,
    ) -> Result<String> {
        let name = function_source_map
                .get_local_name(local_idx)
                .ok_or_else(|| {
                    format_err!(
                        "Unable to get local name at index {} while disassembling location-based instruction", local_idx
                    )
                })?
                .0;
        Ok(name.to_string())
    }

    fn type_for_local(
        &self,
        local_idx: u64,
        locals_sigs: &LocalsSignature,
        function_source_map: &FunctionSourceMap<Location>,
    ) -> Result<String> {
        let sig_tok = locals_sigs
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
            SignatureToken::ByteArray => "bytearray".to_string(),
            SignatureToken::Address => "address".to_string(),
            SignatureToken::Struct(struct_handle_idx, instantiation) => {
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
        instruction: &Bytecode,
        locals_sigs: &LocalsSignature,
        function_source_map: &FunctionSourceMap<Location>,
    ) -> Result<String> {
        match instruction {
            Bytecode::LdAddr(address_idx) => {
                let address = self
                    .source_mapper
                    .bytecode
                    .address_at(*address_idx)
                    .short_str();
                Ok(format!("LdAddr[{}]({})", address_idx, address))
            }
            Bytecode::LdByteArray(byte_array_idx) => {
                let bytearray = self.source_mapper.bytecode.byte_array_at(*byte_array_idx);
                Ok(format!("LdByteArray[{}]({:?})", byte_array_idx, bytearray))
            }
            Bytecode::CopyLoc(local_idx) => {
                let name = self.name_for_local(u64::from(*local_idx), function_source_map)?;
                let ty =
                    self.type_for_local(u64::from(*local_idx), locals_sigs, function_source_map)?;
                Ok(format!("CopyLoc[{}]({}: {})", local_idx, name, ty))
            }
            Bytecode::MoveLoc(local_idx) => {
                let name = self.name_for_local(u64::from(*local_idx), function_source_map)?;
                let ty =
                    self.type_for_local(u64::from(*local_idx), locals_sigs, function_source_map)?;
                Ok(format!("MoveLoc[{}]({}: {})", local_idx, name, ty))
            }
            Bytecode::StLoc(local_idx) => {
                let name = self.name_for_local(u64::from(*local_idx), function_source_map)?;
                let ty =
                    self.type_for_local(u64::from(*local_idx), locals_sigs, function_source_map)?;
                Ok(format!("StLoc[{}]({}: {})", local_idx, name, ty))
            }
            Bytecode::MutBorrowLoc(local_idx) => {
                let name = self.name_for_local(u64::from(*local_idx), function_source_map)?;
                let ty =
                    self.type_for_local(u64::from(*local_idx), locals_sigs, function_source_map)?;
                Ok(format!("MutBorrowLoc[{}]({}: {})", local_idx, name, ty))
            }
            Bytecode::ImmBorrowLoc(local_idx) => {
                let name = self.name_for_local(u64::from(*local_idx), function_source_map)?;
                let ty =
                    self.type_for_local(u64::from(*local_idx), locals_sigs, function_source_map)?;
                Ok(format!("ImmBorrowLoc[{}]({}: {})", local_idx, name, ty))
            }
            Bytecode::MutBorrowField(field_idx) => {
                let name = self.name_for_field(*field_idx)?;
                let ty = self.type_for_field(*field_idx)?;
                Ok(format!("MutBorrowField[{}]({}: {})", field_idx, name, ty))
            }
            Bytecode::ImmBorrowField(field_idx) => {
                let name = self.name_for_field(*field_idx)?;
                let ty = self.type_for_field(*field_idx)?;
                Ok(format!("ImmBorrowField[{}]({}: {})", field_idx, name, ty))
            }
            Bytecode::Pack(struct_idx, types_idx) => {
                let (name, ty_params) = self.struct_type_info(*struct_idx, *types_idx)?;
                Ok(format!("Pack[{}]({}{})", struct_idx, name, ty_params))
            }
            Bytecode::Unpack(struct_idx, types_idx) => {
                let (name, ty_params) = self.struct_type_info(*struct_idx, *types_idx)?;
                Ok(format!("Unpack[{}]({}{})", struct_idx, name, ty_params))
            }
            Bytecode::Exists(struct_idx, types_idx) => {
                let (name, ty_params) = self.struct_type_info(*struct_idx, *types_idx)?;
                Ok(format!("Exists[{}]({}{})", struct_idx, name, ty_params))
            }
            Bytecode::MutBorrowGlobal(struct_idx, types_idx) => {
                let (name, ty_params) = self.struct_type_info(*struct_idx, *types_idx)?;
                Ok(format!(
                    "MutBorrowGlobal[{}]({}{})",
                    struct_idx, name, ty_params
                ))
            }
            Bytecode::ImmBorrowGlobal(struct_idx, types_idx) => {
                let (name, ty_params) = self.struct_type_info(*struct_idx, *types_idx)?;
                Ok(format!(
                    "ImmBorrowGlobal[{}]({}{})",
                    struct_idx, name, ty_params
                ))
            }
            Bytecode::MoveFrom(struct_idx, types_idx) => {
                let (name, ty_params) = self.struct_type_info(*struct_idx, *types_idx)?;
                Ok(format!("MoveFrom[{}]({}{})", struct_idx, name, ty_params))
            }
            Bytecode::MoveToSender(struct_idx, types_idx) => {
                let (name, ty_params) = self.struct_type_info(*struct_idx, *types_idx)?;
                Ok(format!(
                    "MoveToSender[{}]({}{})",
                    struct_idx, name, ty_params
                ))
            }
            Bytecode::Call(method_idx, locals_sig_index) => {
                let function_handle = self.source_mapper.bytecode.function_handle_at(*method_idx);
                let fcall_name = self
                    .source_mapper
                    .bytecode
                    .identifier_at(function_handle.name)
                    .to_string();
                let function_signature = self
                    .source_mapper
                    .bytecode
                    .function_signature_at(function_handle.signature);
                let ty_params = self
                    .source_mapper
                    .bytecode
                    .locals_signature_at(*locals_sig_index)
                    .0
                    .iter()
                    .map(|sig_tok| {
                        Ok((
                            Identifier::new(self.disassemble_sig_tok(
                                sig_tok.clone(),
                                &function_source_map.type_parameters,
                            )?)?,
                            Location::default(),
                        ))
                    })
                    .collect::<Result<Vec<_>>>()?;
                let type_arguments = function_signature
                    .arg_types
                    .iter()
                    .map(|sig_tok| Ok(self.disassemble_sig_tok(sig_tok.clone(), &ty_params)?))
                    .collect::<Result<Vec<String>>>()?
                    .join(", ");
                let type_rets = function_signature
                    .return_types
                    .iter()
                    .map(|sig_tok| Ok(self.disassemble_sig_tok(sig_tok.clone(), &ty_params)?))
                    .collect::<Result<Vec<String>>>()?;
                Ok(format!(
                    "Call[{}]({}{}({}){})",
                    method_idx,
                    fcall_name,
                    Self::format_type_params(
                        &ty_params
                            .into_iter()
                            .map(|x| x.0.to_string())
                            .collect::<Vec<_>>()
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
        let locals_sigs = self
            .source_mapper
            .bytecode
            .locals_signature_at(function_def.code.locals);
        let function_source_map = self
            .source_mapper
            .source_map
            .get_function_source_map(function_definition_index)?;

        let instrs: Vec<String> = function_def
            .code
            .code
            .iter()
            .map(|instruction| {
                self.disassemble_instruction(instruction, locals_sigs, function_source_map)
            })
            .collect::<Result<Vec<String>>>()?;

        let mut instrs: Vec<String> = instrs
            .into_iter()
            .enumerate()
            .map(|(instr_index, dis_instr)| format!("\t{}: {}", instr_index, dis_instr))
            .collect();

        if self.options.print_basic_blocks {
            let cfg = VMControlFlowGraph::new(&function_def.code.code);
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
        function_definition: &FunctionDefinition,
        function_signature: &FunctionSignature,
    ) -> Result<(Vec<String>, Vec<String>)> {
        let locals_signature = self
            .source_mapper
            .bytecode
            .locals_signature_at(function_definition.code.locals);
        let mut locals_names_tys = function_source_map
            .locals
            .iter()
            .enumerate()
            .map(|(local_idx, (name, _))| {
                let ty =
                    self.type_for_local(local_idx as u64, locals_signature, function_source_map)?;
                Ok(format!("{}: {}", name.to_string(), ty))
            })
            .collect::<Result<Vec<String>>>()?;
        let mut locals = locals_names_tys.split_off(function_signature.arg_types.len());
        let args = locals_names_tys;
        if !self.options.print_locals {
            locals = vec![];
        }
        Ok((args, locals))
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
        let function_signature = self
            .source_mapper
            .bytecode
            .function_signature_at(function_handle.signature);

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
            &function_signature.type_formals,
        );
        let name = self
            .source_mapper
            .bytecode
            .identifier_at(function_handle.name)
            .to_string();
        let ret_type: Vec<String> = function_signature
            .return_types
            .iter()
            .cloned()
            .map(|sig_token| {
                let sig_tok_str =
                    self.disassemble_sig_tok(sig_token, &function_source_map.type_parameters)?;
                Ok(sig_tok_str)
            })
            .collect::<Result<Vec<String>>>()?;
        let (args, locals) =
            self.disassemble_locals(function_source_map, function_definition, function_signature)?;
        let bytecode = self.disassemble_bytecode(function_definition_index)?;
        Ok(format!(
            "{visibility_modifier}{name}{ty_params}({args}){ret_type}{body}",
            visibility_modifier = visibility_modifier,
            name = name,
            ty_params = ty_params,
            args = &args.join(", "),
            ret_type = Self::format_ret_type(&ret_type),
            body = Self::format_function_body(locals, bytecode),
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
            match struct_definition.field_information {
                StructFieldInformation::Native => None,
                StructFieldInformation::Declared {
                    field_count,
                    fields,
                } => Some(
                    (fields.0..fields.0 + field_count)
                        .map(|i| {
                            let field_definition = self
                                .source_mapper
                                .bytecode
                                .field_def_at(FieldDefinitionIndex(i));
                            let type_sig = self
                                .source_mapper
                                .bytecode
                                .type_signature_at(field_definition.signature);
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
            &struct_handle.type_formals,
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
        let name = format!(
            "{}.{}",
            self.source_mapper.source_map.module_name.0.short_str(),
            self.source_mapper.source_map.module_name.1.to_string()
        );

        let struct_defs: Vec<String> = (0..self.source_mapper.bytecode.struct_defs().len())
            .map(|i| self.disassemble_struct_def(StructDefinitionIndex(i as TableIndex)))
            .collect::<Result<Vec<String>>>()?;

        let function_defs: Vec<String> = (0..self.source_mapper.bytecode.function_defs().len())
            .map(|i| self.disassemble_function_def(FunctionDefinitionIndex(i as TableIndex)))
            .collect::<Result<Vec<String>>>()?;

        Ok(format!(
            "module {name} {{\n{struct_defs}\n\n{function_defs}\n}}",
            name = name,
            struct_defs = &struct_defs.join("\n"),
            function_defs = &function_defs.join("\n")
        ))
    }
}
