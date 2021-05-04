// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_types::{
    access_path::AccessPath, transaction::TransactionOutput, vm_status::StatusCode,
    vm_status::VMStatus,
};

use crate::create_access_path;

use move_core_types::{
    account_address::AccountAddress,
    language_storage::{ModuleId, StructTag},
};
use move_vm_runtime::data_cache::{MoveStorage, RefBytes};

use std::cell::UnsafeCell;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};

unsafe impl Send for OutcomeArray {}
unsafe impl Sync for OutcomeArray {}

pub struct OutcomeArray {
    results: Vec<UnsafeCell<(VMStatus, TransactionOutput)>>,

    success_num: AtomicUsize,
    failure_num: AtomicUsize,
}

impl OutcomeArray {
    pub fn new(len: usize) -> OutcomeArray {
        OutcomeArray {
            results: (0..len)
                .map(|_| {
                    UnsafeCell::new((
                        VMStatus::Executed,
                        TransactionOutput::new(
                            WriteSet::default(),
                            vec![],
                            0,
                            TransactionStatus::Retry,
                        ),
                    ))
                })
                .collect(),

            success_num: AtomicUsize::new(0),
            failure_num: AtomicUsize::new(0),
        }
    }

    pub fn set_result(&self, idx: usize, res: (VMStatus, TransactionOutput), success: bool) {
        // Only one thread can write at the time, so just set it.

        let entry = &self.results[idx];
        unsafe {
            let mut_entry = &mut *entry.get();
            *mut_entry = res;
        }

        // #[cfg(test)]
        {
            if success {
                self.success_num.fetch_add(1, Ordering::Relaxed);
            } else {
                self.failure_num.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    pub fn get_stats(&self) -> (usize, usize) {
        return (
            self.success_num.load(Ordering::Relaxed),
            self.failure_num.load(Ordering::Relaxed),
        );
    }

    pub fn get_all_results(self) -> Result<Vec<(VMStatus, TransactionOutput)>, VMStatus> {
        let results = self.results;
        Ok(unsafe {
            // This is safe since UnsafeCell has no runtime representation.
            std::mem::transmute::<
                Vec<UnsafeCell<(VMStatus, TransactionOutput)>>,
                Vec<(VMStatus, TransactionOutput)>,
            >(results)
        })
    }
}

unsafe impl Send for WritesPlaceholder {}
unsafe impl Sync for WritesPlaceholder {}

/// A structure that holds placeholders for each write to the database
//
//  The structure is created by one thread creating the scheduling, and
//  at that point it is used as a &mut by that single thread.
//
//  Then it is passed to all threads executing as a shared reference. At
//  this point only a single thread must write to any entry, and others
//  can read from it. Only entries are mutated using interior mutability,
//  but no entries can be added or deleted.
//
pub(crate) struct WritesPlaceholder {
    data: HashMap<AccessPath, BTreeMap<usize, WriteVersionValue>>,
}

use diem_types::{transaction::TransactionStatus, write_set::WriteSet};

impl WritesPlaceholder {
    pub fn new_from(
        data: HashMap<AccessPath, BTreeMap<usize, WriteVersionValue>>,
    ) -> WritesPlaceholder {
        WritesPlaceholder { data }
    }

    /* How to get the latest version of the DB that changed.

    pub fn get_change_set(&self) -> Vec<(AccessPath, Option<Vec<u8>>)> {
        let mut change_set = Vec::with_capacity(self.data.len());
        for (k,_) in self.data.iter() {
            let val = self.read(k, usize::MAX).unwrap();
            change_set.push((k.clone(), val.clone()));
        }
        change_set
    }

    */

    pub fn write(&self, key: &AccessPath, version: usize, data: Option<Vec<u8>>) -> Result<(), ()> {
        // By construction there will only be a single writer, before the
        // write there will be no readers on the variable.
        // So it is safe to go ahead and write without any further check.
        // Then update the flag to enable reads.

        let entry = self
            .data
            .get(key)
            .ok_or_else(|| ())?
            .get(&version)
            .ok_or_else(|| ())?;

        #[cfg(test)]
        {
            // Test the invariant holds
            let flag = entry.flag.load(Ordering::Acquire);
            if flag != FLAG_UNASSIGNED {
                panic!("Cannot write twice to same entry.");
            }
        }

        unsafe {
            let val = &mut *entry.data.get();
            *val = data;
        }

        entry.flag.store(FLAG_DONE, Ordering::Release);
        Ok(())
    }

    pub fn skip_if_not_set(&self, key: &AccessPath, version: usize) -> Result<(), ()> {
        // We only write or skip once per entry
        // So it is safe to go ahead and just do it.
        let entry = self
            .data
            .get(key)
            .ok_or_else(|| ())?
            .get(&version)
            .ok_or_else(|| ())?;

        // Test the invariant holds
        let flag = entry.flag.load(Ordering::Acquire);
        if flag == FLAG_UNASSIGNED {
            entry.flag.store(FLAG_SKIP, Ordering::Release);
        }

        Ok(())
    }

    pub fn skip(&self, key: &AccessPath, version: usize) -> Result<(), ()> {
        // We only write or skip once per entry
        // So it is safe to go ahead and just do it.
        let entry = self
            .data
            .get(key)
            .ok_or_else(|| ())?
            .get(&version)
            .ok_or_else(|| ())?;

        #[cfg(test)]
        {
            // Test the invariant holds
            let flag = entry.flag.load(Ordering::Acquire);
            if flag != FLAG_UNASSIGNED {
                panic!("Cannot write twice to same entry.");
            }
        }

        entry.flag.store(FLAG_SKIP, Ordering::Release);
        Ok(())
    }

    pub fn read(
        &self,
        key: &AccessPath,
        version: usize,
    ) -> Result<&Option<Vec<u8>>, Option<usize>> {
        // Get the smaller key
        let tree = self.data.get(key).ok_or_else(|| None)?;

        let mut iter = tree.range(0..version);

        while let Some((entry_key, entry_val)) = iter.next_back() {
            if *entry_key < version {
                let flag = entry_val.flag.load(Ordering::Acquire);

                // Return this key, must wait.
                if flag == FLAG_UNASSIGNED {
                    return Err(Some(*entry_key));
                }

                // If we are to skip this entry, pick the next one
                if flag == FLAG_SKIP {
                    continue;
                }

                // The entry is populated so return its contents
                if flag == FLAG_DONE {
                    let data_read_ref = unsafe { &*entry_val.data.get() };
                    return Ok(data_read_ref);
                }

                unreachable!();
            }
        }

        Err(None)
    }
}

#[derive(Eq, Ord, PartialEq, PartialOrd, Clone, Debug)]
pub(crate) struct WriteVersionKey {
    path: AccessPath,
    version: usize,
}

const FLAG_UNASSIGNED: usize = 0;
const FLAG_DONE: usize = 2;
const FLAG_SKIP: usize = 3;

#[cfg_attr(any(target_arch = "x86_64"), repr(align(128)))]
pub(crate) struct WriteVersionValue {
    flag: AtomicUsize,
    data: UnsafeCell<Option<Vec<u8>>>,
}

impl WriteVersionValue {
    pub fn new() -> WriteVersionValue {
        WriteVersionValue {
            flag: AtomicUsize::new(FLAG_UNASSIGNED),
            data: UnsafeCell::new(None),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use diem_types::{access_path::AccessPath, account_address::AccountAddress};

    #[test]
    fn create_write_read_placeholder_struct() {
        let ap1 = AccessPath {
            address: AccountAddress::new([1u8; AccountAddress::LENGTH]),
            path: b"/foo/b".to_vec(),
        };
        let ap2 = AccessPath {
            address: AccountAddress::new([2u8; AccountAddress::LENGTH]),
            path: b"/foo/c".to_vec(),
        };

        let mut data = HashMap::new();
        for (path, version) in &[(ap1.clone(), 10), (ap2.clone(), 10), (ap2.clone(), 20)] {
            data.entry(path.clone())
                .or_insert(BTreeMap::new())
                .insert(*version, WriteVersionValue::new());
        }

        let mut placeholder = WritesPlaceholder::new_from(data, 2);

        assert_eq!(2, placeholder.len());

        // Reads that should go the the DB return Err(None)
        let r1 = placeholder.read(&ap1, 5);
        assert_eq!(Err(None), r1);

        // Reads at a version return the previous versions, not this
        // version.
        let r1 = placeholder.read(&ap1, 10);
        assert_eq!(Err(None), r1);

        // Check reads into non-ready structs return the Err(entry)

        // Reads at a higher version return the previous version
        let r1 = placeholder.read(&ap1, 15);
        assert_eq!(Err(Some(10)), r1);

        // Writes populate the entry
        let w1 = placeholder.write(&ap1, 10, Some(vec![0, 0, 0]));

        // Subsequent higher reads read this entry
        let r1 = placeholder.read(&ap1, 15);
        assert_eq!(Ok(Some(vec![0, 0, 0])), r1);

        // Set skip works
        let w1 = placeholder.skip(&ap1, 20);

        // Higher reads skip this entry
        let r1 = placeholder.read(&ap1, 25);
        assert_eq!(Ok(Some(vec![0, 0, 0])), r1);
    }
}

use diem_state_view::{StateView, StateViewId};
use std::{thread, time};

const ONE_MILLISEC: std::time::Duration = time::Duration::from_millis(10);

pub(crate) struct VersionedStateView<'view> {
    version: usize,
    base_view: &'view dyn StateView,
    placeholder: &'view WritesPlaceholder,
}

impl<'view> VersionedStateView<'view> {
    pub fn new(
        version: usize,
        base_view: &'view dyn StateView,
        placeholder: &'view WritesPlaceholder,
    ) -> VersionedStateView<'view> {
        VersionedStateView {
            version,
            base_view,
            placeholder,
        }
    }

    pub fn will_read_block(&self, access_path: &AccessPath) -> bool {
        let read = self.placeholder.read(access_path, self.version);
        if let Err(Some(_)) = read {
            return true;
        }
        return false;
    }

    fn get_ref(&self, access_path: &AccessPath) -> anyhow::Result<Option<RefBytes>> {
        // println!("V{} {}", self.version, access_path);

        let mut loop_iterations = 0;
        loop {
            let read = self.placeholder.read(access_path, self.version);

            // Go to the Database
            if let Err(None) = read {
                return self
                    .base_view
                    .get(access_path)
                    .map(|v| v.map(|v1| RefBytes::Bytes(v1)));
            }

            // Read is a success
            if let Ok(data) = read {
                return Ok(data.as_ref().map(|v| RefBytes::Ref(v)));
            }

            loop_iterations += 1;
            if loop_iterations < 500 {
                ::std::sync::atomic::spin_loop_hint();
            } else {
                thread::sleep(ONE_MILLISEC);
            }
        }
    }
}

impl<'block> StateView for VersionedStateView<'block> {
    // Get some data either through the cache or the `StateView` on a cache miss.
    fn get(&self, access_path: &AccessPath) -> anyhow::Result<Option<Vec<u8>>> {
        // println!("V{} {}", self.version, access_path);

        let mut loop_iterations = 0;
        loop {
            let read = self.placeholder.read(access_path, self.version);

            // Go to the Database
            if let Err(None) = read {
                return self.base_view.get(access_path);
            }

            // Read is a success
            if let Ok(data) = read {
                return Ok(data.clone());
            }

            loop_iterations += 1;
            if loop_iterations < 500 {
                ::std::sync::atomic::spin_loop_hint();
            } else {
                thread::sleep(ONE_MILLISEC);
            }
        }
    }

    fn is_genesis(&self) -> bool {
        self.base_view.is_genesis()
    }

    fn id(&self) -> StateViewId {
        self.base_view.id()
    }
}

impl<'view> MoveStorage for VersionedStateView<'view> {
    fn get_module(&self, module_id: &ModuleId) -> VMResult<Option<Vec<u8>>> {
        // REVIEW: cache this?
        let ap = AccessPath::from(module_id);
        self.get(&ap)
            .map_err(|_| PartialVMError::new(StatusCode::STORAGE_ERROR).finish(Location::Undefined))
    }

    fn get_resource(
        &self,
        address: &AccountAddress,
        struct_tag: &StructTag,
    ) -> PartialVMResult<Option<Vec<u8>>> {
        let ap = create_access_path(*address, struct_tag);
        self.get(&ap)
            .map_err(|_| PartialVMError::new(StatusCode::STORAGE_ERROR))
    }

    fn get_resource_ref(
        &self,
        address: &AccountAddress,
        struct_tag: &StructTag,
    ) -> PartialVMResult<Option<RefBytes>> {
        let ap = create_access_path(*address, struct_tag);
        self.get_ref(&ap)
            .map_err(|_| PartialVMError::new(StatusCode::STORAGE_ERROR))
    }
}

use move_binary_format::errors::{Location, PartialVMError, PartialVMResult, VMResult};
use std::cell::RefCell;
use std::collections::HashMap;

pub struct SingleThreadReadCache<'view> {
    base_view: &'view dyn StateView,
    cache: RefCell<HashMap<AccessPath, Option<Vec<u8>>>>,
}

impl<'view> SingleThreadReadCache<'view> {
    pub fn new(base_view: &'view dyn StateView) -> SingleThreadReadCache<'view> {
        SingleThreadReadCache {
            base_view,
            cache: RefCell::new(HashMap::new()),
        }
    }
}

impl<'view> StateView for SingleThreadReadCache<'view> {
    // Get some data either through the cache or the `StateView` on a cache miss.
    fn get(&self, access_path: &AccessPath) -> anyhow::Result<Option<Vec<u8>>> {
        if self.cache.borrow().contains_key(access_path) {
            Ok(self.cache.borrow().get(access_path).unwrap().clone())
        } else {
            let val = self.base_view.get(access_path)?;
            self.cache
                .borrow_mut()
                .insert(access_path.clone(), val.clone());
            Ok(val)
        }
    }

    fn is_genesis(&self) -> bool {
        self.base_view.is_genesis()
    }

    fn id(&self) -> StateViewId {
        self.base_view.id()
    }
}
