// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Gas metering logic for the Move VM.
use crate::{
    code_cache::module_cache::ModuleCache,
    data_cache::RemoteCache,
    identifier::{create_access_path, resource_storage_key},
};
use libra_types::{
    account_config,
    identifier::Identifier,
    language_storage::ModuleId,
    vm_error::{sub_status, StatusCode, VMStatus},
};
use vm::{errors::VMResult, gas_schedule::*};

//***************************************************************************
// Gas Schedule Loading
//***************************************************************************

lazy_static! {
    /// The ModuleId for the gas schedule module
    pub static ref GAS_SCHEDULE_MODULE: ModuleId =
        { ModuleId::new(account_config::core_code_address(), Identifier::new("GasSchedule").unwrap()) };
}

pub(crate) fn load_gas_schedule(
    module_cache: &dyn ModuleCache,
    data_view: &dyn RemoteCache,
) -> VMResult<CostTable> {
    let address = account_config::association_address();
    let gas_module = module_cache
        .get_loaded_module(&GAS_SCHEDULE_MODULE)
        .map_err(|_| {
            VMStatus::new(StatusCode::GAS_SCHEDULE_ERROR)
                .with_sub_status(sub_status::GSE_UNABLE_TO_LOAD_MODULE)
        })?;

    let gas_struct_def_idx = gas_module.get_struct_def_index(&GAS_SCHEDULE_NAME)?;
    let struct_tag = resource_storage_key(gas_module, *gas_struct_def_idx);
    let access_path = create_access_path(&address, struct_tag);

    let data_blob = data_view
        .get(&access_path)
        .map_err(|_| {
            VMStatus::new(StatusCode::GAS_SCHEDULE_ERROR)
                .with_sub_status(sub_status::GSE_UNABLE_TO_LOAD_RESOURCE)
        })?
        .ok_or_else(|| {
            VMStatus::new(StatusCode::GAS_SCHEDULE_ERROR)
                .with_sub_status(sub_status::GSE_UNABLE_TO_LOAD_RESOURCE)
        })?;
    let table: CostTable = lcs::from_bytes(&data_blob).map_err(|_| {
        VMStatus::new(StatusCode::GAS_SCHEDULE_ERROR)
            .with_sub_status(sub_status::GSE_UNABLE_TO_DESERIALIZE)
    })?;
    Ok(table)
}

//***************************************************************************
// Gas Metering Logic
//***************************************************************************

#[macro_export]
macro_rules! gas {
    (instr: $context:ident, $self:ident, $opcode:path, $mem_size:expr) => {
        $context.deduct_gas(
            $self
                .gas_schedule
                .instruction_cost($opcode as u8)
                .total()
                .mul($mem_size),
        )
    };
    (const_instr: $context:ident, $self:ident, $opcode:path) => {
        $context.deduct_gas($self.gas_schedule.instruction_cost($opcode as u8).total())
    };
    (consume: $context:ident, $expr:expr) => {
        $context.deduct_gas($expr)
    };
}
