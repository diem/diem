// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::{hash, primitive_helpers, signature};
use crate::{
    native_structs::dispatch::resolve_native_struct,
    values::{vector, Value},
};
use libra_types::{
    account_address::AccountAddress,
    account_config,
    identifier::{IdentStr, Identifier},
    language_storage::{ModuleId, TypeTag},
    vm_error::{StatusCode, VMStatus},
};
use once_cell::sync::Lazy;
use std::collections::{HashMap, VecDeque};
use vm::{
    errors::VMResult,
    file_format::{FunctionSignature, Kind, SignatureToken, StructHandleIndex},
    gas_schedule::{
        AbstractMemorySize, CostTable, GasAlgebra, GasCarrier, GasUnits, NativeCostIndex,
    },
};

/// Result of a native function execution that requires charges for execution cost.
///
/// An execution that causes an invariant violation would not return a `NativeResult` but
/// return a `VMResult` error directly.
/// All native functions must return a `VMResult<NativeResult>` where an `Err` is returned
/// when an error condition is met that should not charge for the execution. A common example
/// is a VM invariant violation which should have been forbidden by the verifier.
/// Errors (typically user errors and aborts) that are logically part of the function execution
/// must be expressed in a `NativeResult` via a cost and a VMStatus.
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

/// Struct representing the expected definition for a native function.
pub struct NativeFunction {
    /// Given the vector of aguments, it executes the native function.
    pub dispatch: fn(Vec<TypeTag>, VecDeque<Value>, &CostTable) -> VMResult<NativeResult>,
    /// The signature as defined in it's declaring module.
    /// It should NOT be generally inspected outside of it's declaring module as the various
    /// struct handle indexes are not remapped into the local context.
    pub expected_signature: FunctionSignature,
}

impl NativeFunction {
    /// Returns the number of arguments to the native function, derived from the expected signature.
    pub fn num_args(&self) -> usize {
        self.expected_signature.arg_types.len()
    }
}

/// Looks up the expected native function definition from the module id (address and module) and
/// function name where it was expected to be declared.
pub fn resolve_native_function(
    module: &ModuleId,
    function_name: &IdentStr,
) -> Option<&'static NativeFunction> {
    NATIVE_FUNCTION_MAP.get(module)?.get(function_name)
}

pub fn native_gas(table: &CostTable, key: NativeCostIndex, size: usize) -> GasUnits<GasCarrier> {
    let gas_amt = table.native_cost(key);
    let memory_size = AbstractMemorySize::new(size as GasCarrier);
    gas_amt.total().mul(memory_size)
}

macro_rules! add {
    ($m:ident, $addr:expr, $module:expr, $name:expr, $dis:expr, $args:expr, $ret:expr) => {{
        add!($m, $addr, $module, $name, $dis, vec![], $args, $ret)
    }};
    ($m:ident, $addr:expr, $module:expr, $name:expr, $dis:expr, $kinds:expr, $args:expr, $ret:expr) => {{
        let expected_signature = FunctionSignature {
            return_types: $ret,
            arg_types: $args,
            type_formals: $kinds,
        };
        let f = NativeFunction {
            dispatch: $dis,
            expected_signature,
        };
        let id = ModuleId::new($addr, Identifier::new($module).unwrap());
        let old = $m
            .entry(id)
            .or_insert_with(HashMap::new)
            .insert(Identifier::new($name).unwrap(), f);
        assert!(old.is_none());
    }};
}

/// Helper for finding expected struct handle index.
fn tstruct(
    addr: AccountAddress,
    module_name: &str,
    function_name: &str,
    args: Vec<SignatureToken>,
) -> SignatureToken {
    let id = ModuleId::new(addr, Identifier::new(module_name).unwrap());
    let native_struct =
        resolve_native_struct(&id, &Identifier::new(function_name).unwrap()).unwrap();
    let idx = native_struct.expected_index;
    // TODO assert kinds match
    assert_eq!(args.len(), native_struct.expected_type_formals.len());
    SignatureToken::Struct(idx, args)
}

type NativeFunctionMap = HashMap<ModuleId, HashMap<Identifier, NativeFunction>>;

static NATIVE_FUNCTION_MAP: Lazy<NativeFunctionMap> = Lazy::new(|| {
    use SignatureToken::*;
    let mut m: NativeFunctionMap = HashMap::new();
    let addr = account_config::core_code_address();
    // Hash
    add!(
        m,
        addr,
        "Hash",
        "sha2_256",
        hash::native_sha2_256,
        vec![ByteArray],
        vec![ByteArray]
    );
    add!(
        m,
        addr,
        "Hash",
        "sha3_256",
        hash::native_sha3_256,
        vec![ByteArray],
        vec![ByteArray]
    );
    // Signature
    add!(
        m,
        addr,
        "Signature",
        "ed25519_verify",
        signature::native_ed25519_signature_verification,
        vec![ByteArray, ByteArray, ByteArray],
        vec![Bool]
    );
    add!(
        m,
        addr,
        "Signature",
        "ed25519_threshold_verify",
        signature::native_ed25519_threshold_signature_verification,
        vec![ByteArray, ByteArray, ByteArray, ByteArray],
        vec![U64]
    );
    // AddressUtil
    add!(
        m,
        addr,
        "AddressUtil",
        "address_to_bytes",
        primitive_helpers::native_address_to_bytes,
        vec![Address],
        vec![ByteArray]
    );
    // U64Util
    add!(
        m,
        addr,
        "U64Util",
        "u64_to_bytes",
        primitive_helpers::native_u64_to_bytes,
        vec![U64],
        vec![ByteArray]
    );
    // BytearrayUtil
    add!(
        m,
        addr,
        "BytearrayUtil",
        "bytearray_concat",
        primitive_helpers::native_bytearray_concat,
        vec![ByteArray, ByteArray],
        vec![ByteArray]
    );
    // Vector
    add!(
        m,
        addr,
        "Vector",
        "length",
        vector::native_length,
        vec![Kind::All],
        vec![Reference(Box::new(tstruct(
            addr,
            "Vector",
            "T",
            vec![TypeParameter(0)]
        )))],
        vec![U64]
    );
    add!(
        m,
        addr,
        "Vector",
        "empty",
        vector::native_empty,
        vec![Kind::All],
        vec![],
        vec![tstruct(addr, "Vector", "T", vec![TypeParameter(0)]),]
    );
    add!(
        m,
        addr,
        "Vector",
        "borrow",
        vector::native_borrow,
        vec![Kind::All],
        vec![
            Reference(Box::new(tstruct(
                addr,
                "Vector",
                "T",
                vec![TypeParameter(0)]
            ))),
            U64
        ],
        vec![Reference(Box::new(TypeParameter(0)))]
    );
    add!(
        m,
        addr,
        "Vector",
        "borrow_mut",
        vector::native_borrow,
        vec![Kind::All],
        vec![
            MutableReference(Box::new(tstruct(
                addr,
                "Vector",
                "T",
                vec![TypeParameter(0)]
            ))),
            U64
        ],
        vec![MutableReference(Box::new(TypeParameter(0)))]
    );
    add!(
        m,
        addr,
        "Vector",
        "push_back",
        vector::native_push_back,
        vec![Kind::All],
        vec![
            MutableReference(Box::new(tstruct(
                addr,
                "Vector",
                "T",
                vec![TypeParameter(0)]
            ))),
            TypeParameter(0),
        ],
        vec![]
    );
    add!(
        m,
        addr,
        "Vector",
        "pop_back",
        vector::native_pop,
        vec![Kind::All],
        vec![MutableReference(Box::new(tstruct(
            addr,
            "Vector",
            "T",
            vec![TypeParameter(0)]
        )))],
        vec![TypeParameter(0)]
    );
    add!(
        m,
        addr,
        "Vector",
        "destroy_empty",
        vector::native_destroy_empty,
        vec![Kind::All],
        vec![tstruct(addr, "Vector", "T", vec![TypeParameter(0)])],
        vec![]
    );
    add!(
        m,
        addr,
        "Vector",
        "swap",
        vector::native_swap,
        vec![Kind::All],
        vec![
            MutableReference(Box::new(tstruct(
                addr,
                "Vector",
                "T",
                vec![TypeParameter(0)]
            ))),
            U64,
            U64,
        ],
        vec![]
    );

    //
    // TODO: both API bolow are directly implemented in the interepreter as we lack a
    // good mechanism to expose certain API to native functions.
    // Specifically we need access to some frame information (e.g. type instantiations) and
    // access to the data store.
    // Maybe marking native functions in a certain way (e.g `system` or similar) may
    // be a way for the VM to force a given argument to the native implementation.
    // Alternative models are fine too...
    //

    // Event
    add!(
        m,
        addr,
        "LibraAccount",
        "write_to_event_store",
        |_, _, _| {
            Err(VMStatus::new(StatusCode::UNREACHABLE).with_message(
                "write_to_event_store does not have a native implementation".to_string(),
            ))
        },
        vec![Kind::Unrestricted],
        vec![ByteArray, U64, TypeParameter(0)],
        vec![]
    );
    // LibraAccount
    add!(
        m,
        addr,
        "LibraAccount",
        "save_account",
        |_, _, _| {
            Err(VMStatus::new(StatusCode::UNREACHABLE)
                .with_message("save_account does not have a native implementation".to_string()))
        },
        vec![
            Address,
            // this is LibraAccount.T which happens to be the first struct handle in the
            // binary.
            // TODO: current plan is to rework the description of the native function
            // by using the binary directly and have functions that fetch the arguments
            // go through the signature for extra verification. That is the plan if perf
            // and the model look good.
            Struct(StructHandleIndex::new(0), vec![]),
        ],
        vec![]
    );
    m
});

#[macro_export]
macro_rules! pop_arg {
    ($arguments:ident, $t:ty) => {{
        $arguments.pop_back().unwrap().value_as::<$t>()?
    }};
}
