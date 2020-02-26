// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module contains the public APIs supported by the bytecode verifier.
use crate::{
    check_duplication::DuplicationChecker, code_unit_verifier::CodeUnitVerifier,
    instantiation_loops::InstantiationLoopChecker, resolver::Resolver,
    resources::ResourceTransitiveChecker, signature::SignatureChecker,
    struct_defs::RecursiveStructDefChecker,
};
use anyhow::Error;
use libra_types::{
    language_storage::ModuleId,
    vm_error::{StatusCode, VMStatus},
};
use move_vm_types::native_functions::dispatch::NativeFunction;
use std::collections::BTreeMap;
use vm::{
    access::{ModuleAccess, ScriptAccess},
    errors::{append_err_info, verification_error},
    file_format::{CompiledModule, CompiledScript, SignatureToken},
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
    pub fn new(module: CompiledModule) -> Result<Self, (CompiledModule, Vec<VMStatus>)> {
        // All CompiledModule instances are statically guaranteed to be bounds checked, so there's
        // no need for more checking.
        let mut errors = DuplicationChecker::new(&module).verify();
        if errors.is_empty() {
            errors.append(&mut SignatureChecker::new(&module).verify());
            errors.append(&mut ResourceTransitiveChecker::new(&module).verify());
        }
        if errors.is_empty() {
            errors.append(&mut RecursiveStructDefChecker::new(&module).verify());
        }
        if errors.is_empty() {
            errors.append(&mut InstantiationLoopChecker::new(&module).verify());
        }
        if errors.is_empty() {
            errors.append(&mut CodeUnitVerifier::verify(&module));
        }
        if errors.is_empty() {
            Ok(VerifiedModule(module))
        } else {
            Err((module, errors))
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
    pub fn new(script: CompiledScript) -> Result<Self, (CompiledScript, Vec<VMStatus>)> {
        let fake_module = script.into_module();
        let (fake_module, mut errors) = match VerifiedModule::new(fake_module) {
            Ok(module) => (module.into_inner(), vec![]),
            Err((module, errors)) => (module, errors),
        };
        let script = fake_module.into_script();
        errors.append(
            &mut verify_main_signature(&script)
                .into_iter()
                .map(move |err| append_err_info(err, IndexKind::FunctionDefinition, 0))
                .collect(),
        );
        if errors.is_empty() {
            Ok(VerifiedScript(script))
        } else {
            Err((script, errors))
        }
    }

    /// Returns the corresponding `VerifiedModule` for this script.
    ///
    /// Every `VerifiedScript` is a `VerifiedModule`, but the inverse is not true, so there's no
    /// corresponding `VerifiedModule::into_script` function.
    pub fn into_module(self) -> VerifiedModule {
        VerifiedModule(self.into_inner().into_module())
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
pub fn verify_main_signature(script: &CompiledScript) -> Vec<VMStatus> {
    let function_handle = &script.function_handle_at(script.main().function);
    let return_ = script.signature_at(function_handle.return_);
    if !return_.is_empty() {
        return vec![VMStatus::new(StatusCode::INVALID_MAIN_FUNCTION_SIGNATURE)];
    }

    let arguments = script.signature_at(function_handle.parameters);
    for arg_type in &arguments.0 {
        if !(arg_type.is_primitive()
            || *arg_type == SignatureToken::Vector(Box::new(SignatureToken::U8)))
        {
            return vec![VMStatus::new(StatusCode::INVALID_MAIN_FUNCTION_SIGNATURE)];
        }
    }
    vec![]
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
) -> Vec<VMStatus> {
    let module_id = module.self_id();
    let mut dependency_map = BTreeMap::new();
    for dependency in dependencies {
        let dependency_id = dependency.self_id();
        if module_id != dependency_id {
            dependency_map.insert(dependency_id, dependency);
        }
    }
    let mut errors = vec![];
    let module_view = ModuleView::new(module);
    errors.append(&mut verify_struct_kind(&module_view, &dependency_map));
    errors.append(&mut verify_function_visibility_and_type(
        &module_view,
        &dependency_map,
    ));
    errors.append(&mut verify_all_dependencies_provided(
        &module_view,
        &dependency_map,
    ));
    errors.append(&mut verify_native_functions(&module_view));
    errors.append(&mut verify_native_structs(&module_view));
    errors
}

/// Verifying the dependencies of a script follows the same recipe as `VerifiedScript::new`
/// ---convert to a module and invoke verify_module_dependencies. Each dependency of 'script' is
/// looked up in 'dependencies'.  If not found, an error is included in the returned list of errors.
/// If found, usage of types and functions of the dependency in 'script' is checked against the
/// declarations in the found module and mismatch errors are returned.
pub fn verify_script_dependencies<'a>(
    script: &VerifiedScript,
    dependencies: impl IntoIterator<Item = &'a VerifiedModule>,
) -> Vec<VMStatus> {
    let fake_module = script.clone().into_module();
    verify_module_dependencies(&fake_module, dependencies)
}

fn verify_native_functions(module_view: &ModuleView<VerifiedModule>) -> Vec<VMStatus> {
    let mut errors = vec![];

    let module_id = module_view.id();
    for (idx, native_function_definition_view) in module_view
        .functions()
        .enumerate()
        .filter(|fdv| fdv.1.is_native())
    {
        let function_name = native_function_definition_view.name();
        match NativeFunction::resolve(&module_id, function_name) {
            None => errors.push(verification_error(
                IndexKind::FunctionHandle,
                idx,
                StatusCode::MISSING_DEPENDENCY,
            )),
            Some(vm_native_function) => {
                // check parameters
                let def_params = native_function_definition_view.parameters();
                let native_params = match vm_native_function.parameters(Some(module_view)) {
                    Ok(opt) => match opt {
                        None => {
                            errors.push(verification_error(
                                IndexKind::FunctionHandle,
                                idx,
                                StatusCode::TYPE_MISMATCH,
                            ));
                            continue;
                        }
                        Some(sig) => sig,
                    },
                    Err(e) => {
                        errors.push(e);
                        continue;
                    }
                };
                if def_params != &native_params {
                    errors.push(verification_error(
                        IndexKind::FunctionHandle,
                        idx,
                        StatusCode::TYPE_MISMATCH,
                    ));
                    continue;
                }

                // check return_
                let def_return_ = native_function_definition_view.return_();
                let native_return_ = match vm_native_function.return_(Some(module_view)) {
                    Ok(opt) => match opt {
                        None => {
                            errors.push(verification_error(
                                IndexKind::FunctionHandle,
                                idx,
                                StatusCode::TYPE_MISMATCH,
                            ));
                            continue;
                        }
                        Some(sig) => sig,
                    },
                    Err(e) => {
                        errors.push(e);
                        continue;
                    }
                };
                if def_return_ != &native_return_ {
                    errors.push(verification_error(
                        IndexKind::FunctionHandle,
                        idx,
                        StatusCode::TYPE_MISMATCH,
                    ));
                    continue;
                }

                // check type parameters
                let def_type_parameters = native_function_definition_view.type_parameters();
                let native_type_parameters =
                    match vm_native_function.type_parameters(Some(module_view)) {
                        Ok(opt) => match opt {
                            None => {
                                errors.push(verification_error(
                                    IndexKind::FunctionHandle,
                                    idx,
                                    StatusCode::TYPE_MISMATCH,
                                ));
                                continue;
                            }
                            Some(t_params) => t_params,
                        },
                        Err(e) => {
                            errors.push(e);
                            continue;
                        }
                    };
                if def_type_parameters != &native_type_parameters {
                    errors.push(verification_error(
                        IndexKind::FunctionHandle,
                        idx,
                        StatusCode::TYPE_MISMATCH,
                    ));
                    continue;
                }
            }
        }
    }
    errors
}

// TODO: native structs have been partially removed. Revisit.
fn verify_native_structs(module_view: &ModuleView<VerifiedModule>) -> Vec<VMStatus> {
    module_view
        .structs()
        .enumerate()
        .filter(|sdv| sdv.1.is_native())
        .map(|(idx, _)| {
            verification_error(IndexKind::StructHandle, idx, StatusCode::MISSING_DEPENDENCY)
        })
        .collect()
}

fn verify_all_dependencies_provided(
    module_view: &ModuleView<VerifiedModule>,
    dependency_map: &BTreeMap<ModuleId, &VerifiedModule>,
) -> Vec<VMStatus> {
    let mut errors = vec![];
    for (idx, module_handle_view) in module_view.module_handles().enumerate() {
        let module_id = module_handle_view.module_id();
        if idx != CompiledModule::IMPLEMENTED_MODULE_INDEX as usize
            && !dependency_map.contains_key(&module_id)
        {
            errors.push(verification_error(
                IndexKind::ModuleHandle,
                idx,
                StatusCode::MISSING_DEPENDENCY,
            ));
        }
    }
    errors
}

fn verify_struct_kind(
    module_view: &ModuleView<VerifiedModule>,
    dependency_map: &BTreeMap<ModuleId, &VerifiedModule>,
) -> Vec<VMStatus> {
    let mut errors = vec![];
    for (idx, struct_handle_view) in module_view.struct_handles().enumerate() {
        let owner_module_id = struct_handle_view.module_id();
        if !dependency_map.contains_key(&owner_module_id) {
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
                errors.push(verification_error(
                    IndexKind::StructHandle,
                    idx,
                    StatusCode::TYPE_MISMATCH,
                ));
            }
        } else {
            errors.push(verification_error(
                IndexKind::StructHandle,
                idx,
                StatusCode::LOOKUP_FAILED,
            ));
        }
    }
    errors
}

fn verify_function_visibility_and_type(
    module_view: &ModuleView<VerifiedModule>,
    dependency_map: &BTreeMap<ModuleId, &VerifiedModule>,
) -> Vec<VMStatus> {
    let resolver = Resolver::new(module_view.as_inner());
    let mut errors = vec![];
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
                errors.push(verification_error(
                    IndexKind::FunctionHandle,
                    idx,
                    StatusCode::VISIBILITY_MISMATCH,
                ));
                continue;
            }
            // same type parameter constraints
            if function_definition_view.type_parameters() != function_handle_view.type_parameters()
            {
                errors.push(verification_error(
                    IndexKind::FunctionHandle,
                    idx,
                    StatusCode::TYPE_MISMATCH,
                ));
                continue;
            }
            // same parameters
            let handle_params = function_handle_view.parameters();
            let def_params = function_definition_view.parameters();
            if let Err(err) =
                resolver.compare_cross_module_signatures(handle_params, def_params, owner_module)
            {
                errors.push(append_err_info(err, IndexKind::FunctionHandle, idx));
                continue;
            }
            // same return_
            let handle_return = function_handle_view.return_();
            let def_return = function_definition_view.return_();
            if let Err(err) =
                resolver.compare_cross_module_signatures(handle_return, def_return, owner_module)
            {
                errors.push(append_err_info(err, IndexKind::FunctionHandle, idx));
                continue;
            }
        } else {
            errors.push(verification_error(
                IndexKind::FunctionHandle,
                idx,
                StatusCode::LOOKUP_FAILED,
            ));
            continue;
        }
    }

    errors
}

/// Batch verify a list of modules and panic on any error. The modules should be topologically
/// sorted in their dependency order.
pub fn batch_verify_modules(modules: Vec<CompiledModule>) -> Vec<VerifiedModule> {
    let mut verified_modules = vec![];
    for module in modules.into_iter() {
        let verified_module = VerifiedModule::new(module).expect("stdlib module failed to verify");
        let verification_errors = verify_module_dependencies(&verified_module, &verified_modules);
        for e in &verification_errors {
            println!("{:?} at {:?}", e, verified_module.self_id());
        }
        assert!(verification_errors.is_empty());

        verified_modules.push(verified_module);
    }
    verified_modules
}
