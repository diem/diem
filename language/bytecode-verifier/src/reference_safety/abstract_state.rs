// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines the abstract state for the type and memory safety analysis.
use crate::absint::{AbstractDomain, JoinResult};
use borrow_graph::references::RefID;
use libra_types::vm_error::StatusCode;
use mirai_annotations::{checked_postcondition, checked_precondition, checked_verify};
use std::collections::{BTreeMap, BTreeSet};
use vm::{
    access::ModuleAccess,
    errors::{err_at_offset, VMResult},
    file_format::{
        CompiledModule, FieldHandleIndex, FunctionDefinition, LocalIndex, Signature,
        SignatureToken, StructDefinitionIndex,
    },
};

type BorrowGraph = borrow_graph::graph::BorrowGraph<(), Label>;

/// AbstractValue represents a reference or a non reference value, both on the stack and stored
/// in a local
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AbstractValue {
    Reference(RefID),
    NonReference,
}

impl AbstractValue {
    /// checks if self is a reference
    pub fn is_reference(&self) -> bool {
        match self {
            AbstractValue::Reference(_) => true,
            AbstractValue::NonReference => false,
        }
    }

    /// checks if self is a value
    pub fn is_value(&self) -> bool {
        !self.is_reference()
    }

    /// possibly extracts id from self
    pub fn ref_id(&self) -> Option<RefID> {
        match self {
            AbstractValue::Reference(id) => Some(*id),
            AbstractValue::NonReference => None,
        }
    }
}

/// Label is an element of a label on an edge in the borrow graph.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Label {
    Local(LocalIndex),
    Global(StructDefinitionIndex),
    Field(FieldHandleIndex),
}

// Needed for debugging with the borrow graph
impl std::fmt::Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Label::Local(i) => write!(f, "local#{}", i),
            Label::Global(i) => write!(f, "resource@{}", i),
            Label::Field(i) => write!(f, "field#{}", i),
        }
    }
}

/// AbstractState is the analysis state over which abstract interpretation is performed.
#[derive(Clone, Debug, PartialEq)]
pub struct AbstractState {
    locals: BTreeMap<LocalIndex, AbstractValue>,
    borrow_graph: BorrowGraph,
    num_locals: usize,
    next_id: usize,
}

impl AbstractState {
    /// create a new abstract state
    pub fn new(module: &CompiledModule, function_definition: &FunctionDefinition) -> Self {
        let num_locals = module
            .signature_at(
                function_definition
                    .code
                    .as_ref()
                    .expect("Abstract interpreter should only run on non-native functions")
                    .locals,
            )
            .len();
        // ids in [0, num_locals) are reserved for constructing canonical state
        // id at num_locals is reserved for the frame root
        let next_id = num_locals + 1;
        let mut state = AbstractState {
            locals: BTreeMap::new(),
            borrow_graph: BorrowGraph::new(),
            num_locals,
            next_id,
        };

        let func_handle = module.function_handle_at(function_definition.function);
        let arguments = module.signature_at(func_handle.parameters);
        for (arg_idx, argument) in arguments.0.iter().enumerate() {
            let value = if argument.is_reference() {
                let id = state.new_ref(argument.is_mutable_reference());
                AbstractValue::Reference(id)
            } else {
                AbstractValue::NonReference
            };
            state.locals.insert(arg_idx as LocalIndex, value);
        }
        state.borrow_graph.new_ref(state.frame_root(), true);
        state
    }

    /// returns the frame root id
    fn frame_root(&self) -> RefID {
        RefID::new(self.num_locals)
    }

    //**********************************************************************************************
    // Core API
    //**********************************************************************************************

    pub fn value_for(&mut self, s: &SignatureToken) -> AbstractValue {
        match s {
            SignatureToken::Reference(_) => AbstractValue::Reference(self.new_ref(false)),
            SignatureToken::MutableReference(_) => AbstractValue::Reference(self.new_ref(true)),
            _ => AbstractValue::NonReference,
        }
    }

    /// adds and returns new id to borrow graph
    fn new_ref(&mut self, mut_: bool) -> RefID {
        let id = RefID::new(self.next_id);
        self.borrow_graph.new_ref(id, mut_);
        self.next_id += 1;
        id
    }

    fn add_copy(&mut self, parent: RefID, child: RefID) {
        self.borrow_graph.add_strong_borrow((), parent, child)
    }

    fn add_borrow(&mut self, parent: RefID, child: RefID) {
        self.borrow_graph.add_weak_borrow((), parent, child)
    }

    fn add_field_borrow(&mut self, parent: RefID, field: FieldHandleIndex, child: RefID) {
        self.borrow_graph
            .add_strong_field_borrow((), parent, Label::Field(field), child)
    }

    fn add_local_borrow(&mut self, local: LocalIndex, id: RefID) {
        self.borrow_graph
            .add_strong_field_borrow((), self.frame_root(), Label::Local(local), id)
    }

    fn add_resource_borrow(&mut self, resource: StructDefinitionIndex, id: RefID) {
        self.borrow_graph
            .add_weak_field_borrow((), self.frame_root(), Label::Global(resource), id)
    }

    /// removes `id` from borrow graph
    fn release(&mut self, id: RefID) {
        self.borrow_graph.release(id);
    }

    //**********************************************************************************************
    // Core Predicates
    //**********************************************************************************************

    /// checks if `id` is borrowed, but ignores field borrows
    fn has_full_borrows(&self, id: RefID) -> bool {
        let (full_borrows, _field_borrows) = self.borrow_graph.borrowed_by(id);
        !full_borrows.is_empty()
    }

    /// Checks if `id` is borrowed
    /// - All full/epsilon borrows are considered
    /// - Only field borrows the specified label (or all if one isn't specified) are considered
    fn has_consistent_borrows(&self, id: RefID, label_opt: Option<Label>) -> bool {
        let (full_borrows, field_borrows) = self.borrow_graph.borrowed_by(id);
        !full_borrows.is_empty() || {
            match label_opt {
                None => field_borrows.values().any(|borrows| !borrows.is_empty()),
                Some(label) => field_borrows
                    .get(&label)
                    .map(|borrows| !borrows.is_empty())
                    .unwrap_or(false),
            }
        }
    }

    /// Checks if `id` is mutable borrowed
    /// - All full/epsilon mutable borrows are considered
    /// - Only field mutable borrows the specified label (or all if one isn't specified) are
    ///   considered
    fn has_consistent_mutable_borrows(&self, id: RefID, label_opt: Option<Label>) -> bool {
        let (full_borrows, field_borrows) = self.borrow_graph.borrowed_by(id);
        !self.all_immutable(&full_borrows) || {
            match label_opt {
                None => field_borrows
                    .values()
                    .any(|borrows| !self.all_immutable(borrows)),
                Some(label) => field_borrows
                    .get(&label)
                    .map(|borrows| !self.all_immutable(borrows))
                    .unwrap_or(false),
            }
        }
    }

    /// checks if `id` is writable
    /// - Mutable references are freezable if there are no consistent borrows
    /// - Immutable references are not writable by the typing rules
    fn is_writable(&self, id: RefID) -> bool {
        checked_precondition!(self.borrow_graph.is_mutable(id));
        !self.has_consistent_borrows(id, None)
    }

    /// checks if `id` is freezable
    /// - Mutable references are freezable if there are no consistent mutable borrows
    /// - Immutable references are not freezable by the typing rules
    fn is_freezable(&self, id: RefID, at_field_opt: Option<FieldHandleIndex>) -> bool {
        checked_precondition!(self.borrow_graph.is_mutable(id));
        !self.has_consistent_mutable_borrows(id, at_field_opt.map(Label::Field))
    }

    /// checks if `id` is readable
    /// - Mutable references are readable if they are freezable
    /// - Immutable references are always readable
    fn is_readable(&self, id: RefID, at_field_opt: Option<FieldHandleIndex>) -> bool {
        let is_mutable = self.borrow_graph.is_mutable(id);
        !is_mutable || self.is_freezable(id, at_field_opt)
    }

    /// checks if local@idx is borrowed
    fn is_local_borrowed(&self, idx: LocalIndex) -> bool {
        self.has_consistent_borrows(self.frame_root(), Some(Label::Local(idx)))
    }

    /// checks if local@idx is mutably borrowed
    fn is_local_mutably_borrowed(&self, idx: LocalIndex) -> bool {
        self.has_consistent_mutable_borrows(self.frame_root(), Some(Label::Local(idx)))
    }

    /// checks if global@idx is borrowed
    fn is_global_borrowed(&self, resource: StructDefinitionIndex) -> bool {
        self.has_consistent_borrows(self.frame_root(), Some(Label::Global(resource)))
    }

    /// checks if global@idx is mutably borrowed
    fn is_global_mutably_borrowed(&self, resource: StructDefinitionIndex) -> bool {
        self.has_consistent_mutable_borrows(self.frame_root(), Some(Label::Global(resource)))
    }

    /// checks if the stack frame of the function being analyzed can be safely destroyed.
    /// safe destruction requires that all references in locals have already been destroyed
    /// and all values in locals are copyable and unborrowed.
    fn is_frame_safe_to_destroy(&self) -> bool {
        !self.has_consistent_borrows(self.frame_root(), None)
    }

    //**********************************************************************************************
    // Instruction Entry Points
    //**********************************************************************************************

    /// destroys local@idx
    pub fn release_value(&mut self, value: AbstractValue) {
        match value {
            AbstractValue::Reference(id) => self.release(id),
            AbstractValue::NonReference => (),
        }
    }

    pub fn copy_loc(&mut self, offset: usize, local: LocalIndex) -> VMResult<AbstractValue> {
        match self.locals.get(&local).unwrap() {
            AbstractValue::Reference(id) => {
                let id = *id;
                let new_id = self.new_ref(self.borrow_graph.is_mutable(id));
                self.add_copy(id, new_id);
                Ok(AbstractValue::Reference(new_id))
            }
            AbstractValue::NonReference if self.is_local_mutably_borrowed(local) => Err(
                err_at_offset(StatusCode::COPYLOC_EXISTS_BORROW_ERROR, offset),
            ),
            AbstractValue::NonReference => Ok(AbstractValue::NonReference),
        }
    }

    pub fn move_loc(&mut self, offset: usize, local: LocalIndex) -> VMResult<AbstractValue> {
        match self.locals.remove(&local).unwrap() {
            AbstractValue::Reference(id) => Ok(AbstractValue::Reference(id)),
            AbstractValue::NonReference if self.is_local_borrowed(local) => Err(err_at_offset(
                StatusCode::MOVELOC_EXISTS_BORROW_ERROR,
                offset,
            )),
            AbstractValue::NonReference => Ok(AbstractValue::NonReference),
        }
    }

    pub fn st_loc(
        &mut self,
        offset: usize,
        local: LocalIndex,
        new_value: AbstractValue,
    ) -> VMResult<()> {
        let old_value = self.locals.insert(local, new_value);
        match old_value {
            None => Ok(()),
            Some(AbstractValue::Reference(id)) => {
                self.release(id);
                Ok(())
            }
            Some(AbstractValue::NonReference) if self.is_local_borrowed(local) => Err(
                err_at_offset(StatusCode::STLOC_UNSAFE_TO_DESTROY_ERROR, offset),
            ),
            Some(AbstractValue::NonReference) => Ok(()),
        }
    }

    pub fn freeze_ref(&mut self, offset: usize, id: RefID) -> VMResult<AbstractValue> {
        if !self.is_freezable(id, None) {
            return Err(err_at_offset(
                StatusCode::FREEZEREF_EXISTS_MUTABLE_BORROW_ERROR,
                offset,
            ));
        }

        let frozen_id = self.new_ref(false);
        self.add_copy(id, frozen_id);
        self.release(id);
        Ok(AbstractValue::Reference(frozen_id))
    }

    pub fn comparison(
        &mut self,
        offset: usize,
        v1: AbstractValue,
        v2: AbstractValue,
    ) -> VMResult<AbstractValue> {
        match (v1, v2) {
            (AbstractValue::Reference(id1), AbstractValue::Reference(id2))
                if !self.is_readable(id1, None) || !self.is_readable(id2, None) =>
            {
                // TODO better error code
                return Err(err_at_offset(
                    StatusCode::READREF_EXISTS_MUTABLE_BORROW_ERROR,
                    offset,
                ));
            }
            (AbstractValue::Reference(id1), AbstractValue::Reference(id2)) => {
                self.release(id1);
                self.release(id2)
            }
            (v1, v2) => {
                checked_verify!(v1.is_value());
                checked_verify!(v2.is_value());
            }
        }
        Ok(AbstractValue::NonReference)
    }

    pub fn read_ref(&mut self, offset: usize, id: RefID) -> VMResult<AbstractValue> {
        if !self.is_readable(id, None) {
            return Err(err_at_offset(
                StatusCode::READREF_EXISTS_MUTABLE_BORROW_ERROR,
                offset,
            ));
        }

        self.release(id);
        Ok(AbstractValue::NonReference)
    }

    pub fn write_ref(&mut self, offset: usize, id: RefID) -> VMResult<()> {
        if !self.is_writable(id) {
            return Err(err_at_offset(
                StatusCode::WRITEREF_EXISTS_BORROW_ERROR,
                offset,
            ));
        }

        self.release(id);
        Ok(())
    }

    pub fn borrow_loc(
        &mut self,
        offset: usize,
        mut_: bool,
        local: LocalIndex,
    ) -> VMResult<AbstractValue> {
        // nothing to check in case borrow is mutable since the frame cannot have an full borrow/
        // epsilon outgoing edge
        if !mut_ && self.is_local_mutably_borrowed(local) {
            return Err(err_at_offset(
                StatusCode::BORROWLOC_EXISTS_BORROW_ERROR,
                offset,
            ));
        }

        let new_id = self.new_ref(mut_);
        self.add_local_borrow(local, new_id);
        Ok(AbstractValue::Reference(new_id))
    }

    pub fn borrow_field(
        &mut self,
        offset: usize,
        mut_: bool,
        id: RefID,
        field: FieldHandleIndex,
    ) -> VMResult<AbstractValue> {
        // Any field borrows will be factored out, so don't check in the mutable case
        let is_mut_borrow_with_full_borrows = || mut_ && self.has_full_borrows(id);
        // For new immutable borrow, the reference must be readable at that field
        // This means that there could exist a mutable borrow on some other field
        let is_imm_borrow_with_mut_borrows = || !mut_ && !self.is_readable(id, Some(field));

        if is_mut_borrow_with_full_borrows() || is_imm_borrow_with_mut_borrows() {
            // TODO improve error for mutable case
            return Err(err_at_offset(
                StatusCode::BORROWFIELD_EXISTS_MUTABLE_BORROW_ERROR,
                offset,
            ));
        }

        let field_borrow_id = self.new_ref(mut_);
        self.add_field_borrow(id, field, field_borrow_id);
        self.release(id);
        Ok(AbstractValue::Reference(field_borrow_id))
    }

    pub fn borrow_global(
        &mut self,
        offset: usize,
        mut_: bool,
        resource: StructDefinitionIndex,
    ) -> VMResult<AbstractValue> {
        if (mut_ && self.is_global_borrowed(resource)) || self.is_global_mutably_borrowed(resource)
        {
            return Err(err_at_offset(StatusCode::GLOBAL_REFERENCE_ERROR, offset));
        }

        let new_id = self.new_ref(mut_);
        self.add_resource_borrow(resource, new_id);
        Ok(AbstractValue::Reference(new_id))
    }

    pub fn move_from(
        &mut self,
        offset: usize,
        resource: StructDefinitionIndex,
    ) -> VMResult<AbstractValue> {
        if self.is_global_borrowed(resource) {
            Err(err_at_offset(StatusCode::GLOBAL_REFERENCE_ERROR, offset))
        } else {
            Ok(AbstractValue::NonReference)
        }
    }

    pub fn call(
        &mut self,
        offset: usize,
        arguments: Vec<AbstractValue>,
        acquired_resources: &BTreeSet<StructDefinitionIndex>,
        return_: &Signature,
    ) -> VMResult<Vec<AbstractValue>> {
        // Check acquires
        for acquired_resource in acquired_resources {
            if self.is_global_borrowed(*acquired_resource) {
                return Err(err_at_offset(StatusCode::GLOBAL_REFERENCE_ERROR, offset));
            }
        }

        // Check mutable references can be transfered
        let mut all_references_to_borrow_from = BTreeSet::new();
        let mut mutable_references_to_borrow_from = BTreeSet::new();
        for id in arguments.iter().filter_map(|v| v.ref_id()) {
            if self.borrow_graph.is_mutable(id) {
                if !self.is_writable(id) {
                    return Err(err_at_offset(
                        StatusCode::CALL_BORROWED_MUTABLE_REFERENCE_ERROR,
                        offset,
                    ));
                }
                mutable_references_to_borrow_from.insert(id);
            }
            all_references_to_borrow_from.insert(id);
        }

        // Track borrow relationships of return values on inputs
        let return_values = return_
            .0
            .iter()
            .map(|return_type| match return_type {
                SignatureToken::MutableReference(_) => {
                    let id = self.new_ref(true);
                    for parent in &mutable_references_to_borrow_from {
                        self.add_borrow(*parent, id);
                    }
                    AbstractValue::Reference(id)
                }
                SignatureToken::Reference(_) => {
                    let id = self.new_ref(false);
                    for parent in &all_references_to_borrow_from {
                        self.add_borrow(*parent, id);
                    }
                    AbstractValue::Reference(id)
                }
                _ => AbstractValue::NonReference,
            })
            .collect();

        // Release input references
        for id in all_references_to_borrow_from {
            self.release(id)
        }
        Ok(return_values)
    }

    pub fn ret(&mut self, offset: usize, values: Vec<AbstractValue>) -> VMResult<()> {
        // release all local variables
        let mut released = BTreeSet::new();
        for (_local, stored_value) in self.locals.iter() {
            if let AbstractValue::Reference(id) = stored_value {
                released.insert(*id);
            }
        }
        released.into_iter().for_each(|id| self.release(id));

        // Check that no local or global is borrowed
        if !self.is_frame_safe_to_destroy() {
            return Err(err_at_offset(
                StatusCode::UNSAFE_RET_LOCAL_OR_RESOURCE_STILL_BORROWED,
                offset,
            ));
        }

        // Check mutable references can be transfered
        for id in values.into_iter().filter_map(|v| v.ref_id()) {
            if self.borrow_graph.is_mutable(id) && !self.is_writable(id) {
                return Err(err_at_offset(
                    StatusCode::RET_BORROWED_MUTABLE_REFERENCE_ERROR,
                    offset,
                ));
            }
        }
        Ok(())
    }

    //**********************************************************************************************
    // Abstract Interpreter Entry Points
    //**********************************************************************************************

    /// returns the canonical representation of self
    pub fn construct_canonical_state(&self) -> Self {
        let mut id_map = BTreeMap::new();
        id_map.insert(self.frame_root(), self.frame_root());
        let locals = self
            .locals
            .iter()
            .map(|(local, value)| {
                let new_value = match value {
                    AbstractValue::Reference(old_id) => {
                        let new_id = RefID::new(*local as usize);
                        id_map.insert(*old_id, new_id);
                        AbstractValue::Reference(new_id)
                    }
                    AbstractValue::NonReference => AbstractValue::NonReference,
                };
                (*local, new_value)
            })
            .collect::<BTreeMap<_, _>>();
        checked_verify!(self.locals.len() == locals.len());
        let mut borrow_graph = self.borrow_graph.clone();
        borrow_graph.remap_refs(&id_map);
        let canonical_state = AbstractState {
            locals,
            borrow_graph,
            num_locals: self.num_locals,
            next_id: self.num_locals + 1,
        };
        checked_postcondition!(canonical_state.is_canonical());
        canonical_state
    }

    fn all_immutable(&self, borrows: &BTreeMap<RefID, ()>) -> bool {
        !borrows.keys().any(|x| self.borrow_graph.is_mutable(*x))
    }

    fn is_canonical(&self) -> bool {
        self.num_locals + 1 == self.next_id
            && self.locals.iter().all(|(local, value)| {
                value
                    .ref_id()
                    .map(|id| RefID::new(*local as usize) == id)
                    .unwrap_or(true)
            })
    }

    fn iter_locals(&self) -> impl Iterator<Item = LocalIndex> {
        0..self.num_locals as LocalIndex
    }

    pub fn join_(&self, other: &Self) -> Self {
        checked_precondition!(self.is_canonical() && other.is_canonical());
        checked_precondition!(self.next_id == other.next_id);
        checked_precondition!(self.num_locals == other.num_locals);
        let mut locals = BTreeMap::new();
        let mut self_graph = self.borrow_graph.clone();
        let mut other_graph = other.borrow_graph.clone();
        for local in self.iter_locals() {
            let self_value = self.locals.get(&local);
            let other_value = other.locals.get(&local);
            match (self_value, other_value) {
                // Unavailable on both sides, nothing to add
                (None, None) => (),

                (Some(v), None) => {
                    // A reference exists on one side, but not the other. Release
                    if let AbstractValue::Reference(id) = v {
                        self_graph.release(*id);
                    }
                }
                (None, Some(v)) => {
                    // A reference exists on one side, but not the other. Release
                    if let AbstractValue::Reference(id) = v {
                        other_graph.release(*id);
                    }
                }

                // The local has a value on each side, add it to the state
                (Some(v1), Some(v2)) => {
                    checked_verify!(v1 == v2);
                    checked_verify!(!locals.contains_key(&local));
                    locals.insert(local, *v1);
                }
            }
        }

        let borrow_graph = self_graph.join(&other_graph);
        let next_id = self.next_id;
        let num_locals = self.num_locals;

        Self {
            locals,
            borrow_graph,
            next_id,
            num_locals,
        }
    }
}

impl AbstractDomain for AbstractState {
    /// attempts to join state to self and returns the result
    fn join(&mut self, state: &AbstractState) -> JoinResult {
        let joined = Self::join_(self, state);
        checked_verify!(self.num_locals == joined.num_locals);
        let locals_unchanged = self
            .iter_locals()
            .all(|idx| self.locals.get(&idx) == joined.locals.get(&idx));
        let borrow_graph_unchanged = self.borrow_graph.leq(&joined.borrow_graph);
        if locals_unchanged && borrow_graph_unchanged {
            JoinResult::Unchanged
        } else {
            *self = joined;
            JoinResult::Changed
        }
    }
}
