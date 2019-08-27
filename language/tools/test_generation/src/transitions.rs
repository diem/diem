// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::abstract_state::{AbstractState, AbstractValue, BorrowState};
use vm::file_format::Kind;

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
        return state.stack_peek(index1) == state.stack_peek(index2);
    }
    false
}

/// Pop from the top of the stack.
pub fn stack_pop(state: &AbstractState) -> AbstractState {
    let mut state = state.clone();
    state.stack_pop();
    state
}

/// Push given abstract_value to the top of the stack.
pub fn stack_push(state: &AbstractState, abstract_value: AbstractValue) -> AbstractState {
    let mut state = state.clone();
    state.stack_push(abstract_value);
    state
}

/// Push to the top of the stack from the register.
pub fn stack_push_register(state: &AbstractState) -> AbstractState {
    let mut state = state.clone();
    state.stack_push_register();
    state
}

/// Check whether the local at `index` exists
pub fn local_exists(state: &AbstractState, index: u8) -> bool {
    state.local_exists(index as usize)
}

/// Check whether the local at `index` is of the given availability
pub fn local_availability_is(state: &AbstractState, index: u8, availability: BorrowState) -> bool {
    state.local_availability_is(index as usize, availability)
}

/// Check whether the local at `index` is of the given kind
pub fn local_kind_is(state: &AbstractState, index: u8, kind: Kind) -> bool {
    state.local_kind_is(index as usize, kind)
}

/// Set the availability of local at `index`
pub fn local_set(state: &AbstractState, index: u8, availability: BorrowState) -> AbstractState {
    let mut state = state.clone();
    state.local_set(index as usize, availability);
    state
}

/// Put copy of the local at `index` in register
pub fn local_take(state: &AbstractState, index: u8) -> AbstractState {
    let mut state = state.clone();
    state.local_take(index as usize);
    state
}

/// Put reference to local at `index` in register
pub fn local_take_borrow(state: &AbstractState, index: u8, mutable: bool) -> AbstractState {
    let mut state = state.clone();
    state.local_take_borrow(index as usize, mutable);
    state
}

/// Insert the register value into the locals at `index`
pub fn local_place(state: &AbstractState, index: u8) -> AbstractState {
    let mut state = state.clone();
    state.local_place(index as usize);
    state
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

/// Predicate that is false for every state.
#[macro_export]
macro_rules! state_never {
    () => {
        Box::new(|_| (false))
    };
}
