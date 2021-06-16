// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::compiler::compile_modules_in_file;
use move_binary_format::CompiledModule;
use move_core_types::{
    account_address::AccountAddress,
    identifier::{IdentStr, Identifier},
    language_storage::ModuleId,
};
use move_vm_runtime::move_vm::MoveVM;
use move_vm_test_utils::InMemoryStorage;
use move_vm_types::gas_schedule::GasStatus;
use std::{path::PathBuf, sync::Arc, thread};

const WORKING_ACCOUNT: AccountAddress =
    AccountAddress::new([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2]);

struct Adapter {
    store: InMemoryStorage,
    vm: Arc<MoveVM>,
    functions: Vec<(ModuleId, Identifier)>,
}

impl Adapter {
    fn new(store: InMemoryStorage) -> Self {
        let functions = vec![
            (
                ModuleId::new(WORKING_ACCOUNT, Identifier::new("A").unwrap()),
                Identifier::new("entry_a").unwrap(),
            ),
            (
                ModuleId::new(WORKING_ACCOUNT, Identifier::new("D").unwrap()),
                Identifier::new("entry_d").unwrap(),
            ),
            (
                ModuleId::new(WORKING_ACCOUNT, Identifier::new("E").unwrap()),
                Identifier::new("entry_e").unwrap(),
            ),
            (
                ModuleId::new(WORKING_ACCOUNT, Identifier::new("F").unwrap()),
                Identifier::new("entry_f").unwrap(),
            ),
            (
                ModuleId::new(WORKING_ACCOUNT, Identifier::new("C").unwrap()),
                Identifier::new("just_c").unwrap(),
            ),
        ];
        Self {
            store,
            vm: Arc::new(MoveVM::new(vec![]).unwrap()),
            functions,
        }
    }

    fn publish_modules(&mut self, modules: Vec<CompiledModule>) {
        let mut session = self.vm.new_session(&self.store);
        let mut gas_status = GasStatus::new_unmetered();
        for module in modules {
            let mut binary = vec![];
            module
                .serialize(&mut binary)
                .unwrap_or_else(|_| panic!("failure in module serialization: {:#?}", module));
            session
                .publish_module(binary, WORKING_ACCOUNT, &mut gas_status)
                .unwrap_or_else(|_| panic!("failure publishing module: {:#?}", module));
        }
        let (changeset, _) = session.finish().expect("failure getting write set");
        self.store
            .apply(changeset)
            .expect("failure applying write set");
    }

    fn call_functions(&self) {
        for (module_id, name) in &self.functions {
            self.call_function(module_id, name);
        }
    }

    fn call_functions_async(&self, reps: usize) {
        let mut children = vec![];
        for _ in 0..reps {
            for (module_id, name) in self.functions.clone() {
                let vm = self.vm.clone();
                let data_store = self.store.clone();
                children.push(thread::spawn(move || {
                    let mut gas_status = GasStatus::new_unmetered();
                    let mut session = vm.new_session(&data_store);
                    session
                        .execute_function(&module_id, &name, vec![], vec![], &mut gas_status)
                        .unwrap_or_else(|_| {
                            panic!("Failure executing {:?}::{:?}", module_id, name)
                        });
                }));
            }
        }
        for child in children {
            let _ = child.join();
        }
    }

    fn call_function(&self, module: &ModuleId, name: &IdentStr) {
        let mut gas_status = GasStatus::new_unmetered();
        let mut session = self.vm.new_session(&self.store);
        session
            .execute_function(module, name, vec![], vec![], &mut gas_status)
            .unwrap_or_else(|_| panic!("Failure executing {:?}::{:?}", module, name));
    }
}

fn get_modules() -> Vec<CompiledModule> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("src/tests/loader_tests_modules.move");
    compile_modules_in_file(&path).unwrap()
}

#[test]
fn load() {
    let data_store = InMemoryStorage::new();
    let mut adapter = Adapter::new(data_store);
    let modules = get_modules();
    adapter.publish_modules(modules);
    // calls all functions sequentially
    adapter.call_functions();
}

#[test]
fn load_concurrent() {
    let data_store = InMemoryStorage::new();
    let mut adapter = Adapter::new(data_store);
    let modules = get_modules();
    adapter.publish_modules(modules);
    // makes 15 threads
    adapter.call_functions_async(3);
}

#[test]
fn load_concurrent_many() {
    let data_store = InMemoryStorage::new();
    let mut adapter = Adapter::new(data_store);
    let modules = get_modules();
    adapter.publish_modules(modules);
    // makes 150 threads
    adapter.call_functions_async(30);
}
