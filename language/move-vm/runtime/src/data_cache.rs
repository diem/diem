// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::loader::Loader;

use libra_logger::prelude::*;
use move_core_types::{
    account_address::AccountAddress,
    language_storage::{ModuleId, StructTag, TypeTag},
    value::MoveTypeLayout,
    vm_status::StatusCode,
};
use move_vm_types::{
    data_store::DataStore,
    loaded_data::runtime_types::Type,
    values::{GlobalValue, GlobalValueEffect, Value},
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
        tag: &StructTag,
    ) -> PartialVMResult<Option<Vec<u8>>>;
}

pub struct AccountDataCache {
    data_map: BTreeMap<Type, (MoveTypeLayout, GlobalValue)>,
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
    event_data: Vec<(Vec<u8>, u64, Type, MoveTypeLayout, Value)>,
}

pub struct TransactionEffects {
    pub resources: Vec<(
        AccountAddress,
        Vec<(StructTag, Option<(MoveTypeLayout, Value)>)>,
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
            for (ty, (ty_layout, gv)) in account_cache.data_map {
                match gv.into_effect()? {
                    GlobalValueEffect::None => (),
                    GlobalValueEffect::Deleted => {
                        if let TypeTag::Struct(s_tag) = self.loader.type_to_type_tag(&ty)? {
                            vals.push((s_tag, None))
                        } else {
                            // non-struct top-level value; can't happen
                            return Err(PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR));
                        }
                    }
                    GlobalValueEffect::Changed(val) => {
                        if let TypeTag::Struct(s_tag) = self.loader.type_to_type_tag(&ty)? {
                            vals.push((s_tag, Some((ty_layout, val))))
                        } else {
                            // non-struct top-level value; can't happen
                            return Err(PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR));
                        }
                    }
                }
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
        for (guid, seq_num, ty, ty_layout, val) in self.event_data {
            let ty_tag = self.loader.type_to_type_tag(&ty)?;
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
}

// `DataStore` implementation for the `TransactionDataCache`
impl<'r, 'l, C: RemoteCache> DataStore for TransactionDataCache<'r, 'l, C> {
    // Retrieve data from the local cache or loads it from the remote cache into the local cache.
    // All operations on the global data are based on this API and they all load the data
    // into the cache.
    fn load_resource(
        &mut self,
        addr: AccountAddress,
        ty: &Type,
    ) -> PartialVMResult<&mut GlobalValue> {
        let account_cache = Self::get_mut_or_insert_with(&mut self.account_map, &addr, || {
            (addr, AccountDataCache::new())
        });

        if !account_cache.data_map.contains_key(ty) {
            let ty_tag = match self.loader.type_to_type_tag(ty)? {
                TypeTag::Struct(s_tag) => s_tag,
                _ =>
                // non-struct top-level value; can't happen
                {
                    return Err(PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR))
                }
            };
            let ty_layout = self.loader.type_to_type_layout(ty)?;

            let gv = match self.remote.get_resource(&addr, &ty_tag)? {
                Some(blob) => {
                    let ty_kind_info = self.loader.type_to_kind_info(ty)?;
                    let val = match Value::simple_deserialize(&blob, &ty_kind_info, &ty_layout) {
                        Some(val) => val,
                        None => {
                            let msg =
                                format!("Failed to deserialize resource {} at {}!", ty_tag, addr);
                            error!(
                                "[vm] Failed to deserialize resource {} at {}!",
                                type_tag = ty_tag,
                                account = addr
                            );
                            return Err(PartialVMError::new(
                                StatusCode::FAILED_TO_DESERIALIZE_RESOURCE,
                            )
                            .with_message(msg));
                        }
                    };

                    GlobalValue::cached(val)?
                }
                None => GlobalValue::none(),
            };

            account_cache.data_map.insert(ty.clone(), (ty_layout, gv));
        }

        Ok(account_cache
            .data_map
            .get_mut(ty)
            .map(|(_ty_layout, gv)| gv)
            .expect("global value must exist"))
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

    fn emit_event(
        &mut self,
        guid: Vec<u8>,
        seq_num: u64,
        ty: Type,
        val: Value,
    ) -> PartialVMResult<()> {
        let ty_layout = self.loader.type_to_type_layout(&ty)?;
        Ok(self.event_data.push((guid, seq_num, ty, ty_layout, val)))
    }
}
