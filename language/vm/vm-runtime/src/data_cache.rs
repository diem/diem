// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
//! Scratchpad for on chain values during the execution.

use libra_logger::prelude::*;
use libra_state_view::StateView;
use libra_types::{
    access_path::AccessPath,
    language_storage::ModuleId,
    vm_error::{sub_status, StatusCode, VMStatus},
    write_set::{WriteOp, WriteSet, WriteSetMut},
};
use std::{collections::btree_map::BTreeMap, mem::replace};
use vm::errors::*;
use vm_runtime_types::{
    loaded_data::struct_def::StructDef,
    value::{GlobalRef, Value},
};

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

impl<'block> RemoteCache for BlockDataCache<'block> {
    fn get(&self, access_path: &AccessPath) -> VMResult<Option<Vec<u8>>> {
        BlockDataCache::get(self, access_path)
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
    data_map: BTreeMap<AccessPath, GlobalRef>,
    data_cache: &'txn dyn RemoteCache,
}

impl<'txn> TransactionDataCache<'txn> {
    pub fn new(data_cache: &'txn dyn RemoteCache) -> Self {
        TransactionDataCache {
            data_cache,
            data_map: BTreeMap::new(),
        }
    }

    pub fn exists_module(&self, m: &ModuleId) -> bool {
        let ap = AccessPath::from(m);
        self.data_map.contains_key(&ap) || {
            match self.data_cache.get(&ap) {
                Ok(Some(_)) => true,
                _ => false,
            }
        }
    }

    pub fn publish_resource(&mut self, ap: &AccessPath, root: GlobalRef) -> VMResult<()> {
        self.data_map.insert(ap.clone(), root);
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
        def: StructDef,
    ) -> VMResult<&mut GlobalRef> {
        if !self.data_map.contains_key(ap) {
            match self.data_cache.get(ap)? {
                Some(bytes) => {
                    let res = Value::simple_deserialize(&bytes, def)?;
                    let new_root = GlobalRef::make_root(ap.clone(), res);
                    self.data_map.insert(ap.clone(), new_root);
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
    pub fn make_write_set(
        &mut self,
        to_be_published_modules: Vec<(ModuleId, Vec<u8>)>,
    ) -> VMResult<WriteSet> {
        let mut write_set = WriteSetMut::new(Vec::new());
        let data_map = replace(&mut self.data_map, BTreeMap::new());
        for (key, global_ref) in data_map {
            if !global_ref.is_clean() {
                // if there are pending references get_data() returns None
                // this is the check at the end of a transaction to verify all references
                // are properly released
                let deleted = global_ref.is_deleted();
                if let Some(data) = global_ref.get_data() {
                    if deleted {
                        // The write set will not grow to this size due to the gas limit.
                        // Expressing the bound in terms of the gas limit is impractical
                        // for MIRAI to check to we set a safe upper bound.
                        assume!(write_set.len() < usize::max_value());
                        write_set.push((key, WriteOp::Deletion));
                    } else if let Some(blob) = data.simple_serialize() {
                        // The write set will not grow to this size due to the gas limit.
                        // Expressing the bound in terms of the gas limit is impractical
                        // for MIRAI to check to we set a safe upper bound.
                        assume!(write_set.len() < usize::max_value());
                        write_set.push((key, WriteOp::Value(blob)));
                    } else {
                        return Err(vm_error(
                            Location::new(),
                            StatusCode::VALUE_SERIALIZATION_ERROR,
                        ));
                    }
                } else {
                    return Err(
                        vm_error(Location::new(), StatusCode::DYNAMIC_REFERENCE_ERROR)
                            .with_sub_status(sub_status::DRE_MISSING_RELEASEREF),
                    );
                }
            }
        }

        // Insert the code blob to the writeset.
        if write_set.len() <= usize::max_value() - to_be_published_modules.len() {
            for (key, blob) in to_be_published_modules.into_iter() {
                write_set.push(((&key).into(), WriteOp::Value(blob)));
            }
        } else {
            return Err(vm_error(Location::new(), StatusCode::INVALID_DATA));
        }

        write_set
            .freeze()
            .map_err(|_| vm_error(Location::new(), StatusCode::DATA_FORMAT_ERROR))
    }

    /// Flush out the cache and restart from a clean state
    pub fn clear(&mut self) {
        self.data_map.clear()
    }
}
