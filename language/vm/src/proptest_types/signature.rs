// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::file_format::{
    Kind, Signature, SignatureToken, StructHandle, StructHandleIndex, TableIndex,
    TypeParameterIndex,
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
    All,
}

impl KindGen {
    pub fn strategy() -> impl Strategy<Value = Self> {
        use KindGen::*;

        static KINDS: &[KindGen] = &[Resource, Copyable, All];

        select(KINDS)
    }

    pub fn materialize(self) -> Kind {
        match self {
            KindGen::Resource => Kind::Resource,
            KindGen::Copyable => Kind::Copyable,
            KindGen::All => Kind::All,
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

    pub fn materialize(self, struct_handles: &[StructHandle]) -> Signature {
        Signature(
            self.signatures
                .into_iter()
                .map(move |token| token.materialize(struct_handles))
                .collect(),
        )
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
    Signer,
    TypeParameter(PropIndex),

    // Composite signature tokens.
    Struct(PropIndex),
    Vector(Box<SignatureTokenGen>),
    Reference(Box<SignatureTokenGen>),
    MutableReference(Box<SignatureTokenGen>),
}

impl SignatureTokenGen {
    pub fn strategy() -> impl Strategy<Value = Self> {
        prop::strategy::Union::new_weighted(vec![
            (5, Self::atom_strategy().boxed()),
            (1, Self::reference_strategy().boxed()),
            (1, Self::mutable_reference_strategy().boxed()),
            (1, Self::vector_strategy().boxed()),
        ])
    }

    /// Generates a signature token for an owned (non-reference) type.
    pub fn owned_strategy() -> impl Strategy<Value = Self> {
        prop::strategy::Union::new_weighted(vec![(3, Self::atom_strategy().boxed())])
    }

    pub fn atom_strategy() -> impl Strategy<Value = Self> {
        prop_oneof![
            9 => Self::owned_non_struct_strategy(),
            1 => Self::struct_strategy(),
            // 1=> Self::type_parameter_strategy(),
        ]
    }

    /// Generates a signature token for a non-struct owned type.
    pub fn owned_non_struct_strategy() -> impl Strategy<Value = Self> {
        use SignatureTokenGen::*;

        static OWNED_NON_STRUCTS: &[SignatureTokenGen] = &[Bool, U8, U64, U128, Address, Signer];

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

    pub fn vector_strategy() -> impl Strategy<Value = Self> {
        Self::owned_strategy().prop_map(|atom| SignatureTokenGen::Vector(Box::new(atom)))
    }

    pub fn reference_strategy() -> impl Strategy<Value = Self> {
        // References to references are not supported.
        Self::owned_strategy().prop_map(|atom| SignatureTokenGen::Reference(Box::new(atom)))
    }

    pub fn mutable_reference_strategy() -> impl Strategy<Value = Self> {
        // References to references are not supported.
        Self::owned_strategy().prop_map(|atom| SignatureTokenGen::MutableReference(Box::new(atom)))
    }

    pub fn materialize(self, struct_handles: &[StructHandle]) -> SignatureToken {
        use SignatureTokenGen::*;
        match self {
            Bool => SignatureToken::Bool,
            U8 => SignatureToken::U8,
            U64 => SignatureToken::U64,
            U128 => SignatureToken::U128,
            Address => SignatureToken::Address,
            Signer => SignatureToken::Signer,
            Struct(idx) => {
                let struct_handles_len = struct_handles.len();
                if struct_handles_len == 0 {
                    // we are asked to create a type of a struct that cannot exist
                    // so we fake a U64 instead...
                    SignatureToken::U64
                } else {
                    let struct_idx = idx.index(struct_handles_len);
                    let sh = &struct_handles[struct_idx];
                    if sh.type_parameters.is_empty() {
                        SignatureToken::Struct(StructHandleIndex(struct_idx as TableIndex))
                    } else {
                        let mut type_params = vec![];
                        for kind in &sh.type_parameters {
                            match kind {
                                Kind::Copyable | Kind::All => type_params.push(SignatureToken::U64),
                                Kind::Resource => type_params.push(SignatureToken::Signer),
                            }
                        }
                        SignatureToken::StructInstantiation(
                            StructHandleIndex(struct_idx as TableIndex),
                            type_params,
                        )
                    }
                }
            }
            Vector(token) => SignatureToken::Vector(Box::new(token.materialize(struct_handles))),
            Reference(token) => {
                SignatureToken::Reference(Box::new(token.materialize(struct_handles)))
            }
            MutableReference(token) => {
                SignatureToken::MutableReference(Box::new(token.materialize(struct_handles)))
            }
            TypeParameter(idx) => {
                SignatureToken::TypeParameter(idx.index(struct_handles.len()) as TypeParameterIndex)
            }
        }
    }
}
