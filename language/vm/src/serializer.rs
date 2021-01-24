// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Serialization of transactions and modules.
//!
//! This module exposes two entry points for serialization of `CompiledScript` and
//! `CompiledModule`. The entry points are exposed on the main structs `CompiledScript` and
//! `CompiledModule`.

use crate::{file_format::*, file_format_common::*};
use anyhow::{bail, Result};
use move_core_types::{account_address::AccountAddress, identifier::Identifier};

impl CompiledScript {
    /// Serializes a `CompiledScript` into a binary. The mutable `Vec<u8>` will contain the
    /// binary blob on return.
    pub fn serialize(&self, binary: &mut Vec<u8>) -> Result<()> {
        self.as_inner().serialize(binary)
    }
}

fn write_as_uleb128<T1, T2>(binary: &mut BinaryData, x: T1, max: T2) -> Result<()>
where
    T1: Into<u64>,
    T2: Into<u64>,
{
    let x: u64 = x.into();
    let max: u64 = max.into();
    if x > max {
        bail!("value ({}) cannot exceed ({})", x, max)
    }
    write_u64_as_uleb128(binary, x)
}

fn serialize_signature_index(binary: &mut BinaryData, idx: &SignatureIndex) -> Result<()> {
    write_as_uleb128(binary, idx.0, SIGNATURE_INDEX_MAX)
}

fn serialize_module_handle_index(binary: &mut BinaryData, idx: &ModuleHandleIndex) -> Result<()> {
    write_as_uleb128(binary, idx.0, MODULE_HANDLE_INDEX_MAX)
}

fn serialize_identifier_index(binary: &mut BinaryData, idx: &IdentifierIndex) -> Result<()> {
    write_as_uleb128(binary, idx.0, IDENTIFIER_INDEX_MAX)
}

fn serialize_struct_handle_index(binary: &mut BinaryData, idx: &StructHandleIndex) -> Result<()> {
    write_as_uleb128(binary, idx.0, STRUCT_HANDLE_INDEX_MAX)
}

fn serialize_address_identifier_index(
    binary: &mut BinaryData,
    idx: &AddressIdentifierIndex,
) -> Result<()> {
    write_as_uleb128(binary, idx.0, ADDRESS_INDEX_MAX)
}

fn serialize_struct_def_index(binary: &mut BinaryData, idx: &StructDefinitionIndex) -> Result<()> {
    write_as_uleb128(binary, idx.0, STRUCT_DEF_INDEX_MAX)
}

fn serialize_function_handle_index(
    binary: &mut BinaryData,
    idx: &FunctionHandleIndex,
) -> Result<()> {
    write_as_uleb128(binary, idx.0, FUNCTION_HANDLE_INDEX_MAX)
}

fn serialize_field_handle_index(binary: &mut BinaryData, idx: &FieldHandleIndex) -> Result<()> {
    write_as_uleb128(binary, idx.0, FIELD_HANDLE_INDEX_MAX)
}

fn serialize_field_inst_index(
    binary: &mut BinaryData,
    idx: &FieldInstantiationIndex,
) -> Result<()> {
    write_as_uleb128(binary, idx.0, FIELD_INST_INDEX_MAX)
}

fn serialize_function_inst_index(
    binary: &mut BinaryData,
    idx: &FunctionInstantiationIndex,
) -> Result<()> {
    write_as_uleb128(binary, idx.0, FUNCTION_INST_INDEX_MAX)
}

fn serialize_struct_def_inst_index(
    binary: &mut BinaryData,
    idx: &StructDefInstantiationIndex,
) -> Result<()> {
    write_as_uleb128(binary, idx.0, STRUCT_DEF_INST_INDEX_MAX)
}

fn seiralize_table_offset(binary: &mut BinaryData, offset: u32) -> Result<()> {
    write_as_uleb128(binary, offset, TABLE_OFFSET_MAX)
}

fn serialize_table_size(binary: &mut BinaryData, size: u32) -> Result<()> {
    write_as_uleb128(binary, size, TABLE_SIZE_MAX)
}

fn serialize_constant_pool_index(binary: &mut BinaryData, idx: &ConstantPoolIndex) -> Result<()> {
    write_as_uleb128(binary, idx.0, CONSTANT_INDEX_MAX)
}

fn serialize_bytecode_count(binary: &mut BinaryData, len: usize) -> Result<()> {
    write_as_uleb128(binary, len as u64, BYTECODE_COUNT_MAX)
}

fn serialize_identifier_size(binary: &mut BinaryData, len: usize) -> Result<()> {
    write_as_uleb128(binary, len as u64, IDENTIFIER_SIZE_MAX)
}

fn serialize_constant_size(binary: &mut BinaryData, len: usize) -> Result<()> {
    write_as_uleb128(binary, len as u64, CONSTANT_SIZE_MAX)
}

fn serialize_field_count(binary: &mut BinaryData, len: usize) -> Result<()> {
    write_as_uleb128(binary, len as u64, FIELD_COUNT_MAX)
}

fn serialize_field_offset(binary: &mut BinaryData, offset: u16) -> Result<()> {
    write_as_uleb128(binary, offset, FIELD_OFFSET_MAX)
}

fn serialize_acquires_count(binary: &mut BinaryData, len: usize) -> Result<()> {
    write_as_uleb128(binary, len as u64, ACQUIRES_COUNT_MAX)
}

fn serialize_signature_size(binary: &mut BinaryData, len: usize) -> Result<()> {
    write_as_uleb128(binary, len as u64, SIGNATURE_SIZE_MAX)
}

fn serialize_type_parameter_index(binary: &mut BinaryData, idx: u16) -> Result<()> {
    write_as_uleb128(binary, idx, TYPE_PARAMETER_INDEX_MAX)
}

fn serialize_type_parameter_count(binary: &mut BinaryData, len: usize) -> Result<()> {
    write_as_uleb128(binary, len as u64, TYPE_PARAMETER_COUNT_MAX)
}

fn serialize_bytecode_offset(binary: &mut BinaryData, offset: u16) -> Result<()> {
    write_as_uleb128(binary, offset, BYTECODE_INDEX_MAX)
}

fn serialize_table_count(binary: &mut BinaryData, len: u8) -> Result<()> {
    write_as_uleb128(binary, len, TABLE_COUNT_MAX)
}

fn serialize_local_index(binary: &mut BinaryData, idx: u8) -> Result<()> {
    write_as_uleb128(binary, idx, LOCAL_INDEX_MAX)
}

impl CompiledScriptMut {
    /// Serializes this into a binary format.
    ///
    /// This is intended mainly for test code. Production code will typically use
    /// [`CompiledScript::serialize`].
    pub fn serialize(&self, binary: &mut Vec<u8>) -> Result<()> {
        let mut binary_data = BinaryData::from(binary.clone());
        let mut ser = ScriptSerializer::new(VERSION_MAX);
        let mut temp = BinaryData::new();

        ser.common.serialize_common_tables(&mut temp, self)?;
        if temp.len() > TABLE_CONTENT_SIZE_MAX as usize {
            bail!(
                "table content size ({}) cannot exceed ({})",
                temp.len(),
                TABLE_CONTENT_SIZE_MAX
            );
        }
        ser.common.serialize_header(&mut binary_data)?;
        ser.common.serialize_table_indices(&mut binary_data)?;

        binary_data.extend(temp.as_inner())?;

        ser.serialize_main(&mut binary_data, self)?;

        *binary = binary_data.into_inner();
        Ok(())
    }
}

impl CompiledModule {
    /// Serializes a `CompiledModule` into a binary. The mutable `Vec<u8>` will contain the
    /// binary blob on return.
    pub fn serialize(&self, binary: &mut Vec<u8>) -> Result<()> {
        self.as_inner().serialize(binary)
    }
}

impl CompiledModuleMut {
    /// Serializes this into a binary format.
    ///
    /// This is intended mainly for test code. Production code will typically use
    /// [`CompiledModule::serialize`].
    pub fn serialize(&self, binary: &mut Vec<u8>) -> Result<()> {
        let mut binary_data = BinaryData::from(binary.clone());
        let mut ser = ModuleSerializer::new(VERSION_MAX);
        let mut temp = BinaryData::new();
        ser.serialize_tables(&mut temp, self)?;
        if temp.len() > u32::max_value() as usize {
            bail!(
                "table content size ({}) cannot exceed ({})",
                temp.len(),
                u32::max_value()
            );
        }
        ser.common.serialize_header(&mut binary_data)?;
        ser.serialize_table_indices(&mut binary_data)?;

        binary_data.extend(temp.as_inner())?;

        serialize_module_handle_index(&mut binary_data, &self.self_module_handle_idx)?;

        *binary = binary_data.into_inner();
        Ok(())
    }
}

/// Holds data to compute the header of a generic binary.
///
/// A binary header contains information about the tables serialized.
/// The serializer needs to serialize the tables in order to compute the offset and size
/// of each table.
/// `CommonSerializer` keeps track of the tables common to `CompiledScript` and
/// `CompiledModule`.
#[derive(Debug)]
struct CommonSerializer {
    major_version: u32,
    table_count: u8,
    module_handles: (u32, u32),
    struct_handles: (u32, u32),
    function_handles: (u32, u32),
    function_instantiations: (u32, u32),
    signatures: (u32, u32),
    identifiers: (u32, u32),
    address_identifiers: (u32, u32),
    constant_pool: (u32, u32),
}

/// Holds data to compute the header of a module binary.
#[derive(Debug)]
struct ModuleSerializer {
    common: CommonSerializer,
    struct_defs: (u32, u32),
    struct_def_instantiations: (u32, u32),
    function_defs: (u32, u32),
    field_handles: (u32, u32),
    field_instantiations: (u32, u32),
    friend_decls: (u32, u32),
}

/// Holds data to compute the header of a transaction script binary.
#[derive(Debug)]
struct ScriptSerializer {
    common: CommonSerializer,
}

//
// Helpers
//
fn check_index_in_binary(index: usize) -> Result<u32> {
    if index > u32::max_value() as usize {
        bail!(
            "Compilation unit too big ({}) cannot exceed {}",
            index,
            u32::max_value()
        )
    }
    Ok(index as u32)
}

fn serialize_table_index(
    binary: &mut BinaryData,
    kind: TableType,
    offset: u32,
    count: u32,
) -> Result<()> {
    if count != 0 {
        binary.push(kind as u8)?;
        seiralize_table_offset(binary, offset)?;
        serialize_table_size(binary, count)?;
    }
    Ok(())
}

fn serialize_magic(binary: &mut BinaryData) -> Result<()> {
    for byte in &BinaryConstants::DIEM_MAGIC {
        binary.push(*byte)?;
    }
    Ok(())
}

/// Trait to access tables for both `CompiledScript` and `CompiledModule`,
/// used by `CommonSerializer`.
trait CommonTables {
    fn get_module_handles(&self) -> &[ModuleHandle];
    fn get_struct_handles(&self) -> &[StructHandle];
    fn get_function_handles(&self) -> &[FunctionHandle];
    fn get_function_instantiations(&self) -> &[FunctionInstantiation];
    fn get_identifiers(&self) -> &[Identifier];
    fn get_address_identifiers(&self) -> &[AccountAddress];
    fn get_constant_pool(&self) -> &[Constant];
    fn get_signatures(&self) -> &[Signature];
}

impl CommonTables for CompiledScriptMut {
    fn get_module_handles(&self) -> &[ModuleHandle] {
        &self.module_handles
    }

    fn get_struct_handles(&self) -> &[StructHandle] {
        &self.struct_handles
    }

    fn get_function_handles(&self) -> &[FunctionHandle] {
        &self.function_handles
    }

    fn get_function_instantiations(&self) -> &[FunctionInstantiation] {
        &self.function_instantiations
    }

    fn get_identifiers(&self) -> &[Identifier] {
        &self.identifiers
    }

    fn get_address_identifiers(&self) -> &[AccountAddress] {
        &self.address_identifiers
    }

    fn get_constant_pool(&self) -> &[Constant] {
        &self.constant_pool
    }

    fn get_signatures(&self) -> &[Signature] {
        &self.signatures
    }
}

impl CommonTables for CompiledModuleMut {
    fn get_module_handles(&self) -> &[ModuleHandle] {
        &self.module_handles
    }

    fn get_struct_handles(&self) -> &[StructHandle] {
        &self.struct_handles
    }

    fn get_function_handles(&self) -> &[FunctionHandle] {
        &self.function_handles
    }

    fn get_function_instantiations(&self) -> &[FunctionInstantiation] {
        &self.function_instantiations
    }

    fn get_identifiers(&self) -> &[Identifier] {
        &self.identifiers
    }

    fn get_address_identifiers(&self) -> &[AccountAddress] {
        &self.address_identifiers
    }

    fn get_constant_pool(&self) -> &[Constant] {
        &self.constant_pool
    }

    fn get_signatures(&self) -> &[Signature] {
        &self.signatures
    }
}

/// Serializes a `ModuleHandle`.
///
/// A `ModuleHandle` gets serialized as follows:
/// - `ModuleHandle.address` as a ULEB128 (index into the `AddressPool`)
/// - `ModuleHandle.name` as a ULEB128 (index into the `IdentifierPool`)
fn serialize_module_handle(binary: &mut BinaryData, module_handle: &ModuleHandle) -> Result<()> {
    serialize_address_identifier_index(binary, &module_handle.address)?;
    serialize_identifier_index(binary, &module_handle.name)?;
    Ok(())
}

/// Serializes a `StructHandle`.
///
/// A `StructHandle` gets serialized as follows:
/// - `StructHandle.module` as a ULEB128 (index into the `ModuleHandle` table)
/// - `StructHandle.name` as a ULEB128 (index into the `IdentifierPool`)
/// - `StructHandle.is_nominal_resource` as a 1 byte boolean (0 for false, 1 for true)
fn serialize_struct_handle(binary: &mut BinaryData, struct_handle: &StructHandle) -> Result<()> {
    serialize_module_handle_index(binary, &struct_handle.module)?;
    serialize_identifier_index(binary, &struct_handle.name)?;
    serialize_nominal_resource_flag(binary, struct_handle.is_nominal_resource)?;
    serialize_kinds(binary, &struct_handle.type_parameters)
}

/// Serializes a `FunctionHandle`.
///
/// A `FunctionHandle` gets serialized as follows:
/// - `FunctionHandle.module` as a ULEB128 (index into the `ModuleHandle` table)
/// - `FunctionHandle.name` as a ULEB128 (index into the `IdentifierPool`)
/// - `FunctionHandle.parameters` as a ULEB128 (index into the `SignaturePool`)
/// - `FunctionHandle.return_` as a ULEB128 (index into the `SignaturePool`)
/// - `FunctionHandle.type_parameters` as a `Vec<u8>`
fn serialize_function_handle(
    binary: &mut BinaryData,
    function_handle: &FunctionHandle,
) -> Result<()> {
    serialize_module_handle_index(binary, &function_handle.module)?;
    serialize_identifier_index(binary, &function_handle.name)?;
    serialize_signature_index(binary, &function_handle.parameters)?;
    serialize_signature_index(binary, &function_handle.return_)?;
    serialize_kinds(binary, &function_handle.type_parameters)
}

fn serialize_function_instantiation(
    binary: &mut BinaryData,
    func_inst: &FunctionInstantiation,
) -> Result<()> {
    serialize_function_handle_index(binary, &func_inst.handle)?;
    serialize_signature_index(binary, &func_inst.type_parameters)?;
    Ok(())
}

/// Serializes a string (identifier or user string).
///
/// A `String` gets serialized as follows:
/// - `String` size as a ULEB128
/// - `String` bytes - *exact format to be defined, Rust utf8 right now*
fn serialize_identifier(binary: &mut BinaryData, string: &str) -> Result<()> {
    let bytes = string.as_bytes();
    serialize_identifier_size(binary, bytes.len())?;
    for byte in bytes {
        binary.push(*byte)?;
    }
    Ok(())
}

/// Serializes an `AccountAddress`.
///
/// A `AccountAddress` gets serialized as follows:
/// - 32 bytes in increasing index order
fn serialize_address(binary: &mut BinaryData, address: &AccountAddress) -> Result<()> {
    for byte in address.as_ref() {
        binary.push(*byte)?;
    }
    Ok(())
}

/// Serializes a `Constant`.
///
/// A `Constant` gets serialized as follows:
/// - `type_` serialized (see `serialize_signature_token`)
/// - `data` size as a ULEB128
/// - `data` bytes in increasing index order
fn serialize_constant(binary: &mut BinaryData, constant: &Constant) -> Result<()> {
    serialize_signature_token(binary, &constant.type_)?;
    serialize_constant_size(binary, constant.data.len())?;
    for byte in &constant.data {
        binary.push(*byte)?;
    }

    Ok(())
}

/// Serializes a `StructDefinition`.
///
/// A `StructDefinition` gets serialized as follows:
/// - `StructDefinition.handle` as a ULEB128 (index into the `ModuleHandle` table)
/// - `StructDefinition.field_count` as a ULEB128 (number of fields defined in the type)
/// - `StructDefinition.fields` as a ULEB128 (index into the `FieldDefinition` table)
fn serialize_struct_definition(
    binary: &mut BinaryData,
    struct_definition: &StructDefinition,
) -> Result<()> {
    serialize_struct_handle_index(binary, &struct_definition.struct_handle)?;
    match &struct_definition.field_information {
        StructFieldInformation::Native => binary.push(SerializedNativeStructFlag::NATIVE as u8),
        StructFieldInformation::Declared(fields) => {
            binary.push(SerializedNativeStructFlag::DECLARED as u8)?;
            serialize_field_definitions(binary, fields)
        }
    }
}

fn serialize_struct_def_instantiation(
    binary: &mut BinaryData,
    struct_inst: &StructDefInstantiation,
) -> Result<()> {
    serialize_struct_def_index(binary, &struct_inst.def)?;
    serialize_signature_index(binary, &struct_inst.type_parameters)?;
    Ok(())
}

/// Serializes `FieldDefinition` within a struct.
fn serialize_field_definitions(binary: &mut BinaryData, fields: &[FieldDefinition]) -> Result<()> {
    serialize_field_count(binary, fields.len())?;
    for field_definition in fields {
        serialize_field_definition(binary, field_definition)?;
    }
    Ok(())
}

/// Serializes a `FieldDefinition`.
///
/// A `FieldDefinition` gets serialized as follows:
/// - `FieldDefinition.struct_` as a ULEB128 (index into the `StructHandle` table)
/// - `StructDefinition.name` as a ULEB128 (index into the `IdentifierPool` table)
/// - `StructDefinition.signature` a serialized `TypeSignatureToekn`)
fn serialize_field_definition(
    binary: &mut BinaryData,
    field_definition: &FieldDefinition,
) -> Result<()> {
    serialize_identifier_index(binary, &field_definition.name)?;
    serialize_signature_token(binary, &field_definition.signature.0)
}

/// Serializes a `FunctionDefinition`.
///
/// A `FunctionDefinition` gets serialized as follows:
/// - `FunctionDefinition.function` as a ULEB128 (index into the `FunctionHandle` table)
/// - `FunctionDefinition.visibility` 1 byte for the visibility modifier of the function
/// - `FunctionDefinition.flags` 1 byte for the flags of the function
///   The flags now has only one bit used:
///   - bit 0x2: native indicator, indicates whether the function is a native function.
/// - `FunctionDefinition.code` a variable size stream for the `CodeUnit`
fn serialize_function_definition(
    binary: &mut BinaryData,
    function_definition: &FunctionDefinition,
) -> Result<()> {
    serialize_function_handle_index(binary, &function_definition.function)?;

    binary.push(function_definition.visibility as u8)?;
    let is_native = if function_definition.is_native() {
        FunctionDefinition::NATIVE
    } else {
        0
    };
    binary.push(is_native)?;

    serialize_acquires(binary, &function_definition.acquires_global_resources)?;
    if let Some(code) = &function_definition.code {
        serialize_code_unit(binary, code)?;
    }
    Ok(())
}

fn serialize_field_handle(binary: &mut BinaryData, field_handle: &FieldHandle) -> Result<()> {
    serialize_struct_def_index(binary, &field_handle.owner)?;
    serialize_field_offset(binary, field_handle.field)?;
    Ok(())
}

fn serialize_field_instantiation(
    binary: &mut BinaryData,
    field_inst: &FieldInstantiation,
) -> Result<()> {
    serialize_field_handle_index(binary, &field_inst.handle)?;
    serialize_signature_index(binary, &field_inst.type_parameters)?;
    Ok(())
}

/// Serializes a `FriendDeclaration`.
///
/// A `FriendDeclaration` gets serialized as follows:
/// - `FriendDeclaration.module` as a ULEB128 (index into the `ModuleHandle` table)
fn serialize_friend_declaration(
    binary: &mut BinaryData,
    friend_declaration: &FriendDeclaration,
) -> Result<()> {
    serialize_module_handle_index(binary, &friend_declaration.module)
}

/// Serializes a `Vec<StructDefinitionIndex>`.
fn serialize_acquires(binary: &mut BinaryData, indices: &[StructDefinitionIndex]) -> Result<()> {
    serialize_acquires_count(binary, indices.len())?;
    for def_idx in indices {
        serialize_struct_def_index(binary, def_idx)?;
    }
    Ok(())
}

/// Serializes a `Signature`.
///
/// A `Signature` gets serialized as follows the vector of `SignatureToken`s for locals
fn serialize_signature(binary: &mut BinaryData, signature: &Signature) -> Result<()> {
    serialize_signature_tokens(binary, &signature.0)
}

/// Serializes a slice of `SignatureToken`s.
fn serialize_signature_tokens(binary: &mut BinaryData, tokens: &[SignatureToken]) -> Result<()> {
    serialize_signature_size(binary, tokens.len())?;
    for token in tokens {
        serialize_signature_token(binary, token)?;
    }
    Ok(())
}

fn serialize_signature_token_single_node_impl(
    binary: &mut BinaryData,
    token: &SignatureToken,
) -> Result<()> {
    match token {
        SignatureToken::Bool => binary.push(SerializedType::BOOL as u8)?,
        SignatureToken::U8 => binary.push(SerializedType::U8 as u8)?,
        SignatureToken::U64 => binary.push(SerializedType::U64 as u8)?,
        SignatureToken::U128 => binary.push(SerializedType::U128 as u8)?,
        SignatureToken::Address => binary.push(SerializedType::ADDRESS as u8)?,
        SignatureToken::Signer => binary.push(SerializedType::SIGNER as u8)?,
        SignatureToken::Vector(_) => {
            binary.push(SerializedType::VECTOR as u8)?;
        }
        SignatureToken::Struct(idx) => {
            binary.push(SerializedType::STRUCT as u8)?;
            serialize_struct_handle_index(binary, idx)?;
        }
        SignatureToken::StructInstantiation(idx, type_params) => {
            binary.push(SerializedType::STRUCT_INST as u8)?;
            serialize_struct_handle_index(binary, idx)?;
            serialize_signature_size(binary, type_params.len())?;
        }
        SignatureToken::Reference(_) => {
            binary.push(SerializedType::REFERENCE as u8)?;
        }
        SignatureToken::MutableReference(_) => {
            binary.push(SerializedType::MUTABLE_REFERENCE as u8)?;
        }
        SignatureToken::TypeParameter(idx) => {
            binary.push(SerializedType::TYPE_PARAMETER as u8)?;
            serialize_type_parameter_index(binary, *idx)?;
        }
    }
    Ok(())
}

#[cfg(test)]
pub(crate) fn serialize_signature_token_unchecked(
    binary: &mut BinaryData,
    token: &SignatureToken,
) -> Result<()> {
    for token in token.preorder_traversal() {
        serialize_signature_token_single_node_impl(binary, token)?;
    }
    Ok(())
}

/// Serializes a `SignatureToken`.
///
/// A `SignatureToken` gets serialized as a variable size blob depending on composition.
/// Values for types are defined in `SerializedType`.
pub(crate) fn serialize_signature_token(
    binary: &mut BinaryData,
    token: &SignatureToken,
) -> Result<()> {
    // Non-recursive implementation to avoid overflowing the stack.
    for (token, depth) in token.preorder_traversal_with_depth() {
        if depth > SIGNATURE_TOKEN_DEPTH_MAX {
            bail!("max recursion depth reached")
        }
        serialize_signature_token_single_node_impl(binary, token)?;
    }
    Ok(())
}

fn serialize_nominal_resource_flag(
    binary: &mut BinaryData,
    is_nominal_resource: bool,
) -> Result<()> {
    binary.push(if is_nominal_resource {
        SerializedNominalResourceFlag::NOMINAL_RESOURCE
    } else {
        SerializedNominalResourceFlag::NORMAL_STRUCT
    } as u8)?;
    Ok(())
}

fn serialize_kind(binary: &mut BinaryData, kind: Kind) -> Result<()> {
    binary.push(match kind {
        Kind::All => SerializedKind::ALL,
        Kind::Resource => SerializedKind::RESOURCE,
        Kind::Copyable => SerializedKind::COPYABLE,
    } as u8)?;
    Ok(())
}

fn serialize_kinds(binary: &mut BinaryData, kinds: &[Kind]) -> Result<()> {
    serialize_type_parameter_count(binary, kinds.len())?;
    for kind in kinds {
        serialize_kind(binary, *kind)?;
    }
    Ok(())
}

/// Serializes a `CodeUnit`.
///
/// A `CodeUnit` is serialized as the code field of a `FunctionDefinition`.
/// A `CodeUnit` gets serialized as follows:
/// - `CodeUnit.max_stack_size` as a ULEB128
/// - `CodeUnit.locals` as a ULEB128 (index into the `LocalSignaturePool`)
/// - `CodeUnit.code` as variable size byte stream for the bytecode
fn serialize_code_unit(binary: &mut BinaryData, code: &CodeUnit) -> Result<()> {
    serialize_signature_index(binary, &code.locals)?;
    serialize_code(binary, &code.code)
}

/// Serializes a single `Bytecode` instruction.
fn serialize_instruction_inner(binary: &mut BinaryData, opcode: &Bytecode) -> Result<()> {
    let res = match opcode {
        Bytecode::FreezeRef => binary.push(Opcodes::FREEZE_REF as u8),
        Bytecode::Pop => binary.push(Opcodes::POP as u8),
        Bytecode::Ret => binary.push(Opcodes::RET as u8),
        Bytecode::BrTrue(code_offset) => {
            binary.push(Opcodes::BR_TRUE as u8)?;
            serialize_bytecode_offset(binary, *code_offset)
        }
        Bytecode::BrFalse(code_offset) => {
            binary.push(Opcodes::BR_FALSE as u8)?;
            serialize_bytecode_offset(binary, *code_offset)
        }
        Bytecode::Branch(code_offset) => {
            binary.push(Opcodes::BRANCH as u8)?;
            serialize_bytecode_offset(binary, *code_offset)
        }
        Bytecode::LdU8(value) => {
            binary.push(Opcodes::LD_U8 as u8)?;
            binary.push(*value)
        }
        Bytecode::LdU64(value) => {
            binary.push(Opcodes::LD_U64 as u8)?;
            write_u64(binary, *value)
        }
        Bytecode::LdU128(value) => {
            binary.push(Opcodes::LD_U128 as u8)?;
            write_u128(binary, *value)
        }
        Bytecode::CastU8 => binary.push(Opcodes::CAST_U8 as u8),
        Bytecode::CastU64 => binary.push(Opcodes::CAST_U64 as u8),
        Bytecode::CastU128 => binary.push(Opcodes::CAST_U128 as u8),
        Bytecode::LdConst(const_idx) => {
            binary.push(Opcodes::LD_CONST as u8)?;
            serialize_constant_pool_index(binary, const_idx)
        }
        Bytecode::LdTrue => binary.push(Opcodes::LD_TRUE as u8),
        Bytecode::LdFalse => binary.push(Opcodes::LD_FALSE as u8),
        Bytecode::CopyLoc(local_idx) => {
            binary.push(Opcodes::COPY_LOC as u8)?;
            serialize_local_index(binary, *local_idx)
        }
        Bytecode::MoveLoc(local_idx) => {
            binary.push(Opcodes::MOVE_LOC as u8)?;
            serialize_local_index(binary, *local_idx)
        }
        Bytecode::StLoc(local_idx) => {
            binary.push(Opcodes::ST_LOC as u8)?;
            serialize_local_index(binary, *local_idx)
        }
        Bytecode::MutBorrowLoc(local_idx) => {
            binary.push(Opcodes::MUT_BORROW_LOC as u8)?;
            serialize_local_index(binary, *local_idx)
        }
        Bytecode::ImmBorrowLoc(local_idx) => {
            binary.push(Opcodes::IMM_BORROW_LOC as u8)?;
            serialize_local_index(binary, *local_idx)
        }
        Bytecode::MutBorrowField(field_idx) => {
            binary.push(Opcodes::MUT_BORROW_FIELD as u8)?;
            serialize_field_handle_index(binary, field_idx)
        }
        Bytecode::MutBorrowFieldGeneric(field_idx) => {
            binary.push(Opcodes::MUT_BORROW_FIELD_GENERIC as u8)?;
            serialize_field_inst_index(binary, field_idx)
        }
        Bytecode::ImmBorrowField(field_idx) => {
            binary.push(Opcodes::IMM_BORROW_FIELD as u8)?;
            serialize_field_handle_index(binary, field_idx)
        }
        Bytecode::ImmBorrowFieldGeneric(field_idx) => {
            binary.push(Opcodes::IMM_BORROW_FIELD_GENERIC as u8)?;
            serialize_field_inst_index(binary, field_idx)
        }
        Bytecode::Call(method_idx) => {
            binary.push(Opcodes::CALL as u8)?;
            serialize_function_handle_index(binary, method_idx)
        }
        Bytecode::Pack(class_idx) => {
            binary.push(Opcodes::PACK as u8)?;
            serialize_struct_def_index(binary, class_idx)
        }
        Bytecode::Unpack(class_idx) => {
            binary.push(Opcodes::UNPACK as u8)?;
            serialize_struct_def_index(binary, class_idx)
        }
        Bytecode::CallGeneric(method_idx) => {
            binary.push(Opcodes::CALL_GENERIC as u8)?;
            serialize_function_inst_index(binary, method_idx)
        }
        Bytecode::PackGeneric(class_idx) => {
            binary.push(Opcodes::PACK_GENERIC as u8)?;
            serialize_struct_def_inst_index(binary, class_idx)
        }
        Bytecode::UnpackGeneric(class_idx) => {
            binary.push(Opcodes::UNPACK_GENERIC as u8)?;
            serialize_struct_def_inst_index(binary, class_idx)
        }
        Bytecode::ReadRef => binary.push(Opcodes::READ_REF as u8),
        Bytecode::WriteRef => binary.push(Opcodes::WRITE_REF as u8),
        Bytecode::Add => binary.push(Opcodes::ADD as u8),
        Bytecode::Sub => binary.push(Opcodes::SUB as u8),
        Bytecode::Mul => binary.push(Opcodes::MUL as u8),
        Bytecode::Mod => binary.push(Opcodes::MOD as u8),
        Bytecode::Div => binary.push(Opcodes::DIV as u8),
        Bytecode::BitOr => binary.push(Opcodes::BIT_OR as u8),
        Bytecode::BitAnd => binary.push(Opcodes::BIT_AND as u8),
        Bytecode::Xor => binary.push(Opcodes::XOR as u8),
        Bytecode::Shl => binary.push(Opcodes::SHL as u8),
        Bytecode::Shr => binary.push(Opcodes::SHR as u8),
        Bytecode::Or => binary.push(Opcodes::OR as u8),
        Bytecode::And => binary.push(Opcodes::AND as u8),
        Bytecode::Not => binary.push(Opcodes::NOT as u8),
        Bytecode::Eq => binary.push(Opcodes::EQ as u8),
        Bytecode::Neq => binary.push(Opcodes::NEQ as u8),
        Bytecode::Lt => binary.push(Opcodes::LT as u8),
        Bytecode::Gt => binary.push(Opcodes::GT as u8),
        Bytecode::Le => binary.push(Opcodes::LE as u8),
        Bytecode::Ge => binary.push(Opcodes::GE as u8),
        Bytecode::Abort => binary.push(Opcodes::ABORT as u8),
        Bytecode::Nop => binary.push(Opcodes::NOP as u8),
        Bytecode::Exists(class_idx) => {
            binary.push(Opcodes::EXISTS as u8)?;
            serialize_struct_def_index(binary, class_idx)
        }
        Bytecode::MutBorrowGlobal(class_idx) => {
            binary.push(Opcodes::MUT_BORROW_GLOBAL as u8)?;
            serialize_struct_def_index(binary, class_idx)
        }
        Bytecode::ImmBorrowGlobal(class_idx) => {
            binary.push(Opcodes::IMM_BORROW_GLOBAL as u8)?;
            serialize_struct_def_index(binary, class_idx)
        }
        Bytecode::MoveFrom(class_idx) => {
            binary.push(Opcodes::MOVE_FROM as u8)?;
            serialize_struct_def_index(binary, class_idx)
        }
        Bytecode::MoveTo(class_idx) => {
            binary.push(Opcodes::MOVE_TO as u8)?;
            serialize_struct_def_index(binary, class_idx)
        }
        Bytecode::ExistsGeneric(class_idx) => {
            binary.push(Opcodes::EXISTS_GENERIC as u8)?;
            serialize_struct_def_inst_index(binary, class_idx)
        }
        Bytecode::MutBorrowGlobalGeneric(class_idx) => {
            binary.push(Opcodes::MUT_BORROW_GLOBAL_GENERIC as u8)?;
            serialize_struct_def_inst_index(binary, class_idx)
        }
        Bytecode::ImmBorrowGlobalGeneric(class_idx) => {
            binary.push(Opcodes::IMM_BORROW_GLOBAL_GENERIC as u8)?;
            serialize_struct_def_inst_index(binary, class_idx)
        }
        Bytecode::MoveFromGeneric(class_idx) => {
            binary.push(Opcodes::MOVE_FROM_GENERIC as u8)?;
            serialize_struct_def_inst_index(binary, class_idx)
        }
        Bytecode::MoveToGeneric(class_idx) => {
            binary.push(Opcodes::MOVE_TO_GENERIC as u8)?;
            serialize_struct_def_inst_index(binary, class_idx)
        }
    };
    res?;
    Ok(())
}

/// Serializes a `Bytecode` stream. Serialization of the function body.
fn serialize_code(binary: &mut BinaryData, code: &[Bytecode]) -> Result<()> {
    serialize_bytecode_count(binary, code.len())?;
    for opcode in code {
        serialize_instruction_inner(binary, opcode)?;
    }
    Ok(())
}

/// Compute the table size with a check for underflow
fn checked_calculate_table_size(binary: &mut BinaryData, start: u32) -> Result<u32> {
    let offset = check_index_in_binary(binary.len())?;
    checked_assume!(offset >= start, "table start must be before end");
    Ok(offset - start)
}

impl CommonSerializer {
    pub fn new(major_version: u32) -> CommonSerializer {
        CommonSerializer {
            major_version,
            table_count: 0,
            module_handles: (0, 0),
            struct_handles: (0, 0),
            function_handles: (0, 0),
            function_instantiations: (0, 0),
            signatures: (0, 0),
            identifiers: (0, 0),
            address_identifiers: (0, 0),
            constant_pool: (0, 0),
        }
    }

    fn serialize_header(&mut self, binary: &mut BinaryData) -> Result<()> {
        serialize_magic(binary)?;
        write_u32(binary, self.major_version)?;
        Ok(())
    }

    /// Common binary header serialization.
    fn serialize_table_indices(&mut self, binary: &mut BinaryData) -> Result<()> {
        serialize_table_count(binary, self.table_count)?;

        serialize_table_index(
            binary,
            TableType::MODULE_HANDLES,
            self.module_handles.0,
            self.module_handles.1,
        )?;
        serialize_table_index(
            binary,
            TableType::STRUCT_HANDLES,
            self.struct_handles.0,
            self.struct_handles.1,
        )?;
        serialize_table_index(
            binary,
            TableType::FUNCTION_HANDLES,
            self.function_handles.0,
            self.function_handles.1,
        )?;
        serialize_table_index(
            binary,
            TableType::FUNCTION_INST,
            self.function_instantiations.0,
            self.function_instantiations.1,
        )?;
        serialize_table_index(
            binary,
            TableType::SIGNATURES,
            self.signatures.0,
            self.signatures.1,
        )?;
        serialize_table_index(
            binary,
            TableType::IDENTIFIERS,
            self.identifiers.0,
            self.identifiers.1,
        )?;
        serialize_table_index(
            binary,
            TableType::ADDRESS_IDENTIFIERS,
            self.address_identifiers.0,
            self.address_identifiers.1,
        )?;
        serialize_table_index(
            binary,
            TableType::CONSTANT_POOL,
            self.constant_pool.0,
            self.constant_pool.1,
        )?;
        Ok(())
    }

    fn serialize_common_tables<T: CommonTables>(
        &mut self,
        binary: &mut BinaryData,
        tables: &T,
    ) -> Result<()> {
        verify!(self.table_count == 0); // Should not be necessary, but it helps MIRAI right now
        self.serialize_module_handles(binary, tables.get_module_handles())?;
        self.serialize_struct_handles(binary, tables.get_struct_handles())?;
        self.serialize_function_handles(binary, tables.get_function_handles())?;
        verify!(self.table_count < 6); // Should not be necessary, but it helps MIRAI right now
        self.serialize_function_instantiations(binary, tables.get_function_instantiations())?;
        self.serialize_signatures(binary, tables.get_signatures())?;
        self.serialize_identifiers(binary, tables.get_identifiers())?;
        self.serialize_address_identifiers(binary, tables.get_address_identifiers())?;
        self.serialize_constants(binary, tables.get_constant_pool())?;
        Ok(())
    }

    /// Serializes `ModuleHandle` table.
    fn serialize_module_handles(
        &mut self,
        binary: &mut BinaryData,
        module_handles: &[ModuleHandle],
    ) -> Result<()> {
        if !module_handles.is_empty() {
            self.table_count += 1;
            self.module_handles.0 = check_index_in_binary(binary.len())?;
            for module_handle in module_handles {
                serialize_module_handle(binary, module_handle)?;
            }
            self.module_handles.1 = checked_calculate_table_size(binary, self.module_handles.0)?;
        }
        Ok(())
    }

    /// Serializes `StructHandle` table.
    fn serialize_struct_handles(
        &mut self,
        binary: &mut BinaryData,
        struct_handles: &[StructHandle],
    ) -> Result<()> {
        if !struct_handles.is_empty() {
            self.table_count += 1;
            self.struct_handles.0 = check_index_in_binary(binary.len())?;
            for struct_handle in struct_handles {
                serialize_struct_handle(binary, struct_handle)?;
            }
            self.struct_handles.1 = checked_calculate_table_size(binary, self.struct_handles.0)?;
        }
        Ok(())
    }

    /// Serializes `FunctionHandle` table.
    fn serialize_function_handles(
        &mut self,
        binary: &mut BinaryData,
        function_handles: &[FunctionHandle],
    ) -> Result<()> {
        if !function_handles.is_empty() {
            self.table_count += 1;
            self.function_handles.0 = check_index_in_binary(binary.len())?;
            for function_handle in function_handles {
                serialize_function_handle(binary, function_handle)?;
            }
            self.function_handles.1 =
                checked_calculate_table_size(binary, self.function_handles.0)?;
        }
        Ok(())
    }

    /// Serializes `FunctionInstantiation` table.
    fn serialize_function_instantiations(
        &mut self,
        binary: &mut BinaryData,
        function_instantiations: &[FunctionInstantiation],
    ) -> Result<()> {
        if !function_instantiations.is_empty() {
            self.table_count += 1;
            self.function_instantiations.0 = check_index_in_binary(binary.len())?;
            for function_instantiation in function_instantiations {
                serialize_function_instantiation(binary, function_instantiation)?;
            }
            self.function_instantiations.1 =
                checked_calculate_table_size(binary, self.function_instantiations.0)?;
        }
        Ok(())
    }

    /// Serializes `Identifiers`.
    fn serialize_identifiers(
        &mut self,
        binary: &mut BinaryData,
        identifiers: &[Identifier],
    ) -> Result<()> {
        if !identifiers.is_empty() {
            self.table_count += 1;
            self.identifiers.0 = check_index_in_binary(binary.len())?;
            for identifier in identifiers {
                // User strings and identifiers use the same serialization.
                serialize_identifier(binary, identifier.as_str())?;
            }
            self.identifiers.1 = checked_calculate_table_size(binary, self.identifiers.0)?;
        }
        Ok(())
    }

    /// Serializes `AddressIdentifiers`.
    fn serialize_address_identifiers(
        &mut self,
        binary: &mut BinaryData,
        addresses: &[AccountAddress],
    ) -> Result<()> {
        if !addresses.is_empty() {
            self.table_count += 1;
            self.address_identifiers.0 = check_index_in_binary(binary.len())?;
            for address in addresses {
                serialize_address(binary, address)?;
            }
            self.address_identifiers.1 =
                checked_calculate_table_size(binary, self.address_identifiers.0)?;
        }
        Ok(())
    }

    /// Serializes `ConstantPool`.
    fn serialize_constants(
        &mut self,
        binary: &mut BinaryData,
        constants: &[Constant],
    ) -> Result<()> {
        if !constants.is_empty() {
            self.table_count += 1;
            self.constant_pool.0 = check_index_in_binary(binary.len())?;
            for constant in constants {
                serialize_constant(binary, constant)?;
            }
            self.constant_pool.1 = checked_calculate_table_size(binary, self.constant_pool.0)?;
        }
        Ok(())
    }

    /// Serializes `SignaturePool` table.
    fn serialize_signatures(
        &mut self,
        binary: &mut BinaryData,
        signatures: &[Signature],
    ) -> Result<()> {
        if !signatures.is_empty() {
            self.table_count += 1;
            self.signatures.0 = check_index_in_binary(binary.len())?;
            for signature in signatures {
                serialize_signature(binary, signature)?;
            }
            self.signatures.1 = checked_calculate_table_size(binary, self.signatures.0)?;
        }
        Ok(())
    }
}

impl ModuleSerializer {
    fn new(major_version: u32) -> ModuleSerializer {
        ModuleSerializer {
            common: CommonSerializer::new(major_version),
            struct_defs: (0, 0),
            struct_def_instantiations: (0, 0),
            function_defs: (0, 0),
            field_handles: (0, 0),
            field_instantiations: (0, 0),
            friend_decls: (0, 0),
        }
    }

    fn serialize_tables(
        &mut self,
        binary: &mut BinaryData,
        module: &CompiledModuleMut,
    ) -> Result<()> {
        self.common.serialize_common_tables(binary, module)?;
        self.serialize_struct_definitions(binary, &module.struct_defs)?;
        self.serialize_struct_def_instantiations(binary, &module.struct_def_instantiations)?;
        self.serialize_function_definitions(binary, &module.function_defs)?;
        self.serialize_field_handles(binary, &module.field_handles)?;
        self.serialize_field_instantiations(binary, &module.field_instantiations)?;
        self.serialize_friend_declarations(binary, &module.friend_decls)
    }

    fn serialize_table_indices(&mut self, binary: &mut BinaryData) -> Result<()> {
        self.common.serialize_table_indices(binary)?;
        serialize_table_index(
            binary,
            TableType::STRUCT_DEFS,
            self.struct_defs.0,
            self.struct_defs.1,
        )?;
        serialize_table_index(
            binary,
            TableType::STRUCT_DEF_INST,
            self.struct_def_instantiations.0,
            self.struct_def_instantiations.1,
        )?;
        serialize_table_index(
            binary,
            TableType::FUNCTION_DEFS,
            self.function_defs.0,
            self.function_defs.1,
        )?;
        serialize_table_index(
            binary,
            TableType::FIELD_HANDLE,
            self.field_handles.0,
            self.field_handles.1,
        )?;
        serialize_table_index(
            binary,
            TableType::FIELD_INST,
            self.field_instantiations.0,
            self.field_instantiations.1,
        )?;
        serialize_table_index(
            binary,
            TableType::FRIEND_DECLS,
            self.friend_decls.0,
            self.friend_decls.1,
        )?;
        Ok(())
    }

    /// Serializes `StructDefinition` table.
    fn serialize_struct_definitions(
        &mut self,
        binary: &mut BinaryData,
        struct_definitions: &[StructDefinition],
    ) -> Result<()> {
        if !struct_definitions.is_empty() {
            self.common.table_count = self.common.table_count.wrapping_add(1); // the count will bound to a small number
            self.struct_defs.0 = check_index_in_binary(binary.len())?;
            for struct_definition in struct_definitions {
                serialize_struct_definition(binary, struct_definition)?;
            }
            self.struct_defs.1 = checked_calculate_table_size(binary, self.struct_defs.0)?;
        }
        Ok(())
    }

    /// Serializes `StructInstantiation` table.
    fn serialize_struct_def_instantiations(
        &mut self,
        binary: &mut BinaryData,
        struct_def_instantiations: &[StructDefInstantiation],
    ) -> Result<()> {
        if !struct_def_instantiations.is_empty() {
            self.common.table_count = self.common.table_count.wrapping_add(1); // the count will bound to a small number
            self.struct_def_instantiations.0 = check_index_in_binary(binary.len())?;
            for struct_instantiation in struct_def_instantiations {
                serialize_struct_def_instantiation(binary, struct_instantiation)?;
            }
            self.struct_def_instantiations.1 =
                checked_calculate_table_size(binary, self.struct_def_instantiations.0)?;
        }
        Ok(())
    }

    /// Serializes `FunctionDefinition` table.
    fn serialize_field_handles(
        &mut self,
        binary: &mut BinaryData,
        field_handles: &[FieldHandle],
    ) -> Result<()> {
        if !field_handles.is_empty() {
            self.common.table_count += 1;
            self.field_handles.0 = check_index_in_binary(binary.len())?;
            for field_handle in field_handles {
                serialize_field_handle(binary, field_handle)?;
            }
            self.field_handles.1 = checked_calculate_table_size(binary, self.field_handles.0)?;
        }
        Ok(())
    }

    fn serialize_field_instantiations(
        &mut self,
        binary: &mut BinaryData,
        field_instantiations: &[FieldInstantiation],
    ) -> Result<()> {
        if !field_instantiations.is_empty() {
            self.common.table_count += 1;
            self.field_instantiations.0 = check_index_in_binary(binary.len())?;
            for field_instantiation in field_instantiations {
                serialize_field_instantiation(binary, field_instantiation)?;
            }
            self.field_instantiations.1 =
                checked_calculate_table_size(binary, self.field_instantiations.0)?;
        }
        Ok(())
    }

    fn serialize_function_definitions(
        &mut self,
        binary: &mut BinaryData,
        function_definitions: &[FunctionDefinition],
    ) -> Result<()> {
        if !function_definitions.is_empty() {
            self.common.table_count = self.common.table_count.wrapping_add(1); // the count will bound to a small number
            self.function_defs.0 = check_index_in_binary(binary.len())?;
            for function_definition in function_definitions {
                serialize_function_definition(binary, function_definition)?;
            }
            self.function_defs.1 = checked_calculate_table_size(binary, self.function_defs.0)?;
        }
        Ok(())
    }

    fn serialize_friend_declarations(
        &mut self,
        binary: &mut BinaryData,
        friend_declarations: &[FriendDeclaration],
    ) -> Result<()> {
        if !friend_declarations.is_empty() {
            self.common.table_count = self.common.table_count.wrapping_add(1); // the count will bound to a small number
            self.friend_decls.0 = check_index_in_binary(binary.len())?;
            for friend_declaration in friend_declarations {
                serialize_friend_declaration(binary, friend_declaration)?;
            }
            self.friend_decls.1 = checked_calculate_table_size(binary, self.friend_decls.0)?;
        }
        Ok(())
    }
}

impl ScriptSerializer {
    fn new(major_version: u32) -> ScriptSerializer {
        ScriptSerializer {
            common: CommonSerializer::new(major_version),
        }
    }

    /// Serializes the main function.
    fn serialize_main(
        &mut self,
        binary: &mut BinaryData,
        script: &CompiledScriptMut,
    ) -> Result<()> {
        serialize_kinds(binary, &script.type_parameters)?;
        serialize_signature_index(binary, &script.parameters)?;
        serialize_code_unit(binary, &script.code)?;
        Ok(())
    }
}
