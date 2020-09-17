// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::function_target::FunctionTarget;
use itertools::Itertools;
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

/// An id for a spec block. A spec block can contain assumes and asserts to be enforced at a
/// program point.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct SpecBlockId(u16);

impl SpecBlockId {
    pub fn new(idx: usize) -> Self {
        Self(idx as u16)
    }

    pub fn as_usize(self) -> usize {
        self.0 as usize
    }
}

/// The kind of an assignment in the bytecode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
}

/// An operation -- target of a call. This contains user functions, builtin functions, and
/// operators.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operation {
    // User function
    Function(ModuleId, FunId, Vec<Type>),

    // Pack/Unpack
    Pack(ModuleId, StructId, Vec<Type>),
    Unpack(ModuleId, StructId, Vec<Type>),

    // Resources
    MoveTo(ModuleId, StructId, Vec<Type>),
    MoveFrom(ModuleId, StructId, Vec<Type>),
    Exists(ModuleId, StructId, Vec<Type>),

    // Borrow
    BorrowLoc,
    BorrowField(ModuleId, StructId, Vec<Type>, usize),
    BorrowGlobal(ModuleId, StructId, Vec<Type>),

    // Get
    GetField(ModuleId, StructId, Vec<Type>, usize),
    GetGlobal(ModuleId, StructId, Vec<Type>),

    // Builtins
    Destroy,
    ReadRef,
    WriteRef,
    FreezeRef,

    // Memory model
    WriteBack(BorrowNode),
    Splice(BTreeMap<usize, TempIndex>),
    UnpackRef,
    PackRef,
    UnpackRefDeep,
    PackRefDeep,

    // Unary
    CastU8,
    CastU64,
    CastU128,
    Not,

    // Binary
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

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct StructDecl {
    pub module_id: ModuleId,
    pub struct_id: StructId,
}

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub enum BorrowNode {
    GlobalRoot(StructDecl),
    LocalRoot(TempIndex),
    Reference(TempIndex),
}

impl BorrowNode {
    pub fn get_ref(&self) -> Option<TempIndex> {
        if let BorrowNode::Reference(idx) = self {
            Some(*idx)
        } else {
            None
        }
    }
}

/// A display object for a borrow node.
pub struct BorrowNodeDisplay<'env> {
    node: &'env BorrowNode,
    func_target: &'env FunctionTarget<'env>,
}

impl BorrowNode {
    /// Creates a format object for a borrow node in context of a function target.
    pub fn display<'env>(
        &'env self,
        func_target: &'env FunctionTarget<'env>,
    ) -> BorrowNodeDisplay<'env> {
        BorrowNodeDisplay {
            node: self,
            func_target,
        }
    }
}

impl<'env> fmt::Display for BorrowNodeDisplay<'env> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use BorrowNode::*;
        match self.node {
            GlobalRoot(s) => {
                let ty = Type::Struct(s.module_id, s.struct_id, vec![]);
                let tctx = TypeDisplayContext::WithEnv {
                    env: self.func_target.global_env(),
                    type_param_names: None,
                };
                write!(f, "{}", ty.display(&tctx))?;
            }
            LocalRoot(idx) => {
                write!(
                    f,
                    "LocalRoot({})",
                    self.func_target
                        .get_local_name(*idx)
                        .display(self.func_target.symbol_pool())
                )?;
            }
            Reference(idx) => {
                write!(
                    f,
                    "Reference({})",
                    self.func_target
                        .get_local_name(*idx)
                        .display(self.func_target.symbol_pool())
                )?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Bytecode {
    SpecBlock(AttrId, SpecBlockId),

    Assign(AttrId, TempIndex, TempIndex, AssignKind),

    Call(AttrId, Vec<TempIndex>, Operation, Vec<TempIndex>),
    Ret(AttrId, Vec<TempIndex>),

    Load(AttrId, TempIndex, Constant),
    Branch(AttrId, Label, Label, TempIndex),
    Jump(AttrId, Label),
    Label(AttrId, Label),
    Abort(AttrId, TempIndex),
    Nop(AttrId),
}

impl Bytecode {
    pub fn get_attr_id(&self) -> AttrId {
        use Bytecode::*;
        match self {
            SpecBlock(id, ..)
            | Assign(id, ..)
            | Call(id, ..)
            | Ret(id, ..)
            | Load(id, ..)
            | Branch(id, ..)
            | Jump(id, ..)
            | Label(id, ..)
            | Abort(id, ..)
            | Nop(id) => *id,
        }
    }

    pub fn is_exit(&self) -> bool {
        matches!(self, Bytecode::Ret(..) | Bytecode::Abort(..))
    }

    pub fn is_return(&self) -> bool {
        matches!(self, Bytecode::Ret(..))
    }

    pub fn is_unconditional_branch(&self) -> bool {
        matches!(
            self,
            Bytecode::Ret(..) | Bytecode::Jump(..) | Bytecode::Abort(..)
        )
    }

    pub fn is_conditional_branch(&self) -> bool {
        matches!(self, Bytecode::Branch(..))
    }

    pub fn is_branch(&self) -> bool {
        self.is_conditional_branch() || self.is_unconditional_branch()
    }

    /// Return the destination(s) if self is a branch/jump instruction
    pub fn branch_dests(&self) -> Vec<Label> {
        match self {
            Bytecode::Branch(_, then_label, else_label, _) => vec![*then_label, *else_label],
            Bytecode::Jump(_, label) => vec![*label],
            _ => vec![],
        }
    }

    /// Returns a mapping from labels to code offsets.
    pub fn label_offsets(code: &[Bytecode]) -> BTreeMap<Label, CodeOffset> {
        let mut res = BTreeMap::new();
        for (offs, bc) in code.iter().enumerate() {
            if let Bytecode::Label(_, label) = bc {
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
        assert!(bytecode.is_branch(), "{:?} at {}", bytecode, pc);
        let mut v = vec![];
        for label in bytecode.branch_dests() {
            v.push(*label_offsets.get(&label).expect("label defined"));
        }
        // always give successors in ascending order
        if v.len() > 1 && v[0] > v[1] {
            v.swap(0, 1);
        }
        v
    }

    /// Returns the code offsets at which the code exits(aborts or returns).
    pub fn get_exits(code: &[Bytecode]) -> Vec<CodeOffset> {
        code.iter()
            .enumerate()
            .filter(|(_, bytecode)| bytecode.is_exit())
            .map(|(idx, _)| idx as CodeOffset)
            .collect()
    }

    /// Remaps variables in the instruction.
    pub fn remap_vars<F>(self, f: &mut F) -> Self
    where
        F: FnMut(TempIndex) -> TempIndex,
    {
        use BorrowNode::*;
        use Bytecode::*;
        use Operation::*;
        let map = |f: &mut F, v: Vec<TempIndex>| -> Vec<TempIndex> {
            v.into_iter().map(|i| f(i)).collect()
        };
        match self {
            Load(attr, dst, cons) => Load(attr, f(dst), cons),
            Assign(attr, dest, src, kind) => Assign(attr, f(dest), f(src), kind),
            Call(attr, _, WriteBack(LocalRoot(dest)), srcs) => {
                Call(attr, vec![], WriteBack(LocalRoot(f(dest))), map(f, srcs))
            }
            Call(attr, _, WriteBack(Reference(dest)), srcs) => {
                Call(attr, vec![], WriteBack(Reference(f(dest))), map(f, srcs))
            }
            Call(attr, dests, Splice(m), srcs) => {
                let m = m.into_iter().map(|(p, t)| (p, f(t))).collect();
                Call(attr, map(f, dests), Splice(m), map(f, srcs))
            }
            Call(attr, dests, op, srcs) => Call(attr, map(f, dests), op, map(f, srcs)),
            Ret(attr, rets) => Ret(attr, map(f, rets)),
            Branch(attr, if_label, else_label, cond) => Branch(attr, if_label, else_label, f(cond)),
            Abort(attr, cond) => Abort(attr, f(cond)),
            _ => self,
        }
    }

    /// Return the temporaries this instruction modifies.
    pub fn modifies(&self) -> Vec<TempIndex> {
        use BorrowNode::*;
        use Bytecode::*;
        use Operation::*;
        match self {
            Assign(_, dest, ..)
            | Load(_, dest, ..)
            | Call(_, _, WriteBack(LocalRoot(dest)), ..)
            | Call(_, _, WriteBack(Reference(dest)), ..) => vec![*dest],
            Call(_, _, WriteRef, srcs) => vec![srcs[0]],
            Call(_, dests, ..) => dests.clone(),
            _ => vec![],
        }
    }
}

// =================================================================================================
// Formatting

impl Bytecode {
    /// Creates a format object for a bytecode in context of a function target.
    pub fn display<'env>(
        &'env self,
        func_target: &'env FunctionTarget<'env>,
    ) -> BytecodeDisplay<'env> {
        BytecodeDisplay {
            bytecode: self,
            func_target,
        }
    }
}

/// A display object for a bytecode.
pub struct BytecodeDisplay<'env> {
    bytecode: &'env Bytecode,
    func_target: &'env FunctionTarget<'env>,
}

impl<'env> fmt::Display for BytecodeDisplay<'env> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use Bytecode::*;
        match &self.bytecode {
            SpecBlock(id, _) => {
                // For now, use source location to print spec block. We currently do not have a
                // pretty printer for spec expressions and conditions. We may need to add one as
                // we start to generate spec blocks during transformations.
                let loc = self.func_target.get_bytecode_loc(*id);
                write!(f, "spec at {}..{}", loc.span().start(), loc.span().end())?;
            }
            Assign(_, dst, src, AssignKind::Copy) => {
                write!(f, "{} := copy({})", self.lstr(*dst), self.lstr(*src))?
            }
            Assign(_, dst, src, AssignKind::Move) => {
                write!(f, "{} := move({})", self.lstr(*dst), self.lstr(*src))?
            }
            Assign(_, dst, src, AssignKind::Store) => {
                write!(f, "{} := {}", self.lstr(*dst), self.lstr(*src))?
            }
            Call(_, dsts, oper, args) => {
                if !dsts.is_empty() {
                    self.fmt_locals(f, dsts, false)?;
                    write!(f, " := ")?;
                }
                write!(f, "{}", oper.display(self.func_target))?;
                self.fmt_locals(f, args, true)?;
            }
            Ret(_, srcs) => {
                write!(f, "return ")?;
                self.fmt_locals(f, srcs, false)?;
            }
            Load(_, dst, cons) => {
                write!(f, "{} := {}", self.lstr(*dst), cons)?;
            }
            Branch(_, then_label, else_label, src) => {
                write!(
                    f,
                    "if ({}) goto L{} else goto L{}",
                    self.lstr(*src),
                    then_label.as_usize(),
                    else_label.as_usize()
                )?;
            }
            Jump(_, label) => {
                write!(f, "goto L{}", label.as_usize())?;
            }
            Label(_, label) => {
                write!(f, "L{}:", label.as_usize(),)?;
            }
            Abort(_, src) => {
                write!(f, "abort({})", self.lstr(*src))?;
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
}

impl Operation {
    /// Creates a format object for a bytecode in context of a function target.
    pub fn display<'env>(
        &'env self,
        func_target: &'env FunctionTarget<'env>,
    ) -> OperationDisplay<'env> {
        OperationDisplay {
            oper: self,
            func_target,
        }
    }
}

/// A display object for an operation.
pub struct OperationDisplay<'env> {
    oper: &'env Operation,
    func_target: &'env FunctionTarget<'env>,
}

impl<'env> fmt::Display for OperationDisplay<'env> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use Operation::*;
        match self.oper {
            // User function
            Function(mid, fid, targs) => {
                let func_env = self
                    .func_target
                    .global_env()
                    .get_module(*mid)
                    .into_function(*fid);
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
            }

            // Pack/Unpack
            Pack(mid, sid, targs) => {
                write!(f, "pack {}", self.struct_str(*mid, *sid, targs))?;
            }
            Unpack(mid, sid, targs) => {
                write!(f, "unpack {}", self.struct_str(*mid, *sid, targs))?;
            }

            // Borrow
            BorrowLoc => {
                write!(f, "borrow_local")?;
            }
            BorrowField(mid, sid, targs, offset) => {
                write!(f, "borrow_field<{}>", self.struct_str(*mid, *sid, targs))?;
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
            BorrowGlobal(mid, sid, targs) => {
                write!(f, "borrow_global<{}>", self.struct_str(*mid, *sid, targs))?;
            }
            GetField(mid, sid, targs, offset) => {
                write!(f, "get_field<{}>", self.struct_str(*mid, *sid, targs))?;
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
            GetGlobal(mid, sid, targs) => {
                write!(f, "get_global<{}>", self.struct_str(*mid, *sid, targs))?;
            }

            // Resources
            MoveTo(mid, sid, targs) => {
                write!(f, "move_to<{}>", self.struct_str(*mid, *sid, targs))?;
            }
            MoveFrom(mid, sid, targs) => {
                write!(f, "move_from<{}>", self.struct_str(*mid, *sid, targs))?;
            }
            Exists(mid, sid, targs) => {
                write!(f, "exists<{}>", self.struct_str(*mid, *sid, targs))?;
            }

            // Builtins
            Destroy => {
                write!(f, "destroy")?;
            }
            ReadRef => {
                write!(f, "read_ref")?;
            }
            WriteRef => {
                write!(f, "write_ref")?;
            }
            FreezeRef => {
                write!(f, "freeze_ref")?;
            }

            // Memory model
            UnpackRef => {
                write!(f, "unpack_ref")?;
            }
            PackRef => {
                write!(f, "pack_ref")?;
            }
            PackRefDeep => {
                write!(f, "pack_ref_deep")?;
            }
            UnpackRefDeep => {
                write!(f, "unpack_ref_deep")?;
            }
            WriteBack(node) => write!(f, "write_back[{}]", node.display(self.func_target))?,
            Splice(map) => write!(
                f,
                "splice[{}]",
                map.iter()
                    .map(|(idx, local)| format!(
                        "{} -> {}",
                        idx,
                        self.func_target
                            .symbol_pool()
                            .string(self.func_target.get_local_name(*local))
                    ))
                    .join(", ")
            )?,

            // Unary
            CastU8 => write!(f, "(u8)")?,
            CastU64 => write!(f, "(u64)")?,
            CastU128 => write!(f, "(u128)")?,
            Not => write!(f, "!")?,

            // Binary
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

impl<'env> OperationDisplay<'env> {
    fn fmt_type_args(&self, f: &mut Formatter<'_>, targs: &[Type]) -> fmt::Result {
        if !targs.is_empty() {
            let tctx = TypeDisplayContext::WithEnv {
                env: self.func_target.global_env(),
                type_param_names: None,
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
            type_param_names: None,
        };
        format!("{}", ty.display(&tctx))
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
        }
        Ok(())
    }
}
