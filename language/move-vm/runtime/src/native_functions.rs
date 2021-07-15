// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{interpreter::Interpreter, loader::Resolver};
use move_binary_format::errors::{PartialVMError, PartialVMResult};
use move_core_types::{
    account_address::AccountAddress,
    gas_schedule::CostTable,
    identifier::Identifier,
    value::MoveTypeLayout,
    vm_status::{StatusCode, StatusType},
};
use move_vm_types::{
    data_store::DataStore, gas_schedule::GasStatus, loaded_data::runtime_types::Type,
    natives::function::NativeResult, values::Value,
};
use std::{
    collections::{HashMap, VecDeque},
    fmt::Write,
};

pub type NativeFunction =
    fn(&mut NativeContext, Vec<Type>, VecDeque<Value>) -> PartialVMResult<NativeResult>;

pub type NativeFunctionTable = Vec<(AccountAddress, Identifier, Identifier, NativeFunction)>;

pub(crate) struct NativeFunctions(
    HashMap<AccountAddress, HashMap<String, HashMap<String, NativeFunction>>>,
);

impl NativeFunctions {
    pub fn resolve(
        &self,
        addr: &AccountAddress,
        module_name: &str,
        func_name: &str,
    ) -> Option<NativeFunction> {
        self.0.get(addr)?.get(module_name)?.get(func_name).cloned()
    }

    pub fn new<I>(natives: I) -> PartialVMResult<Self>
    where
        I: IntoIterator<Item = (AccountAddress, Identifier, Identifier, NativeFunction)>,
    {
        let mut map = HashMap::new();
        for (addr, module_name, func_name, func) in natives.into_iter() {
            let modules = map.entry(addr).or_insert_with(HashMap::new);
            let funcs = modules
                .entry(module_name.into_string())
                .or_insert_with(HashMap::new);

            if funcs.insert(func_name.into_string(), func).is_some() {
                return Err(PartialVMError::new(StatusCode::DUPLICATE_NATIVE_FUNCTION));
            }
        }
        Ok(Self(map))
    }
}

pub struct NativeContext<'a> {
    interpreter: &'a mut Interpreter,
    data_store: &'a mut dyn DataStore,
    gas_status: &'a GasStatus<'a>,
    resolver: &'a Resolver<'a>,
}

impl<'a, 'b> NativeContext<'a> {
    pub(crate) fn new(
        interpreter: &'a mut Interpreter,
        data_store: &'a mut dyn DataStore,
        gas_status: &'a mut GasStatus,
        resolver: &'a Resolver<'a>,
    ) -> Self {
        Self {
            interpreter,
            data_store,
            gas_status,
            resolver,
        }
    }
}

impl<'a> NativeContext<'a> {
    pub fn print_stack_trace<B: Write>(&self, buf: &mut B) -> PartialVMResult<()> {
        self.interpreter
            .debug_print_stack_trace(buf, self.resolver.loader())
    }

    pub fn cost_table(&self) -> &CostTable {
        self.gas_status.cost_table()
    }

    pub fn save_event(
        &mut self,
        guid: Vec<u8>,
        seq_num: u64,
        ty: Type,
        val: Value,
    ) -> PartialVMResult<bool> {
        match self.data_store.emit_event(guid, seq_num, ty, val) {
            Ok(()) => Ok(true),
            Err(e) if e.major_status().status_type() == StatusType::InvariantViolation => Err(e),
            Err(_) => Ok(false),
        }
    }

    pub fn type_to_type_layout(&self, ty: &Type) -> PartialVMResult<Option<MoveTypeLayout>> {
        match self.resolver.type_to_type_layout(ty) {
            Ok(ty_layout) => Ok(Some(ty_layout)),
            Err(e) if e.major_status().status_type() == StatusType::InvariantViolation => Err(e),
            Err(_) => Ok(None),
        }
    }
}
