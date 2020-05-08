// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::{StructTag, TypeTag},
};
use proptest::{collection::vec, prelude::*};
impl Arbitrary for TypeTag {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        use TypeTag::*;
        let leaf = prop_oneof![
            Just(Bool),
            Just(U8),
            Just(U64),
            Just(U128),
            Just(Address),
            Just(Vector(Box::new(Bool))),
        ];
        leaf.prop_recursive(
            8,  // levels deep
            16, // max size
            4,  // max number of items per collection
            |inner| {
                (
                    any::<AccountAddress>(),
                    any::<Identifier>(),
                    any::<Identifier>(),
                    vec(inner, 0..4),
                )
                    .prop_map(|(address, module, name, type_params)| {
                        Struct(StructTag {
                            address,
                            module,
                            name,
                            type_params,
                        })
                    })
            },
        )
        .boxed()
    }
}
