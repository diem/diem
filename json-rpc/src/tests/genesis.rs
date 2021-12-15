// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_types::{
    account_address::AccountAddress,
    account_state_blob::AccountStateBlob,
    transaction::{Transaction, WriteSetPayload},
};
use executor::components::apply_chunk_output::process_write_set;
use std::{collections::HashMap, convert::TryInto};
use vm_genesis::{generate_genesis_change_set_for_testing, GenesisOptions};

// generate genesis state blob
pub fn generate_genesis_state() -> HashMap<AccountAddress, AccountStateBlob> {
    let change_set = generate_genesis_change_set_for_testing(GenesisOptions::Compiled);
    let txn = Transaction::GenesisTransaction(WriteSetPayload::Direct(change_set.clone()));
    let mut account_states = HashMap::new();

    process_write_set(
        Some(&txn),
        &mut account_states,
        change_set.write_set().clone(),
    )
    .unwrap();

    account_states
        .into_iter()
        .map(|(addr, state)| (addr, (&state).try_into().unwrap()))
        .collect()
}
