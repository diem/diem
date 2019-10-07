// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    loaded_data::{struct_def::StructDef, types::Type},
    value::{Struct, Value},
};
use libra_types::{account_address::AccountAddress, byte_array::ByteArray};
use proptest::{collection::vec, prelude::*};
use vm::vm_string::VMString;

/// Strategies for property-based tests using `Value` instances.
impl Value {
    /// Returns a [`Strategy`] that generates random primitive (non-struct) `Value` instances.
    pub fn single_value_strategy() -> impl Strategy<Value = Self> {
        prop_oneof![
            any::<AccountAddress>().prop_map(Value::address),
            any::<u64>().prop_map(Value::u64),
            any::<bool>().prop_map(Value::bool),
            any::<VMString>().prop_map(Value::string),
            any::<ByteArray>().prop_map(Value::byte_array),
        ]
    }

    /// Returns a [`Strategy`] that generates arbitrary values, including `Struct`s.
    ///
    /// Arguments are used for recursion and define
    /// - depth of the nested `Struct`
    /// - approximate max number of `Value`s in the whole tree
    /// - number of max `Value`s at each level
    pub fn nested_strategy(
        depth: u32,
        desired_size: u32,
        expected_branch_size: u32,
    ) -> impl Strategy<Value = Self> {
        let leaf = Self::single_value_strategy();
        leaf.prop_recursive(depth, desired_size, expected_branch_size, |inner| {
            Self::struct_strategy_impl(inner)
        })
    }

    /// Returns a [`Strategy`] that generates random `Struct` instances.
    pub fn struct_strategy() -> impl Strategy<Value = Self> {
        Self::struct_strategy_impl(Self::nested_strategy(5, 100, 10))
    }

    fn struct_strategy_impl(base: impl Strategy<Value = Self>) -> impl Strategy<Value = Self> {
        vec(base, 0..10).prop_map(|values| Value::struct_(Struct::new(values)))
    }
}

impl Arbitrary for Value {
    type Parameters = ();
    fn arbitrary_with(_args: ()) -> Self::Strategy {
        Self::nested_strategy(3, 50, 10).boxed()
    }

    type Strategy = BoxedStrategy<Self>;
}

/// Strategies for Type
impl Type {
    /// Generate a random primitive Type, no Struct
    pub fn single_value_strategy() -> impl Strategy<Value = Self> {
        use Type::*;

        prop_oneof![
            Just(Bool),
            Just(U64),
            Just(String),
            Just(ByteArray),
            Just(Address),
        ]
    }

    /// Generate either a primitive Value or a Struct.
    pub fn nested_strategy(
        depth: u32,
        desired_size: u32,
        expected_branch_size: u32,
    ) -> impl Strategy<Value = Self> {
        use Type::*;

        let leaf = Self::single_value_strategy();
        leaf.prop_recursive(depth, desired_size, expected_branch_size, |inner| {
            prop_oneof![
                inner.clone().prop_map(|t| Reference(Box::new(t))),
                inner.clone().prop_map(|t| MutableReference(Box::new(t))),
                vec(inner, 0..10).prop_map(|defs| Struct(StructDef::new(defs))),
            ]
        })
    }
}

impl Arbitrary for Type {
    type Parameters = ();
    fn arbitrary_with(_args: ()) -> Self::Strategy {
        Self::nested_strategy(3, 20, 10).boxed()
    }

    type Strategy = BoxedStrategy<Self>;
}
