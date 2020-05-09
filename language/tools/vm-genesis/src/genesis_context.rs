// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use anyhow::Result;
use bytecode_verifier::VerifiedModule;
use libra_state_view::StateView;
use libra_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_config,
    contract_event::ContractEvent,
    transaction::{Script, TransactionArgument},
    write_set::WriteSet,
};
use libra_vm::data_cache::StateViewCache;
use move_core_types::{
    gas_schedule::{CostTable, GasAlgebra, GasUnits},
    identifier::Identifier,
    language_storage::{ModuleId, TypeTag},
};
use move_vm_runtime::{
    data_cache::{RemoteCache, TransactionDataCache},
    move_vm::MoveVM,
};
use move_vm_types::{
    data_store::DataStore,
    gas_schedule::{zero_cost_schedule, CostStrategy},
    loaded_data::types::FatStructType,
    transaction_metadata::TransactionMetadata,
    values::{GlobalValue, Value},
};
use std::collections::{btree_map::BTreeMap, HashMap};
use vm::{access::ModuleAccess, errors::VMResult};

/// A context that holds state for generating the genesis write set
pub(crate) struct GenesisContext<'a> {
    vm: MoveVM,
    gas_schedule: CostTable,
    interpreter_context: GenesisDataCache<'a>,
    txn_data: TransactionMetadata,
}

impl<'a> GenesisContext<'a> {
    pub fn new(data_cache: &'a StateViewCache<'a>, stdlib_modules: &[VerifiedModule]) -> Self {
        let vm = MoveVM::new();
        let mut interpreter_context = GenesisDataCache::new(data_cache);
        for module in stdlib_modules {
            vm.cache_module(module.clone(), &mut interpreter_context)
                .unwrap_or_else(|_| {
                    panic!("Failure loading stdlib module {}", module.as_inner().name())
                });
        }

        Self {
            vm,
            gas_schedule: zero_cost_schedule(),
            interpreter_context,
            txn_data: TransactionMetadata::default(),
        }
    }

    fn module(name: &str) -> ModuleId {
        ModuleId::new(
            account_config::CORE_CODE_ADDRESS,
            Identifier::new(name).unwrap(),
        )
    }

    fn name(name: &str) -> Identifier {
        Identifier::new(name).unwrap()
    }

    /// Convert the transaction arguments into move values.
    fn convert_txn_args(args: &[TransactionArgument]) -> Vec<Value> {
        args.iter()
            .map(|arg| match arg {
                TransactionArgument::U8(i) => Value::u8(*i),
                TransactionArgument::U64(i) => Value::u64(*i),
                TransactionArgument::U128(i) => Value::u128(*i),
                TransactionArgument::Address(a) => Value::address(*a),
                TransactionArgument::Bool(b) => Value::bool(*b),
                TransactionArgument::U8Vector(v) => Value::vector_u8(v.clone()),
            })
            .collect()
    }

    pub fn exec(
        &mut self,
        module_name: &str,
        function_name: &str,
        type_params: Vec<TypeTag>,
        args: Vec<Value>,
    ) {
        let mut cost_strategy =
            CostStrategy::system(&self.gas_schedule, GasUnits::new(100_000_000));
        self.vm
            .execute_function(
                &Self::module(module_name),
                &Self::name(function_name),
                type_params,
                args,
                &mut cost_strategy,
                &mut self.interpreter_context,
                &self.txn_data,
            )
            .unwrap_or_else(|e| panic!("Error calling {}.{}: {}", module_name, function_name, e))
    }

    pub fn exec_script(&mut self, script: &Script) {
        let mut cost_strategy =
            CostStrategy::system(&self.gas_schedule, GasUnits::new(100_000_000));
        self.vm
            .execute_script(
                script.code().to_vec(),
                script.ty_args().to_vec(),
                Self::convert_txn_args(script.args()),
                &mut cost_strategy,
                &mut self.interpreter_context,
                &self.txn_data,
            )
            .unwrap()
    }

    pub fn set_sender(&mut self, sender: AccountAddress) {
        self.txn_data.sender = sender;
    }

    pub fn into_interpreter_context(self) -> GenesisDataCache<'a> {
        self.interpreter_context
    }
}

// `StateView` has no data given we are creating the genesis
pub(crate) struct GenesisStateView {
    data: HashMap<AccessPath, Vec<u8>>,
}

impl GenesisStateView {
    pub(crate) fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub(crate) fn add_module(&mut self, module_id: &ModuleId, module: &VerifiedModule) {
        let access_path = AccessPath::from(module_id);
        let mut blob = vec![];
        module
            .serialize(&mut blob)
            .expect("serializing stdlib must work");
        self.data.insert(access_path, blob);
    }
}

impl StateView for GenesisStateView {
    fn get(&self, access_path: &AccessPath) -> Result<Option<Vec<u8>>> {
        Ok(self.data.get(access_path).cloned())
    }

    fn multi_get(&self, _access_paths: &[AccessPath]) -> Result<Vec<Option<Vec<u8>>>> {
        unimplemented!()
    }

    fn is_genesis(&self) -> bool {
        true
    }
}

pub struct GenesisDataCache<'txn> {
    data_store: TransactionDataCache<'txn>,
    type_map: BTreeMap<Vec<u8>, FatStructType>,
}

impl<'txn> GenesisDataCache<'txn> {
    pub fn new(cache: &'txn dyn RemoteCache) -> Self {
        Self {
            data_store: TransactionDataCache::new(cache),
            type_map: BTreeMap::new(),
        }
    }

    pub fn get_type_map(&self) -> BTreeMap<Vec<u8>, FatStructType> {
        self.type_map.clone()
    }

    pub fn events(&self) -> &[ContractEvent] {
        self.data_store.event_data()
    }

    pub fn make_write_set(&mut self) -> VMResult<WriteSet> {
        self.data_store.make_write_set()
    }
}

impl<'txn> DataStore for GenesisDataCache<'txn> {
    fn publish_resource(
        &mut self,
        ap: &AccessPath,
        g: (FatStructType, GlobalValue),
    ) -> VMResult<()> {
        self.type_map.insert(ap.path.clone(), g.0.clone());
        self.data_store.publish_resource(ap, g)
    }

    fn borrow_resource(
        &mut self,
        ap: &AccessPath,
        ty: &FatStructType,
    ) -> VMResult<Option<&GlobalValue>> {
        self.data_store.borrow_resource(ap, ty)
    }

    fn move_resource_from(
        &mut self,
        ap: &AccessPath,
        ty: &FatStructType,
    ) -> VMResult<Option<GlobalValue>> {
        self.data_store.move_resource_from(ap, ty)
    }

    fn load_module(&self, module: &ModuleId) -> VMResult<Vec<u8>> {
        self.data_store.load_module(module)
    }

    fn exists_module(&self, key: &ModuleId) -> bool {
        self.data_store.exists_module(key)
    }

    fn publish_module(&mut self, module_id: ModuleId, module: Vec<u8>) -> VMResult<()> {
        self.data_store.publish_module(module_id, module)
    }

    fn emit_event(&mut self, event: ContractEvent) {
        self.data_store.emit_event(event)
    }
}
