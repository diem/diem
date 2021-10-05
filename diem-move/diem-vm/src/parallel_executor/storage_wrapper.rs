// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::data_cache::RemoteStorage;
use diem_parallel_executor::executor::MVHashMapView;
use diem_state_view::{StateView, StateViewId};
use diem_types::{access_path::AccessPath, account_state::AccountState, write_set::WriteOp};
use move_binary_format::errors::VMError;
use move_core_types::{
    account_address::AccountAddress,
    language_storage::{ModuleId, StructTag},
    resolver::{ModuleResolver, ResourceResolver},
};

pub(crate) struct VersionedView<'a, S: StateView> {
    base_view: &'a S,
    hashmap_view: &'a MVHashMapView<'a, AccessPath, WriteOp>,
}

impl<'a, S: StateView> VersionedView<'a, S> {
    pub fn new_view(
        base_view: &'a S,
        hashmap_view: &'a MVHashMapView<'a, AccessPath, WriteOp>,
    ) -> VersionedView<'a, S> {
        VersionedView {
            base_view,
            hashmap_view,
        }
    }
}

impl<'a, S: StateView> StateView for VersionedView<'a, S> {
    fn id(&self) -> StateViewId {
        self.base_view.id()
    }

    // Get some data either through the cache or the `StateView` on a cache miss.
    fn get(&self, access_path: &AccessPath) -> anyhow::Result<Option<Vec<u8>>> {
        match self.hashmap_view.read(access_path) {
            Ok(Some(WriteOp::Value(v))) => Ok(Some(v.clone())),
            Ok(Some(WriteOp::Deletion)) => Ok(None),
            Ok(None) => self.base_view.get(access_path),
            Err(err) => Err(err),
        }
    }

    fn get_account_state(&self, _account: AccountAddress) -> anyhow::Result<Option<AccountState>> {
        todo!()
    }

    fn is_genesis(&self) -> bool {
        self.base_view.is_genesis()
    }
}

impl<'a, S: StateView> ModuleResolver for VersionedView<'a, S> {
    type Error = VMError;

    fn get_module(&self, module_id: &ModuleId) -> Result<Option<Vec<u8>>, Self::Error> {
        RemoteStorage::new(self).get_module(module_id)
    }
}

impl<'a, S: StateView> ResourceResolver for VersionedView<'a, S> {
    type Error = VMError;

    fn get_resource(
        &self,
        address: &AccountAddress,
        tag: &StructTag,
    ) -> Result<Option<Vec<u8>>, Self::Error> {
        RemoteStorage::new(self).get_resource(address, tag)
    }
}
