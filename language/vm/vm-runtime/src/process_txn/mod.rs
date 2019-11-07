// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    code_cache::module_cache::ModuleCache, data_cache::RemoteCache,
};
use libra_config::config::VMPublishingOption;
use libra_types::transaction::SignatureCheckedTransaction;
use std::marker::PhantomData;
use vm::{errors::VMResult, gas_schedule::CostTable};

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
    gas_schedule: &'txn CostTable,
    module_cache: P,
    data_cache: &'txn dyn RemoteCache,
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
        gas_schedule: &'txn CostTable,
        module_cache: P,
        data_cache: &'txn dyn RemoteCache,
    ) -> Self {
        Self {
            txn,
            gas_schedule,
            module_cache,
            data_cache,
            phantom: PhantomData,
        }
    }

    /// Validates this transaction. Returns a `ValidatedTransaction` on success or `VMStatus` on
    /// failure.
    pub fn validate(
        self,
        mode: ValidationMode,
        publishing_option: &VMPublishingOption,
    ) -> VMResult<ValidatedTransaction<'alloc, 'txn, P>> {
        ValidatedTransaction::new(self, mode, publishing_option)
    }
}
