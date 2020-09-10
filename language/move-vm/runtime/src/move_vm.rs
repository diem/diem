// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{data_cache::RemoteCache, runtime::VMRuntime, session::Session};

pub struct MoveVM {
    runtime: VMRuntime,
}

impl MoveVM {
    pub fn new() -> Self {
        Self {
            runtime: VMRuntime::new(),
        }
    }

    pub fn new_session<'r, R: RemoteCache>(&self, remote: &'r R) -> Session<'r, '_, R> {
        self.runtime.new_session(remote)
    }
}

impl Default for MoveVM {
    fn default() -> Self {
        Self::new()
    }
}
