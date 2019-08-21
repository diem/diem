// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use vm::file_format::SignatureToken;

/// The BorrowState denotes whether a local is `Available` or
/// has been moved and is `Unavailable`.
#[derive(Debug, Clone, PartialEq)]
pub enum BorrowState {
    Available,
    Unavailable,
}

/// An AbstractState represents an abstract view of the execution of the
/// Move VM. Rather than considering values of items on the stack or in
/// the locals, we only consider their type, represented by a `SignatureToken`
/// and their availibility, represented by the `BorrowState`.
#[derive(Debug, Clone)]
pub struct AbstractState {
    /// A Vector of `SignatureToken`s representing the VM value stack
    stack: Vec<SignatureToken>,

    /// A HashMap mapping local indicies to `SignatureToken`s and `BorrowState`s
    locals: HashMap<usize, (SignatureToken, BorrowState)>,
}

impl AbstractState {
    /// Create a new AbstractState given a list of `SignatureTokens` that will be
    /// the (available) locals that the state will have.
    pub fn new(initial_locals: &[SignatureToken]) -> AbstractState {
        AbstractState {
            stack: Vec::new(),
            locals: initial_locals
                .iter()
                .enumerate()
                .map(|(i, token)| (i, (token.clone(), BorrowState::Available)))
                .collect(),
        }
    }

    /// Add a `SignatureToken` to the stack
    pub fn stack_push(&mut self, item: SignatureToken) {
        self.stack.push(item);
    }

    /// Remove a `SignatureToken` from the stack if it exists
    pub fn stack_pop(&mut self) -> Option<SignatureToken> {
        self.stack.pop()
    }

    /// Get the `SignatureToken` at index `index` on the stack if it exists.
    /// Index 0 is the top of the stack.
    pub fn stack_peek(&self, index: usize) -> Option<SignatureToken> {
        if index < self.stack.len() {
            Some(self.stack[self.stack.len() - 1 - index].clone())
        } else {
            None
        }
    }

    /// Get the length of the stack.
    pub fn stack_len(&self) -> usize {
        self.stack.len()
    }

    /// Check whether a local is available. Defaults to false if no local is
    /// present at index `i`.
    pub fn local_available(&self, i: usize) -> bool {
        if let Some((_, borrow_state)) = self.locals.get(&i) {
            return *borrow_state == BorrowState::Available;
        }
        false
    }

    /// Get the local at index `i` if it exists
    pub fn get_local(&self, i: usize) -> Option<&(SignatureToken, BorrowState)> {
        self.locals.get(&i)
    }

    /// Get the local at index `i` and set it to `Unavailable`
    pub fn move_local(&mut self, i: usize) -> Option<&(SignatureToken, BorrowState)> {
        // TODO: Change to precondition once MIRAI is integrated
        assert!(self.get_local(i).is_some(), "Failed to move local");
        let (local, _) = self.get_local(i).unwrap().clone();
        self.locals.insert(i, (local, BorrowState::Unavailable));
        self.locals.get(&i)
    }

    /// Insert a local at index `i` as `Available`
    pub fn insert_local(&mut self, i: usize, token: SignatureToken) {
        // TODO: What should the behavior be if there is already a local at i?
        self.locals
            .insert(i, (token.clone(), BorrowState::Available));
    }

    pub fn get_locals(&self) -> &HashMap<usize, (SignatureToken, BorrowState)> {
        &self.locals
    }

    /// TODO: Determine whether the current state is a final state
    pub fn is_final(&self) -> bool {
        self.stack.is_empty()
    }
}
