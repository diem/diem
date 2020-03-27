// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::file_format::*;
use anyhow::{bail, format_err, Result};
use libra_types::account_address::AccountAddress;
use move_core_types::identifier::IdentStr;
use std::{collections::VecDeque, fmt};

//
// Display printing
// Display the top level compilation unit (CompiledScript and CompiledModule) in a more
// readable format. Essentially the printing resolves all table indexes and is a line by line
// for each table and with a reasonable indentation, e.g.
// ```text
// CompiledModule: {
// Struct Handles: [
//   ResourceBox@0x0000000000000000000000000000000000000000000000000000000000000000,]
// Field Handles: [
//   ResourceBox@0x0000000000000000000000000000000000000000000000000000000000000000.item: Value,]
// Function Handles: [
//   ResourceBox@0x0000000000000000000000000000000000000000000000000000000000000000.get(): Value,
//   ResourceBox@0x0000000000000000000000000000000000000000000000000000000000000000.new(Value): ResourceBox@0x0000000000000000000000000000000000000000000000000000000000000000,]
// Struct Definitions: [
//   {public resource ResourceBox@0x0000000000000000000000000000000000000000000000000000000000000000
//       private ResourceBox@0x0000000000000000000000000000000000000000000000000000000000000000.item: Value
//       public ResourceBox@0x0000000000000000000000000000000000000000000000000000000000000000.get(): Value
//       static public ResourceBox@0x0000000000000000000000000000000000000000000000000000000000000000.new(Value): ResourceBox@0x0000000000000000000000000000000000000000000000000000000000000000},]
// Field Definitions: [
//   private ResourceBox@0x0000000000000000000000000000000000000000000000000000000000000000.item: Value,]
// Function Definitions: [
//   public ResourceBox@0x0000000000000000000000000000000000000000000000000000000000000000.get(): Value
//       local(0): ResourceBox@0x0000000000000000000000000000000000000000000000000000000000000000,
//       local(1): &Value,
//       local(2): Value,
//       CopyLoc(0)
//       BorrowField(ResourceBox@0x0000000000000000000000000000000000000000000000000000000000000000.item: Value)
//       StLoc(1)
//       CopyLoc(1)
//       ReadRef
//       StLoc(2)
//       MoveLoc(2)
//       Ret,
//   static public ResourceBox@0x0000000000000000000000000000000000000000000000000000000000000000.new(Value): ResourceBox@0x0000000000000000000000000000000000000000000000000000000000000000
//       local(0): Value,
//       local(1): ResourceBox@0x0000000000000000000000000000000000000000000000000000000000000000,
//       MoveLoc(0)
//       Pack(ResourceBox@0x0000000000000000000000000000000000000000000000000000000000000000)
//       StLoc(1)
//       MoveLoc(1)
//       Ret,]
// Signatures: [
//   Value,
//   (): Value,
//   (Value): ResourceBox@0x0000000000000000000000000000000000000000000000000000000000000000,
//   ResourceBox@0x0000000000000000000000000000000000000000000000000000000000000000,
//   &Value,]
// Strings: [
//   ResourceBox,
//   item,
//   get,
//   new,]
// Addresses: [
//   0x0000000000000000000000000000000000000000000000000000000000000000,]
// }
// ```

// Trait to access tables for both CompiledScript and CompiledModule.
// This is designed mainly for the printer -- public APIs should be based on the accessors in
// `access.rs`.
pub trait TableAccess {
    fn get_field_def_at(&self, idx: FieldDefinitionIndex) -> Result<&FieldDefinition>;

    fn get_module_at(&self, idx: ModuleHandleIndex) -> Result<&ModuleHandle>;
    fn get_struct_at(&self, idx: StructHandleIndex) -> Result<&StructHandle>;
    fn get_function_at(&self, idx: FunctionHandleIndex) -> Result<&FunctionHandle>;

    fn get_identifier_at(&self, idx: IdentifierIndex) -> Result<&IdentStr>;
    fn get_address_at(&self, idx: AddressPoolIndex) -> Result<&AccountAddress>;
    fn get_type_signature_at(&self, idx: TypeSignatureIndex) -> Result<&TypeSignature>;
    fn get_function_signature_at(&self, idx: FunctionSignatureIndex) -> Result<&FunctionSignature>;
    fn get_locals_signature_at(&self, idx: LocalsSignatureIndex) -> Result<&LocalsSignature>;
}

impl TableAccess for CompiledScriptMut {
    fn get_field_def_at(&self, _idx: FieldDefinitionIndex) -> Result<&FieldDefinition> {
        bail!("no field definitions in scripts");
    }

    fn get_module_at(&self, idx: ModuleHandleIndex) -> Result<&ModuleHandle> {
        self.module_handles
            .get(idx.0 as usize)
            .ok_or_else(|| format_err!("bad module handle index {}", idx))
    }

    fn get_struct_at(&self, idx: StructHandleIndex) -> Result<&StructHandle> {
        self.struct_handles
            .get(idx.0 as usize)
            .ok_or_else(|| format_err!("bad struct handle index {}", idx))
    }

    fn get_function_at(&self, idx: FunctionHandleIndex) -> Result<&FunctionHandle> {
        self.function_handles
            .get(idx.0 as usize)
            .ok_or_else(|| format_err!("bad function handle index {}", idx))
    }

    fn get_identifier_at(&self, idx: IdentifierIndex) -> Result<&IdentStr> {
        self.identifiers
            .get(idx.0 as usize)
            .map(|x| x.as_ref())
            .ok_or_else(|| format_err!("bad string index {}", idx))
    }

    fn get_address_at(&self, idx: AddressPoolIndex) -> Result<&AccountAddress> {
        self.address_pool
            .get(idx.0 as usize)
            .ok_or_else(|| format_err!("bad address index {}", idx))
    }

    fn get_type_signature_at(&self, idx: TypeSignatureIndex) -> Result<&TypeSignature> {
        self.type_signatures
            .get(idx.0 as usize)
            .ok_or_else(|| format_err!("bad signature index {}", idx))
    }

    fn get_function_signature_at(&self, idx: FunctionSignatureIndex) -> Result<&FunctionSignature> {
        self.function_signatures
            .get(idx.0 as usize)
            .ok_or_else(|| format_err!("bad signature index {}", idx))
    }

    fn get_locals_signature_at(&self, idx: LocalsSignatureIndex) -> Result<&LocalsSignature> {
        self.locals_signatures
            .get(idx.0 as usize)
            .ok_or_else(|| format_err!("bad signature index {}", idx))
    }
}

impl TableAccess for CompiledModuleMut {
    fn get_field_def_at(&self, idx: FieldDefinitionIndex) -> Result<&FieldDefinition> {
        self.field_defs
            .get(idx.0 as usize)
            .ok_or_else(|| format_err!("bad field definition index {}", idx))
    }

    fn get_module_at(&self, idx: ModuleHandleIndex) -> Result<&ModuleHandle> {
        self.module_handles
            .get(idx.0 as usize)
            .ok_or_else(|| format_err!("bad module handle index {}", idx))
    }

    fn get_struct_at(&self, idx: StructHandleIndex) -> Result<&StructHandle> {
        self.struct_handles
            .get(idx.0 as usize)
            .ok_or_else(|| format_err!("bad struct handle index {}", idx))
    }

    fn get_function_at(&self, idx: FunctionHandleIndex) -> Result<&FunctionHandle> {
        self.function_handles
            .get(idx.0 as usize)
            .ok_or_else(|| format_err!("bad function handle index {}", idx))
    }

    fn get_identifier_at(&self, idx: IdentifierIndex) -> Result<&IdentStr> {
        self.identifiers
            .get(idx.0 as usize)
            .map(|x| x.as_ref())
            .ok_or_else(|| format_err!("bad string index {}", idx))
    }

    fn get_address_at(&self, idx: AddressPoolIndex) -> Result<&AccountAddress> {
        self.address_pool
            .get(idx.0 as usize)
            .ok_or_else(|| format_err!("bad address index {}", idx))
    }

    fn get_type_signature_at(&self, idx: TypeSignatureIndex) -> Result<&TypeSignature> {
        self.type_signatures
            .get(idx.0 as usize)
            .ok_or_else(|| format_err!("bad signature index {}", idx))
    }

    fn get_function_signature_at(&self, idx: FunctionSignatureIndex) -> Result<&FunctionSignature> {
        self.function_signatures
            .get(idx.0 as usize)
            .ok_or_else(|| format_err!("bad signature index {}", idx))
    }

    fn get_locals_signature_at(&self, idx: LocalsSignatureIndex) -> Result<&LocalsSignature> {
        self.locals_signatures
            .get(idx.0 as usize)
            .ok_or_else(|| format_err!("bad signature index {}", idx))
    }
}

impl fmt::Display for CompiledScript {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let inner = self.as_inner();
        write!(f, "CompiledScript: {{\nMain:\n\t")?;
        display_function_definition(&inner.main, inner, f)?;
        display_code(&inner.main.code, inner, "\n\t\t", f)?;
        write!(f, "\nStruct Handles: [")?;
        for struct_handle in &inner.struct_handles {
            write!(f, "\n\t")?;
            display_struct_handle(struct_handle, inner, f)?;
            write!(f, ",")?;
        }
        writeln!(f, "]")?;
        write!(f, "Module Handles: [")?;
        for module_handle in &inner.module_handles {
            write!(f, "\n\t")?;
            display_module_handle(module_handle, inner, f)?;
            write!(f, ",")?;
        }
        writeln!(f, "]")?;
        write!(f, "Function Handles: [")?;
        for function_handle in &inner.function_handles {
            write!(f, "\n\t")?;
            display_function_handle(function_handle, inner, f)?;
            write!(f, ",")?;
        }
        writeln!(f, "]")?;
        write!(f, "Type Signatures: [")?;
        for signature in &inner.type_signatures {
            write!(f, "\n\t")?;
            display_type_signature(signature, inner, f)?;
            write!(f, ",")?;
        }
        writeln!(f, "]")?;
        write!(f, "Function Signatures: [")?;
        for signature in &inner.function_signatures {
            write!(f, "\n\t")?;
            display_function_signature(signature, inner, f)?;
            write!(f, ",")?;
        }
        writeln!(f, "]")?;
        write!(f, "Locals Signatures: [")?;
        for signature in &inner.locals_signatures {
            write!(f, "\n\t")?;
            display_locals_signature(signature, inner, f)?;
            write!(f, ",")?;
        }
        writeln!(f, "]")?;
        write!(f, "Strings: [")?;
        for string in &inner.identifiers {
            write!(f, "\n\t{},", string)?;
        }
        writeln!(f, "]")?;
        write!(f, "ByteArrays: [")?;
        for byte_array in &inner.byte_array_pool {
            write!(f, "\n\t")?;
            display_byte_array(byte_array, f)?;
            write!(f, ",")?;
        }
        writeln!(f, "]")?;
        write!(f, "Addresses: [")?;
        for address in &inner.address_pool {
            write!(f, "\n\t")?;
            display_address(address, f)?;
            write!(f, ",")?;
        }
        writeln!(f, "]")?;
        writeln!(f, "}}")
    }
}

impl fmt::Display for CompiledModule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let inner = self.as_inner();
        writeln!(f, "CompiledModule: {{")?;
        write!(f, "Module Handles: [")?;
        for module_handle in &inner.module_handles {
            write!(f, "\n\t")?;
            display_module_handle(module_handle, inner, f)?;
            write!(f, ",")?;
        }
        writeln!(f, "]")?;
        write!(f, "Struct Handles: [")?;
        for struct_handle in &inner.struct_handles {
            write!(f, "\n\t")?;
            display_struct_handle(struct_handle, inner, f)?;
            write!(f, ",")?;
        }
        writeln!(f, "]")?;
        write!(f, "Function Handles: [")?;
        for function_handle in &inner.function_handles {
            write!(f, "\n\t")?;
            display_function_handle(function_handle, inner, f)?;
            write!(f, ",")?;
        }
        writeln!(f, "]")?;
        write!(f, "Struct Definitions: [")?;
        for struct_def in &inner.struct_defs {
            write!(f, "\n\t{{")?;
            display_struct_definition(struct_def, inner, f)?;
            match &struct_def.field_information {
                StructFieldInformation::Native => write!(f, "native")?,
                StructFieldInformation::Declared {
                    field_count,
                    fields,
                } => {
                    let f_start_idx = *fields;
                    let f_end_idx = f_start_idx.0 as u16 + *field_count;
                    for idx in f_start_idx.0 as u16..f_end_idx {
                        let field_def = inner
                            .field_defs
                            .get(idx as usize)
                            .expect(&format!("bad field definition index {}", idx)[..]);
                        write!(f, "\n\t\t")?;
                        display_field_definition(field_def, inner, f)?;
                    }
                }
            }
            write!(f, "}},")?;
        }
        writeln!(f, "]")?;
        write!(f, "Field Definitions: [")?;
        for field_def in &inner.field_defs {
            write!(f, "\n\t")?;
            display_field_definition(field_def, inner, f)?;
            write!(f, ",")?;
        }
        writeln!(f, "]")?;
        write!(f, "Function Definitions: [")?;
        for function_def in &inner.function_defs {
            write!(f, "\n\t")?;
            display_function_definition(function_def, inner, f)?;
            if function_def.flags & CodeUnit::NATIVE == 0 {
                display_code(&function_def.code, inner, "\n\t\t", f)?;
            }
            write!(f, ",")?;
        }
        writeln!(f, "]")?;
        write!(f, "Type Signatures: [")?;
        for signature in &inner.type_signatures {
            write!(f, "\n\t")?;
            display_type_signature(signature, inner, f)?;
            write!(f, ",")?;
        }
        writeln!(f, "]")?;
        write!(f, "Function Signatures: [")?;
        for signature in &inner.function_signatures {
            write!(f, "\n\t")?;
            display_function_signature(signature, inner, f)?;
            write!(f, ",")?;
        }
        writeln!(f, "]")?;
        write!(f, "Locals Signatures: [")?;
        for signature in &inner.locals_signatures {
            write!(f, "\n\t")?;
            display_locals_signature(signature, inner, f)?;
            write!(f, ",")?;
        }
        writeln!(f, "]")?;
        write!(f, "Strings: [")?;
        for string in &inner.identifiers {
            write!(f, "\n\t{},", string)?;
        }
        writeln!(f, "]")?;
        write!(f, "ByteArrays: [")?;
        for byte_array in &inner.byte_array_pool {
            write!(f, "\n\t")?;
            display_byte_array(byte_array, f)?;
            write!(f, ",")?;
        }
        writeln!(f, "]")?;
        write!(f, "Addresses: [")?;
        for address in &inner.address_pool {
            write!(f, "\n\t")?;
            display_address(address, f)?;
            write!(f, ",")?;
        }
        writeln!(f, "]")?;
        writeln!(f, "}}")
    }
}

fn display_struct_handle<T: TableAccess>(
    struct_: &StructHandle,
    tables: &T,
    f: &mut fmt::Formatter,
) -> fmt::Result {
    write!(
        f,
        "{} ",
        if struct_.is_nominal_resource {
            "resource"
        } else {
            "struct"
        }
    )?;
    write!(f, "{}@", tables.get_identifier_at(struct_.name).unwrap())?;
    display_module_handle(tables.get_module_at(struct_.module).unwrap(), tables, f)
}

fn display_module_handle<T: TableAccess>(
    module: &ModuleHandle,
    tables: &T,
    f: &mut fmt::Formatter,
) -> fmt::Result {
    display_address(tables.get_address_at(module.address).unwrap(), f)?;
    write!(f, ".{}", tables.get_identifier_at(module.name).unwrap())
}

fn display_function_handle<T: TableAccess>(
    function: &FunctionHandle,
    tables: &T,
    f: &mut fmt::Formatter,
) -> fmt::Result {
    display_module_handle(tables.get_module_at(function.module).unwrap(), tables, f)?;
    write!(f, ".{}", tables.get_identifier_at(function.name).unwrap())?;
    display_function_signature(
        tables
            .get_function_signature_at(function.signature)
            .unwrap(),
        tables,
        f,
    )
}

fn display_struct_definition<T: TableAccess>(
    struct_: &StructDefinition,
    tables: &T,
    f: &mut fmt::Formatter,
) -> fmt::Result {
    display_struct_handle(
        tables.get_struct_at(struct_.struct_handle).unwrap(),
        tables,
        f,
    )
}

fn display_field_definition<T: TableAccess>(
    field: &FieldDefinition,
    tables: &T,
    f: &mut fmt::Formatter,
) -> fmt::Result {
    display_struct_handle(tables.get_struct_at(field.struct_).unwrap(), tables, f)?;
    write!(f, ".{}: ", tables.get_identifier_at(field.name).unwrap())?;
    display_type_signature(
        tables.get_type_signature_at(field.signature).unwrap(),
        tables,
        f,
    )
}

fn display_function_definition<T: TableAccess>(
    function: &FunctionDefinition,
    tables: &T,
    f: &mut fmt::Formatter,
) -> fmt::Result {
    display_function_flags(function.flags, f)?;
    display_function_handle(
        tables.get_function_at(function.function).unwrap(),
        tables,
        f,
    )
}

fn display_code<T: TableAccess>(
    code: &CodeUnit,
    tables: &T,
    indentation: &str,
    f: &mut fmt::Formatter,
) -> fmt::Result {
    write!(f, "{}locals({}): ", indentation, code.locals,)?;
    display_locals_signature(
        tables.get_locals_signature_at(code.locals).unwrap(),
        tables,
        f,
    )?;
    write!(f, ",")?;
    for bytecode in &code.code {
        write!(f, "{}", indentation)?;
        display_bytecode(bytecode, tables, f)?;
    }
    Ok(())
}

fn display_address(addr: &AccountAddress, f: &mut fmt::Formatter) -> fmt::Result {
    let hex = format!("{:x}", addr);
    let mut v: VecDeque<char> = hex.chars().collect();
    while v.len() > 1 && v[0] == '0' {
        v.pop_front();
    }
    write!(f, "0x{}", v.into_iter().collect::<String>())
}

fn display_byte_array(byte_array: &[u8], f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "0x{}", hex::encode(byte_array))
}

fn display_type_signature<T: TableAccess>(
    sig: &TypeSignature,
    tables: &T,
    f: &mut fmt::Formatter,
) -> fmt::Result {
    display_signature_token(&sig.0, tables, f)
}

fn display_function_signature<T: TableAccess>(
    sig: &FunctionSignature,
    tables: &T,
    f: &mut fmt::Formatter,
) -> fmt::Result {
    let mut iter = sig.arg_types.iter().peekable();
    write!(f, "(")?;
    while let Some(token) = iter.next() {
        display_signature_token(token, tables, f)?;
        if iter.peek().is_some() {
            write!(f, ", ")?;
        }
    }
    write!(f, "): ")?;

    let mut iter = sig.return_types.iter().peekable();
    write!(f, "(")?;
    while let Some(token) = iter.next() {
        display_signature_token(token, tables, f)?;
        if iter.peek().is_some() {
            write!(f, ", ")?;
        }
    }
    write!(f, ")")?;
    Ok(())
}

fn display_locals_signature<T: TableAccess>(
    sig: &LocalsSignature,
    tables: &T,
    f: &mut fmt::Formatter,
) -> fmt::Result {
    let mut iter = sig.0.iter().peekable();
    while let Some(token) = iter.next() {
        display_signature_token(token, tables, f)?;
        if iter.peek().is_some() {
            write!(f, ", ")?;
        }
    }
    Ok(())
}

fn display_type_actuals<T: TableAccess>(
    types: &[SignatureToken],
    tables: &T,
    f: &mut fmt::Formatter,
) -> fmt::Result {
    if types.is_empty() {
        return Ok(());
    }
    write!(f, "<")?;
    for (i, t) in types.iter().enumerate() {
        if i > 0 {
            write!(f, ", ")?;
        }
        display_signature_token(t, tables, f)?;
    }
    write!(f, ">")
}

fn display_signature_token<T: TableAccess>(
    token: &SignatureToken,
    tables: &T,
    f: &mut fmt::Formatter,
) -> fmt::Result {
    match token {
        SignatureToken::Bool => write!(f, "Bool"),
        SignatureToken::U8 => write!(f, "U8"),
        SignatureToken::U64 => write!(f, "U64"),
        SignatureToken::U128 => write!(f, "U128"),
        SignatureToken::Address => write!(f, "Address"),
        SignatureToken::Vector(ty) => {
            write!(f, "Vector<")?;
            display_signature_token(ty, tables, f)?;
            write!(f, ">")
        }
        SignatureToken::Struct(idx, types) => {
            display_struct_handle(tables.get_struct_at(*idx).unwrap(), tables, f)?;
            display_type_actuals(&types, tables, f)
        }
        SignatureToken::Reference(token) => {
            write!(f, "&")?;
            display_signature_token(token, tables, f)
        }
        SignatureToken::MutableReference(token) => {
            write!(f, "&mut ")?;
            display_signature_token(token, tables, f)
        }
        SignatureToken::TypeParameter(idx) => write!(f, "T{}", idx),
    }
}

fn display_function_flags(flags: u8, f: &mut fmt::Formatter) -> fmt::Result {
    if flags & CodeUnit::NATIVE != 0 {
        write!(f, "native ")?;
    }
    if flags & CodeUnit::PUBLIC != 0 {
        write!(f, "public ")?;
    }
    Ok(())
}

fn display_bytecode<T: TableAccess>(
    bytecode: &Bytecode,
    tables: &T,
    f: &mut fmt::Formatter,
) -> fmt::Result {
    match bytecode {
        Bytecode::LdAddr(idx) => {
            write!(f, "LdAddr(")?;
            display_address(tables.get_address_at(*idx).unwrap(), f)?;
            write!(f, ")")
        }
        Bytecode::MutBorrowField(idx) => {
            write!(f, "MutBorrowField(")?;
            display_field_definition(tables.get_field_def_at(*idx).unwrap(), tables, f)?;
            write!(f, ")")
        }
        Bytecode::ImmBorrowField(idx) => {
            write!(f, "ImmBorrowField(")?;
            display_field_definition(tables.get_field_def_at(*idx).unwrap(), tables, f)?;
            write!(f, ")")
        }
        Bytecode::Call(idx, types_idx) => {
            write!(f, "Call")?;
            display_type_actuals(
                &tables.get_locals_signature_at(*types_idx).unwrap().0,
                tables,
                f,
            )?;
            write!(f, "(")?;
            display_function_handle(tables.get_function_at(*idx).unwrap(), tables, f)?;
            write!(f, ")")
        }
        _ => write!(f, "{:?}", bytecode),
    }
}
