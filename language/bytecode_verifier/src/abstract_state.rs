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
use vm::file_format::{FieldDefinitionIndex, Kind, LocalIndex, StructDefinitionIndex};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AbstractValue {
    Reference(Nonce),
    Value(Kind, BTreeSet<Nonce>),
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
            AbstractValue::Value(Kind::Unrestricted, _) => true,
            AbstractValue::Value(Kind::All, _) | AbstractValue::Value(Kind::Resource, _) => false,
        }
    }

    pub fn is_borrowed_value(&self) -> bool {
        match self {
            AbstractValue::Reference(_) => false,
            AbstractValue::Value(_, nonce_set) => !nonce_set.is_empty(),
        }
    }

    pub fn is_safe_to_destroy(&self) -> bool {
        match self {
            AbstractValue::Reference(_) => true,
            AbstractValue::Value(Kind::All, _) | AbstractValue::Value(Kind::Resource, _) => false,
            AbstractValue::Value(Kind::Unrestricted, borrowed_nonces) => borrowed_nonces.is_empty(),
        }
    }

    pub fn full_value(kind: Kind) -> Self {
        AbstractValue::Value(kind, BTreeSet::new())
    }

    pub fn extract_nonce(&self) -> Option<Nonce> {
        match self {
            AbstractValue::Reference(nonce) => Some(*nonce),
            AbstractValue::Value(_, _) => None,
        }
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
    globals: BTreeMap<StructDefinitionIndex, BTreeSet<Nonce>>,
    borrows: BTreeMap<Nonce, BorrowInfo>,
    partition: Partition,
}

impl AbstractState {
    /// create a new abstract state
    pub fn new(
        locals: BTreeMap<LocalIndex, AbstractValue>,
        globals: BTreeMap<StructDefinitionIndex, BTreeSet<Nonce>>,
    ) -> Self {
        let borrows = BTreeMap::new();
        let mut partition = Partition::default();
        for value in locals.values() {
            if let AbstractValue::Reference(nonce) = value {
                partition.add_nonce(*nonce);
            }
        }
        AbstractState {
            locals,
            globals,
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

    /// returns global@idx
    pub fn global(&mut self, idx: StructDefinitionIndex) -> &BTreeSet<Nonce> {
        self.globals.entry(idx).or_insert_with(BTreeSet::new)
    }

    /// returns global@idx, None if not present
    pub fn global_opt(&self, idx: StructDefinitionIndex) -> Option<&BTreeSet<Nonce>> {
        self.globals.get(&idx)
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

    /// Return true if self may safely be destroyed
    pub fn is_safe_to_destroy(&self) -> bool {
        self.locals.values().all(|x| x.is_safe_to_destroy())
            && self.globals.values().all(|x| x.is_empty())
    }

    /// Return true if local@idx may safely be destroyed
    pub fn is_local_safe_to_destroy(&self, idx: LocalIndex) -> bool {
        self.local(idx).is_safe_to_destroy()
    }

    /// destroys local@idx
    /// call only if self.is_local_safe_to_destroy(idx) returns true
    pub fn destroy_local(&mut self, idx: LocalIndex) {
        let local = self.locals.remove(&idx).unwrap();
        match local {
            AbstractValue::Reference(nonce) => self.destroy_nonce(nonce),
            AbstractValue::Value(kind, borrowed_nonces) => {
                checked_verify!(kind == Kind::Unrestricted && borrowed_nonces.is_empty());
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
        let mut new_globals = BTreeMap::new();
        let mut new_borrows = BTreeMap::new();

        if let Some(borrow_info) = self.borrows.remove(&nonce) {
            new_borrows = self.strong_propagate(nonce, &borrow_info);
            match borrow_info {
                BorrowInfo::BorrowedBy(borrowed_by) => {
                    nonce_set = borrowed_by.clone();
                }
                BorrowInfo::FieldsBorrowedBy(fields_borrowed_by) => {
                    nonce_set = Self::get_union_of_sets(&BTreeSet::new(), &fields_borrowed_by);
                }
            }
        }

        for (idx, value) in &self.locals {
            if let AbstractValue::Value(kind, local_nonce_set) = value {
                if local_nonce_set.contains(&nonce) {
                    let mut local_nonce_set_restrict = local_nonce_set.clone();
                    local_nonce_set_restrict.remove(&nonce);
                    new_locals.insert(
                        idx.clone(),
                        AbstractValue::Value(
                            *kind,
                            local_nonce_set_restrict
                                .union(&nonce_set)
                                .cloned()
                                .collect(),
                        ),
                    );
                } else {
                    new_locals.insert(
                        idx.clone(),
                        AbstractValue::Value(*kind, local_nonce_set.clone()),
                    );
                }
            } else {
                new_locals.insert(idx.clone(), value.clone());
            }
        }

        for (idx, global_nonce_set) in &self.globals {
            if global_nonce_set.contains(&nonce) {
                let mut y_restrict = global_nonce_set.clone();
                y_restrict.remove(&nonce);
                new_globals.insert(idx.clone(), y_restrict.union(&nonce_set).cloned().collect());
            } else {
                new_globals.insert(idx.clone(), global_nonce_set.clone());
            }
        }

        for (src_nonce, borrow_info) in &self.borrows {
            if new_borrows.contains_key(src_nonce) {
                continue;
            }
            match borrow_info {
                BorrowInfo::BorrowedBy(borrowed_by) => {
                    if borrowed_by.contains(&nonce) {
                        let mut borrowed_by_restrict = borrowed_by.clone();
                        borrowed_by_restrict.remove(&nonce);
                        let borrowed_by_update = borrowed_by_restrict
                            .union(&nonce_set)
                            .cloned()
                            .collect::<BTreeSet<_>>();
                        if !borrowed_by_update.is_empty() {
                            new_borrows
                                .insert(*src_nonce, BorrowInfo::BorrowedBy(borrowed_by_update));
                        }
                    } else {
                        new_borrows.insert(*src_nonce, BorrowInfo::BorrowedBy(borrowed_by.clone()));
                    }
                }
                BorrowInfo::FieldsBorrowedBy(fields_borrowed_by) => {
                    let mut new_index_to_nonce_set = BTreeMap::new();
                    for (idx, borrowed_nonce_set) in fields_borrowed_by {
                        if borrowed_nonce_set.contains(&nonce) {
                            let mut borrowed_nonce_set_restrict = borrowed_nonce_set.clone();
                            borrowed_nonce_set_restrict.remove(&nonce);
                            let borrowed_nonce_set_update = borrowed_nonce_set_restrict
                                .union(&nonce_set)
                                .cloned()
                                .collect::<BTreeSet<_>>();
                            if !borrowed_nonce_set_update.is_empty() {
                                new_index_to_nonce_set
                                    .insert(idx.clone(), borrowed_nonce_set_update);
                            }
                        } else {
                            new_index_to_nonce_set.insert(idx.clone(), borrowed_nonce_set.clone());
                        }
                    }
                    if !new_index_to_nonce_set.is_empty() {
                        new_borrows.insert(
                            *src_nonce,
                            BorrowInfo::FieldsBorrowedBy(new_index_to_nonce_set),
                        );
                    }
                }
            }
        }

        self.locals = new_locals;
        self.globals = new_globals;
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
                BorrowInfo::BorrowedBy(borrowed_by) => borrowed_by.clone(),
                BorrowInfo::FieldsBorrowedBy(fields_borrowed_by) => {
                    if fields_borrowed_by.contains_key(&idx) {
                        fields_borrowed_by[&idx].clone()
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
                BorrowInfo::BorrowedBy(borrowed_by) => borrowed_by.clone(),
                BorrowInfo::FieldsBorrowedBy(fields_borrowed_by) => {
                    let empty_set = BTreeSet::new();
                    Self::get_union_of_sets(&empty_set, fields_borrowed_by)
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
            .entry(nonce)
            .and_modify(|borrow_info| match borrow_info {
                BorrowInfo::BorrowedBy(nonce_set) => {
                    nonce_set.insert(new_nonce);
                }
                BorrowInfo::FieldsBorrowedBy(index_to_nonce_set) => {
                    index_to_nonce_set
                        .entry(idx)
                        .and_modify(|nonce_set| {
                            nonce_set.insert(new_nonce);
                        })
                        .or_insert({
                            let mut nonce_set = BTreeSet::new();
                            nonce_set.insert(new_nonce);
                            nonce_set
                        });
                }
            })
            .or_insert({
                let mut nonce_set = BTreeSet::new();
                nonce_set.insert(new_nonce);
                let mut fields_borrowed_by = BTreeMap::new();
                fields_borrowed_by.insert(idx, nonce_set);
                BorrowInfo::FieldsBorrowedBy(fields_borrowed_by)
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
                self.borrows.insert(new_nonce, info);
            }
            self.borrows.entry(borrowee.clone()).or_insert({
                let mut nonce_set = BTreeSet::new();
                nonce_set.insert(new_nonce);
                BorrowInfo::BorrowedBy(nonce_set)
            });
            self.partition.add_equality(new_nonce, borrowee.clone());
        }
    }

    /// update self to reflect a borrow of a value global@idx by new_nonce
    pub fn borrow_from_global_value(&mut self, idx: StructDefinitionIndex, new_nonce: Nonce) {
        self.globals
            .entry(idx)
            .or_insert_with(BTreeSet::new)
            .insert(new_nonce);
    }

    /// update self to reflect a borrow from each nonce in to_borrow_from by new_nonce
    pub fn borrow_from_nonces(&mut self, to_borrow_from: &BTreeSet<Nonce>, new_nonce: Nonce) {
        for nonce in to_borrow_from {
            self.borrow_from_nonce(*nonce, new_nonce);
        }
    }

    /// checks if self is canonical
    pub fn is_canonical(&self) -> bool {
        let mut values = BTreeMap::new();
        let mut references = BTreeMap::new();
        Self::split_locals(&self.locals, &mut values, &mut references);
        references
            .iter()
            .all(|(idx, nonce)| nonce.is(*idx as usize))
    }

    /// returns the canonical representation of self
    pub fn construct_canonical_state(&self) -> Self {
        let mut values = BTreeMap::new();
        let mut references = BTreeMap::new();
        Self::split_locals(&self.locals, &mut values, &mut references);

        let mut locals = BTreeMap::new();
        let mut nonce_map = BTreeMap::new();
        for (idx, nonce) in references {
            nonce_map.insert(nonce, Nonce::new(idx as usize));
            locals.insert(idx, AbstractValue::Reference(Nonce::new(idx as usize)));
        }
        for (idx, (kind, nonce_set)) in values {
            locals.insert(
                idx,
                AbstractValue::Value(kind, Self::map_nonce_set(&nonce_map, &nonce_set)),
            );
        }
        let mut globals = BTreeMap::new();
        for (idx, nonce_set) in &self.globals {
            globals.insert(idx.clone(), Self::map_nonce_set(&nonce_map, &nonce_set));
        }
        let mut borrows = BTreeMap::new();
        for (nonce, borrow_info) in &self.borrows {
            match borrow_info {
                BorrowInfo::BorrowedBy(borrowed_by) => {
                    borrows.insert(
                        nonce_map[&nonce],
                        BorrowInfo::BorrowedBy(Self::map_nonce_set(&nonce_map, &borrowed_by)),
                    );
                }
                BorrowInfo::FieldsBorrowedBy(fields_borrowed_by) => {
                    let mut index_to_nonce_set = BTreeMap::new();
                    for (idx, nonce_set) in fields_borrowed_by {
                        index_to_nonce_set
                            .insert(idx.clone(), Self::map_nonce_set(&nonce_map, &nonce_set));
                    }
                    borrows.insert(
                        nonce_map[&nonce],
                        BorrowInfo::FieldsBorrowedBy(index_to_nonce_set),
                    );
                }
            }
        }
        let partition = self.partition.construct_canonical_partition(&nonce_map);

        AbstractState {
            locals,
            globals,
            borrows,
            partition,
        }
    }

    fn unrestricted_borrowed_value_unavailable(
        state1: &AbstractState,
        state2: &AbstractState,
    ) -> bool {
        state1.locals.keys().any(|idx| {
            state1.locals[idx].is_unrestricted_value()
                && state1.locals[idx].is_borrowed_value()
                && !state2.locals.contains_key(idx)
        })
    }

    fn strong_propagate(
        &self,
        nonce: Nonce,
        borrow_info: &BorrowInfo,
    ) -> BTreeMap<Nonce, BorrowInfo> {
        let mut new_borrows = BTreeMap::new();
        let mut singleton_nonce_set = BTreeSet::new();
        singleton_nonce_set.insert(nonce);
        for (src_nonce, borrowed_by) in &self.borrows {
            if self.partition.is_equal(*src_nonce, nonce) {
                if let BorrowInfo::BorrowedBy(nonce_set) = borrowed_by {
                    if nonce_set == &singleton_nonce_set {
                        new_borrows.insert(*src_nonce, borrow_info.clone());
                    }
                }
            }
        }
        new_borrows
    }

    fn borrow_from_nonce(&mut self, nonce: Nonce, new_nonce: Nonce) {
        self.borrows.entry(nonce).or_insert({
            let mut nonce_set = BTreeSet::new();
            nonce_set.insert(new_nonce);
            BorrowInfo::BorrowedBy(nonce_set)
        });
    }

    fn map_nonce_set(
        nonce_map: &BTreeMap<Nonce, Nonce>,
        nonce_set: &BTreeSet<Nonce>,
    ) -> BTreeSet<Nonce> {
        let mut mapped_nonce_set = BTreeSet::new();
        for nonce in nonce_set {
            mapped_nonce_set.insert(nonce_map[nonce]);
        }
        mapped_nonce_set
    }

    fn split_locals(
        locals: &BTreeMap<LocalIndex, AbstractValue>,
        values: &mut BTreeMap<LocalIndex, (Kind, BTreeSet<Nonce>)>,
        references: &mut BTreeMap<LocalIndex, Nonce>,
    ) {
        for (idx, value) in locals {
            match value {
                AbstractValue::Reference(nonce) => {
                    references.insert(idx.clone(), *nonce);
                }
                AbstractValue::Value(kind, nonces) => {
                    values.insert(idx.clone(), (*kind, nonces.clone()));
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
        for (idx, nonce_set) in index_to_nonce_set1 {
            if index_to_nonce_set2.contains_key(idx) {
                index_to_nonce_set.insert(
                    idx.clone(),
                    nonce_set
                        .union(&index_to_nonce_set2[idx])
                        .cloned()
                        .collect(),
                );
            } else {
                index_to_nonce_set.insert(idx.clone(), nonce_set.clone());
            }
        }
        for (idx, nonce_set) in index_to_nonce_set2 {
            if index_to_nonce_set.contains_key(idx) {
                continue;
            }
            index_to_nonce_set.insert(idx.clone(), nonce_set.clone());
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
            .filter(|idx| !self.locals[idx].is_unrestricted_value())
            .collect::<BTreeSet<_>>()
            != state
                .locals
                .keys()
                .filter(|idx| !state.locals[idx].is_unrestricted_value())
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
        for (idx, nonce) in &references1 {
            locals.insert(idx.clone(), AbstractValue::Reference(*nonce));
        }
        for (idx, (kind1, nonce_set1)) in &values1 {
            if let Some((kind2, nonce_set2)) = values2.get(idx) {
                checked_verify!(kind1 == kind2);
                locals.insert(
                    idx.clone(),
                    AbstractValue::Value(*kind1, nonce_set1.union(nonce_set2).cloned().collect()),
                );
            }
        }

        let mut globals = self.globals.clone();
        for (idx, global_nonce_set) in &state.globals {
            globals
                .entry(*idx)
                .and_modify(|nonce_set| {
                    *nonce_set = global_nonce_set.union(nonce_set).cloned().collect()
                })
                .or_insert_with(|| global_nonce_set.clone());
        }

        let mut borrows = BTreeMap::new();
        for (nonce, borrow_info) in &self.borrows {
            if state.borrows.contains_key(nonce) {
                match borrow_info {
                    BorrowInfo::BorrowedBy(borrowed_by1) => match &state.borrows[nonce] {
                        BorrowInfo::BorrowedBy(borrowed_by2) => {
                            borrows.insert(
                                *nonce,
                                BorrowInfo::BorrowedBy(
                                    borrowed_by1.union(borrowed_by2).cloned().collect(),
                                ),
                            );
                        }
                        BorrowInfo::FieldsBorrowedBy(fields_borrowed_by2) => {
                            borrows.insert(
                                *nonce,
                                BorrowInfo::BorrowedBy(Self::get_union_of_sets(
                                    borrowed_by1,
                                    fields_borrowed_by2,
                                )),
                            );
                        }
                    },
                    BorrowInfo::FieldsBorrowedBy(fields_borrowed_by1) => {
                        match &state.borrows[nonce] {
                            BorrowInfo::BorrowedBy(borrowed_by2) => {
                                borrows.insert(
                                    *nonce,
                                    BorrowInfo::BorrowedBy(Self::get_union_of_sets(
                                        borrowed_by2,
                                        fields_borrowed_by1,
                                    )),
                                );
                            }
                            BorrowInfo::FieldsBorrowedBy(fields_borrowed_by2) => {
                                borrows.insert(
                                    *nonce,
                                    BorrowInfo::FieldsBorrowedBy(Self::get_union_of_maps(
                                        fields_borrowed_by1,
                                        fields_borrowed_by2,
                                    )),
                                );
                            }
                        }
                    }
                }
            } else {
                borrows.insert(*nonce, borrow_info.clone());
            }
        }
        for (nonce, borrow_info) in &state.borrows {
            if !borrows.contains_key(nonce) {
                borrows.insert(*nonce, borrow_info.clone());
            }
        }

        let partition = self.partition.join(&state.partition);

        let next_state = AbstractState {
            locals,
            globals,
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
