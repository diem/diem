// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
//! Loaded representation for runtime types.

use crate::loaded_data::struct_def::StructDef;
use serde::{Deserialize, Serialize};

/// Resolved form of runtime types.
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum Type {
    Bool,
    U8,
    U64,
    U128,
    ByteArray,
    Address,
    Struct(StructDef),
    Reference(Box<Type>),
    MutableReference(Box<Type>),
    TypeVariable(u16),
}

#[cfg(feature = "fuzzing")]
pub mod prop {
    use super::*;
    use crate::native_structs::NativeStructType;
    use proptest::{collection::vec, prelude::*};

    impl Type {
        /// Generate a random primitive Type, no Struct or Vector.
        pub fn single_value_strategy() -> impl Strategy<Value = Self> {
            use Type::*;

            prop_oneof![
                Just(Bool),
                Just(U8),
                Just(U64),
                Just(U128),
                Just(ByteArray),
                Just(Address),
            ]
        }

        /// Generate a primitive Value, a Struct or a Vector.
        pub fn nested_strategy(
            depth: u32,
            desired_size: u32,
            expected_branch_size: u32,
        ) -> impl Strategy<Value = Self> {
            use Type::*;

            let leaf = Self::single_value_strategy();
            leaf.prop_recursive(depth, desired_size, expected_branch_size, |inner| {
                prop_oneof![
                    inner.clone().prop_map(|layout| Struct(StructDef::Native(
                        NativeStructType::new_vec(layout)
                    ))),
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
}
