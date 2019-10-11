use crate::{
    code_cache::module_cache::ModuleCache, data_cache::RemoteCache,
    loaded_data::loaded_module::LoadedModule,
};
use config::config::{VMPublishingOption, VMMode};
use libra_types::transaction::SignatureCheckedTransaction;
use std::marker::PhantomData;
use vm::errors::VMResult;
use vm_cache_map::Arena;

pub mod balance_checker;
pub mod execute;
pub mod validate;
pub mod verify;

use validate::{ValidatedTransaction, ValidationMode};

/// The starting point for processing a transaction. All the different states involved are described
/// through the types present in submodules.
pub struct ProcessTransaction<'alloc, 'txn, P>
where
    'alloc: 'txn,
    P: ModuleCache<'alloc>,
{
    txn: SignatureCheckedTransaction,
    module_cache: P,
    data_cache: &'txn dyn RemoteCache,
    allocator: &'txn Arena<LoadedModule>,
    phantom: PhantomData<&'alloc ()>,
}

impl<'alloc, 'txn, P> ProcessTransaction<'alloc, 'txn, P>
where
    'alloc: 'txn,
    P: ModuleCache<'alloc>,
{
    /// Creates a new instance of `ProcessTransaction`.
    pub fn new(
        txn: SignatureCheckedTransaction,
        module_cache: P,
        data_cache: &'txn dyn RemoteCache,
        allocator: &'txn Arena<LoadedModule>,
    ) -> Self {
        Self {
            txn,
            module_cache,
            data_cache,
            allocator,
            phantom: PhantomData,
        }
    }

    /// Validates this transaction. Returns a `ValidatedTransaction` on success or `VMStatus` on
    /// failure.
    pub fn validate(
        self,
        mode: ValidationMode,
        publishing_option: &VMPublishingOption,
        vm_mode: VMMode,
    ) -> VMResult<ValidatedTransaction<'alloc, 'txn, P>> {
        ValidatedTransaction::new(self, mode, publishing_option, vm_mode)
    }
}
