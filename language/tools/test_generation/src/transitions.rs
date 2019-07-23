// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::abstract_state::AbstractState;
use vm::file_format::SignatureToken;

/// Determine whether the stack is at least of size `index`. If the optional `token` argument
/// is some `SignatureToken`, check whether the type at `index` is that token.
pub fn stack_has(state: &AbstractState, index: usize, token: Option<SignatureToken>) -> bool {
    match token {
        Some(token) => index < state.stack_len() && state.stack_peek(index) == Some(token),
        None => index < state.stack_len(),
    }
}

/// Pop from the top of the stack.
pub fn stack_pop(state: &AbstractState) -> AbstractState {
    let mut state = state.clone();
    state.stack_pop();
    state
}

/// Push to the top of the stack.
pub fn stack_push(state: &AbstractState, token: SignatureToken) -> AbstractState {
    let mut state = state.clone();
    state.stack_push(token);
    state
}

/// Check whether the local at `index` is available.
pub fn local_available(state: &AbstractState, index: u8) -> bool {
    state.local_available(index as usize)
}

/// Push a copy of the local at `index` to the stack.
pub fn stack_push_local_copy(state: &AbstractState, index: u8) -> AbstractState {
    // TODO: Change to precondition once MIRAI is integrated
    assert!(
        state.get_local(index as usize).is_some(),
        "Failed to copy local to stack"
    );
    let (token, _) = state.get_local(index as usize).unwrap();
    stack_push(state, token.clone())
}

/// Move a local at `index` to the stack.
pub fn stack_push_local_move(state: &AbstractState, index: u8) -> AbstractState {
    // TODO: Change to precondition once MIRAI is integrated
    assert!(
        state.get_local(index as usize).is_some(),
        "Failed to move local to stack"
    );
    let mut state = state.clone();
    let (token, _) = state.move_local(index as usize).unwrap().clone();
    stack_push(&state, token)
}

/// Push a reference to the local at `index` to the stack.
pub fn stack_push_local_borrow(state: &AbstractState, index: u8) -> AbstractState {
    // TODO: Change to precondition once MIRAI is integrated
    assert!(
        state.get_local(index as usize).is_some(),
        "Failed to borrow local to stack"
    );
    let state = state.clone();
    let (token, _) = state.get_local(index as usize).unwrap().clone();
    stack_push(&state, SignatureToken::MutableReference(Box::new(token)))
}

/// Pop an element from the stack and insert it into the locals at `index`
pub fn stack_pop_local_insert(state: &AbstractState, index: u8) -> AbstractState {
    // TODO: Change to precondition once MIRAI is integrated
    assert!(
        state.stack_peek(0).is_some(),
        "Failed to insert local from stack"
    );
    let mut state = state.clone();
    let token = state.stack_peek(0).unwrap();
    state.insert_local(index as usize, token.clone());
    state
}

/// Wrapper for enclosing the arguments of `stack_has` so that only the `state` needs
/// to be given.
#[macro_export]
macro_rules! state_stack_has {
    ($e1: expr, $e2: expr) => {
        Box::new(move |state| stack_has(state, $e1, $e2))
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

/// Wrapper for enclosing the arguments of `stack_push_local_copy` so that only the `state` needs
/// to be given.
#[macro_export]
macro_rules! state_stack_push_local_copy {
    ($e: expr) => {
        Box::new(move |state| stack_push_local_copy(state, $e))
    };
}

/// Wrapper for enclosing the arguments of `stack_push_local_move` so that only the `state` needs
/// to be given.
#[macro_export]
macro_rules! state_stack_push_local_move {
    ($e: expr) => {
        Box::new(move |state| stack_push_local_move(state, $e))
    };
}

/// Wrapper for enclosing the arguments of `stack_push_local_borrow` so that only the `state` needs
/// to be given.
#[macro_export]
macro_rules! state_stack_push_local_borrow {
    ($e: expr) => {
        Box::new(move |state| stack_push_local_borrow(state, $e))
    };
}

/// Wrapper for enclosing the arguments of `stack_pop_local_insert` so that only the `state` needs
/// to be given.
#[macro_export]
macro_rules! state_stack_pop_local_insert {
    ($e: expr) => {
        Box::new(move |state| stack_pop_local_insert(state, $e))
    };
}

/// Wrapper for enclosing the arguments of `local_available` so that only the `state` needs
/// to be given.
#[macro_export]
macro_rules! state_local_available {
    ($e: expr) => {
        Box::new(move |state| local_available(state, $e))
    };
}

/// Predicate that is false for every state.
#[macro_export]
macro_rules! state_never {
    () => {
        Box::new(move |_| (false))
    };
}
