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
    transaction::MAX_TRANSACTION_SIZE_IN_BYTES,
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

pub(crate) fn load_gas_schedule<'alloc>(
    module_cache: &dyn ModuleCache<'alloc>,
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
    (instr: $self:ident, $opcode:path, $mem_size:expr) => {
        $self.gas_meter.consume_instruction_gas($opcode, $mem_size)
    };
    (const_instr: $self:ident, $opcode:path) => {
        $self
            .gas_meter
            .consume_instruction_gas($opcode, *CONST_SIZE)
    };
    (consume: $self:ident, $expr:expr) => {
        $self.gas_meter.consume_gas($expr)
    };
}

/// Holds the state of the gas meter.
pub struct GasMeter<'txn> {
    // The current amount of gas that is left ("unburnt gas") in the gas meter.
    current_gas_left: GasUnits<GasCarrier>,

    // The gas schedule used for computing the gas cost per bytecode instruction.
    gas_schedule: &'txn CostTable,

    // We need to disable and enable gas metering for both the prologue and epilogue of the Account
    // contract. The VM will then internally unset/set this flag before executing either of them.
    meter_on: bool,
}

// NB: A number of the functions/methods in this struct will return a VMResult<T>
// since we will need to access stack and memory states, and we need to be able
// to report errors properly from these accesses.
impl<'txn> GasMeter<'txn> {
    /// Create a new gas meter with starting gas amount `gas_amount`
    pub fn new(gas_amount: GasUnits<GasCarrier>, gas_schedule: &'txn CostTable) -> Self {
        GasMeter {
            current_gas_left: gas_amount,
            gas_schedule,
            meter_on: true,
        }
    }

    pub fn cost_table(&self) -> &'txn CostTable {
        &self.gas_schedule
    }

    /// Charges additional gas for the transaction based upon the total size (in bytes) of the
    /// submitted transaction. It is important that we charge for the transaction size since a
    /// transaction can contain arbitrary amounts of bytes in the `note` field. We also want to
    /// disinsentivize large transactions with large notes, so we charge the same amount up to a
    /// cutoff, after which we start charging at a greater rate.
    pub fn charge_transaction_gas(
        &mut self,
        transaction_size: AbstractMemorySize<GasCarrier>,
    ) -> VMResult<()> {
        precondition!(transaction_size.get() <= (MAX_TRANSACTION_SIZE_IN_BYTES as u64));
        let cost = calculate_intrinsic_gas(transaction_size);
        self.consume_gas(cost)
    }

    /// Queries the internal state of the gas meter to determine if it has at
    /// least `needed_gas` amount of gas.
    pub fn has_gas(&self, needed_gas: GasUnits<GasCarrier>) -> bool {
        self.current_gas_left
            .app(&needed_gas, |curr_gas, needed_gas| curr_gas >= needed_gas)
    }

    /// Disables metering of gas.
    ///
    /// We need to disable and enable gas metering for both the prologue and epilogue of the
    /// Account contract. The VM will then internally turn off gas metering before executing either
    /// of them using this method.
    pub fn disable_metering(&mut self) {
        self.meter_on = false;
    }

    /// Re-enables metering of gas.
    ///
    /// After executing the prologue and epilogue in the Account contract gas metering is re-enabled
    /// using this method. The VM is responsible for internally calling this method after disabling
    /// gas metering.
    pub fn enable_metering(&mut self) {
        self.meter_on = true;
    }

    /// Get the amount of gas that remains (that has _not_ been consumed) in the gas meter.
    ///
    /// This method is used by the `GetGasRemaining` bytecode instruction to get the current
    /// amount of gas remaining at the point of call.
    pub fn remaining_gas(&self) -> GasUnits<GasCarrier> {
        self.current_gas_left
    }

    /// A wrapper that calculates and then consumes the gas unless metering is disabled.
    #[inline]
    pub fn consume_instruction_gas(
        &mut self,
        instruction_key: Opcodes,
        memory_size: AbstractMemorySize<GasCarrier>,
    ) -> VMResult<()> {
        if self.meter_on {
            let gas_cost = self.gas_schedule.instruction_cost(instruction_key as u8);
            let gas = gas_cost.total().mul(memory_size);
            self.consume_gas(gas)
        } else {
            Ok(())
        }
    }

    /// Consume the amount of gas given by `gas_amount`. If there is not enough gas
    /// left in the internal state, an `OutOfGasError` is returned.
    pub fn consume_gas(&mut self, gas_amount: GasUnits<GasCarrier>) -> VMResult<()> {
        if !self.meter_on {
            return Ok(());
        }
        if self
            .current_gas_left
            .app(&gas_amount, |curr_gas, gas_amt| curr_gas >= gas_amt)
        {
            self.current_gas_left = self.current_gas_left.sub(gas_amount);
            Ok(())
        } else {
            // Zero out the internal gas state
            self.current_gas_left = GasUnits::new(0);
            Err(VMStatus::new(StatusCode::OUT_OF_GAS))
        }
    }
}
