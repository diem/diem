// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    abstract_state::{AbstractState, AbstractValue, BorrowState, Mutability},
    config::ALLOW_MEMORY_UNSAFE,
    error::VMError,
};
use vm::{
    access::*,
    file_format::{
        FieldDefinitionIndex, FunctionHandleIndex, Kind, SignatureToken, StructDefinitionIndex,
    },
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

/// Determine the abstract value at `index` is of the given kind, if it exists.
/// If it does not exist, return `false`.
pub fn stack_kind_is(state: &AbstractState, index: usize, kind: Kind) -> bool {
    if index < state.stack_len() {
        match state.stack_peek(index) {
            Some(abstract_value) => {
                return abstract_value.kind == kind;
            }
            None => return false,
        }
    }
    false
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
    mutability: Mutability,
) -> Result<AbstractState, VMError> {
    let mut state = state.clone();
    state.local_take_borrow(index as usize, mutability)?;
    Ok(state)
}

/// Insert the register value into the locals at `index`
pub fn local_place(state: &AbstractState, index: u8) -> Result<AbstractState, VMError> {
    let mut state = state.clone();
    state.local_place(index as usize)?;
    Ok(state)
}

/// Determine whether an abstract value on the stack and a abstract value in the locals have the
/// same type
pub fn stack_local_polymorphic_eq(state: &AbstractState, index1: usize, index2: usize) -> bool {
    if stack_has(state, index1, None) {
        if let Some((abstract_value, _)) = state.local_get(index2) {
            return state.stack_peek(index1) == Some(abstract_value.clone());
        }
    }
    false
}

/// Determine whether an abstract value on the stack that is a reference points to something of the
/// same type as another abstract value on the stack
pub fn stack_ref_polymorphic_eq(state: &AbstractState, index1: usize, index2: usize) -> bool {
    if stack_has(state, index2, None) {
        if let Some(abstract_value) = state.stack_peek(index1) {
            match abstract_value.token {
                SignatureToken::MutableReference(token) | SignatureToken::Reference(token) => {
                    let abstract_value_inner = AbstractValue {
                        token: (*token).clone(),
                        kind: SignatureTokenView::new(&state.module, &*token).kind(&[]),
                    };
                    return Some(abstract_value_inner) == state.stack_peek(index2);
                }
                _ => return false,
            }
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
    for (i, token_view) in field_token_views.rev().enumerate() {
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
    for _ in struct_def_view.fields().unwrap() {
        state = stack_pop(&state)?;
    }
    Ok(state)
}

/// Construct a struct from abstract values on the stack
/// The struct is stored in the register after creation
pub fn create_struct(
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
    let struct_kind = if struct_def_view.is_nominal_resource() {
        Kind::Resource
    } else {
        tokens
            .iter()
            .map(|token| SignatureTokenView::new(&state.module, token).kind(&[]))
            .fold(Kind::Unrestricted, |acc_kind, next_kind| {
                match (acc_kind, next_kind) {
                    (Kind::All, _) | (_, Kind::All) => Kind::All,
                    (Kind::Resource, _) | (_, Kind::Resource) => Kind::Resource,
                    (Kind::Unrestricted, Kind::Unrestricted) => Kind::Unrestricted,
                }
            })
    };
    let struct_value = AbstractValue::new_struct(
        SignatureToken::Struct(struct_def.struct_handle, tokens),
        struct_kind,
    );
    state.register_set(struct_value);
    Ok(state)
}

/// Determine if a struct (of the given signature) is at the top of the stack
/// The `struct_index` can be `Some(index)` to check for a particular struct,
/// or `None` to just check that there is a a struct.
pub fn stack_has_struct(
    state: &AbstractState,
    struct_index: Option<StructDefinitionIndex>,
) -> bool {
    if state.stack_len() > 0 {
        if let Some(struct_value) = state.stack_peek(0) {
            match struct_value.token {
                SignatureToken::Struct(struct_handle, _) => match struct_index {
                    Some(struct_index) => {
                        let struct_def = state.module.struct_def_at(struct_index);
                        return struct_handle == struct_def.struct_handle;
                    }
                    None => {
                        return true;
                    }
                },
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

pub fn stack_struct_has_field(state: &AbstractState, field_index: FieldDefinitionIndex) -> bool {
    if let Some(struct_handle_index) = state.stack_peek(0).clone().and_then(|abstract_value| {
        SignatureToken::get_struct_handle_from_reference(&abstract_value.token)
    }) {
        return state
            .module
            .is_field_in_struct(field_index, struct_handle_index);
    }
    false
}

/// Push the field at `field_index` of a struct as an `AbstractValue` to the stack
pub fn stack_struct_borrow_field(
    state: &AbstractState,
    field_index: FieldDefinitionIndex,
) -> Result<AbstractState, VMError> {
    let mut state = state.clone();
    state.register_move();
    let field_signature = state.module.get_field_signature(field_index).0.clone();
    let abstract_value = AbstractValue {
        token: SignatureToken::MutableReference(Box::new(field_signature.clone())),
        kind: SignatureTokenView::new(&state.module, &field_signature).kind(&[]),
    };
    state = stack_push(&state, abstract_value)?;
    Ok(state)
}

/// Determine whether the stack has a reference at `index` with the given mutability.
/// If `mutable` is `Either` then the reference can be either mutable or immutable
pub fn stack_has_reference(state: &AbstractState, index: usize, mutability: Mutability) -> bool {
    if state.stack_len() > index {
        if let Some(abstract_value) = state.stack_peek(index) {
            match abstract_value.token {
                SignatureToken::MutableReference(_) => {
                    if mutability == Mutability::Mutable || mutability == Mutability::Either {
                        return true;
                    }
                }
                SignatureToken::Reference(_) => {
                    if mutability == Mutability::Immutable || mutability == Mutability::Either {
                        return true;
                    }
                }
                _ => return false,
            }
        }
    }
    false
}

/// Dereference the value stored in the register. If the value is not a reference, or
/// the register is empty, return an error.
pub fn register_dereference(state: &AbstractState) -> Result<AbstractState, VMError> {
    let mut state = state.clone();
    if let Some(abstract_value) = state.register_move() {
        match abstract_value.token {
            SignatureToken::MutableReference(token) => {
                state.register_set(AbstractValue {
                    token: *token,
                    kind: abstract_value.kind,
                });
                Ok(state)
            }
            SignatureToken::Reference(token) => {
                state.register_set(AbstractValue {
                    token: *token,
                    kind: abstract_value.kind,
                });
                Ok(state)
            }
            _ => Err(VMError::new(
                "Register does not contain a reference".to_string(),
            )),
        }
    } else {
        Err(VMError::new("Register is empty".to_string()))
    }
}

/// Push a reference to a register value with the given mutability.
pub fn stack_push_register_borrow(
    state: &AbstractState,
    mutability: Mutability,
) -> Result<AbstractState, VMError> {
    let mut state = state.clone();
    if let Some(abstract_value) = state.register_move() {
        match mutability {
            Mutability::Mutable => {
                state.stack_push(AbstractValue {
                    token: SignatureToken::MutableReference(Box::new(abstract_value.token)),
                    kind: abstract_value.kind,
                });
                Ok(state)
            }
            Mutability::Immutable => {
                state.stack_push(AbstractValue {
                    token: SignatureToken::Reference(Box::new(abstract_value.token)),
                    kind: abstract_value.kind,
                });
                Ok(state)
            }
            Mutability::Either => Err(VMError::new("Mutability must be specified".to_string())),
        }
    } else {
        Err(VMError::new("Register is empty".to_string()))
    }
}

/// Determine whether the function at the given index can be constructed from the values on
/// the stack.
pub fn stack_satisfies_function_signature(
    state: &AbstractState,
    function_index: FunctionHandleIndex,
) -> bool {
    let state_copy = state.clone();
    let function_handle = state_copy.module.function_handle_at(function_index);
    let function_signature = state_copy
        .module
        .function_signature_at(function_handle.signature);
    let mut satisfied = true;
    for (i, arg_type) in function_signature.arg_types.iter().rev().enumerate() {
        let abstract_value = AbstractValue {
            token: arg_type.clone(),
            kind: SignatureTokenView::new(&state.module, arg_type).kind(&[]),
        };
        if !stack_has(&state, i, Some(abstract_value)) {
            satisfied = false;
        }
    }
    satisfied
}

/// Simulate calling the function at `function_index`
pub fn stack_function_call(
    state: &AbstractState,
    function_index: FunctionHandleIndex,
) -> Result<AbstractState, VMError> {
    let state_copy = state.clone();
    let mut state = state.clone();
    let function_handle = state_copy.module.function_handle_at(function_index);
    let function_signature = state_copy
        .module
        .function_signature_at(function_handle.signature);
    for return_type in function_signature.return_types.iter() {
        let abstract_value = AbstractValue {
            token: return_type.clone(),
            kind: SignatureTokenView::new(&state.module, return_type).kind(&[]),
        };
        state = stack_push(&state, abstract_value)?;
    }
    Ok(state)
}

/// Pop the number of stack values required to call the function
/// at `function_index`
pub fn stack_function_popn(
    state: &AbstractState,
    function_index: FunctionHandleIndex,
) -> Result<AbstractState, VMError> {
    let state_copy = state.clone();
    let mut state = state.clone();
    let function_handle = state_copy.module.function_handle_at(function_index);
    let function_signature = state_copy
        .module
        .function_signature_at(function_handle.signature);
    let number_of_pops = function_signature.arg_types.iter().len();
    for _ in 0..number_of_pops {
        state = stack_pop(&state)?;
    }
    Ok(state)
}

/// Whether the function acquires any global resources or not
pub fn function_can_acquire_resource(state: &AbstractState) -> bool {
    !state.acquires_global_resources.is_empty()
}

/// TODO: This is a temporary function that represents memory
/// safety for a reference. This should be removed and replaced
/// with appropriate memory safety premises when the borrow checking
/// infrastructure is fully implemented.
/// `index` is `Some(i)` if the instruction can be memory safe when operating
/// on non-reference types.
pub fn memory_safe(state: &AbstractState, index: Option<usize>) -> bool {
    match index {
        Some(index) => {
            if stack_has_reference(state, index, Mutability::Either) {
                ALLOW_MEMORY_UNSAFE
            } else {
                true
            }
        }
        None => ALLOW_MEMORY_UNSAFE,
    }
}

/// Wrapper for enclosing the arguments of `stack_has` so that only the `state` needs
/// to be given.
#[macro_export]
macro_rules! state_stack_has {
    ($e1: expr, $e2: expr) => {
        Box::new(move |state| stack_has(state, $e1, $e2))
    };
}

/// Wrapper for enclosing the arguments of `stack_kind_is` so that only the `state` needs
/// to be given.
#[macro_export]
macro_rules! state_stack_kind_is {
    ($e: expr, $a: expr) => {
        Box::new(move |state| stack_kind_is(state, $e, $a))
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

/// Wrapper for enclosing the arguments of `stack_ref_polymorphic_eq` so that only the `state`
/// needs to be given.
#[macro_export]
macro_rules! state_stack_ref_polymorphic_eq {
    ($e1: expr, $e2: expr) => {
        Box::new(move |state| stack_ref_polymorphic_eq(state, $e1, $e2))
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
macro_rules! state_create_struct {
    ($e: expr) => {
        Box::new(move |state| create_struct(state, $e))
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

/// Wrapper for enclosing the arguments of `struct_has_field` so that only the
/// `state` needs to be given.
#[macro_export]
macro_rules! state_stack_struct_has_field {
    ($e: expr) => {
        Box::new(move |state| stack_struct_has_field(state, $e))
    };
}

/// Wrapper for enclosing the arguments of `stack_struct_borrow_field` so that only the
/// `state` needs to be given.
#[macro_export]
macro_rules! state_stack_struct_borrow_field {
    ($e: expr) => {
        Box::new(move |state| stack_struct_borrow_field(state, $e))
    };
}

/// Wrapper for enclosing the arguments of `stack_has_reference` so that only the
/// `state` needs to be given.
#[macro_export]
macro_rules! state_stack_has_reference {
    ($e1: expr, $e2: expr) => {
        Box::new(move |state| stack_has_reference(state, $e1, $e2))
    };
}

/// Wrapper for enclosing the arguments of `register_dereference` so that only the
/// `state` needs to be given.
#[macro_export]
macro_rules! state_register_dereference {
    () => {
        Box::new(move |state| register_dereference(state))
    };
}

/// Wrapper for enclosing the arguments of `stack_push_register_borrow` so that only the
/// `state` needs to be given.
#[macro_export]
macro_rules! state_stack_push_register_borrow {
    ($e: expr) => {
        Box::new(move |state| stack_push_register_borrow(state, $e))
    };
}

/// Wrapper for enclosing the arguments of `stack_satisfies_function_signature` so that only the
/// `state` needs to be given.
#[macro_export]
macro_rules! state_stack_satisfies_function_signature {
    ($e: expr) => {
        Box::new(move |state| stack_satisfies_function_signature(state, $e))
    };
}

/// Wrapper for enclosing the arguments of `stack_function_popn` so that only the
/// `state` needs to be given.
#[macro_export]
macro_rules! state_stack_function_popn {
    ($e: expr) => {
        Box::new(move |state| stack_function_popn(state, $e))
    };
}

/// Wrapper for enclosing the arguments of `stack_function_call` so that only the
/// `state` needs to be given.
#[macro_export]
macro_rules! state_stack_function_call {
    ($e: expr) => {
        Box::new(move |state| stack_function_call(state, $e))
    };
}

/// Wrapper for enclosing the arguments of `function_can_acquire_resource` so that only the
/// `state` needs to be given.
#[macro_export]
macro_rules! state_function_can_acquire_resource {
    () => {
        Box::new(move |state| function_can_acquire_resource(state))
    };
}

/// Wrapper for enclosing the arguments of `memory_safe` so that only the
/// `state` needs to be given.
#[macro_export]
macro_rules! state_memory_safe {
    ($e: expr) => {
        Box::new(move |state| memory_safe(state, $e))
    };
}

/// Predicate that is false for every state.
#[macro_export]
macro_rules! state_never {
    () => {
        Box::new(|_| (false))
    };
}
