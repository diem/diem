// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! An alternate representation of the file format built on top of the existing format.
//!
//! Some general notes:
//!
//! * These views are not meant to be set in stone. Feel free to change the views exposed as the
//!   format and our understanding evolves.
//! * The typical use for these views would be to materialize all the lazily evaluated data
//!   immediately -- the views are a convenience to make that simpler. They've been written as lazy
//!   iterators to aid understanding of the file format and to make it easy to generate views.

use std::iter::DoubleEndedIterator;

use crate::{
    access::ModuleAccess,
    file_format::{
        CodeUnit, CompiledModule, FieldDefinition, FunctionDefinition, FunctionHandle,
        FunctionSignature, Kind, LocalIndex, LocalsSignature, ModuleHandle, SignatureToken,
        StructDefinition, StructDefinitionIndex, StructFieldInformation, StructHandle,
        StructHandleIndex, TypeSignature,
    },
    SignatureTokenKind,
};
use std::collections::BTreeSet;

use libra_types::{
    identifier::IdentStr,
    language_storage::{ModuleId, StructTag},
};
use std::collections::BTreeMap;

/// Represents a lazily evaluated abstraction over a module.
///
/// `T` here is any sort of `ModuleAccess`. See the documentation in access.rs for more.
pub struct ModuleView<'a, T> {
    module: &'a T,
    name_to_function_definition_view: BTreeMap<&'a IdentStr, FunctionDefinitionView<'a, T>>,
    name_to_struct_definition_view: BTreeMap<&'a IdentStr, StructDefinitionView<'a, T>>,
}

impl<'a, T: ModuleAccess> ModuleView<'a, T> {
    pub fn new(module: &'a T) -> Self {
        let mut name_to_function_definition_view = BTreeMap::new();
        for function_def in module.function_defs() {
            let view = FunctionDefinitionView::new(module, function_def);
            name_to_function_definition_view.insert(view.name(), view);
        }
        let mut name_to_struct_definition_view = BTreeMap::new();
        for struct_def in module.struct_defs() {
            let view = StructDefinitionView::new(module, struct_def);
            name_to_struct_definition_view.insert(view.name(), view);
        }
        Self {
            module,
            name_to_function_definition_view,
            name_to_struct_definition_view,
        }
    }

    pub fn module_handles(
        &self,
    ) -> impl DoubleEndedIterator<Item = ModuleHandleView<'a, T>> + Send {
        let module = self.module;
        module
            .module_handles()
            .iter()
            .map(move |module_handle| ModuleHandleView::new(module, module_handle))
    }

    pub fn struct_handles(
        &self,
    ) -> impl DoubleEndedIterator<Item = StructHandleView<'a, T>> + Send {
        let module = self.module;
        module
            .struct_handles()
            .iter()
            .map(move |struct_handle| StructHandleView::new(module, struct_handle))
    }

    pub fn function_handles(
        &self,
    ) -> impl DoubleEndedIterator<Item = FunctionHandleView<'a, T>> + Send {
        let module = self.module;
        module
            .function_handles()
            .iter()
            .map(move |function_handle| FunctionHandleView::new(module, function_handle))
    }

    pub fn structs(&self) -> impl DoubleEndedIterator<Item = StructDefinitionView<'a, T>> + Send {
        let module = self.module;
        module
            .struct_defs()
            .iter()
            .map(move |struct_def| StructDefinitionView::new(module, struct_def))
    }

    pub fn fields(&self) -> impl DoubleEndedIterator<Item = FieldDefinitionView<'a, T>> + Send {
        let module = self.module;
        module
            .field_defs()
            .iter()
            .map(move |field_def| FieldDefinitionView::new(module, field_def))
    }

    pub fn functions(
        &self,
    ) -> impl DoubleEndedIterator<Item = FunctionDefinitionView<'a, T>> + Send {
        let module = self.module;
        module
            .function_defs()
            .iter()
            .map(move |function_def| FunctionDefinitionView::new(module, function_def))
    }

    pub fn type_signatures(
        &self,
    ) -> impl DoubleEndedIterator<Item = TypeSignatureView<'a, T>> + Send {
        let module = self.module;
        module
            .type_signatures()
            .iter()
            .map(move |type_signature| TypeSignatureView::new(module, type_signature))
    }

    pub fn function_signatures(
        &self,
    ) -> impl DoubleEndedIterator<Item = FunctionSignatureView<'a, T>> + Send {
        let module = self.module;
        module
            .function_signatures()
            .iter()
            .map(move |function_signature| FunctionSignatureView::new(module, function_signature))
    }

    pub fn locals_signatures(
        &self,
    ) -> impl DoubleEndedIterator<Item = LocalsSignatureView<'a, T>> + Send {
        let module = self.module;
        module
            .locals_signatures()
            .iter()
            .map(move |locals_signature| LocalsSignatureView::new(module, locals_signature))
    }

    pub fn function_definition(
        &self,
        name: &'a IdentStr,
    ) -> Option<&FunctionDefinitionView<'a, T>> {
        self.name_to_function_definition_view.get(name)
    }

    pub fn struct_definition(&self, name: &'a IdentStr) -> Option<&StructDefinitionView<'a, T>> {
        self.name_to_struct_definition_view.get(name)
    }

    pub fn function_acquired_resources(
        &self,
        function_handle: &FunctionHandle,
    ) -> BTreeSet<StructDefinitionIndex> {
        if function_handle.module.0 != CompiledModule::IMPLEMENTED_MODULE_INDEX {
            return BTreeSet::new();
        }

        // TODO these unwraps should be VMInvariantViolations
        let function_name = self.as_inner().identifier_at(function_handle.name);
        let function_def = self.function_definition(function_name).unwrap();
        function_def
            .as_inner()
            .acquires_global_resources
            .iter()
            .cloned()
            .collect()
    }

    pub fn id(&self) -> ModuleId {
        self.module.self_id()
    }

    /// Return the `StructHandleIndex` that corresponds to the normalized type `t` in this module's
    /// table of `StructHandle`'s. Returns `None` if there is no corresponding handle
    pub fn resolve_struct(&self, t: &StructTag) -> Option<StructHandleIndex> {
        for (idx, handle) in self.module.struct_handles().iter().enumerate() {
            if &StructHandleView::new(self.module, handle).normalize_struct() == t {
                return Some(StructHandleIndex::new(idx as u16));
            }
        }
        None
    }
}

pub struct ModuleHandleView<'a, T> {
    module: &'a T,
    module_handle: &'a ModuleHandle,
}

impl<'a, T: ModuleAccess> ModuleHandleView<'a, T> {
    pub fn new(module: &'a T, module_handle: &'a ModuleHandle) -> Self {
        Self {
            module,
            module_handle,
        }
    }

    pub fn module_id(&self) -> ModuleId {
        self.module.module_id_for_handle(self.module_handle)
    }
}

pub struct StructHandleView<'a, T> {
    module: &'a T,
    struct_handle: &'a StructHandle,
}

impl<'a, T: ModuleAccess> StructHandleView<'a, T> {
    pub fn new(module: &'a T, struct_handle: &'a StructHandle) -> Self {
        Self {
            module,
            struct_handle,
        }
    }

    pub fn handle(&self) -> &StructHandle {
        &self.struct_handle
    }

    pub fn is_nominal_resource(&self) -> bool {
        self.struct_handle.is_nominal_resource
    }

    pub fn type_formals(&self) -> &Vec<Kind> {
        &self.struct_handle.type_formals
    }

    pub fn module_handle(&self) -> &ModuleHandle {
        self.module.module_handle_at(self.struct_handle.module)
    }

    pub fn name(&self) -> &'a IdentStr {
        self.module.identifier_at(self.struct_handle.name)
    }

    pub fn module_id(&self) -> ModuleId {
        self.module.module_id_for_handle(self.module_handle())
    }

    /// Return the StructHandleIndex of this handle in the module's struct handle table
    pub fn handle_idx(&self) -> StructHandleIndex {
        for (idx, handle) in self.module.struct_handles().iter().enumerate() {
            if handle == self.handle() {
                return StructHandleIndex::new(idx as u16);
            }
        }
        unreachable!("Cannot resolve StructHandle {:?} in module {:?}. This should never happen in a well-formed `StructHandleView`. Perhaps this handle came from a different module?", self.handle(), self.module().name())
    }

    /// Return a normalized representation of this struct type that can be compared across modules
    pub fn normalize_struct(&self) -> StructTag {
        let module_id = self.module_id();
        StructTag {
            module: module_id.name().into(),
            address: *module_id.address(),
            name: self.name().into(),
            // TODO: take type params as input
            type_params: vec![],
        }
    }
}

pub struct FunctionHandleView<'a, T> {
    module: &'a T,
    function_handle: &'a FunctionHandle,
}

impl<'a, T: ModuleAccess> FunctionHandleView<'a, T> {
    pub fn new(module: &'a T, function_handle: &'a FunctionHandle) -> Self {
        Self {
            module,
            function_handle,
        }
    }

    pub fn module_handle(&self) -> &ModuleHandle {
        self.module.module_handle_at(self.function_handle.module)
    }

    pub fn name(&self) -> &'a IdentStr {
        self.module.identifier_at(self.function_handle.name)
    }

    pub fn signature(&self) -> FunctionSignatureView<'a, T> {
        let function_signature = self
            .module
            .function_signature_at(self.function_handle.signature);
        FunctionSignatureView::new(self.module, function_signature)
    }

    pub fn module_id(&self) -> ModuleId {
        self.module.module_id_for_handle(self.module_handle())
    }
}

pub struct StructDefinitionView<'a, T> {
    module: &'a T,
    struct_def: &'a StructDefinition,
    struct_handle_view: StructHandleView<'a, T>,
}

impl<'a, T: ModuleAccess> StructDefinitionView<'a, T> {
    pub fn new(module: &'a T, struct_def: &'a StructDefinition) -> Self {
        let struct_handle = module.struct_handle_at(struct_def.struct_handle);
        let struct_handle_view = StructHandleView::new(module, struct_handle);
        Self {
            module,
            struct_def,
            struct_handle_view,
        }
    }

    pub fn is_nominal_resource(&self) -> bool {
        self.struct_handle_view.is_nominal_resource()
    }

    pub fn is_native(&self) -> bool {
        match &self.struct_def.field_information {
            StructFieldInformation::Native => true,
            StructFieldInformation::Declared { .. } => false,
        }
    }

    pub fn type_formals(&self) -> &Vec<Kind> {
        self.struct_handle_view.type_formals()
    }

    pub fn fields(
        &self,
    ) -> Option<impl DoubleEndedIterator<Item = FieldDefinitionView<'a, T>> + Send> {
        let module = self.module;
        match self.struct_def.field_information {
            StructFieldInformation::Native => None,
            StructFieldInformation::Declared {
                field_count,
                fields,
            } => Some(
                module
                    .field_def_range(field_count, fields)
                    .iter()
                    .map(move |field_def| FieldDefinitionView::new(module, field_def)),
            ),
        }
    }

    pub fn name(&self) -> &'a IdentStr {
        self.struct_handle_view.name()
    }

    /// Return a normalized representation of this struct type that can be compared across modules
    pub fn normalize_struct(&self) -> StructTag {
        self.struct_handle_view.normalize_struct()
    }
}

pub struct FieldDefinitionView<'a, T> {
    module: &'a T,
    field_def: &'a FieldDefinition,
}

impl<'a, T: ModuleAccess> FieldDefinitionView<'a, T> {
    pub fn new(module: &'a T, field_def: &'a FieldDefinition) -> Self {
        Self { module, field_def }
    }

    pub fn name(&self) -> &'a IdentStr {
        self.module.identifier_at(self.field_def.name)
    }

    pub fn type_signature(&self) -> TypeSignatureView<'a, T> {
        let type_signature = self.module.type_signature_at(self.field_def.signature);
        TypeSignatureView::new(self.module, type_signature)
    }

    pub fn signature_token(&self) -> &'a SignatureToken {
        &self.module.type_signature_at(self.field_def.signature).0
    }

    pub fn signature_token_view(&self) -> SignatureTokenView<'a, T> {
        SignatureTokenView::new(self.module, self.signature_token())
    }

    // Field definitions are always private.

    /// The struct this field is defined in.
    pub fn member_of(&self) -> StructHandleView<'a, T> {
        let struct_handle = self.module.struct_handle_at(self.field_def.struct_);
        StructHandleView::new(self.module, struct_handle)
    }

    /// Return a normalized representation of the type of this field's declaring struct that can be
    /// compared across modules
    pub fn normalize_declaring_struct(&self) -> StructTag {
        self.member_of().normalize_struct()
    }
}

pub struct FunctionDefinitionView<'a, T> {
    module: &'a T,
    function_def: &'a FunctionDefinition,
    function_handle_view: FunctionHandleView<'a, T>,
}

impl<'a, T: ModuleAccess> FunctionDefinitionView<'a, T> {
    pub fn new(module: &'a T, function_def: &'a FunctionDefinition) -> Self {
        let function_handle = module.function_handle_at(function_def.function);
        let function_handle_view = FunctionHandleView::new(module, function_handle);
        Self {
            module,
            function_def,
            function_handle_view,
        }
    }

    pub fn is_public(&self) -> bool {
        self.function_def.is_public()
    }

    pub fn is_native(&self) -> bool {
        self.function_def.is_native()
    }

    pub fn locals_signature(&self) -> LocalsSignatureView<'a, T> {
        let locals_signature = self
            .module
            .locals_signature_at(self.function_def.code.locals);
        LocalsSignatureView::new(self.module, locals_signature)
    }

    pub fn name(&self) -> &'a IdentStr {
        self.function_handle_view.name()
    }

    pub fn signature(&self) -> FunctionSignatureView<'a, T> {
        self.function_handle_view.signature()
    }

    pub fn code(&self) -> &'a CodeUnit {
        &self.function_def.code
    }
}

pub struct TypeSignatureView<'a, T> {
    module: &'a T,
    type_signature: &'a TypeSignature,
}

impl<'a, T: ModuleAccess> TypeSignatureView<'a, T> {
    #[inline]
    pub fn new(module: &'a T, type_signature: &'a TypeSignature) -> Self {
        Self {
            module,
            type_signature,
        }
    }

    #[inline]
    pub fn token(&self) -> SignatureTokenView<'a, T> {
        SignatureTokenView::new(self.module, &self.type_signature.0)
    }

    #[inline]
    pub fn kind(&self, type_formals: &[Kind]) -> Kind {
        self.token().kind(type_formals)
    }

    #[inline]
    pub fn contains_nominal_resource(&self, type_formals: &[Kind]) -> bool {
        self.token().contains_nominal_resource(type_formals)
    }
}

pub struct FunctionSignatureView<'a, T> {
    module: &'a T,
    function_signature: &'a FunctionSignature,
}

impl<'a, T: ModuleAccess> FunctionSignatureView<'a, T> {
    #[inline]
    pub fn new(module: &'a T, function_signature: &'a FunctionSignature) -> Self {
        Self {
            module,
            function_signature,
        }
    }

    #[inline]
    pub fn return_tokens(&self) -> impl DoubleEndedIterator<Item = SignatureTokenView<'a, T>> + 'a {
        let module = self.module;
        self.function_signature
            .return_types
            .iter()
            .map(move |token| SignatureTokenView::new(module, token))
    }

    #[inline]
    pub fn arg_tokens(&self) -> impl DoubleEndedIterator<Item = SignatureTokenView<'a, T>> + 'a {
        let module = self.module;
        self.function_signature
            .arg_types
            .iter()
            .map(move |token| SignatureTokenView::new(module, token))
    }

    #[inline]
    pub fn type_formals(&self) -> &Vec<Kind> {
        &self.function_signature.type_formals
    }

    pub fn return_count(&self) -> usize {
        self.function_signature.return_types.len()
    }

    pub fn arg_count(&self) -> usize {
        self.function_signature.arg_types.len()
    }
}

pub struct LocalsSignatureView<'a, T> {
    module: &'a T,
    locals_signature: &'a LocalsSignature,
}

impl<'a, T: ModuleAccess> LocalsSignatureView<'a, T> {
    #[inline]
    pub fn new(module: &'a T, locals_signature: &'a LocalsSignature) -> Self {
        Self {
            module,
            locals_signature,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.locals_signature.0.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn tokens(&self) -> impl DoubleEndedIterator<Item = SignatureTokenView<'a, T>> + 'a {
        let module = self.module;
        self.locals_signature
            .0
            .iter()
            .map(move |token| SignatureTokenView::new(module, token))
    }

    pub fn token_at(&self, index: LocalIndex) -> SignatureTokenView<'a, T> {
        SignatureTokenView::new(self.module, &self.locals_signature.0[index as usize])
    }
}

pub struct SignatureTokenView<'a, T> {
    module: &'a T,
    token: &'a SignatureToken,
}

impl<'a, T: ModuleAccess> SignatureTokenView<'a, T> {
    #[inline]
    pub fn new(module: &'a T, token: &'a SignatureToken) -> Self {
        Self { module, token }
    }

    #[inline]
    pub fn struct_handle(&self) -> Option<StructHandleView<'a, T>> {
        self.struct_index()
            .map(|sh_idx| StructHandleView::new(self.module, self.module.struct_handle_at(sh_idx)))
    }

    #[inline]
    pub fn signature_token(&self) -> &SignatureToken {
        self.token
    }

    #[inline]
    pub fn signature_token_kind(&self) -> SignatureTokenKind {
        self.token.signature_token_kind()
    }

    // TODO: rework views to make the interfaces here cleaner.
    pub fn kind(&self, type_formals: &[Kind]) -> Kind {
        SignatureToken::kind((self.module.struct_handles(), type_formals), self.token)
    }

    /// Determines if the given signature token contains a nominal resource.
    /// More specifically, a signature token contains a nominal resource if
    ///   1) it is a type variable explicitly marked as resource kind.
    ///   2) it is a struct that
    ///       a) is marked as resource.
    ///       b) has a type actual which is a nominal resource.
    ///
    /// Similar to `SignatureTokenView::kind`, the context is used for looking up struct
    /// definitions & type formals.
    // TODO: refactor views so that we get the type formals from self.
    pub fn contains_nominal_resource(&self, type_formals: &[Kind]) -> bool {
        match self.token {
            SignatureToken::Struct(sh_idx, type_arguments) => {
                StructHandleView::new(self.module, self.module.struct_handle_at(*sh_idx))
                    .is_nominal_resource()
                    || type_arguments.iter().any(|token| {
                        Self::new(self.module, token).contains_nominal_resource(type_formals)
                    })
            }
            SignatureToken::Vector(ty) => {
                SignatureTokenView::new(self.module, ty).contains_nominal_resource(type_formals)
            }
            SignatureToken::Reference(_)
            | SignatureToken::MutableReference(_)
            | SignatureToken::Bool
            | SignatureToken::U8
            | SignatureToken::U64
            | SignatureToken::U128
            | SignatureToken::ByteArray
            | SignatureToken::Address
            | SignatureToken::TypeParameter(_) => false,
        }
    }

    #[inline]
    pub fn is_reference(&self) -> bool {
        self.token.is_reference()
    }

    #[inline]
    pub fn is_mutable_reference(&self) -> bool {
        self.token.is_mutable_reference()
    }

    #[inline]
    pub fn struct_index(&self) -> Option<StructHandleIndex> {
        self.token.struct_index()
    }

    /// If `self` is a struct or reference to a struct, return a normalized representation of this
    /// struct type that can be compared across modules
    pub fn normalize_struct(&self) -> Option<StructTag> {
        self.struct_handle().map(|handle| handle.normalize_struct())
    }

    /// Return the equivalent `SignatureToken` for `self` inside `module`
    pub fn resolve_in_module(&self, other_module: &T) -> Option<SignatureToken> {
        if let Some(struct_handle) = self.struct_handle() {
            // Token contains a struct handle from `self.module`. Need to resolve inside
            // `other_module`. We do this by normalizing the struct in `self.token`, then
            // searching for the normalized representation inside `other_module`
            // TODO: do we need to resolve `self.token`'s type actuals?
            let type_actuals = Vec::new();
            ModuleView::new(other_module)
                .resolve_struct(&struct_handle.normalize_struct())
                .map(|handle_idx| SignatureToken::Struct(handle_idx, type_actuals))
        } else {
            Some(self.token.clone())
        }
    }
}

impl<'a, T: ModuleAccess> ::std::fmt::Debug for SignatureTokenView<'a, T> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self.normalize_struct() {
            Some(s) if self.is_reference() => write!(f, "&{:?}", s),
            Some(s) if self.is_mutable_reference() => write!(f, "&mut {:?}", s),
            Some(s) => s.fmt(f),
            None => self.token.fmt(f),
        }
    }
}

/// This is used to expose some view internals to checks and other areas. This might be exposed
/// to external code in the future.
pub trait ViewInternals {
    type ModuleType;
    type Inner;

    fn module(&self) -> Self::ModuleType;
    fn as_inner(&self) -> Self::Inner;
}

macro_rules! impl_view_internals {
    ($view_type:ident, $inner_type:ty, $inner_var:ident) => {
        impl<'a, T: ModuleAccess> ViewInternals for $view_type<'a, T> {
            type ModuleType = &'a T;
            type Inner = &'a $inner_type;

            #[inline]
            fn module(&self) -> Self::ModuleType {
                &self.module
            }

            #[inline]
            fn as_inner(&self) -> Self::Inner {
                &self.$inner_var
            }
        }
    };
}

impl<'a, T: ModuleAccess> ViewInternals for ModuleView<'a, T> {
    type ModuleType = &'a T;
    type Inner = &'a T;

    fn module(&self) -> Self::ModuleType {
        self.module
    }

    fn as_inner(&self) -> Self::Inner {
        self.module
    }
}

impl_view_internals!(ModuleHandleView, ModuleHandle, module_handle);
impl_view_internals!(StructHandleView, StructHandle, struct_handle);
impl_view_internals!(FunctionHandleView, FunctionHandle, function_handle);
impl_view_internals!(StructDefinitionView, StructDefinition, struct_def);
impl_view_internals!(FunctionDefinitionView, FunctionDefinition, function_def);
impl_view_internals!(FieldDefinitionView, FieldDefinition, field_def);
impl_view_internals!(TypeSignatureView, TypeSignature, type_signature);
impl_view_internals!(FunctionSignatureView, FunctionSignature, function_signature);
impl_view_internals!(LocalsSignatureView, LocalsSignature, locals_signature);
impl_view_internals!(SignatureTokenView, SignatureToken, token);
