// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Support for mocking the Libra data store.

use crate::account::AccountData;
use failure::prelude::*;
use lazy_static::lazy_static;
use libra_state_view::StateView;
use libra_types::{
    access_path::AccessPath,
    language_storage::ModuleId,
    transaction::{SignedTransaction, TransactionPayload},
    write_set::{WriteOp, WriteSet},
};
use prost::Message;
use std::convert::TryFrom;
use std::{collections::HashMap, fs::File, io::prelude::*, path::PathBuf};
use vm::{errors::*, CompiledModule};
use vm_runtime::data_cache::RemoteCache;
use walkdir::WalkDir;

lazy_static! {
    /// The write set encoded in the genesis transaction.
    pub static ref GENESIS_WRITE_SET: WriteSet = {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.pop();
        path.push("vm/vm-genesis/genesis/genesis.blob");

        load_genesis(path)
    };

    pub static ref TESTNET_GENESIS: Vec<WriteSet> = {
        let mut base_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        base_path.pop();
        base_path.pop();
        base_path.push("terraform/validator-sets/");

        let files = WalkDir::new(base_path)
            .into_iter()
            .filter_map(|f| f.ok())
            .filter(|f| f.path().ends_with("genesis.blob"))
            .map(|f| load_genesis(f.path().to_path_buf()))
            .collect::<Vec<_>>();

        assert!(!files.is_empty());
        files
    };
}

fn load_genesis(path: PathBuf) -> WriteSet {
    let mut f = File::open(&path).unwrap();
    let mut bytes = vec![];
    f.read_to_end(&mut bytes).unwrap();
    let txn = SignedTransaction::try_from(
        libra_types::proto::types::SignedTransaction::decode(&bytes).unwrap(),
    )
    .unwrap();
    match txn.payload() {
        TransactionPayload::WriteSet(ws) => ws.clone(),
        _ => panic!("Expected writeset txn in genesis txn"),
    }
}

/// An in-memory implementation of [`StateView`] and [`RemoteCache`] for the VM.
///
/// Tests use this to set up state, and pass in a reference to the cache whenever a `StateView` or
/// `RemoteCache` is needed.
#[derive(Debug, Default)]
pub struct FakeDataStore {
    data: HashMap<AccessPath, Vec<u8>>,
}

impl FakeDataStore {
    /// Creates a new `FakeDataStore` with the provided initial data.
    pub fn new(data: HashMap<AccessPath, Vec<u8>>) -> Self {
        FakeDataStore { data }
    }

    /// Adds a [`WriteSet`] to this data store.
    pub fn add_write_set(&mut self, write_set: &WriteSet) {
        for (access_path, write_op) in write_set {
            match write_op {
                WriteOp::Value(blob) => {
                    self.set(access_path.clone(), blob.clone());
                }
                WriteOp::Deletion => {
                    self.remove(access_path);
                }
            }
        }
    }

    /// Sets a (key, value) pair within this data store.
    ///
    /// Returns the previous data if the key was occupied.
    pub fn set(&mut self, access_path: AccessPath, data_blob: Vec<u8>) -> Option<Vec<u8>> {
        self.data.insert(access_path, data_blob)
    }

    /// Deletes a key from this data store.
    ///
    /// Returns the previous data if the key was occupied.
    pub fn remove(&mut self, access_path: &AccessPath) -> Option<Vec<u8>> {
        self.data.remove(access_path)
    }

    /// Adds an [`AccountData`] to this data store.
    pub fn add_account_data(&mut self, account_data: &AccountData) {
        match account_data.to_resource().simple_serialize() {
            Some(blob) => {
                self.set(account_data.make_access_path(), blob);
            }
            None => panic!("can't create Account data"),
        }
    }

    /// Adds a [`CompiledModule`] to this data store.
    ///
    /// Does not do any sort of verification on the module.
    pub fn add_module(&mut self, module_id: &ModuleId, module: &CompiledModule) {
        let access_path = AccessPath::from(module_id);
        let mut blob = vec![];
        module
            .serialize(&mut blob)
            .expect("serializing this module should work");
        self.set(access_path, blob);
    }
}

// This is used by the `execute_block` API.
// TODO: only the "sync" get is implemented
impl StateView for FakeDataStore {
    fn get(&self, access_path: &AccessPath) -> Result<Option<Vec<u8>>> {
        // Since the data is in-memory, it can't fail.
        Ok(self.data.get(access_path).cloned())
    }

    fn multi_get(&self, _access_paths: &[AccessPath]) -> Result<Vec<Option<Vec<u8>>>> {
        unimplemented!();
    }

    fn is_genesis(&self) -> bool {
        self.data.is_empty()
    }
}

// This is used by the `process_transaction` API.
impl RemoteCache for FakeDataStore {
    fn get(&self, access_path: &AccessPath) -> VMResult<Option<Vec<u8>>> {
        Ok(StateView::get(self, access_path).expect("it should not error"))
    }
}
