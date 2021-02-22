// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, Result};
use diem_state_view::StateView;
use diem_types::{
    access_path::AccessPath, transaction::TransactionOutput, vm_status::StatusCode,
    write_set::WriteOp,
};
use move_binary_format::errors::*;
use move_core_types::{
    account_address::AccountAddress,
    language_storage::{ModuleId, StructTag},
};
use move_vm_runtime::data_cache::MoveStorage;
use mvhashmap::MVHashMap;
use std::{borrow::Cow, convert::AsRef, thread, time::Duration, collections::HashSet};

pub struct VersionedDataCache(MVHashMap<AccessPath, Vec<u8>>);

pub struct VersionedStateView<'view> {
    version: usize,
    base_view: &'view dyn StateView,
    placeholder: &'view VersionedDataCache,
}

const ONE_MILLISEC: Duration = Duration::from_millis(10);

impl VersionedDataCache {
    pub fn new(write_sequence: Vec<(AccessPath, usize)>) -> (usize, Self) {
        let (max_dependency_length, mv_hashmap) = MVHashMap::new_from_parallel(write_sequence);
        (max_dependency_length, VersionedDataCache(mv_hashmap))
    }

    pub fn set_skip_all(&self, version: usize, estimated_writes: impl Iterator<Item = AccessPath>) {
        // Put skip in all entires.
        for w in estimated_writes {
            // It should be safe to unwrap here since the MVMap was construted using
            // this estimated writes. If not it is a bug.
            self.as_ref().skip(&w, version).unwrap();
        }
    }

    pub fn apply_output(
        &self,
        output: &TransactionOutput,
        version: usize,
        estimated_writes: impl Iterator<Item = AccessPath>,
    ) -> Result<(), ()> {
        if !output.status().is_discarded() {
            // First make sure all outputs have entries in the MVMap
            let estimated_writes_hs: HashSet<_> = estimated_writes.collect();
            for (k, _) in output.write_set() {
                if !estimated_writes_hs.contains(k) {
                    println!("Missing entry: {:?}", k);

                    // Put skip in all entires.
                    self.set_skip_all(version, estimated_writes_hs.into_iter());
                    return Err(());
                }
            }

            // We are sure all entries are present -- now update them all
            for (k, v) in output.write_set() {
                let val = match v {
                    WriteOp::Deletion => None,
                    WriteOp::Value(data) => Some(data.clone()),
                };

                // Safe because we checked that the entry exists
                self.as_ref().write(k, version, val).unwrap();
            }

            // If any entries are not updated, write a 'skip' flag into them
            for w in estimated_writes_hs {
                // It should be safe to unwrap here since the MVMap was construted using
                // this estimated writes. If not it is a bug.
                self.as_ref()
                    .skip_if_not_set(&w, version)
                    .expect("Entry must exist.");
            }
        } else {
            self.set_skip_all(version, estimated_writes);
        }
        Ok(())
    }
}

impl AsRef<MVHashMap<AccessPath, Vec<u8>>> for VersionedDataCache {
    fn as_ref(&self) -> &MVHashMap<AccessPath, Vec<u8>> {
        &self.0
    }
}

impl<'view> VersionedStateView<'view> {
    pub fn new(
        version: usize,
        base_view: &'view dyn StateView,
        placeholder: &'view VersionedDataCache,
    ) -> VersionedStateView<'view> {
        VersionedStateView {
            version,
            base_view,
            placeholder,
        }
    }

    pub fn will_read_block(&self, access_path: &AccessPath) -> bool {
        let read = self.placeholder.as_ref().read(access_path, self.version);
        if let Err(Some(_)) = read {
            return true;
        }
        return false;
    }

    fn get_bytes_ref(&self, access_path: &AccessPath) -> PartialVMResult<Option<Cow<[u8]>>> {
        let mut loop_iterations = 0;
        loop {
            let read = self.placeholder.as_ref().read(access_path, self.version);

            // Go to the Database
            if let Err(None) = read {
                return self
                    .base_view
                    .get(access_path)
                    .map(|opt| opt.map(Cow::from))
                    .map_err(|_| PartialVMError::new(StatusCode::STORAGE_ERROR));
            }

            // Read is a success
            if let Ok(data) = read {
                return Ok(data.as_ref().map(Cow::from));
            }

            loop_iterations += 1;
            if loop_iterations < 500 {
                ::std::hint::spin_loop();
            } else {
                thread::sleep(ONE_MILLISEC);
            }
        }
    }

    fn get_bytes(&self, access_path: &AccessPath) -> PartialVMResult<Option<Vec<u8>>> {
        self.get_bytes_ref(access_path)
            .map(|opt| opt.map(Cow::into_owned))
    }
}

impl<'view> MoveStorage for VersionedStateView<'view> {
    fn get_module(&self, module_id: &ModuleId) -> VMResult<Option<Vec<u8>>> {
        // REVIEW: cache this?
        let ap = AccessPath::from(module_id);
        self.get_bytes(&ap)
            .map_err(|e| e.finish(Location::Undefined))
    }

    fn get_resource(
        &self,
        address: &AccountAddress,
        struct_tag: &StructTag,
    ) -> PartialVMResult<Option<Cow<[u8]>>> {
        let ap = AccessPath::new(
            *address,
            AccessPath::resource_access_vec(struct_tag.clone()),
        );
        self.get_bytes_ref(&ap)
    }
}

impl<'view> StateView for VersionedStateView<'view> {
    fn get(&self, access_path: &AccessPath) -> Result<Option<Vec<u8>>> {
        self.get_bytes(access_path)
            .map_err(|_| anyhow!("Failed to get data from VersionedStateView"))
    }

    fn is_genesis(&self) -> bool {
        self.base_view.is_genesis()
    }
}

//
// pub struct SingleThreadReadCache<'view> {
//     base_view : &'view dyn StateView,
//     cache : RefCell<HashMap<AccessPath, Option<Vec<u8>>>>,
// }
//
// impl<'view> SingleThreadReadCache<'view> {
//     pub fn new(base_view: &'view StateView) -> SingleThreadReadCache<'view> {
//         SingleThreadReadCache {
//             base_view,
//             cache : RefCell::new(HashMap::new()),
//         }
//     }
// }
//
// impl<'view> StateView for SingleThreadReadCache<'view> {
//     // Get some data either through the cache or the `StateView` on a cache miss.
//     fn get(&self, access_path: &AccessPath) -> anyhow::Result<Option<Vec<u8>>> {
//         if self.cache.borrow().contains_key(access_path) {
//             Ok(self.cache.borrow().get(access_path).unwrap().clone())
//         }
//         else {
//             let val = self.base_view.get(access_path)?;
//             self.cache.borrow_mut().insert(access_path.clone(), val.clone());
//             Ok(val)
//         }
//     }
//
//     fn multi_get(&self, _access_paths: &[AccessPath]) -> anyhow::Result<Vec<Option<Vec<u8>>>> {
//         unimplemented!()
//     }
//
//     fn is_genesis(&self) -> bool {
//         self.base_view.is_genesis()
//     }
// }
