// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Utilities for property-based testing.

use crate::file_format::{
    AddressIdentifierIndex, CompiledModule, CompiledModuleMut, FieldDefinition, FunctionDefinition,
    FunctionHandle, IdentifierIndex, Kind, ModuleHandle, ModuleHandleIndex, SignatureToken,
    StructDefinition, StructFieldInformation, StructHandle, StructHandleIndex, TableIndex,
    TypeSignature,
};
use libra_types::account_address::AccountAddress;
use move_core_types::identifier::Identifier;
use proptest::{
    collection::{btree_set, vec, SizeRange},
    option,
    prelude::*,
    sample::Index as PropIndex,
};

mod functions;
mod signature;

use crate::proptest_types::functions::{FnHandleMaterializeState, FunctionHandleGen};
use functions::{FnDefnMaterializeState, FunctionDefinitionGen};
use signature::{KindGen, SignatureTokenGen};
use std::collections::{BTreeSet, HashMap, HashSet};

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
        //
        // leaf pool generator
        //
        let address_pool_strat = btree_set(any::<AccountAddress>(), 1..=self.size);
        let id_base_range = if self.size < 20 { 10 } else { 1 };
        let identifiers_strat = btree_set(
            any::<Identifier>(),
            id_base_range..=self.size + id_base_range,
        );

        // The number of PropIndex instances in each tuple represents the number of pointers out
        // from an instance of that particular kind of node.

        //
        // Module handle generator
        //
        let module_handles_strat = vec(any::<(PropIndex, PropIndex)>(), 1..=self.size);

        //
        // Struct generators
        //
        let struct_handles_strat = vec(
            any::<(PropIndex, PropIndex, bool, Vec<Kind>)>(),
            1..=self.size,
        );
        let struct_defs_strat = vec(
            StructDefinitionGen::strategy(self.member_count.clone()),
            1..=self.size,
        );

        //
        // Functions generators
        //

        // FunctionHandle will add to the Signature table
        // FunctionDefinition will also add the following pool:
        // FieldHandle, StructInstantiation, FunctionInstantiation, FieldInstantiation
        let function_handles_strat = vec(
            FunctionHandleGen::strategy(
                self.member_count.clone(),
                self.member_count.clone(),
                self.member_count.clone(),
            ),
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

        // Note that prop_test only allows a tuple of length up to 10
        (
            (address_pool_strat, identifiers_strat),
            module_handles_strat,
            (struct_handles_strat, struct_defs_strat),
            (function_handles_strat, function_defs_strat),
        )
            .prop_map(
                |(
                    (address_identifier_gens, identifier_gens),
                    module_handles_gen,
                    (struct_handle_gens, struct_def_gens),
                    (function_handle_gens, function_def_gens),
                )| {
                    //
                    // leaf pools
                    // TODO constant generation
                    let constant_pool = vec![];
                    let constant_pool_len = constant_pool.len();
                    let address_identifiers: Vec<_> = address_identifier_gens.into_iter().collect();
                    let address_identifiers_len = address_identifiers.len();
                    let identifiers: Vec<_> = identifier_gens.into_iter().collect();
                    let identifiers_len = identifiers.len();

                    //
                    // module handles
                    let mut module_handles_set = BTreeSet::new();
                    for (address, name) in module_handles_gen {
                        module_handles_set.insert(ModuleHandle {
                            address: AddressIdentifierIndex(
                                address.index(address_identifiers_len) as TableIndex
                            ),
                            name: IdentifierIndex(name.index(identifiers_len) as TableIndex),
                        });
                    }
                    let module_handles: Vec<_> = module_handles_set.into_iter().collect();
                    let module_handles_len = module_handles.len();

                    //
                    // struct handles
                    let mut struct_handles = vec![];
                    if module_handles_len > 1 {
                        let mut struct_handles_set = BTreeSet::new();
                        for (module_idx, name_idx, is_nominal_resource, _type_parameters) in
                            struct_handle_gens
                        {
                            let mut idx = module_idx.index(module_handles_len);
                            // 0 module handles are reserved for definitions and
                            // generated in the definition step
                            if idx == 0 {
                                idx += 1;
                            }
                            let module = ModuleHandleIndex(idx as TableIndex);
                            let name =
                                IdentifierIndex(name_idx.index(identifiers_len) as TableIndex);
                            if struct_handles_set.insert((module, name)) {
                                struct_handles.push(StructHandle {
                                    module: ModuleHandleIndex(idx as TableIndex),
                                    name: IdentifierIndex(
                                        name_idx.index(identifiers_len) as TableIndex
                                    ),
                                    is_nominal_resource,
                                    // TODO: enable type formals gen when we rework prop tests
                                    // for generics
                                    type_parameters: vec![],
                                })
                            }
                        }
                    }

                    //
                    // Struct definitions.
                    // Struct handles for the definitions are generated in this step
                    let mut state = StDefnMaterializeState::new(identifiers_len, struct_handles);
                    let mut struct_def_to_field_count: HashMap<usize, usize> = HashMap::new();
                    let mut struct_defs: Vec<StructDefinition> = vec![];
                    for struct_def_gen in struct_def_gens {
                        if let (Some(struct_def), offset) = struct_def_gen.materialize(&mut state) {
                            struct_defs.push(struct_def);
                            if offset > 0 {
                                struct_def_to_field_count.insert(struct_defs.len() - 1, offset);
                            }
                        }
                    }
                    let StDefnMaterializeState { struct_handles, .. } = state;

                    //
                    // Function handles. Creates signatures.
                    let mut signatures = vec![];
                    let mut function_handles: Vec<FunctionHandle> = vec![];
                    if module_handles_len > 1 {
                        let mut state = FnHandleMaterializeState::new(
                            module_handles_len,
                            identifiers_len,
                            struct_handles.len(),
                        );
                        for function_handle_gen in function_handle_gens {
                            if let Some(function_handle) =
                                function_handle_gen.materialize(&mut state)
                            {
                                function_handles.push(function_handle);
                            }
                        }
                        signatures = state.signatures();
                    }

                    //
                    // Function Definitions
                    // Here we need pretty much everything if we are going to emit instructions.
                    // signatures and function handles
                    let mut state = FnDefnMaterializeState::new(
                        identifiers_len,
                        constant_pool_len,
                        struct_handles.len(),
                        struct_defs.len(),
                        signatures,
                        function_handles,
                        struct_def_to_field_count,
                    );
                    let mut function_defs: Vec<FunctionDefinition> = vec![];
                    for function_def_gen in function_def_gens {
                        if let Some(function_def) = function_def_gen.materialize(&mut state) {
                            function_defs.push(function_def);
                        }
                    }
                    let (
                        signatures,
                        function_handles,
                        field_handles,
                        struct_def_instantiations,
                        function_instantiations,
                        field_instantiations,
                    ) = state.return_tables();

                    // Build a compiled module
                    CompiledModuleMut {
                        module_handles,
                        struct_handles,
                        function_handles,
                        field_handles,

                        struct_def_instantiations,
                        function_instantiations,
                        field_instantiations,

                        struct_defs,
                        function_defs,

                        signatures,

                        identifiers,
                        address_identifiers,
                        constant_pool,
                    }
                    .freeze()
                    .expect("valid modules should satisfy the bounds checker")
                },
            )
    }
}

#[derive(Debug)]
struct TypeSignatureIndex(u16);

#[derive(Debug)]
struct StDefnMaterializeState {
    identifiers_len: usize,
    struct_handles: Vec<StructHandle>,
    new_handles: BTreeSet<(ModuleHandleIndex, IdentifierIndex)>,
}

impl StDefnMaterializeState {
    fn new(identifiers_len: usize, struct_handles: Vec<StructHandle>) -> Self {
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
struct StructDefinitionGen {
    name_idx: PropIndex,
    // the is_nominal_resource field of generated struct handle is set to true if
    // either any of the fields contains a resource or self.is_nominal_resource is true
    is_nominal_resource: bool,
    type_parameters: Vec<KindGen>,
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
            // XXX 0..4 is the default member_count in CompiledModule
            option::of(vec(FieldDefinitionGen::strategy(), member_count)),
        )
            .prop_map(
                |(name_idx, is_nominal_resource, _type_parameters, is_public, field_defs)| Self {
                    name_idx,
                    is_nominal_resource,
                    // TODO: re-enable type formals gen once we rework prop tests for generics
                    type_parameters: vec![],
                    is_public,
                    field_defs,
                },
            )
    }

    fn materialize(self, state: &mut StDefnMaterializeState) -> (Option<StructDefinition>, usize) {
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
            signature: TypeSignature(self.signature_gen.materialize(state.struct_handles.len())),
        }
    }
}
