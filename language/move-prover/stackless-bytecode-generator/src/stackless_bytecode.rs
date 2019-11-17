// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use vm::file_format::{
    AddressPoolIndex, ByteArrayPoolIndex, CodeOffset, FieldDefinitionIndex, FunctionHandleIndex,
    LocalIndex, LocalsSignatureIndex, StructDefinitionIndex, UserStringIndex,
};

type TempIndex = usize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StacklessBytecode {
    MoveLoc(TempIndex, LocalIndex),   // t = move(l)
    CopyLoc(TempIndex, LocalIndex),   // t = copy(l)
    StLoc(LocalIndex, TempIndex),     // l = t
    BorrowLoc(TempIndex, LocalIndex), // t1 = &t2

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
    LdConst(TempIndex, u64),
    LdAddr(TempIndex, AddressPoolIndex),
    LdByteArray(TempIndex, ByteArrayPoolIndex),
    LdStr(TempIndex, UserStringIndex),

    Not(TempIndex, TempIndex),            // t1 = !t2
    Add(TempIndex, TempIndex, TempIndex), // t1 = t2 binop t3
    Sub(TempIndex, TempIndex, TempIndex),
    Mul(TempIndex, TempIndex, TempIndex),
    Div(TempIndex, TempIndex, TempIndex),
    Mod(TempIndex, TempIndex, TempIndex),
    BitOr(TempIndex, TempIndex, TempIndex),
    BitAnd(TempIndex, TempIndex, TempIndex),
    Xor(TempIndex, TempIndex, TempIndex),
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
    NoOp,
}
