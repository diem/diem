// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//**************************************************************************************************
// Abstract state
//**************************************************************************************************

use crate::{cfgir::absint::*, parser::ast::Var};
use std::cmp::Ordering;
use std::collections::BTreeSet;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LivenessState(pub BTreeSet<Var>);

//**************************************************************************************************
// impls
//**************************************************************************************************

impl LivenessState {
    pub fn initial() -> Self {
        LivenessState(BTreeSet::new())
    }

    pub fn extend(&mut self, other: &Self) {
        self.0.extend(other.0.iter().cloned());
    }
}

impl AbstractDomain for LivenessState {
    fn join(&mut self, other: &Self) -> JoinResult {
        let before = self.0.len();
        self.extend(other);
        let after = self.0.len();
        match before.cmp(&after) {
            Ordering::Less => JoinResult::Changed,
            Ordering::Equal => JoinResult::Unchanged,
            Ordering::Greater => panic!("ICE set union made a set smaller than before"),
        }
    }
}
