// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_types::{
    access_path::AccessPath,
    contract_event::ContractEvent,
    on_chain_config::ConfigStorage,
    vm_error::{StatusCode, VMStatus},
    write_set::{WriteOp, WriteSet, WriteSetMut},
};
use move_core_types::language_storage::ModuleId;
use move_vm_types::{
    data_store::DataStore,
    loaded_data::types::FatStructType,
    values::{GlobalValue, Struct, Value},
};
use std::{collections::btree_map::BTreeMap, mem::replace};
use vm::errors::*;

/// Trait for the Move VM to abstract `StateView` operations.
///
/// Can be used to define a "fake" implementation of the remote cache.
pub trait RemoteCache {
    fn get(&self, access_path: &AccessPath) -> VMResult<Option<Vec<u8>>>;
}

// TODO deprecate this in favor of `MoveStorage`
impl ConfigStorage for &dyn RemoteCache {
    fn fetch_config(&self, access_path: AccessPath) -> Option<Vec<u8>> {
        self.get(&access_path).ok()?
    }
}

/// Transaction data cache. Keep updates within a transaction so they can all be published at
/// once when the transaction succeeeds.
///
/// It also provides an implementation for the opcodes that refer to storage and gives the
/// proper guarantees of reference lifetime.
///
/// Dirty objects are serialized and returned in make_write_set.
///
/// It is a responsibility of the client to publish changes once the transaction is executed.
///
/// The Move VM takes a `DataStore` in input and this is the default and correct implementation
/// for a data store related to a transaction. Clients should create an instance of this type
/// and pass it to the Move VM.
pub struct TransactionDataCache<'txn> {
    data_map: BTreeMap<AccessPath, Option<(FatStructType, GlobalValue)>>,
    module_map: BTreeMap<ModuleId, Vec<u8>>,
    event_data: Vec<ContractEvent>,
    data_cache: &'txn dyn RemoteCache,
}

impl<'txn> TransactionDataCache<'txn> {
    /// Create a `TransactionDataCache` with a `RemoteCache` that provides access to data
    /// not updated in the transaction.
    pub fn new(data_cache: &'txn dyn RemoteCache) -> Self {
        TransactionDataCache {
            data_cache,
            data_map: BTreeMap::new(),
            module_map: BTreeMap::new(),
            event_data: vec![],
        }
    }

    /// Make a write set from the updated (dirty, deleted) global resources along with
    /// published modules.
    ///
    /// Gives all proper guarantees on lifetime of global data as well.
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

    /// Return the events that were published during the execution of the transaction.
    pub fn event_data(&self) -> &[ContractEvent] {
        &self.event_data
    }

    // Retrieve data from the local cache or loads it from the remote cache into the local cache.
    // All operations on the global data are based on this API and they all load the data
    // into the cache.
    fn load_data(
        &mut self,
        ap: &AccessPath,
        ty: &FatStructType,
    ) -> VMResult<&mut Option<(FatStructType, GlobalValue)>> {
        if !self.data_map.contains_key(ap) {
            match self.data_cache.get(ap)? {
                Some(bytes) => {
                    let res = Struct::simple_deserialize(&bytes, ty)?;
                    let gr = GlobalValue::new(Value::struct_(res))?;
                    self.data_map.insert(ap.clone(), Some((ty.clone(), gr)));
                }
                None => {
                    return Err(
                        VMStatus::new(StatusCode::MISSING_DATA).with_message(format!(
                            "Cannot find {:?}::{}::{} for Access Path: {:?}",
                            &ty.address,
                            &ty.module.as_str(),
                            &ty.name.as_str(),
                            ap
                        )),
                    );
                }
            };
        }
        Ok(self.data_map.get_mut(ap).expect("data must exist"))
    }
}

// `DataStore` implementation for the `TransactionDataCache`
impl<'a> DataStore for TransactionDataCache<'a> {
    fn publish_resource(
        &mut self,
        ap: &AccessPath,
        g: (FatStructType, GlobalValue),
    ) -> VMResult<()> {
        self.data_map.insert(ap.clone(), Some(g));
        Ok(())
    }

    fn borrow_resource(
        &mut self,
        ap: &AccessPath,
        ty: &FatStructType,
    ) -> VMResult<Option<&GlobalValue>> {
        let map_entry = self.load_data(ap, ty)?;
        Ok(map_entry.as_ref().map(|(_, g)| g))
    }

    fn move_resource_from(
        &mut self,
        ap: &AccessPath,
        ty: &FatStructType,
    ) -> VMResult<Option<GlobalValue>> {
        let map_entry = self.load_data(ap, ty)?;
        // .take() means that the entry is removed from the data map -- this marks the
        // access path for deletion.
        Ok(map_entry.take().map(|(_, g)| g))
    }

    fn load_module(&self, module: &ModuleId) -> VMResult<Vec<u8>> {
        match self.module_map.get(module) {
            Some(bytes) => Ok(bytes.clone()),
            None => {
                let ap = AccessPath::from(module);
                self.data_cache.get(&ap).and_then(|data| {
                    data.ok_or_else(|| {
                        VMStatus::new(StatusCode::LINKER_ERROR)
                            .with_message(format!("Cannot find {:?} in data cache", module))
                    })
                })
            }
        }
    }

    fn publish_module(&mut self, m: ModuleId, bytes: Vec<u8>) -> VMResult<()> {
        self.module_map.insert(m, bytes);
        Ok(())
    }

    fn exists_module(&self, m: &ModuleId) -> bool {
        self.module_map.contains_key(m) || {
            let ap = AccessPath::from(m);
            matches!(self.data_cache.get(&ap), Ok(Some(_)))
        }
    }

    fn emit_event(&mut self, event: ContractEvent) {
        self.event_data.push(event)
    }
}
