// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_binary_format::{
    errors::{bounds_error, PartialVMError},
    file_format::{
        AddressIdentifierIndex, CompiledModule, FunctionHandleIndex, IdentifierIndex,
        ModuleHandleIndex, SignatureIndex, StructDefinitionIndex, StructHandleIndex, TableIndex,
    },
    internals::ModuleIndex,
    views::{ModuleView, SignatureTokenView},
    IndexKind,
};
use move_core_types::vm_status::StatusCode;
use proptest::{
    prelude::*,
    sample::{self, Index as PropIndex},
};
use std::collections::BTreeMap;

mod code_unit;
pub use code_unit::{ApplyCodeUnitBoundsContext, CodeUnitBoundsMutation};
use move_binary_format::file_format::SignatureToken;

/// Represents the number of pointers that exist out from a node of a particular kind.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PointerKind {
    /// Exactly one pointer out with this index kind as its destination.
    One(IndexKind),
    /// Zero or one pointer out with this index kind as its destination. Like the `?` operator in
    /// regular expressions.
    Optional(IndexKind),
    /// Zero or more pointers out with this index kind as its destination. Like the `*` operator
    /// in regular expressions.
    Star(IndexKind),
}

impl PointerKind {
    /// A list of what pointers (indexes) exist out from a particular kind of node within the
    /// module.
    ///
    /// The only special case is `FunctionDefinition`, which contains a `CodeUnit` that can contain
    /// one of several kinds of pointers out. That is not represented in this table.
    #[inline]
    pub fn pointers_from(src_kind: IndexKind) -> &'static [PointerKind] {
        use IndexKind::*;
        use PointerKind::*;

        match src_kind {
            ModuleHandle => &[One(AddressIdentifier), One(Identifier)],
            StructHandle => &[One(ModuleHandle), One(Identifier)],
            FunctionHandle => &[
                One(ModuleHandle),
                One(Identifier),
                One(Signature),
                One(Signature),
            ],
            StructDefinition => &[One(StructHandle), Star(StructHandle)],
            FunctionDefinition => &[One(FunctionHandle), One(Signature)],
            FriendDeclaration => &[One(AddressIdentifier), One(Identifier)],
            Signature => &[Star(StructHandle)],
            FieldHandle => &[One(StructDefinition)],
            _ => &[],
        }
    }

    #[inline]
    pub fn to_index_kind(self) -> IndexKind {
        match self {
            PointerKind::One(idx) | PointerKind::Optional(idx) | PointerKind::Star(idx) => idx,
        }
    }
}

pub static VALID_POINTER_SRCS: &[IndexKind] = &[
    IndexKind::ModuleHandle,
    IndexKind::StructHandle,
    IndexKind::FunctionHandle,
    IndexKind::FieldHandle,
    IndexKind::StructDefinition,
    IndexKind::FunctionDefinition,
    IndexKind::FriendDeclaration,
    IndexKind::Signature,
];

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn pointer_kind_sanity() {
        for variant in IndexKind::variants() {
            if VALID_POINTER_SRCS.iter().any(|x| x == variant) {
                assert!(
                    !PointerKind::pointers_from(*variant).is_empty(),
                    "expected variant {:?} to be a valid pointer source",
                    variant,
                );
            } else {
                assert!(
                    PointerKind::pointers_from(*variant).is_empty(),
                    "expected variant {:?} to not be a valid pointer source",
                    variant,
                );
            }
        }
    }
}

/// Represents a single mutation to a `CompiledModule` to produce an out-of-bounds situation.
///
/// Use `OutOfBoundsMutation::strategy()` to generate them, preferably using `Vec` to generate
/// many at a time. Then use `ApplyOutOfBoundsContext` to apply those mutations.
#[derive(Debug)]
pub struct OutOfBoundsMutation {
    src_kind: IndexKind,
    src_idx: PropIndex,
    dst_kind: IndexKind,
    offset: usize,
}

impl OutOfBoundsMutation {
    pub fn strategy() -> impl Strategy<Value = Self> {
        (
            Self::src_kind_strategy(),
            any::<PropIndex>(),
            any::<PropIndex>(),
            0..16_usize,
        )
            .prop_map(|(src_kind, src_idx, dst_kind_idx, offset)| {
                let dst_kind = Self::dst_kind(src_kind, dst_kind_idx);
                Self {
                    src_kind,
                    src_idx,
                    dst_kind,
                    offset,
                }
            })
    }

    // Not all source kinds can be made to be out of bounds (e.g. inherent types can't.)
    fn src_kind_strategy() -> impl Strategy<Value = IndexKind> {
        sample::select(VALID_POINTER_SRCS)
    }

    fn dst_kind(src_kind: IndexKind, dst_kind_idx: PropIndex) -> IndexKind {
        dst_kind_idx
            .get(PointerKind::pointers_from(src_kind))
            .to_index_kind()
    }
}

/// This is used for source indexing, to work with pick_slice_idxs.
impl AsRef<PropIndex> for OutOfBoundsMutation {
    #[inline]
    fn as_ref(&self) -> &PropIndex {
        &self.src_idx
    }
}

pub struct ApplyOutOfBoundsContext {
    module: CompiledModule,
    // This is an Option because it gets moved out in apply before apply_one is called. Rust
    // doesn't let you call another con-consuming method after a partial move out.
    mutations: Option<Vec<OutOfBoundsMutation>>,

    // Some precomputations done for signatures.
    sig_structs: Vec<(SignatureIndex, usize)>,
}

impl ApplyOutOfBoundsContext {
    pub fn new(module: CompiledModule, mutations: Vec<OutOfBoundsMutation>) -> Self {
        let sig_structs: Vec<_> = Self::sig_structs(&module).collect();

        Self {
            module,
            mutations: Some(mutations),
            sig_structs,
        }
    }

    pub fn apply(mut self) -> (CompiledModule, Vec<PartialVMError>) {
        // This is a map from (source kind, dest kind) to the actual mutations -- this is done to
        // figure out how many mutations to do for a particular pair, which is required for
        // pick_slice_idxs below.
        let mut mutation_map = BTreeMap::new();
        for mutation in self
            .mutations
            .take()
            .expect("mutations should always be present")
        {
            mutation_map
                .entry((mutation.src_kind, mutation.dst_kind))
                .or_insert_with(Vec::new)
                .push(mutation);
        }

        let mut results = vec![];

        for ((src_kind, dst_kind), mutations) in mutation_map {
            // It would be cool to use an iterator here, if someone could figure out exactly how
            // to get the lifetimes right :)
            results.extend(self.apply_one(src_kind, dst_kind, mutations));
        }
        (self.module, results)
    }

    fn apply_one(
        &mut self,
        src_kind: IndexKind,
        dst_kind: IndexKind,
        mutations: Vec<OutOfBoundsMutation>,
    ) -> Vec<PartialVMError> {
        let src_count = match src_kind {
            IndexKind::Signature => self.sig_structs.len(),
            // For the other sorts it's always possible to change an index.
            src_kind => self.module.kind_count(src_kind),
        };
        // Any signature can be a destination, not just the ones that have structs in them.
        let dst_count = self.module.kind_count(dst_kind);
        let to_mutate = crate::helpers::pick_slice_idxs(src_count, &mutations);

        mutations
            .iter()
            .zip(to_mutate)
            .map(move |(mutation, src_idx)| {
                self.set_index(
                    src_kind,
                    src_idx,
                    dst_kind,
                    dst_count,
                    (dst_count + mutation.offset) as TableIndex,
                )
            })
            .collect()
    }

    /// Sets the particular index in the table
    ///
    /// For example, with `src_kind` set to `ModuleHandle` and `dst_kind` set to `AddressPool`,
    /// this will set self.module_handles[src_idx].address to new_idx.
    ///
    /// This is mainly used for test generation.
    fn set_index(
        &mut self,
        src_kind: IndexKind,
        src_idx: usize,
        dst_kind: IndexKind,
        dst_count: usize,
        new_idx: TableIndex,
    ) -> PartialVMError {
        use IndexKind::*;

        // These are default values, but some of the match arms below mutate them.
        let mut src_idx = src_idx;
        let err = bounds_error(
            StatusCode::INDEX_OUT_OF_BOUNDS,
            dst_kind,
            new_idx,
            dst_count,
        );

        // A dynamic type system would be able to express this next block of code far more
        // concisely. A static type system would require some sort of complicated dependent type
        // structure that Rust doesn't have. As things stand today, every possible case needs to
        // be listed out.

        match (src_kind, dst_kind) {
            (ModuleHandle, AddressIdentifier) => {
                self.module.module_handles[src_idx].address = AddressIdentifierIndex(new_idx)
            }
            (ModuleHandle, Identifier) => {
                self.module.module_handles[src_idx].name = IdentifierIndex(new_idx)
            }
            (StructHandle, ModuleHandle) => {
                self.module.struct_handles[src_idx].module = ModuleHandleIndex(new_idx)
            }
            (StructHandle, Identifier) => {
                self.module.struct_handles[src_idx].name = IdentifierIndex(new_idx)
            }
            (FunctionHandle, ModuleHandle) => {
                self.module.function_handles[src_idx].module = ModuleHandleIndex(new_idx)
            }
            (FunctionHandle, Identifier) => {
                self.module.function_handles[src_idx].name = IdentifierIndex(new_idx)
            }
            (FunctionHandle, Signature) => {
                self.module.function_handles[src_idx].parameters = SignatureIndex(new_idx)
            }
            (StructDefinition, StructHandle) => {
                self.module.struct_defs[src_idx].struct_handle = StructHandleIndex(new_idx)
            }
            (FunctionDefinition, FunctionHandle) => {
                self.module.function_defs[src_idx].function = FunctionHandleIndex(new_idx)
            }
            (FunctionDefinition, Signature) => {
                self.module.function_defs[src_idx]
                    .code
                    .as_mut()
                    .unwrap()
                    .locals = SignatureIndex(new_idx)
            }
            (Signature, StructHandle) => {
                let (actual_src_idx, arg_idx) = self.sig_structs[src_idx];
                src_idx = actual_src_idx.into_index();
                self.module.signatures[src_idx].0[arg_idx]
                    .debug_set_sh_idx(StructHandleIndex(new_idx));
            }
            (FieldHandle, StructDefinition) => {
                self.module.field_handles[src_idx].owner = StructDefinitionIndex(new_idx)
            }
            (FriendDeclaration, AddressIdentifier) => {
                self.module.friend_decls[src_idx].address = AddressIdentifierIndex(new_idx)
            }
            (FriendDeclaration, Identifier) => {
                self.module.friend_decls[src_idx].name = IdentifierIndex(new_idx)
            }
            _ => panic!("Invalid pointer kind: {:?} -> {:?}", src_kind, dst_kind),
        }

        err.at_index(src_kind, src_idx as TableIndex)
    }

    /// Returns the indexes of locals signatures that contain struct handles inside them.
    fn sig_structs(module: &CompiledModule) -> impl Iterator<Item = (SignatureIndex, usize)> + '_ {
        let module_view = ModuleView::new(module);
        module_view
            .signatures()
            .enumerate()
            .flat_map(|(idx, signature)| {
                let idx = SignatureIndex(idx as u16);
                Self::find_struct_tokens(signature.tokens(), move |arg_idx| (idx, arg_idx))
            })
    }

    #[inline]
    fn find_struct_tokens<'b, F, T>(
        tokens: impl IntoIterator<Item = SignatureTokenView<'b, CompiledModule>> + 'b,
        map_fn: F,
    ) -> impl Iterator<Item = T> + 'b
    where
        F: Fn(usize) -> T + 'b,
    {
        tokens
            .into_iter()
            .enumerate()
            .filter_map(move |(arg_idx, token)| {
                struct_handle(token.signature_token()).map(|_| map_fn(arg_idx))
            })
    }
}

fn struct_handle(token: &SignatureToken) -> Option<StructHandleIndex> {
    use SignatureToken::*;

    match token {
        Struct(sh_idx) => Some(*sh_idx),
        StructInstantiation(sh_idx, _) => Some(*sh_idx),
        Reference(token) | MutableReference(token) => struct_handle(token),
        Bool | U8 | U64 | U128 | Address | Signer | Vector(_) | TypeParameter(_) => None,
    }
}
