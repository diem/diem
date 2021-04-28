// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

// TODO (mengxu) remove this when the expr module is in good shape
#![allow(dead_code)]

use num::BigUint;
use std::{
    collections::BTreeMap,
    fmt::{Debug, Display},
};

use move_core_types::account_address::AccountAddress;

use crate::assembly::ty::{StructInstantiation, Type};

//**************************************************************************************************
// Traits
//**************************************************************************************************

pub trait Expr: Debug + Display + Clone + PartialEq + Eq + PartialOrd + Ord {
    fn mk_eq<B: BoolExpr>(lhs: &Self, rhs: &Self) -> B;
    fn mk_ne<B: BoolExpr>(lhs: &Self, rhs: &Self) -> B;
}

pub trait IntExpr: Expr {
    fn mk_concrete(val: &BigUint) -> Self;
    fn mk_symbolic(sym: &str) -> Self;

    fn mk_add(lhs: &Self, rhs: &Self) -> Self;
    fn mk_sub(lhs: &Self, rhs: &Self) -> Self;
    fn mk_mul(lhs: &Self, rhs: &Self) -> Self;
    fn mk_div(lhs: &Self, rhs: &Self) -> Self;
    fn mk_mod(lhs: &Self, rhs: &Self) -> Self;

    fn mk_bitand(lhs: &Self, rhs: &Self) -> Self;
    fn mk_bitor(lhs: &Self, rhs: &Self) -> Self;
    fn mk_bitxor(lhs: &Self, rhs: &Self) -> Self;
    fn mk_bitshl(lhs: &Self, rhs: &Self) -> Self;
    fn mk_bitshr(lhs: &Self, rhs: &Self) -> Self;

    fn mk_lt<B: BoolExpr>(lhs: &Self, rhs: &Self) -> B;
    fn mk_le<B: BoolExpr>(lhs: &Self, rhs: &Self) -> B;
    fn mk_ge<B: BoolExpr>(lhs: &Self, rhs: &Self) -> B;
    fn mk_gt<B: BoolExpr>(lhs: &Self, rhs: &Self) -> B;

    fn mk_cast_u8(exp: &Self) -> Self;
    fn mk_cast_u64(exp: &Self) -> Self;
    fn mk_cast_u128(exp: &Self) -> Self;
}

pub trait BoolExpr: Expr {
    fn mk_concrete(val: bool) -> Self;
    fn mk_symbolic(sym: &str) -> Self;

    fn mk_and(lhs: &Self, rhs: &Self) -> Self;
    fn mk_or(lhs: &Self, rhs: &Self) -> Self;
    fn mk_not(exp: &Self) -> Self;
    fn mk_implies(exp: &Self) -> Self;
}

pub trait AddressExpr: Expr {
    fn mk_concrete(val: AccountAddress) -> Self;
    fn mk_symbolic(sym: &str) -> Self;
}

pub trait VectorExpr: Expr {
    fn mk_concrete<B: BoolExpr, I: IntExpr, A: AddressExpr, S: StructExpr>(
        val: &[BaseExpr<B, I, A, Self, S>],
    ) -> Self;
    fn mk_symbolic(sym: String) -> Self;

    fn mk_empty() -> Self;
    fn mk_single<B: BoolExpr, I: IntExpr, A: AddressExpr, S: StructExpr>(
        exp: &BaseExpr<B, I, A, Self, S>,
    ) -> Self;

    fn mk_concat(lhs: &Self, rhs: &Self) -> Self;

    fn mk_access_element<B: BoolExpr, I: IntExpr, A: AddressExpr, S: StructExpr>(
        vec: &Self,
        idx: &I,
    ) -> BaseExpr<B, I, A, Self, S>;

    fn mk_update_element<B: BoolExpr, I: IntExpr, A: AddressExpr, S: StructExpr>(
        vec: &Self,
        idx: &I,
        exp: &BaseExpr<B, I, A, Self, S>,
    ) -> Self;

    fn mk_length<I: IntExpr>(exp: &Self) -> I;
    fn mk_contains<B: BoolExpr, I: IntExpr, A: AddressExpr, S: StructExpr>(
        vec: &Self,
        exp: &BaseExpr<B, I, A, Self, S>,
    ) -> B;
    fn mk_index_of<B: BoolExpr, I: IntExpr, A: AddressExpr, S: StructExpr>(
        vec: &Self,
        exp: &BaseExpr<B, I, A, Self, S>,
    ) -> I;
}

pub trait StructExpr: Expr {
    fn mk_pack<B: BoolExpr, I: IntExpr, A: AddressExpr, V: VectorExpr>(
        fields: &[&BaseExpr<B, I, A, V, Self>],
        layout: &StructInstantiation,
    ) -> Self;
    fn mk_unpack<B: BoolExpr, I: IntExpr, A: AddressExpr, V: VectorExpr>(
        object: &Self,
        layout: &StructInstantiation,
    ) -> Vec<BaseExpr<B, I, A, V, Self>>;

    fn mk_access_field<B: BoolExpr, I: IntExpr, A: AddressExpr, V: VectorExpr>(
        object: &Self,
        layout: &StructInstantiation,
        field_num: usize,
    ) -> BaseExpr<B, I, A, V, Self>;
    fn mk_update_field<B: BoolExpr, I: IntExpr, A: AddressExpr, V: VectorExpr>(
        object: &Self,
        layout: &StructInstantiation,
        field_num: usize,
        field_val: &BaseExpr<B, I, A, V, Self>,
    ) -> Self;
}

//**************************************************************************************************
// Expression
//**************************************************************************************************

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ExprId(usize);

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum BaseExpr<B: BoolExpr, I: IntExpr, A: AddressExpr, V: VectorExpr, S: StructExpr> {
    Bool(B),
    Int(I),
    Address(A),
    Vector(V),
    Struct(S),
    Argument,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct TypedExpr<B: BoolExpr, I: IntExpr, A: AddressExpr, V: VectorExpr, S: StructExpr> {
    id: ExprId,
    ty: Type,
    expr: BaseExpr<B, I, A, V, S>,
}

pub struct ExprManager<B: BoolExpr, I: IntExpr, A: AddressExpr, V: VectorExpr, S: StructExpr> {
    exprs: BTreeMap<ExprId, TypedExpr<B, I, A, V, S>>,
}
