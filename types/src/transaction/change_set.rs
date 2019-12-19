// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{contract_event::ContractEvent, write_set::WriteSet};
use libra_crypto::HashValue;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ChangeSet {
    // The number of total WriteSet that has been applied to the libra network.
    sequence_number: u64,
    write_set: WriteSet,
    events: Vec<ContractEvent>,
}

impl ChangeSet {
    pub fn new(sequence_number: u64, write_set: WriteSet, events: Vec<ContractEvent>) -> Self {
        Self {
            sequence_number,
            write_set,
            events,
        }
    }

    pub fn sequence_number(&self) -> u64 {
        self.sequence_number
    }

    pub fn into_inner(self) -> (WriteSet, Vec<ContractEvent>) {
        (self.write_set, self.events)
    }

    pub fn write_set(&self) -> &WriteSet {
        &self.write_set
    }

    pub fn events(&self) -> &[ContractEvent] {
        &self.events
    }

    pub fn get_hash(&self) -> HashValue {
        let bytes = lcs::to_bytes(&self).unwrap();
        HashValue::from_sha3_256(&bytes)
    }
}
