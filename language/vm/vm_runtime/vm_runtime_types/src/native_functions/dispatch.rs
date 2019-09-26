// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::{hash, primitive_helpers, signature};
use crate::{
    native_structs::{dispatch::dispatch_native_struct, vector::NativeVector},
    value::Value,
};
use std::collections::{HashMap, VecDeque};
use types::{
    account_address::AccountAddress,
    account_config,
    identifier::{IdentStr, Identifier},
    language_storage::ModuleId,
};
use vm::file_format::{FunctionSignature, Kind, SignatureToken};

/// Enum representing the result of running a native function
pub enum NativeReturnStatus {
    /// Represents a successful execution.
    Success {
        /// The cost for running that function
        cost: u64,
        /// The `Vec<Value>` values will be pushed on the stack
        return_values: Vec<Value>,
    },
    /// Represents the execution of an abort instruction with the given error code
    Aborted {
        /// The cost for running that function up to the point of the abort
        cost: u64,
        /// The error code aborted on
        error_code: u64,
    },
    /// `InvalidArguments` should not occur unless there is some error in the bytecode verifier
    InvalidArguments,
}

/// Struct representing the expected definition for a native function
pub struct NativeFunction {
    /// Given the vector of aguments, it executes the native function
    pub dispatch: fn(VecDeque<Value>) -> NativeReturnStatus,
    /// The signature as defined in it's declaring module.
    /// It should NOT be generally inspected outside of it's declaring module as the various
    /// struct handle indexes are not remapped into the local context
    pub expected_signature: FunctionSignature,
}

impl NativeFunction {
    /// Returns the number of arguments to the native function, derived from the expected signature
    pub fn num_args(&self) -> usize {
        self.expected_signature.arg_types.len()
    }
}

/// Looks up the expected native function definition from the module id (address and module) and
/// function name where it was expected to be declared
pub fn dispatch_native_function(
    module: &ModuleId,
    function_name: &IdentStr,
) -> Option<&'static NativeFunction> {
    NATIVE_FUNCTION_MAP.get(module)?.get(function_name)
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

/// Helper for finding expected struct handle index
fn tstruct(
    addr: AccountAddress,
    module_name: &str,
    function_name: &str,
    args: Vec<SignatureToken>,
) -> SignatureToken {
    let id = ModuleId::new(addr, Identifier::new(module_name).unwrap());
    let native_struct =
        dispatch_native_struct(&id, &Identifier::new(function_name).unwrap()).unwrap();
    let idx = native_struct.expected_index;
    // TODO assert kinds match
    assert_eq!(args.len(), native_struct.expected_type_formals.len());
    SignatureToken::Struct(idx, args)
}

type NativeFunctionMap = HashMap<ModuleId, HashMap<Identifier, NativeFunction>>;

lazy_static! {
    static ref NATIVE_FUNCTION_MAP: NativeFunctionMap = {
        use SignatureToken::*;
        let mut m: NativeFunctionMap = HashMap::new();
        let addr = account_config::core_code_address();
        // Hash
        add!(m, addr, "Hash", "sha2_256",
            hash::native_sha2_256,
            vec![ByteArray],
            vec![ByteArray]
        );
        add!(m, addr, "Hash", "sha3_256",
            hash::native_sha3_256,
            vec![ByteArray],
            vec![ByteArray]
        );
        // Signature
        add!(m, addr, "Signature", "ed25519_verify",
            signature::native_ed25519_signature_verification,
            vec![ByteArray, ByteArray, ByteArray],
            vec![Bool]
        );
        add!(m, addr, "Signature", "ed25519_threshold_verify",
            signature::native_ed25519_threshold_signature_verification,
            vec![ByteArray, ByteArray, ByteArray, ByteArray],
            vec![U64]
        );
        // AddressUtil
        add!(m, addr, "AddressUtil", "address_to_bytes",
            primitive_helpers::native_address_to_bytes,
            vec![Address],
            vec![ByteArray]
        );
        // U64Util
        add!(m, addr, "U64Util", "u64_to_bytes",
            primitive_helpers::native_u64_to_bytes,
            vec![U64],
            vec![ByteArray]
        );
        // BytearrayUtil
        add!(m, addr, "BytearrayUtil", "bytearray_concat",
            primitive_helpers::native_bytearray_concat,
            vec![ByteArray, ByteArray],
            vec![ByteArray]
        );
        // Vector
        add!(m, addr, "Vector", "length",
            NativeVector::native_length,
            vec![Kind::All],
            vec![Reference(Box::new(tstruct(addr, "Vector", "T", vec![TypeParameter(0)])))],
            vec![U64]
        );
        add!(m, addr, "Vector", "empty",
            NativeVector::native_empty,
            vec![Kind::All],
            vec![],
            vec![
                tstruct(addr, "Vector", "T", vec![TypeParameter(0)]),
            ]
        );
        add!(m, addr, "Vector", "borrow",
            NativeVector::native_borrow,
            vec![Kind::All],
            vec![
                Reference(Box::new(tstruct(addr, "Vector", "T", vec![TypeParameter(0)]))),
                U64],
            vec![
                Reference(Box::new(TypeParameter(0)))
            ]
        );
        add!(m, addr, "Vector", "borrow_mut",
            NativeVector::native_borrow,
            vec![Kind::All],
            vec![
                MutableReference(Box::new(tstruct(addr, "Vector", "T", vec![TypeParameter(0)]))),
                 U64],
            vec![
                MutableReference(Box::new(TypeParameter(0)))
            ]
        );
        add!(m, addr, "Vector", "push_back",
            NativeVector::native_push_back,
            vec![Kind::All],
            vec![
                MutableReference(Box::new(tstruct(addr, "Vector", "T", vec![TypeParameter(0)]))),
                TypeParameter(0),
            ],
            vec![]
        );
        add!(m, addr, "Vector", "pop_back",
            NativeVector::native_pop,
            vec![Kind::All],
            vec![MutableReference(Box::new(tstruct(addr, "Vector", "T", vec![TypeParameter(0)])))],
            vec![TypeParameter(0)]
        );
        add!(m, addr, "Vector", "destroy_empty",
            NativeVector::native_destroy_empty,
            vec![Kind::All],
            vec![tstruct(addr, "Vector", "T", vec![TypeParameter(0)])],
            vec![]
        );
        add!(m, addr, "Vector", "swap",
            NativeVector::native_swap,
            vec![Kind::All],
            vec![
                MutableReference(Box::new(tstruct(addr, "Vector", "T", vec![TypeParameter(0)]))),
                U64,
                U64,
            ],
            vec![]
        );
        // Event
        add!(m, addr, "Event", "write_to_event_store",
            |_| { NativeReturnStatus::InvalidArguments },
            vec![Kind::Unrestricted],
            vec![ByteArray, U64, TypeParameter(0)],
            vec![]
        );
        m
    };
}

#[macro_export]
macro_rules! pop_arg {
    ($arguments:ident, $t:ty) => {{
        match $arguments.pop_back().unwrap().value_as::<$t>() {
            Some(val) => val,
            None => return NativeReturnStatus::InvalidArguments,
        }
    }};
}
