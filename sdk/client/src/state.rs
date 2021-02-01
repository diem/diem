// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{Error, Result};
use diem_json_rpc_types::response::JsonRpcResponse;

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

#[derive(Debug)]
pub(crate) struct StateManager {
    last_known_state: std::sync::Mutex<Option<State>>,
}

impl Clone for StateManager {
    fn clone(&self) -> Self {
        Self::default()
    }
}

impl Default for StateManager {
    fn default() -> Self {
        Self {
            last_known_state: std::sync::Mutex::new(None),
        }
    }
}

impl StateManager {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn last_known_state(&self) -> Option<State> {
        self.last_known_state.lock().unwrap().clone()
    }

    pub(crate) fn update_state(&self, resp_state: &State) -> Result<()> {
        let mut state_writer = self.last_known_state.lock().unwrap();
        if let Some(state) = &*state_writer {
            if state.chain_id != resp_state.chain_id {
                return Err(Error::chain_id(state.chain_id, resp_state.chain_id));
            }
            if resp_state < state {
                return Err(Error::stale(state, resp_state));
            }
        }
        *state_writer = Some(resp_state.clone());
        Ok(())
    }
}
