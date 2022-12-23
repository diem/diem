// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//**************************************************************************************************
// Abstract state
//**************************************************************************************************

use crate::{
    cfgir::absint::*,
    diag,
    diagnostics::{
        codes::{DiagnosticCode, ReferenceSafety},
        Diagnostics,
    },
    hlir::{
        ast::{TypeName_, *},
        translate::{display_var, DisplayVar},
    },
    parser::ast::{Field, StructName, Var},
    shared::*,
};
use move_ir_types::location::*;
use move_symbol_pool::Symbol;

use crate::shared::unique_map::UniqueMap;
use borrow_graph::references::RefID;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
enum Label {
    Local(Symbol),
    Resource(Symbol),
    Field(Symbol),
}

type BorrowGraph = borrow_graph::graph::BorrowGraph<Loc, Label>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Value {
    NonRef,
    Ref(RefID),
}
pub type Values = Vec<Value>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BorrowState {
    locals: UniqueMap<Var, Value>,
    acquired_resources: BTreeMap<StructName, Loc>,
    borrows: BorrowGraph,
    next_id: usize,
    // true if the previous pass had errors
    prev_had_errors: bool,
}

//**************************************************************************************************
// impls
//**************************************************************************************************

pub fn assert_single_value(mut values: Values) -> Value {
    assert!(values.len() == 1);
    values.pop().unwrap()
}

impl Value {
    pub fn is_ref(&self) -> bool {
        match self {
            Value::Ref(_) => true,
            Value::NonRef => false,
        }
    }

    pub fn as_vref(&self) -> Option<RefID> {
        match self {
            Value::Ref(id) => Some(*id),
            Value::NonRef => None,
        }
    }

    fn remap_refs(&mut self, id_map: &BTreeMap<RefID, RefID>) {
        match self {
            Value::Ref(id) if id_map.contains_key(id) => *id = id_map[id],
            _ => (),
        }
    }
}

impl BorrowState {
    pub fn initial<T>(
        locals: &UniqueMap<Var, T>,
        acquired_resources: BTreeMap<StructName, Loc>,
        prev_had_errors: bool,
    ) -> Self {
        let mut new_state = BorrowState {
            locals: locals.ref_map(|_, _| Value::NonRef),
            borrows: BorrowGraph::new(),
            next_id: locals.len() + 1,
            acquired_resources,
            prev_had_errors,
        };
        new_state.borrows.new_ref(Self::LOCAL_ROOT, true);
        new_state
    }

    fn borrow_error<F: Fn() -> String>(
        borrows: &BorrowGraph,
        loc: Loc,
        full_borrows: &BTreeMap<RefID, Loc>,
        field_borrows: &BTreeMap<Label, BTreeMap<RefID, Loc>>,
        code: impl DiagnosticCode,
        msg: F,
    ) -> Diagnostics {
        if full_borrows.is_empty() && field_borrows.is_empty() {
            return Diagnostics::new();
        }

        let mut_adj = |id| {
            if borrows.is_mutable(id) {
                "mutably "
            } else {
                ""
            }
        };
        let mut diag = diag!(code, (loc, msg()));
        for (borrower, rloc) in full_borrows {
            let adj = mut_adj(*borrower);
            diag.add_secondary_label((
                *rloc,
                format!("It is still being {}borrowed by this reference", adj),
            ))
        }
        for (field_lbl, borrowers) in field_borrows {
            for (borrower, rloc) in borrowers {
                let adj = mut_adj(*borrower);
                let field = match field_lbl {
                    Label::Field(f) => f,
                    Label::Local(_) | Label::Resource(_) => panic!(
                        "ICE local/resource should not be field borrows as they only exist from \
                         the virtual 'root' reference"
                    ),
                };
                diag.add_secondary_label((
                    *rloc,
                    format!(
                        "Field '{}' is still being {}borrowed by this reference",
                        field, adj
                    ),
                ))
            }
        }
        assert!(diag.secondary_labels_len() >= 1);
        Diagnostics::from(vec![diag])
    }

    const LOCAL_ROOT: RefID = RefID::new(0);

    fn field_label(field: &Field) -> Label {
        Label::Field(field.value().to_owned())
    }

    fn local_label(local: &Var) -> Label {
        Label::Local(local.value().to_owned())
    }

    fn resource_label(resource: &StructName) -> Label {
        Label::Resource(resource.value().to_owned())
    }

    //**********************************************************************************************
    // Core API
    //**********************************************************************************************

    fn single_type_value(&mut self, s: &SingleType) -> Value {
        match &s.value {
            SingleType_::Base(_) => Value::NonRef,
            SingleType_::Ref(mut_, _) => Value::Ref(self.declare_new_ref(*mut_)),
        }
    }

    fn declare_new_ref(&mut self, mut_: bool) -> RefID {
        fn new_id(next: &mut usize) -> RefID {
            *next += 1;
            RefID::new(*next)
        }

        let id = new_id(&mut self.next_id);
        self.borrows.new_ref(id, mut_);
        id
    }

    fn add_copy(&mut self, loc: Loc, parent: RefID, child: RefID) {
        self.borrows.add_strong_borrow(loc, parent, child)
    }

    fn add_borrow(&mut self, loc: Loc, parent: RefID, child: RefID) {
        self.borrows.add_weak_borrow(loc, parent, child)
    }

    fn add_field_borrow(&mut self, loc: Loc, parent: RefID, field: Field, child: RefID) {
        self.borrows
            .add_strong_field_borrow(loc, parent, Self::field_label(&field), child)
    }

    fn add_local_borrow(&mut self, loc: Loc, local: &Var, id: RefID) {
        self.borrows
            .add_strong_field_borrow(loc, Self::LOCAL_ROOT, Self::local_label(local), id)
    }

    fn add_resource_borrow(&mut self, loc: Loc, resource: &StructName, id: RefID) {
        self.borrows.add_weak_field_borrow(
            loc,
            Self::LOCAL_ROOT,
            Self::resource_label(resource),
            id,
        )
    }

    fn writable<F: Fn() -> String>(&self, loc: Loc, msg: F, id: RefID) -> Diagnostics {
        assert!(self.borrows.is_mutable(id), "ICE type checking failed");
        let (full_borrows, field_borrows) = self.borrows.borrowed_by(id);
        Self::borrow_error(
            &self.borrows,
            loc,
            &full_borrows,
            &field_borrows,
            ReferenceSafety::Dangling,
            msg,
        )
    }

    fn freezable<F: Fn() -> String>(
        &self,
        loc: Loc,
        code: impl DiagnosticCode,
        msg: F,
        id: RefID,
        at_field_opt: Option<&Field>,
    ) -> Diagnostics {
        assert!(self.borrows.is_mutable(id), "ICE type checking failed");
        let (full_borrows, field_borrows) = self.borrows.borrowed_by(id);
        let mut_filter_set = |s: BTreeMap<RefID, Loc>| {
            s.into_iter()
                .filter(|(id, _loc)| self.borrows.is_mutable(*id))
                .collect::<BTreeMap<_, _>>()
        };
        let mut_full_borrows = mut_filter_set(full_borrows);
        let mut_field_borrows = field_borrows
            .into_iter()
            .filter_map(|(f, borrowers)| {
                match (at_field_opt, &f) {
                    // Borrow at the same field, so keep
                    (Some(at_field), Label::Field(f_)) if *f_ == at_field.value() => (),
                    // Borrow not at the same field, so skip
                    (Some(_at_field), _) => return None,
                    // Not freezing at a field, so consider any field borrows
                    (None, _) => (),
                }
                let borrowers = mut_filter_set(borrowers);
                if borrowers.is_empty() {
                    None
                } else {
                    Some((f, borrowers))
                }
            })
            .collect();
        Self::borrow_error(
            &self.borrows,
            loc,
            &mut_full_borrows,
            &mut_field_borrows,
            code,
            msg,
        )
    }

    fn readable<F: Fn() -> String>(
        &self,
        loc: Loc,
        code: impl DiagnosticCode,
        msg: F,
        id: RefID,
        at_field_opt: Option<&Field>,
    ) -> Diagnostics {
        let is_mutable = self.borrows.is_mutable(id);
        if is_mutable {
            self.freezable(loc, code, msg, id, at_field_opt)
        } else {
            // immutable reference is always readable
            Diagnostics::new()
        }
    }

    fn release(&mut self, ref_id: RefID) {
        self.borrows.release(ref_id)
    }

    fn divergent_control_flow(&mut self) {
        *self = Self::initial(
            &self.locals,
            self.acquired_resources.clone(),
            self.prev_had_errors,
        );
    }

    fn local_borrowed_by(&self, local: &Var) -> BTreeMap<RefID, Loc> {
        let (full_borrows, mut field_borrows) = self.borrows.borrowed_by(Self::LOCAL_ROOT);
        assert!(full_borrows.is_empty());
        field_borrows
            .remove(&Self::local_label(local))
            .unwrap_or_default()
    }

    fn resource_borrowed_by(&self, resource: &StructName) -> BTreeMap<RefID, Loc> {
        let (full_borrows, mut field_borrows) = self.borrows.borrowed_by(Self::LOCAL_ROOT);
        assert!(full_borrows.is_empty());
        field_borrows
            .remove(&Self::resource_label(resource))
            .unwrap_or_default()
    }

    // returns empty errors if borrowed_by is empty
    // Returns errors otherwise
    fn check_use_borrowed_by(
        borrows: &BorrowGraph,
        loc: Loc,
        local: &Var,
        full_borrows: &BTreeMap<RefID, Loc>,
        code: impl DiagnosticCode,
        verb: &'static str,
    ) -> Diagnostics {
        Self::borrow_error(
            borrows,
            loc,
            full_borrows,
            &BTreeMap::new(),
            code,
            move || {
                let local_str = match display_var(local.value()) {
                    DisplayVar::Tmp => panic!("ICE invalid use of tmp local {}", local.value()),
                    DisplayVar::Orig(s) => s,
                };
                format!("Invalid {} of local '{}'", verb, local_str)
            },
        )
    }

    //**********************************************************************************************
    // Command Entry Points
    //**********************************************************************************************

    pub fn bind_arguments(&mut self, parameter_types: &[(Var, SingleType)]) {
        for (local, ty) in parameter_types.iter() {
            let value = self.single_type_value(ty);
            let diags = self.assign_local(local.loc(), local, value);
            assert!(diags.is_empty())
        }
    }

    pub fn release_values(&mut self, values: Values) {
        for value in values {
            self.release_value(value)
        }
    }

    pub fn release_value(&mut self, value: Value) {
        if let Value::Ref(id) = value {
            self.release(id)
        }
    }

    pub fn assign_local(&mut self, loc: Loc, local: &Var, new_value: Value) -> Diagnostics {
        let old_value = self.locals.remove(local).unwrap();
        self.locals.add(*local, new_value).unwrap();
        match old_value {
            Value::Ref(id) => {
                self.release(id);
                Diagnostics::new()
            }
            Value::NonRef => {
                let borrowed_by = self.local_borrowed_by(local);
                Self::check_use_borrowed_by(
                    &self.borrows,
                    loc,
                    local,
                    &borrowed_by,
                    ReferenceSafety::Dangling,
                    "assignment",
                )
            }
        }
    }

    pub fn mutate(&mut self, loc: Loc, rvalue: Value) -> Diagnostics {
        let id = match rvalue {
            Value::NonRef => {
                assert!(
                    self.prev_had_errors,
                    "ICE borrow checking failed {:#?}",
                    loc
                );
                return Diagnostics::new();
            }
            Value::Ref(id) => id,
        };

        let diags = self.writable(loc, || "Invalid mutation of reference.".into(), id);
        self.release(id);
        diags
    }

    pub fn return_(&mut self, loc: Loc, rvalues: Values) -> Diagnostics {
        let mut released = BTreeSet::new();
        for (_, _local, stored_value) in &self.locals {
            if let Value::Ref(id) = stored_value {
                released.insert(*id);
            }
        }
        released.into_iter().for_each(|id| self.release(id));

        // Check locals are not borrowed
        let mut diags = Diagnostics::new();
        for (local, stored_value) in self.locals.key_cloned_iter() {
            if let Value::NonRef = stored_value {
                let borrowed_by = self.local_borrowed_by(&local);
                let local_diags = Self::borrow_error(
                    &self.borrows,
                    loc,
                    &borrowed_by,
                    &BTreeMap::new(),
                    ReferenceSafety::InvalidReturn,
                    || {
                        format!(
                            "Invalid return. Local variable '{}' is still being borrowed.",
                            local
                        )
                    },
                );
                diags.extend(local_diags)
            }
        }

        // Check resources are not borrowed
        for resource in self.acquired_resources.keys() {
            let borrowed_by = self.resource_borrowed_by(resource);
            let resource_diags = Self::borrow_error(
                &self.borrows,
                loc,
                &borrowed_by,
                &BTreeMap::new(),
                ReferenceSafety::InvalidReturn,
                || {
                    format!(
                        "Invalid return. Resource variable '{}' is still being borrowed.",
                        resource
                    )
                },
            );
            diags.extend(resource_diags)
        }

        // check any returned reference is not borrowed
        for rvalue in rvalues {
            match rvalue {
                Value::Ref(id) if self.borrows.is_mutable(id) => {
                    let (fulls, fields) = self.borrows.borrowed_by(id);
                    let msg = || {
                        "Invalid return of reference. Cannot transfer a mutable reference that is \
                         being borrowed"
                            .into()
                    };
                    let ds = Self::borrow_error(
                        &self.borrows,
                        loc,
                        &fulls,
                        &fields,
                        ReferenceSafety::InvalidTransfer,
                        msg,
                    );
                    diags.extend(ds);
                }
                _ => (),
            }
        }

        self.divergent_control_flow();
        diags
    }

    pub fn abort(&mut self) {
        self.divergent_control_flow()
    }

    //**********************************************************************************************
    // Expression Entry Points
    //**********************************************************************************************

    pub fn move_local(&mut self, loc: Loc, local: &Var) -> (Diagnostics, Value) {
        let old_value = self.locals.remove(local).unwrap();
        self.locals.add(*local, Value::NonRef).unwrap();
        match old_value {
            Value::Ref(id) => (Diagnostics::new(), Value::Ref(id)),
            Value::NonRef => {
                let borrowed_by = self.local_borrowed_by(local);
                let diags = Self::check_use_borrowed_by(
                    &self.borrows,
                    loc,
                    local,
                    &borrowed_by,
                    ReferenceSafety::Dangling,
                    "move",
                );
                (diags, Value::NonRef)
            }
        }
    }

    pub fn copy_local(&mut self, loc: Loc, local: &Var) -> (Diagnostics, Value) {
        match self.locals.get(local).unwrap() {
            Value::Ref(id) => {
                let id = *id;
                let new_id = self.declare_new_ref(self.borrows.is_mutable(id));
                self.add_copy(loc, id, new_id);
                (Diagnostics::new(), Value::Ref(new_id))
            }
            Value::NonRef => {
                let borrowed_by = self.local_borrowed_by(local);
                let borrows = &self.borrows;
                // check that it is 'readable'
                let mut_borrows = borrowed_by
                    .into_iter()
                    .filter(|(id, _loc)| borrows.is_mutable(*id))
                    .collect();
                let diags = Self::check_use_borrowed_by(
                    &self.borrows,
                    loc,
                    local,
                    &mut_borrows,
                    ReferenceSafety::MutOwns,
                    "copy",
                );
                (diags, Value::NonRef)
            }
        }
    }

    pub fn borrow_local(&mut self, loc: Loc, mut_: bool, local: &Var) -> (Diagnostics, Value) {
        assert!(
            !self.locals.get(local).unwrap().is_ref(),
            "ICE borrow ref {:#?}. Should have been caught in typing",
            loc
        );
        let new_id = self.declare_new_ref(mut_);
        // fails if there are full/epsilon borrows on the local
        let borrowed_by = self.local_borrowed_by(local);
        let diags = if !mut_ {
            let borrows = &self.borrows;
            // check that it is 'readable'
            let mut_borrows = borrowed_by
                .into_iter()
                .filter(|(id, _loc)| borrows.is_mutable(*id))
                .collect();
            Self::check_use_borrowed_by(
                borrows,
                loc,
                local,
                &mut_borrows,
                ReferenceSafety::RefTrans,
                "borrow",
            )
        } else {
            Diagnostics::new()
        };
        self.add_local_borrow(loc, local, new_id);
        (diags, Value::Ref(new_id))
    }

    pub fn freeze(&mut self, loc: Loc, rvalue: Value) -> (Diagnostics, Value) {
        let id = match rvalue {
            Value::NonRef => {
                assert!(
                    self.prev_had_errors,
                    "ICE borrow checking failed {:#?}",
                    loc
                );
                return (Diagnostics::new(), Value::NonRef);
            }
            Value::Ref(id) => id,
        };

        let diags = self.freezable(
            loc,
            ReferenceSafety::MutOwns,
            || "Invalid freeze.".into(),
            id,
            None,
        );
        let frozen_id = self.declare_new_ref(false);
        self.add_copy(loc, id, frozen_id);
        self.release(id);
        (diags, Value::Ref(frozen_id))
    }

    pub fn dereference(&mut self, loc: Loc, rvalue: Value) -> (Diagnostics, Value) {
        let id = match rvalue {
            Value::NonRef => {
                assert!(
                    self.prev_had_errors,
                    "ICE borrow checking failed {:#?}",
                    loc
                );
                return (Diagnostics::new(), Value::NonRef);
            }
            Value::Ref(id) => id,
        };

        let diags = self.readable(
            loc,
            ReferenceSafety::MutOwns,
            || "Invalid dereference.".into(),
            id,
            None,
        );
        self.release(id);
        (diags, Value::NonRef)
    }

    pub fn borrow_field(
        &mut self,
        loc: Loc,
        mut_: bool,
        rvalue: Value,
        field: &Field,
    ) -> (Diagnostics, Value) {
        let id = match rvalue {
            Value::NonRef => {
                assert!(
                    self.prev_had_errors,
                    "ICE borrow checking failed {:#?}",
                    loc
                );
                return (Diagnostics::new(), Value::NonRef);
            }
            Value::Ref(id) => id,
        };

        let diags = if mut_ {
            let msg = || format!("Invalid mutable borrow at field '{}'.", field);
            let (full_borrows, _field_borrows) = self.borrows.borrowed_by(id);
            // Any field borrows will be factored out
            Self::borrow_error(
                &self.borrows,
                loc,
                &full_borrows,
                &BTreeMap::new(),
                ReferenceSafety::MutOwns,
                msg,
            )
        } else {
            let msg = || format!("Invalid immutable borrow at field '{}'.", field);
            self.readable(loc, ReferenceSafety::RefTrans, msg, id, Some(field))
        };
        let field_borrow_id = self.declare_new_ref(mut_);
        self.add_field_borrow(loc, id, *field, field_borrow_id);
        self.release(id);
        (diags, Value::Ref(field_borrow_id))
    }

    pub fn borrow_global(&mut self, loc: Loc, mut_: bool, t: &BaseType) -> (Diagnostics, Value) {
        let new_id = self.declare_new_ref(mut_);
        let resource = match &t.value {
            BaseType_::Apply(_, sp!(_, TypeName_::ModuleType(_, s)), _) => s,
            _ => panic!("ICE type checking failed"),
        };
        let borrowed_by = self.resource_borrowed_by(resource);
        let borrows = &self.borrows;
        let msg = || format!("Invalid borrowing of resource '{}'", resource);
        let diags = if mut_ {
            Self::borrow_error(
                borrows,
                loc,
                &borrowed_by,
                &BTreeMap::new(),
                ReferenceSafety::MutOwns,
                msg,
            )
        } else {
            let mut_borrows = borrowed_by
                .into_iter()
                .filter(|(id, _loc)| borrows.is_mutable(*id))
                .collect();
            Self::borrow_error(
                borrows,
                loc,
                &mut_borrows,
                &BTreeMap::new(),
                ReferenceSafety::RefTrans,
                msg,
            )
        };
        self.add_resource_borrow(loc, resource, new_id);
        (diags, Value::Ref(new_id))
    }

    pub fn move_from(&mut self, loc: Loc, t: &BaseType) -> (Diagnostics, Value) {
        let resource = match &t.value {
            BaseType_::Apply(_, sp!(_, TypeName_::ModuleType(_, s)), _) => s,
            _ => panic!("ICE type checking failed"),
        };
        let borrowed_by = self.resource_borrowed_by(resource);
        let borrows = &self.borrows;
        let msg = || format!("Invalid extraction of resource '{}'", resource);
        let diags = Self::borrow_error(
            borrows,
            loc,
            &borrowed_by,
            &BTreeMap::new(),
            ReferenceSafety::Dangling,
            msg,
        );
        (diags, Value::NonRef)
    }

    pub fn call(
        &mut self,
        loc: Loc,
        args: Values,
        resources: &BTreeMap<StructName, Loc>,
        return_ty: &Type,
    ) -> (Diagnostics, Values) {
        let mut diags = Diagnostics::new();
        // Check acquires
        for resource in resources.keys() {
            let borrowed_by = self.resource_borrowed_by(resource);
            let borrows = &self.borrows;
            // TODO point to location of acquire
            let msg = || format!("Invalid acquiring of resource '{}'", resource);
            let ds = Self::borrow_error(
                borrows,
                loc,
                &borrowed_by,
                &BTreeMap::new(),
                ReferenceSafety::Dangling,
                msg,
            );
            diags.extend(ds);
        }

        // Check mutable arguments are not borrowed
        args.iter()
            .filter_map(|arg| arg.as_vref().filter(|id| self.borrows.is_mutable(*id)))
            .for_each(|mut_id| {
                let (fulls, fields) = self.borrows.borrowed_by(mut_id);
                let msg = || {
                    "Invalid usage of reference as function argument. Cannot transfer a mutable \
                     reference that is being borrowed"
                        .into()
                };
                let ds = Self::borrow_error(
                    &self.borrows,
                    loc,
                    &fulls,
                    &fields,
                    ReferenceSafety::InvalidTransfer,
                    msg,
                );
                diags.extend(ds);
            });

        let mut all_parents = BTreeSet::new();
        let mut mut_parents = BTreeSet::new();
        args.into_iter()
            .filter_map(|arg| arg.as_vref())
            .for_each(|id| {
                all_parents.insert(id);
                if self.borrows.is_mutable(id) {
                    mut_parents.insert(id);
                }
            });

        let values = match &return_ty.value {
            Type_::Unit => vec![],
            Type_::Single(s) => vec![self.single_type_value(s)],
            Type_::Multiple(ss) => ss.iter().map(|s| self.single_type_value(s)).collect(),
        };
        for value in &values {
            if let Value::Ref(id) = value {
                let parents = if self.borrows.is_mutable(*id) {
                    &mut_parents
                } else {
                    &all_parents
                };
                parents.iter().for_each(|p| self.add_borrow(loc, *p, *id));
            }
        }
        all_parents.into_iter().for_each(|id| self.release(id));

        (diags, values)
    }

    //**********************************************************************************************
    // Abstract State
    //**********************************************************************************************

    pub fn canonicalize_locals(&mut self, local_numbers: &UniqueMap<Var, usize>) {
        let mut all_refs = self.borrows.all_refs();
        let mut id_map = BTreeMap::new();
        for (_, local_, value) in &self.locals {
            if let Value::Ref(id) = value {
                assert!(all_refs.remove(id));
                id_map.insert(*id, RefID::new(*local_numbers.get_(local_).unwrap() + 1));
            }
        }
        all_refs.remove(&Self::LOCAL_ROOT);
        assert!(all_refs.is_empty());

        self.locals
            .iter_mut()
            .for_each(|(_, _, v)| v.remap_refs(&id_map));
        self.borrows.remap_refs(&id_map);
        self.next_id = self.locals.len() + 1;
    }

    pub fn join_(mut self, mut other: Self) -> Self {
        let mut released = BTreeSet::new();
        let mut locals = UniqueMap::new();
        for (local, self_value) in self.locals.key_cloned_iter() {
            let joined_value = match (self_value, other.locals.get(&local).unwrap()) {
                (Value::Ref(id1), Value::Ref(id2)) => {
                    assert!(id1 == id2);
                    Value::Ref(*id1)
                }
                (Value::NonRef, Value::Ref(released_id))
                | (Value::Ref(released_id), Value::NonRef) => {
                    released.insert(*released_id);
                    Value::NonRef
                }
                (Value::NonRef, Value::NonRef) => Value::NonRef,
            };
            locals.add(local, joined_value).unwrap();
        }
        for released_id in released {
            if self.borrows.contains_id(released_id) {
                self.release(released_id);
            }
            if other.borrows.contains_id(released_id) {
                other.release(released_id);
            }
        }

        let borrows = self.borrows.join(&other.borrows);
        let next_id = locals.len() + 1;
        let acquired_resources = self.acquired_resources.clone();
        let prev_had_errors = self.prev_had_errors;
        assert!(next_id == self.next_id);
        assert!(next_id == other.next_id);
        assert!(acquired_resources == other.acquired_resources);
        assert!(prev_had_errors == other.prev_had_errors);

        Self {
            locals,
            acquired_resources,
            borrows,
            next_id,
            prev_had_errors,
        }
    }

    fn leq(&self, other: &Self) -> bool {
        let BorrowState {
            locals: self_locals,
            borrows: self_borrows,
            next_id: self_next,
            acquired_resources: self_resources,
            prev_had_errors: self_prev_had_errors,
        } = self;
        let BorrowState {
            locals: other_locals,
            borrows: other_borrows,
            next_id: other_next,
            acquired_resources: other_resources,
            prev_had_errors: other_prev_had_errors,
        } = other;
        assert!(self_next == other_next, "ICE canonicalization failed");
        assert!(
            self_resources == other_resources,
            "ICE acquired resources static for the function"
        );
        assert!(
            self_prev_had_errors == other_prev_had_errors,
            "ICE previous errors flag changed"
        );
        self_locals == other_locals && self_borrows.leq(other_borrows)
    }
}

impl AbstractDomain for BorrowState {
    fn join(&mut self, other: &Self) -> JoinResult {
        let joined = self.clone().join_(other.clone());
        if !self.leq(&joined) {
            *self = joined;
            JoinResult::Changed
        } else {
            JoinResult::Unchanged
        }
    }
}

//**************************************************************************************************
// Display
//**************************************************************************************************

impl std::fmt::Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Label::Local(s) => write!(f, "local%{}", s),
            Label::Resource(s) => write!(f, "resource%{}", s),
            Label::Field(s) => write!(f, "{}", s),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::NonRef => write!(f, "_"),
            Value::Ref(id) => write!(f, "{:?}", id),
        }
    }
}

impl BorrowState {
    #[allow(dead_code)]
    pub fn display(&self) {
        println!("NEXT ID: {}", self.next_id);
        println!("LOCALS:");
        for (_, var, value) in &self.locals {
            println!("  {}: {}", var, value)
        }
        println!("BORROWS: ");
        self.borrows.display();
        println!();
    }
}
