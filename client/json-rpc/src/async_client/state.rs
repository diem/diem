// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::async_client::JsonRpcResponse;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct State {
    pub chain_id: u8,
    pub version: u64,
    pub timestamp_usecs: u64,
}

impl State {
    pub fn from_response(resp: &JsonRpcResponse) -> Self {
        Self {
            chain_id: resp.diem_chain_id,
            version: resp.diem_ledger_version,
            timestamp_usecs: resp.diem_ledger_timestampusec,
        }
    }
}

pub struct StateManager {
    last_known_state: std::sync::RwLock<Option<State>>,
}

impl Default for StateManager {
    fn default() -> Self {
        Self {
            last_known_state: std::sync::RwLock::new(None),
        }
    }
}

impl StateManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn last_known_state(&self) -> Option<State> {
        let data = self.last_known_state.read().unwrap();
        data.clone()
    }

    pub fn update_state(&self, resp_state: State) -> bool {
        let mut state_writer = self.last_known_state.write().unwrap();
        if let Some(state) = &*state_writer {
            if &resp_state < state {
                return false;
            }
        }
        *state_writer = Some(resp_state);
        true
    }
}
