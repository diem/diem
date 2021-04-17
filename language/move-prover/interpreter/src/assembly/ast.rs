// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

use bytecode::stackless_bytecode::{AbortAction, Constant, Label};
use move_model::ast::{Exp, MemoryLabel, TempIndex};

use crate::{
    assembly::{
        display::{AstDebug, AstWriter},
        ty::{BaseType, StructInstantiation, Type},
    },
    shared::ident::FunctionIdent,
};

//**************************************************************************************************
// AST
//**************************************************************************************************

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Program {
    pub functions: BTreeMap<FunctionIdent, Function>,
    pub global_slots: BTreeMap<MemoryLabel, GlobalSlot>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Function {
    pub local_slots: Vec<LocalSlot>,
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LocalSlot {
    pub ty: Type,
    pub is_arg: bool,
    pub name: String,
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct GlobalSlot {
    pub insts: BTreeSet<StructInstantiation>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Instruction {
    // assignment
    Copy {
        dst: TempIndex,
        src: TempIndex,
    },
    Move {
        dst: TempIndex,
        src: TempIndex,
    },
    Load {
        dst: TempIndex,
        val: Constant,
    },
    // function
    Call {
        ident: FunctionIdent,
        ty_args: Vec<BaseType>,
        args: Vec<TempIndex>,
        rets: Vec<TempIndex>,
        on_abort: Option<AbortAction>,
    },
    // struct
    Pack {
        inst: StructInstantiation,
        fields: Vec<TempIndex>,
        packed: TempIndex,
    },
    Unpack {
        inst: StructInstantiation,
        fields: Vec<TempIndex>,
        packed: TempIndex,
    },
    GetField {
        inst: StructInstantiation,
        field: TempIndex,
        packed: TempIndex,
    },
    BorrowField {
        inst: StructInstantiation,
        field: TempIndex,
        packed: TempIndex,
    },
    MoveTo {
        inst: StructInstantiation,
        addr: TempIndex,
        src: TempIndex,
        on_abort: Option<AbortAction>,
    },
    MoveFrom {
        inst: StructInstantiation,
        addr: TempIndex,
        dst: TempIndex,
        on_abort: Option<AbortAction>,
    },
    GetGlobal {
        inst: StructInstantiation,
        addr: TempIndex,
        dst: TempIndex,
        on_abort: Option<AbortAction>,
    },
    BorrowGlobal {
        inst: StructInstantiation,
        addr: TempIndex,
        dst: TempIndex,
        on_abort: Option<AbortAction>,
    },
    ExistsGlobal {
        inst: StructInstantiation,
        addr: TempIndex,
        res: TempIndex,
    },
    // scope
    StructScopeInit {
        val: TempIndex,
    },
    StructScopeFini {
        val: TempIndex,
    },
    OpaqueCallInit {
        ident: FunctionIdent,
        ty_args: Vec<BaseType>,
    },
    OpaqueCallFini {
        ident: FunctionIdent,
        ty_args: Vec<BaseType>,
    },
    // memory
    WriteBackGlobalStruct {
        inst: StructInstantiation,
        src: TempIndex,
    },
    WriteBackGlobalField {
        inst: StructInstantiation,
        field_num: usize,
        src: TempIndex,
    },
    WriteBackLocalValue {
        src: TempIndex,
        dst: TempIndex,
    },
    WriteBackLocalField {
        inst: StructInstantiation,
        field_num: usize,
        src: TempIndex,
        dst: TempIndex,
    },
    WriteBackLocalVecElement {
        src: TempIndex,
        dst: TempIndex,
    },
    WriteBackRefValue {
        src: TempIndex,
        dst: TempIndex,
    },
    WriteBackRefField {
        inst: StructInstantiation,
        field_num: usize,
        src: TempIndex,
        dst: TempIndex,
    },
    WriteBackRefVecElement {
        src: TempIndex,
        dst: TempIndex,
    },
    // reference
    BorrowLocal {
        src: TempIndex,
        dst: TempIndex,
    },
    ReadRef {
        src: TempIndex,
        dst: TempIndex,
    },
    WriteRef {
        src: TempIndex,
        dst: TempIndex,
    },
    FreezeRef {
        src: TempIndex,
        dst: TempIndex,
    },
    // built-in
    Destroy {
        index: TempIndex,
    },
    HavocValue {
        index: TempIndex,
    },
    HavocDeref {
        index: TempIndex,
    },
    HavocMutation {
        index: TempIndex,
    },
    // cast
    CastU8 {
        src: TempIndex,
        dst: TempIndex,
        on_abort: Option<AbortAction>,
    },
    CastU64 {
        src: TempIndex,
        dst: TempIndex,
        on_abort: Option<AbortAction>,
    },
    CastU128 {
        src: TempIndex,
        dst: TempIndex,
        on_abort: Option<AbortAction>,
    },
    // unary
    Not {
        src: TempIndex,
        dst: TempIndex,
    },
    // binary (arithmetic)
    Add {
        lhs: TempIndex,
        rhs: TempIndex,
        dst: TempIndex,
        on_abort: Option<AbortAction>,
    },
    Sub {
        lhs: TempIndex,
        rhs: TempIndex,
        dst: TempIndex,
        on_abort: Option<AbortAction>,
    },
    Mul {
        lhs: TempIndex,
        rhs: TempIndex,
        dst: TempIndex,
        on_abort: Option<AbortAction>,
    },
    Div {
        lhs: TempIndex,
        rhs: TempIndex,
        dst: TempIndex,
        on_abort: Option<AbortAction>,
    },
    Mod {
        lhs: TempIndex,
        rhs: TempIndex,
        dst: TempIndex,
        on_abort: Option<AbortAction>,
    },
    // binary (bitwise)
    BitAnd {
        lhs: TempIndex,
        rhs: TempIndex,
        dst: TempIndex,
    },
    BitOr {
        lhs: TempIndex,
        rhs: TempIndex,
        dst: TempIndex,
    },
    BitXor {
        lhs: TempIndex,
        rhs: TempIndex,
        dst: TempIndex,
    },
    BitShl {
        lhs: TempIndex,
        rhs: TempIndex,
        dst: TempIndex,
    },
    BitShr {
        lhs: TempIndex,
        rhs: TempIndex,
        dst: TempIndex,
    },
    // binary (comparison)
    Lt {
        lhs: TempIndex,
        rhs: TempIndex,
        dst: TempIndex,
    },
    Le {
        lhs: TempIndex,
        rhs: TempIndex,
        dst: TempIndex,
    },
    Ge {
        lhs: TempIndex,
        rhs: TempIndex,
        dst: TempIndex,
    },
    Gt {
        lhs: TempIndex,
        rhs: TempIndex,
        dst: TempIndex,
    },
    // binary (equality)
    Eq {
        lhs: TempIndex,
        rhs: TempIndex,
        dst: TempIndex,
    },
    Neq {
        lhs: TempIndex,
        rhs: TempIndex,
        dst: TempIndex,
    },
    // binary (boolean)
    And {
        lhs: TempIndex,
        rhs: TempIndex,
        dst: TempIndex,
    },
    Or {
        lhs: TempIndex,
        rhs: TempIndex,
        dst: TempIndex,
    },
    // information
    SaveGlobal {
        label: MemoryLabel,
        inst: StructInstantiation,
    },
    // control flow
    Label {
        label: Label,
    },
    Jump {
        label: Label,
    },
    Stop {
        label: Label,
    },
    Branch {
        cond: TempIndex,
        then_label: Label,
        else_label: Label,
    },
    // termination
    Abort {
        index: TempIndex,
    },
    Return {
        tuple: Vec<TempIndex>,
    },
    // expression
    Assume {
        exp: Exp,
    },
    Assert {
        exp: Exp,
    },
    // nop
    Nop,
}

//**************************************************************************************************
// Display
//**************************************************************************************************

struct InstructionDisplay<'a> {
    function: &'a Function,
    instruction: &'a Instruction,
}

impl fmt::Display for InstructionDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // utility function pointers
        let util_local_name =
            |index: &TempIndex| self.function.local_slots.get(*index).unwrap().name.as_str();
        let util_local_names = |indices: &[TempIndex]| {
            indices
                .iter()
                .map(|index| util_local_name(index))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let util_type_args = |ty_args: &[BaseType]| {
            ty_args
                .iter()
                .map(|ty| ty.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        };
        let util_show_abort_handler =
            |f: &mut fmt::Formatter, on_abort: &Option<AbortAction>| match on_abort {
                None => write!(f, "abort"),
                Some(action) => write!(
                    f,
                    "store {}, goto L{}",
                    self.function.local_slots.get(action.1).unwrap().name,
                    action.0.as_usize()
                ),
            };

        match self.instruction {
            Instruction::Move { dst, src } => {
                write!(
                    f,
                    "  {} := move({})",
                    util_local_name(dst),
                    util_local_name(src),
                )
            }
            Instruction::Copy { dst, src } => {
                write!(
                    f,
                    "  {} := copy({})",
                    util_local_name(dst),
                    util_local_name(src),
                )
            }
            Instruction::Load { dst, val } => {
                write!(f, "  {} := load({:?})", util_local_name(dst), val)
            }
            Instruction::Call {
                ident,
                ty_args,
                args,
                rets,
                on_abort,
            } => {
                write!(
                    f,
                    "  {} := call {}<{}>({}), on_error: ",
                    util_local_names(rets),
                    ident,
                    util_type_args(ty_args),
                    util_local_names(args)
                )?;
                util_show_abort_handler(f, on_abort)
            }
            Instruction::Pack {
                inst,
                fields,
                packed,
            } => {
                write!(
                    f,
                    "  {} := pack<{}>({})",
                    util_local_name(packed),
                    inst,
                    util_local_names(fields)
                )
            }
            Instruction::Unpack {
                inst,
                fields,
                packed,
            } => {
                write!(
                    f,
                    "  {} := unpack<{}>({})",
                    util_local_names(fields),
                    inst,
                    util_local_name(packed)
                )
            }
            Instruction::GetField {
                inst,
                field,
                packed,
            } => {
                write!(
                    f,
                    "  {} := get_field<{}>({})",
                    util_local_name(field),
                    inst,
                    util_local_name(packed)
                )
            }
            Instruction::BorrowField {
                inst,
                field,
                packed,
            } => {
                write!(
                    f,
                    "  {} := borrow_field<{}>({})",
                    util_local_name(field),
                    inst,
                    util_local_name(packed)
                )
            }
            Instruction::MoveTo {
                inst,
                addr,
                src,
                on_abort,
            } => {
                write!(
                    f,
                    "  *{} <- move_to<{}>({})",
                    util_local_name(addr),
                    inst,
                    util_local_name(src),
                )?;
                util_show_abort_handler(f, on_abort)
            }
            Instruction::MoveFrom {
                inst,
                addr,
                dst,
                on_abort,
            } => {
                write!(
                    f,
                    "  {} := move_from<{}>({})",
                    util_local_name(dst),
                    inst,
                    util_local_name(addr),
                )?;
                util_show_abort_handler(f, on_abort)
            }
            Instruction::GetGlobal {
                inst,
                addr,
                dst,
                on_abort,
            } => {
                write!(
                    f,
                    "  {} := get_global<{}>({})",
                    util_local_name(dst),
                    inst,
                    util_local_name(addr),
                )?;
                util_show_abort_handler(f, on_abort)
            }
            Instruction::BorrowGlobal {
                inst,
                addr,
                dst,
                on_abort,
            } => {
                write!(
                    f,
                    "  {} := borrow_global<{}>({})",
                    util_local_name(dst),
                    inst,
                    util_local_name(addr),
                )?;
                util_show_abort_handler(f, on_abort)
            }
            Instruction::ExistsGlobal { inst, addr, res } => {
                write!(
                    f,
                    "  {} := exists<{}>({})",
                    util_local_name(res),
                    inst,
                    util_local_name(addr),
                )
            }
            Instruction::StructScopeInit { val } => {
                write!(f, "  struct_scope_init({})", util_local_name(val))
            }
            Instruction::StructScopeFini { val } => {
                write!(f, "  struct_scope_fini({})", util_local_name(val))
            }
            Instruction::OpaqueCallInit { ident, ty_args } => {
                write!(
                    f,
                    "  opaque_call_init {}<{}>",
                    ident,
                    util_type_args(ty_args)
                )
            }
            Instruction::OpaqueCallFini { ident, ty_args } => {
                write!(
                    f,
                    "  opaque_call_fini {}<{}>",
                    ident,
                    util_type_args(ty_args)
                )
            }
            Instruction::WriteBackGlobalStruct { inst, src } => {
                write!(
                    f,
                    "  write_back_global_struct<{}>({})",
                    inst,
                    util_local_name(src)
                )
            }
            Instruction::WriteBackGlobalField {
                inst,
                field_num,
                src,
            } => {
                write!(
                    f,
                    "  write_back_global_field<{}>[{}]({})",
                    inst,
                    field_num,
                    util_local_name(src)
                )
            }
            Instruction::WriteBackLocalValue { src, dst } => {
                write!(
                    f,
                    "  {} <- write_back_local_value({})",
                    util_local_name(dst),
                    util_local_name(src)
                )
            }
            Instruction::WriteBackLocalField {
                inst,
                field_num,
                src,
                dst,
            } => {
                write!(
                    f,
                    "  {} <- write_back_local_field<{}>[{}]({})",
                    util_local_name(dst),
                    inst,
                    field_num,
                    util_local_name(src)
                )
            }
            Instruction::WriteBackLocalVecElement { src, dst } => {
                write!(
                    f,
                    "  {} <- write_back_local_vec_element({})",
                    util_local_name(dst),
                    util_local_name(src)
                )
            }
            Instruction::WriteBackRefValue { src, dst } => {
                write!(
                    f,
                    "  {} <- write_back_ref_value({})",
                    util_local_name(dst),
                    util_local_name(src)
                )
            }
            Instruction::WriteBackRefField {
                inst,
                field_num,
                src,
                dst,
            } => {
                write!(
                    f,
                    "  {} <- write_back_ref_field<{}>[{}]({})",
                    util_local_name(dst),
                    inst,
                    field_num,
                    util_local_name(src)
                )
            }
            Instruction::WriteBackRefVecElement { src, dst } => {
                write!(
                    f,
                    "  {} <- write_back_ref_vec_element({})",
                    util_local_name(dst),
                    util_local_name(src)
                )
            }
            Instruction::BorrowLocal { src, dst } => {
                write!(
                    f,
                    "  {} := borrow_local({})",
                    util_local_name(dst),
                    util_local_name(src)
                )
            }
            Instruction::ReadRef { src, dst } => {
                write!(
                    f,
                    "  {} := read_ref({})",
                    util_local_name(dst),
                    util_local_name(src)
                )
            }
            Instruction::WriteRef { src, dst } => {
                write!(
                    f,
                    "  {} := write_ref({})",
                    util_local_name(dst),
                    util_local_name(src)
                )
            }
            Instruction::FreezeRef { src, dst } => {
                write!(
                    f,
                    "  {} := freeze_ref({})",
                    util_local_name(dst),
                    util_local_name(src)
                )
            }
            Instruction::Destroy { index } => {
                write!(f, "  destroy({})", util_local_name(index))
            }
            Instruction::HavocValue { index } => {
                write!(f, "  havoc_value({})", util_local_name(index))
            }
            Instruction::HavocDeref { index } => {
                write!(f, "  havoc_deref({})", util_local_name(index))
            }
            Instruction::HavocMutation { index } => {
                write!(f, "  havoc_mutation({})", util_local_name(index))
            }
            Instruction::CastU8 { src, dst, on_abort } => {
                write!(
                    f,
                    "  {} := cast<u8>({})",
                    util_local_name(dst),
                    util_local_name(src)
                )?;
                util_show_abort_handler(f, on_abort)
            }
            Instruction::CastU64 { src, dst, on_abort } => {
                write!(
                    f,
                    "  {} := cast<u64>({})",
                    util_local_name(dst),
                    util_local_name(src)
                )?;
                util_show_abort_handler(f, on_abort)
            }
            Instruction::CastU128 { src, dst, on_abort } => {
                write!(
                    f,
                    "  {} := cast<u128>({})",
                    util_local_name(dst),
                    util_local_name(src)
                )?;
                util_show_abort_handler(f, on_abort)
            }
            Instruction::Not { src, dst } => {
                write!(f, "  {} := !{}", util_local_name(dst), util_local_name(src))
            }
            Instruction::Add {
                lhs,
                rhs,
                dst,
                on_abort,
            } => {
                write!(
                    f,
                    "  {} := {} + {}",
                    util_local_name(dst),
                    util_local_name(lhs),
                    util_local_name(rhs),
                )?;
                util_show_abort_handler(f, on_abort)
            }
            Instruction::Sub {
                lhs,
                rhs,
                dst,
                on_abort,
            } => {
                write!(
                    f,
                    "  {} := {} - {}",
                    util_local_name(dst),
                    util_local_name(lhs),
                    util_local_name(rhs),
                )?;
                util_show_abort_handler(f, on_abort)
            }
            Instruction::Mul {
                lhs,
                rhs,
                dst,
                on_abort,
            } => {
                write!(
                    f,
                    "  {} := {} * {}",
                    util_local_name(dst),
                    util_local_name(lhs),
                    util_local_name(rhs),
                )?;
                util_show_abort_handler(f, on_abort)
            }
            Instruction::Div {
                lhs,
                rhs,
                dst,
                on_abort,
            } => {
                write!(
                    f,
                    "  {} := {} / {}",
                    util_local_name(dst),
                    util_local_name(lhs),
                    util_local_name(rhs),
                )?;
                util_show_abort_handler(f, on_abort)
            }
            Instruction::Mod {
                lhs,
                rhs,
                dst,
                on_abort,
            } => {
                write!(
                    f,
                    "  {} := {} % {}",
                    util_local_name(dst),
                    util_local_name(lhs),
                    util_local_name(rhs),
                )?;
                util_show_abort_handler(f, on_abort)
            }
            Instruction::BitAnd { lhs, rhs, dst } => {
                write!(
                    f,
                    "  {} := {} & {}",
                    util_local_name(dst),
                    util_local_name(lhs),
                    util_local_name(rhs),
                )
            }
            Instruction::BitOr { lhs, rhs, dst } => {
                write!(
                    f,
                    "  {} := {} | {}",
                    util_local_name(dst),
                    util_local_name(lhs),
                    util_local_name(rhs),
                )
            }
            Instruction::BitXor { lhs, rhs, dst } => {
                write!(
                    f,
                    "  {} := {} ^ {}",
                    util_local_name(dst),
                    util_local_name(lhs),
                    util_local_name(rhs),
                )
            }
            Instruction::BitShl { lhs, rhs, dst } => {
                write!(
                    f,
                    "  {} := {} << {}",
                    util_local_name(dst),
                    util_local_name(lhs),
                    util_local_name(rhs),
                )
            }
            Instruction::BitShr { lhs, rhs, dst } => {
                write!(
                    f,
                    "  {} := {} >> {}",
                    util_local_name(dst),
                    util_local_name(lhs),
                    util_local_name(rhs),
                )
            }
            Instruction::Lt { lhs, rhs, dst } => {
                write!(
                    f,
                    "  {} := {} < {}",
                    util_local_name(dst),
                    util_local_name(lhs),
                    util_local_name(rhs),
                )
            }
            Instruction::Le { lhs, rhs, dst } => {
                write!(
                    f,
                    "  {} := {} <= {}",
                    util_local_name(dst),
                    util_local_name(lhs),
                    util_local_name(rhs),
                )
            }
            Instruction::Ge { lhs, rhs, dst } => {
                write!(
                    f,
                    "  {} := {} >= {}",
                    util_local_name(dst),
                    util_local_name(lhs),
                    util_local_name(rhs),
                )
            }
            Instruction::Gt { lhs, rhs, dst } => {
                write!(
                    f,
                    "  {} := {} > {}",
                    util_local_name(dst),
                    util_local_name(lhs),
                    util_local_name(rhs),
                )
            }
            Instruction::Eq { lhs, rhs, dst } => {
                write!(
                    f,
                    "  {} := {} == {}",
                    util_local_name(dst),
                    util_local_name(lhs),
                    util_local_name(rhs),
                )
            }
            Instruction::Neq { lhs, rhs, dst } => {
                write!(
                    f,
                    "  {} := {} != {}",
                    util_local_name(dst),
                    util_local_name(lhs),
                    util_local_name(rhs),
                )
            }
            Instruction::And { lhs, rhs, dst } => {
                write!(
                    f,
                    "  {} := {} && {}",
                    util_local_name(dst),
                    util_local_name(lhs),
                    util_local_name(rhs),
                )
            }
            Instruction::Or { lhs, rhs, dst } => {
                write!(
                    f,
                    "  {} := {} || {}",
                    util_local_name(dst),
                    util_local_name(lhs),
                    util_local_name(rhs),
                )
            }
            Instruction::SaveGlobal { label, inst } => {
                write!(f, "  {} <- save_global<{}>()", label.to_string(), inst)
            }
            Instruction::Label { label } => {
                write!(f, "L{}:", label.as_usize())
            }
            Instruction::Jump { label } => {
                write!(f, "  goto L{}", label.as_usize())
            }
            Instruction::Stop { label } => {
                write!(f, "  stop L{}", label.as_usize())
            }
            Instruction::Branch {
                cond,
                then_label,
                else_label,
            } => {
                write!(
                    f,
                    "  {}? L{} : L{}",
                    util_local_name(cond),
                    then_label.as_usize(),
                    else_label.as_usize()
                )
            }
            Instruction::Abort { index } => {
                write!(f, "  abort({})", util_local_name(index))
            }
            Instruction::Return { tuple } => {
                write!(f, "  return({})", util_local_names(tuple))
            }
            // TODO (mengxu) display expressions in a better way
            Instruction::Assume { exp } => {
                write!(f, "  assume {:?}", exp)
            }
            Instruction::Assert { exp } => {
                write!(f, "  assert {:?}", exp)
            }
            Instruction::Nop => {
                write!(f, "  nop")
            }
        }
    }
}

//**************************************************************************************************
// Debug
//**************************************************************************************************

impl AstDebug for Program {
    fn ast_debug(&self, w: &mut AstWriter) {
        for (func_ident, func_body) in &self.functions {
            w.write(&format!("fn {} ", func_ident));
            w.block(|w| func_body.ast_debug(w));
        }
    }
}

impl AstDebug for Function {
    fn ast_debug(&self, w: &mut AstWriter) {
        // local slots
        w.numbered_list(self.local_slots.iter(), |w, num, slot| {
            let mark = if slot.is_arg { "*" } else { " " };
            w.writeln(&format!("{}{} {}: {}", mark, num, slot.name, slot.ty));
        });
        if !self.local_slots.is_empty() {
            w.new_line();
        }

        // instructions
        w.numbered_list(self.instructions.iter(), |w, num, inst| {
            w.writeln(&format!(
                "{}: {}",
                num,
                InstructionDisplay {
                    function: self,
                    instruction: inst
                }
            ));
        });
    }
}
