// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Serialization of transactions and modules.
//!
//! This module exposes two entry points for serialization of `CompiledScript` and
//! `CompiledModule`. The entry points are exposed on the main structs `CompiledScript` and
//! `CompiledModule`.

use crate::{file_format::*, file_format_common::*};
use failure::*;
use std::ops::Deref;
use types::{account_address::AccountAddress, byte_array::ByteArray};

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
        let mut ser = ScriptSerializer::new(1, 0);
        let mut temp: Vec<u8> = Vec::new();
        ser.serialize(&mut temp, self)?;
        ser.serialize_header(binary)?;
        binary.append(&mut temp);
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
        let mut ser = ModuleSerializer::new(1, 0);
        let mut temp: Vec<u8> = Vec::new();
        ser.serialize(&mut temp, self)?;
        ser.serialize_header(binary)?;
        binary.append(&mut temp);
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
    type_signatures: (u32, u32),
    function_signatures: (u32, u32),
    locals_signatures: (u32, u32),
    string_pool: (u32, u32),
    address_pool: (u32, u32),
    byte_array_pool: (u32, u32),
}

/// Holds data to compute the header of a module binary.
#[derive(Debug)]
struct ModuleSerializer {
    common: CommonSerializer,
    struct_defs: (u32, u32),
    field_defs: (u32, u32),
    function_defs: (u32, u32),
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

fn serialize_table(binary: &mut Vec<u8>, kind: TableType, offset: u32, count: u32) {
    if count != 0 {
        binary.push(kind as u8);
        write_u32(binary, offset);
        write_u32(binary, count);
    }
}

fn serialize_magic(binary: &mut Vec<u8>) {
    for byte in &BinaryConstants::LIBRA_MAGIC {
        binary.push(*byte);
    }
}

/// Trait to access tables for both `CompiledScript` and `CompiledModule`,
/// used by `CommonSerializer`.
trait CommonTables {
    fn get_module_handles(&self) -> &[ModuleHandle];
    fn get_struct_handles(&self) -> &[StructHandle];
    fn get_function_handles(&self) -> &[FunctionHandle];
    fn get_string_pool(&self) -> &[String];
    fn get_address_pool(&self) -> &[AccountAddress];
    fn get_byte_array_pool(&self) -> &[ByteArray];
    fn get_type_signatures(&self) -> &[TypeSignature];
    fn get_function_signatures(&self) -> &[FunctionSignature];
    fn get_locals_signatures(&self) -> &[LocalsSignature];
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

    fn get_string_pool(&self) -> &[String] {
        &self.string_pool
    }

    fn get_address_pool(&self) -> &[AccountAddress] {
        &self.address_pool
    }

    fn get_byte_array_pool(&self) -> &[ByteArray] {
        &self.byte_array_pool
    }

    fn get_type_signatures(&self) -> &[TypeSignature] {
        &self.type_signatures
    }

    fn get_function_signatures(&self) -> &[FunctionSignature] {
        &self.function_signatures
    }

    fn get_locals_signatures(&self) -> &[LocalsSignature] {
        &self.locals_signatures
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

    fn get_string_pool(&self) -> &[String] {
        &self.string_pool
    }

    fn get_address_pool(&self) -> &[AccountAddress] {
        &self.address_pool
    }

    fn get_byte_array_pool(&self) -> &[ByteArray] {
        &self.byte_array_pool
    }

    fn get_type_signatures(&self) -> &[TypeSignature] {
        &self.type_signatures
    }

    fn get_function_signatures(&self) -> &[FunctionSignature] {
        &self.function_signatures
    }

    fn get_locals_signatures(&self) -> &[LocalsSignature] {
        &self.locals_signatures
    }
}

/// Serializes a `ModuleHandle`.
///
/// A `ModuleHandle` gets serialized as follows:
/// - `ModuleHandle.address` as a ULEB128 (index into the `AddressPool`)
/// - `ModuleHandle.name` as a ULEB128 (index into the `StringPool`)
fn serialize_module_handle(binary: &mut Vec<u8>, module_handle: &ModuleHandle) {
    write_u16_as_uleb128(binary, module_handle.address.0);
    write_u16_as_uleb128(binary, module_handle.name.0);
}

/// Serializes a `StructHandle`.
///
/// A `StructHandle` gets serialized as follows:
/// - `StructHandle.module` as a ULEB128 (index into the `ModuleHandle` table)
/// - `StructHandle.name` as a ULEB128 (index into the `StringPool`)
/// - `StructHandle.is_resource` as a 1 byte boolean (0 for false, 1 for true)
fn serialize_struct_handle(binary: &mut Vec<u8>, struct_handle: &StructHandle) {
    write_u16_as_uleb128(binary, struct_handle.module.0);
    write_u16_as_uleb128(binary, struct_handle.name.0);
    if struct_handle.is_resource {
        binary.push(1);
    } else {
        binary.push(0);
    }
}

/// Serializes a `FunctionHandle`.
///
/// A `FunctionHandle` gets serialized as follows:
/// - `FunctionHandle.module` as a ULEB128 (index into the `ModuleHandle` table)
/// - `FunctionHandle.name` as a ULEB128 (index into the `StringPool`)
/// - `FunctionHandle.signature` as a ULEB128 (index into the `FunctionSignaturePool`)
fn serialize_function_handle(binary: &mut Vec<u8>, function_handle: &FunctionHandle) {
    write_u16_as_uleb128(binary, function_handle.module.0);
    write_u16_as_uleb128(binary, function_handle.name.0);
    write_u16_as_uleb128(binary, function_handle.signature.0);
}

/// Serializes a `String`.
///
/// A `String` gets serialized as follows:
/// - `String` size as a ULEB128
/// - `String` bytes - *exact format to be defined, Rust utf8 right now*
fn serialize_string(binary: &mut Vec<u8>, string: &str) -> Result<()> {
    let bytes = string.as_bytes();
    let len = bytes.len();
    if len > u32::max_value() as usize {
        bail!("string size ({}) cannot exceed {}", len, u32::max_value())
    }
    write_u32_as_uleb128(binary, len as u32);
    for byte in bytes {
        binary.push(*byte);
    }
    Ok(())
}

/// Serializes a `ByteArray`.
///
/// A `ByteArray` gets serialized as follows:
/// - `ByteArray` size as a ULEB128
/// - `ByteArray` bytes in increasing index order
fn serialize_byte_array(binary: &mut Vec<u8>, byte_array: &ByteArray) -> Result<()> {
    let bytes = byte_array.as_bytes();
    let len = bytes.len();
    if len > u32::max_value() as usize {
        bail!(
            "byte arrays size ({}) cannot exceed {}",
            len,
            u32::max_value()
        )
    }
    write_u32_as_uleb128(binary, len as u32);
    for byte in bytes {
        binary.push(*byte);
    }
    Ok(())
}

/// Serializes an `AccountAddress`.
///
/// A `AccountAddress` gets serialized as follows:
/// - 32 bytes in increasing index order
fn serialize_address(binary: &mut Vec<u8>, address: &AccountAddress) -> Result<()> {
    for byte in address.as_ref() {
        binary.push(*byte);
    }
    Ok(())
}

/// Serializes a `StructDefinition`.
///
/// A `StructDefinition` gets serialized as follows:
/// - `StructDefinition.handle` as a ULEB128 (index into the `ModuleHandle` table)
/// - `StructDefinition.field_count` as a ULEB128 (number of fields defined in the type)
/// - `StructDefinition.fields` as a ULEB128 (index into the `FieldDefinition` table)
fn serialize_struct_definition(binary: &mut Vec<u8>, struct_definition: &StructDefinition) {
    write_u16_as_uleb128(binary, struct_definition.struct_handle.0);
    write_u16_as_uleb128(binary, struct_definition.field_count);
    write_u16_as_uleb128(binary, struct_definition.fields.0);
}

/// Serializes a `FieldDefinition`.
///
/// A `FieldDefinition` gets serialized as follows:
/// - `FieldDefinition.struct_` as a ULEB128 (index into the `StructHandle` table)
/// - `StructDefinition.name` as a ULEB128 (index into the `StringPool` table)
/// - `StructDefinition.signature` as a ULEB128 (index into the `TypeSignaturePool`)
fn serialize_field_definition(binary: &mut Vec<u8>, field_definition: &FieldDefinition) {
    write_u16_as_uleb128(binary, field_definition.struct_.0);
    write_u16_as_uleb128(binary, field_definition.name.0);
    write_u16_as_uleb128(binary, field_definition.signature.0);
}

/// Serializes a `FunctionDefinition`.
///
/// A `FunctionDefinition` gets serialized as follows:
/// - `FunctionDefinition.function` as a ULEB128 (index into the `FunctionHandle` table)
/// - `StructDefinition.flags` 1 byte for the flags of the function
/// - `StructDefinition.code` a variable size stream for the `CodeUnit`
fn serialize_function_definition(
    binary: &mut Vec<u8>,
    function_definition: &FunctionDefinition,
) -> Result<()> {
    write_u16_as_uleb128(binary, function_definition.function.0);
    binary.push(function_definition.flags);
    serialize_code_unit(binary, &function_definition.code)
}

/// Serializes a `TypeSignature`.
///
/// A `TypeSignature` gets serialized as follows:
/// - `SignatureType::TYPE_SIGNATURE` as 1 byte
/// - The `SignatureToken` as a blob
fn serialize_type_signature(binary: &mut Vec<u8>, signature: &TypeSignature) -> Result<()> {
    binary.push(SignatureType::TYPE_SIGNATURE as u8);
    serialize_signature_token(binary, &signature.0)
}

/// Serializes a `FunctionSignature`.
///
/// A `FunctionSignature` gets serialized as follows:
/// - `SignatureType::FUNCTION_SIGNATURE` as 1 byte
/// - The vector of `SignatureToken`s for the return values
/// - The vector of `SignatureToken`s for the arguments
fn serialize_function_signature(binary: &mut Vec<u8>, signature: &FunctionSignature) -> Result<()> {
    binary.push(SignatureType::FUNCTION_SIGNATURE as u8);
    serialize_signature_tokens(binary, &signature.return_types)?;
    serialize_signature_tokens(binary, &signature.arg_types)
}

/// Serializes a `LocalsSignature`.
///
/// A `LocalsSignature` gets serialized as follows:
/// - `SignatureType::LOCAL_SIGNATURE` as 1 byte
/// - The vector of `SignatureToken`s for locals
fn serialize_locals_signature(binary: &mut Vec<u8>, signature: &LocalsSignature) -> Result<()> {
    binary.push(SignatureType::LOCAL_SIGNATURE as u8);
    serialize_signature_tokens(binary, &signature.0)
}

/// Serializes a slice of `SignatureToken`s.
fn serialize_signature_tokens(binary: &mut Vec<u8>, tokens: &[SignatureToken]) -> Result<()> {
    let len = tokens.len();
    if len > u8::max_value() as usize {
        bail!(
            "arguments/locals size ({}) cannot exceed {}",
            len,
            u8::max_value(),
        )
    }
    binary.push(len as u8);
    for token in tokens {
        serialize_signature_token(binary, token)?;
    }
    Ok(())
}

/// Serializes a `SignatureToken`.
///
/// A `SignatureToken` gets serialized as a variable size blob depending on composition.
/// Values for types are defined in `SerializedType`.
fn serialize_signature_token(binary: &mut Vec<u8>, token: &SignatureToken) -> Result<()> {
    match token {
        SignatureToken::Bool => binary.push(SerializedType::BOOL as u8),
        SignatureToken::U64 => binary.push(SerializedType::INTEGER as u8),
        SignatureToken::String => binary.push(SerializedType::STRING as u8),
        SignatureToken::ByteArray => binary.push(SerializedType::BYTEARRAY as u8),
        SignatureToken::Address => binary.push(SerializedType::ADDRESS as u8),
        SignatureToken::Struct(idx) => {
            binary.push(SerializedType::STRUCT as u8);
            write_u16_as_uleb128(binary, idx.0);
        }
        SignatureToken::Reference(boxed_token) => {
            binary.push(SerializedType::REFERENCE as u8);
            serialize_signature_token(binary, boxed_token.deref())?
        }
        SignatureToken::MutableReference(boxed_token) => {
            binary.push(SerializedType::MUTABLE_REFERENCE as u8);
            serialize_signature_token(binary, boxed_token.deref())?
        }
    }
    Ok(())
}

/// Serializes a `CodeUnit`.
///
/// A `CodeUnit` is serialized as the lst firld of a `FunctionDefinition`.
/// A `CodeUnit` gets serialized as follows:
/// - `CodeUnit.max_stack_size` as a ULEB128
/// - `CodeUnit.locals` as a ULEB128 (index into the `LocalSignaturePool`)
/// - `CodeUnit.code` as variable size byte stream for the bytecode
fn serialize_code_unit(binary: &mut Vec<u8>, code: &CodeUnit) -> Result<()> {
    write_u16_as_uleb128(binary, code.max_stack_size);
    write_u16_as_uleb128(binary, code.locals.0);
    serialize_code(binary, &code.code)
}

/// Serializes a single `Bytecode` instruction.
pub(crate) fn serialize_instruction(binary: &mut Vec<u8>, opcode: &Bytecode) {
    match opcode {
        Bytecode::FreezeRef => binary.push(Opcodes::FREEZE_REF as u8),
        Bytecode::Pop => binary.push(Opcodes::POP as u8),
        Bytecode::Ret => binary.push(Opcodes::RET as u8),
        Bytecode::BrTrue(code_offset) => {
            binary.push(Opcodes::BR_TRUE as u8);
            write_u16(binary, *code_offset);
        }
        Bytecode::BrFalse(code_offset) => {
            binary.push(Opcodes::BR_FALSE as u8);
            write_u16(binary, *code_offset);
        }
        Bytecode::Branch(code_offset) => {
            binary.push(Opcodes::BRANCH as u8);
            write_u16(binary, *code_offset);
        }
        Bytecode::LdConst(value) => {
            binary.push(Opcodes::LD_CONST as u8);
            write_u64(binary, *value);
        }
        Bytecode::LdAddr(address_idx) => {
            binary.push(Opcodes::LD_ADDR as u8);
            write_u16_as_uleb128(binary, address_idx.0);
        }
        Bytecode::LdByteArray(byte_array_idx) => {
            binary.push(Opcodes::LD_BYTEARRAY as u8);
            write_u16_as_uleb128(binary, byte_array_idx.0);
        }
        Bytecode::LdStr(string_idx) => {
            binary.push(Opcodes::LD_STR as u8);
            write_u16_as_uleb128(binary, string_idx.0);
        }
        Bytecode::LdTrue => binary.push(Opcodes::LD_TRUE as u8),
        Bytecode::LdFalse => binary.push(Opcodes::LD_FALSE as u8),
        Bytecode::CopyLoc(local_idx) => {
            binary.push(Opcodes::COPY_LOC as u8);
            binary.push(*local_idx);
        }
        Bytecode::MoveLoc(local_idx) => {
            binary.push(Opcodes::MOVE_LOC as u8);
            binary.push(*local_idx);
        }
        Bytecode::StLoc(local_idx) => {
            binary.push(Opcodes::ST_LOC as u8);
            binary.push(*local_idx);
        }
        Bytecode::BorrowLoc(local_idx) => {
            binary.push(Opcodes::LD_REF_LOC as u8);
            binary.push(*local_idx);
        }
        Bytecode::BorrowField(field_idx) => {
            binary.push(Opcodes::LD_REF_FIELD as u8);
            write_u16_as_uleb128(binary, field_idx.0);
        }
        Bytecode::Call(method_idx) => {
            binary.push(Opcodes::CALL as u8);
            write_u16_as_uleb128(binary, method_idx.0);
        }
        Bytecode::Pack(class_idx) => {
            binary.push(Opcodes::PACK as u8);
            write_u16_as_uleb128(binary, class_idx.0);
        }
        Bytecode::Unpack(class_idx) => {
            binary.push(Opcodes::UNPACK as u8);
            write_u16_as_uleb128(binary, class_idx.0);
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
        Bytecode::Or => binary.push(Opcodes::OR as u8),
        Bytecode::And => binary.push(Opcodes::AND as u8),
        Bytecode::Not => binary.push(Opcodes::NOT as u8),
        Bytecode::Eq => binary.push(Opcodes::EQ as u8),
        Bytecode::Neq => binary.push(Opcodes::NEQ as u8),
        Bytecode::Lt => binary.push(Opcodes::LT as u8),
        Bytecode::Gt => binary.push(Opcodes::GT as u8),
        Bytecode::Le => binary.push(Opcodes::LE as u8),
        Bytecode::Ge => binary.push(Opcodes::GE as u8),
        Bytecode::Assert => binary.push(Opcodes::ASSERT as u8),
        Bytecode::GetTxnGasUnitPrice => binary.push(Opcodes::GET_TXN_GAS_UNIT_PRICE as u8),
        Bytecode::GetTxnMaxGasUnits => binary.push(Opcodes::GET_TXN_MAX_GAS_UNITS as u8),
        Bytecode::GetGasRemaining => binary.push(Opcodes::GET_GAS_REMAINING as u8),
        Bytecode::GetTxnSenderAddress => binary.push(Opcodes::GET_TXN_SENDER as u8),
        Bytecode::Exists(class_idx) => {
            binary.push(Opcodes::EXISTS as u8);
            write_u16_as_uleb128(binary, class_idx.0);
        }
        Bytecode::BorrowGlobal(class_idx) => {
            binary.push(Opcodes::BORROW_REF as u8);
            write_u16_as_uleb128(binary, class_idx.0);
        }
        Bytecode::ReleaseRef => binary.push(Opcodes::RELEASE_REF as u8),
        Bytecode::MoveFrom(class_idx) => {
            binary.push(Opcodes::MOVE_FROM as u8);
            write_u16_as_uleb128(binary, class_idx.0);
        }
        Bytecode::MoveToSender(class_idx) => {
            binary.push(Opcodes::MOVE_TO as u8);
            write_u16_as_uleb128(binary, class_idx.0);
        }
        Bytecode::CreateAccount => binary.push(Opcodes::CREATE_ACCOUNT as u8),
        Bytecode::EmitEvent => binary.push(Opcodes::EMIT_EVENT as u8),
        Bytecode::GetTxnSequenceNumber => binary.push(Opcodes::GET_TXN_SEQUENCE_NUMBER as u8),
        Bytecode::GetTxnPublicKey => binary.push(Opcodes::GET_TXN_PUBLIC_KEY as u8),
    }
}

/// Serializes a `Bytecode` stream. Serialization of the function body.
fn serialize_code(binary: &mut Vec<u8>, code: &[Bytecode]) -> Result<()> {
    let code_size = code.len();
    if code_size > u16::max_value() as usize {
        bail!(
            "code size ({}) cannot exceed {}",
            code_size,
            u16::max_value(),
        )
    }
    write_u16(binary, code_size as u16);
    for opcode in code {
        serialize_instruction(binary, opcode)
    }
    Ok(())
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
            type_signatures: (0, 0),
            function_signatures: (0, 0),
            locals_signatures: (0, 0),
            string_pool: (0, 0),
            address_pool: (0, 0),
            byte_array_pool: (0, 0),
        }
    }

    /// Common binary header serialization.
    fn serialize_header(&mut self, binary: &mut Vec<u8>) -> Result<u32> {
        serialize_magic(binary);
        binary.push(self.major_version);
        binary.push(self.minor_version);
        binary.push(self.table_count);

        let start_offset = check_index_in_binary(binary.len())? + u32::from(self.table_count) * 9;

        serialize_table(
            binary,
            TableType::MODULE_HANDLES,
            self.module_handles.0 + start_offset,
            self.module_handles.1,
        );
        serialize_table(
            binary,
            TableType::STRUCT_HANDLES,
            self.struct_handles.0 + start_offset,
            self.struct_handles.1,
        );
        serialize_table(
            binary,
            TableType::FUNCTION_HANDLES,
            self.function_handles.0 + start_offset,
            self.function_handles.1,
        );
        serialize_table(
            binary,
            TableType::TYPE_SIGNATURES,
            self.type_signatures.0 + start_offset,
            self.type_signatures.1,
        );
        serialize_table(
            binary,
            TableType::FUNCTION_SIGNATURES,
            self.function_signatures.0 + start_offset,
            self.function_signatures.1,
        );
        serialize_table(
            binary,
            TableType::LOCALS_SIGNATURES,
            self.locals_signatures.0 + start_offset,
            self.locals_signatures.1,
        );
        serialize_table(
            binary,
            TableType::STRING_POOL,
            self.string_pool.0 + start_offset,
            self.string_pool.1,
        );
        serialize_table(
            binary,
            TableType::ADDRESS_POOL,
            self.address_pool.0 + start_offset,
            self.address_pool.1,
        );
        serialize_table(
            binary,
            TableType::BYTE_ARRAY_POOL,
            self.byte_array_pool.0 + start_offset,
            self.byte_array_pool.1,
        );
        Ok(start_offset)
    }

    fn serialize_common<T: CommonTables>(
        &mut self,
        binary: &mut Vec<u8>,
        tables: &T,
    ) -> Result<()> {
        self.serialize_module_handles(binary, tables.get_module_handles())?;
        self.serialize_struct_handles(binary, tables.get_struct_handles())?;
        self.serialize_function_handles(binary, tables.get_function_handles())?;
        self.serialize_type_signatures(binary, tables.get_type_signatures())?;
        self.serialize_function_signatures(binary, tables.get_function_signatures())?;
        self.serialize_locals_signatures(binary, tables.get_locals_signatures())?;
        self.serialize_strings(binary, tables.get_string_pool())?;
        self.serialize_addresses(binary, tables.get_address_pool())?;
        self.serialize_byte_arrays(binary, tables.get_byte_array_pool())?;
        Ok(())
    }

    /// Serializes `ModuleHandle` table.
    fn serialize_module_handles(
        &mut self,
        binary: &mut Vec<u8>,
        module_handles: &[ModuleHandle],
    ) -> Result<()> {
        if !module_handles.is_empty() {
            self.table_count += 1;
            self.module_handles.0 = check_index_in_binary(binary.len())?;
            for module_handle in module_handles {
                serialize_module_handle(binary, module_handle);
            }
            self.module_handles.1 = check_index_in_binary(binary.len())? - self.module_handles.0;
        }
        Ok(())
    }

    /// Serializes `StructHandle` table.
    fn serialize_struct_handles(
        &mut self,
        binary: &mut Vec<u8>,
        struct_handles: &[StructHandle],
    ) -> Result<()> {
        if !struct_handles.is_empty() {
            self.table_count += 1;
            self.struct_handles.0 = check_index_in_binary(binary.len())?;
            for struct_handle in struct_handles {
                serialize_struct_handle(binary, struct_handle);
            }
            self.struct_handles.1 = check_index_in_binary(binary.len())? - self.struct_handles.0;
        }
        Ok(())
    }

    /// Serializes `FunctionHandle` table.
    fn serialize_function_handles(
        &mut self,
        binary: &mut Vec<u8>,
        function_handles: &[FunctionHandle],
    ) -> Result<()> {
        if !function_handles.is_empty() {
            self.table_count += 1;
            self.function_handles.0 = check_index_in_binary(binary.len())?;
            for function_handle in function_handles {
                serialize_function_handle(binary, function_handle);
            }
            self.function_handles.1 =
                check_index_in_binary(binary.len())? - self.function_handles.0;
        }
        Ok(())
    }

    /// Serializes `StringPool`.
    fn serialize_strings(&mut self, binary: &mut Vec<u8>, strings: &[String]) -> Result<()> {
        if !strings.is_empty() {
            self.table_count += 1;
            self.string_pool.0 = check_index_in_binary(binary.len())?;
            for string in strings {
                serialize_string(binary, string)?;
            }
            self.string_pool.1 = check_index_in_binary(binary.len())? - self.string_pool.0;
        }
        Ok(())
    }

    /// Serializes `ByteArrayPool`.
    fn serialize_byte_arrays(
        &mut self,
        binary: &mut Vec<u8>,
        byte_arrays: &[ByteArray],
    ) -> Result<()> {
        if !byte_arrays.is_empty() {
            self.table_count += 1;
            self.byte_array_pool.0 = check_index_in_binary(binary.len())?;
            for byte_array in byte_arrays {
                serialize_byte_array(binary, byte_array)?;
            }
            self.byte_array_pool.1 = check_index_in_binary(binary.len())? - self.byte_array_pool.0;
        }
        Ok(())
    }

    /// Serializes `AddressPool`.
    fn serialize_addresses(
        &mut self,
        binary: &mut Vec<u8>,
        addresses: &[AccountAddress],
    ) -> Result<()> {
        if !addresses.is_empty() {
            self.table_count += 1;
            self.address_pool.0 = check_index_in_binary(binary.len())?;
            for address in addresses {
                serialize_address(binary, address)?;
            }
            self.address_pool.1 = check_index_in_binary(binary.len())? - self.address_pool.0;
        }
        Ok(())
    }

    /// Serializes `TypeSignaturePool` table.
    fn serialize_type_signatures(
        &mut self,
        binary: &mut Vec<u8>,
        signatures: &[TypeSignature],
    ) -> Result<()> {
        if !signatures.is_empty() {
            self.table_count += 1;
            self.type_signatures.0 = check_index_in_binary(binary.len())?;
            for signature in signatures {
                serialize_type_signature(binary, signature)?;
            }
            self.type_signatures.1 = check_index_in_binary(binary.len())? - self.type_signatures.0;
        }
        Ok(())
    }

    /// Serializes `FunctionSignaturePool` table.
    fn serialize_function_signatures(
        &mut self,
        binary: &mut Vec<u8>,
        signatures: &[FunctionSignature],
    ) -> Result<()> {
        if !signatures.is_empty() {
            self.table_count += 1;
            self.function_signatures.0 = check_index_in_binary(binary.len())?;
            for signature in signatures {
                serialize_function_signature(binary, signature)?;
            }
            self.function_signatures.1 =
                check_index_in_binary(binary.len())? - self.function_signatures.0;
        }
        Ok(())
    }

    /// Serializes `LocalSignaturePool` table.
    fn serialize_locals_signatures(
        &mut self,
        binary: &mut Vec<u8>,
        signatures: &[LocalsSignature],
    ) -> Result<()> {
        if !signatures.is_empty() {
            self.table_count += 1;
            self.locals_signatures.0 = check_index_in_binary(binary.len())?;
            for signature in signatures {
                serialize_locals_signature(binary, signature)?;
            }
            self.locals_signatures.1 =
                check_index_in_binary(binary.len())? - self.locals_signatures.0;
        }
        Ok(())
    }
}

impl ModuleSerializer {
    fn new(major_version: u8, minor_version: u8) -> ModuleSerializer {
        ModuleSerializer {
            common: CommonSerializer::new(major_version, minor_version),
            struct_defs: (0, 0),
            field_defs: (0, 0),
            function_defs: (0, 0),
        }
    }

    fn serialize(&mut self, binary: &mut Vec<u8>, module: &CompiledModuleMut) -> Result<()> {
        self.common.serialize_common(binary, module)?;
        self.serialize_struct_definitions(binary, &module.struct_defs)?;
        self.serialize_field_definitions(binary, &module.field_defs)?;
        self.serialize_function_definitions(binary, &module.function_defs)
    }

    fn serialize_header(&mut self, binary: &mut Vec<u8>) -> Result<()> {
        let start_offset = self.common.serialize_header(binary)?;
        serialize_table(
            binary,
            TableType::STRUCT_DEFS,
            self.struct_defs.0 + start_offset,
            self.struct_defs.1,
        );
        serialize_table(
            binary,
            TableType::FIELD_DEFS,
            self.field_defs.0 + start_offset,
            self.field_defs.1,
        );
        serialize_table(
            binary,
            TableType::FUNCTION_DEFS,
            self.function_defs.0 + start_offset,
            self.function_defs.1,
        );
        Ok(())
    }

    /// Serializes `StructDefinition` table.
    fn serialize_struct_definitions(
        &mut self,
        binary: &mut Vec<u8>,
        struct_definitions: &[StructDefinition],
    ) -> Result<()> {
        if !struct_definitions.is_empty() {
            self.common.table_count += 1;
            self.struct_defs.0 = check_index_in_binary(binary.len())?;
            for struct_definition in struct_definitions {
                serialize_struct_definition(binary, struct_definition);
            }
            self.struct_defs.1 = check_index_in_binary(binary.len())? - self.struct_defs.0;
        }
        Ok(())
    }

    /// Serializes `FieldDefinition` table.
    fn serialize_field_definitions(
        &mut self,
        binary: &mut Vec<u8>,
        field_definitions: &[FieldDefinition],
    ) -> Result<()> {
        if !field_definitions.is_empty() {
            self.common.table_count += 1;
            self.field_defs.0 = check_index_in_binary(binary.len())?;
            for field_definition in field_definitions {
                serialize_field_definition(binary, field_definition);
            }
            self.field_defs.1 = check_index_in_binary(binary.len())? - self.field_defs.0;
        }
        Ok(())
    }

    /// Serializes `FunctionDefinition` table.
    fn serialize_function_definitions(
        &mut self,
        binary: &mut Vec<u8>,
        function_definitions: &[FunctionDefinition],
    ) -> Result<()> {
        if !function_definitions.is_empty() {
            self.common.table_count += 1;
            self.function_defs.0 = check_index_in_binary(binary.len())?;
            for function_definition in function_definitions {
                serialize_function_definition(binary, function_definition)?;
            }
            self.function_defs.1 = check_index_in_binary(binary.len())? - self.function_defs.0;
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

    fn serialize(&mut self, binary: &mut Vec<u8>, script: &CompiledScriptMut) -> Result<()> {
        self.common.serialize_common(binary, script)?;
        self.serialize_main(binary, &script.main)
    }

    fn serialize_header(&mut self, binary: &mut Vec<u8>) -> Result<()> {
        let start_offset = self.common.serialize_header(binary)?;
        serialize_table(
            binary,
            TableType::MAIN,
            self.main.0 + start_offset,
            self.main.1,
        );
        Ok(())
    }

    /// Serializes the main function.
    fn serialize_main(&mut self, binary: &mut Vec<u8>, main: &FunctionDefinition) -> Result<()> {
        self.common.table_count += 1;
        self.main.0 = check_index_in_binary(binary.len())?;
        serialize_function_definition(binary, main)?;
        self.main.1 = check_index_in_binary(binary.len())? - self.main.0;
        Ok(())
    }
}
