// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines the abstract state for the type and memory safety analysis.
use crate::{
    absint::{AbstractDomain, JoinResult},
    borrow_graph::BorrowGraph,
    nonce::Nonce,
};
use mirai_annotations::{checked_postcondition, checked_precondition, checked_verify};
use std::collections::{BTreeMap, BTreeSet};
use vm::{
    file_format::{
        CompiledModule, FieldDefinitionIndex, Kind, LocalIndex, SignatureToken,
        StructDefinitionIndex,
    },
    views::{FunctionDefinitionView, ViewInternals},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypedAbstractValue {
    pub signature: SignatureToken,
    pub value: AbstractValue,
}

/// AbstractValue represents a value either on the evaluation stack or
/// in a local on a frame of the function stack.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AbstractValue {
    Reference(Nonce),
    Value(Kind),
}

impl AbstractValue {
    /// checks if self is a reference
    pub fn is_reference(&self) -> bool {
        match self {
            AbstractValue::Reference(_) => true,
            AbstractValue::Value(_) => false,
        }
    }

    /// checks if self is a value
    pub fn is_value(&self) -> bool {
        !self.is_reference()
    }

    /// checks if self is a non-resource value
    pub fn is_unrestricted_value(&self) -> bool {
        match self {
            AbstractValue::Reference(_) => false,
            AbstractValue::Value(Kind::Unrestricted) => true,
            AbstractValue::Value(Kind::All) | AbstractValue::Value(Kind::Resource) => false,
        }
    }

    /// possibly extracts nonce from self
    pub fn extract_nonce(&self) -> Option<Nonce> {
        match self {
            AbstractValue::Reference(nonce) => Some(*nonce),
            AbstractValue::Value(_) => None,
        }
    }
}

/// LabelElem is an element of a label on an edge in the borrow graph.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum LabelElem {
    Local(LocalIndex),
    Global(StructDefinitionIndex),
    Field(FieldDefinitionIndex),
}

impl Default for LabelElem {
    fn default() -> Self {
        LabelElem::Local(0)
    }
}

/// AbstractState is the analysis state over which abstract interpretation is performed.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AbstractState {
    locals: BTreeMap<LocalIndex, TypedAbstractValue>,
    borrow_graph: BorrowGraph<LabelElem>,
    frame_root: Nonce,
    next_id: usize,
}

impl AbstractState {
    /// create a new abstract state
    pub fn new(function_definition_view: FunctionDefinitionView<CompiledModule>) -> Self {
        let function_signature_view = function_definition_view.signature();
        let mut locals = BTreeMap::new();
        let mut borrow_graph = BorrowGraph::new();
        for (arg_idx, arg_type_view) in function_signature_view.arg_tokens().enumerate() {
            if arg_type_view.is_reference() {
                let nonce = Nonce::new(arg_idx);
                borrow_graph.add_nonce(nonce);
                locals.insert(
                    arg_idx as LocalIndex,
                    TypedAbstractValue {
                        signature: arg_type_view.as_inner().clone(),
                        value: AbstractValue::Reference(nonce),
                    },
                );
            } else {
                let arg_kind = arg_type_view
                    .kind(&function_definition_view.signature().as_inner().type_formals);
                locals.insert(
                    arg_idx as LocalIndex,
                    TypedAbstractValue {
                        signature: arg_type_view.as_inner().clone(),
                        value: AbstractValue::Value(arg_kind),
                    },
                );
            }
        }
        let frame_root = Nonce::new(function_definition_view.locals_signature().len());
        borrow_graph.add_nonce(frame_root);
        // nonces in [0, frame_root.inner()] are reserved for constructing canonical state
        let next_id = frame_root.inner() + 1;
        AbstractState {
            locals,
            borrow_graph,
            frame_root,
            next_id,
        }
    }

    /// checks if local@idx is available
    pub fn is_available(&self, idx: LocalIndex) -> bool {
        self.locals.contains_key(&idx)
    }

    /// returns local@idx
    pub fn local(&self, idx: LocalIndex) -> &TypedAbstractValue {
        &self.locals[&idx]
    }

    /// removes local@idx
    pub fn remove_local(&mut self, idx: LocalIndex) -> TypedAbstractValue {
        self.locals.remove(&idx).unwrap()
    }

    /// inserts local@idx
    pub fn insert_local(&mut self, idx: LocalIndex, abs_type: TypedAbstractValue) {
        self.locals.insert(idx, abs_type);
    }

    /// checks if a local may be safely destroyed
    pub fn is_local_safe_to_destroy(&self, idx: LocalIndex) -> bool {
        match self.locals[&idx].value {
            AbstractValue::Reference(_) => true,
            AbstractValue::Value(Kind::All) | AbstractValue::Value(Kind::Resource) => false,
            AbstractValue::Value(Kind::Unrestricted) => !self.is_local_borrowed(idx),
        }
    }

    /// checks if the stack frame of the function being analyzed can be safely destroyed.
    /// safe destruction requires that all references in locals have already been destroyed
    /// and all values in locals are unrestricted and unborrowed.
    pub fn is_frame_safe_to_destroy(&self) -> bool {
        self.locals
            .values()
            .all(|x| x.value.is_unrestricted_value())
            && !self.is_nonce_borrowed(self.frame_root)
    }

    /// destroys local@idx
    pub fn destroy_local(&mut self, idx: LocalIndex) {
        checked_precondition!(self.is_local_safe_to_destroy(idx));
        let local = self.locals.remove(&idx).unwrap();
        match local.value {
            AbstractValue::Reference(nonce) => self.remove_nonce(nonce),
            AbstractValue::Value(kind) => {
                checked_verify!(kind == Kind::Unrestricted);
            }
        }
    }

    /// adds new nonce to borrow graph
    pub fn add_nonce(&mut self) -> Nonce {
        let nonce = Nonce::new(self.next_id);
        self.borrow_graph.add_nonce(nonce);
        self.next_id += 1;
        nonce
    }

    /// removes `nonce` from borrow graph
    pub fn remove_nonce(&mut self, nonce: Nonce) {
        self.borrow_graph.remove_nonce(nonce);
    }

    /// checks if `nonce` is borrowed
    pub fn is_nonce_borrowed(&self, nonce: Nonce) -> bool {
        !self.borrow_graph.all_borrows(nonce).is_empty()
    }

    /// checks if a local is borrowed
    pub fn is_local_borrowed(&self, idx: LocalIndex) -> bool {
        !self
            .borrow_graph
            .consistent_borrows(self.frame_root, LabelElem::Local(idx))
            .is_empty()
    }

    /// checks if a global is borrowed
    pub fn is_global_borrowed(&self, idx: StructDefinitionIndex) -> bool {
        !self
            .borrow_graph
            .consistent_borrows(self.frame_root, LabelElem::Global(idx))
            .is_empty()
    }

    /// checks if `nonce` is freezable
    pub fn is_nonce_freezable(&self, nonce: Nonce) -> bool {
        let borrows = self.borrow_graph.all_borrows(nonce);
        self.all_nonces_immutable(borrows)
    }

    /// update self to reflect a borrow of a value global@idx by new_nonce
    pub fn borrow_global_value(&mut self, mut_: bool, idx: StructDefinitionIndex) -> Option<Nonce> {
        if mut_ {
            if self.is_global_borrowed(idx) {
                return None;
            }
        } else {
            let borrowed_nonces = self
                .borrow_graph
                .consistent_borrows(self.frame_root, LabelElem::Global(idx));
            if !self.all_nonces_immutable(borrowed_nonces) {
                return None;
            }
        }
        let new_nonce = self.add_nonce();
        self.borrow_graph
            .add_weak_edge(self.frame_root, vec![LabelElem::Global(idx)], new_nonce);
        Some(new_nonce)
    }

    /// update self to reflect a borrow of field@idx from nonce by new_nonce
    pub fn borrow_field_from_nonce(
        &mut self,
        operand: &TypedAbstractValue,
        mut_: bool,
        idx: FieldDefinitionIndex,
    ) -> Option<Nonce> {
        let nonce = operand.value.extract_nonce().unwrap();
        if mut_ {
            if !self.borrow_graph.nil_borrows(nonce).is_empty() {
                return None;
            }
        } else if operand.signature.is_mutable_reference() {
            let borrowed_nonces = self
                .borrow_graph
                .consistent_borrows(nonce, LabelElem::Field(idx));
            if !self.all_nonces_immutable(borrowed_nonces) {
                return None;
            }
        }
        let new_nonce = self.add_nonce();
        self.borrow_graph
            .add_strong_edge(nonce, vec![LabelElem::Field(idx)], new_nonce);
        Some(new_nonce)
    }

    /// update self to reflect a borrow of a value local@idx by new_nonce
    pub fn borrow_local_value(&mut self, mut_: bool, idx: LocalIndex) -> Option<Nonce> {
        checked_precondition!(self.locals[&idx].value.is_value());
        if !mut_ {
            // nothing to check in case borrow is mutable since the frame cannot have a NIL outgoing edge
            let borrowed_nonces = self
                .borrow_graph
                .consistent_borrows(self.frame_root, LabelElem::Local(idx));
            if !self.all_nonces_immutable(borrowed_nonces) {
                return None;
            }
        }
        let new_nonce = self.add_nonce();
        self.borrow_graph
            .add_strong_edge(self.frame_root, vec![LabelElem::Local(idx)], new_nonce);
        Some(new_nonce)
    }

    /// update self to reflect a borrow of a reference local@idx by new_nonce
    pub fn borrow_from_local_reference(&mut self, idx: LocalIndex) -> Nonce {
        checked_precondition!(self.locals[&idx].value.is_reference());
        let new_nonce = self.add_nonce();
        self.borrow_graph.add_strong_edge(
            self.locals[&idx].value.extract_nonce().unwrap(),
            vec![],
            new_nonce,
        );
        new_nonce
    }

    /// update self to reflect a borrow from each nonce in to_borrow_from by new_nonce
    pub fn borrow_from_nonces(&mut self, to_borrow_from: &BTreeSet<Nonce>) -> Nonce {
        let new_nonce = self.add_nonce();
        for nonce in to_borrow_from {
            self.borrow_graph.add_weak_edge(*nonce, vec![], new_nonce);
        }
        new_nonce
    }

    /// returns the canonical representation of self
    pub fn construct_canonical_state(&self) -> Self {
        let mut new_locals = BTreeMap::new();
        let mut nonce_map = BTreeMap::new();
        for (idx, abs_type) in &self.locals {
            if let AbstractValue::Reference(nonce) = abs_type.value {
                let new_nonce = Nonce::new(*idx as usize);
                new_locals.insert(
                    *idx,
                    TypedAbstractValue {
                        signature: abs_type.signature.clone(),
                        value: AbstractValue::Reference(new_nonce),
                    },
                );
                nonce_map.insert(nonce, new_nonce);
            } else {
                new_locals.insert(*idx, abs_type.clone());
            }
        }
        nonce_map.insert(self.frame_root, self.frame_root);
        let canonical_state = AbstractState {
            locals: new_locals,
            borrow_graph: self.borrow_graph.rename_nonces(nonce_map),
            frame_root: self.frame_root,
            next_id: self.frame_root.inner() + 1,
        };
        checked_postcondition!(canonical_state.is_canonical());
        canonical_state
    }

    fn all_nonces_immutable(&self, borrows: BTreeSet<Nonce>) -> bool {
        !self.locals.values().any(|abs_type| {
            abs_type.signature.is_mutable_reference()
                && borrows.contains(&abs_type.value.extract_nonce().unwrap())
        })
    }

    fn is_canonical(&self) -> bool {
        self.locals.iter().all(|(x, y)| {
            !y.value.is_reference() || Nonce::new(*x as usize) == y.value.extract_nonce().unwrap()
        })
    }

    fn borrowed_value_unavailable(state1: &AbstractState, state2: &AbstractState) -> bool {
        state1.locals.keys().any(|idx| {
            state1.locals[idx].value.is_value()
                && state1.is_local_borrowed(*idx)
                && !state2.locals.contains_key(idx)
        })
    }

    fn split_locals(
        locals: &BTreeMap<LocalIndex, TypedAbstractValue>,
        values: &mut BTreeMap<LocalIndex, Kind>,
        references: &mut BTreeMap<LocalIndex, Nonce>,
    ) {
        for (idx, abs_type) in locals {
            match abs_type.value {
                AbstractValue::Reference(nonce) => {
                    references.insert(idx.clone(), nonce);
                }
                AbstractValue::Value(kind) => {
                    values.insert(idx.clone(), kind);
                }
            }
        }
    }
}

impl AbstractDomain for AbstractState {
    /// attempts to join state to self and returns the result
    fn join(&mut self, state: &AbstractState) -> JoinResult {
        checked_precondition!(self.is_canonical() && state.is_canonical());
        // A join failure occurs in each of the following situations:
        // - a reference or resource is available along one path but not the other
        // - a value is borrowed along one path but unavailable along the other
        if self
            .locals
            .keys()
            .filter(|idx| !self.locals[idx].value.is_unrestricted_value())
            .collect::<BTreeSet<_>>()
            != state
                .locals
                .keys()
                .filter(|idx| !state.locals[idx].value.is_unrestricted_value())
                .collect::<BTreeSet<_>>()
        {
            return JoinResult::Error;
        }
        if Self::borrowed_value_unavailable(self, state)
            || Self::borrowed_value_unavailable(state, self)
        {
            return JoinResult::Error;
        }

        let mut values1 = BTreeMap::new();
        let mut references1 = BTreeMap::new();
        Self::split_locals(&self.locals, &mut values1, &mut references1);
        let mut values2 = BTreeMap::new();
        let mut references2 = BTreeMap::new();
        Self::split_locals(&state.locals, &mut values2, &mut references2);
        checked_verify!(references1 == references2);

        let mut locals = BTreeMap::new();
        for (idx, nonce) in &references1 {
            locals.insert(
                idx.clone(),
                TypedAbstractValue {
                    signature: self.locals[idx].signature.clone(),
                    value: AbstractValue::Reference(*nonce),
                },
            );
        }
        for (idx, kind1) in &values1 {
            if let Some(kind2) = values2.get(idx) {
                checked_verify!(kind1 == kind2);
                locals.insert(
                    idx.clone(),
                    TypedAbstractValue {
                        signature: self.locals[idx].signature.clone(),
                        value: AbstractValue::Value(*kind1),
                    },
                );
            }
        }

        let locals_unchanged = self.locals.keys().all(|idx| locals.contains_key(idx));
        let borrow_graph_unchanged = self.borrow_graph.abstracts(&state.borrow_graph);
        if locals_unchanged && borrow_graph_unchanged {
            JoinResult::Unchanged
        } else {
            self.locals = locals;
            self.borrow_graph.join(&state.borrow_graph);
            JoinResult::Changed
        }
    }
}
