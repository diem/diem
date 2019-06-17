// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use proptest::{prelude::*, sample::Index as PropIndex};
use proptest_helpers::pick_slice_idxs;
use std::collections::BTreeMap;
use vm::{
    errors::{VMStaticViolation, VerificationError},
    file_format::{
        AddressPoolIndex, ByteArrayPoolIndex, Bytecode, CodeOffset, CompiledModule,
        FieldDefinitionIndex, FunctionHandleIndex, LocalIndex, StringPoolIndex,
        StructDefinitionIndex, TableIndex,
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
    module: &'a mut CompiledModule,
    // This is so apply_one can be called after mutations has been iterated on.
    mutations: Option<Vec<CodeUnitBoundsMutation>>,
}

macro_rules! new_bytecode {
    ($dst_len: expr, $offset: expr, $idx_type: ident, $bytecode_ident: tt) => {{
        let dst_len = $dst_len;
        let new_idx = (dst_len + $offset) as TableIndex;
        (
            $bytecode_ident($idx_type::new(new_idx)),
            VMStaticViolation::IndexOutOfBounds($idx_type::KIND, dst_len, new_idx as usize),
        )
    }};
}

macro_rules! code_bytecode {
    ($code_len: expr, $offset: expr, $bytecode_ident: tt) => {{
        let code_len = $code_len;
        let new_idx = code_len + $offset;
        (
            $bytecode_ident(new_idx as CodeOffset),
            VMStaticViolation::IndexOutOfBounds(IndexKind::CodeDefinition, code_len, new_idx),
        )
    }};
}

macro_rules! locals_bytecode {
    ($locals_len: expr, $offset: expr, $bytecode_ident: tt) => {{
        let locals_len = $locals_len;
        let new_idx = locals_len + $offset;
        (
            $bytecode_ident(new_idx as LocalIndex),
            VMStaticViolation::IndexOutOfBounds(IndexKind::LocalPool, locals_len, new_idx),
        )
    }};
}

impl<'a> ApplyCodeUnitBoundsContext<'a> {
    pub fn new(module: &'a mut CompiledModule, mutations: Vec<CodeUnitBoundsMutation>) -> Self {
        Self {
            module,
            mutations: Some(mutations),
        }
    }

    pub fn apply(mut self) -> Vec<VerificationError> {
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

    fn apply_one(
        &mut self,
        idx: usize,
        mutations: Vec<CodeUnitBoundsMutation>,
    ) -> Vec<VerificationError> {
        // For this function def, find all the places where a bounds mutation can be applied.
        let (code_len, locals_len) = {
            let code = &mut self.module.function_defs[idx].code;
            (
                code.code.len(),
                self.module.locals_signatures[code.locals.into_index()].len(),
            )
        };

        let mut interesting: Vec<&mut Bytecode> = self.module.function_defs[idx]
            .code
            .code
            .iter_mut()
            .filter(|bytecode| is_interesting(*bytecode))
            .collect();
        let to_mutate = pick_slice_idxs(interesting.len(), &mutations);

        // These have to be computed upfront because self.module is being mutated below.
        let address_pool_len = self.module.address_pool.len();
        let string_pool_len = self.module.string_pool.len();
        let byte_array_pool_len = self.module.byte_array_pool.len();
        let function_handles_len = self.module.function_handles.len();
        let field_defs_len = self.module.field_defs.len();
        let struct_defs_len = self.module.struct_defs.len();

        mutations
            .iter()
            .zip(to_mutate)
            .map(|(mutation, bytecode_idx)| {
                let offset = mutation.offset;
                use Bytecode::*;

                let (new_bytecode, err) = match interesting[bytecode_idx] {
                    LdAddr(_) => new_bytecode!(address_pool_len, offset, AddressPoolIndex, LdAddr),
                    LdStr(_) => new_bytecode!(string_pool_len, offset, StringPoolIndex, LdStr),
                    LdByteArray(_) => {
                        new_bytecode!(byte_array_pool_len, offset, ByteArrayPoolIndex, LdByteArray)
                    }
                    BorrowField(_) => {
                        new_bytecode!(field_defs_len, offset, FieldDefinitionIndex, BorrowField)
                    }
                    Call(_) => {
                        new_bytecode!(function_handles_len, offset, FunctionHandleIndex, Call)
                    }
                    Pack(_) => new_bytecode!(struct_defs_len, offset, StructDefinitionIndex, Pack),
                    Unpack(_) => {
                        new_bytecode!(struct_defs_len, offset, StructDefinitionIndex, Unpack)
                    }
                    Exists(_) => {
                        new_bytecode!(struct_defs_len, offset, StructDefinitionIndex, Exists)
                    }
                    BorrowGlobal(_) => {
                        new_bytecode!(struct_defs_len, offset, StructDefinitionIndex, BorrowGlobal)
                    }
                    MoveFrom(_) => {
                        new_bytecode!(struct_defs_len, offset, StructDefinitionIndex, MoveFrom)
                    }
                    MoveToSender(_) => {
                        new_bytecode!(struct_defs_len, offset, StructDefinitionIndex, MoveToSender)
                    }
                    BrTrue(_) => code_bytecode!(code_len, offset, BrTrue),
                    BrFalse(_) => code_bytecode!(code_len, offset, BrFalse),
                    Branch(_) => code_bytecode!(code_len, offset, Branch),
                    CopyLoc(_) => locals_bytecode!(locals_len, offset, CopyLoc),
                    MoveLoc(_) => locals_bytecode!(locals_len, offset, MoveLoc),
                    StLoc(_) => locals_bytecode!(locals_len, offset, StLoc),
                    BorrowLoc(_) => locals_bytecode!(locals_len, offset, BorrowLoc),

                    // List out the other options explicitly so there's a compile error if a new
                    // bytecode gets added.
                    FreezeRef | ReleaseRef | Pop | Ret | LdConst(_) | LdTrue | LdFalse
                    | ReadRef | WriteRef | Add | Sub | Mul | Mod | Div | BitOr | BitAnd | Xor
                    | Or | And | Not | Eq | Neq | Lt | Gt | Le | Ge | Assert
                    | GetTxnGasUnitPrice | GetTxnMaxGasUnits | GetGasRemaining
                    | GetTxnSenderAddress | CreateAccount | EmitEvent | GetTxnSequenceNumber
                    | GetTxnPublicKey => panic!(
                        "Bytecode has no internal index: {:?}",
                        interesting[bytecode_idx]
                    ),
                };

                *interesting[bytecode_idx] = new_bytecode;

                VerificationError {
                    kind: IndexKind::FunctionDefinition,
                    idx,
                    err,
                }
            })
            .collect()
    }
}

fn is_interesting(bytecode: &Bytecode) -> bool {
    use Bytecode::*;

    match bytecode {
        LdAddr(_) | LdStr(_) | LdByteArray(_) | BorrowField(_) | Call(_) | Pack(_) | Unpack(_)
        | Exists(_) | BorrowGlobal(_) | MoveFrom(_) | MoveToSender(_) | BrTrue(_) | BrFalse(_)
        | Branch(_) | CopyLoc(_) | MoveLoc(_) | StLoc(_) | BorrowLoc(_) => true,

        // List out the other options explicitly so there's a compile error if a new
        // bytecode gets added.
        FreezeRef | ReleaseRef | Pop | Ret | LdConst(_) | LdTrue | LdFalse | ReadRef | WriteRef
        | Add | Sub | Mul | Mod | Div | BitOr | BitAnd | Xor | Or | And | Not | Eq | Neq | Lt
        | Gt | Le | Ge | Assert | GetTxnGasUnitPrice | GetTxnMaxGasUnits | GetGasRemaining
        | GetTxnSenderAddress | CreateAccount | EmitEvent | GetTxnSequenceNumber
        | GetTxnPublicKey => false,
    }
}
