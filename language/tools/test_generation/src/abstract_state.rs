// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use vm::file_format::{empty_module, CompiledModule, CompiledModuleMut, Kind, SignatureToken};

/// The BorrowState denotes whether a local is `Available` or
/// has been moved and is `Unavailable`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BorrowState {
    Available,
    Unavailable,
}

/// This models a value on the stack or in the locals
#[derive(Debug, Clone, PartialEq)]
pub struct AbstractValue {
    /// Represents the type of the value
    pub token: SignatureToken,

    /// Represents the kind of the value
    pub kind: Kind,
}

impl AbstractValue {
    /// Create a new primitive `AbstractValue` given its type; the kind will be `Unrestricted`
    pub fn new_primitive(token: SignatureToken) -> AbstractValue {
        assert!(
            match token {
                SignatureToken::Struct(_, _) => false,
                SignatureToken::Reference(_) => false,
                SignatureToken::MutableReference(_) => false,
                _ => true,
            },
            "AbstractValue::new_primitive must be applied with primitive type"
        );
        AbstractValue {
            token,
            kind: Kind::Unrestricted,
        }
    }

    /// Create a new reference `AbstractValue` given its type and kind
    pub fn new_reference(token: SignatureToken, kind: Kind) -> AbstractValue {
        assert!(
            match token {
                SignatureToken::Reference(_) => true,
                SignatureToken::MutableReference(_) => true,
                _ => false,
            },
            "AbstractValue::new_reference must be applied with a reference type"
        );
        AbstractValue { token, kind }
    }

    /// Create a new struct `AbstractValue` given its type and kind
    pub fn new_struct(token: SignatureToken, kind: Kind) -> AbstractValue {
        assert!(
            match token {
                SignatureToken::Struct(_, _) => true,
                _ => false,
            },
            "AbstractValue::new_struct must be applied with a struct type"
        );
        AbstractValue { token, kind }
    }
}

/// An AbstractState represents an abstract view of the execution of the
/// Move VM. Rather than considering values of items on the stack or in
/// the locals, we only consider their type, represented by a `AbstractValue`
/// and their availibility, represented by the `BorrowState`.
#[derive(Debug, Clone)]
pub struct AbstractState {
    /// A Vector of `AbstractValue`s representing the VM value stack
    stack: Vec<AbstractValue>,

    /// A HashMap mapping local indicies to `AbstractValue`s and `BorrowState`s
    locals: HashMap<usize, (AbstractValue, BorrowState)>,

    /// Temporary location for storing the results of instruction effects for
    /// access by subsequent instructions' effects
    register: Option<AbstractValue>,

    /// The module state
    pub module: CompiledModule,
}

impl AbstractState {
    /// Create a new AbstractState with empty stack, locals, and register
    pub fn new() -> AbstractState {
        AbstractState {
            stack: Vec::new(),
            locals: HashMap::new(),
            register: None,
            module: empty_module()
                .freeze()
                .expect("Empty module should pass the bounds checker"),
        }
    }

    /// Create a new AbstractState given a list of `SignatureTokens` that will be
    /// the (available) locals that the state will have, as well as the module state
    pub fn from_locals(
        module: CompiledModuleMut,
        locals: HashMap<usize, (AbstractValue, BorrowState)>,
    ) -> AbstractState {
        AbstractState {
            stack: Vec::new(),
            locals,
            register: None,
            module: module
                .freeze()
                .expect("Module should pass the bounds checker"),
        }
    }

    /// Get the register value and set it to `None`
    fn take_register(&mut self) -> Option<AbstractValue> {
        let value = self.register.clone();
        self.register = None;
        value
    }

    /// Set the register value and set it to `None`
    pub fn set_register(&mut self, value: AbstractValue) {
        self.register = Some(value.clone());
    }

    /// Add a `AbstractValue` to the stack
    pub fn stack_push(&mut self, item: AbstractValue) {
        self.stack.push(item);
    }

    /// Add a `AbstractValue` to the stack from the register
    pub fn stack_push_register(&mut self) {
        if let Some(abstract_value) = self.take_register() {
            self.stack.push(abstract_value);
        } else {
            panic!("Error: No value in register");
        }
    }

    /// Remove a `AbstractValue` from the stack if it exists to the register
    pub fn stack_pop(&mut self) {
        self.register = self.stack.pop();
    }

    /// Get the `AbstractValue` at index `index` on the stack if it exists.
    /// Index 0 is the top of the stack.
    pub fn stack_peek(&self, index: usize) -> Option<AbstractValue> {
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
    pub fn local_get(&self, i: usize) -> Option<&(AbstractValue, BorrowState)> {
        self.locals.get(&i)
    }

    /// Place the local at index `i` if it exists into the register
    pub fn local_take(&mut self, i: usize) {
        checked_precondition!(self.local_exists(i), "Failed to get local");
        let (abstract_value, _) = self.locals.get(&i).unwrap();
        self.register = Some(abstract_value.clone());
    }

    /// Place a reference to the local at index `i` if it exists into the register
    pub fn local_take_borrow(&mut self, i: usize, mutable: bool) {
        checked_precondition!(self.local_exists(i), "Failed to get reference to local");
        let (abstract_value, _) = self.locals.get(&i).unwrap();
        let ref_token = if mutable {
            SignatureToken::MutableReference(Box::new(abstract_value.token.clone()))
        } else {
            SignatureToken::Reference(Box::new(abstract_value.token.clone()))
        };
        self.register = Some(AbstractValue::new_reference(ref_token, abstract_value.kind));
    }

    /// Set the availability of the local at index `i`
    pub fn local_set(&mut self, i: usize, availability: BorrowState) {
        checked_precondition!(self.local_exists(i), "Failed to change local availability");
        let (abstract_value, _) = self.locals.get(&i).unwrap().clone();
        self.locals.insert(i, (abstract_value, availability));
    }

    /// Check whether a local is in a particular `BorrowState`
    pub fn local_availability_is(&self, i: usize, availability: BorrowState) -> bool {
        checked_precondition!(self.local_exists(i), "Failed to check local availability");
        let (_, availability1) = self.locals.get(&i).unwrap();
        availability == *availability1
    }

    /// Check whether a local is in a particular `Kind`
    pub fn local_kind_is(&self, i: usize, kind: Kind) -> bool {
        checked_precondition!(self.local_exists(i), "Failed to check local kind");
        let (abstract_value, _) = self.locals.get(&i).unwrap();
        abstract_value.kind == kind
    }

    /// Insert a local at index `i` as `Available`
    pub fn local_insert(
        &mut self,
        i: usize,
        abstract_value: AbstractValue,
        availability: BorrowState,
    ) {
        self.locals.insert(i, (abstract_value, availability));
    }

    /// Insert a local at index `i` as `Available` from the register
    pub fn local_place(&mut self, i: usize) {
        let value = self.take_register();
        assert!(value.is_some(), "Failed to insert local from stack");
        let abstract_value = value.unwrap();
        self.locals
            .insert(i, (abstract_value.clone(), BorrowState::Available));
    }

    /// Get all of the locals
    pub fn get_locals(&self) -> &HashMap<usize, (AbstractValue, BorrowState)> {
        &self.locals
    }

    /// TODO: Determine whether the current state is a final state
    pub fn is_final(&self) -> bool {
        self.stack.is_empty()
    }
}
