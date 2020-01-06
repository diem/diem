// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//**************************************************************************************************
// Abstract state
//**************************************************************************************************

use crate::{
    cfgir::{absint::*, ast::*},
    errors::*,
    naming::ast::TypeName_,
    parser::ast::{Field, StructName, Var},
    shared::*,
};

use crate::shared::unique_map::UniqueMap;
use borrow_graph::references::RefID;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
enum Label {
    Local(String),
    Resource(String),
    Field(String),
}

type BorrowGraph = borrow_graph::graph::BorrowGraph<Loc, Label>;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Value {
    NonRef,
    Ref(RefID),
}
pub type Values = Vec<Value>;

#[derive(Clone, Debug, PartialEq)]
pub struct BorrowState {
    locals: UniqueMap<Var, Value>,
    borrows: BorrowGraph,
    next_id: usize,
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

    fn remap_refs(&mut self, id_map: &BTreeMap<RefID, RefID>) {
        match self {
            Value::Ref(id) if id_map.contains_key(id) => *id = id_map[id],
            _ => (),
        }
    }
}

impl BorrowState {
    pub fn initial<T>(locals: &UniqueMap<Var, T>) -> Self {
        let mut new_state = BorrowState {
            locals: locals.ref_map(|_, _| Value::NonRef),
            borrows: BorrowGraph::new(),
            next_id: locals.len() + 1,
        };
        new_state.borrows.new_ref(Self::LOCAL_ROOT, true);
        new_state
    }

    fn borrow_error<F: Fn() -> String>(
        borrows: &BorrowGraph,
        loc: Loc,
        full_borrows: &BTreeMap<RefID, Loc>,
        field_borrows: &BTreeMap<Label, BTreeMap<RefID, Loc>>,
        msg: F,
    ) -> Errors {
        if full_borrows.is_empty() && field_borrows.is_empty() {
            return Errors::new();
        }

        let mut_adj = |id| {
            if borrows.is_mutable(id) {
                "mutably "
            } else {
                ""
            }
        };
        let mut error = vec![(loc, msg())];
        for (borrower, rloc) in full_borrows {
            let adj = mut_adj(*borrower);
            error.push((
                *rloc,
                format!("It is still being {}borrowed by this reference", adj),
            ))
        }
        for (field_lbl, borrowers) in field_borrows {
            for (borrower, rloc) in borrowers {
                let adj = mut_adj(*borrower);
                let field = match field_lbl {
                    Label::Field(f) => f,
                    Label::Local(_) |
                    Label::Resource(_) => panic!("ICE local/resource should not be field borrows as they only exist from the virtual 'root' reference"),
                };
                error.push((
                    *rloc,
                    format!(
                        "Field '{}' is still being {}borrowed by this reference",
                        field, adj
                    ),
                ))
            }
        }
        assert!(error.len() >= 2);
        vec![error]
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

    fn writable<F: Fn() -> String>(&self, loc: Loc, msg: F, id: RefID) -> Errors {
        assert!(self.borrows.is_mutable(id), "ICE type checking failed");
        let (full_borrows, field_borrows) = self.borrows.borrowed_by(id);
        Self::borrow_error(&self.borrows, loc, &full_borrows, &field_borrows, msg)
    }

    fn freezable<F: Fn() -> String>(
        &self,
        loc: Loc,
        msg: F,
        id: RefID,
        at_field_opt: Option<&Field>,
    ) -> Errors {
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
                let valid_field = at_field_opt
                    .map(|at_field| match &f {
                        Label::Field(f_) => f_ == at_field.value(),
                        _ => false,
                    })
                    .unwrap_or(true);
                if !valid_field {
                    return None;
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
            msg,
        )
    }

    fn readable<F: Fn() -> String>(
        &self,
        loc: Loc,
        msg: F,
        id: RefID,
        at_field_opt: Option<&Field>,
    ) -> Errors {
        let is_mutable = self.borrows.is_mutable(id);
        if is_mutable {
            self.freezable(loc, msg, id, at_field_opt)
        } else {
            // immutable reference is always readable
            Errors::new()
        }
    }

    fn release(&mut self, ref_id: RefID) {
        self.borrows.release(ref_id)
    }

    fn divergent_control_flow(&mut self) {
        *self = Self::initial(&self.locals);
    }

    fn local_borrowed_by(&self, local: &Var) -> BTreeMap<RefID, Loc> {
        let (full_borrows, mut field_borrows) = self.borrows.borrowed_by(Self::LOCAL_ROOT);
        assert!(full_borrows.is_empty());
        field_borrows
            .remove(&Self::local_label(local))
            .unwrap_or_else(BTreeMap::new)
    }

    fn resource_borrowed_by(&self, resource: &StructName) -> BTreeMap<RefID, Loc> {
        let (full_borrows, mut field_borrows) = self.borrows.borrowed_by(Self::LOCAL_ROOT);
        assert!(full_borrows.is_empty());
        field_borrows
            .remove(&Self::resource_label(resource))
            .unwrap_or_else(BTreeMap::new)
    }

    // returns empty errors if borrowed_by is empty
    // Returns errors otherwise
    fn check_use_borrowed_by(
        borrows: &BorrowGraph,
        loc: Loc,
        local: &Var,
        full_borrows: &BTreeMap<RefID, Loc>,
        verb: &'static str,
    ) -> Errors {
        Self::borrow_error(borrows, loc, full_borrows, &BTreeMap::new(), move || {
            format!("Invalid {} of local '{}'", verb, local)
        })
    }

    //**********************************************************************************************
    // Command Entry Points
    //**********************************************************************************************

    pub fn bind_arguments(&mut self, parameter_types: &[(Var, SingleType)]) {
        for (local, ty) in parameter_types.iter() {
            let value = self.single_type_value(ty);
            let errors = self.assign_local(local.loc(), local, value);
            assert!(errors.is_empty())
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

    pub fn assign_local(&mut self, loc: Loc, local: &Var, new_value: Value) -> Errors {
        let old_value = self.locals.remove(local).unwrap();
        self.locals.add(local.clone(), new_value).unwrap();
        match old_value {
            Value::Ref(id) => {
                self.release(id);
                Errors::new()
            }
            Value::NonRef => {
                let borrowed_by = self.local_borrowed_by(local);
                Self::check_use_borrowed_by(&self.borrows, loc, local, &borrowed_by, "assignment")
            }
        }
    }

    pub fn mutate(&mut self, loc: Loc, rvalue: Value) -> Errors {
        let id = match rvalue {
            Value::NonRef => panic!("ICE type checking failed"),
            Value::Ref(id) => id,
        };

        let errors = self.writable(loc, || "Invalid mutation of reference.".into(), id);
        self.release(id);
        errors
    }

    pub fn return_(&mut self, loc: Loc, rvalues: Values) -> Errors {
        let mut released = BTreeSet::new();
        for (_local, stored_value) in self.locals.iter() {
            if let Value::Ref(id) = stored_value {
                released.insert(*id);
            }
        }
        released.into_iter().for_each(|id| self.release(id));

        let mut errors = Errors::new();
        for (local, stored_value) in self.locals.iter() {
            if let Value::NonRef = stored_value {
                let borrowed_by = self.local_borrowed_by(&local);
                let mut local_errors =
                    Self::borrow_error(&self.borrows, loc, &borrowed_by, &BTreeMap::new(), || {
                        format!("Invalid return. Local '{}' is still being borrowed.", local)
                    });
                errors.append(&mut local_errors)
            }
        }

        for rvalue in rvalues {
            match rvalue {
                Value::Ref(id) if self.borrows.is_mutable(id) => {
                    let (fulls, fields) = self.borrows.borrowed_by(id);
                    let msg = || {
                        "Invalid return of reference. Cannot transfer a mutable reference that is being borrowed".into()
                    };
                    let mut es = Self::borrow_error(&self.borrows, loc, &fulls, &fields, msg);
                    errors.append(&mut es);
                }
                _ => (),
            }
        }

        self.divergent_control_flow();
        errors
    }

    pub fn abort(&mut self) {
        self.divergent_control_flow()
    }

    //**********************************************************************************************
    // Expression Entry Points
    //**********************************************************************************************

    pub fn move_local(&mut self, loc: Loc, local: &Var) -> (Errors, Value) {
        let old_value = self.locals.remove(local).unwrap();
        self.locals.add(local.clone(), Value::NonRef).unwrap();
        match old_value {
            Value::Ref(id) => (vec![], Value::Ref(id)),
            Value::NonRef => {
                let borrowed_by = self.local_borrowed_by(local);
                let errors =
                    Self::check_use_borrowed_by(&self.borrows, loc, local, &borrowed_by, "move");
                (errors, Value::NonRef)
            }
        }
    }

    pub fn copy_local(&mut self, loc: Loc, local: &Var) -> (Errors, Value) {
        match self.locals.get(local).unwrap() {
            Value::Ref(id) => {
                let id = *id;
                let new_id = self.declare_new_ref(self.borrows.is_mutable(id));
                self.add_copy(loc, id, new_id);
                (vec![], Value::Ref(new_id))
            }
            Value::NonRef => {
                let borrowed_by = self.local_borrowed_by(local);
                let errors =
                    Self::check_use_borrowed_by(&self.borrows, loc, local, &borrowed_by, "copy");
                (errors, Value::NonRef)
            }
        }
    }

    pub fn borrow_local(&mut self, loc: Loc, mut_: bool, local: &Var) -> (Errors, Value) {
        assert!(
            !self.locals.get(local).unwrap().is_ref(),
            "ICE borrow ref {:#?}. Should have been caught in typing",
            loc
        );
        let new_id = self.declare_new_ref(mut_);
        let borrowed_by = self.local_borrowed_by(local);
        let borrows = &self.borrows;
        let errors = if mut_ {
            Self::check_use_borrowed_by(borrows, loc, local, &borrowed_by, "mutable borrow")
        } else {
            let filtered_mut = borrowed_by
                .into_iter()
                .filter(|(id, _loc)| borrows.is_mutable(*id));
            let mut_borrows = filtered_mut.collect::<BTreeMap<_, _>>();
            Self::check_use_borrowed_by(borrows, loc, local, &mut_borrows, "borrow")
        };
        self.add_local_borrow(loc, local, new_id);
        (errors, Value::Ref(new_id))
    }

    pub fn freeze(&mut self, loc: Loc, rvalue: Value) -> (Errors, Value) {
        let id = match rvalue {
            Value::NonRef => panic!("ICE type checking failed"),
            Value::Ref(id) => id,
        };

        let errors = self.freezable(loc, || "Invalid freeze.".into(), id, None);
        let frozen_id = self.declare_new_ref(false);
        self.add_copy(loc, id, frozen_id);
        self.release(id);
        (errors, Value::Ref(frozen_id))
    }

    pub fn dereference(&mut self, loc: Loc, rvalue: Value) -> (Errors, Value) {
        let id = match rvalue {
            Value::NonRef => panic!("ICE type checking failed"),
            Value::Ref(id) => id,
        };

        let errors = self.readable(loc, || "Invalid dereference.".into(), id, None);
        self.release(id);
        (errors, Value::NonRef)
    }

    pub fn borrow_field(
        &mut self,
        loc: Loc,
        mut_: bool,
        rvalue: Value,
        field: &Field,
    ) -> (Errors, Value) {
        let id = match rvalue {
            Value::NonRef => panic!("ICE type checking failed"),
            Value::Ref(id) => id,
        };

        let errors = if mut_ {
            let msg = || format!("Invalid mutable borrow at field '{}'.", field);
            let (full_borrows, _field_borrows) = self.borrows.borrowed_by(id);
            // Any field borrows will be factored out
            Self::borrow_error(&self.borrows, loc, &full_borrows, &BTreeMap::new(), msg)
        } else {
            let msg = || format!("Invalid immutable borrow at field '{}'.", field);
            self.readable(loc, msg, id, Some(&field))
        };
        let field_borrow_id = self.declare_new_ref(mut_);
        self.add_field_borrow(loc, id, field.clone(), field_borrow_id);
        self.release(id);
        (errors, Value::Ref(field_borrow_id))
    }

    pub fn borrow_global(&mut self, loc: Loc, mut_: bool, t: &BaseType) -> (Errors, Value) {
        let new_id = self.declare_new_ref(mut_);
        let resource = match &t.value {
            BaseType_::Apply(_, sp!(_, TypeName_::ModuleType(_, s)), _) => s,
            _ => panic!("ICE type checking failed"),
        };
        let borrowed_by = self.resource_borrowed_by(resource);
        let borrows = &self.borrows;
        let msg = || format!("Invalid borrowing of resource '{}'", resource);
        let errors = if mut_ {
            Self::borrow_error(borrows, loc, &borrowed_by, &BTreeMap::new(), msg)
        } else {
            let filtered_mut = borrowed_by
                .into_iter()
                .filter(|(id, _loc)| borrows.is_mutable(*id));
            let mut_borrows = filtered_mut.collect::<BTreeMap<_, _>>();
            Self::borrow_error(borrows, loc, &mut_borrows, &BTreeMap::new(), msg)
        };
        self.add_resource_borrow(loc, resource, new_id);
        (errors, Value::Ref(new_id))
    }

    pub fn move_from(&mut self, loc: Loc, t: &BaseType) -> (Errors, Value) {
        let resource = match &t.value {
            BaseType_::Apply(_, sp!(_, TypeName_::ModuleType(_, s)), _) => s,
            _ => panic!("ICE type checking failed"),
        };
        let borrowed_by = self.resource_borrowed_by(resource);
        let borrows = &self.borrows;
        let msg = || format!("Invalid extraction of resource '{}'", resource);
        let errors = Self::borrow_error(borrows, loc, &borrowed_by, &BTreeMap::new(), msg);
        (errors, Value::NonRef)
    }

    pub fn call(
        &mut self,
        loc: Loc,
        args: Values,
        acquires: &BTreeSet<BaseType>,
        return_ty: &Type,
    ) -> (Errors, Values) {
        let mut errors = vec![];
        let resources = acquires
            .iter()
            .map(|t| match &t.value {
                BaseType_::Apply(_, sp!(_, TypeName_::ModuleType(_, s)), _) => s,
                _ => panic!("ICE type checking failed"),
            })
            .collect::<BTreeSet<_>>();
        for resource in &resources {
            let borrowed_by = self.resource_borrowed_by(resource);
            let borrows = &self.borrows;
            // TODO point to location of acquire
            let msg = || format!("Invalid acquiring of resource '{}'", resource);
            let mut es = Self::borrow_error(borrows, loc, &borrowed_by, &BTreeMap::new(), msg);
            errors.append(&mut es);
        }
        for arg in &args {
            match arg {
                Value::Ref(id) if self.borrows.is_mutable(*id) => {
                    let (fulls, fields) = self.borrows.borrowed_by(*id);
                    let msg = || {
                        "Invalid usage of reference as function argument. Cannot transfer a mutable reference that is being borrowed".into()
                    };
                    let mut es = Self::borrow_error(&self.borrows, loc, &fulls, &fields, msg);
                    errors.append(&mut es);
                }
                _ => (),
            }
        }

        let mut all_parents = BTreeSet::new();
        let mut mut_parents = BTreeSet::new();
        for arg in args {
            if let Value::Ref(id) = arg {
                all_parents.insert(id);
                if self.borrows.is_mutable(id) {
                    mut_parents.insert(id);
                }
            }
        }

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
                resources
                    .iter()
                    .for_each(|r| self.add_resource_borrow(loc, r, *id));
            }
        }
        all_parents.into_iter().for_each(|id| self.release(id));

        (errors, values)
    }

    //**********************************************************************************************
    // Abstract State
    //**********************************************************************************************

    pub fn canonicalize_locals(&mut self, local_numbers: &UniqueMap<Var, usize>) {
        let mut all_refs = self.borrows.all_refs();
        let mut id_map = BTreeMap::new();
        for (local, value) in self.locals.iter() {
            if let Value::Ref(id) = value {
                assert!(all_refs.remove(id));
                id_map.insert(*id, RefID::new(*local_numbers.get(&local).unwrap() + 1));
            }
        }
        all_refs.remove(&Self::LOCAL_ROOT);
        assert!(all_refs.is_empty());

        self.locals
            .iter_mut()
            .for_each(|(_, v)| v.remap_refs(&id_map));
        self.borrows.remap_refs(&id_map);
        self.next_id = self.locals.len() + 1;
    }

    pub fn join_(mut self, mut other: Self) -> Self {
        let mut released = BTreeSet::new();
        let mut locals = UniqueMap::new();
        for (local, self_value) in self.locals.iter() {
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
        assert!(next_id == self.next_id);
        assert!(next_id == other.next_id);

        Self {
            locals,
            borrows,
            next_id,
        }
    }

    fn leq(&self, other: &Self) -> bool {
        let BorrowState {
            locals: self_locals,
            borrows: self_borrows,
            next_id: self_next,
        } = self;
        let BorrowState {
            locals: other_locals,
            borrows: other_borrows,
            next_id: other_next,
        } = other;
        assert!(self_next == other_next, "ICE canonicalization failed");
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
