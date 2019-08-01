// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::{hash, primitive_helpers, signature, vector};
use crate::{native_structs::dispatch::dispatch_native_struct, value::Local};
use std::collections::{HashMap, VecDeque};
use types::{account_address::AccountAddress, language_storage::ModuleId};
use vm::file_format::{FunctionSignature, SignatureToken};

/// Enum representing the result of running a native function
pub enum NativeReturnStatus {
    /// Represents a successful execution.
    Success {
        /// The cost for running that function
        cost: u64,
        /// The `Vec<Local>` values will be pushed on the stack
        return_values: Vec<Local>,
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
    pub dispatch: fn(VecDeque<Local>) -> NativeReturnStatus,
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
    function_name: &str,
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
            kind_constraints: $kinds,
        };
        let f = NativeFunction {
            dispatch: $dis,
            expected_signature,
        };
        let addr = AccountAddress::from_hex_literal($addr).unwrap();
        let id = ModuleId::new(addr, $module.into());
        let old = $m
            .entry(id)
            .or_insert_with(HashMap::new)
            .insert($name.into(), f);
        assert!(old.is_none());
    }};
}

/// Helper for finding expected struct handle index
fn tstruct(
    addr_str: &str,
    module_name: &str,
    function_name: &str,
    args: Vec<SignatureToken>,
) -> SignatureToken {
    let addr = AccountAddress::from_hex_literal(addr_str).unwrap();
    let id = ModuleId::new(addr, module_name.into());
    let native_struct = dispatch_native_struct(&id, function_name).unwrap();
    let idx = native_struct.expected_index;
    // TODO assert kinds match
    assert!(args.len() == native_struct.expected_type_parameters.len());
    SignatureToken::Struct(idx, args)
}

type NativeFunctionMap = HashMap<ModuleId, HashMap<String, NativeFunction>>;

lazy_static! {
    static ref NATIVE_FUNCTION_MAP: NativeFunctionMap = {
        use SignatureToken::*;
        let mut m: NativeFunctionMap = HashMap::new();
        // Hash
        add!(m, "0x0", "Hash", "keccak256",
            hash::native_keccak_256,
            vec![ByteArray],
            vec![ByteArray]
        );
        add!(m, "0x0", "Hash", "ripemd160",
            hash::native_ripemd_160,
            vec![ByteArray],
            vec![ByteArray]
        );
        add!(m, "0x0", "Hash", "sha2_256",
            hash::native_sha2_256,
            vec![ByteArray],
            vec![ByteArray]
        );
        add!(m, "0x0", "Hash", "sha3_256",
            hash::native_sha3_256,
            vec![ByteArray],
            vec![ByteArray]
        );
        // Signature
        add!(m, "0x0", "Signature", "ed25519_verify",
            signature::native_ed25519_signature_verification,
            vec![ByteArray, ByteArray, ByteArray],
            vec![Bool]
        );
        add!(m, "0x0", "Signature", "ed25519_threshold_verify",
            signature::native_ed25519_threshold_signature_verification,
            vec![ByteArray, ByteArray, ByteArray, ByteArray],
            vec![U64]
        );
        // AddressUtil
        add!(m, "0x0", "AddressUtil", "address_to_bytes",
            primitive_helpers::native_address_to_bytes,
            vec![Address],
            vec![ByteArray]
        );
        // U64Util
        add!(m, "0x0", "U64Util", "u64_to_bytes",
            primitive_helpers::native_u64_to_bytes,
            vec![U64],
            vec![ByteArray]
        );
        // BytearrayUtil
        add!(m, "0x0", "BytearrayUtil", "bytearray_concat",
            primitive_helpers::native_bytearray_concat,
            vec![ByteArray, ByteArray],
            vec![ByteArray]
        );
        // Vector
        add!(m, "0x0", "Vector", "length",
            vector::native_length,
            vec![Reference(Box::new(tstruct("0x0", "Vector", "T", vec![])))],
            vec![U64]
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
