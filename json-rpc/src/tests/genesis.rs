// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use compiled_stdlib::StdLibOptions;
use diem_types::{
    account_address::AccountAddress,
    account_state_blob::AccountStateBlob,
    transaction::{Transaction, WriteSetPayload},
};
use executor::process_write_set;
use executor_types::ProofReader;
use scratchpad::SparseMerkleTree;
use std::{collections::HashMap, sync::Arc};
use vm_genesis::generate_genesis_change_set_for_testing;

// generate genesis state blob
pub fn generate_genesis_state() -> (
    HashMap<AccountAddress, AccountStateBlob>,
    Arc<SparseMerkleTree>,
) {
    let change_set = generate_genesis_change_set_for_testing(StdLibOptions::Compiled);
    let txn = Transaction::GenesisTransaction(WriteSetPayload::Direct(change_set.clone()));
    let proof_reader = ProofReader::new(HashMap::new());
    let tree: SparseMerkleTree = Default::default();
    let mut account_states = HashMap::new();

    process_write_set(
        &txn,
        &mut account_states,
        &proof_reader,
        change_set.write_set().clone(),
        &tree,
    )
    .unwrap()
}
