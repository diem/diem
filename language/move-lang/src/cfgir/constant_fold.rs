// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::cfg::BlockCFG;
use crate::{
    expansion::ast::{Value, Value_},
    hlir::ast::{Command, Command_, Exp, ExpListItem, UnannotatedExp_},
    naming::ast::{BuiltinTypeName, BuiltinTypeName_},
    parser::ast::{BinOp, BinOp_, UnaryOp, UnaryOp_},
    shared::*,
};
use move_ir_types::location::*;
use std::convert::TryFrom;

/// returns true if anything changed
pub fn optimize(cfg: &mut BlockCFG) -> bool {
    let mut changed = false;
    for block in cfg.blocks_mut().values_mut() {
        for cmd in block {
            changed = optimize_cmd(cmd) || changed;
        }
    }
    changed
}

//**************************************************************************************************
// Scaffolding
//**************************************************************************************************

fn optimize_cmd(sp!(_, cmd_): &mut Command) -> bool {
    use Command_ as C;
    match cmd_ {
        C::Assign(_ls, e) => optimize_exp(e),
        C::Mutate(el, er) => {
            let c1 = optimize_exp(er);
            let c2 = optimize_exp(el);
            c1 || c2
        }
        C::Return(e) | C::Abort(e) | C::IgnoreAndPop { exp: e, .. } | C::JumpIf { cond: e, .. } => {
            optimize_exp(e)
        }

        C::Jump(_) => false,
        C::Break | C::Continue => panic!("ICE break/continue not translated to jumps"),
    }
}

fn optimize_exp(e: &mut Exp) -> bool {
    use UnannotatedExp_ as E;
    match &mut e.exp.value {
        //************************************
        // Pass through cases
        //************************************
        E::Unit { .. }
        | E::Value(_)
        | E::Constant(_)
        | E::UnresolvedError
        | E::Spec(_, _)
        | E::BorrowLocal(_, _)
        | E::Move { .. }
        | E::Copy { .. }
        | E::Unreachable => false,

        E::ModuleCall(mcall) => optimize_exp(&mut mcall.arguments),
        E::Builtin(_, e) | E::Freeze(e) | E::Dereference(e) | E::Borrow(_, e, _) => optimize_exp(e),

        E::Pack(_, _, fields) => fields
            .iter_mut()
            .map(|(_, _, e)| optimize_exp(e))
            .any(|changed| changed),

        E::ExpList(es) => es
            .iter_mut()
            .map(|item| optimize_exp_item(item))
            .any(|changed| changed),

        //************************************
        // Foldable cases
        //************************************
        e_ @ E::UnaryExp(_, _) => {
            let (op, er) = match e_ {
                E::UnaryExp(op, er) => (op, er),
                _ => unreachable!(),
            };
            let changed = optimize_exp(er);
            let v = match foldable_exp(er) {
                Some(v) => v,
                None => return changed,
            };
            match fold_unary_op(e.exp.loc, op, v) {
                Some(folded) => {
                    *e_ = folded;
                    true
                }
                None => changed,
            }
        }

        e_ @ E::BinopExp(_, _, _) => {
            let (e1, op, e2) = match e_ {
                E::BinopExp(e1, op, e2) => (e1, op, e2),
                _ => unreachable!(),
            };
            let changed1 = optimize_exp(e1);
            let changed2 = optimize_exp(e2);
            let changed = changed1 || changed2;
            let (v1, v2) = match (foldable_exp(e1), foldable_exp(e2)) {
                (Some(v1), Some(v2)) => (v1, v2),
                _ => return changed,
            };
            match fold_binary_op(e.exp.loc, op, v1, v2) {
                Some(folded) => {
                    *e_ = folded;
                    true
                }
                None => changed,
            }
        }

        e_ @ E::Cast(_, _) => {
            let (e, bt) = match e_ {
                E::Cast(e, bt) => (e, bt),
                _ => unreachable!(),
            };
            let changed = optimize_exp(e);
            let v = match foldable_exp(e) {
                Some(v) => v,
                None => return changed,
            };
            match fold_cast(e.exp.loc, bt, v) {
                Some(folded) => {
                    *e_ = folded;
                    true
                }
                None => changed,
            }
        }
    }
}

fn optimize_exp_item(item: &mut ExpListItem) -> bool {
    match item {
        ExpListItem::Single(e, _) | ExpListItem::Splat(_, e, _) => optimize_exp(e),
    }
}

//**************************************************************************************************
// Folding
//**************************************************************************************************

fn fold_unary_op(loc: Loc, sp!(_, op_): &UnaryOp, v: FoldableValue) -> Option<UnannotatedExp_> {
    use FoldableValue as FV;
    use UnaryOp_ as U;
    let folded = match (op_, v) {
        (U::Not, FV::Bool(b)) => FV::Bool(!b),
        (op_, v) => panic!("ICE unknown unary op. combo while folding: {} {:?}", op_, v),
    };
    Some(evalue_(loc, folded))
}

fn fold_binary_op(
    loc: Loc,
    sp!(_, op_): &BinOp,
    v1: FoldableValue,
    v2: FoldableValue,
) -> Option<UnannotatedExp_> {
    use BinOp_ as B;
    use FoldableValue as FV;
    let v = match (op_, v1, v2) {
        //************************************
        // Checked arith
        //************************************
        (B::Add, FV::U8(u1), FV::U8(u2)) => FV::U8(u1.checked_add(u2)?),
        (B::Add, FV::U64(u1), FV::U64(u2)) => FV::U64(u1.checked_add(u2)?),
        (B::Add, FV::U128(u1), FV::U128(u2)) => FV::U128(u1.checked_add(u2)?),

        (B::Sub, FV::U8(u1), FV::U8(u2)) => FV::U8(u1.checked_sub(u2)?),
        (B::Sub, FV::U64(u1), FV::U64(u2)) => FV::U64(u1.checked_sub(u2)?),
        (B::Sub, FV::U128(u1), FV::U128(u2)) => FV::U128(u1.checked_sub(u2)?),

        (B::Mul, FV::U8(u1), FV::U8(u2)) => FV::U8(u1.checked_mul(u2)?),
        (B::Mul, FV::U64(u1), FV::U64(u2)) => FV::U64(u1.checked_mul(u2)?),
        (B::Mul, FV::U128(u1), FV::U128(u2)) => FV::U128(u1.checked_mul(u2)?),

        (B::Mod, FV::U8(u1), FV::U8(u2)) => FV::U8(u1.checked_rem(u2)?),
        (B::Mod, FV::U64(u1), FV::U64(u2)) => FV::U64(u1.checked_rem(u2)?),
        (B::Mod, FV::U128(u1), FV::U128(u2)) => FV::U128(u1.checked_rem(u2)?),

        (B::Div, FV::U8(u1), FV::U8(u2)) => FV::U8(u1.checked_div(u2)?),
        (B::Div, FV::U64(u1), FV::U64(u2)) => FV::U64(u1.checked_div(u2)?),
        (B::Div, FV::U128(u1), FV::U128(u2)) => FV::U128(u1.checked_div(u2)?),

        (B::Shl, FV::U8(u1), FV::U8(u2)) => FV::U8(u1.checked_shl(u2 as u32)?),
        (B::Shl, FV::U64(u1), FV::U8(u2)) => FV::U64(u1.checked_shl(u2 as u32)?),
        (B::Shl, FV::U128(u1), FV::U8(u2)) => FV::U128(u1.checked_shl(u2 as u32)?),

        (B::Shr, FV::U8(u1), FV::U8(u2)) => FV::U8(u1.checked_shr(u2 as u32)?),
        (B::Shr, FV::U64(u1), FV::U8(u2)) => FV::U64(u1.checked_shr(u2 as u32)?),
        (B::Shr, FV::U128(u1), FV::U8(u2)) => FV::U128(u1.checked_shr(u2 as u32)?),

        //************************************
        // Pure arith
        //************************************
        (B::BitOr, FV::U8(u1), FV::U8(u2)) => FV::U8(u1 | u2),
        (B::BitOr, FV::U64(u1), FV::U64(u2)) => FV::U64(u1 | u2),
        (B::BitOr, FV::U128(u1), FV::U128(u2)) => FV::U128(u1 | u2),

        (B::BitAnd, FV::U8(u1), FV::U8(u2)) => FV::U8(u1 & u2),
        (B::BitAnd, FV::U64(u1), FV::U64(u2)) => FV::U64(u1 & u2),
        (B::BitAnd, FV::U128(u1), FV::U128(u2)) => FV::U128(u1 & u2),

        (B::Xor, FV::U8(u1), FV::U8(u2)) => FV::U8(u1 ^ u2),
        (B::Xor, FV::U64(u1), FV::U64(u2)) => FV::U64(u1 ^ u2),
        (B::Xor, FV::U128(u1), FV::U128(u2)) => FV::U128(u1 ^ u2),

        //************************************
        // Logical
        //************************************
        (B::And, FV::Bool(b1), FV::Bool(b2)) => FV::Bool(b1 && b2),
        (B::Or, FV::Bool(b1), FV::Bool(b2)) => FV::Bool(b1 || b2),

        //************************************
        // Comparisons
        //************************************
        (B::Lt, FV::U8(u1), FV::U8(u2)) => FV::Bool(u1 < u2),
        (B::Lt, FV::U64(u1), FV::U64(u2)) => FV::Bool(u1 < u2),
        (B::Lt, FV::U128(u1), FV::U128(u2)) => FV::Bool(u1 < u2),

        (B::Gt, FV::U8(u1), FV::U8(u2)) => FV::Bool(u1 > u2),
        (B::Gt, FV::U64(u1), FV::U64(u2)) => FV::Bool(u1 > u2),
        (B::Gt, FV::U128(u1), FV::U128(u2)) => FV::Bool(u1 > u2),

        (B::Le, FV::U8(u1), FV::U8(u2)) => FV::Bool(u1 <= u2),
        (B::Le, FV::U64(u1), FV::U64(u2)) => FV::Bool(u1 <= u2),
        (B::Le, FV::U128(u1), FV::U128(u2)) => FV::Bool(u1 <= u2),

        (B::Ge, FV::U8(u1), FV::U8(u2)) => FV::Bool(u1 >= u2),
        (B::Ge, FV::U64(u1), FV::U64(u2)) => FV::Bool(u1 >= u2),
        (B::Ge, FV::U128(u1), FV::U128(u2)) => FV::Bool(u1 >= u2),

        (B::Eq, v1, v2) => FV::Bool(v1 == v2),
        (B::Neq, v1, v2) => FV::Bool(v1 != v2),

        (op_, v1, v2) => panic!(
            "ICE unknown binary op. combo while folding: {:?} {} {:?}",
            v1, op_, v2
        ),
    };
    Some(evalue_(loc, v))
}

fn fold_cast(loc: Loc, sp!(_, bt_): &BuiltinTypeName, v: FoldableValue) -> Option<UnannotatedExp_> {
    use BuiltinTypeName_ as BT;
    use FoldableValue as FV;
    let cast = match (bt_, v) {
        (BT::U8, FV::U8(u)) => FV::U8(u),
        (BT::U8, FV::U64(u)) => FV::U8(u8::try_from(u).ok()?),
        (BT::U8, FV::U128(u)) => FV::U8(u8::try_from(u).ok()?),

        (BT::U64, FV::U8(u)) => FV::U64(u as u64),
        (BT::U64, FV::U64(u)) => FV::U64(u),
        (BT::U64, FV::U128(u)) => FV::U64(u64::try_from(u).ok()?),

        (BT::U128, FV::U8(u)) => FV::U128(u as u128),
        (BT::U128, FV::U64(u)) => FV::U128(u as u128),
        (BT::U128, FV::U128(u)) => FV::U128(u),

        (_, v) => panic!("ICE unexpected cast while folding: {:?} as {:?}", v, bt_),
    };
    Some(evalue_(loc, cast))
}

fn evalue_(loc: Loc, fv: FoldableValue) -> UnannotatedExp_ {
    use FoldableValue as FV;
    use UnannotatedExp_ as E;
    use Value_ as V;

    let v = match fv {
        FV::U8(u) => V::U8(u),
        FV::U64(u) => V::U64(u),
        FV::U128(u) => V::U128(u),
        FV::Bool(b) => V::Bool(b),
        FV::Address(a) => V::Address(a),
        FV::Bytearray(b) => V::Bytearray(b),
    };

    E::Value(sp(loc, v))
}

//**************************************************************************************************
// Foldable Value
//**************************************************************************************************

#[derive(Debug)]
enum FoldableValue {
    U8(u8),
    U64(u64),
    U128(u128),
    Bool(bool),
    Address(Address),
    Bytearray(Vec<u8>),
}

fn foldable_value(sp!(_, v_): &Value) -> Option<FoldableValue> {
    use FoldableValue as FV;
    use Value_ as V;
    Some(match v_ {
        V::U8(u) => FV::U8(*u),
        V::U64(u) => FV::U64(*u),
        V::U128(u) => FV::U128(*u),
        V::Bool(b) => FV::Bool(*b),
        V::Address(a) => FV::Address(*a),
        V::Bytearray(b) => FV::Bytearray(b.clone()),
    })
}

fn foldable_exp(e: &Exp) -> Option<FoldableValue> {
    use UnannotatedExp_ as E;
    match &e.exp.value {
        E::Value(v) => foldable_value(v),
        _ => None,
    }
}

impl PartialEq for FoldableValue {
    fn eq(&self, other: &FoldableValue) -> bool {
        match (self, other) {
            (FoldableValue::U8(x), FoldableValue::U8(y)) => x == y,
            (FoldableValue::U64(x), FoldableValue::U64(y)) => x == y,
            (FoldableValue::U128(x), FoldableValue::U128(y)) => x == y,
            (FoldableValue::Bool(x), FoldableValue::Bool(y)) => x == y,
            (FoldableValue::Address(x), FoldableValue::Address(y)) => x == y,
            (FoldableValue::Bytearray(x), FoldableValue::Bytearray(y)) => x == y,
            _ => panic!("ICE type checking failed. bad eq on FoldableValue"),
        }
    }
}
