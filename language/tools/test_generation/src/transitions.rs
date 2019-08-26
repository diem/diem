// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::abstract_state::{AbstractState, BorrowState};
use vm::file_format::SignatureToken;

/// Determine whether the stack is at least of size `index`. If the optional `token` argument
/// is some `SignatureToken`, check whether the type at `index` is that token.
pub fn stack_has(state: &AbstractState, index: usize, token: Option<SignatureToken>) -> bool {
    match token {
        Some(token) => index < state.stack_len() && state.stack_peek(index) == Some(token),
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

/// Push given token to the top of the stack.
pub fn stack_push(state: &AbstractState, token: SignatureToken) -> AbstractState {
    let mut state = state.clone();
    state.stack_push(token);
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

/// Check whether the local at `index` is available (`true`) or unavailable (`false`)
pub fn local_is(state: &AbstractState, index: u8, availability: BorrowState) -> bool {
    state.local_is(index as usize, availability)
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

/// Determine whether a token on the stack and a token in the locals have the same type
pub fn stack_local_polymorphic_eq(state: &AbstractState, index1: usize, index2: usize) -> bool {
    if stack_has(state, index1, None) {
        if let Some((token, _)) = state.local_get(index2) {
            return state.stack_peek(index1) == Some(token.clone());
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

/// Wrapper for determining whether two tokens on the stack have the same type
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

/// Wrapper for determining whether a token on the stack and a token in the locals
/// have the same type
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

/// Wrapper for enclosing the arguments of `local_exists` so that only the `state` needs
/// to be given.
#[macro_export]
macro_rules! state_local_is {
    ($e: expr, $a: expr) => {
        Box::new(move |state| local_is(state, $e, $a))
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

/// Wrapper for enclosing the arguments of `local_get` so that only the `state` needs
/// to be given.
#[macro_export]
macro_rules! state_local_take {
    ($e: expr) => {
        Box::new(move |state| local_take(state, $e))
    };
}

/// Wrapper for enclosing the arguments of `local_get_borrow` so that only the `state` needs
/// to be given.
#[macro_export]
macro_rules! state_local_take_borrow {
    ($e: expr, $mutable: expr) => {
        Box::new(move |state| local_take_borrow(state, $e, $mutable))
    };
}

/// Wrapper for enclosing the arguments of `stack_local_insert` so that only the `state` needs
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
