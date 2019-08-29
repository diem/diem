// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    abstract_state::{AbstractState, AbstractValue, BorrowState},
    common::VMError,
};
use vm::{
    access::*,
    file_format::{Kind, SignatureToken, StructDefinitionIndex},
    views::{SignatureTokenView, StructDefinitionView, ViewInternals},
};

/// Determine whether the stack is at least of size `index`. If the optional `abstract_value`
/// argument is some `AbstractValue`, check whether the type at `index` is that abstract_value.
pub fn stack_has(
    state: &AbstractState,
    index: usize,
    abstract_value: Option<AbstractValue>,
) -> bool {
    match abstract_value {
        Some(abstract_value) => {
            index < state.stack_len() && state.stack_peek(index) == Some(abstract_value)
        }
        None => index < state.stack_len(),
    }
}

/// Determine whether two tokens on the stack have the same type
pub fn stack_has_polymorphic_eq(state: &AbstractState, index1: usize, index2: usize) -> bool {
    if stack_has(state, index2, None) {
        state.stack_peek(index1) == state.stack_peek(index2)
    } else {
        false
    }
}

/// Pop from the top of the stack.
pub fn stack_pop(state: &AbstractState) -> Result<AbstractState, VMError> {
    let mut state = state.clone();
    state.stack_pop()?;
    Ok(state)
}

/// Push given abstract_value to the top of the stack.
pub fn stack_push(
    state: &AbstractState,
    abstract_value: AbstractValue,
) -> Result<AbstractState, VMError> {
    let mut state = state.clone();
    state.stack_push(abstract_value);
    Ok(state)
}

/// Push to the top of the stack from the register.
pub fn stack_push_register(state: &AbstractState) -> Result<AbstractState, VMError> {
    let mut state = state.clone();
    state.stack_push_register()?;
    Ok(state)
}

/// Check whether the local at `index` exists
pub fn local_exists(state: &AbstractState, index: u8) -> bool {
    state.local_exists(index as usize)
}

/// Check whether the local at `index` is of the given availability
pub fn local_availability_is(state: &AbstractState, index: u8, availability: BorrowState) -> bool {
    state
        .local_availability_is(index as usize, availability)
        .unwrap_or_else(|_| false)
}

/// Check whether the local at `index` is of the given kind
pub fn local_kind_is(state: &AbstractState, index: u8, kind: Kind) -> bool {
    state
        .local_kind_is(index as usize, kind)
        .unwrap_or_else(|_| false)
}

/// Set the availability of local at `index`
pub fn local_set(
    state: &AbstractState,
    index: u8,
    availability: BorrowState,
) -> Result<AbstractState, VMError> {
    let mut state = state.clone();
    state.local_set(index as usize, availability)?;
    Ok(state)
}

/// Put copy of the local at `index` in register
pub fn local_take(state: &AbstractState, index: u8) -> Result<AbstractState, VMError> {
    let mut state = state.clone();
    state.local_take(index as usize)?;
    Ok(state)
}

/// Put reference to local at `index` in register
pub fn local_take_borrow(
    state: &AbstractState,
    index: u8,
    mutable: bool,
) -> Result<AbstractState, VMError> {
    let mut state = state.clone();
    state.local_take_borrow(index as usize, mutable)?;
    Ok(state)
}

/// Insert the register value into the locals at `index`
pub fn local_place(state: &AbstractState, index: u8) -> Result<AbstractState, VMError> {
    let mut state = state.clone();
    state.local_place(index as usize)?;
    Ok(state)
}

/// Determine whether a abstract_value on the stack and a abstract_value in the locals have the same
/// type
pub fn stack_local_polymorphic_eq(state: &AbstractState, index1: usize, index2: usize) -> bool {
    if stack_has(state, index1, None) {
        if let Some((abstract_value, _)) = state.local_get(index2) {
            return state.stack_peek(index1) == Some(abstract_value.clone());
        }
    }
    false
}

/// Determine whether the struct at the given index can be constructed from the values on
/// the stack.
pub fn stack_satisfies_struct_signature(
    state: &AbstractState,
    struct_index: StructDefinitionIndex,
) -> bool {
    let struct_def = state.module.struct_def_at(struct_index);
    let struct_def = StructDefinitionView::new(&state.module, struct_def);
    let field_token_views = struct_def
        .fields()
        .into_iter()
        .flatten()
        .map(|field| field.type_signature().token());
    let mut satisfied = true;
    for (i, token_view) in field_token_views.enumerate() {
        let abstract_value = AbstractValue {
            token: token_view.as_inner().clone(),
            kind: token_view.kind(&[]),
        };
        if !stack_has(state, i, Some(abstract_value)) {
            satisfied = false;
        }
    }
    satisfied
}

/// Pop the number of stack values required to construct the struct
/// at `struct_index`
pub fn stack_struct_popn(
    state: &AbstractState,
    struct_index: StructDefinitionIndex,
) -> Result<AbstractState, VMError> {
    let state_copy = state.clone();
    let mut state = state.clone();
    let struct_def = state_copy.module.struct_def_at(struct_index);
    let struct_def_view = StructDefinitionView::new(&state_copy.module, struct_def);
    let number_of_pops = struct_def_view.fields().into_iter().len();
    for _ in 0..number_of_pops {
        state = stack_pop(&state)?;
    }
    Ok(state)
}

/// Construct a stack from abstract values on the stack
/// The stack is stored in the register after creation
pub fn stack_create_struct(
    state: &AbstractState,
    struct_index: StructDefinitionIndex,
) -> Result<AbstractState, VMError> {
    let state_copy = state.clone();
    let mut state = state.clone();
    let struct_def = state_copy.module.struct_def_at(struct_index);
    let struct_def_view = StructDefinitionView::new(&state_copy.module, struct_def);
    let tokens: Vec<SignatureToken> = struct_def_view
        .fields()
        .into_iter()
        .flatten()
        .map(|field| field.type_signature().token().as_inner().clone())
        .collect();
    // The logic for determine the struct kind was sourced from `SignatureTokenView`
    // TODO: This will need to be updated when struct type actuals are handled
    let struct_kind = match struct_def_view.is_nominal_resource() {
        true => Kind::Resource,
        false => tokens
            .iter()
            .map(|token| SignatureTokenView::new(&state.module, token).kind(&[]))
            .fold(Kind::Unrestricted, |acc_kind, next_kind| {
                match (acc_kind, next_kind) {
                    (Kind::All, _) | (_, Kind::All) => Kind::All,
                    (Kind::Resource, _) | (_, Kind::Resource) => Kind::Resource,
                    (Kind::Unrestricted, Kind::Unrestricted) => Kind::Unrestricted,
                }
            }),
    };
    let struct_value = AbstractValue::new_struct(
        SignatureToken::Struct(struct_def.struct_handle, tokens),
        struct_kind,
    );
    state.set_register(struct_value);
    Ok(state)
}

/// Determine if a struct of the given signature is at the top of the stack
pub fn stack_has_struct(state: &AbstractState, struct_index: StructDefinitionIndex) -> bool {
    if state.stack_len() > 0 {
        let struct_def = state.module.struct_def_at(struct_index);
        if let Some(struct_value) = state.stack_peek(0) {
            match struct_value.token {
                SignatureToken::Struct(struct_handle, _) => {
                    return struct_handle == struct_def.struct_handle;
                }
                _ => return false,
            }
        }
    }
    false
}

/// Determine if a struct at the given index is a resource
pub fn struct_is_resource(state: &AbstractState, struct_index: StructDefinitionIndex) -> bool {
    let struct_def = state.module.struct_def_at(struct_index);
    StructDefinitionView::new(&state.module, struct_def).is_nominal_resource()
}

/// Push the fields of a struct as `AbstractValue`s to the stack
pub fn stack_unpack_struct(
    state: &AbstractState,
    struct_index: StructDefinitionIndex,
) -> Result<AbstractState, VMError> {
    let state_copy = state.clone();
    let mut state = state.clone();
    let struct_def = state_copy.module.struct_def_at(struct_index);
    let struct_def_view = StructDefinitionView::new(&state_copy.module, struct_def);
    let token_views = struct_def_view
        .fields()
        .into_iter()
        .flatten()
        .map(|field| field.type_signature().token());
    for token_view in token_views {
        let abstract_value = AbstractValue {
            token: token_view.as_inner().clone(),
            kind: token_view.kind(&[]),
        };
        state = stack_push(&state, abstract_value)?;
    }
    Ok(state)
}

/// Wrapper for enclosing the arguments of `stack_has` so that only the `state` needs
/// to be given.
#[macro_export]
macro_rules! state_stack_has {
    ($e1: expr, $e2: expr) => {
        Box::new(move |state| stack_has(state, $e1, $e2))
    };
}

/// Wrapper for for enclosing the arguments of `stack_has_polymorphic_eq` so that only the `state`
/// needs to be given.
#[macro_export]
macro_rules! state_stack_has_polymorphic_eq {
    ($e1: expr, $e2: expr) => {
        Box::new(move |state| stack_has_polymorphic_eq(state, $e1, $e2))
    };
}

/// Wrapper for enclosing the arguments of `stack_pop` so that only the `state` needs
/// to be given.
#[macro_export]
macro_rules! state_stack_pop {
    () => {
        Box::new(move |state| stack_pop(state))
    };
}

/// Wrapper for enclosing the arguments of `stack_push` so that only the `state` needs
/// to be given.
#[macro_export]
macro_rules! state_stack_push {
    ($e: expr) => {
        Box::new(move |state| stack_push(state, $e))
    };
}

/// Wrapper for enclosing the arguments of `stack_push_register` so that only the `state` needs
/// to be given.
#[macro_export]
macro_rules! state_stack_push_register {
    () => {
        Box::new(move |state| stack_push_register(state))
    };
}

/// Wrapper for enclosing the arguments of `stack_local_polymorphic_eq` so that only the `state`
/// needs to be given.
#[macro_export]
macro_rules! state_stack_local_polymorphic_eq {
    ($e1: expr, $e2: expr) => {
        Box::new(move |state| stack_local_polymorphic_eq(state, $e1, $e2))
    };
}

/// Wrapper for enclosing the arguments of `local_exists` so that only the `state` needs
/// to be given.
#[macro_export]
macro_rules! state_local_exists {
    ($e: expr) => {
        Box::new(move |state| local_exists(state, $e))
    };
}

/// Wrapper for enclosing the arguments of `local_availability_is` so that only the `state` needs
/// to be given.
#[macro_export]
macro_rules! state_local_availability_is {
    ($e: expr, $a: expr) => {
        Box::new(move |state| local_availability_is(state, $e, $a))
    };
}

/// Wrapper for enclosing the arguments of `local_kind_is` so that only the `state` needs
/// to be given.
#[macro_export]
macro_rules! state_local_kind_is {
    ($e: expr, $a: expr) => {
        Box::new(move |state| local_kind_is(state, $e, $a))
    };
}

/// Wrapper for enclosing the arguments of `local_set` so that only the `state` needs
/// to be given.
#[macro_export]
macro_rules! state_local_set {
    ($e: expr, $a: expr) => {
        Box::new(move |state| local_set(state, $e, $a))
    };
}

/// Wrapper for enclosing the arguments of `local_take` so that only the `state` needs
/// to be given.
#[macro_export]
macro_rules! state_local_take {
    ($e: expr) => {
        Box::new(move |state| local_take(state, $e))
    };
}

/// Wrapper for enclosing the arguments of `local_take_borrow` so that only the `state` needs
/// to be given.
#[macro_export]
macro_rules! state_local_take_borrow {
    ($e: expr, $mutable: expr) => {
        Box::new(move |state| local_take_borrow(state, $e, $mutable))
    };
}

/// Wrapper for enclosing the arguments of `local_palce` so that only the `state` needs
/// to be given.
#[macro_export]
macro_rules! state_local_place {
    ($e: expr) => {
        Box::new(move |state| local_place(state, $e))
    };
}

/// Wrapper for enclosing the arguments of `stack_satisfies_struct_signature` so that only the
/// `state` needs to be given.
#[macro_export]
macro_rules! state_stack_satisfies_struct_signature {
    ($e: expr) => {
        Box::new(move |state| stack_satisfies_struct_signature(state, $e))
    };
}

/// Wrapper for enclosing the arguments of `stack_struct_popn` so that only the
/// `state` needs to be given.
#[macro_export]
macro_rules! state_stack_struct_popn {
    ($e: expr) => {
        Box::new(move |state| stack_struct_popn(state, $e))
    };
}

/// Wrapper for enclosing the arguments of `stack_pack_struct` so that only the
/// `state` needs to be given.
#[macro_export]
macro_rules! state_stack_create_struct {
    ($e: expr) => {
        Box::new(move |state| stack_create_struct(state, $e))
    };
}

/// Wrapper for enclosing the arguments of `stack_has_struct` so that only the
/// `state` needs to be given.
#[macro_export]
macro_rules! state_stack_has_struct {
    ($e: expr) => {
        Box::new(move |state| stack_has_struct(state, $e))
    };
}

/// Wrapper for enclosing the arguments of `stack_unpack_struct` so that only the
/// `state` needs to be given.
#[macro_export]
macro_rules! state_stack_unpack_struct {
    ($e: expr) => {
        Box::new(move |state| stack_unpack_struct(state, $e))
    };
}

/// Wrapper for enclosing the arguments of `struct_is_resource` so that only the
/// `state` needs to be given.
#[macro_export]
macro_rules! state_struct_is_resource {
    ($e: expr) => {
        Box::new(move |state| struct_is_resource(state, $e))
    };
}

/// Predicate that is false for every state.
#[macro_export]
macro_rules! state_never {
    () => {
        Box::new(|_| (false))
    };
}
