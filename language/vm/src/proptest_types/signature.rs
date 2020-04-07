// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::file_format::{
    Kind, Signature, SignatureToken, StructHandleIndex, TableIndex, TypeParameterIndex,
};
use proptest::{
    collection::{vec, SizeRange},
    prelude::*,
    sample::{select, Index as PropIndex},
};

#[derive(Clone, Debug)]
pub enum KindGen {
    Resource,
    Copyable,
}

impl KindGen {
    pub fn strategy() -> impl Strategy<Value = Self> {
        use KindGen::*;

        static KINDS: &[KindGen] = &[Resource, Copyable];

        select(KINDS)
    }

    pub fn materialize(self) -> Kind {
        match self {
            KindGen::Resource => Kind::Resource,
            KindGen::Copyable => Kind::Copyable,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SignatureGen {
    signatures: Vec<SignatureTokenGen>,
}

impl SignatureGen {
    pub fn strategy(sig_count: impl Into<SizeRange>) -> impl Strategy<Value = Self> {
        vec(SignatureTokenGen::strategy(), sig_count).prop_map(|signatures| Self { signatures })
    }

    pub fn materialize(self, struct_handles_len: usize) -> Signature {
        Signature(SignatureTokenGen::map_materialize(self.signatures, struct_handles_len).collect())
    }
}

#[derive(Clone, Debug)]
pub enum SignatureTokenGen {
    // Atomic signature tokens.
    Bool,
    U8,
    U64,
    U128,
    Address,
    TypeParameter(PropIndex),

    // Composite signature tokens.
    Struct(PropIndex),
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
        prop_oneof![
            9 => Self::owned_non_struct_strategy(),
            // TODO: move struct_strategy out of atom strategy
            //       once features are implemented
            1 => Self::struct_strategy(),
            // TODO: for now, do not generate type parameters
            //       enable this once related features are implemented
            // 1=> Self::type_parameter_strategy(),
        ]
    }

    /// Generates a signature token for a non-struct owned type.
    pub fn owned_non_struct_strategy() -> impl Strategy<Value = Self> {
        use SignatureTokenGen::*;

        static OWNED_NON_STRUCTS: &[SignatureTokenGen] = &[Bool, U8, U64, U128, Address];

        select(OWNED_NON_STRUCTS)
    }

    // TODO: remove allow(dead_code) once related features are implemented
    #[allow(dead_code)]
    pub fn type_parameter_strategy() -> impl Strategy<Value = Self> {
        any::<PropIndex>().prop_map(SignatureTokenGen::TypeParameter)
    }

    pub fn struct_strategy() -> impl Strategy<Value = Self> {
        any::<PropIndex>().prop_map(SignatureTokenGen::Struct)
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
            U8 => SignatureToken::U8,
            U64 => SignatureToken::U64,
            U128 => SignatureToken::U128,
            Address => SignatureToken::Address,
            Struct(idx) => {
                if struct_handles_len == 0 {
                    // we are asked to create a type of a struct that cannot exist
                    // so we fake a U64 instead...
                    SignatureToken::U64
                } else {
                    SignatureToken::Struct(StructHandleIndex(
                        idx.index(struct_handles_len) as TableIndex
                    ))
                }
            }
            Reference(token) => {
                SignatureToken::Reference(Box::new(token.materialize(struct_handles_len)))
            }
            MutableReference(token) => {
                SignatureToken::MutableReference(Box::new(token.materialize(struct_handles_len)))
            }
            TypeParameter(idx) => {
                SignatureToken::TypeParameter(idx.index(struct_handles_len) as TypeParameterIndex)
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
