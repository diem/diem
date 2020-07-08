// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ChainId(pub String);

impl Default for ChainId {
    fn default() -> Self {
        ChainId::new("default")
    }
}

impl ChainId {
    pub fn new(chain_id: &str) -> Self {
        ChainId(chain_id.to_string())
    }
}
