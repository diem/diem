// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{data_cache::RemoteCache, move_vm::MoveVM};
use move_core_types::{
    account_address::AccountAddress,
    gas_schedule::{GasAlgebra, GasUnits},
    identifier::{IdentStr, Identifier},
    language_storage::{ModuleId, StructTag},
    vm_status::{StatusCode, VMStatus},
};
use move_lang::{compiled_unit::CompiledUnit, shared::Address};
use move_vm_types::gas_schedule::{zero_cost_schedule, CostStrategy};
use std::{collections::HashMap, path::PathBuf};
use vm::{
    errors::{PartialVMResult, VMResult},
    file_format::{AddressIdentifierIndex, IdentifierIndex, ModuleHandle, TableIndex},
    CompiledModule,
};

const WORKING_ACCOUNT: AccountAddress =
    AccountAddress::new([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2]);

struct Adapter {
    store: DataStore,
    vm: MoveVM,
}

impl Adapter {
    fn new(store: DataStore) -> Self {
        Self {
            store,
            vm: MoveVM::new(),
        }
    }

    fn publish_modules(&mut self, modules: Vec<CompiledModule>, target: &str, dest: &str) {
        let mut session = self.vm.new_session(&self.store);
        let cost_table = zero_cost_schedule();
        let mut cost_strategy = CostStrategy::system(&cost_table, GasUnits::new(0));
        for module in modules {
            let mut binary = vec![];
            module
                .serialize(&mut binary)
                .unwrap_or_else(|_| panic!("failure in module serialization: {:#?}", module));
            session
                .publish_module(binary, WORKING_ACCOUNT, &mut cost_strategy)
                .unwrap_or_else(|err| {
                    panic!("failure ({:?}) publishing module: {:#?}", err, module)
                });
        }
        let data = session.finish().expect("failure getting write set");
        for (module_id, module) in data.modules {
            if module_id.name().as_str() == target {
                let m = CompiledModule::deserialize(&module).expect("serilaize");
                let mut mm = m.into_inner();
                mm.identifiers
                    .push(Identifier::new(dest).expect("identifier"));
                let mh = ModuleHandle {
                    address: AddressIdentifierIndex(
                        (mm.address_identifiers.len() - 1) as TableIndex,
                    ),
                    name: IdentifierIndex((mm.identifiers.len() - 1) as TableIndex),
                };
                mm.module_handles.push(mh);
                let mut blob = vec![];
                mm.serialize(&mut blob).expect("module must serialize");
                self.store.add_module(module_id, blob);
            } else {
                self.store.add_module(module_id, module);
            }
        }
    }

    fn call_function(&self, module: &ModuleId, name: &IdentStr) -> Result<(), VMStatus> {
        let cost_table = zero_cost_schedule();
        let mut cost_strategy = CostStrategy::system(&cost_table, GasUnits::new(0));
        let mut session = self.vm.new_session(&self.store);
        session.execute_function(
            module,
            name,
            vec![],
            vec![],
            WORKING_ACCOUNT,
            &mut cost_strategy,
            |e| e,
        )
    }
}

#[derive(Clone, Debug)]
struct DataStore {
    modules: HashMap<ModuleId, Vec<u8>>,
}

impl DataStore {
    fn empty() -> Self {
        Self {
            modules: HashMap::new(),
        }
    }

    fn add_module(&mut self, module_id: ModuleId, binary: Vec<u8>) {
        self.modules.insert(module_id, binary);
    }
}

impl RemoteCache for DataStore {
    fn get_module(&self, module_id: &ModuleId) -> VMResult<Option<Vec<u8>>> {
        match self.modules.get(module_id) {
            None => Ok(None),
            Some(binary) => Ok(Some(binary.clone())),
        }
    }

    fn get_resource(
        &self,
        _address: &AccountAddress,
        _tag: &StructTag,
    ) -> PartialVMResult<Option<Vec<u8>>> {
        Ok(None)
    }
}

fn compile_file(addr: &[u8; AccountAddress::LENGTH]) -> Vec<CompiledModule> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("src/unit_tests/modules_circular.move");
    let s = path.to_str().expect("no path specified").to_owned();
    let (_, modules) =
        move_lang::move_compile(&[s], &[], Some(Address::new(*addr))).expect("Error compiling...");

    let mut compiled_modules = vec![];
    for module in modules {
        match module {
            CompiledUnit::Module { module, .. } => compiled_modules.push(module),
            CompiledUnit::Script { .. } => (),
        }
    }
    compiled_modules
}

#[test]
fn all_good() {
    let data_store = DataStore::empty();
    let mut adapter = Adapter::new(data_store);
    let modules = compile_file(&[0; 16]);
    adapter.publish_modules(modules, "", "");
    adapter
        .call_function(
            &ModuleId::new(WORKING_ACCOUNT, Identifier::new("A").unwrap()),
            Identifier::new("entry_a").unwrap().as_ident_str(),
        )
        .expect("A::entry_a() must work");
    adapter
        .call_function(
            &ModuleId::new(WORKING_ACCOUNT, Identifier::new("B").unwrap()),
            Identifier::new("entry_b").unwrap().as_ident_str(),
        )
        .expect("B::entry_b() must work");
    adapter
        .call_function(
            &ModuleId::new(WORKING_ACCOUNT, Identifier::new("C").unwrap()),
            Identifier::new("entry_c").unwrap().as_ident_str(),
        )
        .expect("C::entry_c() must work");
    adapter
        .call_function(
            &ModuleId::new(WORKING_ACCOUNT, Identifier::new("D").unwrap()),
            Identifier::new("entry_d").unwrap().as_ident_str(),
        )
        .expect("D::entry_d() must work");
    adapter
        .call_function(
            &ModuleId::new(WORKING_ACCOUNT, Identifier::new("E").unwrap()),
            Identifier::new("entry_e").unwrap().as_ident_str(),
        )
        .expect("E::entry_e() must work");
    adapter
        .call_function(
            &ModuleId::new(WORKING_ACCOUNT, Identifier::new("F").unwrap()),
            Identifier::new("entry_f").unwrap().as_ident_str(),
        )
        .expect("F::entry_f() must work");
}

#[test]
fn circular_c() {
    let data_store = DataStore::empty();
    let mut adapter = Adapter::new(data_store);
    let modules = compile_file(&[0; 16]);
    adapter.publish_modules(modules, "C", "A");
    assert_eq!(
        adapter
            .call_function(
                &ModuleId::new(WORKING_ACCOUNT, Identifier::new("A").unwrap()),
                Identifier::new("entry_a").unwrap().as_ident_str(),
            )
            .unwrap_err(),
        VMStatus::Error(StatusCode::UNEXPECTED_VERIFIER_ERROR),
    );
    assert_eq!(
        adapter
            .call_function(
                &ModuleId::new(WORKING_ACCOUNT, Identifier::new("B").unwrap()),
                Identifier::new("entry_b").unwrap().as_ident_str(),
            )
            .unwrap_err(),
        VMStatus::Error(StatusCode::UNEXPECTED_VERIFIER_ERROR),
    );
    assert_eq!(
        adapter
            .call_function(
                &ModuleId::new(WORKING_ACCOUNT, Identifier::new("C").unwrap()),
                Identifier::new("entry_c").unwrap().as_ident_str(),
            )
            .unwrap_err(),
        VMStatus::Error(StatusCode::UNEXPECTED_VERIFIER_ERROR),
    );
    assert_eq!(
        adapter
            .call_function(
                &ModuleId::new(WORKING_ACCOUNT, Identifier::new("D").unwrap()),
                Identifier::new("entry_d").unwrap().as_ident_str(),
            )
            .unwrap_err(),
        VMStatus::Error(StatusCode::UNEXPECTED_VERIFIER_ERROR),
    );
    assert_eq!(
        adapter
            .call_function(
                &ModuleId::new(WORKING_ACCOUNT, Identifier::new("E").unwrap()),
                Identifier::new("entry_e").unwrap().as_ident_str(),
            )
            .unwrap_err(),
        VMStatus::Error(StatusCode::UNEXPECTED_VERIFIER_ERROR),
    );
    assert_eq!(
        adapter
            .call_function(
                &ModuleId::new(WORKING_ACCOUNT, Identifier::new("F").unwrap()),
                Identifier::new("entry_f").unwrap().as_ident_str(),
            )
            .unwrap_err(),
        VMStatus::Error(StatusCode::UNEXPECTED_VERIFIER_ERROR),
    );
}

#[test]
fn circular_b() {
    let data_store = DataStore::empty();
    let mut adapter = Adapter::new(data_store);
    let modules = compile_file(&[0; 16]);
    adapter.publish_modules(modules, "B", "F");
    assert_eq!(
        adapter
            .call_function(
                &ModuleId::new(WORKING_ACCOUNT, Identifier::new("A").unwrap()),
                Identifier::new("entry_a").unwrap().as_ident_str(),
            )
            .unwrap_err(),
        VMStatus::Error(StatusCode::UNEXPECTED_VERIFIER_ERROR),
    );
    assert_eq!(
        adapter
            .call_function(
                &ModuleId::new(WORKING_ACCOUNT, Identifier::new("B").unwrap()),
                Identifier::new("entry_b").unwrap().as_ident_str(),
            )
            .unwrap_err(),
        VMStatus::Error(StatusCode::UNEXPECTED_VERIFIER_ERROR),
    );
    adapter
        .call_function(
            &ModuleId::new(WORKING_ACCOUNT, Identifier::new("C").unwrap()),
            Identifier::new("entry_c").unwrap().as_ident_str(),
        )
        .expect("C::entry_c() must work");
    adapter
        .call_function(
            &ModuleId::new(WORKING_ACCOUNT, Identifier::new("D").unwrap()),
            Identifier::new("entry_d").unwrap().as_ident_str(),
        )
        .expect("D::entry_d() must work");
    assert_eq!(
        adapter
            .call_function(
                &ModuleId::new(WORKING_ACCOUNT, Identifier::new("E").unwrap()),
                Identifier::new("entry_e").unwrap().as_ident_str(),
            )
            .unwrap_err(),
        VMStatus::Error(StatusCode::UNEXPECTED_VERIFIER_ERROR),
    );
    assert_eq!(
        adapter
            .call_function(
                &ModuleId::new(WORKING_ACCOUNT, Identifier::new("F").unwrap()),
                Identifier::new("entry_f").unwrap().as_ident_str(),
            )
            .unwrap_err(),
        VMStatus::Error(StatusCode::UNEXPECTED_VERIFIER_ERROR),
    );
}

#[test]
fn circular_f() {
    let data_store = DataStore::empty();
    let mut adapter = Adapter::new(data_store);
    let modules = compile_file(&[0; 16]);
    adapter.publish_modules(modules, "F", "E");
    adapter
        .call_function(
            &ModuleId::new(WORKING_ACCOUNT, Identifier::new("A").unwrap()),
            Identifier::new("entry_a").unwrap().as_ident_str(),
        )
        .expect("A::entry_a() must work");
    adapter
        .call_function(
            &ModuleId::new(WORKING_ACCOUNT, Identifier::new("B").unwrap()),
            Identifier::new("entry_b").unwrap().as_ident_str(),
        )
        .expect("B::entry_b() must work");
    adapter
        .call_function(
            &ModuleId::new(WORKING_ACCOUNT, Identifier::new("C").unwrap()),
            Identifier::new("entry_c").unwrap().as_ident_str(),
        )
        .expect("C::entry_c() must work");
    adapter
        .call_function(
            &ModuleId::new(WORKING_ACCOUNT, Identifier::new("D").unwrap()),
            Identifier::new("entry_d").unwrap().as_ident_str(),
        )
        .expect("D::entry_d() must work");
    assert_eq!(
        adapter
            .call_function(
                &ModuleId::new(WORKING_ACCOUNT, Identifier::new("E").unwrap()),
                Identifier::new("entry_e").unwrap().as_ident_str(),
            )
            .unwrap_err(),
        VMStatus::Error(StatusCode::UNEXPECTED_VERIFIER_ERROR),
    );
    assert_eq!(
        adapter
            .call_function(
                &ModuleId::new(WORKING_ACCOUNT, Identifier::new("F").unwrap()),
                Identifier::new("entry_f").unwrap().as_ident_str(),
            )
            .unwrap_err(),
        VMStatus::Error(StatusCode::UNEXPECTED_VERIFIER_ERROR),
    );
}
