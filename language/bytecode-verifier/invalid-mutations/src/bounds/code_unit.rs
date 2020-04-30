// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_proptest_helpers::pick_slice_idxs;
use libra_types::vm_error::{StatusCode, VMStatus};
use proptest::{prelude::*, sample::Index as PropIndex};
use std::collections::BTreeMap;
use vm::{
    errors::{append_err_info, bytecode_offset_err},
    file_format::{
        Bytecode, CodeOffset, CompiledModuleMut, ConstantPoolIndex, FieldHandleIndex,
        FieldInstantiationIndex, FunctionHandleIndex, FunctionInstantiationIndex, LocalIndex,
        StructDefInstantiationIndex, StructDefinitionIndex, TableIndex,
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
        let code = self.module.function_defs[idx].code.as_mut().unwrap();
        let locals_len = self.module.signatures[code.locals.into_index()].len();
        let code = &mut code.code;
        let code_len = code.len();

        let interesting_offsets: Vec<usize> = (0..code.len())
            .filter(|bytecode_idx| is_interesting(&code[*bytecode_idx]))
            .collect();
        let to_mutate = pick_slice_idxs(interesting_offsets.len(), &mutations);

        // These have to be computed upfront because self.module is being mutated below.
        let constant_pool_len = self.module.constant_pool.len();
        let function_handles_len = self.module.function_handles.len();
        let field_handle_len = self.module.field_handles.len();
        let struct_defs_len = self.module.struct_defs.len();
        let struct_inst_len = self.module.struct_def_instantiations.len();
        let function_inst_len = self.module.function_instantiations.len();
        let field_inst_len = self.module.field_instantiations.len();

        mutations
            .iter()
            .zip(to_mutate)
            .map(|(mutation, interesting_offsets_idx)| {
                let bytecode_idx = interesting_offsets[interesting_offsets_idx];
                let offset = mutation.offset;
                use Bytecode::*;

                let (new_bytecode, err) = match code[bytecode_idx] {
                    LdConst(_) => new_bytecode!(
                        constant_pool_len,
                        bytecode_idx,
                        offset,
                        ConstantPoolIndex,
                        LdConst
                    ),
                    ImmBorrowField(_) => new_bytecode!(
                        field_handle_len,
                        bytecode_idx,
                        offset,
                        FieldHandleIndex,
                        ImmBorrowField
                    ),
                    ImmBorrowFieldGeneric(_) => new_bytecode!(
                        field_inst_len,
                        bytecode_idx,
                        offset,
                        FieldInstantiationIndex,
                        ImmBorrowFieldGeneric
                    ),
                    MutBorrowField(_) => new_bytecode!(
                        field_handle_len,
                        bytecode_idx,
                        offset,
                        FieldHandleIndex,
                        MutBorrowField
                    ),
                    MutBorrowFieldGeneric(_) => new_bytecode!(
                        field_inst_len,
                        bytecode_idx,
                        offset,
                        FieldInstantiationIndex,
                        MutBorrowFieldGeneric
                    ),
                    Call(_) => struct_bytecode!(
                        function_handles_len,
                        bytecode_idx,
                        offset,
                        FunctionHandleIndex,
                        Call
                    ),
                    CallGeneric(_) => struct_bytecode!(
                        function_inst_len,
                        bytecode_idx,
                        offset,
                        FunctionInstantiationIndex,
                        CallGeneric
                    ),
                    Pack(_) => struct_bytecode!(
                        struct_defs_len,
                        bytecode_idx,
                        offset,
                        StructDefinitionIndex,
                        Pack
                    ),
                    PackGeneric(_) => struct_bytecode!(
                        struct_inst_len,
                        bytecode_idx,
                        offset,
                        StructDefInstantiationIndex,
                        PackGeneric
                    ),
                    Unpack(_) => struct_bytecode!(
                        struct_defs_len,
                        bytecode_idx,
                        offset,
                        StructDefinitionIndex,
                        Unpack
                    ),
                    UnpackGeneric(_) => struct_bytecode!(
                        struct_inst_len,
                        bytecode_idx,
                        offset,
                        StructDefInstantiationIndex,
                        UnpackGeneric
                    ),
                    Exists(_) => struct_bytecode!(
                        struct_defs_len,
                        bytecode_idx,
                        offset,
                        StructDefinitionIndex,
                        Exists
                    ),
                    ExistsGeneric(_) => struct_bytecode!(
                        struct_inst_len,
                        bytecode_idx,
                        offset,
                        StructDefInstantiationIndex,
                        ExistsGeneric
                    ),
                    MutBorrowGlobal(_) => struct_bytecode!(
                        struct_defs_len,
                        bytecode_idx,
                        offset,
                        StructDefinitionIndex,
                        MutBorrowGlobal
                    ),
                    MutBorrowGlobalGeneric(_) => struct_bytecode!(
                        struct_inst_len,
                        bytecode_idx,
                        offset,
                        StructDefInstantiationIndex,
                        MutBorrowGlobalGeneric
                    ),
                    ImmBorrowGlobal(_) => struct_bytecode!(
                        struct_defs_len,
                        bytecode_idx,
                        offset,
                        StructDefinitionIndex,
                        ImmBorrowGlobal
                    ),
                    ImmBorrowGlobalGeneric(_) => struct_bytecode!(
                        struct_inst_len,
                        bytecode_idx,
                        offset,
                        StructDefInstantiationIndex,
                        ImmBorrowGlobalGeneric
                    ),
                    MoveFrom(_) => struct_bytecode!(
                        struct_defs_len,
                        bytecode_idx,
                        offset,
                        StructDefinitionIndex,
                        MoveFrom
                    ),
                    MoveFromGeneric(_) => struct_bytecode!(
                        struct_inst_len,
                        bytecode_idx,
                        offset,
                        StructDefInstantiationIndex,
                        MoveFromGeneric
                    ),
                    MoveToSender(_) => struct_bytecode!(
                        struct_defs_len,
                        bytecode_idx,
                        offset,
                        StructDefinitionIndex,
                        MoveToSender
                    ),
                    MoveToSenderGeneric(_) => struct_bytecode!(
                        struct_inst_len,
                        bytecode_idx,
                        offset,
                        StructDefInstantiationIndex,
                        MoveToSenderGeneric
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
                    FreezeRef | Pop | Ret | LdU8(_) | LdU64(_) | LdU128(_) | CastU8 | CastU64
                    | CastU128 | LdTrue | LdFalse | ReadRef | WriteRef | Add | Sub | Mul | Mod
                    | Div | BitOr | BitAnd | Xor | Shl | Shr | Or | And | Not | Eq | Neq | Lt
                    | Gt | Le | Ge | Abort | GetTxnGasUnitPrice | GetTxnMaxGasUnits
                    | GetGasRemaining | GetTxnSenderAddress | GetTxnSequenceNumber
                    | GetTxnPublicKey | Nop => {
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
        LdConst(_)
        | ImmBorrowField(_)
        | ImmBorrowFieldGeneric(_)
        | MutBorrowField(_)
        | MutBorrowFieldGeneric(_)
        | Call(_)
        | CallGeneric(_)
        | Pack(_)
        | PackGeneric(_)
        | Unpack(_)
        | UnpackGeneric(_)
        | Exists(_)
        | ExistsGeneric(_)
        | MutBorrowGlobal(_)
        | MutBorrowGlobalGeneric(_)
        | ImmBorrowGlobal(_)
        | ImmBorrowGlobalGeneric(_)
        | MoveFrom(_)
        | MoveFromGeneric(_)
        | MoveToSender(_)
        | MoveToSenderGeneric(_)
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
        FreezeRef | Pop | Ret | LdU8(_) | LdU64(_) | LdU128(_) | CastU8 | CastU64 | CastU128
        | LdTrue | LdFalse | ReadRef | WriteRef | Add | Sub | Mul | Mod | Div | BitOr | BitAnd
        | Xor | Shl | Shr | Or | And | Not | Eq | Neq | Lt | Gt | Le | Ge | Abort
        | GetTxnGasUnitPrice | GetTxnMaxGasUnits | GetGasRemaining | GetTxnSenderAddress
        | GetTxnSequenceNumber | GetTxnPublicKey | Nop => false,
    }
}
