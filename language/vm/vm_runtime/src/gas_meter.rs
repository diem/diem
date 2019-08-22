// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Gas metering logic for the Move VM.
use crate::{
    code_cache::module_cache::ModuleCache, execution_stack::ExecutionStack,
    loaded_data::function::FunctionReference,
};
use types::{account_address::ADDRESS_LENGTH, transaction::MAX_TRANSACTION_SIZE_IN_BYTES};
use vm::{access::ModuleAccess, errors::*, file_format::Bytecode, gas_schedule::*};
use vm_runtime_types::value::Local;

/// Holds the state of the gas meter.
pub struct GasMeter {
    // The current amount of gas that is left ("unburnt gas") in the gas meter.
    current_gas_left: GasUnits<GasCarrier>,

    // We need to disable and enable gas metering for both the prologue and epilogue of the Account
    // contract. The VM will then internally unset/set this flag before executing either of them.
    meter_on: bool,
}

// NB: A number of the functions/methods in this struct will return a VMResult<T>
// since we will need to access stack and memory states, and we need to be able
// to report errors properly from these accesses.
impl GasMeter {
    /// Create a new gas meter with starting gas amount `gas_amount`
    pub fn new(gas_amount: GasUnits<GasCarrier>) -> Self {
        GasMeter {
            current_gas_left: gas_amount,
            meter_on: true,
        }
    }

    /// Charges additional gas for the transaction based upon the total size (in bytes) of the
    /// submitted transaction. It is important that we charge for the transaction size since a
    /// transaction can contain arbitrary amounts of bytes in the `note` field. We also want to
    /// disinsentivize large transactions with large notes, so we charge the same amount up to a
    /// cutoff, after which we start charging at a greater rate.
    pub fn charge_transaction_gas<'alloc, 'txn, P>(
        &mut self,
        transaction_size: AbstractMemorySize<GasCarrier>,
        stk: &ExecutionStack<'alloc, 'txn, P>,
    ) -> VMResult<()>
    where
        'alloc: 'txn,
        P: ModuleCache<'alloc>,
    {
        precondition!(transaction_size.get() <= (MAX_TRANSACTION_SIZE_IN_BYTES as u64));
        let cost = calculate_intrinsic_gas(transaction_size);
        self.consume_gas(cost, stk)
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

    /// A wrapper that calculates and then consumes the gas unless metering is disabled.
    pub fn calculate_and_consume<'alloc, 'txn, P>(
        &mut self,
        instr: &Bytecode,
        stk: &ExecutionStack<'alloc, 'txn, P>,
        memory_size: AbstractMemorySize<GasCarrier>,
    ) -> VMResult<()>
    where
        'alloc: 'txn,
        P: ModuleCache<'alloc>,
    {
        if self.meter_on {
            let instruction_gas = try_runtime!(self.gas_for_instruction(instr, stk, memory_size));
            self.consume_gas(instruction_gas, stk)
        } else {
            Ok(Ok(()))
        }
    }

    /// Calculate the gas usage for an instruction taking into account the current stack state, and
    /// the size of memory that is being accessed.
    pub fn gas_for_instruction<'alloc, 'txn, P>(
        &mut self,
        instr: &Bytecode,
        stk: &ExecutionStack<'alloc, 'txn, P>,
        memory_size: AbstractMemorySize<GasCarrier>,
    ) -> VMResult<GasUnits<GasCarrier>>
    where
        'alloc: 'txn,
        P: ModuleCache<'alloc>,
    {
        // Get the base cost for the instruction.
        let instruction_reqs = match instr {
            Bytecode::Add
            | Bytecode::Sub
            | Bytecode::Mul
            | Bytecode::Mod
            | Bytecode::Div
            | Bytecode::BitOr
            | Bytecode::BitAnd
            | Bytecode::Xor
            | Bytecode::Or
            | Bytecode::And
            | Bytecode::Not
            | Bytecode::Eq
            | Bytecode::Neq
            | Bytecode::Lt
            | Bytecode::Gt
            | Bytecode::Le
            | Bytecode::LdTrue
            | Bytecode::LdFalse
            | Bytecode::LdConst(_)
            | Bytecode::Branch(_)
            | Bytecode::Abort
            | Bytecode::Pop
            | Bytecode::BrTrue(_)
            | Bytecode::BrFalse(_)
            | Bytecode::GetTxnGasUnitPrice
            | Bytecode::GetTxnMaxGasUnits
            | Bytecode::GetGasRemaining
            | Bytecode::GetTxnPublicKey
            | Bytecode::GetTxnSenderAddress
            | Bytecode::GetTxnSequenceNumber
            | Bytecode::Ge
            | Bytecode::FreezeRef => {
                let default_gas = static_cost_instr(instr, AbstractMemorySize::new(1));
                Self::gas_of(default_gas)
            }
            Bytecode::LdAddr(_) => {
                let size = AbstractMemorySize::new(ADDRESS_LENGTH as GasCarrier);
                let default_gas = static_cost_instr(instr, size);
                Self::gas_of(default_gas)
            }
            Bytecode::LdByteArray(idx) => {
                let byte_array_ref = stk.top_frame()?.module().byte_array_at(*idx);
                let byte_array_len = AbstractMemorySize::new(byte_array_ref.len() as GasCarrier);
                let byte_array_len = words_in(byte_array_len);
                let default_gas = static_cost_instr(instr, byte_array_len);
                Self::gas_of(default_gas)
            }
            // We charge by the length of the string being stored on the stack.
            Bytecode::LdStr(idx) => {
                let string_ref = stk.top_frame()?.module().string_at(*idx);
                let str_len = AbstractMemorySize::new(string_ref.len() as GasCarrier);
                let str_len = words_in(str_len);
                let default_gas = static_cost_instr(instr, str_len);
                Self::gas_of(default_gas)
            }
            Bytecode::StLoc(_) => {
                // Get the local to store
                let local = stk.peek()?;
                // Get the size of the local
                let size = local.size();
                let default_gas = static_cost_instr(instr, size);
                Self::gas_of(default_gas)
            }
            // Note that a moveLoc incurs a copy overhead
            Bytecode::CopyLoc(local_idx) | Bytecode::MoveLoc(local_idx) => {
                let local = stk.top_frame()?.get_local(*local_idx)?;
                let size = local.size();
                let default_gas = static_cost_instr(instr, size);
                Self::gas_of(default_gas)
            }
            // A return does not affect the value stack at all, and simply pops the call stack
            // -- the callee's frame then knows that the return value(s) will be at the top of the
            // value stack.  Because of this, the cost of the instruction is not dependent upon the
            // size of the value being returned.
            Bytecode::Ret => {
                let default_gas = static_cost_instr(instr, AbstractMemorySize::new(1));
                Self::gas_of(default_gas)
            }
            Bytecode::Call(call_idx, _) => {
                let self_module = &stk.top_frame()?.module();
                let function_ref = try_runtime!(stk
                    .module_cache
                    .resolve_function_ref(self_module, *call_idx))
                .ok_or(VMInvariantViolation::LinkerError)?;
                if function_ref.is_native() {
                    GasUnits::new(0) // This will be costed at the call site/by the native function
                } else {
                    let call_size = AbstractMemorySize::new(function_ref.arg_count() as GasCarrier);
                    let call_gas = static_cost_instr(instr, call_size);
                    Self::gas_of(call_gas)
                }
            }
            Bytecode::Unpack(_, _) => {
                let size = stk.peek()?.size();
                Self::gas_of(static_cost_instr(instr, size))
            }
            Bytecode::Pack(struct_idx, _) => {
                let struct_def = &stk.top_frame()?.module().struct_def_at(*struct_idx);
                // Similar logic applies here as in Call, so we probably don't need to take
                // into account the size of the values on the value stack that we are placing into
                // the struct.
                let member_count = struct_def.declared_field_count()?;
                let arg_count = AbstractMemorySize::new(u64::from(member_count));

                let total_size = arg_count.add(*STRUCT_SIZE);
                let new_gas = static_cost_instr(instr, total_size);
                Self::gas_of(new_gas)
            }
            Bytecode::WriteRef => {
                // Get a reference to the value that we are going to write
                let write_val = stk.peek_at(1)?;
                // Grab the reference that's going to be written to
                let ref_val = stk.peek()?;
                // Get the size of this value and charge accordingly.
                let size = write_val.size();
                let mut default_gas = static_cost_instr(instr, size);
                // Determine if the reference is global. If so charge for any expansion of global
                // memory along with the write operation that will be incurred.
                if let Local::GlobalRef(_) = ref_val {
                    // Charge for any memory expansion
                    let new_val_size = ref_val.size();
                    let size_difference = if new_val_size.app(&size, |new_vl_size, size| new_vl_size > size) {
                        new_val_size.sub(size)
                    } else {
                        // The difference is always >= 0
                        AbstractMemorySize::new(0)
                    };
                    default_gas.memory_gas = default_gas.memory_gas
                        // Charge for the iops on global memory
                        .add(size.mul(*GLOBAL_MEMORY_PER_BYTE_WRITE_COST))
                        // Charge for any memory expansion
                        .add(size_difference.mul(*GLOBAL_MEMORY_PER_BYTE_COST));
                };
                Self::gas_of(default_gas)
            }
            | Bytecode::ReadRef => {
                // NB: We don't charge for reads from global memory: we charge once for the read
                // from global memory that is performed by a BorrowGlobal operation. After this,
                // all ReadRefs will be reading from local cache and we don't need to distinguish.
                let size = stk.peek()?.size();
                let default_gas = static_cost_instr(instr, size);
                Self::gas_of(default_gas)
            }
            | Bytecode::MutBorrowLoc(_)
            | Bytecode::ImmBorrowLoc(_)
            | Bytecode::MutBorrowField(_)
            | Bytecode::ImmBorrowField(_) => {
                let default_gas = static_cost_instr(instr, AbstractMemorySize::new(1));
                Self::gas_of(default_gas)
            }
            Bytecode::CreateAccount => Self::gas_of(static_cost_instr(instr, *DEFAULT_ACCOUNT_SIZE)),
            // Releasing a reference is not dependent on the size of the underlying data
            Bytecode::ReleaseRef => {
                Self::gas_of(static_cost_instr(instr, AbstractMemorySize::new(1)))
            }
            // Note that we charge twice for these operations; once at the start of
            // `execute_single_instruction` we charge once with size 1. This then covers the cost
            // of accessing the value and guards (somewhat) against abusive memory accesses. Once
            // we have the value/resource in hand we then charge a cost that is dependent on the
            // size of the value being moved.
            //
            // Borrowing a global causes a read of the underlying data. Therefore the cost is
            // dependent on the size of the data being borrowed.
            Bytecode::BorrowGlobal(_, _)
            // In the process of determining if a resource exists, we need to load/read that
            // memory. We therefore need to charge for this query based on the size of the data
            // being accessed.
            | Bytecode::Exists(_, _)
            // A MoveFrom does not trigger a write to memory. But it does push the value of that
            // size onto the stack. So we charge based upon the size of the instruction.
            | Bytecode::MoveFrom(_, _)
            // A MoveToSender causes a write of the resource to storage. We therefore charge based
            // on the size of the resource being moved.
            | Bytecode::MoveToSender(_, _) => {
                let mem_size = if memory_size.get() > 1 {
                    memory_size.sub(AbstractMemorySize::new(1))
                } else {
                    AbstractMemorySize::new(0) // We already charged for size 1
                };
                Self::gas_of(static_cost_instr(instr, mem_size))
            }
        };
        Ok(Ok(instruction_reqs))
    }

    /// Get the amount of gas that remains (that has _not_ been consumed) in the gas meter.
    ///
    /// This method is used by the `GetGasRemaining` bytecode instruction to get the current
    /// amount of gas remaining at the point of call.
    pub fn remaining_gas(&self) -> GasUnits<GasCarrier> {
        self.current_gas_left
    }

    /// Consume the amount of gas given by `gas_amount`. If there is not enough gas
    /// left in the internal state, an `OutOfGasError` is returned.
    pub fn consume_gas<'alloc, 'txn, P>(
        &mut self,
        gas_amount: GasUnits<GasCarrier>,
        stk: &ExecutionStack<'alloc, 'txn, P>,
    ) -> VMResult<()>
    where
        'alloc: 'txn,
        P: ModuleCache<'alloc>,
    {
        if !self.meter_on {
            return Ok(Ok(()));
        }
        if self
            .current_gas_left
            .app(&gas_amount, |curr_gas, gas_amt| curr_gas >= gas_amt)
        {
            self.current_gas_left = self.current_gas_left.sub(gas_amount);
            Ok(Ok(()))
        } else {
            // Zero out the internal gas state
            self.current_gas_left = GasUnits::new(0);
            let location = stk.location().unwrap_or_default();
            Ok(Err(VMRuntimeError {
                loc: location,
                err: VMErrorKind::OutOfGasError,
            }))
        }
    }

    /// Take a GasCost from our gas schedule and convert it to a total gas charge in `GasUnits`.
    ///
    /// This is used internally for converting from a `GasCost` which is a triple of numbers
    /// represeing instruction, stack, and memory consumption into a number of `GasUnits`.
    fn gas_of(gas_cost: GasCost) -> GasUnits<GasCarrier> {
        gas_cost.instruction_gas.add(gas_cost.memory_gas)
    }
}
