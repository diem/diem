// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::file_format::{FunctionSignature, SignatureToken, StructHandleIndex, TableIndex};
use proptest::{
    collection::{vec, SizeRange},
    prelude::*,
    sample::{select, Index as PropIndex},
};

#[derive(Clone, Debug)]
pub struct FunctionSignatureGen {
    return_types: Vec<SignatureTokenGen>,
    arg_types: Vec<SignatureTokenGen>,
}

impl FunctionSignatureGen {
    pub fn strategy(
        return_count: impl Into<SizeRange>,
        arg_count: impl Into<SizeRange>,
    ) -> impl Strategy<Value = Self> {
        (
            vec(SignatureTokenGen::strategy(), return_count),
            vec(SignatureTokenGen::strategy(), arg_count),
        )
            .prop_map(|(return_types, arg_types)| Self {
                return_types,
                arg_types,
            })
    }

    pub fn materialize(self, struct_handles_len: usize) -> FunctionSignature {
        FunctionSignature {
            return_types: SignatureTokenGen::map_materialize(self.return_types, struct_handles_len)
                .collect(),
            arg_types: SignatureTokenGen::map_materialize(self.arg_types, struct_handles_len)
                .collect(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum SignatureTokenGen {
    // Atomic signature tokens.
    Bool,
    Integer,
    String,
    ByteArray,
    Address,
    Struct(PropIndex),

    // Composite signature tokens.
    Reference(Box<SignatureTokenGen>),
    MutableReference(Box<SignatureTokenGen>),
}

impl SignatureTokenGen {
    pub fn strategy() -> impl Strategy<Value = Self> {
        prop::strategy::Union::new_weighted(vec![
            (5, Self::atom_strategy().boxed()),
            (1, Self::reference_strategy().boxed()),
            (1, Self::mutable_reference_strategy().boxed()),
        ])
    }

    /// Generates a signature token for an owned (non-reference) type.
    pub fn owned_strategy() -> impl Strategy<Value = Self> {
        prop::strategy::Union::new_weighted(vec![(3, Self::atom_strategy().boxed())])
    }

    pub fn atom_strategy() -> impl Strategy<Value = Self> {
        use SignatureTokenGen::*;

        prop_oneof![
            9 => Self::owned_non_struct_strategy(),
            1 => any::<PropIndex>().prop_map(Struct),
        ]
    }

    /// Generates a signature token for a non-struct owned type.
    pub fn owned_non_struct_strategy() -> impl Strategy<Value = Self> {
        use SignatureTokenGen::*;

        static OWNED_NON_STRUCTS: &[SignatureTokenGen] =
            &[Bool, Integer, String, ByteArray, Address];

        select(OWNED_NON_STRUCTS)
    }

    pub fn reference_strategy() -> impl Strategy<Value = Self> {
        // References to references are not supported.
        Self::owned_strategy().prop_map(|atom| SignatureTokenGen::Reference(Box::new(atom)))
    }

    pub fn mutable_reference_strategy() -> impl Strategy<Value = Self> {
        // References to references are not supported.
        Self::owned_strategy().prop_map(|atom| SignatureTokenGen::MutableReference(Box::new(atom)))
    }

    pub fn materialize(self, struct_handles_len: usize) -> SignatureToken {
        use SignatureTokenGen::*;

        match self {
            Bool => SignatureToken::Bool,
            Integer => SignatureToken::U64,
            String => SignatureToken::String,
            ByteArray => SignatureToken::ByteArray,
            Address => SignatureToken::Address,
            Struct(idx) => SignatureToken::Struct(StructHandleIndex::new(
                idx.index(struct_handles_len) as TableIndex,
            )),
            Reference(token) => {
                SignatureToken::Reference(Box::new(token.materialize(struct_handles_len)))
            }
            MutableReference(token) => {
                SignatureToken::MutableReference(Box::new(token.materialize(struct_handles_len)))
            }
        }
    }

    /// Convenience function to materialize many tokens.
    #[inline]
    pub fn map_materialize(
        tokens: impl IntoIterator<Item = Self>,
        struct_handles_len: usize,
    ) -> impl Iterator<Item = SignatureToken> {
        tokens
            .into_iter()
            .map(move |token| token.materialize(struct_handles_len))
    }
}
