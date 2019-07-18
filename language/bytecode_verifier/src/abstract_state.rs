// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines the abstract state for the type and memory safety analysis.
use crate::{
    absint::{AbstractDomain, JoinResult},
    nonce::Nonce,
    partition::Partition,
};
use mirai_annotations::checked_verify;
use std::collections::{BTreeMap, BTreeSet};
use vm::file_format::{FieldDefinitionIndex, LocalIndex};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AbstractValue {
    Reference(Nonce),
    Value(bool, BTreeSet<Nonce>),
}

impl AbstractValue {
    pub fn is_reference(&self) -> bool {
        match self {
            AbstractValue::Reference(_) => true,
            AbstractValue::Value(_, _) => false,
        }
    }

    pub fn is_value(&self) -> bool {
        !self.is_reference()
    }

    pub fn is_unrestricted_value(&self) -> bool {
        match self {
            AbstractValue::Reference(_) => false,
            AbstractValue::Value(is_resource, _) => !*is_resource,
        }
    }

    pub fn is_borrowed_value(&self) -> bool {
        match self {
            AbstractValue::Reference(_) => false,
            AbstractValue::Value(_, nonce_set) => !nonce_set.is_empty(),
        }
    }

    pub fn full_value(is_resource: bool) -> Self {
        AbstractValue::Value(is_resource, BTreeSet::new())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum BorrowInfo {
    BorrowedBy(BTreeSet<Nonce>),
    FieldsBorrowedBy(BTreeMap<FieldDefinitionIndex, BTreeSet<Nonce>>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AbstractState {
    locals: BTreeMap<LocalIndex, AbstractValue>,
    borrows: BTreeMap<Nonce, BorrowInfo>,
    partition: Partition,
}

impl AbstractState {
    /// create a new abstract state
    pub fn new(locals: BTreeMap<LocalIndex, AbstractValue>) -> Self {
        let borrows = BTreeMap::new();
        let mut partition = Partition::default();
        for value in locals.values() {
            if let AbstractValue::Reference(nonce) = value {
                partition.add_nonce(nonce.clone());
            }
        }
        AbstractState {
            locals,
            borrows,
            partition,
        }
    }

    /// checks if local@idx is available
    pub fn is_available(&self, idx: LocalIndex) -> bool {
        self.locals.contains_key(&idx)
    }

    /// returns local@idx
    pub fn local(&self, idx: LocalIndex) -> &AbstractValue {
        &self.locals[&idx]
    }

    /// removes local@idx
    pub fn remove_local(&mut self, idx: LocalIndex) -> AbstractValue {
        self.locals.remove(&idx).unwrap()
    }

    /// inserts local@idx
    pub fn insert_local(&mut self, idx: LocalIndex, value: AbstractValue) {
        self.locals.insert(idx, value);
    }

    /// checks if local@idx is a reference
    pub fn is_reference(&self, idx: LocalIndex) -> bool {
        self.locals[&idx].is_reference()
    }

    /// checks if local@idx is a value
    pub fn is_value(&self, idx: LocalIndex) -> bool {
        self.locals[&idx].is_value()
    }

    /// destroys local@idx
    /// call only if self.is_safe_to_destroy(idx) returns true
    pub fn destroy_local(&mut self, idx: LocalIndex) {
        let local = self.locals.remove(&idx).unwrap();
        match local {
            AbstractValue::Reference(nonce) => self.destroy_nonce(nonce),
            AbstractValue::Value(is_resource, borrowed_nonces) => {
                checked_verify!(!is_resource && borrowed_nonces.is_empty());
            }
        }
    }

    /// nonce must be fresh
    pub fn add_nonce(&mut self, nonce: Nonce) {
        self.partition.add_nonce(nonce);
    }

    /// destroys nonce
    /// borrows of nonce become borrows of any nonce' such that nonce borrows from nonce'
    pub fn destroy_nonce(&mut self, nonce: Nonce) {
        let mut nonce_set = BTreeSet::new();
        let mut new_locals = BTreeMap::new();
        let mut new_borrows = BTreeMap::new();

        if let Some(borrow_info) = self.borrows.remove(&nonce) {
            new_borrows = self.strong_propagate(nonce.clone(), &borrow_info);
            match borrow_info {
                BorrowInfo::BorrowedBy(x) => {
                    nonce_set = x.clone();
                }
                BorrowInfo::FieldsBorrowedBy(y) => {
                    nonce_set = Self::get_union_of_sets(&BTreeSet::new(), &y);
                }
            }
        }

        for (x, value) in &self.locals {
            if let AbstractValue::Value(is_resource, y) = value {
                if y.contains(&nonce) {
                    let mut y_restrict = y.clone();
                    y_restrict.remove(&nonce);
                    new_locals.insert(
                        x.clone(),
                        AbstractValue::Value(
                            *is_resource,
                            y_restrict.union(&nonce_set).cloned().collect(),
                        ),
                    );
                } else {
                    new_locals.insert(x.clone(), AbstractValue::Value(*is_resource, y.clone()));
                }
            } else {
                new_locals.insert(x.clone(), value.clone());
            }
        }

        for (x, borrow_info) in &self.borrows {
            if new_borrows.contains_key(x) {
                continue;
            }
            match borrow_info {
                BorrowInfo::BorrowedBy(y) => {
                    if y.contains(&nonce) {
                        let mut y_restrict = y.clone();
                        y_restrict.remove(&nonce);
                        let y_update = y_restrict
                            .union(&nonce_set)
                            .cloned()
                            .collect::<BTreeSet<_>>();
                        if !y_update.is_empty() {
                            new_borrows.insert(x.clone(), BorrowInfo::BorrowedBy(y_update));
                        }
                    } else {
                        new_borrows.insert(x.clone(), BorrowInfo::BorrowedBy(y.clone()));
                    }
                }
                BorrowInfo::FieldsBorrowedBy(w) => {
                    let mut new_index_to_nonce_set = BTreeMap::new();
                    for (idx, y) in w {
                        if y.contains(&nonce) {
                            let mut y_restrict = y.clone();
                            y_restrict.remove(&nonce);
                            let y_update = y_restrict
                                .union(&nonce_set)
                                .cloned()
                                .collect::<BTreeSet<_>>();
                            if !y_update.is_empty() {
                                new_index_to_nonce_set.insert(idx.clone(), y_update);
                            }
                        } else {
                            new_index_to_nonce_set.insert(idx.clone(), y.clone());
                        }
                    }
                    if !new_index_to_nonce_set.is_empty() {
                        new_borrows.insert(
                            x.clone(),
                            BorrowInfo::FieldsBorrowedBy(new_index_to_nonce_set),
                        );
                    }
                }
            }
        }

        self.locals = new_locals;
        self.borrows = new_borrows;
        self.partition.remove_nonce(nonce);
    }

    /// checks if there are any pending borrows on value
    pub fn is_full(&self, value: &AbstractValue) -> bool {
        match value {
            AbstractValue::Reference(nonce) => !self.borrows.contains_key(&nonce),
            AbstractValue::Value(_, nonce_set) => nonce_set.is_empty(),
        }
    }

    /// returns the set of nonces borrowing from nonce that might alias some idx-extension of nonce
    pub fn borrowed_nonces_for_field(
        &self,
        idx: FieldDefinitionIndex,
        nonce: Nonce,
    ) -> BTreeSet<Nonce> {
        if self.borrows.contains_key(&nonce) {
            match &self.borrows[&nonce] {
                BorrowInfo::BorrowedBy(x) => x.clone(),
                BorrowInfo::FieldsBorrowedBy(y) => {
                    if y.contains_key(&idx) {
                        y[&idx].clone()
                    } else {
                        BTreeSet::new()
                    }
                }
            }
        } else {
            BTreeSet::new()
        }
    }

    /// returns the set of nonces borrowing from nonce that might alias some extension of nonce
    pub fn borrowed_nonces(&self, nonce: Nonce) -> BTreeSet<Nonce> {
        if self.borrows.contains_key(&nonce) {
            match &self.borrows[&nonce] {
                BorrowInfo::BorrowedBy(x) => x.clone(),
                BorrowInfo::FieldsBorrowedBy(y) => {
                    let empty_set = BTreeSet::new();
                    Self::get_union_of_sets(&empty_set, y)
                }
            }
        } else {
            BTreeSet::new()
        }
    }

    /// update self to reflect a borrow of idx from nonce by new_nonce
    pub fn borrow_field_from_nonce(
        &mut self,
        idx: FieldDefinitionIndex,
        nonce: Nonce,
        new_nonce: Nonce,
    ) {
        self.borrows
            .entry(nonce.clone())
            .and_modify(|borrow_info| match borrow_info {
                BorrowInfo::BorrowedBy(nonce_set) => {
                    nonce_set.insert(new_nonce.clone());
                }
                BorrowInfo::FieldsBorrowedBy(index_to_nonce_set) => {
                    index_to_nonce_set
                        .entry(idx)
                        .and_modify(|nonce_set| {
                            nonce_set.insert(new_nonce.clone());
                        })
                        .or_insert({
                            let mut x = BTreeSet::new();
                            x.insert(new_nonce.clone());
                            x
                        });
                }
            })
            .or_insert({
                let mut x = BTreeSet::new();
                x.insert(new_nonce.clone());
                let mut y = BTreeMap::new();
                y.insert(idx, x);
                BorrowInfo::FieldsBorrowedBy(y)
            });
    }

    /// update self to reflect a borrow of a value local@idx by new_nonce
    pub fn borrow_from_local_value(&mut self, idx: LocalIndex, new_nonce: Nonce) {
        checked_verify!(self.locals[&idx].is_value());
        self.locals.entry(idx).and_modify(|value| {
            if let AbstractValue::Value(_, nonce_set) = value {
                nonce_set.insert(new_nonce);
            }
        });
    }

    /// update self to reflect a borrow of a reference local@idx by new_nonce
    pub fn borrow_from_local_reference(&mut self, idx: LocalIndex, new_nonce: Nonce) {
        checked_verify!(self.locals[&idx].is_reference());
        if let AbstractValue::Reference(borrowee) = &self.locals[&idx] {
            if let Some(info) = self.borrows.remove(borrowee) {
                self.borrows.insert(new_nonce.clone(), info);
            }
            self.borrows.entry(borrowee.clone()).or_insert({
                let mut x = BTreeSet::new();
                x.insert(new_nonce.clone());
                BorrowInfo::BorrowedBy(x)
            });
            self.partition.add_equality(new_nonce, borrowee.clone());
        }
    }

    /// update self to reflect a borrow from each nonce in to_borrow_from by new_nonce
    pub fn borrow_from_nonces(&mut self, to_borrow_from: &BTreeSet<Nonce>, new_nonce: Nonce) {
        for x in to_borrow_from {
            self.borrow_from_nonce(x.clone(), new_nonce.clone());
        }
    }

    /// checks if self is canonical
    pub fn is_canonical(&self) -> bool {
        let mut values = BTreeMap::new();
        let mut references = BTreeMap::new();
        Self::split_locals(&self.locals, &mut values, &mut references);
        references.iter().all(|(x, y)| y.is(*x as usize))
    }

    /// returns the canonical representation of self
    pub fn construct_canonical_state(&self) -> Self {
        let mut values = BTreeMap::new();
        let mut references = BTreeMap::new();
        Self::split_locals(&self.locals, &mut values, &mut references);

        let mut locals = BTreeMap::new();
        let mut nonce_map = BTreeMap::new();
        for (x, y) in references {
            nonce_map.insert(y, Nonce::new(x as usize));
            locals.insert(x, AbstractValue::Reference(Nonce::new(x as usize)));
        }
        for (x, (is_resource, nonce_set)) in values {
            locals.insert(
                x,
                AbstractValue::Value(is_resource, Self::map_nonce_set(&nonce_map, &nonce_set)),
            );
        }
        let mut borrows = BTreeMap::new();
        for (x, borrow_info) in &self.borrows {
            match borrow_info {
                BorrowInfo::BorrowedBy(y) => {
                    borrows.insert(
                        nonce_map[&x].clone(),
                        BorrowInfo::BorrowedBy(Self::map_nonce_set(&nonce_map, &y)),
                    );
                }
                BorrowInfo::FieldsBorrowedBy(w) => {
                    let mut index_to_nonce_set = BTreeMap::new();
                    for (idx, y) in w {
                        index_to_nonce_set.insert(idx.clone(), Self::map_nonce_set(&nonce_map, &y));
                    }
                    borrows.insert(
                        nonce_map[&x].clone(),
                        BorrowInfo::FieldsBorrowedBy(index_to_nonce_set),
                    );
                }
            }
        }
        let partition = self.partition.construct_canonical_partition(&nonce_map);

        AbstractState {
            locals,
            borrows,
            partition,
        }
    }

    fn unrestricted_borrowed_value_unavailable(
        state1: &AbstractState,
        state2: &AbstractState,
    ) -> bool {
        state1.locals.keys().any(|x| {
            state1.locals[x].is_unrestricted_value()
                && state1.locals[x].is_borrowed_value()
                && !state2.locals.contains_key(x)
        })
    }

    fn strong_propagate(
        &self,
        nonce: Nonce,
        borrow_info: &BorrowInfo,
    ) -> BTreeMap<Nonce, BorrowInfo> {
        let mut new_borrows = BTreeMap::new();
        let mut singleton_nonce_set = BTreeSet::new();
        singleton_nonce_set.insert(nonce.clone());
        for (x, y) in &self.borrows {
            if self.partition.is_equal(x.clone(), nonce.clone()) {
                if let BorrowInfo::BorrowedBy(nonce_set) = y {
                    if nonce_set == &singleton_nonce_set {
                        new_borrows.insert(x.clone(), borrow_info.clone());
                    }
                }
            }
        }
        new_borrows
    }

    fn borrow_from_nonce(&mut self, nonce: Nonce, new_nonce: Nonce) {
        self.borrows.entry(nonce.clone()).or_insert({
            let mut x = BTreeSet::new();
            x.insert(new_nonce);
            BorrowInfo::BorrowedBy(x)
        });
    }

    fn map_nonce_set(
        nonce_map: &BTreeMap<Nonce, Nonce>,
        nonce_set: &BTreeSet<Nonce>,
    ) -> BTreeSet<Nonce> {
        let mut mapped_nonce_set = BTreeSet::new();
        for x in nonce_set {
            mapped_nonce_set.insert(nonce_map[x].clone());
        }
        mapped_nonce_set
    }

    fn split_locals(
        locals: &BTreeMap<LocalIndex, AbstractValue>,
        values: &mut BTreeMap<LocalIndex, (bool, BTreeSet<Nonce>)>,
        references: &mut BTreeMap<LocalIndex, Nonce>,
    ) {
        for (x, y) in locals {
            match y {
                AbstractValue::Reference(nonce) => {
                    references.insert(x.clone(), nonce.clone());
                }
                AbstractValue::Value(is_resource, nonces) => {
                    values.insert(x.clone(), (*is_resource, nonces.clone()));
                }
            }
        }
    }

    fn get_union_of_sets(
        nonce_set: &BTreeSet<Nonce>,
        index_to_nonce_set: &BTreeMap<FieldDefinitionIndex, BTreeSet<Nonce>>,
    ) -> BTreeSet<Nonce> {
        index_to_nonce_set
            .values()
            .fold(nonce_set.clone(), |mut acc, set| {
                for x in set {
                    acc.insert(x.clone());
                }
                acc
            })
    }

    fn get_union_of_maps(
        index_to_nonce_set1: &BTreeMap<FieldDefinitionIndex, BTreeSet<Nonce>>,
        index_to_nonce_set2: &BTreeMap<FieldDefinitionIndex, BTreeSet<Nonce>>,
    ) -> BTreeMap<FieldDefinitionIndex, BTreeSet<Nonce>> {
        let mut index_to_nonce_set = BTreeMap::new();
        for (x, y) in index_to_nonce_set1 {
            if index_to_nonce_set2.contains_key(x) {
                index_to_nonce_set.insert(
                    x.clone(),
                    y.union(&index_to_nonce_set2[x]).cloned().collect(),
                );
            } else {
                index_to_nonce_set.insert(x.clone(), y.clone());
            }
        }
        for (x, y) in index_to_nonce_set2 {
            if index_to_nonce_set.contains_key(x) {
                continue;
            }
            index_to_nonce_set.insert(x.clone(), y.clone());
        }
        index_to_nonce_set
    }
}

impl AbstractDomain for AbstractState {
    /// attempts to join state to self and returns the result
    /// both self.is_canonical() and state.is_canonical() must be true
    fn join(&mut self, state: &AbstractState) -> JoinResult {
        // A join failure occurs in each of the following situations:
        // - an unrestricted value is borrowed along one path but unavailable along the other
        // - a value that is not unrestricted, i.e., either reference or resource, is available
        //   along one path but not the other
        if Self::unrestricted_borrowed_value_unavailable(self, state)
            || Self::unrestricted_borrowed_value_unavailable(state, self)
        {
            return JoinResult::Error;
        }
        if self
            .locals
            .keys()
            .filter(|x| !self.locals[x].is_unrestricted_value())
            .collect::<BTreeSet<_>>()
            != state
                .locals
                .keys()
                .filter(|x| !state.locals[x].is_unrestricted_value())
                .collect::<BTreeSet<_>>()
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
        for (x, y) in &references1 {
            locals.insert(x.clone(), AbstractValue::Reference(y.clone()));
        }
        for (x, (is_resource1, nonce_set1)) in &values1 {
            if let Some((is_resource2, nonce_set2)) = values2.get(x) {
                checked_verify!(is_resource1 == is_resource2);
                locals.insert(
                    x.clone(),
                    AbstractValue::Value(
                        *is_resource1,
                        nonce_set1.union(nonce_set2).cloned().collect(),
                    ),
                );
            }
        }

        let mut borrows = BTreeMap::new();
        for (x, borrow_info) in &self.borrows {
            if state.borrows.contains_key(x) {
                match borrow_info {
                    BorrowInfo::BorrowedBy(y1) => match &state.borrows[x] {
                        BorrowInfo::BorrowedBy(y2) => {
                            borrows.insert(
                                x.clone(),
                                BorrowInfo::BorrowedBy(y1.union(y2).cloned().collect()),
                            );
                        }
                        BorrowInfo::FieldsBorrowedBy(w2) => {
                            borrows.insert(
                                x.clone(),
                                BorrowInfo::BorrowedBy(Self::get_union_of_sets(y1, w2)),
                            );
                        }
                    },
                    BorrowInfo::FieldsBorrowedBy(w1) => match &state.borrows[x] {
                        BorrowInfo::BorrowedBy(y2) => {
                            borrows.insert(
                                x.clone(),
                                BorrowInfo::BorrowedBy(Self::get_union_of_sets(y2, w1)),
                            );
                        }
                        BorrowInfo::FieldsBorrowedBy(w2) => {
                            borrows.insert(
                                x.clone(),
                                BorrowInfo::FieldsBorrowedBy(Self::get_union_of_maps(w1, w2)),
                            );
                        }
                    },
                }
            } else {
                borrows.insert(x.clone(), borrow_info.clone());
            }
        }
        for (x, borrow_info) in &state.borrows {
            if !borrows.contains_key(x) {
                borrows.insert(x.clone(), borrow_info.clone());
            }
        }

        let partition = self.partition.join(&state.partition);

        let next_state = AbstractState {
            locals,
            borrows,
            partition,
        };
        if next_state == *self {
            JoinResult::Unchanged
        } else {
            *self = next_state;
            JoinResult::Changed
        }
    }
}
