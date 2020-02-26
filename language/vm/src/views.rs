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

use crate::{access::ModuleAccess, file_format::*, SignatureTokenKind};
use std::collections::BTreeSet;

use libra_types::language_storage::ModuleId;
use move_core_types::identifier::IdentStr;
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

    pub fn field_handles(&self) -> impl DoubleEndedIterator<Item = FieldHandleView<'a, T>> + Send {
        let module = self.module;
        module
            .field_handles()
            .iter()
            .map(move |field_handle| FieldHandleView::new(module, field_handle))
    }

    pub fn struct_instantiations(
        &self,
    ) -> impl DoubleEndedIterator<Item = StructInstantiationView<'a, T>> + Send {
        let module = self.module;
        module
            .struct_instantiations()
            .iter()
            .map(move |struct_inst| StructInstantiationView::new(module, struct_inst))
    }

    pub fn function_instantiations(
        &self,
    ) -> impl DoubleEndedIterator<Item = FunctionInstantiationView<'a, T>> + Send {
        let module = self.module;
        module
            .function_instantiations()
            .iter()
            .map(move |func_inst| FunctionInstantiationView::new(module, func_inst))
    }

    pub fn field_instantiations(
        &self,
    ) -> impl DoubleEndedIterator<Item = FieldInstantiationView<'a, T>> + Send {
        let module = self.module;
        module
            .field_instantiations()
            .iter()
            .map(move |field_inst| FieldInstantiationView::new(module, field_inst))
    }

    pub fn signatures(&self) -> impl DoubleEndedIterator<Item = SignatureView<'a, T>> + Send {
        let module = self.module;
        module
            .signatures()
            .iter()
            .map(move |signature| SignatureView::new(module, signature))
    }

    pub fn structs(&self) -> impl DoubleEndedIterator<Item = StructDefinitionView<'a, T>> + Send {
        let module = self.module;
        module
            .struct_defs()
            .iter()
            .map(move |struct_def| StructDefinitionView::new(module, struct_def))
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

    pub fn type_parameters(&self) -> &Vec<Kind> {
        &self.struct_handle.type_parameters
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

    pub fn parameters(&self) -> &'a Signature {
        self.module.signature_at(self.function_handle.parameters)
    }

    pub fn return_(&self) -> &'a Signature {
        self.module.signature_at(self.function_handle.return_)
    }

    #[inline]
    pub fn return_tokens(&self) -> impl DoubleEndedIterator<Item = SignatureTokenView<'a, T>> + 'a {
        let module = self.module;
        let return_ = self.module.signature_at(self.function_handle.return_);
        return_
            .0
            .iter()
            .map(move |token| SignatureTokenView::new(module, token))
    }

    #[inline]
    pub fn arg_tokens(&self) -> impl DoubleEndedIterator<Item = SignatureTokenView<'a, T>> + 'a {
        let module = self.module;
        let parameters = self.module.signature_at(self.function_handle.parameters);
        parameters
            .0
            .iter()
            .map(move |token| SignatureTokenView::new(module, token))
    }

    #[inline]
    pub fn type_parameters(&self) -> &Vec<Kind> {
        &self.function_handle.type_parameters
    }

    pub fn return_count(&self) -> usize {
        self.module
            .signature_at(self.function_handle.return_)
            .0
            .len()
    }

    pub fn arg_count(&self) -> usize {
        self.module
            .signature_at(self.function_handle.parameters)
            .0
            .len()
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

    pub fn type_parameters(&self) -> &Vec<Kind> {
        self.struct_handle_view.type_parameters()
    }

    pub fn fields(
        &self,
    ) -> Option<impl DoubleEndedIterator<Item = FieldDefinitionView<'a, T>> + Send> {
        let module = self.module;
        match &self.struct_def.field_information {
            StructFieldInformation::Native => None,
            StructFieldInformation::Declared(fields) => Some(
                fields
                    .iter()
                    .map(move |field_def| FieldDefinitionView::new(module, field_def)),
            ),
        }
    }

    pub fn name(&self) -> &'a IdentStr {
        self.struct_handle_view.name()
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
        TypeSignatureView::new(self.module, &self.field_def.signature)
    }

    pub fn signature_token(&self) -> &'a SignatureToken {
        &self.field_def.signature.0
    }

    pub fn signature_token_view(&self) -> SignatureTokenView<'a, T> {
        SignatureTokenView::new(self.module, self.signature_token())
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

    pub fn locals_signature(&self) -> SignatureView<'a, T> {
        let locals_signature = self.module.signature_at(self.function_def.code.locals);
        SignatureView::new(self.module, locals_signature)
    }

    pub fn name(&self) -> &'a IdentStr {
        self.function_handle_view.name()
    }

    pub fn parameters(&self) -> &'a Signature {
        self.function_handle_view.parameters()
    }

    pub fn return_(&self) -> &'a Signature {
        self.function_handle_view.return_()
    }

    pub fn type_parameters(&self) -> &Vec<Kind> {
        self.function_handle_view.type_parameters()
    }

    pub fn return_tokens(&self) -> impl DoubleEndedIterator<Item = SignatureTokenView<'a, T>> + 'a {
        self.function_handle_view.return_tokens()
    }

    pub fn arg_tokens(&self) -> impl DoubleEndedIterator<Item = SignatureTokenView<'a, T>> + 'a {
        self.function_handle_view.arg_tokens()
    }

    pub fn return_count(&self) -> usize {
        self.function_handle_view.return_count()
    }

    pub fn arg_count(&self) -> usize {
        self.function_handle_view.arg_count()
    }

    pub fn code(&self) -> &'a CodeUnit {
        &self.function_def.code
    }
}

pub struct StructInstantiationView<'a, T> {
    #[allow(unused)]
    module: &'a T,
    #[allow(unused)]
    struct_inst: &'a StructDefInstantiation,
}

impl<'a, T: ModuleAccess> StructInstantiationView<'a, T> {
    #[inline]
    pub fn new(module: &'a T, struct_inst: &'a StructDefInstantiation) -> Self {
        Self {
            module,
            struct_inst,
        }
    }
}

pub struct FieldHandleView<'a, T> {
    #[allow(unused)]
    module: &'a T,
    #[allow(unused)]
    field_handle: &'a FieldHandle,
}

impl<'a, T: ModuleAccess> FieldHandleView<'a, T> {
    #[inline]
    pub fn new(module: &'a T, field_handle: &'a FieldHandle) -> Self {
        Self {
            module,
            field_handle,
        }
    }
}

pub struct FunctionInstantiationView<'a, T> {
    #[allow(unused)]
    module: &'a T,
    #[allow(unused)]
    func_inst: &'a FunctionInstantiation,
}

impl<'a, T: ModuleAccess> FunctionInstantiationView<'a, T> {
    #[inline]
    pub fn new(module: &'a T, func_inst: &'a FunctionInstantiation) -> Self {
        Self { module, func_inst }
    }
}

pub struct FieldInstantiationView<'a, T> {
    #[allow(unused)]
    module: &'a T,
    #[allow(unused)]
    field_inst: &'a FieldInstantiation,
}

impl<'a, T: ModuleAccess> FieldInstantiationView<'a, T> {
    #[inline]
    pub fn new(module: &'a T, field_inst: &'a FieldInstantiation) -> Self {
        Self { module, field_inst }
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
}

pub struct SignatureView<'a, T> {
    module: &'a T,
    signature: &'a Signature,
}

impl<'a, T: ModuleAccess> SignatureView<'a, T> {
    #[inline]
    pub fn new(module: &'a T, signature: &'a Signature) -> Self {
        Self { module, signature }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.signature.0.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn tokens(&self) -> impl DoubleEndedIterator<Item = SignatureTokenView<'a, T>> + 'a {
        let module = self.module;
        self.signature
            .0
            .iter()
            .map(move |token| SignatureTokenView::new(module, token))
    }

    pub fn token_at(&self, index: LocalIndex) -> SignatureTokenView<'a, T> {
        SignatureTokenView::new(self.module, &self.signature.0[index as usize])
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
    pub fn signature_token(&self) -> &SignatureToken {
        self.token
    }

    #[inline]
    pub fn signature_token_kind(&self) -> SignatureTokenKind {
        self.token.signature_token_kind()
    }

    #[inline]
    pub fn is_reference(&self) -> bool {
        self.token.is_reference()
    }

    #[inline]
    pub fn is_mutable_reference(&self) -> bool {
        self.token.is_mutable_reference()
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
impl_view_internals!(SignatureView, Signature, signature);
impl_view_internals!(SignatureTokenView, SignatureToken, token);
