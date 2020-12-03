// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::control_flow_graph::VMControlFlowGraph;
use diem_types::vm_status::StatusCode;
use move_core_types::{
    account_address::AccountAddress, identifier::IdentStr, language_storage::ModuleId,
};
use vm::{
    access::{ModuleAccess, ScriptAccess},
    errors::{PartialVMError, PartialVMResult},
    file_format::{
        AddressIdentifierIndex, CodeUnit, CompiledScript, Constant, ConstantPoolIndex, FieldHandle,
        FieldHandleIndex, FieldInstantiation, FieldInstantiationIndex, FunctionDefinition,
        FunctionDefinitionIndex, FunctionHandle, FunctionHandleIndex, FunctionInstantiation,
        FunctionInstantiationIndex, IdentifierIndex, Kind, ModuleHandle, ModuleHandleIndex,
        Signature, SignatureIndex, SignatureToken, StructDefInstantiation,
        StructDefInstantiationIndex, StructDefinition, StructDefinitionIndex, StructHandle,
        StructHandleIndex,
    },
    CompiledModule,
};

// A `BinaryIndexedView` provides table indexed access for both `CompiledModule` and
// `CompiledScript`.
// Operations that are not allowed for `CompiledScript` return an error.
// A typical use of a `BinaryIndexedView` is while resolving indexes in bytecodes.
#[derive(Clone, Copy)]
pub(crate) enum BinaryIndexedView<'a> {
    Module(&'a CompiledModule),
    Script(&'a CompiledScript),
}

impl<'a> BinaryIndexedView<'a> {
    pub(crate) fn identifier_at(&self, idx: IdentifierIndex) -> &IdentStr {
        match self {
            BinaryIndexedView::Module(module) => module.identifier_at(idx),
            BinaryIndexedView::Script(script) => script.identifier_at(idx),
        }
    }

    pub(crate) fn address_identifier_at(&self, idx: AddressIdentifierIndex) -> &AccountAddress {
        match self {
            BinaryIndexedView::Module(module) => module.address_identifier_at(idx),
            BinaryIndexedView::Script(script) => script.address_identifier_at(idx),
        }
    }

    pub(crate) fn constant_at(&self, idx: ConstantPoolIndex) -> &Constant {
        match self {
            BinaryIndexedView::Module(module) => module.constant_at(idx),
            BinaryIndexedView::Script(script) => script.constant_at(idx),
        }
    }

    pub(crate) fn signature_at(&self, idx: SignatureIndex) -> &Signature {
        match self {
            BinaryIndexedView::Module(module) => module.signature_at(idx),
            BinaryIndexedView::Script(script) => script.signature_at(idx),
        }
    }

    pub(crate) fn module_handle_at(&self, idx: ModuleHandleIndex) -> &ModuleHandle {
        match self {
            BinaryIndexedView::Module(module) => module.module_handle_at(idx),
            BinaryIndexedView::Script(script) => script.module_handle_at(idx),
        }
    }

    pub(crate) fn struct_handle_at(&self, idx: StructHandleIndex) -> &StructHandle {
        match self {
            BinaryIndexedView::Module(module) => module.struct_handle_at(idx),
            BinaryIndexedView::Script(script) => script.struct_handle_at(idx),
        }
    }

    pub(crate) fn function_handle_at(&self, idx: FunctionHandleIndex) -> &FunctionHandle {
        match self {
            BinaryIndexedView::Module(module) => module.function_handle_at(idx),
            BinaryIndexedView::Script(script) => script.function_handle_at(idx),
        }
    }

    pub(crate) fn function_instantiation_at(
        &self,
        idx: FunctionInstantiationIndex,
    ) -> &FunctionInstantiation {
        match self {
            BinaryIndexedView::Module(module) => module.function_instantiation_at(idx),
            BinaryIndexedView::Script(script) => script.function_instantiation_at(idx),
        }
    }

    pub(crate) fn field_handle_at(&self, idx: FieldHandleIndex) -> PartialVMResult<&FieldHandle> {
        match self {
            BinaryIndexedView::Module(module) => Ok(module.field_handle_at(idx)),
            BinaryIndexedView::Script(_) => {
                Err(PartialVMError::new(StatusCode::INVALID_OPERATION_IN_SCRIPT))
            }
        }
    }

    pub(crate) fn struct_instantiation_at(
        &self,
        idx: StructDefInstantiationIndex,
    ) -> PartialVMResult<&StructDefInstantiation> {
        match self {
            BinaryIndexedView::Module(module) => Ok(module.struct_instantiation_at(idx)),
            BinaryIndexedView::Script(_) => {
                Err(PartialVMError::new(StatusCode::INVALID_OPERATION_IN_SCRIPT))
            }
        }
    }

    pub(crate) fn field_instantiation_at(
        &self,
        idx: FieldInstantiationIndex,
    ) -> PartialVMResult<&FieldInstantiation> {
        match self {
            BinaryIndexedView::Module(module) => Ok(module.field_instantiation_at(idx)),
            BinaryIndexedView::Script(_) => {
                Err(PartialVMError::new(StatusCode::INVALID_OPERATION_IN_SCRIPT))
            }
        }
    }

    pub(crate) fn struct_def_at(
        &self,
        idx: StructDefinitionIndex,
    ) -> PartialVMResult<&StructDefinition> {
        match self {
            BinaryIndexedView::Module(module) => Ok(module.struct_def_at(idx)),
            BinaryIndexedView::Script(_) => {
                Err(PartialVMError::new(StatusCode::INVALID_OPERATION_IN_SCRIPT))
            }
        }
    }

    pub(crate) fn function_def_at(
        &self,
        idx: FunctionDefinitionIndex,
    ) -> PartialVMResult<&FunctionDefinition> {
        match self {
            BinaryIndexedView::Module(module) => Ok(module.function_def_at(idx)),
            BinaryIndexedView::Script(_) => {
                Err(PartialVMError::new(StatusCode::INVALID_OPERATION_IN_SCRIPT))
            }
        }
    }

    // Return the `Kind` of a `SignatureToken` given a context.
    // A `TypeParameter` used the kind of the `constraints`.
    // `StructInstantiation` have to "merge" the kinds of the instantiation with that
    // of the generic type.
    pub(crate) fn kind(&self, ty: &SignatureToken, constraints: &[Kind]) -> Kind {
        use SignatureToken::*;

        match ty {
            // The primitive types & references have kind unrestricted.
            Bool | U8 | U64 | U128 | Address | Reference(_) | MutableReference(_) => Kind::Copyable,
            Signer => Kind::Resource,
            TypeParameter(idx) => constraints[*idx as usize],
            Vector(ty) => self.kind(ty, constraints),
            Struct(idx) => {
                let sh = self.struct_handle_at(*idx);
                if sh.is_nominal_resource {
                    Kind::Resource
                } else {
                    Kind::Copyable
                }
            }
            StructInstantiation(idx, type_args) => {
                let sh = self.struct_handle_at(*idx);
                if sh.is_nominal_resource {
                    return Kind::Resource;
                }
                // Gather the kinds of the type actuals.
                let kinds = type_args
                    .iter()
                    .map(|ty| self.kind(ty, constraints))
                    .collect::<Vec<_>>();
                // Derive the kind of the struct.
                //   - If any of the type actuals is `all`, then the struct is `all`.
                //     - `all` means some part of the type can be either `resource` or
                //       `unrestricted`.
                //     - Therefore it is also impossible to determine the kind of the type as a
                //       whole, and thus `all`.
                //   - If none of the type actuals is `all`, then the struct is a resource if
                //     and only if one of the type actuals is `resource`.
                kinds.iter().cloned().fold(Kind::Copyable, Kind::join)
            }
        }
    }

    pub(crate) fn module_id_for_handle(&self, module_handle: &ModuleHandle) -> ModuleId {
        ModuleId::new(
            *self.address_identifier_at(module_handle.address),
            self.identifier_at(module_handle.name).to_owned(),
        )
    }
}

const EMPTY_SIGNATURE: &Signature = &Signature(vec![]);

// A `FunctionView` holds all the information needed by the verifier for a
// `FunctionDefinition` and its `FunctionHandle` in a single view.
// A control flow graph is built for a function when the `FunctionView` is
// created.
// A `FunctionView` is created for all module functions except native functions.
// It is also created for a script.
pub(crate) struct FunctionView<'a> {
    index: Option<FunctionDefinitionIndex>,
    code: &'a CodeUnit,
    parameters: &'a Signature,
    return_: &'a Signature,
    locals: &'a Signature,
    type_parameters: &'a [Kind],
    cfg: VMControlFlowGraph,
}

impl<'a> FunctionView<'a> {
    // Creates a `FunctionView` for a module function.
    pub(crate) fn function(
        module: &'a CompiledModule,
        index: FunctionDefinitionIndex,
        code: &'a CodeUnit,
        function_handle: &'a FunctionHandle,
    ) -> Self {
        Self {
            index: Some(index),
            code,
            parameters: module.signature_at(function_handle.parameters),
            return_: module.signature_at(function_handle.return_),
            locals: module.signature_at(code.locals),
            type_parameters: &function_handle.type_parameters,
            cfg: VMControlFlowGraph::new(&code.code),
        }
    }

    // Creates a `FunctionView` for a script.
    pub(crate) fn script(script: &'a CompiledScript) -> Self {
        let code = &script.as_inner().code;
        let parameters = script.signature_at(script.as_inner().parameters);
        let locals = script.signature_at(code.locals);
        let type_parameters = &script.as_inner().type_parameters;
        Self {
            index: None,
            code,
            parameters,
            return_: EMPTY_SIGNATURE,
            locals,
            type_parameters,
            cfg: VMControlFlowGraph::new(&code.code),
        }
    }

    pub(crate) fn index(&self) -> Option<FunctionDefinitionIndex> {
        self.index
    }

    pub(crate) fn code(&self) -> &CodeUnit {
        self.code
    }

    pub(crate) fn parameters(&self) -> &Signature {
        self.parameters
    }

    pub(crate) fn return_(&self) -> &Signature {
        self.return_
    }

    pub(crate) fn locals(&self) -> &Signature {
        self.locals
    }

    pub(crate) fn type_parameters(&self) -> &[Kind] {
        self.type_parameters
    }

    pub(crate) fn cfg(&self) -> &VMControlFlowGraph {
        &self.cfg
    }
}
