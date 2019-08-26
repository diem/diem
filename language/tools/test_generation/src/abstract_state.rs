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

    /// Temporary location for storing the results of instruction effects for
    /// access by subsequent instructions
    register: Option<SignatureToken>,
}

impl AbstractState {
    /// Create a new AbstractState given a list of `SignatureTokens` that will be
    /// the (available) locals that the state will have.
    pub fn new() -> AbstractState {
        AbstractState {
            stack: Vec::new(),
            locals: HashMap::new(),
            register: None,
        }
    }

    /// Create a new AbstractState given a list of `SignatureTokens` that will be
    /// the (available) locals that the state will have.
    pub fn from_locals(locals: HashMap<usize, (SignatureToken, BorrowState)>) -> AbstractState {
        AbstractState {
            stack: Vec::new(),
            locals,
            register: None,
        }
    }

    /// Get the register value and set it to `None`
    fn get_register(&mut self) -> Option<SignatureToken> {
        let value = self.register.clone();
        self.register = None;
        value
    }

    /// Add a `SignatureToken` to the stack
    pub fn stack_push(&mut self, item: SignatureToken) {
        self.stack.push(item);
    }

    /// Add a `SignatureToken` to the stack from the register
    pub fn stack_push_register(&mut self) {
        if let Some(token) = self.get_register() {
            self.stack.push(token);
        } else {
            panic!("Error: No value in register");
        }
    }

    /// Remove a `SignatureToken` from the stack if it exists to the register
    pub fn stack_pop(&mut self) {
        self.register = self.stack.pop();
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

    /// Check if the local at index `i` exists
    pub fn local_exists(&self, i: usize) -> bool {
        self.locals.get(&i).is_some()
    }

    /// Get the local at index `i` if it exists
    pub fn local_get(&self, i: usize) -> Option<&(SignatureToken, BorrowState)> {
        self.locals.get(&i)
    }

    /// Place the local at index `i` if it exists into the register
    pub fn local_take(&mut self, i: usize) {
        checked_precondition!(self.local_exists(i), "Failed to get local");
        let (token, _) = self.locals.get(&i).unwrap();
        self.register = Some(token.clone());
    }

    /// Place a reference to the local at index `i` if it exists into the register
    pub fn local_take_borrow(&mut self, i: usize, mutable: bool) {
        checked_precondition!(self.local_exists(i), "Failed to get reference to local");
        let (token, _) = self.locals.get(&i).unwrap();
        let ref_token = if mutable {
            SignatureToken::MutableReference(Box::new(token.clone()))
        } else {
            SignatureToken::Reference(Box::new(token.clone()))
        };
        self.register = Some(ref_token);
    }

    /// Set the availability of the local at index `i`
    pub fn local_set(&mut self, i: usize, availability: BorrowState) {
        checked_precondition!(self.local_exists(i), "Failed to change local availability");
        let (token, _) = self.locals.get(&i).unwrap().clone();
        self.locals.insert(i, (token, availability));
    }

    /// Check whether a local is in a particular `BorrowState`
    pub fn local_is(&self, i: usize, availability: BorrowState) -> bool {
        checked_precondition!(self.local_exists(i), "Failed to check local availability");
        let (_, availability1) = self.locals.get(&i).unwrap();
        availability == *availability1
    }

    /// Insert a local at index `i` as `Available`
    pub fn local_insert(&mut self, i: usize, token: SignatureToken, availability: BorrowState) {
        self.locals.insert(i, (token, availability));
    }

    /// Insert a local at index `i` as `Available` from the register
    pub fn local_place(&mut self, i: usize) {
        let value = self.get_register();
        assert!(value.is_some(), "Failed to insert local from stack");
        let token = value.unwrap();
        self.locals
            .insert(i, (token.clone(), BorrowState::Available));
    }

    /// Get all of the locals
    pub fn get_locals(&self) -> &HashMap<usize, (SignatureToken, BorrowState)> {
        &self.locals
    }

    /// TODO: Determine whether the current state is a final state
    pub fn is_final(&self) -> bool {
        self.stack.is_empty()
    }
}
