// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Defines the VM context for running instruction synthesis.

/// Create a VM loaded with the modules defined by the module generator passed in.
///
/// Returns back handles that can be used to reference the created VM, the root_module, and the
/// module cache of all loaded modules in the VM.
#[macro_export]
macro_rules! with_loaded_vm {
    ($num_modules:expr, $table_size:expr, $root_account:expr => $vm:ident, $mod:ident, $module_cache:ident) => {
        use vm::access::ModuleAccess;

        let mut modules = ::stdlib::stdlib_modules().to_vec();
        let (root, mut callee_modules) = generate_padded_modules($num_modules, $table_size);
        modules.append(&mut callee_modules);
        modules.push(root);
        // The last module is the root module based upon how we generate modules.
        let root_module = modules
            .last()
            .expect("[VM Setup] Unable to get root module");
        let allocator = Arena::new();
        let module_id = root_module.self_id();
        let $module_cache = VMModuleCache::new(&allocator);
        let entry_idx = FunctionDefinitionIndex::new(0);
        let mut data_cache = FakeDataStore::default();
        $module_cache.cache_module(root_module.clone());
        let $mod = $module_cache
            .get_loaded_module(&module_id)
            .expect("[Module Lookup] Runtime error while looking up module");
        for m in modules.clone() {
            $module_cache.cache_module(m);
        }
        let entry_func = FunctionRef::new(&$mod, entry_idx);
        // Create the inhabitor to build the resources that have been published
        let mut inhabitor = RandomInhabitor::new(&$mod, &$module_cache);
        $root_account.modules = modules;
        for (access_path, blob) in $root_account.generate_resources(&mut inhabitor).into_iter() {
            data_cache.set(access_path, blob);
        }
        let gas_schedule = CostTable::zero();
        let txn_data = TransactionMetadata::default();
        let mut $vm =
            InterpreterForCostSynthesis::new(&$module_cache, &txn_data, &data_cache, &gas_schedule);
        $vm.push_frame(entry_func, vec![]);
    };
}
