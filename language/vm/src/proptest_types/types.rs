// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    file_format::{
        AbilitySet, FieldDefinition, IdentifierIndex, ModuleHandleIndex, SignatureToken,
        StructDefinition, StructFieldInformation, StructHandle, StructHandleIndex, TableIndex,
        TypeSignature,
    },
    proptest_types::signature::{AbilitySetGen, SignatureTokenGen},
};
use proptest::{
    collection::{vec, SizeRange},
    option,
    prelude::*,
    sample::Index as PropIndex,
    std_facade::hash_set::HashSet,
};
use std::{cmp::max, collections::BTreeSet};

#[derive(Debug)]
struct TypeSignatureIndex(u16);

#[derive(Debug)]
pub struct StDefnMaterializeState {
    pub identifiers_len: usize,
    pub struct_handles: Vec<StructHandle>,
    pub new_handles: BTreeSet<(ModuleHandleIndex, IdentifierIndex)>,
}

impl StDefnMaterializeState {
    pub fn new(identifiers_len: usize, struct_handles: Vec<StructHandle>) -> Self {
        Self {
            identifiers_len,
            struct_handles,
            new_handles: BTreeSet::new(),
        }
    }

    fn add_struct_handle(&mut self, handle: StructHandle) -> Option<StructHandleIndex> {
        if self.new_handles.insert((handle.module, handle.name)) {
            self.struct_handles.push(handle);
            Some(StructHandleIndex((self.struct_handles.len() - 1) as u16))
        } else {
            None
        }
    }

    fn potential_abilities(&self, ty: &SignatureToken) -> AbilitySet {
        use SignatureToken::*;

        match ty {
            Bool | U8 | U64 | U128 | Address => AbilitySet::PRIMITIVES,

            Reference(_) | MutableReference(_) => AbilitySet::REFERENCES,
            Signer => AbilitySet::SIGNER,
            TypeParameter(_) => AbilitySet::ALL,
            Vector(ty) => {
                let inner = self.potential_abilities(ty);
                inner.intersect(AbilitySet::VECTOR)
            }
            Struct(idx) => {
                let sh = &self.struct_handles[idx.0 as usize];
                sh.abilities
            }
            StructInstantiation(idx, type_args) => {
                let sh = &self.struct_handles[idx.0 as usize];

                // Gather the abilities of the type actuals.
                let type_args_abilities = type_args.iter().map(|ty| self.potential_abilities(ty));
                type_args_abilities.fold(sh.abilities, |acc, ty_arg_abilities| {
                    acc.intersect(ty_arg_abilities)
                })
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct StructHandleGen {
    module_idx: PropIndex,
    name_idx: PropIndex,
    abilities: AbilitySetGen,
    type_parameters: Vec<AbilitySetGen>,
}

impl StructHandleGen {
    pub fn strategy(ability_count: impl Into<SizeRange>) -> impl Strategy<Value = Self> {
        (
            any::<PropIndex>(),
            any::<PropIndex>(),
            AbilitySetGen::strategy(),
            vec(AbilitySetGen::strategy(), ability_count),
        )
            .prop_map(|(module_idx, name_idx, abilities, type_parameters)| Self {
                module_idx,
                name_idx,
                abilities,
                type_parameters,
            })
    }

    pub fn materialize(self, module_len: usize, identifiers_len: usize) -> StructHandle {
        let idx = max(self.module_idx.index(module_len) as TableIndex, 1);
        let mut type_parameters = vec![];
        for type_param in self.type_parameters {
            type_parameters.push(type_param.materialize());
        }
        StructHandle {
            module: ModuleHandleIndex(idx as TableIndex),
            name: IdentifierIndex(self.name_idx.index(identifiers_len) as TableIndex),
            abilities: self.abilities.materialize(),
            type_parameters,
        }
    }
}

#[derive(Clone, Debug)]
pub struct StructDefinitionGen {
    name_idx: PropIndex,
    abilities: AbilitySetGen,
    type_parameters: Vec<AbilitySetGen>,
    is_public: bool,
    field_defs: Option<Vec<FieldDefinitionGen>>,
}

impl StructDefinitionGen {
    pub fn strategy(
        field_count: impl Into<SizeRange>,
        type_parameter_count: impl Into<SizeRange>,
    ) -> impl Strategy<Value = Self> {
        (
            any::<PropIndex>(),
            AbilitySetGen::strategy(),
            vec(AbilitySetGen::strategy(), type_parameter_count),
            any::<bool>(),
            option::of(vec(FieldDefinitionGen::strategy(), field_count)),
        )
            .prop_map(
                |(name_idx, abilities, type_parameters, is_public, field_defs)| Self {
                    name_idx,
                    abilities,
                    type_parameters,
                    is_public,
                    field_defs,
                },
            )
    }

    pub fn materialize(
        self,
        state: &mut StDefnMaterializeState,
    ) -> (Option<StructDefinition>, usize) {
        let mut field_names = HashSet::new();
        let mut fields = vec![];
        match self.field_defs {
            None => (),
            Some(field_defs_gen) => {
                for fd_gen in field_defs_gen {
                    let field = fd_gen.materialize(state);
                    if field_names.insert(field.name) {
                        fields.push(field);
                    }
                }
            }
        };
        let abilities = fields
            .iter()
            .fold(self.abilities.materialize(), |acc, field| {
                acc.intersect(state.potential_abilities(&field.signature.0))
            });
        let handle = StructHandle {
            // 0 represents the current module
            module: ModuleHandleIndex(0),
            name: IdentifierIndex(self.name_idx.index(state.identifiers_len) as TableIndex),
            abilities,
            type_parameters: self
                .type_parameters
                .into_iter()
                .map(|abilities| abilities.materialize())
                .collect(),
        };
        match state.add_struct_handle(handle) {
            Some(struct_handle) => {
                if fields.is_empty() {
                    (
                        Some(StructDefinition {
                            struct_handle,
                            field_information: StructFieldInformation::Native,
                        }),
                        0,
                    )
                } else {
                    let field_count = fields.len();
                    let field_information = StructFieldInformation::Declared(fields);
                    (
                        Some(StructDefinition {
                            struct_handle,
                            field_information,
                        }),
                        field_count,
                    )
                }
            }
            None => (None, 0),
        }
    }
}

#[derive(Clone, Debug)]
struct FieldDefinitionGen {
    name_idx: PropIndex,
    signature_gen: SignatureTokenGen,
}

impl FieldDefinitionGen {
    fn strategy() -> impl Strategy<Value = Self> {
        (any::<PropIndex>(), SignatureTokenGen::atom_strategy()).prop_map(
            |(name_idx, signature_gen)| Self {
                name_idx,
                signature_gen,
            },
        )
    }

    fn materialize(self, state: &StDefnMaterializeState) -> FieldDefinition {
        FieldDefinition {
            name: IdentifierIndex(self.name_idx.index(state.identifiers_len) as TableIndex),
            signature: TypeSignature(self.signature_gen.materialize(&state.struct_handles)),
        }
    }
}
