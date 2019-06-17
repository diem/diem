use crate::{
    code_cache::module_cache::ModuleCache, data_cache::RemoteCache,
    loaded_data::loaded_module::LoadedModule,
};
use config::config::VMPublishingOption;
use std::marker::PhantomData;
use types::transaction::SignedTransaction;
use vm_cache_map::Arena;

pub mod execute;
pub mod validate;
pub mod verify;

use types::vm_error::VMStatus;
use validate::{ValidatedTransaction, ValidationMode};

/// The starting point for processing a transaction. All the different states involved are described
/// through the types present in submodules.
pub struct ProcessTransaction<'alloc, 'txn, P>
where
    'alloc: 'txn,
    P: ModuleCache<'alloc>,
{
    txn: SignedTransaction,
    module_cache: P,
    data_cache: &'txn RemoteCache,
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
        txn: SignedTransaction,
        module_cache: P,
        data_cache: &'txn RemoteCache,
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
    ) -> Result<ValidatedTransaction<'alloc, 'txn, P>, VMStatus> {
        ValidatedTransaction::new(self, mode, publishing_option)
    }
}
