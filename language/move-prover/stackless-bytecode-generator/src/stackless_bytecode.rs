// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::function_target::FunctionTarget;
use num::BigUint;
use spec_lang::{
    env::{FunId, ModuleId, StructId},
    ty::{Type, TypeDisplayContext},
};
use std::{collections::BTreeMap, fmt, fmt::Formatter, rc::Rc};
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
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
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
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
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

// =================================================================================================
// Formatting

impl Bytecode {
    /// Creates a format object for a bytecode in context of a function target.
    pub fn display<'env>(&self, func_target: &'env FunctionTarget<'env>) -> BytecodeDisplay<'env> {
        BytecodeDisplay {
            bytecode: self.clone(),
            func_target,
        }
    }
}

/// A display object for a bytecode.
pub struct BytecodeDisplay<'env> {
    bytecode: Bytecode,
    func_target: &'env FunctionTarget<'env>,
}

impl<'env> fmt::Display for BytecodeDisplay<'env> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use Bytecode::*;
        match &self.bytecode {
            Assign(_, dst, src, AssignKind::Copy) => {
                write!(f, "{} := copy({})", self.lstr(*dst), self.lstr(*src))?
            }
            Assign(_, dst, src, AssignKind::Move) => {
                write!(f, "{} := move({})", self.lstr(*dst), self.lstr(*src))?
            }
            Assign(_, dst, src, AssignKind::Store) => {
                write!(f, "{} := {}", self.lstr(*dst), self.lstr(*src))?
            }
            ReadRef(_, dst, src) => write!(f, "{} := *{}", self.lstr(*dst), self.lstr(*src))?,
            WriteRef(_, dst, src) => write!(f, "*{} := {}", self.lstr(*dst), self.lstr(*src))?,
            FreezeRef(_, dst, src) => {
                write!(f, "{} := freeze({})", self.lstr(*dst), self.lstr(*src))?
            }
            Call(_, dsts, mid, fid, targs, args) => {
                let func_env = self
                    .func_target
                    .global_env()
                    .get_module(*mid)
                    .into_function(*fid);
                if !dsts.is_empty() {
                    self.fmt_locals(f, dsts, false)?;
                    write!(f, " := ")?;
                }
                write!(
                    f,
                    "{}::{}",
                    func_env
                        .module_env
                        .get_name()
                        .display(func_env.symbol_pool()),
                    func_env.get_name().display(func_env.symbol_pool())
                )?;
                self.fmt_type_args(f, targs)?;
                self.fmt_locals(f, args, true)?;
            }
            Ret(_, srcs) => {
                write!(f, "return ")?;
                self.fmt_locals(f, srcs, false)?;
            }
            Pack(_, dst, mid, sid, targs, args) => {
                write!(
                    f,
                    "{} := pack {}",
                    self.lstr(*dst),
                    self.struct_str(*mid, *sid, targs)
                )?;
                self.fmt_locals(f, args, true)?;
            }
            Unpack(_, dsts, mid, sid, targs, src) => {
                self.fmt_locals(f, dsts, false)?;
                write!(f, " := unpack {}", self.struct_str(*mid, *sid, targs))?;
                self.fmt_type_args(f, targs)?;
                self.fmt_locals(f, &[*src], true)?;
            }
            BorrowLoc(_, dst, src) => {
                write!(
                    f,
                    "{} := {}{}",
                    self.lstr(*dst),
                    self.borrow_op(dst),
                    self.lstr(*src)
                )?;
            }
            BorrowField(_, dst, src, mid, sid, offset) => {
                write!(
                    f,
                    "{} := {}{}",
                    self.lstr(*dst),
                    self.borrow_op(dst),
                    self.lstr(*src)
                )?;
                let struct_env = self
                    .func_target
                    .global_env()
                    .get_module(*mid)
                    .into_struct(*sid);
                let field_env = struct_env.get_field_by_offset(*offset);
                write!(
                    f,
                    ".{}",
                    field_env.get_name().display(struct_env.symbol_pool())
                )?;
            }
            BorrowGlobal(_, dst, addr, mid, sid, targs) => {
                write!(
                    f,
                    "{} := {}global<{}>",
                    self.lstr(*dst),
                    self.borrow_op(dst),
                    self.struct_str(*mid, *sid, targs)
                )?;
                self.fmt_locals(f, &[*addr], true)?;
            }
            MoveToSender(_, src, mid, sid, targs) => {
                write!(f, "move_to_sender<{}>", self.struct_str(*mid, *sid, targs))?;
                self.fmt_locals(f, &[*src], true)?;
            }
            MoveFrom(_, dst, src, mid, sid, targs) => {
                write!(
                    f,
                    "{} := move_from<{}>",
                    self.lstr(*dst),
                    self.struct_str(*mid, *sid, targs)
                )?;
                self.fmt_locals(f, &[*src], true)?;
            }
            Exists(_, dst, src, mid, sid, targs) => {
                write!(
                    f,
                    "{} := exists<{}>",
                    self.lstr(*dst),
                    self.struct_str(*mid, *sid, targs)
                )?;
                self.fmt_locals(f, &[*src], true)?;
            }
            Load(_, dst, cons) => {
                write!(f, "{} := {}", self.lstr(*dst), cons)?;
            }
            Unary(_, op, dst, src) => {
                write!(f, "{} := {} {}", self.lstr(*dst), op, self.lstr(*src))?;
            }
            Binary(_, op, dst, src1, src2) => {
                write!(
                    f,
                    "{} := {} {} {}",
                    self.lstr(*dst),
                    self.lstr(*src1),
                    op,
                    self.lstr(*src2)
                )?;
            }
            Branch(_, label, cond) => {
                use BranchCond::*;
                match cond {
                    True(src) => {
                        write!(f, "if ({}) ", self.lstr(*src))?;
                    }
                    False(src) => {
                        write!(f, "if (!{}) ", self.lstr(*src))?;
                    }
                    _ => {}
                }
                write!(f, "goto L{}", label.as_usize())?;
            }
            Labeled(_, label, bytecode) => {
                write!(
                    f,
                    "L{}: {}",
                    label.as_usize(),
                    bytecode.display(self.func_target)
                )?;
            }
            Abort(_, src) => {
                write!(f, "abort({})", self.lstr(*src))?;
            }
            Destroy(_, src) => {
                write!(f, "destroy({})", self.lstr(*src))?;
            }
            Nop(_) => {
                write!(f, "nop")?;
            }
        }
        Ok(())
    }
}

impl<'env> BytecodeDisplay<'env> {
    fn fmt_locals(
        &self,
        f: &mut Formatter<'_>,
        locals: &[TempIndex],
        always_brace: bool,
    ) -> fmt::Result {
        if !always_brace && locals.len() == 1 {
            write!(f, "{}", self.lstr(locals[0]))?
        } else {
            write!(f, "(")?;
            for (i, l) in locals.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", self.lstr(*l))?;
            }
            write!(f, ")")?;
        }
        Ok(())
    }

    fn lstr(&self, idx: TempIndex) -> Rc<String> {
        self.func_target
            .symbol_pool()
            .string(self.func_target.get_local_name(idx))
    }

    fn borrow_op(&self, idx: &TempIndex) -> &str {
        if self.func_target.get_local_type(*idx).is_mutable_reference() {
            "&mut "
        } else {
            "&"
        }
    }

    fn fmt_type_args(&self, f: &mut Formatter<'_>, targs: &[Type]) -> fmt::Result {
        if !targs.is_empty() {
            let tctx = TypeDisplayContext::WithEnv {
                env: self.func_target.global_env(),
            };
            write!(f, "<")?;
            for (i, ty) in targs.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", ty.display(&tctx))?;
            }
            write!(f, ">")?;
        }
        Ok(())
    }

    fn struct_str(&self, mid: ModuleId, sid: StructId, targs: &[Type]) -> String {
        let ty = Type::Struct(mid, sid, targs.to_vec());
        let tctx = TypeDisplayContext::WithEnv {
            env: self.func_target.global_env(),
        };
        format!("{}", ty.display(&tctx))
    }
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use BinaryOp::*;
        match self {
            Add => write!(f, "+")?,
            Sub => write!(f, "-")?,
            Mul => write!(f, "*")?,
            Div => write!(f, "/")?,
            Mod => write!(f, "%")?,
            BitOr => write!(f, "|")?,
            BitAnd => write!(f, "&")?,
            Xor => write!(f, "^")?,
            Shl => write!(f, "<<")?,
            Shr => write!(f, "<<")?,
            Lt => write!(f, "<")?,
            Gt => write!(f, ">")?,
            Le => write!(f, "<=")?,
            Ge => write!(f, ">=")?,
            Or => write!(f, "||")?,
            And => write!(f, "&&")?,
            Eq => write!(f, "==")?,
            Neq => write!(f, "!=")?,
        }
        Ok(())
    }
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use UnaryOp::*;
        match self {
            CastU8 => write!(f, "(u8)")?,
            CastU64 => write!(f, "(u64)")?,
            CastU128 => write!(f, "(u128)")?,
            Not => write!(f, "!")?,
        }
        Ok(())
    }
}

impl fmt::Display for Constant {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use Constant::*;
        match self {
            Bool(x) => write!(f, "{}", x)?,
            U8(x) => write!(f, "{}", x)?,
            U64(x) => write!(f, "{}", x)?,
            U128(x) => write!(f, "{}", x)?,
            Address(x) => write!(f, "0x{}", x.to_str_radix(16))?,
            ByteArray(x) => write!(f, "{:?}", x)?,
            TxnSenderAddress => write!(f, "txn_sender")?,
        }
        Ok(())
    }
}
