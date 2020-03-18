// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use vm::file_format::{
    AddressPoolIndex, ByteArrayPoolIndex, CodeOffset, FieldDefinitionIndex, FunctionHandleIndex,
    LocalsSignatureIndex, StructDefinitionIndex,
};

pub type TempIndex = usize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StacklessBytecode {
    MoveLoc(TempIndex, TempIndex),   // t = move(l)
    CopyLoc(TempIndex, TempIndex),   // t = copy(l)
    StLoc(TempIndex, TempIndex),     // l = t
    BorrowLoc(TempIndex, TempIndex), // t1 = &t2

    ReadRef(TempIndex, TempIndex),   // t1 = *t2
    WriteRef(TempIndex, TempIndex),  // *t1 = t2
    FreezeRef(TempIndex, TempIndex), // t1 = immutable(t2)

    Call(
        Vec<TempIndex>,
        FunctionHandleIndex,
        LocalsSignatureIndex,
        Vec<TempIndex>,
    ), /* t1_vec = call(index) with
        * t2_vec as parameters */
    Ret(Vec<TempIndex>),

    Pack(
        TempIndex,
        StructDefinitionIndex,
        LocalsSignatureIndex,
        Vec<TempIndex>,
    ), /* t1 = struct(index) with t2_vec
        * as fields */
    Unpack(
        Vec<TempIndex>,
        StructDefinitionIndex,
        LocalsSignatureIndex,
        TempIndex,
    ), // t1_vec = t2's fields
    BorrowField(TempIndex, TempIndex, FieldDefinitionIndex), // t1 = t2.field
    MoveToSender(TempIndex, StructDefinitionIndex, LocalsSignatureIndex), /* move_to_sender<struct_index>(t) */
    MoveFrom(
        TempIndex,
        TempIndex,
        StructDefinitionIndex,
        LocalsSignatureIndex,
    ), /* t1 = move_from<struct_index>(t2) */
    BorrowGlobal(
        TempIndex,
        TempIndex,
        StructDefinitionIndex,
        LocalsSignatureIndex,
    ), /* t1 = borrow_global<struct_index>(t2) */
    Exists(
        TempIndex,
        TempIndex,
        StructDefinitionIndex,
        LocalsSignatureIndex,
    ), /* t1 = exists<struct_index>(t2) */

    GetGasRemaining(TempIndex),
    GetTxnSequenceNumber(TempIndex),
    GetTxnPublicKey(TempIndex),
    GetTxnSenderAddress(TempIndex),
    GetTxnMaxGasUnits(TempIndex),
    GetTxnGasUnitPrice(TempIndex),

    LdTrue(TempIndex),
    LdFalse(TempIndex),
    LdU8(TempIndex, u8),
    LdU64(TempIndex, u64),
    LdU128(TempIndex, u128),
    LdAddr(TempIndex, AddressPoolIndex),
    LdByteArray(TempIndex, ByteArrayPoolIndex),

    CastU8(TempIndex, TempIndex),
    CastU64(TempIndex, TempIndex),
    CastU128(TempIndex, TempIndex),

    Not(TempIndex, TempIndex),            // t1 = !t2
    Add(TempIndex, TempIndex, TempIndex), // t1 = t2 binop t3
    Sub(TempIndex, TempIndex, TempIndex),
    Mul(TempIndex, TempIndex, TempIndex),
    Div(TempIndex, TempIndex, TempIndex),
    Mod(TempIndex, TempIndex, TempIndex),
    BitOr(TempIndex, TempIndex, TempIndex),
    BitAnd(TempIndex, TempIndex, TempIndex),
    Xor(TempIndex, TempIndex, TempIndex),
    Shl(TempIndex, TempIndex, TempIndex),
    Shr(TempIndex, TempIndex, TempIndex),
    Lt(TempIndex, TempIndex, TempIndex),
    Gt(TempIndex, TempIndex, TempIndex),
    Le(TempIndex, TempIndex, TempIndex),
    Ge(TempIndex, TempIndex, TempIndex),
    Or(TempIndex, TempIndex, TempIndex),
    And(TempIndex, TempIndex, TempIndex),
    Eq(TempIndex, TempIndex, TempIndex),
    Neq(TempIndex, TempIndex, TempIndex),

    Branch(CodeOffset),
    BrTrue(CodeOffset, TempIndex),  // if(t) goto code_ooffset
    BrFalse(CodeOffset, TempIndex), // if(!t) goto code_offset

    Abort(TempIndex), // abort t
    Pop(TempIndex),
}

impl StacklessBytecode {
    pub fn is_unconditional_branch(&self) -> bool {
        matches!(
            self,
            StacklessBytecode::Ret(_) | StacklessBytecode::Abort(_) | StacklessBytecode::Branch(_)
        )
    }

    pub fn is_conditional_branch(&self) -> bool {
        matches!(
            self,
            StacklessBytecode::BrFalse(_, _) | StacklessBytecode::BrTrue(_, _)
        )
    }

    pub fn is_branch(&self) -> bool {
        self.is_conditional_branch() || self.is_unconditional_branch()
    }

    /// Return the destination of branching if self is a branching instruction
    pub fn branch_dest(&self) -> Option<&CodeOffset> {
        match self {
            StacklessBytecode::BrFalse(offset, _)
            | StacklessBytecode::BrTrue(offset, _)
            | StacklessBytecode::Branch(offset) => Some(offset),
            _ => None,
        }
    }

    /// Return the successor offsets of this instruction
    pub fn get_successors(pc: CodeOffset, code: &[StacklessBytecode]) -> Vec<CodeOffset> {
        let bytecode = &code[pc as usize];
        let mut v = vec![];

        if let Some(offset) = bytecode.branch_dest() {
            v.push(*offset);
        }

        let next_pc = pc + 1;
        if next_pc >= code.len() as CodeOffset {
            return v;
        }

        if !bytecode.is_unconditional_branch() && !v.contains(&next_pc) {
            // avoid duplicates
            v.push(next_pc);
        }

        // always give successors in ascending order
        if v.len() > 1 && v[0] > v[1] {
            v.swap(0, 1);
        }

        v
    }
}
