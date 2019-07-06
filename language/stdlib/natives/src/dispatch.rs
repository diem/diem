// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{hash, primitive_helpers, signature};
pub use failure::Error;
use failure::*;
use types::{account_address::AccountAddress, byte_array::ByteArray};

pub type Result<T> = ::std::result::Result<T, Error>;

pub enum NativeReturnType {
    ByteArray(ByteArray),
    Bool(bool),
    UInt64(u64),
    Void,
}

pub struct CostedReturnType {
    cost: u64,
    return_value: NativeReturnType,
}

impl CostedReturnType {
    pub fn new(cost: u64, return_value: NativeReturnType) -> Self {
        CostedReturnType { cost, return_value }
    }

    pub fn cost(&self) -> u64 {
        self.cost
    }

    pub fn get_return_value(self) -> NativeReturnType {
        self.return_value
    }
}

pub trait StackAccessor {
    fn get_byte_array(&mut self) -> Result<ByteArray>;
    fn get_u64(&mut self) -> Result<u64>;
    fn get_address(&mut self) -> Result<AccountAddress>;
}

pub fn dispatch_native_call<T: StackAccessor>(
    accessor: T,
    module_name: &str,
    function_name: &str,
) -> Result<CostedReturnType> {
    match module_name {
        "Hash" => match function_name {
            "keccak256" => hash::native_keccak_256(accessor),
            "ripemd160" => hash::native_ripemd_160(accessor),
            "sha2_256" => hash::native_sha2_256(accessor),
            "sha3_256" => hash::native_sha3_256(accessor),
            &_ => bail!(
                "Unknown native function `{}.{}'",
                module_name,
                function_name
            ),
        },
        "Signature" => match function_name {
            "ed25519_verify" => signature::native_ed25519_signature_verification(accessor),
            &_ => bail!(
                "Unknown native function `{}.{}'",
                module_name,
                function_name
            ),
        },
        "AddressUtil" => match function_name {
            "address_to_bytes" => primitive_helpers::native_address_to_bytes(accessor),
            &_ => bail!(
                "Unknown native function `{}.{}'",
                module_name,
                function_name
            ),
        },
        "U64Util" => match function_name {
            "u64_to_bytes" => primitive_helpers::native_u64_to_bytes(accessor),
            &_ => bail!(
                "Unknown native function `{}.{}'",
                module_name,
                function_name
            ),
        },
        "BytearrayUtil" => match function_name {
            "bytearray_concat" => primitive_helpers::native_bytearray_concat(accessor),
            &_ => bail!(
                "Unknown native function `{}.{}'",
                module_name,
                function_name
            ),
        },
        &_ => bail!("Unknown native module {}", module_name),
    }
}
