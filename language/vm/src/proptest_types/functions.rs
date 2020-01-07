// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    file_format::{
        AddressPoolIndex, ByteArrayPoolIndex, Bytecode, CodeOffset, CodeUnit, FieldDefinitionIndex,
        FunctionDefinition, FunctionHandle, FunctionHandleIndex, FunctionSignature,
        FunctionSignatureIndex, IdentifierIndex, LocalIndex, LocalsSignature, LocalsSignatureIndex,
        ModuleHandleIndex, StructDefinitionIndex, TableIndex, NO_TYPE_ACTUALS,
    },
    proptest_types::{
        signature::{FunctionSignatureGen, SignatureTokenGen},
        TableSize,
    },
};
use proptest::{
    collection::{vec, SizeRange},
    prelude::*,
    sample::{select, Index as PropIndex},
};

/// Represents state required to materialize final data structures for function definitions.
#[derive(Debug)]
pub struct FnDefnMaterializeState {
    pub struct_handles_len: usize,
    pub address_pool_len: usize,
    pub identifiers_len: usize,
    pub byte_array_pool_len: usize,
    pub type_signatures_len: usize,
    pub field_defs_len: usize,
    pub struct_defs_len: usize,
    pub function_defs_len: usize,
    // This is the final length of function_handles, after all the definitions add their own
    // handles.
    pub function_handles_len: usize,
    // These get mutated by `FunctionDefinitionGen`.
    pub function_signatures: Vec<FunctionSignature>,
    pub locals_signatures: Vec<LocalsSignature>,
    pub function_handles: Vec<FunctionHandle>,
}

impl FnDefnMaterializeState {
    #[inline]
    fn add_function_signature(&mut self, sig: FunctionSignature) -> FunctionSignatureIndex {
        precondition!(self.function_signatures.len() < TableSize::max_value() as usize);
        self.function_signatures.push(sig);
        FunctionSignatureIndex::new((self.function_signatures.len() - 1) as TableIndex)
    }

    #[inline]
    fn add_locals_signature(&mut self, sig: LocalsSignature) -> LocalsSignatureIndex {
        precondition!(self.locals_signatures.len() < TableSize::max_value() as usize);
        self.locals_signatures.push(sig);
        LocalsSignatureIndex::new((self.locals_signatures.len() - 1) as TableIndex)
    }

    #[inline]
    fn add_function_handle(&mut self, handle: FunctionHandle) -> FunctionHandleIndex {
        precondition!(self.function_handles.len() < TableSize::max_value() as usize);
        self.function_handles.push(handle);
        FunctionHandleIndex::new((self.function_handles.len() - 1) as TableIndex)
    }
}

#[derive(Clone, Debug)]
pub struct FunctionDefinitionGen {
    name: PropIndex,
    signature: FunctionSignatureGen,
    is_public: bool,
    acquires: Vec<PropIndex>,
    code: CodeUnitGen,
}

impl FunctionDefinitionGen {
    pub fn strategy(
        return_count: impl Into<SizeRange>,
        arg_count: impl Into<SizeRange>,
        kind_count: impl Into<SizeRange>,
        acquires_count: impl Into<SizeRange>,
        code_len: impl Into<SizeRange>,
    ) -> impl Strategy<Value = Self> {
        let return_count = return_count.into();
        let arg_count = arg_count.into();
        (
            any::<PropIndex>(),
            FunctionSignatureGen::strategy(return_count, arg_count.clone(), kind_count.into()),
            any::<bool>(),
            vec(any::<PropIndex>(), acquires_count.into()),
            CodeUnitGen::strategy(arg_count, code_len),
        )
            .prop_map(|(name, signature, is_public, acquires, code)| Self {
                name,
                signature,
                is_public,
                acquires,
                code,
            })
    }

    pub fn materialize(self, state: &mut FnDefnMaterializeState) -> FunctionDefinition {
        // This precondition should never fail because the table size cannot be greater
        // than TableSize::max_value()
        checked_precondition!(
            state.function_signatures.len() < TableSize::max_value() as usize
                && state.locals_signatures.len() < TableSize::max_value() as usize
                && state.function_handles.len() < TableSize::max_value() as usize
        );
        let signature = self.signature.materialize(state.struct_handles_len);

        let handle = FunctionHandle {
            // 0 represents the current module
            module: ModuleHandleIndex::new(0),
            // XXX need to guarantee uniqueness of names?
            name: IdentifierIndex::new(self.name.index(state.identifiers_len) as TableIndex),
            signature: state.add_function_signature(signature),
        };
        let function_handle = state.add_function_handle(handle);
        let acquires_global_resources = self
            .acquires
            .into_iter()
            .map(|idx| StructDefinitionIndex::new(idx.index(state.struct_defs_len) as TableIndex))
            .collect();
        FunctionDefinition {
            function: function_handle,
            // XXX is this even correct?
            flags: if self.is_public {
                CodeUnit::PUBLIC
            } else {
                // No qualifiers.
                0
            },
            acquires_global_resources,
            code: self.code.materialize(state),
        }
    }
}

#[derive(Clone, Debug)]
struct CodeUnitGen {
    locals_signature: Vec<SignatureTokenGen>,
    code: Vec<BytecodeGen>,
}

impl CodeUnitGen {
    fn strategy(
        arg_count: impl Into<SizeRange>,
        code_len: impl Into<SizeRange>,
    ) -> impl Strategy<Value = Self> {
        (
            vec(SignatureTokenGen::strategy(), arg_count),
            vec(BytecodeGen::garbage_strategy(), code_len),
        )
            .prop_map(|(locals_signature, code)| Self {
                locals_signature,
                code,
            })
    }

    fn materialize(self, state: &mut FnDefnMaterializeState) -> CodeUnit {
        precondition!(state.locals_signatures.len() < TableSize::max_value() as usize);
        let locals_signature = LocalsSignature(
            self.locals_signature
                .into_iter()
                .map(|sig| sig.materialize(state.struct_handles_len))
                .collect(),
        );

        // Not all bytecodes will be successfully materialized -- count how many will.
        let code_len = self
            .code
            .iter()
            .filter(|code| code.will_materialize(state, &locals_signature))
            .count();

        let code = self
            .code
            .into_iter()
            .filter_map(|code| code.materialize(state, code_len, &locals_signature))
            .collect();

        CodeUnit {
            max_stack_size: 0,
            locals: state.add_locals_signature(locals_signature),
            // XXX actually generate code
            code,
        }
    }
}

#[derive(Clone, Debug)]
enum BytecodeGen {
    // "Simple" means this doesn't refer to any other indexes.
    Simple(Bytecode),
    // All of these refer to other indexes.
    LdAddr(PropIndex),
    LdByteArray(PropIndex),
    MutBorrowField(PropIndex),
    ImmBorrowField(PropIndex),
    Call(PropIndex, PropIndex),
    Pack(PropIndex, PropIndex),
    Unpack(PropIndex, PropIndex),
    Exists(PropIndex, PropIndex),
    MutBorrowGlobal(PropIndex, PropIndex),
    ImmBorrowGlobal(PropIndex, PropIndex),
    MoveFrom(PropIndex, PropIndex),
    MoveToSender(PropIndex, PropIndex),
    BrTrue(PropIndex),
    BrFalse(PropIndex),
    Branch(PropIndex),
    CopyLoc(PropIndex),
    MoveLoc(PropIndex),
    StLoc(PropIndex),
    MutBorrowLoc(PropIndex),
    ImmBorrowLoc(PropIndex),
}

impl BytecodeGen {
    // This just generates nonsensical bytecodes. This will be cleaned up later as the generation
    // model is refined.
    fn garbage_strategy() -> impl Strategy<Value = Self> {
        use BytecodeGen::*;

        prop_oneof![
            Self::simple_bytecode_strategy().prop_map(Simple),
            any::<PropIndex>().prop_map(LdAddr),
            any::<PropIndex>().prop_map(LdByteArray),
            any::<PropIndex>().prop_map(ImmBorrowField),
            any::<PropIndex>().prop_map(MutBorrowField),
            (any::<PropIndex>(), any::<PropIndex>(),).prop_map(|(idx, types)| Call(idx, types)),
            (any::<PropIndex>(), any::<PropIndex>(),).prop_map(|(idx, types)| Pack(idx, types)),
            (any::<PropIndex>(), any::<PropIndex>(),).prop_map(|(idx, types)| Unpack(idx, types)),
            (any::<PropIndex>(), any::<PropIndex>(),).prop_map(|(idx, types)| Exists(idx, types)),
            (any::<PropIndex>(), any::<PropIndex>(),)
                .prop_map(|(idx, types)| ImmBorrowGlobal(idx, types)),
            (any::<PropIndex>(), any::<PropIndex>(),)
                .prop_map(|(idx, types)| MutBorrowGlobal(idx, types)),
            (any::<PropIndex>(), any::<PropIndex>(),).prop_map(|(idx, types)| MoveFrom(idx, types)),
            (any::<PropIndex>(), any::<PropIndex>(),)
                .prop_map(|(idx, types)| MoveToSender(idx, types)),
            any::<PropIndex>().prop_map(BrTrue),
            any::<PropIndex>().prop_map(BrFalse),
            any::<PropIndex>().prop_map(Branch),
            any::<PropIndex>().prop_map(CopyLoc),
            any::<PropIndex>().prop_map(MoveLoc),
            any::<PropIndex>().prop_map(StLoc),
            any::<PropIndex>().prop_map(MutBorrowLoc),
            any::<PropIndex>().prop_map(ImmBorrowLoc),
        ]
    }

    /// Whether this code will be materialized into a Some(bytecode).
    fn will_materialize(
        &self,
        state: &FnDefnMaterializeState,
        locals_signature: &LocalsSignature,
    ) -> bool {
        // This method should remain in sync with the `None` below.
        use BytecodeGen::*;

        match self {
            MutBorrowField(_) | ImmBorrowField(_) => state.field_defs_len != 0,
            CopyLoc(_) | MoveLoc(_) | StLoc(_) | MutBorrowLoc(_) | ImmBorrowLoc(_) => {
                !locals_signature.is_empty()
            }
            _ => true,
        }
    }

    fn materialize(
        self,
        state: &FnDefnMaterializeState,
        code_len: usize,
        locals_signature: &LocalsSignature,
    ) -> Option<Bytecode> {
        // This method returns an Option<Bytecode> because some bytecodes cannot be represented if
        // some tables are empty.
        //
        // Once more sensible function bodies are generated this will probably have to start using
        // prop_flat_map anyway, so revisit this then.

        let bytecode = match self {
            BytecodeGen::Simple(bytecode) => bytecode,
            BytecodeGen::LdAddr(idx) => Bytecode::LdAddr(AddressPoolIndex::new(
                idx.index(state.address_pool_len) as TableIndex,
            )),
            BytecodeGen::LdByteArray(idx) => Bytecode::LdByteArray(ByteArrayPoolIndex::new(
                idx.index(state.byte_array_pool_len) as TableIndex,
            )),
            BytecodeGen::MutBorrowField(idx) => {
                // Again, once meaningful bytecodes are generated this won't actually be a
                // possibility since it would be impossible to load a field from a struct that
                // doesn't have any.
                if state.field_defs_len == 0 {
                    return None;
                }
                Bytecode::MutBorrowField(FieldDefinitionIndex::new(
                    idx.index(state.field_defs_len) as TableIndex
                ))
            }
            BytecodeGen::ImmBorrowField(idx) => {
                // Same situation as above
                if state.field_defs_len == 0 {
                    return None;
                }
                Bytecode::ImmBorrowField(FieldDefinitionIndex::new(
                    idx.index(state.field_defs_len) as TableIndex
                ))
            }
            BytecodeGen::Call(idx, _types_idx) => Bytecode::Call(
                FunctionHandleIndex::new(idx.index(state.function_handles_len) as TableIndex),
                // TODO: generate random index to type actuals once generics is fully implemented
                NO_TYPE_ACTUALS,
            ),
            BytecodeGen::Pack(idx, _types_idx) => Bytecode::Pack(
                StructDefinitionIndex::new(idx.index(state.struct_defs_len) as TableIndex),
                // TODO: generate random index to type actuals once generics is fully implemented
                NO_TYPE_ACTUALS,
            ),
            BytecodeGen::Unpack(idx, _types_idx) => Bytecode::Unpack(
                StructDefinitionIndex::new(idx.index(state.struct_defs_len) as TableIndex),
                // TODO: generate random index to type actuals once generics is fully implemented
                NO_TYPE_ACTUALS,
            ),
            BytecodeGen::Exists(idx, _types_idx) => Bytecode::Exists(
                StructDefinitionIndex::new(idx.index(state.struct_defs_len) as TableIndex),
                // TODO: generate random index to type actuals once generics is fully implemented
                NO_TYPE_ACTUALS,
            ),
            BytecodeGen::ImmBorrowGlobal(idx, _types_idx) => Bytecode::ImmBorrowGlobal(
                StructDefinitionIndex::new(idx.index(state.struct_defs_len) as TableIndex),
                // TODO: generate random index to type actuals once generics is fully implemented
                NO_TYPE_ACTUALS,
            ),
            BytecodeGen::MutBorrowGlobal(idx, _types_idx) => Bytecode::MutBorrowGlobal(
                StructDefinitionIndex::new(idx.index(state.struct_defs_len) as TableIndex),
                // TODO: generate random index to type actuals once generics is fully implemented
                NO_TYPE_ACTUALS,
            ),
            BytecodeGen::MoveFrom(idx, _types_idx) => Bytecode::MoveFrom(
                StructDefinitionIndex::new(idx.index(state.struct_defs_len) as TableIndex),
                // TODO: generate random index to type actuals once generics is fully implemented
                NO_TYPE_ACTUALS,
            ),
            BytecodeGen::MoveToSender(idx, _types_idx) => Bytecode::MoveToSender(
                StructDefinitionIndex::new(idx.index(state.struct_defs_len) as TableIndex),
                // TODO: generate random index to type actuals once generics is fully implemented
                NO_TYPE_ACTUALS,
            ),
            BytecodeGen::BrTrue(idx) => Bytecode::BrTrue(idx.index(code_len) as CodeOffset),
            BytecodeGen::BrFalse(idx) => Bytecode::BrFalse(idx.index(code_len) as CodeOffset),
            BytecodeGen::Branch(idx) => Bytecode::Branch(idx.index(code_len) as CodeOffset),
            BytecodeGen::CopyLoc(idx) => {
                if locals_signature.is_empty() {
                    return None;
                }
                Bytecode::CopyLoc(idx.index(locals_signature.len()) as LocalIndex)
            }
            BytecodeGen::MoveLoc(idx) => {
                if locals_signature.is_empty() {
                    return None;
                }
                Bytecode::MoveLoc(idx.index(locals_signature.len()) as LocalIndex)
            }
            BytecodeGen::StLoc(idx) => {
                if locals_signature.is_empty() {
                    return None;
                }
                Bytecode::StLoc(idx.index(locals_signature.len()) as LocalIndex)
            }
            BytecodeGen::MutBorrowLoc(idx) => {
                if locals_signature.is_empty() {
                    return None;
                }
                Bytecode::MutBorrowLoc(idx.index(locals_signature.len()) as LocalIndex)
            }
            BytecodeGen::ImmBorrowLoc(idx) => {
                if locals_signature.is_empty() {
                    return None;
                }
                Bytecode::ImmBorrowLoc(idx.index(locals_signature.len()) as LocalIndex)
            }
        };

        Some(bytecode)
    }

    fn simple_bytecode_strategy() -> impl Strategy<Value = Bytecode> {
        prop_oneof![
            // The numbers are relative weights, somewhat arbitrarily picked.
            9 => Self::just_bytecode_strategy(),
            1 => any::<u64>().prop_map(Bytecode::LdU64),
        ]
    }

    fn just_bytecode_strategy() -> impl Strategy<Value = Bytecode> {
        use Bytecode::*;

        static JUST_BYTECODES: &[Bytecode] = &[
            FreezeRef,
            Pop,
            Ret,
            LdTrue,
            LdFalse,
            ReadRef,
            WriteRef,
            Add,
            Sub,
            Mul,
            Mod,
            Div,
            BitOr,
            BitAnd,
            Xor,
            Or,
            And,
            Eq,
            Neq,
            Lt,
            Gt,
            Le,
            Ge,
            Abort,
            GetTxnGasUnitPrice,
            GetTxnMaxGasUnits,
            GetTxnSenderAddress,
            GetTxnSequenceNumber,
            GetTxnPublicKey,
        ];
        select(JUST_BYTECODES)
    }
}
