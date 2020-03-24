// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
//! Scratchpad for on chain values during the execution.

use libra_logger::prelude::*;
use libra_state_view::StateView;
use libra_types::{
    access_path::AccessPath,
    language_storage::ModuleId,
    on_chain_config::ConfigStorage,
    vm_error::{StatusCode, VMStatus},
    write_set::{WriteOp, WriteSet, WriteSetMut},
};
use move_vm_types::{
    loaded_data::types::{StructType, Type},
    values::{GlobalValue, Value},
};
use std::{collections::btree_map::BTreeMap, mem::replace};
use vm::errors::*;

/// The wrapper around the StateVersionView for the block.
/// It keeps track of the value that have been changed during execution of a block.
/// It's effectively the write set for the block.
pub struct BlockDataCache<'block> {
    data_view: &'block dyn StateView,
    // TODO: an AccessPath corresponds to a top level resource but that may not be the
    // case moving forward, so we need to review this.
    // Also need to relate this to a ResourceKey.
    data_map: BTreeMap<AccessPath, Vec<u8>>,
}

impl<'block> BlockDataCache<'block> {
    pub fn new(data_view: &'block dyn StateView) -> Self {
        BlockDataCache {
            data_view,
            data_map: BTreeMap::new(),
        }
    }

    pub fn get(&self, access_path: &AccessPath) -> VMResult<Option<Vec<u8>>> {
        match self.data_map.get(access_path) {
            Some(data) => Ok(Some(data.clone())),
            None => match self.data_view.get(&access_path) {
                Ok(remote_data) => Ok(remote_data),
                // TODO: should we forward some error info?
                Err(_) => {
                    crit!("[VM] Error getting data from storage for {:?}", access_path);
                    Err(VMStatus::new(StatusCode::STORAGE_ERROR))
                }
            },
        }
    }

    pub fn push_write_set(&mut self, write_set: &WriteSet) {
        for (ref ap, ref write_op) in write_set.iter() {
            match write_op {
                WriteOp::Value(blob) => {
                    self.data_map.insert(ap.clone(), blob.clone());
                }
                WriteOp::Deletion => {
                    self.data_map.remove(ap);
                }
            }
        }
    }

    pub fn is_genesis(&self) -> bool {
        self.data_view.is_genesis() && self.data_map.is_empty()
    }
}

/// Trait for the StateVersionView or a mock implementation of the remote cache.
/// Unit and integration tests should use this to mock implementations of "storage"
pub trait RemoteCache {
    fn get(&self, access_path: &AccessPath) -> VMResult<Option<Vec<u8>>>;
}

impl ConfigStorage for Box<&dyn RemoteCache> {
    fn fetch_config(&self, access_path: AccessPath) -> Option<Vec<u8>> {
        self.get(&access_path)
            .ok()?
            .and_then(|bytes| lcs::from_bytes::<Vec<u8>>(&bytes).ok())
    }
}

impl<'block> RemoteCache for BlockDataCache<'block> {
    fn get(&self, access_path: &AccessPath) -> VMResult<Option<Vec<u8>>> {
        BlockDataCache::get(self, access_path)
    }
}

/// Adapter to convert a `StateView` into a `RemoteCache`.
pub struct RemoteStorage<'a>(&'a dyn StateView);

impl<'a> RemoteStorage<'a> {
    pub fn new(state_store: &'a dyn StateView) -> Self {
        Self(state_store)
    }
}

impl<'a> RemoteCache for RemoteStorage<'a> {
    fn get(&self, access_path: &AccessPath) -> VMResult<Option<Vec<u8>>> {
        self.0
            .get(access_path)
            .map_err(|_| VMStatus::new(StatusCode::STORAGE_ERROR))
    }
}

/// Global cache for a transaction.
/// Materializes Values from the RemoteCache and keeps an Rc to them.
/// It also implements the opcodes that talk to storage and gives the proper guarantees of
/// reference lifetime.
/// Dirty objects are serialized and returned in make_write_set
pub struct TransactionDataCache<'txn> {
    // TODO: an AccessPath corresponds to a top level resource but that may not be the
    // case moving forward, so we need to review this.
    // Also need to relate this to a ResourceKey.
    data_map: BTreeMap<AccessPath, Option<(StructType, GlobalValue)>>,
    module_map: BTreeMap<ModuleId, Vec<u8>>,
    data_cache: &'txn dyn RemoteCache,
}

impl<'txn> TransactionDataCache<'txn> {
    pub fn new(data_cache: &'txn dyn RemoteCache) -> Self {
        TransactionDataCache {
            data_cache,
            data_map: BTreeMap::new(),
            module_map: BTreeMap::new(),
        }
    }

    pub fn exists_module(&self, m: &ModuleId) -> bool {
        self.module_map.contains_key(m) || {
            let ap = AccessPath::from(m);
            matches!(self.data_cache.get(&ap), Ok(Some(_)))
        }
    }

    pub fn load_module(&self, module: &ModuleId) -> VMResult<Vec<u8>> {
        match self.module_map.get(module) {
            Some(bytes) => Ok(bytes.clone()),
            None => {
                let ap = AccessPath::from(module);
                self.data_cache
                    .get(&ap)
                    .and_then(|data| data.ok_or_else(|| VMStatus::new(StatusCode::LINKER_ERROR)))
            }
        }
    }

    pub fn publish_module(&mut self, m: ModuleId, bytes: Vec<u8>) -> VMResult<()> {
        self.module_map.insert(m, bytes);
        Ok(())
    }

    pub fn publish_resource(
        &mut self,
        ap: &AccessPath,
        g: (StructType, GlobalValue),
    ) -> VMResult<()> {
        self.data_map.insert(ap.clone(), Some(g));
        Ok(())
    }

    // Retrieve data from the local cache or loads it from the remote cache into the local cache.
    // All operations on the global data are based on this API and they all load the data
    // into the cache.
    // TODO: this may not be the most efficient model because we always load data into the
    // cache even when that would not be strictly needed. Review once we have the whole story
    // working
    pub(crate) fn load_data(
        &mut self,
        ap: &AccessPath,
        ty: StructType,
    ) -> VMResult<&mut Option<(StructType, GlobalValue)>> {
        if !self.data_map.contains_key(ap) {
            match self.data_cache.get(ap)? {
                Some(bytes) => {
                    let res =
                        Value::simple_deserialize(&bytes, Type::Struct(Box::new(ty.clone())))?;
                    let gr = GlobalValue::new(res)?;
                    self.data_map.insert(ap.clone(), Some((ty, gr)));
                }
                None => {
                    return Err(vm_error(Location::new(), StatusCode::MISSING_DATA));
                }
            };
        }
        Ok(self.data_map.get_mut(ap).expect("data must exist"))
    }

    /// Make a write set from the updated (dirty, deleted) global resources along with
    /// to-be-published modules.
    /// Consume the TransactionDataCache and must be called at the end of a transaction.
    /// This also ends up checking that reference count around global resources is correct
    /// at the end of the transactions (all ReleaseRef are properly called)
    pub fn make_write_set(&mut self) -> VMResult<WriteSet> {
        if self.data_map.len() + self.module_map.len() > usize::max_value() {
            return Err(vm_error(Location::new(), StatusCode::INVALID_DATA));
        }

        let mut sorted_ws: BTreeMap<AccessPath, WriteOp> = BTreeMap::new();

        let data_map = replace(&mut self.data_map, BTreeMap::new());
        for (key, global_val) in data_map {
            match global_val {
                Some((layout, global_val)) => {
                    if !global_val.is_clean()? {
                        // into_owned_struct will check if all references are properly released
                        // at the end of a transaction
                        let data = global_val.into_owned_struct()?;
                        let blob = match data.simple_serialize(&layout) {
                            Some(blob) => blob,
                            None => {
                                return Err(vm_error(
                                    Location::new(),
                                    StatusCode::VALUE_SERIALIZATION_ERROR,
                                ))
                            }
                        };
                        sorted_ws.insert(key, WriteOp::Value(blob));
                    }
                }
                None => {
                    sorted_ws.insert(key, WriteOp::Deletion);
                }
            }
        }

        let module_map = replace(&mut self.module_map, BTreeMap::new());
        for (module_id, module) in module_map {
            sorted_ws.insert((&module_id).into(), WriteOp::Value(module));
        }

        let mut write_set = WriteSetMut::new(Vec::new());
        for (key, value) in sorted_ws {
            write_set.push((key, value));
        }
        write_set
            .freeze()
            .map_err(|_| vm_error(Location::new(), StatusCode::DATA_FORMAT_ERROR))
    }

    /// Flush out the cache and restart from a clean state
    pub fn clear(&mut self) {
        self.data_map.clear();
        self.module_map.clear();
    }
}
