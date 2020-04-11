// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use num::BigUint;
use spec_lang::{
    env::{FunId, ModuleId, StructId},
    ty::Type,
};
use std::collections::BTreeMap;
use vm::file_format::CodeOffset;

pub type TempIndex = usize;

/// A label for a branch destination.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct Label(u16);

impl Label {
    pub fn new(idx: usize) -> Self {
        Self(idx as u16)
    }

    pub fn as_usize(self) -> usize {
        self.0 as usize
    }
}

/// An id for an attribute attached to an instruction.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct AttrId(u16);

impl AttrId {
    pub fn new(idx: usize) -> Self {
        Self(idx as u16)
    }

    pub fn as_usize(self) -> usize {
        self.0 as usize
    }
}

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
    Assign(AttrId, TempIndex, TempIndex, AssignKind),

    ReadRef(AttrId, TempIndex, TempIndex),
    WriteRef(AttrId, TempIndex, TempIndex),
    FreezeRef(AttrId, TempIndex, TempIndex),

    Call(
        AttrId,
        Vec<TempIndex>,
        ModuleId,
        FunId,
        Vec<Type>,
        Vec<TempIndex>,
    ),
    Ret(AttrId, Vec<TempIndex>),

    Pack(
        AttrId,
        TempIndex,
        ModuleId,
        StructId,
        Vec<Type>,
        Vec<TempIndex>,
    ),
    Unpack(
        AttrId,
        Vec<TempIndex>,
        ModuleId,
        StructId,
        Vec<Type>,
        TempIndex,
    ),

    BorrowLoc(AttrId, TempIndex, TempIndex),
    BorrowField(AttrId, TempIndex, TempIndex, ModuleId, StructId, usize),
    BorrowGlobal(AttrId, TempIndex, TempIndex, ModuleId, StructId, Vec<Type>),

    MoveToSender(AttrId, TempIndex, ModuleId, StructId, Vec<Type>),
    MoveFrom(AttrId, TempIndex, TempIndex, ModuleId, StructId, Vec<Type>),
    Exists(AttrId, TempIndex, TempIndex, ModuleId, StructId, Vec<Type>),

    Load(AttrId, TempIndex, Constant),
    Unary(AttrId, UnaryOp, TempIndex, TempIndex),
    Binary(AttrId, BinaryOp, TempIndex, TempIndex, TempIndex),

    Branch(AttrId, Label, BranchCond),
    Labeled(AttrId, Label, Box<Bytecode>),
    Abort(AttrId, TempIndex),
    Destroy(AttrId, TempIndex),
    Nop(AttrId),
}

impl Bytecode {
    pub fn get_attr_id(&self) -> AttrId {
        use Bytecode::*;
        match self {
            Assign(id, ..)
            | ReadRef(id, ..)
            | WriteRef(id, ..)
            | FreezeRef(id, ..)
            | Call(id, ..)
            | Ret(id, ..)
            | Pack(id, ..)
            | Unpack(id, ..)
            | BorrowLoc(id, ..)
            | BorrowField(id, ..)
            | BorrowGlobal(id, ..)
            | MoveToSender(id, ..)
            | MoveFrom(id, ..)
            | Exists(id, ..)
            | Load(id, ..)
            | Unary(id, ..)
            | Binary(id, ..)
            | Branch(id, ..)
            | Labeled(id, ..)
            | Abort(id, ..)
            | Destroy(id, ..)
            | Nop(id) => *id,
        }
    }

    pub fn is_unconditional_branch(&self) -> bool {
        matches!(
            self,
            Bytecode::Ret(..) | Bytecode::Abort(..) | Bytecode::Branch(_, _, BranchCond::Always)
        )
    }

    pub fn is_conditional_branch(&self) -> bool {
        matches!(
            self,
            Bytecode::Branch(_, _, BranchCond::False(_)) | Bytecode::Branch(_, _, BranchCond::True(_))
        )
    }

    pub fn is_branch(&self) -> bool {
        self.is_conditional_branch() || self.is_unconditional_branch()
    }

    pub fn skip_labelled(&self) -> &Bytecode {
        if let Bytecode::Labeled(_, _, code) = self {
            code
        } else {
            self
        }
    }

    /// Return the destination of branching if self is a branching instruction
    pub fn branch_dest(&self) -> Option<Label> {
        match self {
            Bytecode::Branch(_, label, _) => Some(*label),
            _ => None,
        }
    }

    /// Returns a mapping from labels to code offsets.
    pub fn label_offsets(code: &[Bytecode]) -> BTreeMap<Label, CodeOffset> {
        let mut res = BTreeMap::new();
        for (offs, bc) in code.iter().enumerate() {
            if let Bytecode::Labeled(_, label, ..) = bc {
                assert!(res.insert(*label, offs as CodeOffset).is_none());
            }
        }
        res
    }

    /// Return the successor offsets of this instruction. In addition to the code, a map
    /// of label to code offset need to be passed in.
    pub fn get_successors(
        pc: CodeOffset,
        code: &[Bytecode],
        label_offsets: &BTreeMap<Label, CodeOffset>,
    ) -> Vec<CodeOffset> {
        let bytecode = &code[pc as usize];
        let mut v = vec![];

        if let Some(label) = bytecode.branch_dest() {
            v.push(*label_offsets.get(&label).expect("label defined"));
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
