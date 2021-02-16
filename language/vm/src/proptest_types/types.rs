// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    file_format::{
        FieldDefinition, IdentifierIndex, ModuleHandleIndex, SignatureToken, StructDefinition,
        StructFieldInformation, StructHandle, StructHandleIndex, TableIndex, TypeSignature,
    },
    proptest_types::signature::{KindGen, SignatureTokenGen},
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

    fn contains_nominal_resource(&self, signature: &SignatureToken) -> bool {
        use SignatureToken::*;

        match signature {
            Signer => true,
            Struct(struct_handle_index) => {
                self.struct_handles[struct_handle_index.0 as usize].is_nominal_resource
            }
            StructInstantiation(struct_handle_index, type_args) => {
                self.struct_handles[struct_handle_index.0 as usize].is_nominal_resource
                    || type_args.iter().any(|t| self.contains_nominal_resource(t))
            }
            Vector(targ) => self.contains_nominal_resource(targ),
            Reference(token) | MutableReference(token) => self.contains_nominal_resource(token),
            Bool | U8 | U64 | U128 | Address | TypeParameter(_) => false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct StructHandleGen {
    module_idx: PropIndex,
    name_idx: PropIndex,
    is_nominal_resource: bool,
    type_parameters: Vec<KindGen>,
}

impl StructHandleGen {
    pub fn strategy(kind_count: impl Into<SizeRange>) -> impl Strategy<Value = Self> {
        (
            any::<PropIndex>(),
            any::<PropIndex>(),
            any::<bool>(),
            vec(KindGen::strategy(), kind_count),
        )
            .prop_map(
                |(module_idx, name_idx, is_nominal_resource, type_parameters)| Self {
                    module_idx,
                    name_idx,
                    is_nominal_resource,
                    type_parameters,
                },
            )
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
            is_nominal_resource: self.is_nominal_resource,
            type_parameters,
        }
    }
}

#[derive(Clone, Debug)]
pub struct StructDefinitionGen {
    name_idx: PropIndex,
    is_nominal_resource: bool,
    type_parameters: Vec<KindGen>,
    is_public: bool,
    field_defs: Option<Vec<FieldDefinitionGen>>,
}

impl StructDefinitionGen {
    pub fn strategy(
        field_count: impl Into<SizeRange>,
        kind_count: impl Into<SizeRange>,
    ) -> impl Strategy<Value = Self> {
        (
            any::<PropIndex>(),
            any::<bool>(),
            vec(KindGen::strategy(), kind_count),
            any::<bool>(),
            option::of(vec(FieldDefinitionGen::strategy(), field_count)),
        )
            .prop_map(
                |(name_idx, is_nominal_resource, type_parameters, is_public, field_defs)| Self {
                    name_idx,
                    is_nominal_resource,
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
        let is_nominal_resource = if fields.is_empty() {
            self.is_nominal_resource
        } else {
            self.is_nominal_resource
                || fields.iter().any(|field| {
                    let field_sig = &field.signature.0;
                    state.contains_nominal_resource(field_sig)
                })
        };
        let handle = StructHandle {
            // 0 represents the current module
            module: ModuleHandleIndex(0),
            name: IdentifierIndex(self.name_idx.index(state.identifiers_len) as TableIndex),
            is_nominal_resource,
            type_parameters: self
                .type_parameters
                .into_iter()
                .map(|kind| kind.materialize())
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
