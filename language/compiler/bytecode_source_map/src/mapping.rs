// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::marking::MarkedSourceMapping;
use crate::source_map::{ModuleSourceMap, SourceName};
use failure::prelude::*;
use libra_types::identifier::IdentStr;
use vm::access::ModuleAccess;
use vm::file_format::{
    CompiledModule, CompiledScript, FieldDefinitionIndex, FunctionDefinition,
    FunctionDefinitionIndex, Kind, SignatureToken, StructDefinition, StructDefinitionIndex,
    StructFieldInformation, TableIndex, TypeSignature,
};

/// Holds the various options that we support while disassembling code.
#[derive(Debug, Default)]
pub struct SourceMappingOptions {
    /// Only print public functions
    pub only_public: bool,

    // TODO: Need to implement this.
    /// Print the bytecode for the instructions within the function.
    pub print_code: bool,
}

/// An object that associates source code with compiled bytecode and source map.
#[derive(Debug)]
pub struct SourceMapping<Location: Clone + Eq> {
    // The various options that we can set for disassembly.
    options: SourceMappingOptions,

    // The resulting bytecode from compiling the source map
    bytecode: CompiledModule,

    // The source map for the bytecode made w.r.t. to the `source_code`
    source_map: ModuleSourceMap<Location>,

    // The source code for the bytecode. This is not required for disassembly, but it is required
    // for being able to print out corresponding source code for marked functions and structs.
    // Unused for now, this will be used when we start printing function/struct markings
    _source_code: Option<String>,

    // Function and struct markings. These are used to lift up annotations/messages on the bytecode
    // into the disassembled program and/or IR source code.
    marks: Option<MarkedSourceMapping>,
}

impl SourceMappingOptions {
    pub fn new() -> Self {
        Self {
            only_public: false,
            print_code: false,
        }
    }

    pub fn only_public(&mut self) {
        self.only_public = true;
    }

    pub fn print_code(&mut self) {
        self.print_code = true;
    }
}

impl<Location: Clone + Eq> SourceMapping<Location> {
    pub fn new(
        source_map: ModuleSourceMap<Location>,
        bytecode: CompiledModule,
        options: SourceMappingOptions,
    ) -> Self {
        Self {
            options,
            source_map,
            bytecode,
            _source_code: None,
            marks: None,
        }
    }

    pub fn new_from_script(
        source_map: ModuleSourceMap<Location>,
        bytecode: CompiledScript,
        options: SourceMappingOptions,
    ) -> Self {
        Self::new(source_map, bytecode.into_module(), options)
    }

    pub fn with_marks(&mut self, marks: MarkedSourceMapping) {
        self.marks = Some(marks);
    }

    pub fn with_source_code(&mut self, source_code: String) {
        self._source_code = Some(source_code);
    }

    //***************************************************************************
    // Helpers
    //***************************************************************************

    fn get_function_def(
        &self,
        function_definition_index: FunctionDefinitionIndex,
    ) -> Result<&FunctionDefinition> {
        if function_definition_index.0 as usize >= self.bytecode.function_defs().len() {
            bail!("Invalid function definition index supplied when marking function")
        }
        Ok(self.bytecode.function_def_at(function_definition_index))
    }

    fn get_struct_def(
        &self,
        struct_definition_index: StructDefinitionIndex,
    ) -> Result<&StructDefinition> {
        if struct_definition_index.0 as usize >= self.bytecode.struct_defs().len() {
            bail!("Invalid struct definition index supplied when marking struct")
        }
        Ok(self.bytecode.struct_def_at(struct_definition_index))
    }

    // These need to be in the context of a function or a struct definition since type parameters
    // can refer to function/struct type parameters.
    fn disassemble_sig_tok(
        &self,
        sig_tok: SignatureToken,
        type_param_context: &[SourceName<Location>],
    ) -> Result<String> {
        Ok(match sig_tok {
            SignatureToken::Bool => "bool".to_string(),
            SignatureToken::U64 => "u64".to_string(),
            SignatureToken::String => "string".to_string(),
            SignatureToken::ByteArray => "bytearray".to_string(),
            SignatureToken::Address => "address".to_string(),
            SignatureToken::Struct(struct_handle_idx, _) => self
                .bytecode
                .identifier_at(self.bytecode.struct_handle_at(struct_handle_idx).name)
                .to_string(),
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
                        "Type parameter index out of bounds while disassembling type signature"
                    )
                })?
                .0
                .to_string(),
        })
    }

    //***************************************************************************
    // Disassemblers
    //***************************************************************************

    fn disassemble_type_formals(
        source_map_ty_params: &[SourceName<Location>],
        kinds: &[Kind],
    ) -> String {
        let ty_params: Vec<String> = source_map_ty_params
            .iter()
            .zip(kinds.iter())
            .map(|((name, _), kind)| format!("{}: {:#?}", name.as_str(), kind))
            .collect();
        if ty_params.is_empty() {
            "".to_string()
        } else {
            format!("<{}>", ty_params.join(", "))
        }
    }

    pub fn disassemble_function_def(
        &self,
        function_definition_index: FunctionDefinitionIndex,
    ) -> Result<String> {
        let function_definition = self.get_function_def(function_definition_index)?;
        let function_handle = self
            .bytecode
            .function_handle_at(function_definition.function);
        let function_signature = self
            .bytecode
            .function_signature_at(function_handle.signature);

        let function_source_map = self
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
                Ok(sig_tok_str.to_string())
            })
            .collect::<Result<Vec<String>>>()?;
        let args: Vec<String> = function_signature
            .arg_types
            .iter()
            .zip(function_source_map.locals.iter())
            .map(|(typ, (name, _))| {
                let ty_str =
                    self.disassemble_sig_tok(typ.clone(), &function_source_map.type_parameters)?;
                Ok(format!("{}: {}", name.to_string(), ty_str))
            })
            .collect::<Result<Vec<String>>>()?;
        Ok(format!(
            "{visibility_modifier}{name}{ty_params}({args}):{ret_type}",
            visibility_modifier = visibility_modifier,
            name = name,
            ty_params = ty_params,
            args = &args.join(", "),
            ret_type = &ret_type.join(" * "),
        ))
    }

    // The struct defs will filter out the structs that we print to only be the ones that are
    // defined in the module in question.
    pub fn disassemble_struct_def(&self, struct_def_idx: StructDefinitionIndex) -> Result<String> {
        let struct_definition = self.get_struct_def(struct_def_idx)?;
        let struct_handle = self
            .bytecode
            .struct_handle_at(struct_definition.struct_handle);
        let struct_source_map = self.source_map.get_struct_source_map(struct_def_idx)?;

        let field_info: Option<Vec<(&IdentStr, &TypeSignature)>> = match struct_definition
            .field_information
        {
            StructFieldInformation::Native => None,
            StructFieldInformation::Declared {
                field_count,
                fields,
            } => Some(
                (fields.0..fields.0 + field_count)
                    .map(|i| {
                        let field_definition = self.bytecode.field_def_at(FieldDefinitionIndex(i));
                        let type_sig = self.bytecode.type_signature_at(field_definition.signature);
                        let field_name = self.bytecode.identifier_at(field_definition.name);
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

        let name = self.bytecode.identifier_at(struct_handle.name).to_string();

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
            self.source_map.module_name.0,
            self.source_map.module_name.1.to_string()
        );

        let struct_defs: Vec<String> = (0..self.bytecode.struct_defs().len())
            .map(|i| self.disassemble_struct_def(StructDefinitionIndex(i as TableIndex)))
            .collect::<Result<Vec<String>>>()?;

        let function_defs: Vec<String> = (0..self.bytecode.function_defs().len())
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
