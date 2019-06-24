// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Defines the VM context for running instruction synthesis.
use types::access_path::AccessPath;
use vm::errors::VMInvariantViolation;
use vm_runtime::data_cache::RemoteCache;

/// A fake data cache used to build a transaction processor.
///
/// This is a simple fake data cache that doesn't cache anything. If we try `get`ing anything from
/// it then we return that we did not error, and that we did not find the data.
#[derive(Default)]
pub struct FakeDataCache;

impl FakeDataCache {
    /// Create a fake data cache.
    pub fn new() -> Self {
        FakeDataCache
    }
}

impl RemoteCache for FakeDataCache {
    fn get(&self, _access_path: &AccessPath) -> Result<Option<Vec<u8>>, VMInvariantViolation> {
        Ok(None)
    }
}

/// Create a VM loaded with the modules defined by the module generator passed in.
///
/// Returns back handles that can be used to reference the created VM, the root_module, and the
/// module cache of all loaded modules in the VM.
#[macro_export]
macro_rules! with_loaded_vm {
    ($module_generator:expr => $vm:ident, $mod:ident, $module_cache:ident) => {
        let mut modules = STDLIB_MODULES.clone();
        let mut generated_modules = $module_generator.collect();
        modules.append(&mut generated_modules);
        // The last module is the root module based upon how we generate modules.
        let root_module = modules
            .last()
            .expect("[VM Setup] Unable to get root module");
        let allocator = Arena::new();
        let module_id = root_module.self_id();
        let $module_cache = VMModuleCache::new(&allocator);
        let entry_idx = FunctionDefinitionIndex::new(0);
        let data_cache = FakeDataCache::new();
        $module_cache.cache_module(root_module.clone());
        let $mod = $module_cache
            .get_loaded_module(&module_id)
            .expect("[Module Cache] Internal error encountered when fetching module.")
            .expect("[Module Cache] Unable to find module in module cache.");
        for m in modules {
            $module_cache.cache_module(m);
        }
        let entry_func = FunctionRef::new(&$mod, entry_idx)
            .expect("[Entry Function] Unable to build function reference for entry function.");
        let mut $vm =
            TransactionExecutor::new(&$module_cache, &data_cache, TransactionMetadata::default());
        $vm.execution_stack.push_frame(entry_func);
    };
}
