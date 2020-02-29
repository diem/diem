// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    cfgir::absint::*,
    parser::ast::Var,
    shared::{unique_map::UniqueMap, *},
};
use move_ir_types::location::*;

//**************************************************************************************************
// Abstract state
//**************************************************************************************************

#[derive(Clone)]
pub enum LocalState {
    // Local does not have a value
    Unavailable(Loc),
    // Local has a value
    Available(Loc),
    // Available in some branches but not all. If it is a resource, cannot be assigned
    MaybeUnavailable { available: Loc, unavailable: Loc },
}

impl LocalState {
    pub fn is_available(&self) -> bool {
        match self {
            LocalState::Available(_) => true,
            LocalState::Unavailable(_) | LocalState::MaybeUnavailable { .. } => false,
        }
    }
}

#[derive(Clone)]
pub struct LocalStates {
    local_states: UniqueMap<Var, LocalState>,
}

impl LocalStates {
    pub fn initial<T>(function_arguments: &[(Var, T)], local_types: &UniqueMap<Var, T>) -> Self {
        let mut local_states = UniqueMap::new();
        for (var, _) in local_types.iter() {
            let state = LocalState::Unavailable(var.loc());
            local_states.add(var.clone(), state).unwrap();
        }
        for (var, _) in function_arguments {
            let state = LocalState::Available(var.loc());
            local_states.remove(&var);
            local_states.add(var.clone(), state).unwrap();
        }
        LocalStates { local_states }
    }

    pub fn get_state(&self, local: &Var) -> &LocalState {
        self.local_states.get(local).unwrap()
    }

    pub fn set_state(&mut self, local: Var, state: LocalState) {
        self.local_states.remove(&local);
        self.local_states.add(local, state).unwrap();
    }

    pub fn iter(&self) -> impl Iterator<Item = (Var, &LocalState)> {
        self.local_states.iter()
    }

    #[allow(dead_code)]
    pub fn debug(&self) {
        use LocalState as L;
        for (var, state) in self.iter() {
            print!("{}: ", var);
            match state {
                L::Unavailable(_) => println!("Unavailable"),
                L::Available(_) => println!("Available"),
                L::MaybeUnavailable { .. } => println!("MaybeUnavailable"),
            }
        }
    }
}

impl AbstractDomain for LocalStates {
    fn join(&mut self, other: &Self) -> JoinResult {
        use LocalState as L;
        let mut result = JoinResult::Unchanged;
        for (local, other_state) in other.local_states.iter() {
            match (self.get_state(&local), other_state) {
                // equal so nothing to do
                (L::Unavailable(_), L::Unavailable(_))
                | (L::Available(_), L::Available(_))
                | (L::MaybeUnavailable { .. }, L::MaybeUnavailable { .. }) => (),
                // if its partially assigned, stays partially assigned
                (L::MaybeUnavailable { .. }, _) => (),

                // if was partially assigned in other, its now partially assigned
                (_, L::MaybeUnavailable { .. }) => {
                    result = JoinResult::Changed;
                    let state = other_state.clone();
                    self.set_state(local.clone(), state)
                }

                // Available in one but not the other, so maybe unavailable
                (L::Available(available), L::Unavailable(unavailable))
                | (L::Unavailable(unavailable), L::Available(available)) => {
                    result = JoinResult::Changed;
                    let available = *available;
                    let unavailable = *unavailable;
                    let state = L::MaybeUnavailable {
                        available,
                        unavailable,
                    };

                    self.set_state(local.clone(), state)
                }
            }
        }
        result
    }
}
