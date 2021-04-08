// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::file_format::{Constant, SignatureToken};
use move_core_types::account_address::AccountAddress;
use proptest::{
    arbitrary::any,
    collection::{btree_set, vec, SizeRange},
    strategy::Strategy,
};

// A `ConstantPoolGen` gives you back a `Strategy` that makes a constant pool with
// `Address` and `Vector<U8>` only.
// TODO: make it more general and plug in a constant API
#[derive(Clone, Debug)]
pub struct ConstantPoolGen {
    addresses: Vec<AccountAddress>,
    byte_arrays: Vec<Vec<u8>>,
}

impl ConstantPoolGen {
    // Return a `Strategy` that builds a ConstantPool with a number of `addresse`s and
    // `vector<u8>` respectively driven by `address_count` and `byte_array_count`
    pub fn strategy(
        address_count: impl Into<SizeRange>,
        byte_array_count: impl Into<SizeRange>,
    ) -> impl Strategy<Value = Self> {
        // get unique sets of addresses and vector<U8> (empty vector allowed)
        (
            btree_set(any::<AccountAddress>(), address_count),
            btree_set(vec(any::<u8>(), 0..=20), byte_array_count),
        )
            .prop_map(|(addresses, byte_arrays)| Self {
                addresses: addresses.into_iter().collect(),
                byte_arrays: byte_arrays.into_iter().collect(),
            })
    }

    // Return the `ConstantPool`
    pub fn constant_pool(self) -> Vec<Constant> {
        let mut constants = vec![];
        for address in self.addresses {
            constants.push(Constant {
                type_: SignatureToken::Address,
                data: address.to_vec(),
            });
        }
        for mut byte_array in self.byte_arrays {
            // TODO: below is a trick to make serialization easy (size being one byte)
            // This can hopefully help soon in defining an API for constants
            // As a restriction is a non issue as we don't have input over big numbers (>100)
            if byte_array.len() > 127 {
                continue;
            }
            byte_array.push(byte_array.len() as u8);
            byte_array.reverse();
            constants.push(Constant {
                type_: SignatureToken::Vector(Box::new(SignatureToken::U8)),
                data: byte_array,
            });
        }
        constants
    }
}
