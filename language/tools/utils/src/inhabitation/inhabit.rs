// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use vm::file_format::{
    AddressPoolIndex, ByteArrayPoolIndex, Bytecode, CompiledModuleMut, LocalsSignatureIndex,
    SignatureToken, StructDefinitionIndex, StructFieldInformation, TableIndex, UserStringIndex,
};

/// Generate a sequence of instructions whose overall effect is to push a single value of type token
/// on the stack, specifically without consuming any values that existed on the stack prior to the
/// execution of the instruction sequence.
pub fn inhabit_with_bytecode_seq(
    module: &CompiledModuleMut,
    token: &SignatureToken,
) -> Vec<Bytecode> {
    match token {
        SignatureToken::String => vec![Bytecode::LdStr(UserStringIndex::new(0))],
        SignatureToken::Address => vec![Bytecode::LdAddr(AddressPoolIndex::new(0))],
        SignatureToken::U64 => vec![Bytecode::LdConst(0)],
        SignatureToken::Bool => vec![Bytecode::LdFalse],
        SignatureToken::ByteArray => vec![Bytecode::LdByteArray(ByteArrayPoolIndex::new(0))],
        SignatureToken::Struct(handle_idx, _fixme) => {
            let empty_ty_param_index = module
                .locals_signatures
                .iter()
                .position(|sig| sig.is_empty())
                .expect("locals signatures must have an empty locals signature");
            let struct_def_idx = module
                .struct_defs
                .iter()
                .position(|struct_def| struct_def.struct_handle == *handle_idx)
                .expect("struct def should exist for every struct handle in the test generator");
            let struct_def = &module.struct_defs[struct_def_idx];
            let (num_fields, index) = match struct_def.field_information {
                StructFieldInformation::Native => panic!("Can't inhabit native structs"),
                StructFieldInformation::Declared {
                    field_count,
                    fields,
                } => (field_count as usize, fields.0 as usize),
            };
            let fields = &module.field_defs[index..(index + num_fields)];
            let mut bytecodes: Vec<Bytecode> = fields
                .iter()
                .flat_map(|field| {
                    inhabit_with_bytecode_seq(
                        &module,
                        &module.type_signatures[field.signature.0 as usize].0,
                    )
                })
                .collect();
            bytecodes.push(Bytecode::Pack(
                StructDefinitionIndex(struct_def_idx as TableIndex),
                LocalsSignatureIndex(empty_ty_param_index as TableIndex),
            ));
            bytecodes
        }
        _ => unimplemented!("Unsupported return type: {:#?}", token),
    }
}
