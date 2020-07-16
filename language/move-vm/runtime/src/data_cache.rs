// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::loader::Loader;

use move_core_types::{
    account_address::AccountAddress,
    language_storage::{ModuleId, TypeTag},
    value::MoveTypeLayout,
    vm_status::StatusCode,
};
use move_vm_types::{
    data_store::DataStore,
    loaded_data::runtime_types::Type,
    values::{GlobalValue, Value},
};
use std::collections::btree_map::BTreeMap;
use vm::errors::*;

/// Trait for the Move VM to abstract `StateView` operations.
///
/// Can be used to define a "fake" implementation of the remote cache.
pub trait RemoteCache {
    fn get_module(&self, module_id: &ModuleId) -> VMResult<Option<Vec<u8>>>;
    fn get_resource(
        &self,
        address: &AccountAddress,
        tag: &TypeTag,
    ) -> PartialVMResult<Option<Vec<u8>>>;
}

pub struct AccountDataCache {
    data_map: BTreeMap<Type, Option<GlobalValue>>,
    module_map: BTreeMap<ModuleId, Vec<u8>>,
}

impl AccountDataCache {
    fn new() -> Self {
        Self {
            data_map: BTreeMap::new(),
            module_map: BTreeMap::new(),
        }
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
pub(crate) struct TransactionDataCache<'r, 'l, R> {
    remote: &'r R,
    loader: &'l Loader,
    account_map: BTreeMap<AccountAddress, AccountDataCache>,
    event_data: Vec<(Vec<u8>, u64, Type, Value)>,
}

pub struct TransactionEffects {
    pub resources: Vec<(
        AccountAddress,
        Vec<(TypeTag, Option<(MoveTypeLayout, Value)>)>,
    )>,
    pub modules: Vec<(ModuleId, Vec<u8>)>,
    pub events: Vec<(Vec<u8>, u64, TypeTag, MoveTypeLayout, Value)>,
}

impl<'r, 'l, R: RemoteCache> TransactionDataCache<'r, 'l, R> {
    /// Create a `TransactionDataCache` with a `RemoteCache` that provides access to data
    /// not updated in the transaction.
    pub(crate) fn new(remote: &'r R, loader: &'l Loader) -> Self {
        TransactionDataCache {
            remote,
            loader,
            account_map: BTreeMap::new(),
            event_data: vec![],
        }
    }

    /// Make a write set from the updated (dirty, deleted) global resources along with
    /// published modules.
    ///
    /// Gives all proper guarantees on lifetime of global data as well.
    pub(crate) fn into_effects(self) -> PartialVMResult<TransactionEffects> {
        let mut modules = vec![];
        let mut resources = vec![];
        for (addr, account_cache) in self.account_map {
            let mut vals = vec![];
            for (ty, gv_opt) in account_cache.data_map {
                match gv_opt {
                    None => {
                        let ty_tag = self.loader.type_to_type_tag(&ty)?;
                        vals.push((ty_tag, None));
                    }
                    Some(gv) => {
                        if gv.is_dirty()? {
                            let ty_tag = self.loader.type_to_type_tag(&ty)?;
                            let ty_layout = self.loader.type_to_type_layout(&ty)?;
                            let val = Value::struct_(gv.into_owned_struct()?);
                            vals.push((ty_tag, Some((ty_layout, val))));
                        }
                    }
                };
            }
            if !vals.is_empty() {
                resources.push((addr, vals));
            }
            modules.extend(
                account_cache
                    .module_map
                    .into_iter()
                    .map(|(module_id, blob)| (module_id, blob)),
            );
        }

        let mut events = vec![];
        for (guid, seq_num, ty, val) in self.event_data {
            let ty_tag = self.loader.type_to_type_tag(&ty)?;
            let ty_layout = self.loader.type_to_type_layout(&ty)?;
            events.push((guid, seq_num, ty_tag, ty_layout, val))
        }

        Ok(TransactionEffects {
            resources,
            modules,
            events,
        })
    }

    pub(crate) fn num_mutated_accounts(&self) -> u64 {
        self.account_map.keys().len() as u64
    }

    fn get_mut_or_insert_with<'a, K, V, F>(map: &'a mut BTreeMap<K, V>, k: &K, gen: F) -> &'a mut V
    where
        F: FnOnce() -> (K, V),
        K: Ord,
    {
        if !map.contains_key(k) {
            let (k, v) = gen();
            map.insert(k, v);
        }
        map.get_mut(k).unwrap()
    }

    // Retrieve data from the local cache or loads it from the remote cache into the local cache.
    // All operations on the global data are based on this API and they all load the data
    // into the cache.
    fn load_data(
        &mut self,
        addr: AccountAddress,
        ty: &Type,
    ) -> PartialVMResult<&mut Option<GlobalValue>> {
        let account_cache = Self::get_mut_or_insert_with(&mut self.account_map, &addr, || {
            (addr, AccountDataCache::new())
        });

        if !account_cache.data_map.contains_key(ty) {
            let ty_tag = self.loader.type_to_type_tag(ty)?;

            let blob = self.remote.get_resource(&addr, &ty_tag)?.ok_or_else(|| {
                PartialVMError::new(StatusCode::MISSING_DATA)
                    .with_message(format!("Cannot find resource of type {}", ty_tag))
            })?;

            let ty_layout = self.loader.type_to_type_layout(ty)?;
            let ty_kind_info = self.loader.type_to_kind_info(ty)?;
            let val = Value::simple_deserialize(&blob, &ty_kind_info, &ty_layout)?;
            let gv = GlobalValue::new(val)?;

            account_cache.data_map.insert(ty.clone(), Some(gv));
        }

        Ok(account_cache.data_map.get_mut(ty).unwrap())
    }
}

// `DataStore` implementation for the `TransactionDataCache`
impl<'r, 'l, C: RemoteCache> DataStore for TransactionDataCache<'r, 'l, C> {
    fn publish_resource(
        &mut self,
        addr: AccountAddress,
        ty: Type,
        g: GlobalValue,
    ) -> PartialVMResult<()> {
        let account_cache = Self::get_mut_or_insert_with(&mut self.account_map, &addr, || {
            (addr, AccountDataCache::new())
        });

        account_cache.data_map.insert(ty, Some(g));

        Ok(())
    }

    fn borrow_resource(
        &mut self,
        addr: AccountAddress,
        ty: &Type,
    ) -> PartialVMResult<Option<&GlobalValue>> {
        Ok(self.load_data(addr, ty)?.as_ref())
    }

    fn move_resource_from(
        &mut self,
        addr: AccountAddress,
        ty: &Type,
    ) -> PartialVMResult<Option<GlobalValue>> {
        // .take() means that the entry is removed from the data map -- this marks the
        // access path for deletion.
        Ok(self.load_data(addr, ty)?.take())
    }

    fn load_module(&self, module_id: &ModuleId) -> VMResult<Vec<u8>> {
        if let Some(account_cache) = self.account_map.get(module_id.address()) {
            if let Some(blob) = account_cache.module_map.get(module_id) {
                return Ok(blob.clone());
            }
        }
        match self.remote.get_module(module_id)? {
            Some(bytes) => Ok(bytes),
            None => Err(PartialVMError::new(StatusCode::LINKER_ERROR)
                .with_message(format!("Cannot find {:?} in data cache", module_id))
                .finish(Location::Undefined)),
        }
    }

    fn publish_module(&mut self, module_id: &ModuleId, blob: Vec<u8>) -> VMResult<()> {
        let account_cache =
            Self::get_mut_or_insert_with(&mut self.account_map, module_id.address(), || {
                (*module_id.address(), AccountDataCache::new())
            });

        account_cache.module_map.insert(module_id.clone(), blob);

        Ok(())
    }

    fn exists_module(&self, module_id: &ModuleId) -> VMResult<bool> {
        if let Some(account_cache) = self.account_map.get(module_id.address()) {
            if account_cache.module_map.contains_key(module_id) {
                return Ok(true);
            }
        }
        Ok(self.remote.get_module(module_id)?.is_some())
    }

    fn emit_event(&mut self, guid: Vec<u8>, seq_num: u64, ty: Type, val: Value) {
        self.event_data.push((guid, seq_num, ty, val))
    }
}
