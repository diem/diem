use anyhow::{format_err, Result};
use move_core_types::{
    account_address::AccountAddress,
    effects::{ChangeSet, Event},
    language_storage::{ModuleId, StructTag},
};
use move_vm_runtime::{data_cache::RemoteCache, move_vm::MoveVM, session::Session};
use move_vm_txn_effect_converter::convert_txn_effects_to_move_changeset_and_events;
use vm::errors::{PartialVMResult, VMResult};

/// A dummy storage containing no modules or resources.
pub struct BlankStorage;

impl BlankStorage {
    pub fn new() -> Self {
        Self
    }
}

impl RemoteCache for BlankStorage {
    fn get_module(&self, _module_id: &ModuleId) -> VMResult<Option<Vec<u8>>> {
        Ok(None)
    }

    fn get_resource(
        &self,
        _address: &AccountAddress,
        _tag: &StructTag,
    ) -> PartialVMResult<Option<Vec<u8>>> {
        Ok(None)
    }
}

/// A storage adapter created by stacking a change set on top of an existing storage backend.
/// The new storage can be used for additional computations without modifying the base.
pub struct DeltaStorage<'a, S> {
    base: &'a S,
    delta: ChangeSet,
}

impl<'a, S: RemoteCache> RemoteCache for DeltaStorage<'a, S> {
    fn get_module(&self, module_id: &ModuleId) -> VMResult<Option<Vec<u8>>> {
        if let Some(account_storage) = self.delta.accounts.get(module_id.address()) {
            if let Some(blob_opt) = account_storage.modules.get(module_id.name()) {
                return Ok(blob_opt.clone());
            }
        }

        self.base.get_module(module_id)
    }

    fn get_resource(
        &self,
        address: &AccountAddress,
        tag: &StructTag,
    ) -> PartialVMResult<Option<Vec<u8>>> {
        if let Some(account_storage) = self.delta.accounts.get(address) {
            if let Some(blob_opt) = account_storage.resources.get(tag) {
                return Ok(blob_opt.clone());
            }
        }

        self.base.get_resource(address, tag)
    }
}

impl<'a, S: RemoteCache> DeltaStorage<'a, S> {
    pub fn new(base: &'a S, delta: ChangeSet) -> Self {
        Self { base, delta }
    }
}

pub fn build_genesis_simple(
    genesis_modules: impl IntoIterator<Item = (ModuleId, Vec<u8>)>,
    run_initialization_session: impl FnOnce(&mut Session<DeltaStorage<BlankStorage>>) -> Result<()>,
) -> Result<(ChangeSet, Vec<Event>)> {
    let move_vm = MoveVM::new();

    let mut delta = ChangeSet::new();
    for (module_id, blob) in genesis_modules.into_iter() {
        delta.publish_module(module_id, blob)?;
    }

    let storage = DeltaStorage::new(&BlankStorage, delta);
    let mut session = move_vm.new_session(&storage);
    run_initialization_session(&mut session)?;

    let txn_effects = session
        .finish()
        .map_err(|err| format_err!("failed to conclude session: {:?}", err))?;

    Ok(convert_txn_effects_to_move_changeset_and_events(
        txn_effects,
    )?)
}
