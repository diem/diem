// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Native Function Support
//!
//! All move native functions have the following signature:
//!
//! `pub fn native_function(
//!     context: &mut impl NativeContext,
//!     ty_args: Vec<Type>,
//!     mut arguments: VecDeque<Value>,
//! ) -> VMResult<NativeResult>;`
//!
//! arguments are passed with first argument at position 0 and so forth.
//! Popping values from `arguments` gives the aguments in reverse order (last first).
//! This module contains the declarations and utilities to implement a native
//! function.

use crate::{
    loaded_data::{runtime_types::Type, types::FatType},
    values::{Struct, Value},
};
use libra_types::{
    account_address::AccountAddress, contract_event::ContractEvent, language_storage::ModuleId,
    vm_error::VMStatus,
};
use move_core_types::{
    gas_schedule::{
        AbstractMemorySize, CostTable, GasAlgebra, GasCarrier, GasUnits, NativeCostIndex,
    },
    identifier::IdentStr,
};
use std::fmt::Write;
use vm::errors::VMResult;

/// `NativeContext` - Native function context.
///
/// This is the API, the "privileges", a native function is given.
/// Normally a native function will only need the `CostTable`.
/// The set of native functions and their linkage is entirely inside the MoveVM
/// runtime.
pub trait NativeContext {
    /// Prints stack trace.
    fn print_stack_trace<B: Write>(&self, buf: &mut B) -> VMResult<()>;
    /// Gets cost table ref.
    fn cost_table(&self) -> &CostTable;
    // Save a resource under the address specified by `account_address`
    fn save_under_address(
        &mut self,
        ty_args: &[Type],
        module_id: &ModuleId,
        struct_name: &IdentStr,
        resource_to_save: Struct,
        account_address: AccountAddress,
    ) -> VMResult<()>;
    /// Saves contract event.
    fn save_event(&mut self, event: ContractEvent) -> VMResult<()>;
    /// Converts types to fet types.
    fn convert_to_fat_types(&self, types: Vec<Type>) -> VMResult<Vec<FatType>>;
}

/// Result of a native function execution requires charges for execution cost.
///
/// An execution that causes an invariant violation would not return a `NativeResult` but
/// return a `VMResult` error directly.
/// All native functions must return a `VMResult<NativeResult>` where an `Err` is returned
/// when an error condition is met that should not charge for the execution. A common example
/// is a VM invariant violation which should have been forbidden by the verifier.
/// Errors (typically user errors and aborts) that are logically part of the function execution
/// must be expressed in a `NativeResult` with a cost and a VMStatus.
pub struct NativeResult {
    /// The cost for running that function, whether successfully or not.
    pub cost: GasUnits<GasCarrier>,
    /// Result of execution. This is either the return values or the error to report.
    pub result: VMResult<Vec<Value>>,
}

impl NativeResult {
    /// Return values of a successful execution.
    pub fn ok(cost: GasUnits<GasCarrier>, values: Vec<Value>) -> Self {
        NativeResult {
            cost,
            result: Ok(values),
        }
    }

    /// `VMStatus` of a failed execution. The failure is a runtime failure in the function
    /// and not an invariant failure of the VM which would raise a `VMResult` error directly.
    pub fn err(cost: GasUnits<GasCarrier>, err: VMStatus) -> Self {
        NativeResult {
            cost,
            result: Err(err),
        }
    }
}

/// Return the native gas entry in `CostTable` for the given key.
/// The key is the specific native function index known to `CostTable`.
pub fn native_gas(table: &CostTable, key: NativeCostIndex, size: usize) -> GasUnits<GasCarrier> {
    let gas_amt = table.native_cost(key);
    let memory_size = AbstractMemorySize::new(size as GasCarrier);
    gas_amt.total().mul(memory_size)
}

/// Return the argument at the top of the stack.
///
/// Arguments are passed to a native as a stack with first arg at the bottom of the stack.
/// Calling this API can help in making the code more readable.
/// It's good practice to pop all arguments in locals of the native function on function entry.
#[macro_export]
macro_rules! pop_arg {
    ($arguments:ident, $t:ty) => {{
        $arguments.pop_back().unwrap().value_as::<$t>()?
    }};
}
