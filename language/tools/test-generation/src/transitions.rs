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
        FieldDefinitionIndex, FunctionHandleIndex, Kind, LocalsSignatureIndex, SignatureToken,
        StructDefinitionIndex,
    },
    views::{FunctionHandleView, SignatureTokenView, StructDefinitionView, ViewInternals},
};

use std::collections::HashMap;

//---------------------------------------------------------------------------
// Type Instantiations from Unification with the Abstract Stack
//---------------------------------------------------------------------------

/// A substitution is a mapping from type formal index to the `SignatureToken` representing the
/// type instantiation for that index.
pub struct Subst {
    pub subst: HashMap<usize, SignatureToken>,
}

impl Subst {
    pub fn new() -> Self {
        Self {
            subst: HashMap::new(),
        }
    }

    /// NB that the position of arguments here matters. We can build a substitution if the `instr_sig`
    /// is a type parameter, and the `stack_sig` is a concrete type. But, if the instruction signature is a
    /// concrete type, but the stack signature is a type parameter, they cannot unify and no
    /// substitution is created.
    pub fn check_and_add(
        &mut self,
        state: &AbstractState,
        stack_sig: SignatureToken,
        instr_sig: SignatureToken,
    ) -> bool {
        match (stack_sig, instr_sig) {
            (tok, SignatureToken::TypeParameter(idx)) => {
                if let Some(other_type) = self.subst.get(&(idx as usize)).cloned() {
                    // If we have already defined a subtitution for this type parameter, then make
                    // sure the signature token on the stack is amenable with the type selection.
                    tok == other_type
                } else {
                    // Otherwise record that the type parameter maps to this signature token.
                    self.subst.insert(idx as usize, tok);
                    true
                }
            }
            // A type parameter on the stack _cannot_ be unified with a non type parameter. But
            // that case has already been taken care of above. This case is added for explicitness,
            // but it could be rolled into the catch-all at the bottom of this match.
            (SignatureToken::TypeParameter(_), _) => false,
            // Build a substitution from recursing into structs
            (SignatureToken::Struct(sig1, params1), SignatureToken::Struct(sig2, params2)) => {
                if sig1 != sig2 {
                    return false;
                }
                assert!(params1.len() == params2.len());
                for (s1, s2) in params1.into_iter().zip(params2.into_iter()) {
                    if !self.check_and_add(state, s1, s2) {
                        return false;
                    }
                }
                true
            }
            (x, y) => x == y,
        }
    }

    /// Return the instantiation from the substitution that has been built.
    pub fn instantiation(self) -> Vec<SignatureToken> {
        let mut vec = self.subst.into_iter().collect::<Vec<_>>();
        vec.sort_by(|a, b| a.0.cmp(&b.0));
        vec.into_iter().map(|x| x.1).collect()
    }
}

//---------------------------------------------------------------------------
// Kind Operations
//---------------------------------------------------------------------------

/// Given a signature token, returns the kind of this token in the module context, and kind
/// instantiation for the function.
pub fn kind_for_token(state: &AbstractState, token: &SignatureToken, kinds: &[Kind]) -> Kind {
    let context = (&state.module.module.struct_handles()[..], kinds);
    SignatureToken::kind(context, token)
}

/// Given a locals signature index, determine the kind for each signature token. Restricted for
/// determining kinds at the top-level only. This is reflected in the use of
/// `state.instantiation[..]` as the kind context.
pub fn kinds_for_instantiation(
    state: &AbstractState,
    instantiation: &[SignatureToken],
) -> Vec<Kind> {
    instantiation
        .iter()
        .map(|token| {
            let context = (
                &state.module.module.struct_handles()[..],
                &state.instantiation[..],
            );
            SignatureToken::kind(context, token)
        })
        .collect()
}

/// Determine whether the stack contains an integer value at given index.
pub fn stack_has_integer(state: &AbstractState, index: usize) -> bool {
    index < state.stack_len()
        && match state.stack_peek(index) {
            Some(AbstractValue { token, .. }) => token.is_integer(),
            None => false,
        }
}

pub fn stack_top_is_castable_to(state: &AbstractState, typ: SignatureToken) -> bool {
    stack_has_integer(state, 0)
        && match typ {
            SignatureToken::U8 => stack_has(
                state,
                0,
                Some(AbstractValue::new_primitive(SignatureToken::U8)),
            ),
            SignatureToken::U64 => {
                stack_has(
                    state,
                    0,
                    Some(AbstractValue::new_primitive(SignatureToken::U8)),
                ) || stack_has(
                    state,
                    0,
                    Some(AbstractValue::new_primitive(SignatureToken::U64)),
                )
            }
            SignatureToken::U128 => true,
            _ => false,
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

/// Determine if the abstract value at `index` has a kind that is a subkind of the kind for the
/// instruction kind. e.g. if the instruction takes a type of kind `All` then it is OK to fit in a
/// value with a type of kind `Unrestricted`.
pub fn stack_kind_is_subkind(state: &AbstractState, index: usize, instruction_kind: Kind) -> bool {
    if !stack_has(state, index, None) {
        return false;
    }
    let stack_value = state.stack_peek(index).unwrap();
    stack_value.kind.is_sub_kind_of(instruction_kind)
}

/// Check whether the local at `index` is of the given kind
pub fn local_kind_is(state: &AbstractState, index: u8, kind: Kind) -> bool {
    state
        .local_kind_is(index as usize, kind)
        .unwrap_or_else(|_| false)
}

//---------------------------------------------------------------------------
// Stack & Local Predicates
//---------------------------------------------------------------------------

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

/// Determine whether an abstract value on the stack that is a reference points to something of the
/// same type as another abstract value on the stack
pub fn stack_ref_polymorphic_eq(state: &AbstractState, index1: usize, index2: usize) -> bool {
    if stack_has(state, index2, None) {
        if let Some(abstract_value) = state.stack_peek(index1) {
            match abstract_value.token {
                SignatureToken::MutableReference(token) | SignatureToken::Reference(token) => {
                    let abstract_value_inner = AbstractValue {
                        token: (*token).clone(),
                        kind: kind_for_token(&state, &*token, &state.instantiation[..]),
                    };
                    return Some(abstract_value_inner) == state.stack_peek(index2);
                }
                _ => return false,
            }
        }
    }
    false
}

//---------------------------------------------------------------------------
// Stack and Local Operations
//---------------------------------------------------------------------------

/// Pop from the top of the stack.
pub fn stack_pop(state: &AbstractState) -> Result<AbstractState, VMError> {
    let mut state = state.clone();
    state.stack_pop()?;
    Ok(state)
}

pub enum StackBinOpResult {
    Left,
    Right,
    Other(AbstractValue),
}

/// Perform a binary operation using the top two values on the stack as operands.
pub fn stack_bin_op(
    state: &AbstractState,
    res: StackBinOpResult,
) -> Result<AbstractState, VMError> {
    let mut state = state.clone();
    let right = {
        state.stack_pop()?;
        state.register_move().unwrap()
    };
    let left = {
        state.stack_pop()?;
        state.register_move().unwrap()
    };
    state.stack_push(match res {
        StackBinOpResult::Left => left,
        StackBinOpResult::Right => right,
        StackBinOpResult::Other(val) => val,
    });
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

//---------------------------------------------------------------------------
// Struct Predicates and Operations
//---------------------------------------------------------------------------

/// Determine whether the struct at the given index can be constructed from the values on
/// the stack.
/// Note that this function is bidirectional; if there is an instantiation, we check it. Otherwise,
/// we infer the types that are needed.
pub fn stack_satisfies_struct_signature(
    state: &AbstractState,
    struct_index: StructDefinitionIndex,
    instantiation: Option<LocalsSignatureIndex>,
) -> (bool, Subst) {
    let instantiation = instantiation.map(|index| state.module.instantiantiation_at(index));
    let struct_def = state.module.module.struct_def_at(struct_index);
    let struct_def = StructDefinitionView::new(&state.module.module, struct_def);
    // Get the type formals for the struct, and the kinds that they expect.
    let type_formals = struct_def.type_formals();
    let field_token_views = struct_def
        .fields()
        .into_iter()
        .flatten()
        .map(|field| field.type_signature().token());
    let mut satisfied = true;
    let mut substitution = Subst::new();
    for (i, token_view) in field_token_views.rev().enumerate() {
        let ty = if let Some(subst) = &instantiation {
            token_view.as_inner().substitute(subst)
        } else {
            token_view.as_inner().clone()
        };
        let has = if let SignatureToken::TypeParameter(idx) = &ty {
            if stack_kind_is_subkind(state, i, type_formals[*idx as usize]) {
                let stack_tok = state.stack_peek(i).unwrap();
                substitution.check_and_add(state, stack_tok.token, ty)
            } else {
                false
            }
        } else {
            let abstract_value = AbstractValue {
                token: ty,
                kind: token_view.kind(type_formals),
            };
            stack_has(state, i, Some(abstract_value))
        };

        if !has {
            satisfied = false;
        }
    }
    (satisfied, substitution)
}

pub fn get_struct_instantiation_for_state(
    state: &AbstractState,
    struct_index: StructDefinitionIndex,
    instantiation: Option<LocalsSignatureIndex>,
) -> Vec<SignatureToken> {
    if let Some(index) = instantiation {
        return state.module.instantiantiation_at(index).clone();
    }
    let mut partial_instantiation =
        stack_satisfies_struct_signature(state, struct_index, instantiation).1;
    let struct_def = state.module.module.struct_def_at(struct_index);
    let struct_def = StructDefinitionView::new(&state.module.module, struct_def);
    let typs = struct_def.type_formals();
    for (index, kind) in typs.iter().enumerate() {
        if !partial_instantiation.subst.contains_key(&index) {
            match kind {
                Kind::All | Kind::Unrestricted => {
                    partial_instantiation
                        .subst
                        .insert(index, SignatureToken::U64);
                }
                Kind::Resource => {
                    unimplemented!("[Struct Instantiation] Need to fill in resource type params");
                }
            }
        }
    }
    partial_instantiation.instantiation()
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
                        let struct_def = state.module.module.struct_def_at(struct_index);
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
    let struct_def = state.module.module.struct_def_at(struct_index);
    StructDefinitionView::new(&state.module.module, struct_def).is_nominal_resource()
}

pub fn stack_struct_has_field(state: &AbstractState, field_index: FieldDefinitionIndex) -> bool {
    if let Some(struct_handle_index) = state.stack_peek(0).and_then(|abstract_value| {
        SignatureToken::get_struct_handle_from_reference(&abstract_value.token)
    }) {
        return state
            .module
            .module
            .is_field_in_struct(field_index, struct_handle_index);
    }
    false
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

/// Pop the number of stack values required to construct the struct
/// at `struct_index`
pub fn stack_struct_popn(
    state: &AbstractState,
    struct_index: StructDefinitionIndex,
) -> Result<AbstractState, VMError> {
    let state_copy = state.clone();
    let mut state = state.clone();
    let struct_def = state_copy.module.module.struct_def_at(struct_index);
    let struct_def_view = StructDefinitionView::new(&state_copy.module.module, struct_def);
    for _ in struct_def_view.fields().unwrap() {
        state.stack_pop()?;
    }
    Ok(state)
}

/// Construct a struct from abstract values on the stack
/// The struct is stored in the register after creation
pub fn create_struct(
    state: &AbstractState,
    struct_index: StructDefinitionIndex,
    instantiation: LocalsSignatureIndex,
) -> Result<AbstractState, VMError> {
    let state_copy = state.clone();
    let mut state = state.clone();
    let struct_def = state_copy.module.module.struct_def_at(struct_index);
    // Get the type, and kind of this struct
    let ty_instantiation = state.module.instantiantiation_at(instantiation);
    let sig_tok = SignatureToken::Struct(struct_def.struct_handle, ty_instantiation.clone());
    let struct_kind = kind_for_token(&state, &sig_tok, &state.instantiation[..]);
    let struct_value = AbstractValue::new_struct(sig_tok, struct_kind);
    state.register_set(struct_value);
    Ok(state)
}

pub fn stack_unpack_struct_instantiation(state: &AbstractState) -> Vec<SignatureToken> {
    if let Some(av) = state.stack_peek(0) {
        match av.token {
            SignatureToken::Struct(_, toks) => toks,
            _ => panic!("Invalid unpack -- non-struct value found at top of stack"),
        }
    } else {
        panic!("Invalid unpack -- precondition not satisfied");
    }
}

/// Push the fields of a struct as `AbstractValue`s to the stack
pub fn stack_unpack_struct(
    state: &AbstractState,
    struct_index: StructDefinitionIndex,
    instantiation: LocalsSignatureIndex,
) -> Result<AbstractState, VMError> {
    let state_copy = state.clone();
    let mut state = state.clone();
    let ty_instantiation = state.module.instantiantiation_at(instantiation).clone();
    let kinds = kinds_for_instantiation(&state_copy, &ty_instantiation);
    let struct_def = state_copy.module.module.struct_def_at(struct_index);
    let struct_def_view = StructDefinitionView::new(&state_copy.module.module, struct_def);
    let token_views = struct_def_view
        .fields()
        .into_iter()
        .flatten()
        .map(|field| field.type_signature().token());
    for token_view in token_views {
        let abstract_value = AbstractValue {
            token: token_view.as_inner().substitute(&ty_instantiation),
            kind: kind_for_token(&state, &token_view.as_inner(), &kinds),
        };
        state = stack_push(&state, abstract_value)?;
    }
    Ok(state)
}

pub fn struct_ref_instantiation(state: &mut AbstractState) -> Result<Vec<SignatureToken>, VMError> {
    let token = state.register_move().unwrap().token;
    if let Some(type_actuals) = token.get_type_actuals_from_reference() {
        Ok(type_actuals.to_vec())
    } else {
        Err(VMError::new("Invalid field borrow".to_string()))
    }
}

/// Push the field at `field_index` of a struct as an `AbstractValue` to the stack
pub fn stack_struct_borrow_field(
    state: &AbstractState,
    field_index: FieldDefinitionIndex,
) -> Result<AbstractState, VMError> {
    let mut state = state.clone();
    let typs = struct_ref_instantiation(&mut state)?;
    let kinds = kinds_for_instantiation(&state, &typs);
    let field_signature = state
        .module
        .module
        .get_field_signature(field_index)
        .0
        .clone();
    let reified_field_sig = field_signature.substitute(&typs);
    // NB: We determine the kind on the non-reified_field_sig; we want any local references to
    // type parameters to point to (struct) local type parameters. We could possibly also use the
    // reified_field_sig coupled with the top-level instantiation, but I need to convince myself of
    // the correctness of this.
    let abstract_value = AbstractValue {
        token: SignatureToken::MutableReference(Box::new(reified_field_sig)),
        kind: kind_for_token(&state, &field_signature, &kinds),
    };
    state = stack_push(&state, abstract_value)?;
    Ok(state)
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
        println!("{:?}", state);
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

//---------------------------------------------------------------------------
// Function Call Predicates and Operations
//---------------------------------------------------------------------------

/// Determine whether the function at the given index can be constructed from the values on
/// the stack.
pub fn stack_satisfies_function_signature(
    state: &AbstractState,
    function_index: FunctionHandleIndex,
) -> (bool, Subst) {
    let state_copy = state.clone();
    let function_handle = state_copy.module.module.function_handle_at(function_index);
    let function_signature = state_copy
        .module
        .module
        .function_signature_at(function_handle.signature);
    let type_formals = &function_signature.type_formals;
    let mut satisfied = true;
    let mut substitution = Subst::new();
    for (i, arg_type) in function_signature.arg_types.iter().rev().enumerate() {
        let has = if let SignatureToken::TypeParameter(idx) = arg_type {
            if stack_kind_is_subkind(state, i, type_formals[*idx as usize]) {
                let stack_tok = state.stack_peek(i).unwrap();
                substitution.check_and_add(state, stack_tok.token, arg_type.clone())
            } else {
                false
            }
        } else {
            let kind = SignatureTokenView::new(&state.module.module, arg_type).kind(type_formals);
            let abstract_value = AbstractValue {
                token: arg_type.clone(),
                kind,
            };
            stack_has(&state, i, Some(abstract_value))
        };
        if !has {
            satisfied = false;
        }
    }
    (satisfied, substitution)
}

/// Whether the function acquires any global resources or not
pub fn function_can_acquire_resource(state: &AbstractState) -> bool {
    !state.acquires_global_resources.is_empty()
}

/// Simulate calling the function at `function_index`
pub fn stack_function_call(
    state: &AbstractState,
    function_index: FunctionHandleIndex,
    instantiation: LocalsSignatureIndex,
) -> Result<AbstractState, VMError> {
    let state_copy = state.clone();
    let mut state = state.clone();
    let function_handle = state_copy.module.module.function_handle_at(function_index);
    let function_signature = state_copy
        .module
        .module
        .function_signature_at(function_handle.signature);
    let ty_instantiation = state_copy.module.instantiantiation_at(instantiation);
    let kinds = kinds_for_instantiation(&state_copy, &ty_instantiation);
    for return_type in function_signature.return_types.iter() {
        let abstract_value = AbstractValue {
            token: return_type.substitute(&ty_instantiation),
            kind: kind_for_token(&state, return_type, &kinds),
        };
        state = stack_push(&state, abstract_value)?;
    }
    Ok(state)
}

pub fn get_function_instantiation_for_state(
    state: &AbstractState,
    function_index: FunctionHandleIndex,
) -> Vec<SignatureToken> {
    let mut partial_instantiation = stack_satisfies_function_signature(state, function_index).1;
    let function_handle = state.module.module.function_handle_at(function_index);
    let function_handle = FunctionHandleView::new(&state.module.module, function_handle);
    let signature = function_handle.signature();
    let typs = signature.type_formals();
    for (index, kind) in typs.iter().enumerate() {
        if !partial_instantiation.subst.contains_key(&index) {
            match kind {
                Kind::All | Kind::Unrestricted => {
                    partial_instantiation
                        .subst
                        .insert(index, SignatureToken::U64);
                }
                Kind::Resource => {
                    unimplemented!("[Struct Instantiation] Need to fill in resource type params");
                }
            }
        }
    }
    partial_instantiation.instantiation()
}

/// Pop the number of stack values required to call the function
/// at `function_index`
pub fn stack_function_popn(
    state: &AbstractState,
    function_index: FunctionHandleIndex,
) -> Result<AbstractState, VMError> {
    let state_copy = state.clone();
    let mut state = state.clone();
    let function_handle = state_copy.module.module.function_handle_at(function_index);
    let function_signature = state_copy
        .module
        .module
        .function_signature_at(function_handle.signature);
    let number_of_pops = function_signature.arg_types.iter().len();
    for _ in 0..number_of_pops {
        state.stack_pop()?;
    }
    Ok(state)
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

//---------------------------------------------------------------------------
// Macros
//---------------------------------------------------------------------------

/// Wrapper for enclosing the arguments of `stack_has` so that only the `state` needs
/// to be given.
#[macro_export]
macro_rules! state_stack_has {
    ($e1: expr, $e2: expr) => {
        Box::new(move |state| stack_has(state, $e1, $e2))
    };
}

/// Determines if the type at the top of the abstract stack is castable to the given type.
#[macro_export]
macro_rules! state_stack_is_castable {
    ($e1: expr) => {
        Box::new(move |state| stack_top_is_castable_to(state, $e1))
    };
}

/// Wrapper for enclosing the arguments of `stack_has_integer` so that only the `state` needs
/// to be given.
#[macro_export]
macro_rules! state_stack_has_integer {
    ($e1: expr) => {
        Box::new(move |state| stack_has_integer(state, $e1))
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
    ($e: expr, $is_exact: expr, $exact_instantiation: expr) => {
        Box::new(move |state| {
            if $is_exact {
                stack_satisfies_struct_signature(state, $e, Some($exact_instantiation)).0
            } else {
                stack_satisfies_struct_signature(state, $e, None).0
            }
        })
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
    ($e1: expr, $e2: expr) => {
        Box::new(move |state| create_struct(state, $e1, $e2))
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
    ($e: expr, $instantiation: expr) => {
        Box::new(move |state| stack_unpack_struct(state, $e, $instantiation))
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
        Box::new(move |state| stack_satisfies_function_signature(state, $e).0)
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
    ($e: expr, $instantiation: expr) => {
        Box::new(move |state| stack_function_call(state, $e, $instantiation))
    };
}

/// Determine the proper type instantiation for function call in the current state.
#[macro_export]
macro_rules! function_instantiation_for_state {
    ($e: expr) => {
        Box::new(move |state| get_function_instantiation_for_state(state, $e))
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

#[macro_export]
macro_rules! state_stack_bin_op {
    (#left) => {
        Box::new(move |state| stack_bin_op(state, crate::transitions::StackBinOpResult::Left))
    };
    (#right) => {
        Box::new(move |state| stack_bin_op(state, crate::transitions::StackBinOpResult::Right))
    };
    () => {
        state_stack_bin_op!(#left);
    };
    ($e: expr) => {
        Box::new(move |state| stack_bin_op(state, crate::transitions::StackBinOpResult::Other($e)))
    }
}

/// Predicate that is false for every state, unless control operations are allowed.
#[macro_export]
macro_rules! state_control_flow {
    () => {
        Box::new(|state| state.is_control_flow_allowed())
    };
}

/// Determine the proper type instantiation for struct in the current state.
#[macro_export]
macro_rules! struct_instantiation_for_state {
    ($e: expr, $is_exact: expr, $exact_instantiation: expr) => {
        Box::new(move |state| {
            if $is_exact {
                get_struct_instantiation_for_state(state, $e, Some($exact_instantiation))
            } else {
                get_struct_instantiation_for_state(state, $e, None)
            }
        })
    };
}

/// Determine the proper type instantiation for struct in the current state.
#[macro_export]
macro_rules! unpack_instantiation_for_state {
    () => {
        Box::new(move |state| stack_unpack_struct_instantiation(state))
    };
}

/// A wrapper around type instantiation, that allows specifying an "exact" instantiation index, or
/// if the instantiation should be inferred from the current state.
#[macro_export]
macro_rules! with_ty_param {
    (($is_exact: expr, $exact_instantiation: expr) => $ty:ident, $body:expr) => {
        Box::new(move |$ty| {
            let $ty = if $is_exact { $exact_instantiation } else { $ty };
            $body
        })
    };
}
