// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_proptest_helpers::pick_slice_idxs;
use libra_types::vm_error::{StatusCode, VMStatus};
use proptest::{prelude::*, sample::Index as PropIndex};
use std::collections::BTreeMap;
use vm::{
    errors::{append_err_info, bytecode_offset_err},
    file_format::{
        AddressPoolIndex, ByteArrayPoolIndex, Bytecode, CodeOffset, CompiledModuleMut,
        FieldDefinitionIndex, FunctionHandleIndex, LocalIndex, StructDefinitionIndex, TableIndex,
        UserStringIndex, NO_TYPE_ACTUALS,
    },
    internals::ModuleIndex,
    IndexKind,
};

/// Represents a single mutation onto a code unit to make it out of bounds.
#[derive(Debug)]
pub struct CodeUnitBoundsMutation {
    function_def: PropIndex,
    bytecode: PropIndex,
    offset: usize,
}

impl CodeUnitBoundsMutation {
    pub fn strategy() -> impl Strategy<Value = Self> {
        (any::<PropIndex>(), any::<PropIndex>(), 0..16 as usize).prop_map(
            |(function_def, bytecode, offset)| Self {
                function_def,
                bytecode,
                offset,
            },
        )
    }
}

impl AsRef<PropIndex> for CodeUnitBoundsMutation {
    #[inline]
    fn as_ref(&self) -> &PropIndex {
        &self.bytecode
    }
}

pub struct ApplyCodeUnitBoundsContext<'a> {
    module: &'a mut CompiledModuleMut,
    // This is so apply_one can be called after mutations has been iterated on.
    mutations: Option<Vec<CodeUnitBoundsMutation>>,
}

macro_rules! new_bytecode {
    ($dst_len: expr, $bytecode_idx: expr, $offset: expr, $idx_type: ident, $bytecode_ident: tt) => {{
        let dst_len = $dst_len;
        let new_idx = (dst_len + $offset) as TableIndex;
        (
            $bytecode_ident($idx_type::new(new_idx)),
            bytecode_offset_err(
                $idx_type::KIND,
                new_idx as usize,
                dst_len,
                $bytecode_idx,
                StatusCode::INDEX_OUT_OF_BOUNDS,
            ),
        )
    }};
}

macro_rules! struct_bytecode {
    ($dst_len: expr, $bytecode_idx: expr, $offset: expr, $idx_type: ident, $bytecode_ident: tt) => {{
        let dst_len = $dst_len;
        let new_idx = (dst_len + $offset) as TableIndex;
        (
            // TODO: check this again once generics is implemented
            $bytecode_ident($idx_type::new(new_idx), NO_TYPE_ACTUALS),
            bytecode_offset_err(
                $idx_type::KIND,
                new_idx as usize,
                dst_len,
                $bytecode_idx,
                StatusCode::INDEX_OUT_OF_BOUNDS,
            ),
        )
    }};
}

macro_rules! code_bytecode {
    ($code_len: expr, $bytecode_idx: expr, $offset: expr, $bytecode_ident: tt) => {{
        let code_len = $code_len;
        let new_idx = code_len + $offset;
        (
            $bytecode_ident(new_idx as CodeOffset),
            bytecode_offset_err(
                IndexKind::CodeDefinition,
                new_idx,
                code_len,
                $bytecode_idx,
                StatusCode::INDEX_OUT_OF_BOUNDS,
            ),
        )
    }};
}

macro_rules! locals_bytecode {
    ($locals_len: expr, $bytecode_idx: expr, $offset: expr, $bytecode_ident: tt) => {{
        let locals_len = $locals_len;
        let new_idx = locals_len + $offset;
        (
            $bytecode_ident(new_idx as LocalIndex),
            bytecode_offset_err(
                IndexKind::LocalPool,
                new_idx,
                locals_len,
                $bytecode_idx,
                StatusCode::INDEX_OUT_OF_BOUNDS,
            ),
        )
    }};
}

impl<'a> ApplyCodeUnitBoundsContext<'a> {
    pub fn new(module: &'a mut CompiledModuleMut, mutations: Vec<CodeUnitBoundsMutation>) -> Self {
        Self {
            module,
            mutations: Some(mutations),
        }
    }

    pub fn apply(mut self) -> Vec<VMStatus> {
        let function_def_len = self.module.function_defs.len();

        let mut mutation_map = BTreeMap::new();
        for mutation in self
            .mutations
            .take()
            .expect("mutations should always be present")
        {
            let picked_idx = mutation.function_def.index(function_def_len);
            mutation_map
                .entry(picked_idx)
                .or_insert_with(|| vec![])
                .push(mutation);
        }

        let mut results = vec![];

        for (idx, mutations) in mutation_map {
            results.extend(self.apply_one(idx, mutations));
        }
        results
    }

    fn apply_one(&mut self, idx: usize, mutations: Vec<CodeUnitBoundsMutation>) -> Vec<VMStatus> {
        // For this function def, find all the places where a bounds mutation can be applied.
        let (code_len, locals_len) = {
            let code = &mut self.module.function_defs[idx].code;
            (
                code.code.len(),
                self.module.locals_signatures[code.locals.into_index()].len(),
            )
        };

        let code = &mut self.module.function_defs[idx].code.code;
        let interesting_offsets: Vec<usize> = (0..code.len())
            .filter(|bytecode_idx| is_interesting(&code[*bytecode_idx]))
            .collect();
        let to_mutate = pick_slice_idxs(interesting_offsets.len(), &mutations);

        // These have to be computed upfront because self.module is being mutated below.
        let address_pool_len = self.module.address_pool.len();
        let user_strings_len = self.module.user_strings.len();
        let byte_array_pool_len = self.module.byte_array_pool.len();
        let function_handles_len = self.module.function_handles.len();
        let field_defs_len = self.module.field_defs.len();
        let struct_defs_len = self.module.struct_defs.len();

        mutations
            .iter()
            .zip(to_mutate)
            .map(|(mutation, interesting_offsets_idx)| {
                let bytecode_idx = interesting_offsets[interesting_offsets_idx];
                let offset = mutation.offset;
                use Bytecode::*;

                let (new_bytecode, err) = match code[bytecode_idx] {
                    LdAddr(_) => new_bytecode!(
                        address_pool_len,
                        bytecode_idx,
                        offset,
                        AddressPoolIndex,
                        LdAddr
                    ),
                    LdStr(_) => new_bytecode!(
                        user_strings_len,
                        bytecode_idx,
                        offset,
                        UserStringIndex,
                        LdStr
                    ),
                    LdByteArray(_) => new_bytecode!(
                        byte_array_pool_len,
                        bytecode_idx,
                        offset,
                        ByteArrayPoolIndex,
                        LdByteArray
                    ),
                    ImmBorrowField(_) => new_bytecode!(
                        field_defs_len,
                        bytecode_idx,
                        offset,
                        FieldDefinitionIndex,
                        ImmBorrowField
                    ),
                    MutBorrowField(_) => new_bytecode!(
                        field_defs_len,
                        bytecode_idx,
                        offset,
                        FieldDefinitionIndex,
                        MutBorrowField
                    ),
                    Call(_, _) => struct_bytecode!(
                        function_handles_len,
                        bytecode_idx,
                        offset,
                        FunctionHandleIndex,
                        Call
                    ),
                    Pack(_, _) => struct_bytecode!(
                        struct_defs_len,
                        bytecode_idx,
                        offset,
                        StructDefinitionIndex,
                        Pack
                    ),
                    Unpack(_, _) => struct_bytecode!(
                        struct_defs_len,
                        bytecode_idx,
                        offset,
                        StructDefinitionIndex,
                        Unpack
                    ),
                    Exists(_, _) => struct_bytecode!(
                        struct_defs_len,
                        bytecode_idx,
                        offset,
                        StructDefinitionIndex,
                        Exists
                    ),
                    MutBorrowGlobal(_, _) => struct_bytecode!(
                        struct_defs_len,
                        bytecode_idx,
                        offset,
                        StructDefinitionIndex,
                        MutBorrowGlobal
                    ),
                    ImmBorrowGlobal(_, _) => struct_bytecode!(
                        struct_defs_len,
                        bytecode_idx,
                        offset,
                        StructDefinitionIndex,
                        ImmBorrowGlobal
                    ),
                    MoveFrom(_, _) => struct_bytecode!(
                        struct_defs_len,
                        bytecode_idx,
                        offset,
                        StructDefinitionIndex,
                        MoveFrom
                    ),
                    MoveToSender(_, _) => struct_bytecode!(
                        struct_defs_len,
                        bytecode_idx,
                        offset,
                        StructDefinitionIndex,
                        MoveToSender
                    ),
                    BrTrue(_) => code_bytecode!(code_len, bytecode_idx, offset, BrTrue),
                    BrFalse(_) => code_bytecode!(code_len, bytecode_idx, offset, BrFalse),
                    Branch(_) => code_bytecode!(code_len, bytecode_idx, offset, Branch),
                    CopyLoc(_) => locals_bytecode!(locals_len, bytecode_idx, offset, CopyLoc),
                    MoveLoc(_) => locals_bytecode!(locals_len, bytecode_idx, offset, MoveLoc),
                    StLoc(_) => locals_bytecode!(locals_len, bytecode_idx, offset, StLoc),
                    MutBorrowLoc(_) => {
                        locals_bytecode!(locals_len, bytecode_idx, offset, MutBorrowLoc)
                    }
                    ImmBorrowLoc(_) => {
                        locals_bytecode!(locals_len, bytecode_idx, offset, ImmBorrowLoc)
                    }

                    // List out the other options explicitly so there's a compile error if a new
                    // bytecode gets added.
                    FreezeRef | Pop | Ret | LdConst(_) | LdTrue | LdFalse | ReadRef | WriteRef
                    | Add | Sub | Mul | Mod | Div | BitOr | BitAnd | Xor | Or | And | Not | Eq
                    | Neq | Lt | Gt | Le | Ge | Abort | GetTxnGasUnitPrice | GetTxnMaxGasUnits
                    | GetGasRemaining | GetTxnSenderAddress | GetTxnSequenceNumber
                    | GetTxnPublicKey => {
                        panic!("Bytecode has no internal index: {:?}", code[bytecode_idx])
                    }
                };

                code[bytecode_idx] = new_bytecode;

                append_err_info(err, IndexKind::FunctionDefinition, idx)
            })
            .collect()
    }
}

fn is_interesting(bytecode: &Bytecode) -> bool {
    use Bytecode::*;

    match bytecode {
        LdAddr(_)
        | LdStr(_)
        | LdByteArray(_)
        | ImmBorrowField(_)
        | MutBorrowField(_)
        | Call(_, _)
        | Pack(_, _)
        | Unpack(_, _)
        | Exists(_, _)
        | MutBorrowGlobal(_, _)
        | ImmBorrowGlobal(_, _)
        | MoveFrom(_, _)
        | MoveToSender(_, _)
        | BrTrue(_)
        | BrFalse(_)
        | Branch(_)
        | CopyLoc(_)
        | MoveLoc(_)
        | StLoc(_)
        | MutBorrowLoc(_)
        | ImmBorrowLoc(_) => true,

        // List out the other options explicitly so there's a compile error if a new
        // bytecode gets added.
        FreezeRef | Pop | Ret | LdConst(_) | LdTrue | LdFalse | ReadRef | WriteRef | Add | Sub
        | Mul | Mod | Div | BitOr | BitAnd | Xor | Or | And | Not | Eq | Neq | Lt | Gt | Le
        | Ge | Abort | GetTxnGasUnitPrice | GetTxnMaxGasUnits | GetGasRemaining
        | GetTxnSenderAddress | GetTxnSequenceNumber | GetTxnPublicKey => false,
    }
}
