// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::write_set::WriteSet;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ChangeSet {
    write_set: WriteSet,
}

impl ChangeSet {
    pub fn new(write_set: WriteSet) -> Self {
        Self { write_set }
    }

    pub fn into_inner(self) -> WriteSet {
        self.write_set
    }

    pub fn write_set(&self) -> &WriteSet {
        &self.write_set
    }
}
