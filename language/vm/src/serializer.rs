// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Serialization of transactions and modules.
//!
//! This module exposes two entry points for serialization of `CompiledScript` and
//! `CompiledModule`. The entry points are exposed on the main structs `CompiledScript` and
//! `CompiledModule`.

use crate::{file_format::*, file_format_common::*};
use anyhow::{bail, Result};
use libra_types::account_address::AccountAddress;
use move_core_types::identifier::Identifier;

impl CompiledScript {
    /// Serializes a `CompiledScript` into a binary. The mutable `Vec<u8>` will contain the
    /// binary blob on return.
    pub fn serialize(&self, binary: &mut Vec<u8>) -> Result<()> {
        self.as_inner().serialize(binary)
    }
}

impl CompiledScriptMut {
    /// Serializes this into a binary format.
    ///
    /// This is intended mainly for test code. Production code will typically use
    /// [`CompiledScript::serialize`].
    pub fn serialize(&self, binary: &mut Vec<u8>) -> Result<()> {
        let mut binary_data = BinaryData::from(binary.clone());
        let mut ser = ScriptSerializer::new(1, 0);
        let mut temp = BinaryData::new();
        ser.serialize(&mut temp, self)?;
        ser.serialize_header(&mut binary_data)?;
        binary_data.extend(temp.as_inner())?;
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
        let mut ser = ModuleSerializer::new(1, 0);
        let mut temp = BinaryData::new();
        ser.serialize(&mut temp, self)?;
        ser.serialize_header(&mut binary_data)?;
        binary_data.extend(temp.as_inner())?;
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
    major_version: u8,
    minor_version: u8,
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
}

/// Holds data to compute the header of a transaction script binary.
#[derive(Debug)]
struct ScriptSerializer {
    common: CommonSerializer,
    main: (u32, u32),
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

fn unchecked_serialize_table(
    binary: &mut BinaryData,
    kind: TableType,
    offset: u32,
    count: u32,
) -> Result<()> {
    if count != 0 {
        binary.push(kind as u8)?;
        write_u32(binary, offset)?;
        write_u32(binary, count)?;
    }
    Ok(())
}

fn checked_serialize_table(
    binary: &mut BinaryData,
    kind: TableType,
    start: u32,
    offset: u32,
    length: u32,
) -> Result<()> {
    if let Some(start_offset) = start.checked_add(offset) {
        unchecked_serialize_table(binary, kind, start_offset, length)?;
    } else {
        bail!(
            "binary size ({}) cannot exceed {}",
            binary.len(),
            usize::max_value(),
        );
    }
    Ok(())
}

fn serialize_magic(binary: &mut BinaryData) -> Result<()> {
    for byte in &BinaryConstants::LIBRA_MAGIC {
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
    write_u16_as_uleb128(binary, module_handle.address.0)?;
    write_u16_as_uleb128(binary, module_handle.name.0)?;
    Ok(())
}

/// Serializes a `StructHandle`.
///
/// A `StructHandle` gets serialized as follows:
/// - `StructHandle.module` as a ULEB128 (index into the `ModuleHandle` table)
/// - `StructHandle.name` as a ULEB128 (index into the `IdentifierPool`)
/// - `StructHandle.is_nominal_resource` as a 1 byte boolean (0 for false, 1 for true)
fn serialize_struct_handle(binary: &mut BinaryData, struct_handle: &StructHandle) -> Result<()> {
    write_u16_as_uleb128(binary, struct_handle.module.0)?;
    write_u16_as_uleb128(binary, struct_handle.name.0)?;
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
    write_u16_as_uleb128(binary, function_handle.module.0)?;
    write_u16_as_uleb128(binary, function_handle.name.0)?;
    write_u16_as_uleb128(binary, function_handle.parameters.0)?;
    write_u16_as_uleb128(binary, function_handle.return_.0)?;
    serialize_kinds(binary, &function_handle.type_parameters)
}

fn serialize_function_instantiation(
    binary: &mut BinaryData,
    func_inst: &FunctionInstantiation,
) -> Result<()> {
    write_u16_as_uleb128(binary, func_inst.handle.0)?;
    write_u16_as_uleb128(binary, func_inst.type_parameters.0)
}

/// Serializes a string (identifier or user string).
///
/// A `String` gets serialized as follows:
/// - `String` size as a ULEB128
/// - `String` bytes - *exact format to be defined, Rust utf8 right now*
fn serialize_string(binary: &mut BinaryData, string: &str) -> Result<()> {
    let bytes = string.as_bytes();
    let len = bytes.len();
    if len > u32::max_value() as usize {
        bail!("string size ({}) cannot exceed {}", len, u32::max_value())
    }
    write_u32_as_uleb128(binary, len as u32)?;
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

    let len = constant.data.len();
    if len > u32::max_value() as usize {
        bail!(
            "constant's value size ({}) cannot exceed {}",
            len,
            u32::max_value()
        )
    }
    write_u32_as_uleb128(binary, len as u32)?;
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
    write_u16_as_uleb128(binary, struct_definition.struct_handle.0)?;
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
    write_u16_as_uleb128(binary, struct_inst.def.0)?;
    write_u16_as_uleb128(binary, struct_inst.type_parameters.0)
}

/// Serializes `FieldDefinition` within a struct.
fn serialize_field_definitions(binary: &mut BinaryData, fields: &[FieldDefinition]) -> Result<()> {
    write_u32_as_uleb128(binary, fields.len() as u32)?;
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
    write_u16_as_uleb128(binary, field_definition.name.0)?;
    serialize_signature_token(binary, &field_definition.signature.0)
}

/// Serializes a `FunctionDefinition`.
///
/// A `FunctionDefinition` gets serialized as follows:
/// - `FunctionDefinition.function` as a ULEB128 (index into the `FunctionHandle` table)
/// - `FunctionDefinition.flags` 1 byte for the flags of the function
/// - `FunctionDefinition.code` a variable size stream for the `CodeUnit`
fn serialize_function_definition(
    binary: &mut BinaryData,
    function_definition: &FunctionDefinition,
) -> Result<()> {
    write_u16_as_uleb128(binary, function_definition.function.0)?;

    let is_public = if function_definition.is_public() {
        FunctionDefinition::PUBLIC
    } else {
        0
    };
    let is_native = if function_definition.is_native() {
        FunctionDefinition::NATIVE
    } else {
        0
    };
    binary.push(is_public | is_native)?;

    serialize_acquires(binary, &function_definition.acquires_global_resources)?;
    if let Some(code) = &function_definition.code {
        serialize_code_unit(binary, code)?;
    }
    Ok(())
}

fn serialize_field_handle(binary: &mut BinaryData, field_handle: &FieldHandle) -> Result<()> {
    write_u16_as_uleb128(binary, field_handle.owner.0)?;
    write_u16_as_uleb128(binary, field_handle.field)
}

fn serialize_field_instantiation(
    binary: &mut BinaryData,
    field_inst: &FieldInstantiation,
) -> Result<()> {
    write_u16_as_uleb128(binary, field_inst.handle.0)?;
    write_u16_as_uleb128(binary, field_inst.type_parameters.0)
}

/// Serializes a `Vec<StructDefinitionIndex>`.
fn serialize_acquires(binary: &mut BinaryData, indices: &[StructDefinitionIndex]) -> Result<()> {
    let len = indices.len();
    if len > u8::max_value() as usize {
        bail!(
            "acquires_global_resources size ({}) cannot exceed {}",
            len,
            u8::max_value(),
        )
    }
    binary.push(len as u8)?;
    for def_idx in indices {
        write_u16_as_uleb128(binary, def_idx.0)?;
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
    let len = tokens.len();
    if len > u8::max_value() as usize {
        bail!(
            "arguments/locals size ({}) cannot exceed {}",
            len,
            u8::max_value(),
        )
    }
    binary.push(len as u8)?;
    for token in tokens {
        serialize_signature_token(binary, token)?;
    }
    Ok(())
}

/// Serializes a `SignatureToken`.
///
/// A `SignatureToken` gets serialized as a variable size blob depending on composition.
/// Values for types are defined in `SerializedType`.
fn serialize_signature_token(binary: &mut BinaryData, token: &SignatureToken) -> Result<()> {
    // Non-recursive implementation to avoid overflowing the stack.

    for token in token.preorder_traversal() {
        match token {
            SignatureToken::Bool => binary.push(SerializedType::BOOL as u8)?,
            SignatureToken::U8 => binary.push(SerializedType::U8 as u8)?,
            SignatureToken::U64 => binary.push(SerializedType::U64 as u8)?,
            SignatureToken::U128 => binary.push(SerializedType::U128 as u8)?,
            SignatureToken::Address => binary.push(SerializedType::ADDRESS as u8)?,
            SignatureToken::Vector(_) => {
                binary.push(SerializedType::VECTOR as u8)?;
            }
            SignatureToken::Struct(idx) => {
                binary.push(SerializedType::STRUCT as u8)?;
                write_u16_as_uleb128(binary, idx.0)?;
            }
            SignatureToken::StructInstantiation(idx, type_params) => {
                binary.push(SerializedType::STRUCT_INST as u8)?;
                write_u16_as_uleb128(binary, idx.0)?;
                let len = type_params.len();
                if len > u8::max_value() as usize {
                    bail!(
                        "arguments/locals size ({}) cannot exceed {}",
                        len,
                        u8::max_value(),
                    )
                }
                binary.push(len as u8)?;
            }
            SignatureToken::Reference(_) => {
                binary.push(SerializedType::REFERENCE as u8)?;
            }
            SignatureToken::MutableReference(_) => {
                binary.push(SerializedType::MUTABLE_REFERENCE as u8)?;
            }
            SignatureToken::TypeParameter(idx) => {
                binary.push(SerializedType::TYPE_PARAMETER as u8)?;
                write_u16_as_uleb128(binary, *idx)?;
            }
        }
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
    write_u32_as_uleb128(binary, kinds.len() as u32)?;
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
    write_u16_as_uleb128(binary, code.locals.0)?;
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
            write_u16(binary, *code_offset)
        }
        Bytecode::BrFalse(code_offset) => {
            binary.push(Opcodes::BR_FALSE as u8)?;
            write_u16(binary, *code_offset)
        }
        Bytecode::Branch(code_offset) => {
            binary.push(Opcodes::BRANCH as u8)?;
            write_u16(binary, *code_offset)
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
        Bytecode::LdConst(byte_array_idx) => {
            binary.push(Opcodes::LD_CONST as u8)?;
            write_u16_as_uleb128(binary, byte_array_idx.0)
        }
        Bytecode::LdTrue => binary.push(Opcodes::LD_TRUE as u8),
        Bytecode::LdFalse => binary.push(Opcodes::LD_FALSE as u8),
        Bytecode::CopyLoc(local_idx) => {
            binary.push(Opcodes::COPY_LOC as u8)?;
            binary.push(*local_idx)
        }
        Bytecode::MoveLoc(local_idx) => {
            binary.push(Opcodes::MOVE_LOC as u8)?;
            binary.push(*local_idx)
        }
        Bytecode::StLoc(local_idx) => {
            binary.push(Opcodes::ST_LOC as u8)?;
            binary.push(*local_idx)
        }
        Bytecode::MutBorrowLoc(local_idx) => {
            binary.push(Opcodes::MUT_BORROW_LOC as u8)?;
            binary.push(*local_idx)
        }
        Bytecode::ImmBorrowLoc(local_idx) => {
            binary.push(Opcodes::IMM_BORROW_LOC as u8)?;
            binary.push(*local_idx)
        }
        Bytecode::MutBorrowField(field_idx) => {
            binary.push(Opcodes::MUT_BORROW_FIELD as u8)?;
            write_u16_as_uleb128(binary, field_idx.0)
        }
        Bytecode::MutBorrowFieldGeneric(field_idx) => {
            binary.push(Opcodes::MUT_BORROW_FIELD_GENERIC as u8)?;
            write_u16_as_uleb128(binary, field_idx.0)
        }
        Bytecode::ImmBorrowField(field_idx) => {
            binary.push(Opcodes::IMM_BORROW_FIELD as u8)?;
            write_u16_as_uleb128(binary, field_idx.0)
        }
        Bytecode::ImmBorrowFieldGeneric(field_idx) => {
            binary.push(Opcodes::IMM_BORROW_FIELD_GENERIC as u8)?;
            write_u16_as_uleb128(binary, field_idx.0)
        }
        Bytecode::Call(method_idx) => {
            binary.push(Opcodes::CALL as u8)?;
            write_u16_as_uleb128(binary, method_idx.0)
        }
        Bytecode::Pack(class_idx) => {
            binary.push(Opcodes::PACK as u8)?;
            write_u16_as_uleb128(binary, class_idx.0)
        }
        Bytecode::Unpack(class_idx) => {
            binary.push(Opcodes::UNPACK as u8)?;
            write_u16_as_uleb128(binary, class_idx.0)
        }
        Bytecode::CallGeneric(method_idx) => {
            binary.push(Opcodes::CALL_GENERIC as u8)?;
            write_u16_as_uleb128(binary, method_idx.0)
        }
        Bytecode::PackGeneric(class_idx) => {
            binary.push(Opcodes::PACK_GENERIC as u8)?;
            write_u16_as_uleb128(binary, class_idx.0)
        }
        Bytecode::UnpackGeneric(class_idx) => {
            binary.push(Opcodes::UNPACK_GENERIC as u8)?;
            write_u16_as_uleb128(binary, class_idx.0)
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
        Bytecode::GetTxnSenderAddress => binary.push(Opcodes::GET_TXN_SENDER as u8),
        Bytecode::Exists(class_idx) => {
            binary.push(Opcodes::EXISTS as u8)?;
            write_u16_as_uleb128(binary, class_idx.0)
        }
        Bytecode::MutBorrowGlobal(class_idx) => {
            binary.push(Opcodes::MUT_BORROW_GLOBAL as u8)?;
            write_u16_as_uleb128(binary, class_idx.0)
        }
        Bytecode::ImmBorrowGlobal(class_idx) => {
            binary.push(Opcodes::IMM_BORROW_GLOBAL as u8)?;
            write_u16_as_uleb128(binary, class_idx.0)
        }
        Bytecode::MoveFrom(class_idx) => {
            binary.push(Opcodes::MOVE_FROM as u8)?;
            write_u16_as_uleb128(binary, class_idx.0)
        }
        Bytecode::MoveToSender(class_idx) => {
            binary.push(Opcodes::MOVE_TO as u8)?;
            write_u16_as_uleb128(binary, class_idx.0)
        }
        Bytecode::ExistsGeneric(class_idx) => {
            binary.push(Opcodes::EXISTS_GENERIC as u8)?;
            write_u16_as_uleb128(binary, class_idx.0)
        }
        Bytecode::MutBorrowGlobalGeneric(class_idx) => {
            binary.push(Opcodes::MUT_BORROW_GLOBAL_GENERIC as u8)?;
            write_u16_as_uleb128(binary, class_idx.0)
        }
        Bytecode::ImmBorrowGlobalGeneric(class_idx) => {
            binary.push(Opcodes::IMM_BORROW_GLOBAL_GENERIC as u8)?;
            write_u16_as_uleb128(binary, class_idx.0)
        }
        Bytecode::MoveFromGeneric(class_idx) => {
            binary.push(Opcodes::MOVE_FROM_GENERIC as u8)?;
            write_u16_as_uleb128(binary, class_idx.0)
        }
        Bytecode::MoveToSenderGeneric(class_idx) => {
            binary.push(Opcodes::MOVE_TO_GENERIC as u8)?;
            write_u16_as_uleb128(binary, class_idx.0)
        }
        Bytecode::Nop => binary.push(Opcodes::NOP as u8),
    };
    res?;
    Ok(())
}

/// Serializes a `Bytecode` stream. Serialization of the function body.
fn serialize_code(binary: &mut BinaryData, code: &[Bytecode]) -> Result<()> {
    let code_size = code.len();
    if code_size > u16::max_value() as usize {
        bail!(
            "code size ({}) cannot exceed {}",
            code_size,
            u16::max_value(),
        )
    }
    write_u16(binary, code_size as u16)?;
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
    pub fn new(major_version: u8, minor_version: u8) -> CommonSerializer {
        CommonSerializer {
            major_version,
            minor_version,
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

    /// Common binary header serialization.
    fn serialize_header(&mut self, binary: &mut BinaryData) -> Result<u32> {
        serialize_magic(binary)?;
        binary.push(self.major_version)?;
        binary.push(self.minor_version)?;
        binary.push(self.table_count)?;

        let start_offset;
        if let Some(table_count_op) = self
            .table_count
            .checked_mul(BinaryConstants::TABLE_HEADER_SIZE)
        {
            if let Some(checked_start_offset) =
                check_index_in_binary(binary.len())?.checked_add(u32::from(table_count_op))
            {
                start_offset = checked_start_offset;
            } else {
                bail!(
                    "binary size ({}) cannot exceed {}",
                    binary.len(),
                    usize::max_value()
                );
            }
        } else {
            bail!(
                "binary size ({}) cannot exceed {}",
                binary.len(),
                usize::max_value()
            );
        }

        checked_serialize_table(
            binary,
            TableType::MODULE_HANDLES,
            self.module_handles.0,
            start_offset,
            self.module_handles.1,
        )?;
        checked_serialize_table(
            binary,
            TableType::STRUCT_HANDLES,
            self.struct_handles.0,
            start_offset,
            self.struct_handles.1,
        )?;
        checked_serialize_table(
            binary,
            TableType::FUNCTION_HANDLES,
            self.function_handles.0,
            start_offset,
            self.function_handles.1,
        )?;
        checked_serialize_table(
            binary,
            TableType::FUNCTION_INST,
            self.function_instantiations.0,
            start_offset,
            self.function_instantiations.1,
        )?;
        checked_serialize_table(
            binary,
            TableType::SIGNATURES,
            self.signatures.0,
            start_offset,
            self.signatures.1,
        )?;
        checked_serialize_table(
            binary,
            TableType::IDENTIFIERS,
            self.identifiers.0,
            start_offset,
            self.identifiers.1,
        )?;
        checked_serialize_table(
            binary,
            TableType::ADDRESS_IDENTIFIERS,
            self.address_identifiers.0,
            start_offset,
            self.address_identifiers.1,
        )?;
        checked_serialize_table(
            binary,
            TableType::CONSTANT_POOL,
            self.constant_pool.0,
            start_offset,
            self.constant_pool.1,
        )?;
        Ok(start_offset)
    }

    fn serialize_common<T: CommonTables>(
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
                serialize_string(binary, identifier.as_str())?;
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
    fn new(major_version: u8, minor_version: u8) -> ModuleSerializer {
        ModuleSerializer {
            common: CommonSerializer::new(major_version, minor_version),
            struct_defs: (0, 0),
            struct_def_instantiations: (0, 0),
            function_defs: (0, 0),
            field_handles: (0, 0),
            field_instantiations: (0, 0),
        }
    }

    fn serialize(&mut self, binary: &mut BinaryData, module: &CompiledModuleMut) -> Result<()> {
        self.common.serialize_common(binary, module)?;
        self.serialize_struct_definitions(binary, &module.struct_defs)?;
        self.serialize_struct_def_instantiations(binary, &module.struct_def_instantiations)?;
        self.serialize_function_definitions(binary, &module.function_defs)?;
        self.serialize_field_handles(binary, &module.field_handles)?;
        self.serialize_field_instantiations(binary, &module.field_instantiations)
    }

    fn serialize_header(&mut self, binary: &mut BinaryData) -> Result<()> {
        let start_offset = self.common.serialize_header(binary)?;
        checked_serialize_table(
            binary,
            TableType::STRUCT_DEFS,
            self.struct_defs.0,
            start_offset,
            self.struct_defs.1,
        )?;
        checked_serialize_table(
            binary,
            TableType::STRUCT_DEF_INST,
            self.struct_def_instantiations.0,
            start_offset,
            self.struct_def_instantiations.1,
        )?;
        checked_serialize_table(
            binary,
            TableType::FUNCTION_DEFS,
            self.function_defs.0,
            start_offset,
            self.function_defs.1,
        )?;
        checked_serialize_table(
            binary,
            TableType::FIELD_HANDLE,
            self.field_handles.0,
            start_offset,
            self.field_handles.1,
        )?;
        checked_serialize_table(
            binary,
            TableType::FIELD_INST,
            self.field_instantiations.0,
            start_offset,
            self.field_instantiations.1,
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
}

impl ScriptSerializer {
    fn new(major_version: u8, minor_version: u8) -> ScriptSerializer {
        ScriptSerializer {
            common: CommonSerializer::new(major_version, minor_version),
            main: (0, 0),
        }
    }

    fn serialize(&mut self, binary: &mut BinaryData, script: &CompiledScriptMut) -> Result<()> {
        self.common.serialize_common(binary, script)?;
        self.serialize_main(binary, &script)
    }

    fn serialize_header(&mut self, binary: &mut BinaryData) -> Result<()> {
        let start_offset = self.common.serialize_header(binary)?;
        checked_serialize_table(
            binary,
            TableType::MAIN,
            self.main.0,
            start_offset,
            self.main.1,
        )?;
        Ok(())
    }

    /// Serializes the main function.
    fn serialize_main(
        &mut self,
        binary: &mut BinaryData,
        script: &CompiledScriptMut,
    ) -> Result<()> {
        self.common.table_count += 1;
        self.main.0 = check_index_in_binary(binary.len())?;
        serialize_kinds(binary, &script.type_parameters)?;
        write_u16_as_uleb128(binary, script.parameters.0)?;
        serialize_code_unit(binary, &script.code)?;
        self.main.1 = checked_calculate_table_size(binary, self.main.0)?;
        Ok(())
    }
}
