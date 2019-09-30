// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    counters::*, loaded_data::loaded_module::LoadedModule, runtime::VMRuntime, VMExecutor,
    VMVerifier,
};
use libra_state_view::StateView;
use libra_types::{
    transaction::{SignedTransaction, TransactionOutput},
    vm_error::VMStatus,
};
use libra_vm_cache_map::Arena;
use std::sync::Arc;

rental! {
    mod move_vm_definition {
        use super::*;

        #[rental]
        pub struct MoveVMImpl {
            alloc: Box<Arena<LoadedModule>>,
            runtime: VMRuntime<'alloc>,
        }
    }
}

use libra_config::config::VMConfig;
pub use move_vm_definition::MoveVMImpl;

/// A wrapper to make VMRuntime standalone and thread safe.
#[derive(Clone)]
pub struct MoveVM {
    inner: Arc<MoveVMImpl>,
}

impl MoveVM {
    pub fn new(config: &VMConfig) -> Self {
        let inner = MoveVMImpl::new(Box::new(Arena::new()), |arena| {
            VMRuntime::new(&*arena, config)
        });
        Self {
            inner: Arc::new(inner),
        }
    }
}

impl VMVerifier for MoveVM {
    fn validate_transaction(
        &self,
        transaction: SignedTransaction,
        state_view: &dyn StateView,
    ) -> Option<VMStatus> {
        // TODO: This should be implemented as an async function.
        record_stats! {time_hist | TXN_VALIDATION_TIME_TAKEN | {
            self.inner
                .rent(move |runtime| runtime.verify_transaction(transaction, state_view))
            }
        }
    }
}

impl VMExecutor for MoveVM {
    fn execute_block(
        transactions: Vec<SignedTransaction>,
        config: &VMConfig,
        state_view: &dyn StateView,
    ) -> Vec<TransactionOutput> {
        let vm = MoveVMImpl::new(Box::new(Arena::new()), |arena| {
            // XXX This means that scripts and modules are NOT tested against the whitelist! This
            // needs to be fixed.
            VMRuntime::new(&*arena, config)
        });
        vm.rent(|runtime| runtime.execute_block_transactions(transactions, state_view))
    }
}

#[test]
fn vm_thread_safe() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    assert_send::<MoveVM>();
    assert_sync::<MoveVM>();
    assert_send::<MoveVMImpl>();
    assert_sync::<MoveVMImpl>();
}
