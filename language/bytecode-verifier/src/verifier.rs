// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module contains the public APIs supported by the bytecode verifier.
use crate::{
    check_duplication::DuplicationChecker, code_unit_verifier::CodeUnitVerifier,
    constants::ConstantsChecker, instantiation_loops::InstantiationLoopChecker, resolver::Resolver,
    resources::ResourceTransitiveChecker, signature::SignatureChecker,
    struct_defs::RecursiveStructDefChecker,
};
use anyhow::Error;
use libra_types::{
    language_storage::ModuleId,
    vm_error::{StatusCode, VMStatus},
};
use std::collections::BTreeMap;
use vm::{
    access::{ModuleAccess, ScriptAccess},
    errors::{append_err_info, verification_error, VMResult},
    file_format::{CompiledModule, CompiledScript, ScriptConversionInfo, SignatureToken},
    views::{ModuleView, ViewInternals},
    IndexKind,
};

/// A module that has been verified for internal consistency.
///
/// This does not include cross-module checking -- that needs to be done separately.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedModule(CompiledModule);

impl VerifiedModule {
    /// Verifies this `CompiledModule`, returning a `VerifiedModule` on success.
    ///
    /// On failure, returns the original `CompiledModule` and a list of verification errors.
    ///
    /// There is a partial order on the checks. For example, the duplication check must precede the
    /// structural recursion check. In general, later checks are more expensive.
    pub fn new(module: CompiledModule) -> Result<Self, (CompiledModule, VMStatus)> {
        // All CompiledModule instances are statically guaranteed to be bounds checked, so there's
        // no need for more checking.
        match verify_module(&module) {
            Ok(()) => Ok(VerifiedModule(module)),
            Err(e) => Err((module, e)),
        }
    }

    /// Returns a new `VerifiedModule` that **does not do any verification.**
    ///
    /// THIS IS INCREDIBLY DANGEROUS BECAUSE IT BREAKS CORE ASSUMPTIONS. DO NOT USE THIS OUTSIDE OF
    /// TESTS.
    #[allow(non_snake_case)]
    #[doc(hidden)]
    pub fn bypass_verifier_DANGEROUS_FOR_TESTING_ONLY(module: CompiledModule) -> VerifiedModule {
        VerifiedModule(module)
    }

    /// Serializes this module into the provided buffer.
    ///
    /// This is merely a convenience wrapper around `module.as_inner().serialize(buf)`.
    ///
    /// `VerifiedModule` instances cannot be deserialized directly, since the input is potentially
    /// untrusted. Instead, one must go through `CompiledModule`.
    pub fn serialize(&self, buf: &mut Vec<u8>) -> Result<(), Error> {
        self.as_inner().serialize(buf)
    }

    /// Returns a reference to the `CompiledModule` within.
    pub fn as_inner(&self) -> &CompiledModule {
        &self.0
    }

    /// Returns the `CompiledModule` within. Conversion back to `VerifiedModule` will require
    /// going through the verifier again.
    pub fn into_inner(self) -> CompiledModule {
        self.0
    }
}

fn verify_module(module: &CompiledModule) -> VMResult<()> {
    DuplicationChecker::new(&module).verify()?;
    SignatureChecker::new(&module).verify()?;
    ResourceTransitiveChecker::new(&module).verify()?;
    ConstantsChecker::new(&module).verify()?;
    RecursiveStructDefChecker::new(&module).verify()?;
    InstantiationLoopChecker::new(&module).verify()?;
    CodeUnitVerifier::verify(&module)
}

impl ModuleAccess for VerifiedModule {
    fn as_module(&self) -> &CompiledModule {
        self.as_inner()
    }
}

/// A script that has been verified for internal consistency.
///
/// This does not include cross-module checking -- that needs to be done separately.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedScript(CompiledScript);

impl VerifiedScript {
    /// Verifies this `CompiledScript`, returning a `VerifiedScript` on success.
    ///
    /// On failure, returns the original `CompiledScript` and a list of verification errors.
    ///
    /// Verification of a script is done in two steps:
    /// - Convert the script into a module and run all the usual verification performed on a module
    /// - Check the signature of the main function of the script
    ///
    /// This approach works because critical operations such as MoveFrom, MoveToSender, and
    /// BorrowGlobal that are not allowed in the script function take a StructDefinitionIndex as an
    /// argument. Since the module constructed from a script is guaranteed to have an empty vector
    /// of struct definitions, the bounds checker will catch any occurrences of these illegal
    /// operations.
    #[allow(deprecated)]
    pub fn new(script: CompiledScript) -> Result<Self, (CompiledScript, VMStatus)> {
        let (info, fake_module) = script.into_module();
        let script = match VerifiedModule::new(fake_module) {
            Ok(module) => module.into_inner().into_script(info),
            Err((module, errors)) => return Err((module.into_script(info), errors)),
        };
        match verify_main_signature(&script) {
            Ok(()) => Ok(VerifiedScript(script)),
            Err(err) => {
                let err = append_err_info(err, IndexKind::FunctionDefinition, 0);
                Err((script, err))
            }
        }
    }

    /// Returns the corresponding `VerifiedModule` for this script.
    ///
    /// Every `VerifiedScript` is a `VerifiedModule`, but the inverse is not true, so there's no
    /// corresponding `VerifiedModule::into_script` function.
    pub fn into_module(self) -> (ScriptConversionInfo, VerifiedModule) {
        let (info, module) = self.into_inner().into_module();
        (info, VerifiedModule(module))
    }

    /// Serializes this script into the provided buffer.
    ///
    /// This is merely a convenience wrapper around `script.as_inner().serialize(buf)`.
    ///
    /// `VerifiedScript` instances cannot be deserialized directly, since the input is potentially
    /// untrusted. Instead, one must go through `CompiledScript`.
    pub fn serialize(&self, buf: &mut Vec<u8>) -> Result<(), Error> {
        self.as_inner().serialize(buf)
    }

    /// Returns a reference to the `CompiledScript` within.
    pub fn as_inner(&self) -> &CompiledScript {
        &self.0
    }

    /// Returns the `CompiledScript` within. Conversion back to `VerifiedScript` will require
    /// going through the verifier again.
    pub fn into_inner(self) -> CompiledScript {
        self.0
    }
}

impl ScriptAccess for VerifiedScript {
    fn as_script(&self) -> &CompiledScript {
        self.as_inner()
    }
}

/// This function checks the extra requirements on the signature of the main function of a script.
pub fn verify_main_signature(script: &CompiledScript) -> VMResult<()> {
    let arguments = script.signature_at(script.as_inner().parameters);
    for arg_type in &arguments.0 {
        if !(arg_type.is_primitive()
            || *arg_type == SignatureToken::Vector(Box::new(SignatureToken::U8)))
        {
            return Err(VMStatus::new(StatusCode::INVALID_MAIN_FUNCTION_SIGNATURE));
        }
    }
    Ok(())
}

/// Verification of a module in isolation (using `VerifiedModule::new`) trusts that struct and
/// function handles not implemented in the module are declared correctly. The following procedure
/// justifies this trust by checking that these declarations match the definitions in the module
/// dependencies. Each dependency of 'module' is looked up in 'dependencies'.  If not found, an
/// error is included in the returned list of errors.  If found, usage of types and functions of the
/// dependency in 'module' is checked against the declarations in the found module and mismatch
/// errors are returned.
pub fn verify_module_dependencies<'a>(
    module: &VerifiedModule,
    dependencies: impl IntoIterator<Item = &'a VerifiedModule>,
) -> VMResult<()> {
    let module_id = module.self_id();
    let mut dependency_map = BTreeMap::new();
    for dependency in dependencies {
        let dependency_id = dependency.self_id();
        if module_id != dependency_id {
            dependency_map.insert(dependency_id, dependency);
        }
    }
    verify_dependencies(module, &dependency_map)
}

pub fn verify_dependencies(
    module: &VerifiedModule,
    dependency_map: &BTreeMap<ModuleId, &VerifiedModule>,
) -> VMResult<()> {
    let module_view = ModuleView::new(module);
    verify_struct_kind(&module_view, &dependency_map)?;
    verify_function_visibility_and_type(&module_view, &dependency_map)?;
    verify_all_dependencies_provided(&module_view, &dependency_map)
}

/// Verifying the dependencies of a script follows the same recipe as `VerifiedScript::new`
/// ---convert to a module and invoke verify_module_dependencies. Each dependency of 'script' is
/// looked up in 'dependencies'.  If not found, an error is included in the returned list of errors.
/// If found, usage of types and functions of the dependency in 'script' is checked against the
/// declarations in the found module and mismatch errors are returned.
pub fn verify_script_dependencies<'a>(
    script: &VerifiedScript,
    dependencies: impl IntoIterator<Item = &'a VerifiedModule>,
) -> VMResult<()> {
    let (_, fake_module) = script.clone().into_module();
    verify_module_dependencies(&fake_module, dependencies)
}

pub fn verify_script_dependency_map(
    script: &VerifiedScript,
    dependency_map: &BTreeMap<ModuleId, &VerifiedModule>,
) -> VMResult<()> {
    let (_, fake_module) = script.clone().into_module();
    verify_dependencies(&fake_module, dependency_map)
}

fn verify_all_dependencies_provided(
    module_view: &ModuleView<VerifiedModule>,
    dependency_map: &BTreeMap<ModuleId, &VerifiedModule>,
) -> VMResult<()> {
    for (idx, module_handle_view) in module_view.module_handles().enumerate() {
        let module_id = module_handle_view.module_id();
        if idx != module_view.self_handle_idx().0 as usize
            && !dependency_map.contains_key(&module_id)
        {
            return Err(verification_error(
                IndexKind::ModuleHandle,
                idx,
                StatusCode::MISSING_DEPENDENCY,
            ));
        }
    }
    Ok(())
}

fn verify_struct_kind(
    module_view: &ModuleView<VerifiedModule>,
    dependency_map: &BTreeMap<ModuleId, &VerifiedModule>,
) -> VMResult<()> {
    for (idx, struct_handle_view) in module_view.struct_handles().enumerate() {
        let owner_module_id = struct_handle_view.module_id();
        if !dependency_map.contains_key(&owner_module_id) {
            // REVIEW: when does it happen?
            continue;
        }
        let struct_name = struct_handle_view.name();
        let owner_module = &dependency_map[&owner_module_id];
        let owner_module_view = ModuleView::new(*owner_module);
        if let Some(struct_definition_view) = owner_module_view.struct_definition(struct_name) {
            if struct_handle_view.is_nominal_resource()
                != struct_definition_view.is_nominal_resource()
                || struct_handle_view.type_parameters() != struct_definition_view.type_parameters()
            {
                return Err(verification_error(
                    IndexKind::StructHandle,
                    idx,
                    StatusCode::TYPE_MISMATCH,
                ));
            }
        } else {
            return Err(verification_error(
                IndexKind::StructHandle,
                idx,
                StatusCode::LOOKUP_FAILED,
            ));
        }
    }
    Ok(())
}

fn verify_function_visibility_and_type(
    module_view: &ModuleView<VerifiedModule>,
    dependency_map: &BTreeMap<ModuleId, &VerifiedModule>,
) -> VMResult<()> {
    let resolver = Resolver::new(module_view.as_inner());
    for (idx, function_handle_view) in module_view.function_handles().enumerate() {
        let owner_module_id = function_handle_view.module_id();
        if !dependency_map.contains_key(&owner_module_id) {
            // REVIEW: when does it happen? definitions? should we check correctness?
            continue;
        }
        let function_name = function_handle_view.name();
        let owner_module = dependency_map[&owner_module_id];
        let owner_module_view = ModuleView::new(owner_module);
        if let Some(function_definition_view) = owner_module_view.function_definition(function_name)
        {
            if !function_definition_view.is_public() {
                return Err(verification_error(
                    IndexKind::FunctionHandle,
                    idx,
                    StatusCode::VISIBILITY_MISMATCH,
                ));
            }
            // same type parameter constraints
            if function_definition_view.type_parameters() != function_handle_view.type_parameters()
            {
                return Err(verification_error(
                    IndexKind::FunctionHandle,
                    idx,
                    StatusCode::TYPE_MISMATCH,
                ));
            }
            // same parameters
            let handle_params = function_handle_view.parameters();
            let def_params = function_definition_view.parameters();
            resolver
                .compare_cross_module_signatures(handle_params, def_params, owner_module)
                .map_err(|err| append_err_info(err, IndexKind::FunctionHandle, idx))?;
            // same return_
            let handle_return = function_handle_view.return_();
            let def_return = function_definition_view.return_();
            resolver
                .compare_cross_module_signatures(handle_return, def_return, owner_module)
                .map_err(|err| append_err_info(err, IndexKind::FunctionHandle, idx))?;
        } else {
            return Err(verification_error(
                IndexKind::FunctionHandle,
                idx,
                StatusCode::LOOKUP_FAILED,
            ));
        }
    }
    Ok(())
}

/// Batch verify a list of modules and panic on any error. The modules should be topologically
/// sorted in their dependency order.
pub fn batch_verify_modules(modules: Vec<CompiledModule>) -> Vec<VerifiedModule> {
    let mut verified_modules = vec![];
    for module in modules.into_iter() {
        let verified_module = VerifiedModule::new(module).expect("stdlib module failed to verify");
        if let Err(e) = verify_module_dependencies(&verified_module, &verified_modules) {
            panic!("{:?} at {:?}", e, verified_module.self_id())
        }
        verified_modules.push(verified_module);
    }
    verified_modules
}
