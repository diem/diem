// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_json_rpc_types::response::JsonRpcResponse;
use std::cmp::max;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
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

cfg_async_or_blocking! {
    use crate::{Error, Result};

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

        pub(crate) fn update_state(&self, ignore_stale: bool, req_state: Option<&State>, resp_state: &State) -> Result<()> {
            // Ensure the response is fulfilled at a more recent ledger version than
            // when we made the request, though not necessarily the globally most
            // recent version.
            if let Some(req_state) = req_state {
                if !ignore_stale && resp_state < req_state {
                    return Err(Error::stale(format!(
                        "received response with stale metadata: {:?}, expected a response more recent than: {:?}",
                        resp_state,
                        req_state,
                    )));
                }
            }

            let mut state_writer = self.last_known_state.lock().unwrap();
            let curr_state = &*state_writer;

            assert!(
                req_state <= curr_state.as_ref(),
                "request state is not an ancestor state of the current latest state: \
                 request state: {:?}, current state: {:?}",
                req_state,
                curr_state,
            );

            // Compute the most recent state
            let new_state = if let Some(curr_state) = curr_state {
                // For now, trust-on-first-use for the chain id
                if curr_state.chain_id != resp_state.chain_id {
                    return Err(Error::chain_id(curr_state.chain_id, resp_state.chain_id));
                }
                max(curr_state, resp_state)
            } else {
                resp_state
            };

            // Store the new state
            *state_writer = Some(new_state.clone());
            Ok(())
        }
    }
}
