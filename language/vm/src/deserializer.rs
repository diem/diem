// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{errors::*, file_format::*, file_format_common::*};
use byteorder::{LittleEndian, ReadBytesExt};
use libra_types::{
    account_address::AccountAddress,
    vm_error::{StatusCode, VMStatus},
};
use move_core_types::identifier::Identifier;
use std::{
    collections::HashSet,
    convert::TryInto,
    io::{Cursor, Read},
};

impl CompiledScript {
    /// Deserializes a &[u8] slice into a `CompiledScript` instance.
    pub fn deserialize(binary: &[u8]) -> BinaryLoaderResult<Self> {
        let deserialized = CompiledScriptMut::deserialize_no_check_bounds(binary)?;
        deserialized
            .freeze()
            .map_err(|_| VMStatus::new(StatusCode::MALFORMED))
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
        deserialized
            .freeze()
            .map_err(|_| VMStatus::new(StatusCode::MALFORMED))
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
        if count != BinaryConstants::LIBRA_MAGIC_SIZE || magic != BinaryConstants::LIBRA_MAGIC {
            return Err(VMStatus::new(StatusCode::BAD_MAGIC));
        }
    } else {
        return Err(
            VMStatus::new(StatusCode::MALFORMED).with_message("Bad binary header".to_string())
        );
    }
    let major_ver = 1u8;
    let minor_ver = 0u8;
    if let Ok(ver) = cursor.read_u8() {
        if ver != major_ver {
            return Err(VMStatus::new(StatusCode::UNKNOWN_VERSION));
        }
    } else {
        return Err(
            VMStatus::new(StatusCode::MALFORMED).with_message("Bad binary header".to_string())
        );
    }
    if let Ok(ver) = cursor.read_u8() {
        if ver != minor_ver {
            return Err(VMStatus::new(StatusCode::UNKNOWN_VERSION));
        }
    } else {
        return Err(
            VMStatus::new(StatusCode::MALFORMED).with_message("Bad binary header".to_string())
        );
    }
    cursor.read_u8().map_err(|_| {
        VMStatus::new(StatusCode::MALFORMED).with_message("Bad binary header".to_string())
    })
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
        Err(VMStatus::new(StatusCode::MALFORMED).with_message("Error reading table".to_string()))
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
            return Err(VMStatus::new(StatusCode::BAD_HEADER_TABLE));
        }
        if table.count == 0 {
            return Err(VMStatus::new(StatusCode::BAD_HEADER_TABLE));
        }
        let count = u64::from(table.count);
        if let Some(checked_offset) = current_offset.checked_add(count) {
            current_offset = checked_offset;
        }
        if current_offset > length {
            return Err(VMStatus::new(StatusCode::BAD_HEADER_TABLE));
        }
        if !table_types.insert(table.kind) {
            return Err(VMStatus::new(StatusCode::DUPLICATE_TABLE));
        }
    }
    if current_offset != length {
        return Err(VMStatus::new(StatusCode::BAD_HEADER_TABLE));
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
    fn get_function_instantiations(&mut self) -> &mut Vec<FunctionInstantiation>;
    fn get_signatures(&mut self) -> &mut SignaturePool;
    fn get_identifiers(&mut self) -> &mut IdentifierPool;
    fn get_address_identifiers(&mut self) -> &mut AddressIdentifierPool;
    fn get_constant_pool(&mut self) -> &mut ConstantPool;
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

    fn get_function_instantiations(&mut self) -> &mut Vec<FunctionInstantiation> {
        &mut self.function_instantiations
    }

    fn get_signatures(&mut self) -> &mut SignaturePool {
        &mut self.signatures
    }

    fn get_identifiers(&mut self) -> &mut IdentifierPool {
        &mut self.identifiers
    }

    fn get_address_identifiers(&mut self) -> &mut AddressIdentifierPool {
        &mut self.address_identifiers
    }

    fn get_constant_pool(&mut self) -> &mut ConstantPool {
        &mut self.constant_pool
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

    fn get_function_instantiations(&mut self) -> &mut Vec<FunctionInstantiation> {
        &mut self.function_instantiations
    }

    fn get_signatures(&mut self) -> &mut SignaturePool {
        &mut self.signatures
    }

    fn get_identifiers(&mut self) -> &mut IdentifierPool {
        &mut self.identifiers
    }

    fn get_address_identifiers(&mut self) -> &mut AddressIdentifierPool {
        &mut self.address_identifiers
    }

    fn get_constant_pool(&mut self) -> &mut ConstantPool {
        &mut self.constant_pool
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
            TableType::FUNCTION_INST => {
                load_function_instantiations(binary, table, common.get_function_instantiations())?;
            }
            TableType::SIGNATURES => {
                load_signatures(binary, table, common.get_signatures())?;
            }
            TableType::CONSTANT_POOL => {
                load_constant_pool(binary, table, common.get_constant_pool())?;
            }
            TableType::IDENTIFIERS => {
                load_identifiers(binary, table, common.get_identifiers())?;
            }
            TableType::ADDRESS_IDENTIFIERS => {
                load_address_identifiers(binary, table, common.get_address_identifiers())?;
            }
            TableType::FUNCTION_DEFS
            | TableType::STRUCT_DEFS
            | TableType::STRUCT_DEF_INST
            | TableType::FIELD_HANDLE
            | TableType::FIELD_INST
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
            TableType::STRUCT_DEF_INST => {
                load_struct_instantiations(binary, table, &mut module.struct_def_instantiations)?;
            }
            TableType::FUNCTION_DEFS => {
                load_function_defs(binary, table, &mut module.function_defs)?;
            }
            TableType::FIELD_HANDLE => {
                load_field_handles(binary, table, &mut module.field_handles)?;
            }
            TableType::FIELD_INST => {
                load_field_instantiations(binary, table, &mut module.field_instantiations)?;
            }
            TableType::MODULE_HANDLES
            | TableType::STRUCT_HANDLES
            | TableType::FUNCTION_HANDLES
            | TableType::FUNCTION_INST
            | TableType::IDENTIFIERS
            | TableType::ADDRESS_IDENTIFIERS
            | TableType::CONSTANT_POOL
            | TableType::SIGNATURES => {
                continue;
            }
            TableType::MAIN => {
                return Err(VMStatus::new(StatusCode::MALFORMED)
                    .with_message("Bad table in Module".to_string()))
            }
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
                // `check_tables()` ensures that the table indices are in bounds
                assume!(start <= usize::max_value() - (table.count as usize));
                let end: usize = start + table.count as usize;
                let mut cursor = Cursor::new(&binary[start..end]);
                let main = load_function_def(&mut cursor)?;
                script.main = main;
            }
            TableType::MODULE_HANDLES
            | TableType::STRUCT_HANDLES
            | TableType::FUNCTION_HANDLES
            | TableType::FUNCTION_INST
            | TableType::SIGNATURES
            | TableType::IDENTIFIERS
            | TableType::ADDRESS_IDENTIFIERS
            | TableType::CONSTANT_POOL => {
                continue;
            }
            TableType::STRUCT_DEFS
            | TableType::STRUCT_DEF_INST
            | TableType::FUNCTION_DEFS
            | TableType::FIELD_INST
            | TableType::FIELD_HANDLE => {
                return Err(VMStatus::new(StatusCode::MALFORMED)
                    .with_message("Bad table in Script".to_string()));
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
            address: AddressIdentifierIndex(address),
            name: IdentifierIndex(name),
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
        let is_nominal_resource = load_nominal_resource_flag(&mut cursor)?;
        let type_parameters = load_kinds(&mut cursor)?;
        struct_handles.push(StructHandle {
            module: ModuleHandleIndex(module_handle),
            name: IdentifierIndex(name),
            is_nominal_resource,
            type_parameters,
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
        let parameters = read_uleb_u16_internal(&mut cursor)?;
        let return_ = read_uleb_u16_internal(&mut cursor)?;
        let type_parameters = load_kinds(&mut cursor)?;

        function_handles.push(FunctionHandle {
            module: ModuleHandleIndex(module_handle),
            name: IdentifierIndex(name),
            parameters: SignatureIndex(parameters),
            return_: SignatureIndex(return_),
            type_parameters,
        });
    }
    Ok(())
}

/// Builds the `StructInstantiation` table.
fn load_struct_instantiations(
    binary: &[u8],
    table: &Table,
    struct_insts: &mut Vec<StructDefInstantiation>,
) -> BinaryLoaderResult<()> {
    let start = table.offset as usize;
    let end = start + table.count as usize;
    let mut cursor = Cursor::new(&binary[start..end]);
    loop {
        if cursor.position() == u64::from(table.count) {
            break;
        }
        let def = read_uleb_u16_internal(&mut cursor)?;
        let type_parameters = read_uleb_u16_internal(&mut cursor)?;
        struct_insts.push(StructDefInstantiation {
            def: StructDefinitionIndex(def),
            type_parameters: SignatureIndex(type_parameters),
        });
    }
    Ok(())
}

/// Builds the `FunctionInstantiation` table.
fn load_function_instantiations(
    binary: &[u8],
    table: &Table,
    func_insts: &mut Vec<FunctionInstantiation>,
) -> BinaryLoaderResult<()> {
    let start = table.offset as usize;
    let end = start + table.count as usize;
    let mut cursor = Cursor::new(&binary[start..end]);
    loop {
        if cursor.position() == u64::from(table.count) {
            break;
        }
        let handle = read_uleb_u16_internal(&mut cursor)?;
        let type_parameters = read_uleb_u16_internal(&mut cursor)?;
        func_insts.push(FunctionInstantiation {
            handle: FunctionHandleIndex(handle),
            type_parameters: SignatureIndex(type_parameters),
        });
    }
    Ok(())
}

/// Builds the `IdentifierPool`.
fn load_identifiers(
    binary: &[u8],
    table: &Table,
    identifiers: &mut IdentifierPool,
) -> BinaryLoaderResult<()> {
    let start = table.offset as usize;
    let end = start + table.count as usize;
    let mut cursor = Cursor::new(&binary[start..end]);
    while cursor.position() < u64::from(table.count) {
        let size = read_uleb_u32_internal(&mut cursor)? as usize;
        if size > std::u16::MAX as usize {
            return Err(VMStatus::new(StatusCode::MALFORMED)
                .with_message("Bad Identifier pool size".to_string()));
        }
        let mut buffer: Vec<u8> = vec![0u8; size];
        if let Ok(count) = cursor.read(&mut buffer) {
            if count != size {
                return Err(VMStatus::new(StatusCode::MALFORMED)
                    .with_message("Bad Identifier pool size".to_string()));
            }
            let s = Identifier::from_utf8(buffer).map_err(|_| {
                VMStatus::new(StatusCode::MALFORMED).with_message("Invalid Identifier".to_string())
            })?;

            identifiers.push(s);
        }
    }
    Ok(())
}

/// Builds the `AddressIdentifierPool`.
fn load_address_identifiers(
    binary: &[u8],
    table: &Table,
    addresses: &mut AddressIdentifierPool,
) -> BinaryLoaderResult<()> {
    let mut start = table.offset as usize;
    if table.count as usize % AccountAddress::LENGTH != 0 {
        return Err(VMStatus::new(StatusCode::MALFORMED)
            .with_message("Bad Address Identifier pool size".to_string()));
    }
    for _i in 0..table.count as usize / AccountAddress::LENGTH {
        let end_addr = start + AccountAddress::LENGTH;
        let address = (&binary[start..end_addr]).try_into();
        if address.is_err() {
            return Err(VMStatus::new(StatusCode::MALFORMED)
                .with_message("Invalid Address format".to_string()));
        }
        start = end_addr;

        addresses.push(address.unwrap());
    }
    Ok(())
}

/// Builds the `ConstantPool`.
fn load_constant_pool(
    binary: &[u8],
    table: &Table,
    constants: &mut ConstantPool,
) -> BinaryLoaderResult<()> {
    let start = table.offset as usize;
    let end = start + table.count as usize;
    let mut cursor = Cursor::new(&binary[start..end]);
    while cursor.position() < u64::from(table.count) {
        constants.push(load_constant(&mut cursor)?)
    }
    Ok(())
}

/// Build a single `Constant`
fn load_constant(cursor: &mut Cursor<&[u8]>) -> BinaryLoaderResult<Constant> {
    let type_ = load_signature_token(cursor)?;
    let size = read_uleb_u32_internal(cursor)? as usize;
    if size > std::u16::MAX as usize {
        return Err(
            VMStatus::new(StatusCode::MALFORMED).with_message("Bad Constant data size".to_string())
        );
    }
    let mut data: Vec<u8> = vec![0u8; size];
    let count = cursor.read(&mut data).map_err(|_| {
        VMStatus::new(StatusCode::MALFORMED).with_message("Unexpected end of table".to_string())
    })?;
    if count != size {
        return Err(
            VMStatus::new(StatusCode::MALFORMED).with_message("Bad Constant data size".to_string())
        );
    }
    Ok(Constant { type_, data })
}

/// Builds the `SignaturePool`.
fn load_signatures(
    binary: &[u8],
    table: &Table,
    signatures: &mut SignaturePool,
) -> BinaryLoaderResult<()> {
    let start = table.offset as usize;
    let end = start + table.count as usize;
    let mut cursor = Cursor::new(&binary[start..end]);
    while cursor.position() < u64::from(table.count) {
        signatures.push(Signature(load_signature_tokens(&mut cursor)?));
    }
    Ok(())
}

fn load_signature_tokens(cursor: &mut Cursor<&[u8]>) -> BinaryLoaderResult<Vec<SignatureToken>> {
    let len = cursor.read_u8().map_err(|_| {
        VMStatus::new(StatusCode::MALFORMED).with_message("Unexpected EOF".to_string())
    })?;
    let mut tokens = vec![];
    for _ in 0..len {
        tokens.push(load_signature_token(cursor)?);
    }
    Ok(tokens)
}

/// Deserializes a `SignatureToken`.
fn load_signature_token(cursor: &mut Cursor<&[u8]>) -> BinaryLoaderResult<SignatureToken> {
    if let Ok(byte) = cursor.read_u8() {
        match SerializedType::from_u8(byte)? {
            SerializedType::BOOL => Ok(SignatureToken::Bool),
            SerializedType::U8 => Ok(SignatureToken::U8),
            SerializedType::U64 => Ok(SignatureToken::U64),
            SerializedType::U128 => Ok(SignatureToken::U128),
            SerializedType::ADDRESS => Ok(SignatureToken::Address),
            SerializedType::VECTOR => {
                let ty = load_signature_token(cursor)?;
                Ok(SignatureToken::Vector(Box::new(ty)))
            }
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
                Ok(SignatureToken::Struct(StructHandleIndex(sh_idx)))
            }
            SerializedType::STRUCT_INST => {
                let sh_idx = read_uleb_u16_internal(cursor)?;
                let type_params = load_signature_tokens(cursor)?;
                Ok(SignatureToken::StructInstantiation(
                    StructHandleIndex(sh_idx),
                    type_params,
                ))
            }
            SerializedType::TYPE_PARAMETER => {
                let idx = read_uleb_u16_internal(cursor)?;
                Ok(SignatureToken::TypeParameter(idx))
            }
        }
    } else {
        Err(VMStatus::new(StatusCode::MALFORMED).with_message("Unexpected EOF".to_string()))
    }
}

fn load_nominal_resource_flag(cursor: &mut Cursor<&[u8]>) -> BinaryLoaderResult<bool> {
    if let Ok(byte) = cursor.read_u8() {
        Ok(match SerializedNominalResourceFlag::from_u8(byte)? {
            SerializedNominalResourceFlag::NOMINAL_RESOURCE => true,
            SerializedNominalResourceFlag::NORMAL_STRUCT => false,
        })
    } else {
        Err(VMStatus::new(StatusCode::MALFORMED).with_message("Unexpected EOF".to_string()))
    }
}

fn load_kind(cursor: &mut Cursor<&[u8]>) -> BinaryLoaderResult<Kind> {
    if let Ok(byte) = cursor.read_u8() {
        Ok(match SerializedKind::from_u8(byte)? {
            SerializedKind::ALL => Kind::All,
            SerializedKind::COPYABLE => Kind::Copyable,
            SerializedKind::RESOURCE => Kind::Resource,
        })
    } else {
        Err(VMStatus::new(StatusCode::MALFORMED).with_message("Unexpected EOF".to_string()))
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
        let field_information_flag = match cursor.read_u8() {
            Ok(byte) => SerializedNativeStructFlag::from_u8(byte)?,
            Err(_) => {
                return Err(VMStatus::new(StatusCode::MALFORMED)
                    .with_message("Invalid field info in struct".to_string()))
            }
        };
        let field_information = match field_information_flag {
            SerializedNativeStructFlag::NATIVE => StructFieldInformation::Native,
            SerializedNativeStructFlag::DECLARED => {
                let fields = load_field_defs(&mut cursor)?;
                StructFieldInformation::Declared(fields)
            }
        };
        struct_defs.push(StructDefinition {
            struct_handle: StructHandleIndex(struct_handle),
            field_information,
        });
    }
    Ok(())
}

fn load_field_defs(cursor: &mut Cursor<&[u8]>) -> BinaryLoaderResult<Vec<FieldDefinition>> {
    let mut fields = Vec::new();
    let field_count = read_uleb_u32_internal(cursor)?;
    for _ in 0..field_count {
        fields.push(load_field_def(cursor)?);
    }
    Ok(fields)
}

fn load_field_def(cursor: &mut Cursor<&[u8]>) -> BinaryLoaderResult<FieldDefinition> {
    let name = read_uleb_u16_internal(cursor)?;
    let signature = load_signature_token(cursor)?;
    Ok(FieldDefinition {
        name: IdentifierIndex(name),
        signature: TypeSignature(signature),
    })
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

fn load_field_handles(
    binary: &[u8],
    table: &Table,
    field_handles: &mut Vec<FieldHandle>,
) -> BinaryLoaderResult<()> {
    let start = table.offset as usize;
    let end = start + table.count as usize;
    let mut cursor = Cursor::new(&binary[start..end]);
    loop {
        if cursor.position() == u64::from(table.count) {
            break;
        }
        let struct_idx = read_uleb_u16_internal(&mut cursor)?;
        let offset = read_uleb_u16_internal(&mut cursor)?;
        field_handles.push(FieldHandle {
            owner: StructDefinitionIndex(struct_idx),
            field: offset,
        });
    }
    Ok(())
}

fn load_field_instantiations(
    binary: &[u8],
    table: &Table,
    field_insts: &mut Vec<FieldInstantiation>,
) -> BinaryLoaderResult<()> {
    let start = table.offset as usize;
    let end = start + table.count as usize;
    let mut cursor = Cursor::new(&binary[start..end]);
    loop {
        if cursor.position() == u64::from(table.count) {
            break;
        }
        let handle = read_uleb_u16_internal(&mut cursor)?;
        let inst = read_uleb_u16_internal(&mut cursor)?;
        field_insts.push(FieldInstantiation {
            handle: FieldHandleIndex(handle),
            type_parameters: SignatureIndex(inst),
        });
    }
    Ok(())
}

/// Deserializes a `FunctionDefinition`.
fn load_function_def(cursor: &mut Cursor<&[u8]>) -> BinaryLoaderResult<FunctionDefinition> {
    let function = read_uleb_u16_internal(cursor)?;

    let flags = cursor.read_u8().map_err(|_| {
        VMStatus::new(StatusCode::MALFORMED).with_message("Unexpected EOF".to_string())
    })?;
    let acquires_global_resources = load_struct_definition_indices(cursor)?;
    let code_unit = load_code_unit(cursor)?;
    Ok(FunctionDefinition {
        function: FunctionHandleIndex(function),
        flags,
        acquires_global_resources,
        code: code_unit,
    })
}

/// Deserializes a `Vec<StructDefinitionIndex>`.
fn load_struct_definition_indices(
    cursor: &mut Cursor<&[u8]>,
) -> BinaryLoaderResult<Vec<StructDefinitionIndex>> {
    let len = cursor.read_u8().map_err(|_| {
        VMStatus::new(StatusCode::MALFORMED).with_message("Unexpected EOF".to_string())
    })?;
    let mut indices = vec![];
    for _ in 0..len {
        indices.push(StructDefinitionIndex(read_uleb_u16_internal(cursor)?));
    }
    Ok(indices)
}

/// Deserializes a `CodeUnit`.
fn load_code_unit(cursor: &mut Cursor<&[u8]>) -> BinaryLoaderResult<CodeUnit> {
    let max_stack_size = read_uleb_u16_internal(cursor)?;
    let locals = read_uleb_u16_internal(cursor)?;

    let mut code_unit = CodeUnit {
        max_stack_size,
        locals: SignatureIndex(locals),
        code: vec![],
    };

    load_code(cursor, &mut code_unit.code)?;
    Ok(code_unit)
}

/// Deserializes a code stream (`Bytecode`s).
fn load_code(cursor: &mut Cursor<&[u8]>, code: &mut Vec<Bytecode>) -> BinaryLoaderResult<()> {
    let bytecode_count = read_u16_internal(cursor)?;
    while code.len() < bytecode_count as usize {
        let byte = cursor.read_u8().map_err(|_| {
            VMStatus::new(StatusCode::MALFORMED).with_message("Unexpected EOF".to_string())
        })?;
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
            Opcodes::LD_U8 => {
                let value = cursor.read_u8().map_err(|_| {
                    VMStatus::new(StatusCode::MALFORMED).with_message("Unexpected EOF".to_string())
                })?;
                Bytecode::LdU8(value)
            }
            Opcodes::LD_U64 => {
                let value = read_u64_internal(cursor)?;
                Bytecode::LdU64(value)
            }
            Opcodes::LD_U128 => {
                let value = read_u128_internal(cursor)?;
                Bytecode::LdU128(value)
            }
            Opcodes::CAST_U8 => Bytecode::CastU8,
            Opcodes::CAST_U64 => Bytecode::CastU64,
            Opcodes::CAST_U128 => Bytecode::CastU128,
            Opcodes::LD_CONST => {
                let idx = read_uleb_u16_internal(cursor)?;
                Bytecode::LdConst(ConstantPoolIndex(idx))
            }
            Opcodes::LD_TRUE => Bytecode::LdTrue,
            Opcodes::LD_FALSE => Bytecode::LdFalse,
            Opcodes::COPY_LOC => {
                let idx = cursor.read_u8().map_err(|_| {
                    VMStatus::new(StatusCode::MALFORMED).with_message("Unexpected EOF".to_string())
                })?;
                Bytecode::CopyLoc(idx)
            }
            Opcodes::MOVE_LOC => {
                let idx = cursor.read_u8().map_err(|_| {
                    VMStatus::new(StatusCode::MALFORMED).with_message("Unexpected EOF".to_string())
                })?;
                Bytecode::MoveLoc(idx)
            }
            Opcodes::ST_LOC => {
                let idx = cursor.read_u8().map_err(|_| {
                    VMStatus::new(StatusCode::MALFORMED).with_message("Unexpected EOF".to_string())
                })?;
                Bytecode::StLoc(idx)
            }
            Opcodes::MUT_BORROW_LOC => {
                let idx = cursor.read_u8().map_err(|_| {
                    VMStatus::new(StatusCode::MALFORMED).with_message("Unexpected EOF".to_string())
                })?;
                Bytecode::MutBorrowLoc(idx)
            }
            Opcodes::IMM_BORROW_LOC => {
                let idx = cursor.read_u8().map_err(|_| {
                    VMStatus::new(StatusCode::MALFORMED).with_message("Unexpected EOF".to_string())
                })?;
                Bytecode::ImmBorrowLoc(idx)
            }
            Opcodes::MUT_BORROW_FIELD => {
                let idx = read_uleb_u16_internal(cursor)?;
                Bytecode::MutBorrowField(FieldHandleIndex(idx))
            }
            Opcodes::MUT_BORROW_FIELD_GENERIC => {
                let idx = read_uleb_u16_internal(cursor)?;
                Bytecode::MutBorrowFieldGeneric(FieldInstantiationIndex(idx))
            }
            Opcodes::IMM_BORROW_FIELD => {
                let idx = read_uleb_u16_internal(cursor)?;
                Bytecode::ImmBorrowField(FieldHandleIndex(idx))
            }
            Opcodes::IMM_BORROW_FIELD_GENERIC => {
                let idx = read_uleb_u16_internal(cursor)?;
                Bytecode::ImmBorrowFieldGeneric(FieldInstantiationIndex(idx))
            }
            Opcodes::CALL => {
                let idx = read_uleb_u16_internal(cursor)?;
                Bytecode::Call(FunctionHandleIndex(idx))
            }
            Opcodes::CALL_GENERIC => {
                let idx = read_uleb_u16_internal(cursor)?;
                Bytecode::CallGeneric(FunctionInstantiationIndex(idx))
            }
            Opcodes::PACK => {
                let idx = read_uleb_u16_internal(cursor)?;
                Bytecode::Pack(StructDefinitionIndex(idx))
            }
            Opcodes::PACK_GENERIC => {
                let idx = read_uleb_u16_internal(cursor)?;
                Bytecode::PackGeneric(StructDefInstantiationIndex(idx))
            }
            Opcodes::UNPACK => {
                let idx = read_uleb_u16_internal(cursor)?;
                Bytecode::Unpack(StructDefinitionIndex(idx))
            }
            Opcodes::UNPACK_GENERIC => {
                let idx = read_uleb_u16_internal(cursor)?;
                Bytecode::UnpackGeneric(StructDefInstantiationIndex(idx))
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
            Opcodes::SHL => Bytecode::Shl,
            Opcodes::SHR => Bytecode::Shr,
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
                Bytecode::Exists(StructDefinitionIndex(idx))
            }
            Opcodes::EXISTS_GENERIC => {
                let idx = read_uleb_u16_internal(cursor)?;
                Bytecode::ExistsGeneric(StructDefInstantiationIndex(idx))
            }
            Opcodes::MUT_BORROW_GLOBAL => {
                let idx = read_uleb_u16_internal(cursor)?;
                Bytecode::MutBorrowGlobal(StructDefinitionIndex(idx))
            }
            Opcodes::MUT_BORROW_GLOBAL_GENERIC => {
                let idx = read_uleb_u16_internal(cursor)?;
                Bytecode::MutBorrowGlobalGeneric(StructDefInstantiationIndex(idx))
            }
            Opcodes::IMM_BORROW_GLOBAL => {
                let idx = read_uleb_u16_internal(cursor)?;
                Bytecode::ImmBorrowGlobal(StructDefinitionIndex(idx))
            }
            Opcodes::IMM_BORROW_GLOBAL_GENERIC => {
                let idx = read_uleb_u16_internal(cursor)?;
                Bytecode::ImmBorrowGlobalGeneric(StructDefInstantiationIndex(idx))
            }
            Opcodes::MOVE_FROM => {
                let idx = read_uleb_u16_internal(cursor)?;
                Bytecode::MoveFrom(StructDefinitionIndex(idx))
            }
            Opcodes::MOVE_FROM_GENERIC => {
                let idx = read_uleb_u16_internal(cursor)?;
                Bytecode::MoveFromGeneric(StructDefInstantiationIndex(idx))
            }
            Opcodes::MOVE_TO => {
                let idx = read_uleb_u16_internal(cursor)?;
                Bytecode::MoveToSender(StructDefinitionIndex(idx))
            }
            Opcodes::MOVE_TO_GENERIC => {
                let idx = read_uleb_u16_internal(cursor)?;
                Bytecode::MoveToSenderGeneric(StructDefInstantiationIndex(idx))
            }
            Opcodes::GET_TXN_SEQUENCE_NUMBER => Bytecode::GetTxnSequenceNumber,
            Opcodes::GET_TXN_PUBLIC_KEY => Bytecode::GetTxnPublicKey,
            Opcodes::FREEZE_REF => Bytecode::FreezeRef,
            Opcodes::NOP => Bytecode::Nop,
        };
        code.push(bytecode);
    }
    Ok(())
}

//
// Helpers to read uleb128 and uncompressed integers
//

fn read_uleb_u16_internal(cursor: &mut Cursor<&[u8]>) -> BinaryLoaderResult<u16> {
    read_uleb128_as_u16(cursor).map_err(|_| VMStatus::new(StatusCode::BAD_ULEB_U16))
}

fn read_uleb_u32_internal(cursor: &mut Cursor<&[u8]>) -> BinaryLoaderResult<u32> {
    read_uleb128_as_u32(cursor).map_err(|_| VMStatus::new(StatusCode::BAD_ULEB_U32))
}

fn read_u16_internal(cursor: &mut Cursor<&[u8]>) -> BinaryLoaderResult<u16> {
    cursor
        .read_u16::<LittleEndian>()
        .map_err(|_| VMStatus::new(StatusCode::BAD_U16))
}

fn read_u32_internal(cursor: &mut Cursor<&[u8]>) -> BinaryLoaderResult<u32> {
    cursor
        .read_u32::<LittleEndian>()
        .map_err(|_| VMStatus::new(StatusCode::BAD_U32))
}

fn read_u64_internal(cursor: &mut Cursor<&[u8]>) -> BinaryLoaderResult<u64> {
    cursor
        .read_u64::<LittleEndian>()
        .map_err(|_| VMStatus::new(StatusCode::BAD_U64))
}

fn read_u128_internal(cursor: &mut Cursor<&[u8]>) -> BinaryLoaderResult<u128> {
    cursor
        .read_u128::<LittleEndian>()
        .map_err(|_| VMStatus::new(StatusCode::BAD_U128))
}

impl TableType {
    fn from_u8(value: u8) -> BinaryLoaderResult<TableType> {
        match value {
            0x1 => Ok(TableType::MODULE_HANDLES),
            0x2 => Ok(TableType::STRUCT_HANDLES),
            0x3 => Ok(TableType::FUNCTION_HANDLES),
            0x4 => Ok(TableType::FUNCTION_INST),
            0x5 => Ok(TableType::SIGNATURES),
            0x6 => Ok(TableType::CONSTANT_POOL),
            0x7 => Ok(TableType::IDENTIFIERS),
            0x8 => Ok(TableType::ADDRESS_IDENTIFIERS),
            0x9 => Ok(TableType::MAIN),
            0xA => Ok(TableType::STRUCT_DEFS),
            0xB => Ok(TableType::STRUCT_DEF_INST),
            0xC => Ok(TableType::FUNCTION_DEFS),
            0xD => Ok(TableType::FIELD_HANDLE),
            0xE => Ok(TableType::FIELD_INST),
            _ => Err(VMStatus::new(StatusCode::UNKNOWN_TABLE_TYPE)),
        }
    }
}

impl SerializedType {
    fn from_u8(value: u8) -> BinaryLoaderResult<SerializedType> {
        match value {
            0x1 => Ok(SerializedType::BOOL),
            0x2 => Ok(SerializedType::U8),
            0x3 => Ok(SerializedType::U64),
            0x4 => Ok(SerializedType::U128),
            0x5 => Ok(SerializedType::ADDRESS),
            0x6 => Ok(SerializedType::REFERENCE),
            0x7 => Ok(SerializedType::MUTABLE_REFERENCE),
            0x8 => Ok(SerializedType::STRUCT),
            0x9 => Ok(SerializedType::TYPE_PARAMETER),
            0xA => Ok(SerializedType::VECTOR),
            0xB => Ok(SerializedType::STRUCT_INST),
            _ => Err(VMStatus::new(StatusCode::UNKNOWN_SERIALIZED_TYPE)),
        }
    }
}

impl SerializedNominalResourceFlag {
    fn from_u8(value: u8) -> BinaryLoaderResult<SerializedNominalResourceFlag> {
        match value {
            0x1 => Ok(SerializedNominalResourceFlag::NOMINAL_RESOURCE),
            0x2 => Ok(SerializedNominalResourceFlag::NORMAL_STRUCT),
            _ => Err(VMStatus::new(StatusCode::UNKNOWN_NOMINAL_RESOURCE)),
        }
    }
}

impl SerializedKind {
    fn from_u8(value: u8) -> BinaryLoaderResult<SerializedKind> {
        match value {
            0x1 => Ok(SerializedKind::ALL),
            0x2 => Ok(SerializedKind::COPYABLE),
            0x3 => Ok(SerializedKind::RESOURCE),
            _ => Err(VMStatus::new(StatusCode::UNKNOWN_KIND)),
        }
    }
}

impl SerializedNativeStructFlag {
    fn from_u8(value: u8) -> BinaryLoaderResult<SerializedNativeStructFlag> {
        match value {
            0x1 => Ok(SerializedNativeStructFlag::NATIVE),
            0x2 => Ok(SerializedNativeStructFlag::DECLARED),
            _ => Err(VMStatus::new(StatusCode::UNKNOWN_NATIVE_STRUCT_FLAG)),
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
            0x06 => Ok(Opcodes::LD_U64),
            0x07 => Ok(Opcodes::LD_CONST),
            0x08 => Ok(Opcodes::LD_TRUE),
            0x09 => Ok(Opcodes::LD_FALSE),
            0x0A => Ok(Opcodes::COPY_LOC),
            0x0B => Ok(Opcodes::MOVE_LOC),
            0x0C => Ok(Opcodes::ST_LOC),
            0x0D => Ok(Opcodes::MUT_BORROW_LOC),
            0x0E => Ok(Opcodes::IMM_BORROW_LOC),
            0x0F => Ok(Opcodes::MUT_BORROW_FIELD),
            0x10 => Ok(Opcodes::IMM_BORROW_FIELD),
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
            0x2D => Ok(Opcodes::MUT_BORROW_GLOBAL),
            0x2E => Ok(Opcodes::IMM_BORROW_GLOBAL),
            0x2F => Ok(Opcodes::MOVE_FROM),
            0x30 => Ok(Opcodes::MOVE_TO),
            0x31 => Ok(Opcodes::GET_TXN_SEQUENCE_NUMBER),
            0x32 => Ok(Opcodes::GET_TXN_PUBLIC_KEY),
            0x33 => Ok(Opcodes::FREEZE_REF),
            0x34 => Ok(Opcodes::SHL),
            0x35 => Ok(Opcodes::SHR),
            0x36 => Ok(Opcodes::LD_U8),
            0x37 => Ok(Opcodes::LD_U128),
            0x38 => Ok(Opcodes::CAST_U8),
            0x39 => Ok(Opcodes::CAST_U64),
            0x3A => Ok(Opcodes::CAST_U128),
            0x3B => Ok(Opcodes::MUT_BORROW_FIELD_GENERIC),
            0x3C => Ok(Opcodes::IMM_BORROW_FIELD_GENERIC),
            0x3D => Ok(Opcodes::CALL_GENERIC),
            0x3E => Ok(Opcodes::PACK_GENERIC),
            0x3F => Ok(Opcodes::UNPACK_GENERIC),
            0x40 => Ok(Opcodes::EXISTS_GENERIC),
            0x41 => Ok(Opcodes::MUT_BORROW_GLOBAL_GENERIC),
            0x42 => Ok(Opcodes::IMM_BORROW_GLOBAL_GENERIC),
            0x43 => Ok(Opcodes::MOVE_FROM_GENERIC),
            0x44 => Ok(Opcodes::MOVE_TO_GENERIC),
            0x45 => Ok(Opcodes::NOP),
            _ => Err(VMStatus::new(StatusCode::UNKNOWN_OPCODE)),
        }
    }
}
