// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{errors::*, file_format::*, file_format_common::*};
use byteorder::{LittleEndian, ReadBytesExt};
use std::{
    collections::HashSet,
    convert::TryInto,
    io::{Cursor, Read},
    str::from_utf8,
};
use types::{account_address::ADDRESS_LENGTH, byte_array::ByteArray};

impl CompiledScript {
    /// Deserializes a &[u8] slice into a `CompiledScript` instance.
    pub fn deserialize(binary: &[u8]) -> BinaryLoaderResult<Self> {
        let deserialized = CompiledScriptMut::deserialize_no_check_bounds(binary)?;
        deserialized.freeze().map_err(|_| BinaryError::Malformed)
    }
}

impl CompiledScriptMut {
    // exposed as a public function to enable testing the deserializer
    #[doc(hidden)]
    pub fn deserialize_no_check_bounds(binary: &[u8]) -> BinaryLoaderResult<Self> {
        deserialize_compiled_script(binary)
    }
}

impl CompiledModule {
    /// Deserialize a &[u8] slice into a `CompiledModule` instance.
    pub fn deserialize(binary: &[u8]) -> BinaryLoaderResult<Self> {
        let deserialized = CompiledModuleMut::deserialize_no_check_bounds(binary)?;
        deserialized.freeze().map_err(|_| BinaryError::Malformed)
    }
}

impl CompiledModuleMut {
    // exposed as a public function to enable testing the deserializer
    pub fn deserialize_no_check_bounds(binary: &[u8]) -> BinaryLoaderResult<Self> {
        deserialize_compiled_module(binary)
    }
}

/// Table info: table type, offset where the table content starts from, count of bytes for
/// the table content.
#[derive(Clone, Debug)]
struct Table {
    kind: TableType,
    offset: u32,
    count: u32,
}

impl Table {
    fn new(kind: TableType, offset: u32, count: u32) -> Table {
        Table {
            kind,
            offset,
            count,
        }
    }
}

/// Module internal function that manages deserialization of transactions.
fn deserialize_compiled_script(binary: &[u8]) -> BinaryLoaderResult<CompiledScriptMut> {
    let binary_len = binary.len() as u64;
    let mut cursor = Cursor::new(binary);
    let table_count = check_binary(&mut cursor)?;
    let mut tables: Vec<Table> = Vec::new();
    read_tables(&mut cursor, table_count, &mut tables)?;
    check_tables(&mut tables, cursor.position(), binary_len)?;

    build_compiled_script(binary, &tables)
}

/// Module internal function that manages deserialization of modules.
fn deserialize_compiled_module(binary: &[u8]) -> BinaryLoaderResult<CompiledModuleMut> {
    let binary_len = binary.len() as u64;
    let mut cursor = Cursor::new(binary);
    let table_count = check_binary(&mut cursor)?;
    let mut tables: Vec<Table> = Vec::new();
    read_tables(&mut cursor, table_count, &mut tables)?;
    check_tables(&mut tables, cursor.position(), binary_len)?;

    build_compiled_module(binary, &tables)
}

/// Verifies the correctness of the "static" part of the binary's header.
///
/// Returns the offset where the count of tables in the binary.
fn check_binary(cursor: &mut Cursor<&[u8]>) -> BinaryLoaderResult<u8> {
    let mut magic = [0u8; BinaryConstants::LIBRA_MAGIC_SIZE];
    if let Ok(count) = cursor.read(&mut magic) {
        if count != BinaryConstants::LIBRA_MAGIC_SIZE {
            return Err(BinaryError::Malformed);
        } else if magic != BinaryConstants::LIBRA_MAGIC {
            return Err(BinaryError::BadMagic);
        }
    } else {
        return Err(BinaryError::Malformed);
    }
    let major_ver = 1u8;
    let minor_ver = 0u8;
    if let Ok(ver) = cursor.read_u8() {
        if ver != major_ver {
            return Err(BinaryError::UnknownVersion);
        }
    } else {
        return Err(BinaryError::Malformed);
    }
    if let Ok(ver) = cursor.read_u8() {
        if ver != minor_ver {
            return Err(BinaryError::UnknownVersion);
        }
    } else {
        return Err(BinaryError::Malformed);
    }
    if let Ok(count) = cursor.read_u8() {
        Ok(count)
    } else {
        Err(BinaryError::Malformed)
    }
}

/// Reads all the table headers.
///
/// Return a Vec<Table> that contains all the table headers defined and checked.
fn read_tables(
    cursor: &mut Cursor<&[u8]>,
    table_count: u8,
    tables: &mut Vec<Table>,
) -> BinaryLoaderResult<()> {
    for _count in 0..table_count {
        tables.push(read_table(cursor)?);
    }
    Ok(())
}

/// Reads a table from a slice at a given offset.
/// If a table is not recognized an error is returned.
fn read_table(cursor: &mut Cursor<&[u8]>) -> BinaryLoaderResult<Table> {
    if let Ok(kind) = cursor.read_u8() {
        let table_offset = read_u32_internal(cursor)?;
        let count = read_u32_internal(cursor)?;
        Ok(Table::new(TableType::from_u8(kind)?, table_offset, count))
    } else {
        Err(BinaryError::Malformed)
    }
}

/// Verify correctness of tables.
///
/// Tables cannot have duplicates, must cover the entire blob and must be disjoint.
fn check_tables(tables: &mut Vec<Table>, end_tables: u64, length: u64) -> BinaryLoaderResult<()> {
    // there is no real reason to pass a mutable reference but we are sorting next line
    tables.sort_by(|t1, t2| t1.offset.cmp(&t2.offset));

    let mut current_offset = end_tables;
    let mut table_types = HashSet::new();
    for table in tables {
        let offset = u64::from(table.offset);
        if offset != current_offset {
            return Err(BinaryError::BadHeaderTable);
        }
        if table.count == 0 {
            return Err(BinaryError::BadHeaderTable);
        }
        let count = u64::from(table.count);
        if let Some(checked_offset) = current_offset.checked_add(count) {
            current_offset = checked_offset;
        }
        if current_offset > length {
            return Err(BinaryError::BadHeaderTable);
        }
        if !table_types.insert(table.kind) {
            return Err(BinaryError::DuplicateTable);
        }
    }
    if current_offset != length {
        return Err(BinaryError::BadHeaderTable);
    }
    Ok(())
}

//
// Trait to read common tables from CompiledScript or CompiledModule
//

trait CommonTables {
    fn get_module_handles(&mut self) -> &mut Vec<ModuleHandle>;
    fn get_struct_handles(&mut self) -> &mut Vec<StructHandle>;
    fn get_function_handles(&mut self) -> &mut Vec<FunctionHandle>;

    fn get_type_signatures(&mut self) -> &mut TypeSignaturePool;
    fn get_function_signatures(&mut self) -> &mut FunctionSignaturePool;
    fn get_locals_signatures(&mut self) -> &mut LocalsSignaturePool;

    fn get_string_pool(&mut self) -> &mut StringPool;
    fn get_byte_array_pool(&mut self) -> &mut ByteArrayPool;
    fn get_address_pool(&mut self) -> &mut AddressPool;
}

impl CommonTables for CompiledScriptMut {
    fn get_module_handles(&mut self) -> &mut Vec<ModuleHandle> {
        &mut self.module_handles
    }

    fn get_struct_handles(&mut self) -> &mut Vec<StructHandle> {
        &mut self.struct_handles
    }

    fn get_function_handles(&mut self) -> &mut Vec<FunctionHandle> {
        &mut self.function_handles
    }

    fn get_type_signatures(&mut self) -> &mut TypeSignaturePool {
        &mut self.type_signatures
    }

    fn get_function_signatures(&mut self) -> &mut FunctionSignaturePool {
        &mut self.function_signatures
    }

    fn get_locals_signatures(&mut self) -> &mut LocalsSignaturePool {
        &mut self.locals_signatures
    }

    fn get_string_pool(&mut self) -> &mut StringPool {
        &mut self.string_pool
    }

    fn get_byte_array_pool(&mut self) -> &mut ByteArrayPool {
        &mut self.byte_array_pool
    }

    fn get_address_pool(&mut self) -> &mut AddressPool {
        &mut self.address_pool
    }
}

impl CommonTables for CompiledModuleMut {
    fn get_module_handles(&mut self) -> &mut Vec<ModuleHandle> {
        &mut self.module_handles
    }

    fn get_struct_handles(&mut self) -> &mut Vec<StructHandle> {
        &mut self.struct_handles
    }

    fn get_function_handles(&mut self) -> &mut Vec<FunctionHandle> {
        &mut self.function_handles
    }

    fn get_type_signatures(&mut self) -> &mut TypeSignaturePool {
        &mut self.type_signatures
    }

    fn get_function_signatures(&mut self) -> &mut FunctionSignaturePool {
        &mut self.function_signatures
    }

    fn get_locals_signatures(&mut self) -> &mut LocalsSignaturePool {
        &mut self.locals_signatures
    }

    fn get_string_pool(&mut self) -> &mut StringPool {
        &mut self.string_pool
    }

    fn get_byte_array_pool(&mut self) -> &mut ByteArrayPool {
        &mut self.byte_array_pool
    }

    fn get_address_pool(&mut self) -> &mut AddressPool {
        &mut self.address_pool
    }
}

/// Builds and returns a `CompiledScriptMut`.
fn build_compiled_script(binary: &[u8], tables: &[Table]) -> BinaryLoaderResult<CompiledScriptMut> {
    let mut script = CompiledScriptMut::default();
    build_common_tables(binary, tables, &mut script)?;
    build_script_tables(binary, tables, &mut script)?;
    Ok(script)
}

/// Builds and returns a `CompiledModuleMut`.
fn build_compiled_module(binary: &[u8], tables: &[Table]) -> BinaryLoaderResult<CompiledModuleMut> {
    let mut module = CompiledModuleMut::default();
    build_common_tables(binary, tables, &mut module)?;
    build_module_tables(binary, tables, &mut module)?;
    Ok(module)
}

/// Builds the common tables in a compiled unit.
fn build_common_tables(
    binary: &[u8],
    tables: &[Table],
    common: &mut impl CommonTables,
) -> BinaryLoaderResult<()> {
    for table in tables {
        match table.kind {
            TableType::MODULE_HANDLES => {
                load_module_handles(binary, table, common.get_module_handles())?;
            }
            TableType::STRUCT_HANDLES => {
                load_struct_handles(binary, table, common.get_struct_handles())?;
            }
            TableType::FUNCTION_HANDLES => {
                load_function_handles(binary, table, common.get_function_handles())?;
            }
            TableType::ADDRESS_POOL => {
                load_address_pool(binary, table, common.get_address_pool())?;
            }
            TableType::STRING_POOL => {
                load_string_pool(binary, table, common.get_string_pool())?;
            }
            TableType::BYTE_ARRAY_POOL => {
                load_byte_array_pool(binary, table, common.get_byte_array_pool())?;
            }
            TableType::TYPE_SIGNATURES => {
                load_type_signatures(binary, table, common.get_type_signatures())?;
            }
            TableType::FUNCTION_SIGNATURES => {
                load_function_signatures(binary, table, common.get_function_signatures())?;
            }
            TableType::LOCALS_SIGNATURES => {
                load_locals_signatures(binary, table, common.get_locals_signatures())?;
            }
            TableType::FUNCTION_DEFS
            | TableType::FIELD_DEFS
            | TableType::STRUCT_DEFS
            | TableType::MAIN => continue,
        }
    }
    Ok(())
}

/// Builds tables related to a `CompiledModuleMut`.
fn build_module_tables(
    binary: &[u8],
    tables: &[Table],
    module: &mut CompiledModuleMut,
) -> BinaryLoaderResult<()> {
    for table in tables {
        match table.kind {
            TableType::STRUCT_DEFS => {
                load_struct_defs(binary, table, &mut module.struct_defs)?;
            }
            TableType::FIELD_DEFS => {
                load_field_defs(binary, table, &mut module.field_defs)?;
            }
            TableType::FUNCTION_DEFS => {
                load_function_defs(binary, table, &mut module.function_defs)?;
            }
            TableType::MODULE_HANDLES
            | TableType::STRUCT_HANDLES
            | TableType::FUNCTION_HANDLES
            | TableType::ADDRESS_POOL
            | TableType::STRING_POOL
            | TableType::BYTE_ARRAY_POOL
            | TableType::TYPE_SIGNATURES
            | TableType::FUNCTION_SIGNATURES
            | TableType::LOCALS_SIGNATURES => {
                continue;
            }
            TableType::MAIN => return Err(BinaryError::Malformed),
        }
    }
    Ok(())
}

/// Builds tables related to a `CompiledScriptMut`.
fn build_script_tables(
    binary: &[u8],
    tables: &[Table],
    script: &mut CompiledScriptMut,
) -> BinaryLoaderResult<()> {
    for table in tables {
        match table.kind {
            TableType::MAIN => {
                let start: usize = table.offset as usize;
                let end: usize = start + table.count as usize;
                let mut cursor = Cursor::new(&binary[start..end]);
                let main = load_function_def(&mut cursor)?;
                script.main = main;
            }
            TableType::MODULE_HANDLES
            | TableType::STRUCT_HANDLES
            | TableType::FUNCTION_HANDLES
            | TableType::ADDRESS_POOL
            | TableType::STRING_POOL
            | TableType::BYTE_ARRAY_POOL
            | TableType::TYPE_SIGNATURES
            | TableType::FUNCTION_SIGNATURES
            | TableType::LOCALS_SIGNATURES => {
                continue;
            }
            TableType::STRUCT_DEFS | TableType::FIELD_DEFS | TableType::FUNCTION_DEFS => {
                return Err(BinaryError::Malformed);
            }
        }
    }
    Ok(())
}

/// Builds the `ModuleHandle` table.
fn load_module_handles(
    binary: &[u8],
    table: &Table,
    module_handles: &mut Vec<ModuleHandle>,
) -> BinaryLoaderResult<()> {
    let start = table.offset as usize;
    let end = start + table.count as usize;
    let mut cursor = Cursor::new(&binary[start..end]);
    loop {
        if cursor.position() == u64::from(table.count) {
            break;
        }
        let address = read_uleb_u16_internal(&mut cursor)?;
        let name = read_uleb_u16_internal(&mut cursor)?;
        module_handles.push(ModuleHandle {
            address: AddressPoolIndex(address),
            name: StringPoolIndex(name),
        });
    }
    Ok(())
}

/// Builds the `StructHandle` table.
fn load_struct_handles(
    binary: &[u8],
    table: &Table,
    struct_handles: &mut Vec<StructHandle>,
) -> BinaryLoaderResult<()> {
    let start = table.offset as usize;
    let end = start + table.count as usize;
    let mut cursor = Cursor::new(&binary[start..end]);
    loop {
        if cursor.position() == u64::from(table.count) {
            break;
        }
        let module_handle = read_uleb_u16_internal(&mut cursor)?;
        let name = read_uleb_u16_internal(&mut cursor)?;
        let kind = load_kind(&mut cursor)?;
        let kind_constraints = load_kinds(&mut cursor)?;
        struct_handles.push(StructHandle {
            module: ModuleHandleIndex(module_handle),
            name: StringPoolIndex(name),
            kind,
            kind_constraints,
        });
    }
    Ok(())
}

/// Builds the `FunctionHandle` table.
fn load_function_handles(
    binary: &[u8],
    table: &Table,
    function_handles: &mut Vec<FunctionHandle>,
) -> BinaryLoaderResult<()> {
    let start = table.offset as usize;
    let end = start + table.count as usize;
    let mut cursor = Cursor::new(&binary[start..end]);
    loop {
        if cursor.position() == u64::from(table.count) {
            break;
        }
        let module_handle = read_uleb_u16_internal(&mut cursor)?;
        let name = read_uleb_u16_internal(&mut cursor)?;
        let signature = read_uleb_u16_internal(&mut cursor)?;
        function_handles.push(FunctionHandle {
            module: ModuleHandleIndex(module_handle),
            name: StringPoolIndex(name),
            signature: FunctionSignatureIndex(signature),
        });
    }
    Ok(())
}

/// Builds the `AddressPool`.
fn load_address_pool(
    binary: &[u8],
    table: &Table,
    addresses: &mut AddressPool,
) -> BinaryLoaderResult<()> {
    let mut start = table.offset as usize;
    if table.count as usize % ADDRESS_LENGTH != 0 {
        return Err(BinaryError::Malformed);
    }
    for _i in 0..table.count as usize / ADDRESS_LENGTH {
        let end_addr = start + ADDRESS_LENGTH;
        let address = (&binary[start..end_addr]).try_into();
        if address.is_err() {
            return Err(BinaryError::Malformed);
        }
        start = end_addr;

        addresses.push(address.unwrap());
    }
    Ok(())
}

/// Builds the `StringPool`.
fn load_string_pool(
    binary: &[u8],
    table: &Table,
    strings: &mut StringPool,
) -> BinaryLoaderResult<()> {
    let start = table.offset as usize;
    let end = start + table.count as usize;
    let mut cursor = Cursor::new(&binary[start..end]);
    while cursor.position() < u64::from(table.count) {
        let size = read_uleb_u32_internal(&mut cursor)? as usize;
        if size > std::u16::MAX as usize {
            return Err(BinaryError::Malformed);
        }
        let mut buffer: Vec<u8> = vec![0u8; size];
        if let Ok(count) = cursor.read(&mut buffer) {
            if count != size {
                return Err(BinaryError::Malformed);
            }
            let s = match from_utf8(&buffer) {
                Ok(bytes) => bytes,
                Err(_) => return Err(BinaryError::Malformed),
            };

            strings.push(String::from(s));
        }
    }
    Ok(())
}

/// Builds the `ByteArrayPool`.
fn load_byte_array_pool(
    binary: &[u8],
    table: &Table,
    byte_arrays: &mut ByteArrayPool,
) -> BinaryLoaderResult<()> {
    let start = table.offset as usize;
    let end = start + table.count as usize;
    let mut cursor = Cursor::new(&binary[start..end]);
    while cursor.position() < u64::from(table.count) {
        let size = read_uleb_u32_internal(&mut cursor)? as usize;
        if size > std::u16::MAX as usize {
            return Err(BinaryError::Malformed);
        }
        let mut byte_array: Vec<u8> = vec![0u8; size];
        if let Ok(count) = cursor.read(&mut byte_array) {
            if count != size {
                return Err(BinaryError::Malformed);
            }

            byte_arrays.push(ByteArray::new(byte_array));
        }
    }
    Ok(())
}

/// Builds the `TypeSignaturePool`.
fn load_type_signatures(
    binary: &[u8],
    table: &Table,
    type_signatures: &mut TypeSignaturePool,
) -> BinaryLoaderResult<()> {
    let start = table.offset as usize;
    let end = start + table.count as usize;
    let mut cursor = Cursor::new(&binary[start..end]);
    while cursor.position() < u64::from(table.count) {
        if let Ok(byte) = cursor.read_u8() {
            if byte != SignatureType::TYPE_SIGNATURE as u8 {
                return Err(BinaryError::UnexpectedSignatureType);
            }
        }
        let token = load_signature_token(&mut cursor)?;
        type_signatures.push(TypeSignature(token));
    }
    Ok(())
}

/// Builds the `FunctionSignaturePool`.
fn load_function_signatures(
    binary: &[u8],
    table: &Table,
    function_signatures: &mut FunctionSignaturePool,
) -> BinaryLoaderResult<()> {
    let start = table.offset as usize;
    let end = start + table.count as usize;
    let mut cursor = Cursor::new(&binary[start..end]);
    while cursor.position() < u64::from(table.count) {
        if let Ok(byte) = cursor.read_u8() {
            if byte != SignatureType::FUNCTION_SIGNATURE as u8 {
                return Err(BinaryError::UnexpectedSignatureType);
            }
        }

        // Return signature
        let token_count = cursor.read_u8().map_err(|_| BinaryError::Malformed)?;
        let mut returns_signature: Vec<SignatureToken> = Vec::new();
        for _i in 0..token_count {
            let token = load_signature_token(&mut cursor)?;
            returns_signature.push(token);
        }

        // Arguments signature
        let token_count = cursor.read_u8().map_err(|_| BinaryError::Malformed)?;
        let mut args_signature: Vec<SignatureToken> = Vec::new();
        for _i in 0..token_count {
            let token = load_signature_token(&mut cursor)?;
            args_signature.push(token);
        }
        let kind_constraints = load_kinds(&mut cursor)?;
        function_signatures.push(FunctionSignature {
            return_types: returns_signature,
            arg_types: args_signature,
            kind_constraints,
        });
    }
    Ok(())
}

/// Builds the `LocalsSignaturePool`.
fn load_locals_signatures(
    binary: &[u8],
    table: &Table,
    locals_signatures: &mut LocalsSignaturePool,
) -> BinaryLoaderResult<()> {
    let start = table.offset as usize;
    let end = start + table.count as usize;
    let mut cursor = Cursor::new(&binary[start..end]);
    while cursor.position() < u64::from(table.count) {
        if let Ok(byte) = cursor.read_u8() {
            if byte != SignatureType::LOCAL_SIGNATURE as u8 {
                return Err(BinaryError::UnexpectedSignatureType);
            }
        }

        let token_count = cursor.read_u8().map_err(|_| BinaryError::Malformed)?;
        let mut local_signature: Vec<SignatureToken> = Vec::new();
        for _i in 0..token_count {
            let token = load_signature_token(&mut cursor)?;
            local_signature.push(token);
        }

        locals_signatures.push(LocalsSignature(local_signature));
    }
    Ok(())
}

/// Deserializes a `SignatureToken`.
fn load_signature_token(cursor: &mut Cursor<&[u8]>) -> BinaryLoaderResult<SignatureToken> {
    if let Ok(byte) = cursor.read_u8() {
        match SerializedType::from_u8(byte)? {
            SerializedType::BOOL => Ok(SignatureToken::Bool),
            SerializedType::INTEGER => Ok(SignatureToken::U64),
            SerializedType::STRING => Ok(SignatureToken::String),
            SerializedType::BYTEARRAY => Ok(SignatureToken::ByteArray),
            SerializedType::ADDRESS => Ok(SignatureToken::Address),
            SerializedType::REFERENCE => {
                let ref_token = load_signature_token(cursor)?;
                Ok(SignatureToken::Reference(Box::new(ref_token)))
            }
            SerializedType::MUTABLE_REFERENCE => {
                let ref_token = load_signature_token(cursor)?;
                Ok(SignatureToken::MutableReference(Box::new(ref_token)))
            }
            SerializedType::STRUCT => {
                let sh_idx = read_uleb_u16_internal(cursor)?;
                let types = load_signature_tokens(cursor)?;
                Ok(SignatureToken::Struct(StructHandleIndex(sh_idx), types))
            }
            SerializedType::TYPE_PARAMETER => {
                let idx = read_uleb_u16_internal(cursor)?;
                Ok(SignatureToken::TypeParameter(idx))
            }
        }
    } else {
        Err(BinaryError::Malformed)
    }
}

fn load_signature_tokens(cursor: &mut Cursor<&[u8]>) -> BinaryLoaderResult<Vec<SignatureToken>> {
    let len = read_uleb_u16_internal(cursor)?;
    let mut tokens = vec![];
    for _ in 0..len {
        tokens.push(load_signature_token(cursor)?);
    }
    Ok(tokens)
}

fn load_kind(cursor: &mut Cursor<&[u8]>) -> BinaryLoaderResult<Kind> {
    if let Ok(byte) = cursor.read_u8() {
        match SerializedKind::from_u8(byte)? {
            SerializedKind::RESOURCE => Ok(Kind::Resource),
            SerializedKind::COPYABLE => Ok(Kind::Copyable),
        }
    } else {
        Err(BinaryError::Malformed)
    }
}

fn load_kinds(cursor: &mut Cursor<&[u8]>) -> BinaryLoaderResult<Vec<Kind>> {
    let len = read_uleb_u16_internal(cursor)?;
    let mut kinds = vec![];
    for _ in 0..len {
        kinds.push(load_kind(cursor)?);
    }
    Ok(kinds)
}

/// Builds the `StructDefinition` table.
fn load_struct_defs(
    binary: &[u8],
    table: &Table,
    struct_defs: &mut Vec<StructDefinition>,
) -> BinaryLoaderResult<()> {
    let start = table.offset as usize;
    let end = start + table.count as usize;
    let mut cursor = Cursor::new(&binary[start..end]);
    while cursor.position() < u64::from(table.count) {
        let struct_handle = read_uleb_u16_internal(&mut cursor)?;
        let field_count = read_uleb_u16_internal(&mut cursor)?;
        let fields = read_uleb_u16_internal(&mut cursor)?;
        struct_defs.push(StructDefinition {
            struct_handle: StructHandleIndex(struct_handle),
            field_count,
            fields: FieldDefinitionIndex(fields),
        });
    }
    Ok(())
}

/// Builds the `FieldDefinition` table.
fn load_field_defs(
    binary: &[u8],
    table: &Table,
    field_defs: &mut Vec<FieldDefinition>,
) -> BinaryLoaderResult<()> {
    let start = table.offset as usize;
    let end = start + table.count as usize;
    let mut cursor = Cursor::new(&binary[start..end]);
    while cursor.position() < u64::from(table.count) {
        let struct_ = read_uleb_u16_internal(&mut cursor)?;
        let name = read_uleb_u16_internal(&mut cursor)?;
        let signature = read_uleb_u16_internal(&mut cursor)?;
        field_defs.push(FieldDefinition {
            struct_: StructHandleIndex(struct_),
            name: StringPoolIndex(name),
            signature: TypeSignatureIndex(signature),
        });
    }
    Ok(())
}

/// Builds the `FunctionDefinition` table.
fn load_function_defs(
    binary: &[u8],
    table: &Table,
    func_defs: &mut Vec<FunctionDefinition>,
) -> BinaryLoaderResult<()> {
    let start = table.offset as usize;
    let end = start + table.count as usize;
    let mut cursor = Cursor::new(&binary[start..end]);
    while cursor.position() < u64::from(table.count) {
        let func_def = load_function_def(&mut cursor)?;
        func_defs.push(func_def);
    }
    Ok(())
}

/// Deserializes a `FunctionDefinition`.
fn load_function_def(cursor: &mut Cursor<&[u8]>) -> BinaryLoaderResult<FunctionDefinition> {
    let function = read_uleb_u16_internal(cursor)?;

    let flags = cursor.read_u8().map_err(|_| BinaryError::Malformed)?;
    let code_unit = load_code_unit(cursor)?;
    Ok(FunctionDefinition {
        function: FunctionHandleIndex(function),
        flags,
        code: code_unit,
    })
}

/// Deserializes a `CodeUnit`.
fn load_code_unit(cursor: &mut Cursor<&[u8]>) -> BinaryLoaderResult<CodeUnit> {
    let max_stack_size = read_uleb_u16_internal(cursor)?;
    let locals = read_uleb_u16_internal(cursor)?;

    let mut code_unit = CodeUnit {
        max_stack_size,
        locals: LocalsSignatureIndex(locals),
        code: vec![],
    };

    load_code(cursor, &mut code_unit.code)?;
    Ok(code_unit)
}

/// Deserializes a code stream (`Bytecode`s).
fn load_code(cursor: &mut Cursor<&[u8]>, code: &mut Vec<Bytecode>) -> BinaryLoaderResult<()> {
    let bytecode_count = read_u16_internal(cursor)?;
    while code.len() < bytecode_count as usize {
        let byte = cursor.read_u8().map_err(|_| BinaryError::Malformed)?;
        let bytecode = match Opcodes::from_u8(byte)? {
            Opcodes::POP => Bytecode::Pop,
            Opcodes::RET => Bytecode::Ret,
            Opcodes::BR_TRUE => {
                let jump = read_u16_internal(cursor)?;
                Bytecode::BrTrue(jump)
            }
            Opcodes::BR_FALSE => {
                let jump = read_u16_internal(cursor)?;
                Bytecode::BrFalse(jump)
            }
            Opcodes::BRANCH => {
                let jump = read_u16_internal(cursor)?;
                Bytecode::Branch(jump)
            }
            Opcodes::LD_CONST => {
                let value = read_u64_internal(cursor)?;
                Bytecode::LdConst(value)
            }
            Opcodes::LD_ADDR => {
                let idx = read_uleb_u16_internal(cursor)?;
                Bytecode::LdAddr(AddressPoolIndex(idx))
            }
            Opcodes::LD_STR => {
                let idx = read_uleb_u16_internal(cursor)?;
                Bytecode::LdStr(StringPoolIndex(idx))
            }
            Opcodes::LD_TRUE => Bytecode::LdTrue,
            Opcodes::LD_FALSE => Bytecode::LdFalse,
            Opcodes::COPY_LOC => {
                let idx = cursor.read_u8().map_err(|_| BinaryError::Malformed)?;
                Bytecode::CopyLoc(idx)
            }
            Opcodes::MOVE_LOC => {
                let idx = cursor.read_u8().map_err(|_| BinaryError::Malformed)?;
                Bytecode::MoveLoc(idx)
            }
            Opcodes::ST_LOC => {
                let idx = cursor.read_u8().map_err(|_| BinaryError::Malformed)?;
                Bytecode::StLoc(idx)
            }
            Opcodes::LD_REF_LOC => {
                let idx = cursor.read_u8().map_err(|_| BinaryError::Malformed)?;
                Bytecode::BorrowLoc(idx)
            }
            Opcodes::LD_REF_FIELD => {
                let idx = read_uleb_u16_internal(cursor)?;
                Bytecode::BorrowField(FieldDefinitionIndex(idx))
            }
            Opcodes::LD_BYTEARRAY => {
                let idx = read_uleb_u16_internal(cursor)?;
                Bytecode::LdByteArray(ByteArrayPoolIndex(idx))
            }
            Opcodes::CALL => {
                let idx = read_uleb_u16_internal(cursor)?;
                let types_idx = read_uleb_u16_internal(cursor)?;
                Bytecode::Call(FunctionHandleIndex(idx), LocalsSignatureIndex(types_idx))
            }
            Opcodes::PACK => {
                let idx = read_uleb_u16_internal(cursor)?;
                let types_idx = read_uleb_u16_internal(cursor)?;
                Bytecode::Pack(StructDefinitionIndex(idx), LocalsSignatureIndex(types_idx))
            }
            Opcodes::UNPACK => {
                let idx = read_uleb_u16_internal(cursor)?;
                let types_idx = read_uleb_u16_internal(cursor)?;
                Bytecode::Unpack(StructDefinitionIndex(idx), LocalsSignatureIndex(types_idx))
            }
            Opcodes::READ_REF => Bytecode::ReadRef,
            Opcodes::WRITE_REF => Bytecode::WriteRef,
            Opcodes::ADD => Bytecode::Add,
            Opcodes::SUB => Bytecode::Sub,
            Opcodes::MUL => Bytecode::Mul,
            Opcodes::MOD => Bytecode::Mod,
            Opcodes::DIV => Bytecode::Div,
            Opcodes::BIT_OR => Bytecode::BitOr,
            Opcodes::BIT_AND => Bytecode::BitAnd,
            Opcodes::XOR => Bytecode::Xor,
            Opcodes::OR => Bytecode::Or,
            Opcodes::AND => Bytecode::And,
            Opcodes::NOT => Bytecode::Not,
            Opcodes::EQ => Bytecode::Eq,
            Opcodes::NEQ => Bytecode::Neq,
            Opcodes::LT => Bytecode::Lt,
            Opcodes::GT => Bytecode::Gt,
            Opcodes::LE => Bytecode::Le,
            Opcodes::GE => Bytecode::Ge,
            Opcodes::ABORT => Bytecode::Abort,
            Opcodes::GET_TXN_GAS_UNIT_PRICE => Bytecode::GetTxnGasUnitPrice,
            Opcodes::GET_TXN_MAX_GAS_UNITS => Bytecode::GetTxnMaxGasUnits,
            Opcodes::GET_GAS_REMAINING => Bytecode::GetGasRemaining,
            Opcodes::GET_TXN_SENDER => Bytecode::GetTxnSenderAddress,
            Opcodes::EXISTS => {
                let idx = read_uleb_u16_internal(cursor)?;
                let types_idx = read_uleb_u16_internal(cursor)?;
                Bytecode::Exists(StructDefinitionIndex(idx), LocalsSignatureIndex(types_idx))
            }
            Opcodes::BORROW_REF => {
                let idx = read_uleb_u16_internal(cursor)?;
                let types_idx = read_uleb_u16_internal(cursor)?;
                Bytecode::BorrowGlobal(StructDefinitionIndex(idx), LocalsSignatureIndex(types_idx))
            }
            Opcodes::RELEASE_REF => Bytecode::ReleaseRef,
            Opcodes::MOVE_FROM => {
                let idx = read_uleb_u16_internal(cursor)?;
                let types_idx = read_uleb_u16_internal(cursor)?;
                Bytecode::MoveFrom(StructDefinitionIndex(idx), LocalsSignatureIndex(types_idx))
            }
            Opcodes::MOVE_TO => {
                let idx = read_uleb_u16_internal(cursor)?;
                let types_idx = read_uleb_u16_internal(cursor)?;
                Bytecode::MoveToSender(StructDefinitionIndex(idx), LocalsSignatureIndex(types_idx))
            }
            Opcodes::CREATE_ACCOUNT => Bytecode::CreateAccount,
            Opcodes::EMIT_EVENT => Bytecode::EmitEvent,
            Opcodes::GET_TXN_SEQUENCE_NUMBER => Bytecode::GetTxnSequenceNumber,
            Opcodes::GET_TXN_PUBLIC_KEY => Bytecode::GetTxnPublicKey,
            Opcodes::FREEZE_REF => Bytecode::FreezeRef,
        };
        code.push(bytecode);
    }
    Ok(())
}

//
// Helpers to read uleb128 and uncompressed integers
//

fn read_uleb_u16_internal(cursor: &mut Cursor<&[u8]>) -> BinaryLoaderResult<u16> {
    read_uleb128_as_u16(cursor).map_err(|_| BinaryError::Malformed)
}

fn read_uleb_u32_internal(cursor: &mut Cursor<&[u8]>) -> BinaryLoaderResult<u32> {
    read_uleb128_as_u32(cursor).map_err(|_| BinaryError::Malformed)
}

fn read_u16_internal(cursor: &mut Cursor<&[u8]>) -> BinaryLoaderResult<u16> {
    cursor
        .read_u16::<LittleEndian>()
        .map_err(|_| BinaryError::Malformed)
}

fn read_u32_internal(cursor: &mut Cursor<&[u8]>) -> BinaryLoaderResult<u32> {
    cursor
        .read_u32::<LittleEndian>()
        .map_err(|_| BinaryError::Malformed)
}

fn read_u64_internal(cursor: &mut Cursor<&[u8]>) -> BinaryLoaderResult<u64> {
    cursor
        .read_u64::<LittleEndian>()
        .map_err(|_| BinaryError::Malformed)
}

impl TableType {
    fn from_u8(value: u8) -> BinaryLoaderResult<TableType> {
        match value {
            0x1 => Ok(TableType::MODULE_HANDLES),
            0x2 => Ok(TableType::STRUCT_HANDLES),
            0x3 => Ok(TableType::FUNCTION_HANDLES),
            0x4 => Ok(TableType::ADDRESS_POOL),
            0x5 => Ok(TableType::STRING_POOL),
            0x6 => Ok(TableType::BYTE_ARRAY_POOL),
            0x7 => Ok(TableType::MAIN),
            0x8 => Ok(TableType::STRUCT_DEFS),
            0x9 => Ok(TableType::FIELD_DEFS),
            0xA => Ok(TableType::FUNCTION_DEFS),
            0xB => Ok(TableType::TYPE_SIGNATURES),
            0xC => Ok(TableType::FUNCTION_SIGNATURES),
            0xD => Ok(TableType::LOCALS_SIGNATURES),
            _ => Err(BinaryError::UnknownTableType),
        }
    }
}

#[allow(dead_code)]
impl SignatureType {
    fn from_u8(value: u8) -> BinaryLoaderResult<SignatureType> {
        match value {
            0x1 => Ok(SignatureType::TYPE_SIGNATURE),
            0x2 => Ok(SignatureType::FUNCTION_SIGNATURE),
            0x3 => Ok(SignatureType::LOCAL_SIGNATURE),
            _ => Err(BinaryError::UnknownSignatureType),
        }
    }
}

impl SerializedType {
    fn from_u8(value: u8) -> BinaryLoaderResult<SerializedType> {
        match value {
            0x1 => Ok(SerializedType::BOOL),
            0x2 => Ok(SerializedType::INTEGER),
            0x3 => Ok(SerializedType::STRING),
            0x4 => Ok(SerializedType::ADDRESS),
            0x5 => Ok(SerializedType::REFERENCE),
            0x6 => Ok(SerializedType::MUTABLE_REFERENCE),
            0x7 => Ok(SerializedType::STRUCT),
            0x8 => Ok(SerializedType::BYTEARRAY),
            0x9 => Ok(SerializedType::TYPE_PARAMETER),
            _ => Err(BinaryError::UnknownSerializedType),
        }
    }
}

impl SerializedKind {
    fn from_u8(value: u8) -> BinaryLoaderResult<SerializedKind> {
        match value {
            0x1 => Ok(SerializedKind::RESOURCE),
            0x2 => Ok(SerializedKind::COPYABLE),
            _ => Err(BinaryError::UnknownSerializedType),
        }
    }
}

impl Opcodes {
    fn from_u8(value: u8) -> BinaryLoaderResult<Opcodes> {
        match value {
            0x01 => Ok(Opcodes::POP),
            0x02 => Ok(Opcodes::RET),
            0x03 => Ok(Opcodes::BR_TRUE),
            0x04 => Ok(Opcodes::BR_FALSE),
            0x05 => Ok(Opcodes::BRANCH),
            0x06 => Ok(Opcodes::LD_CONST),
            0x07 => Ok(Opcodes::LD_ADDR),
            0x08 => Ok(Opcodes::LD_STR),
            0x09 => Ok(Opcodes::LD_TRUE),
            0x0A => Ok(Opcodes::LD_FALSE),
            0x0B => Ok(Opcodes::COPY_LOC),
            0x0C => Ok(Opcodes::MOVE_LOC),
            0x0D => Ok(Opcodes::ST_LOC),
            0x0E => Ok(Opcodes::LD_REF_LOC),
            0x0F => Ok(Opcodes::LD_REF_FIELD),
            0x10 => Ok(Opcodes::LD_BYTEARRAY),
            0x11 => Ok(Opcodes::CALL),
            0x12 => Ok(Opcodes::PACK),
            0x13 => Ok(Opcodes::UNPACK),
            0x14 => Ok(Opcodes::READ_REF),
            0x15 => Ok(Opcodes::WRITE_REF),
            0x16 => Ok(Opcodes::ADD),
            0x17 => Ok(Opcodes::SUB),
            0x18 => Ok(Opcodes::MUL),
            0x19 => Ok(Opcodes::MOD),
            0x1A => Ok(Opcodes::DIV),
            0x1B => Ok(Opcodes::BIT_OR),
            0x1C => Ok(Opcodes::BIT_AND),
            0x1D => Ok(Opcodes::XOR),
            0x1E => Ok(Opcodes::OR),
            0x1F => Ok(Opcodes::AND),
            0x20 => Ok(Opcodes::NOT),
            0x21 => Ok(Opcodes::EQ),
            0x22 => Ok(Opcodes::NEQ),
            0x23 => Ok(Opcodes::LT),
            0x24 => Ok(Opcodes::GT),
            0x25 => Ok(Opcodes::LE),
            0x26 => Ok(Opcodes::GE),
            0x27 => Ok(Opcodes::ABORT),
            0x28 => Ok(Opcodes::GET_TXN_GAS_UNIT_PRICE),
            0x29 => Ok(Opcodes::GET_TXN_MAX_GAS_UNITS),
            0x2A => Ok(Opcodes::GET_GAS_REMAINING),
            0x2B => Ok(Opcodes::GET_TXN_SENDER),
            0x2C => Ok(Opcodes::EXISTS),
            0x2D => Ok(Opcodes::BORROW_REF),
            0x2E => Ok(Opcodes::RELEASE_REF),
            0x2F => Ok(Opcodes::MOVE_FROM),
            0x30 => Ok(Opcodes::MOVE_TO),
            0x31 => Ok(Opcodes::CREATE_ACCOUNT),
            0x32 => Ok(Opcodes::EMIT_EVENT),
            0x33 => Ok(Opcodes::GET_TXN_SEQUENCE_NUMBER),
            0x34 => Ok(Opcodes::GET_TXN_PUBLIC_KEY),
            0x35 => Ok(Opcodes::FREEZE_REF),
            _ => Err(BinaryError::UnknownOpcode),
        }
    }
}
