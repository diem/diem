// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Utilities for property-based testing.

use crate::file_format::{
    AddressPoolIndex, CompiledModule, CompiledModuleMut, FieldDefinition, FieldDefinitionIndex,
    FunctionHandle, FunctionSignatureIndex, IdentifierIndex, Kind, LocalsSignature, MemberCount,
    ModuleHandle, ModuleHandleIndex, SignatureToken, StructDefinition, StructFieldInformation,
    StructHandle, StructHandleIndex, TableIndex, TypeSignature, TypeSignatureIndex,
};
use libra_proptest_helpers::GrowingSubset;
use libra_types::account_address::AccountAddress;
use move_core_types::identifier::Identifier;
use proptest::{
    collection::{vec, SizeRange},
    option,
    prelude::*,
    sample::Index as PropIndex,
};

mod functions;
mod signature;

use functions::{FnDefnMaterializeState, FunctionDefinitionGen};
use signature::{FunctionSignatureGen, KindGen, SignatureTokenGen};

/// Represents how large [`CompiledModule`] tables can be.
pub type TableSize = u16;

impl CompiledModule {
    /// Convenience wrapper around [`CompiledModuleStrategyGen`][CompiledModuleStrategyGen] that
    /// generates valid modules with the given size.
    pub fn valid_strategy(size: usize) -> impl Strategy<Value = Self> {
        CompiledModuleStrategyGen::new(size as TableSize).generate()
    }
}

/// Contains configuration to generate [`CompiledModule`] instances.
///
/// If you don't care about customizing these parameters, see [`CompiledModule::valid_strategy`].
///
/// A `CompiledModule` can be looked at as a graph, with several kinds of nodes, and a nest of
/// pointers among those nodes. This graph has some properties:
///
/// 1. The graph has cycles. Generating DAGs is often simpler, but is not an option in this case.
/// 2. The actual structure of the graph is well-defined in terms of the kinds of nodes and
///    pointers that exist.
///
/// TODO: the graph also has pointers *out* of it, via address references to other modules.
/// This doesn't need to be handled when viewing modules in isolation, but some verification passes
/// will need to look at the entire set of modules. The work to make generating such modules
/// possible remains to be done.
///
/// Intermediate types
/// ------------------
///
/// The pointers are represented as indexes into vectors of other kinds of nodes. One of the
/// bigger problems is that the number of types, functions etc isn't known upfront so it is
/// impossible to know what range to pick from for the index types (`ModuleHandleIndex`,
/// `StructHandleIndex`, etc). To deal with this, the code generates a bunch of intermediate
/// structures (sometimes tuples, other times more complicated structures with their own internal
/// constraints), with "holes" represented by [`Index`](proptest::sample::Index) instances. Once all
/// the lengths are known, there's a final "materialize" step at the end that "fills in" these
/// holes.
///
/// One alternative would have been to generate lengths up front, then create vectors of those
/// lengths. This would have worked fine for generation but would have made shrinking take much
/// longer, because the shrinker would be less aware of the overall structure of the problem and
/// would have ended up redoing a lot of work. The approach taken here does end up being more
/// verbose but should perform optimally.
///
/// See [`proptest` issue #130](https://github.com/AltSysrq/proptest/issues/130) for more discussion
/// about this.
#[derive(Clone, Debug)]
pub struct CompiledModuleStrategyGen {
    size: usize,
    /// Range of number of fields in a struct and number of arguments in a function to generate.
    /// The default value is 0..4.
    member_count: SizeRange,
    /// Length of code units (function definition). XXX the unit might change here.
    code_len: SizeRange,
}

impl CompiledModuleStrategyGen {
    /// Create a new configuration for randomly generating [`CompiledModule`] instances.
    pub fn new(size: TableSize) -> Self {
        Self {
            size: size as usize,
            member_count: (0..4).into(),
            code_len: (0..50).into(),
        }
    }

    /// Set a new range for the number of fields in a struct or the number of arguments in a
    /// function.
    #[inline]
    pub fn member_count(&mut self, count: impl Into<SizeRange>) -> &mut Self {
        self.member_count = count.into();
        self
    }

    /// Create a `proptest` strategy for `CompiledModule` instances using this configuration.
    pub fn generate(self) -> impl Strategy<Value = CompiledModule> {
        // Base data -- everything points to this eventually.
        let address_pool_strat = vec(any::<AccountAddress>(), 1..=self.size);
        // This ensures that there are no empty ByteArrays
        // TODO: Should we enable empty ByteArrays in Move, e.g. let byte_array = b"";
        let byte_array_pool_strat = vec(vec(any::<u8>(), 0..=self.size), 1..=self.size);
        let identifiers_strat = vec(any::<Identifier>(), 1..=self.size);
        let type_signatures_strat = vec(SignatureTokenGen::strategy(), 1..=self.size);
        // Ensure at least one owned non-struct type signature.
        let owned_non_struct_strat = vec(
            SignatureTokenGen::owned_non_struct_strategy(),
            1..=self.size,
        );
        let owned_type_sigs_strat = vec(SignatureTokenGen::owned_strategy(), 1..=self.size);
        let function_signatures_strat = vec(
            FunctionSignatureGen::strategy(
                self.member_count.clone(),
                self.member_count.clone(),
                self.member_count.clone(),
            ),
            1..=self.size,
        );

        // The number of PropIndex instances in each tuple represents the number of pointers out
        // from an instance of that particular kind of node.
        let module_handles_strat = vec(any::<(PropIndex, PropIndex)>(), 1..=self.size);
        let struct_handles_strat = vec(
            any::<(PropIndex, PropIndex, bool, Vec<Kind>)>(),
            1..=self.size,
        );
        let function_handles_strat = vec(any::<(PropIndex, PropIndex, PropIndex)>(), 1..=self.size);
        let struct_defs_strat = vec(
            StructDefinitionGen::strategy(self.member_count.clone()),
            1..=self.size,
        );
        let function_defs_strat = vec(
            FunctionDefinitionGen::strategy(
                self.member_count.clone(),
                self.member_count.clone(),
                self.member_count.clone(),
                self.member_count.clone(),
                self.code_len,
            ),
            1..=self.size,
        );
        // Note that prop_test only allows a tuple of length up to ten
        // therefore, we need to treat the last two items as a pair to
        // ensure we have less than 10 elements in the tuple.
        (
            address_pool_strat,
            byte_array_pool_strat,
            identifiers_strat,
            type_signatures_strat,
            owned_non_struct_strat,
            owned_type_sigs_strat,
            function_signatures_strat,
            (
                module_handles_strat,
                struct_handles_strat,
                function_handles_strat,
            ),
            (struct_defs_strat, function_defs_strat),
        )
            .prop_map(
                |(
                    address_pool,
                    byte_array_pool,
                    identifiers,
                    type_signatures,
                    owned_non_structs,
                    owned_type_sigs,
                    function_signatures,
                    (module_handles, struct_handles, function_handles),
                    (struct_defs, function_defs),
                )| {
                    let address_pool_len = address_pool.len();
                    let identifiers_len = identifiers.len();
                    let byte_array_pool_len = byte_array_pool.len();
                    let module_handles_len = module_handles.len();
                    // StDefnMaterializeState adds one new handle for each definition, so the total
                    // number of struct handles is the sum of the number of generated struct
                    // handles (i.e. representing structs in external modules) and the number of
                    // internal ones.
                    let struct_handles_len = struct_handles.len() + struct_defs.len();
                    // XXX FnDefnMaterializeState below adds more function signatures. This line
                    // means that no signatures generated later will be used by handles generated
                    // earlier.
                    //
                    // Instead, one could use function_signatures.len() + function_defs.len() to
                    // use signatures from later.
                    let function_signatures_len = function_signatures.len();
                    // FnDefnMaterializeState below adds function handles equal to the number of
                    // function definitions.
                    let function_handles_len = function_handles.len() + function_defs.len();

                    let owned_type_sigs: Vec<_> =
                        SignatureTokenGen::map_materialize(owned_non_structs, struct_handles_len)
                            .chain(SignatureTokenGen::map_materialize(
                                owned_type_sigs,
                                struct_handles_len,
                            ))
                            .map(TypeSignature)
                            .collect();
                    let owned_type_indexes = type_indexes(&owned_type_sigs);

                    // Put the owned type signatures first so they're in the range
                    // 0..owned_type_sigs.len(). These are the signatures that will be used to pick
                    // field definition sigs from.
                    // Note that this doesn't result in a distribution that's spread out -- it
                    // would be nice to achieve that.
                    let type_signatures: Vec<_> = owned_type_sigs
                        .into_iter()
                        .chain(
                            SignatureTokenGen::map_materialize(type_signatures, struct_handles_len)
                                .map(TypeSignature),
                        )
                        .collect();
                    let function_signatures = function_signatures
                        .into_iter()
                        .map(|sig| sig.materialize(struct_handles_len))
                        .collect();

                    let module_handles: Vec<_> = module_handles
                        .into_iter()
                        .map(|(address_idx, name_idx)| ModuleHandle {
                            address: AddressPoolIndex::new(
                                address_idx.index(address_pool_len) as TableIndex
                            ),
                            name: IdentifierIndex::new(
                                name_idx.index(identifiers_len) as TableIndex
                            ),
                        })
                        .collect();

                    let struct_handles: Vec<_> = struct_handles
                        .into_iter()
                        .map(
                            |(module_idx, name_idx, is_nominal_resource, _type_formals)| {
                                StructHandle {
                                    module: ModuleHandleIndex::new(
                                        module_idx.index(module_handles_len) as TableIndex,
                                    ),
                                    name: IdentifierIndex::new(
                                        name_idx.index(identifiers_len) as TableIndex
                                    ),
                                    is_nominal_resource,
                                    // TODO: re-enable type formals gen when we rework prop tests
                                    // for generics
                                    type_formals: vec![],
                                }
                            },
                        )
                        .collect();

                    let function_handles: Vec<_> = function_handles
                        .into_iter()
                        .map(|(module_idx, name_idx, signature_idx)| FunctionHandle {
                            module: ModuleHandleIndex::new(
                                module_idx.index(module_handles_len) as TableIndex
                            ),
                            name: IdentifierIndex::new(
                                name_idx.index(identifiers_len) as TableIndex
                            ),
                            signature: FunctionSignatureIndex::new(
                                signature_idx.index(function_signatures_len) as TableIndex,
                            ),
                        })
                        .collect();

                    // Struct definitions also generate field definitions.
                    let mut state = StDefnMaterializeState {
                        identifiers_len,
                        owned_type_indexes,
                        struct_handles,
                        type_signatures,
                        // field_defs will be filled out by StructDefinitionGen::materialize
                        field_defs: vec![],
                    };
                    let struct_defs: Vec<_> = struct_defs
                        .into_iter()
                        .map(|def| def.materialize(&mut state))
                        .collect();

                    let StDefnMaterializeState {
                        struct_handles,
                        type_signatures,
                        field_defs,
                        ..
                    } = state;
                    assert_eq!(struct_handles_len, struct_handles.len());

                    // Definitions get generated at the end. But some of the other pools need to be
                    // involved here, so temporarily give up ownership to the state accumulators.
                    let mut state = FnDefnMaterializeState {
                        struct_handles_len,
                        address_pool_len,
                        identifiers_len,
                        byte_array_pool_len,
                        function_handles_len,
                        type_signatures_len: type_signatures.len(),
                        field_defs_len: field_defs.len(),
                        struct_defs_len: struct_defs.len(),
                        function_defs_len: function_defs.len(),
                        function_signatures,
                        // locals will be filled out by FunctionDefinitionGen::materialize
                        locals_signatures: vec![LocalsSignature(vec![])],
                        function_handles,
                    };

                    let function_defs = function_defs
                        .into_iter()
                        .map(|def| def.materialize(&mut state))
                        .collect();

                    let FnDefnMaterializeState {
                        function_signatures,
                        locals_signatures,
                        function_handles,
                        ..
                    } = state;
                    assert_eq!(function_handles_len, function_handles.len());

                    // Put it all together.
                    CompiledModuleMut {
                        module_handles,
                        struct_handles,
                        function_handles,

                        struct_defs,
                        field_defs,
                        function_defs,

                        type_signatures,
                        function_signatures,
                        locals_signatures,

                        identifiers,
                        byte_array_pool,
                        address_pool,
                    }
                    .freeze()
                    .expect("valid modules should satisfy the bounds checker")
                },
            )
    }
}

#[derive(Debug)]
struct StDefnMaterializeState {
    identifiers_len: usize,
    // Struct definitions need to be nonrecursive -- this is ensured by only picking signatures
    // that either have no struct handle (represented as None), or have a handle less than the
    // one for the definition currently being added.
    owned_type_indexes: GrowingSubset<Option<StructHandleIndex>, TypeSignatureIndex>,
    // These get mutated by StructDefinitionGen.
    struct_handles: Vec<StructHandle>,
    field_defs: Vec<FieldDefinition>,
    type_signatures: Vec<TypeSignature>,
}

impl StDefnMaterializeState {
    fn next_struct_handle(&self) -> StructHandleIndex {
        StructHandleIndex::new(self.struct_handles.len() as TableIndex)
    }

    fn add_struct_handle(&mut self, handle: StructHandle) -> StructHandleIndex {
        self.struct_handles.push(handle);
        StructHandleIndex::new((self.struct_handles.len() - 1) as TableIndex)
    }

    /// Adds field defs to the pool. Returns the number of fields added and the index of the first
    /// field.
    fn add_field_defs(
        &mut self,
        new_defs: impl IntoIterator<Item = FieldDefinition>,
    ) -> (MemberCount, FieldDefinitionIndex) {
        let old_len = self.field_defs.len();
        self.field_defs.extend(new_defs);
        let new_len = self.field_defs.len();
        (
            (new_len - old_len) as MemberCount,
            FieldDefinitionIndex::new(old_len as TableIndex),
        )
    }

    fn contains_nominal_resource(&self, signature: &SignatureToken) -> bool {
        use SignatureToken::*;

        match signature {
            Struct(struct_handle_index, targs) => {
                self.struct_handles[struct_handle_index.0 as usize].is_nominal_resource
                    || targs.iter().any(|t| self.contains_nominal_resource(t))
            }
            Vector(targ) => self.contains_nominal_resource(targ),
            Reference(token) | MutableReference(token) => self.contains_nominal_resource(token),
            Bool | U8 | U64 | U128 | Address | TypeParameter(_) => false,
        }
    }
}

#[derive(Clone, Debug)]
struct StructDefinitionGen {
    name_idx: PropIndex,
    // the is_nominal_resource field of generated struct handle is set to true if
    // either any of the fields contains a resource or self.is_nominal_resource is true
    is_nominal_resource: bool,
    type_formals: Vec<KindGen>,
    is_public: bool,
    field_defs: Option<Vec<FieldDefinitionGen>>,
}

impl StructDefinitionGen {
    fn strategy(member_count: impl Into<SizeRange>) -> impl Strategy<Value = Self> {
        (
            any::<PropIndex>(),
            any::<bool>(),
            // TODO: how to not hard-code the number?
            vec(KindGen::strategy(), 0..10),
            any::<bool>(),
            // XXX 0..4 is the default member_count in CompiledModule -- is 0 (structs without
            // fields) possible?
            option::of(vec(FieldDefinitionGen::strategy(), member_count)),
        )
            .prop_map(
                |(name_idx, is_nominal_resource, _type_formals, is_public, field_defs)| Self {
                    name_idx,
                    is_nominal_resource,
                    // TODO: re-enable type formals gen once we rework prop tests for generics
                    type_formals: vec![],
                    is_public,
                    field_defs,
                },
            )
    }

    fn materialize(self, state: &mut StDefnMaterializeState) -> StructDefinition {
        let sh_idx = state.next_struct_handle();
        state.owned_type_indexes.advance_to(&Some(sh_idx));
        let struct_handle = sh_idx;

        match self.field_defs {
            None => {
                let is_nominal_resource = self.is_nominal_resource;
                let handle = StructHandle {
                    // 0 represents the current module
                    module: ModuleHandleIndex::new(0),
                    name: IdentifierIndex::new(
                        self.name_idx.index(state.identifiers_len) as TableIndex
                    ),
                    is_nominal_resource,
                    type_formals: self
                        .type_formals
                        .into_iter()
                        .map(|kind| kind.materialize())
                        .collect(),
                };
                state.add_struct_handle(handle);
                let field_information = StructFieldInformation::Native;
                StructDefinition {
                    struct_handle,
                    field_information,
                }
            }
            Some(field_defs_gen) => {
                // Each struct defines one or more fields. The collect() is to work around the
                // borrow checker -- it's annoying.
                let field_defs: Vec<_> = field_defs_gen
                    .into_iter()
                    .map(|field| field.materialize(sh_idx, state))
                    .collect();
                let is_nominal_resource = self.is_nominal_resource
                    || field_defs.iter().any(|field| {
                        let field_sig = &state.type_signatures[field.signature.0 as usize].0;
                        state.contains_nominal_resource(field_sig)
                    });
                let (field_count, fields) = state.add_field_defs(field_defs);

                let handle = StructHandle {
                    // 0 represents the current module
                    module: ModuleHandleIndex::new(0),
                    name: IdentifierIndex::new(
                        self.name_idx.index(state.identifiers_len) as TableIndex
                    ),
                    is_nominal_resource,
                    type_formals: self
                        .type_formals
                        .into_iter()
                        .map(|kind| kind.materialize())
                        .collect(),
                };
                state.add_struct_handle(handle);
                let field_information = StructFieldInformation::Declared {
                    field_count,
                    fields,
                };
                StructDefinition {
                    struct_handle,
                    field_information,
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
struct FieldDefinitionGen {
    name_idx: PropIndex,
    signature_idx: PropIndex,
    // XXX flags?
}

impl FieldDefinitionGen {
    fn strategy() -> impl Strategy<Value = Self> {
        (any::<PropIndex>(), any::<PropIndex>()).prop_map(|(name_idx, signature_idx)| Self {
            name_idx,
            signature_idx,
        })
    }

    fn materialize(
        self,
        sh_idx: StructHandleIndex,
        state: &StDefnMaterializeState,
    ) -> FieldDefinition {
        FieldDefinition {
            struct_: sh_idx,
            name: IdentifierIndex::new(self.name_idx.index(state.identifiers_len) as TableIndex),
            signature: *state.owned_type_indexes.pick_value(&self.signature_idx),
        }
    }
}

fn type_indexes<'a>(
    signatures: impl IntoIterator<Item = &'a TypeSignature>,
) -> GrowingSubset<Option<StructHandleIndex>, TypeSignatureIndex> {
    signatures
        .into_iter()
        .enumerate()
        .map(|(idx, signature)| {
            // Any signatures that don't have a struct handle in them can always be picked.
            // None is less than Some(0) so set those to None.
            (
                signature.0.struct_index(),
                TypeSignatureIndex::new(idx as TableIndex),
            )
        })
        .collect()
}
