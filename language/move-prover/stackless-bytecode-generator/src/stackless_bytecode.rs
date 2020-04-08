// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use num::BigUint;
use spec_lang::{
    env::{FunId, ModuleId, StructId},
    ty::Type,
};
use vm::file_format::CodeOffset;

pub type TempIndex = usize;

/// The kind of an assignment in the bytecode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssignKind {
    /// The assign copies the lhs value.
    Copy,
    /// The assign moves the lhs value.
    Move,
    /// The assign stores the lhs value.
    // TODO: figure out why we can't treat this as either copy or move. The lifetime analysis
    // currently makes a difference of this case. It originates from stack code where Copy
    // and Move push on the stack and Store pops.
    Store,
}

/// A constant value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Constant {
    Bool(bool),
    U8(u8),
    U64(u64),
    U128(u128),
    Address(BigUint),
    ByteArray(Vec<u8>),
    TxnSenderAddress,
}

/// A unary operation.
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum UnaryOp {
    CastU8,
    CastU64,
    CastU128,
    Not,
}

/// A binary operation.
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    BitOr,
    BitAnd,
    Xor,
    Shl,
    Shr,
    Lt,
    Gt,
    Le,
    Ge,
    Or,
    And,
    Eq,
    Neq,
}

/// A branch condition.
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum BranchCond {
    Always,
    True(TempIndex),
    False(TempIndex),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Bytecode {
    Assign(TempIndex, TempIndex, AssignKind),

    ReadRef(TempIndex, TempIndex),
    WriteRef(TempIndex, TempIndex),
    FreezeRef(TempIndex, TempIndex),

    Call(Vec<TempIndex>, ModuleId, FunId, Vec<Type>, Vec<TempIndex>),
    Ret(Vec<TempIndex>),

    Pack(TempIndex, ModuleId, StructId, Vec<Type>, Vec<TempIndex>),
    Unpack(Vec<TempIndex>, ModuleId, StructId, Vec<Type>, TempIndex),

    BorrowLoc(TempIndex, TempIndex),
    BorrowField(TempIndex, TempIndex, ModuleId, StructId, usize),
    BorrowGlobal(TempIndex, TempIndex, ModuleId, StructId, Vec<Type>),

    MoveToSender(TempIndex, ModuleId, StructId, Vec<Type>),
    MoveFrom(TempIndex, TempIndex, ModuleId, StructId, Vec<Type>),
    Exists(TempIndex, TempIndex, ModuleId, StructId, Vec<Type>),

    Load(TempIndex, Constant),
    Unary(UnaryOp, TempIndex, TempIndex),
    Binary(BinaryOp, TempIndex, TempIndex, TempIndex),

    Branch(CodeOffset, BranchCond),
    Abort(TempIndex),
    Destroy(TempIndex),
}

impl Bytecode {
    pub fn is_unconditional_branch(&self) -> bool {
        matches!(
            self,
            Bytecode::Ret(_) | Bytecode::Abort(_) | Bytecode::Branch(_, BranchCond::Always)
        )
    }

    pub fn is_conditional_branch(&self) -> bool {
        matches!(
            self,
            Bytecode::Branch(_, BranchCond::False(_)) | Bytecode::Branch(_, BranchCond::True(_))
        )
    }

    pub fn is_branch(&self) -> bool {
        self.is_conditional_branch() || self.is_unconditional_branch()
    }

    /// Return the destination of branching if self is a branching instruction
    pub fn branch_dest(&self) -> Option<&CodeOffset> {
        match self {
            Bytecode::Branch(offset, _) => Some(offset),
            _ => None,
        }
    }

    /// Return the successor offsets of this instruction
    pub fn get_successors(pc: CodeOffset, code: &[Bytecode]) -> Vec<CodeOffset> {
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
