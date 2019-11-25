// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{borrow_graph::BorrowGraph, error::VMError};
use std::{collections::HashMap, fmt};
use vm::file_format::{
    empty_module, CompiledModule, CompiledModuleMut, Kind, SignatureToken, StructDefinitionIndex,
};

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

/// This models the mutability of a reference
#[derive(Debug, Clone, PartialEq)]
pub enum Mutability {
    /// Represents a mutable reference
    Mutable,

    /// Represents an immutable reference
    Immutable,

    /// When we don't need to specify whether
    /// the reference is mutable or immutable
    Either,
}

impl AbstractValue {
    /// Create a new primitive `AbstractValue` given its type; the kind will be `Unrestricted`
    pub fn new_primitive(token: SignatureToken) -> AbstractValue {
        checked_precondition!(
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
        checked_precondition!(
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
        checked_precondition!(
            match token {
                SignatureToken::Struct(_, _) => true,
                _ => false,
            },
            "AbstractValue::new_struct must be applied with a struct type"
        );
        AbstractValue { token, kind }
    }

    pub fn new_value(token: SignatureToken, kind: Kind) -> AbstractValue {
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

    /// The global resources acquired by the function corresponding to this abstract state
    pub acquires_global_resources: Vec<StructDefinitionIndex>,

    /// This flag is set when applying an instruction that should result in an error
    /// in the VM runtime.
    aborted: bool,

    /// This graph stores borrow information needed to ensure that bytecode instructions
    /// are memory safe
    borrow_graph: BorrowGraph,
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
            acquires_global_resources: Vec::new(),
            aborted: false,
            borrow_graph: BorrowGraph::new(0),
        }
    }

    /// Create a new AbstractState given a list of `SignatureTokens` that will be
    /// the (available) locals that the state will have, as well as the module state
    pub fn from_locals(
        module: CompiledModuleMut,
        locals: HashMap<usize, (AbstractValue, BorrowState)>,
        acquires_global_resources: Vec<StructDefinitionIndex>,
    ) -> AbstractState {
        let locals_len = locals.len();
        AbstractState {
            stack: Vec::new(),
            locals,
            register: None,
            module: module
                .freeze()
                .expect("Module should pass the bounds checker"),
            acquires_global_resources,
            aborted: false,
            borrow_graph: BorrowGraph::new(locals_len as u8),
        }
    }

    /// Get the register value
    pub fn register_copy(&mut self) -> Option<AbstractValue> {
        self.register.clone()
    }

    /// Get the register value and set it to `None`
    pub fn register_move(&mut self) -> Option<AbstractValue> {
        let value = self.register.clone();
        self.register = None;
        value
    }

    /// Set the register value and set it to `None`
    pub fn register_set(&mut self, value: AbstractValue) {
        self.register = Some(value.clone());
    }

    /// Add a `AbstractValue` to the stack
    pub fn stack_push(&mut self, item: AbstractValue) {
        // Programs that are large enough to exceed this bound
        // will not be generated
        assume!(self.stack.len() < usize::max_value());
        self.stack.push(item);
    }

    /// Add a `AbstractValue` to the stack from the register
    /// If the register is `None` return a `VMError`
    pub fn stack_push_register(&mut self) -> Result<(), VMError> {
        if let Some(abstract_value) = self.register_move() {
            // Programs that are large enough to exceed this bound
            // will not be generated
            assume!(self.stack.len() < usize::max_value());
            self.stack.push(abstract_value);
            Ok(())
        } else {
            Err(VMError::new("Error: No value in register".to_string()))
        }
    }

    /// Remove an `AbstractValue` from the stack if it exists to the register
    /// If it does not exist return a `VMError`.
    pub fn stack_pop(&mut self) -> Result<(), VMError> {
        if self.stack.is_empty() {
            Err(VMError::new("Pop attempted on empty stack".to_string()))
        } else {
            self.register = self.stack.pop();
            Ok(())
        }
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
    /// If it does not exist return a `VMError`.
    pub fn local_take(&mut self, i: usize) -> Result<(), VMError> {
        if let Some((abstract_value, _)) = self.locals.get(&i) {
            self.register = Some(abstract_value.clone());
            Ok(())
        } else {
            Err(VMError::new(format!("Local does not exist at index {}", i)))
        }
    }

    /// Place a reference to the local at index `i` if it exists into the register
    /// If it does not exist return a `VMError`.
    pub fn local_take_borrow(&mut self, i: usize, mutability: Mutability) -> Result<(), VMError> {
        if let Some((abstract_value, _)) = self.locals.get(&i) {
            let ref_token = match mutability {
                Mutability::Mutable => {
                    SignatureToken::MutableReference(Box::new(abstract_value.token.clone()))
                }
                Mutability::Immutable => {
                    SignatureToken::Reference(Box::new(abstract_value.token.clone()))
                }
                Mutability::Either => {
                    return Err(VMError::new("Mutability cannot be Either".to_string()))
                }
            };
            self.register = Some(AbstractValue::new_reference(ref_token, abstract_value.kind));
            Ok(())
        } else {
            Err(VMError::new(format!("Local does not exist at index {}", i)))
        }
    }

    /// Set the availability of the local at index `i`
    /// If it does not exist return a `VMError`.
    pub fn local_set(&mut self, i: usize, availability: BorrowState) -> Result<(), VMError> {
        if let Some((abstract_value, _)) = self.locals.clone().get(&i) {
            self.locals
                .insert(i, (abstract_value.clone(), availability));
            Ok(())
        } else {
            Err(VMError::new(format!("Local does not exist at index {}", i)))
        }
    }

    /// Check whether a local is in a particular `BorrowState`
    /// If the local does not exist return a `VMError`.
    pub fn local_availability_is(
        &self,
        i: usize,
        availability: BorrowState,
    ) -> Result<bool, VMError> {
        if let Some((_, availability1)) = self.locals.get(&i) {
            Ok(availability == *availability1)
        } else {
            Err(VMError::new(format!("Local does not exist at index {}", i)))
        }
    }

    /// Check whether a local is in a particular `Kind`
    /// If the local does not exist return a `VMError`.
    pub fn local_kind_is(&self, i: usize, kind: Kind) -> Result<bool, VMError> {
        if let Some((abstract_value, _)) = self.locals.get(&i) {
            Ok(abstract_value.kind == kind)
        } else {
            Err(VMError::new(format!("Local does not exist at index {}", i)))
        }
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
    /// If the register value is `None` return a `VMError`.
    pub fn local_place(&mut self, i: usize) -> Result<(), VMError> {
        if let Some(abstract_value) = self.register_move() {
            self.locals
                .insert(i, (abstract_value.clone(), BorrowState::Available));
            Ok(())
        } else {
            Err(VMError::new(
                "Could not insert local, register is empty".to_string(),
            ))
        }
    }

    /// Get all of the locals
    pub fn get_locals(&self) -> &HashMap<usize, (AbstractValue, BorrowState)> {
        &self.locals
    }

    /// Set the abstract state to be `aborted` when a precondition of an instruction
    /// fails. (This will happen if `NEGATE_PRECONDITIONs` is true).
    pub fn abort(&mut self) {
        self.aborted = true;
    }

    /// Whether the state is aborted
    pub fn has_aborted(&self) -> bool {
        self.aborted
    }

    /// The final state is one where the stack is empty
    pub fn is_final(&self) -> bool {
        self.stack.is_empty()
    }
}

impl fmt::Display for AbstractState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Stack: {:?} | Locals: {:?}", self.stack, self.locals)
    }
}

impl fmt::Display for AbstractValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({:?} {:?})", self.token, self.kind)
    }
}
