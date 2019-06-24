// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module contains the public APIs supported by the bytecode verifier.
use crate::{
    check_duplication::DuplicationChecker, code_unit_verifier::CodeUnitVerifier,
    resources::ResourceTransitiveChecker, signature::SignatureChecker,
    struct_defs::RecursiveStructDefChecker,
};
use std::collections::BTreeMap;
use types::language_storage::CodeKey;
use vm::{
    access::{ModuleAccess, ScriptAccess},
    errors::{VMStaticViolation, VerificationError},
    file_format::{CompiledModule, CompiledScript},
    resolver::Resolver,
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
    pub fn new(module: CompiledModule) -> Result<Self, (CompiledModule, Vec<VerificationError>)> {
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
            errors.append(&mut CodeUnitVerifier::new(&module).verify());
        }
        if errors.is_empty() {
            Ok(VerifiedModule(module))
        } else {
            Err((module, errors))
        }
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
    pub fn new(script: CompiledScript) -> Result<Self, (CompiledScript, Vec<VerificationError>)> {
        let fake_module = script.into_module();
        let (fake_module, mut errors) = match VerifiedModule::new(fake_module) {
            Ok(module) => (module.into_inner(), vec![]),
            Err((module, errors)) => (module, errors),
        };
        let script = fake_module.into_script();
        errors.append(
            &mut verify_main_signature(&script)
                .into_iter()
                .map(move |err| VerificationError {
                    kind: IndexKind::FunctionDefinition,
                    idx: 0,
                    err,
                })
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

/// Legacy version of `VerifiedModule`. Will be removed soon, please use `VerifiedModule` instead.
pub fn verify_module(module: CompiledModule) -> (CompiledModule, Vec<VerificationError>) {
    match VerifiedModule::new(module) {
        Ok(module) => (module.into_inner(), vec![]),
        Err((module, errors)) => (module, errors),
    }
}

/// Legacy version of `VerifiedScript`. Will be removed soon, please use `VerifiedScript` instead.
pub fn verify_script(script: CompiledScript) -> (CompiledScript, Vec<VerificationError>) {
    match VerifiedScript::new(script) {
        Ok(script) => (script.into_inner(), vec![]),
        Err((script, errors)) => (script, errors),
    }
}

/// This function checks the extra requirements on the signature of the main function of a script.
pub fn verify_main_signature(script: &CompiledScript) -> Vec<VMStaticViolation> {
    let function_handle = &script.function_handle_at(script.main().function);
    let function_signature = &script.function_signature_at(function_handle.signature);
    if !function_signature.return_types.is_empty() {
        return vec![VMStaticViolation::InvalidMainFunctionSignature];
    }
    for arg_type in &function_signature.arg_types {
        if !arg_type.is_primitive() {
            return vec![VMStaticViolation::InvalidMainFunctionSignature];
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
pub fn verify_module_dependencies(
    module: CompiledModule,
    dependencies: &[CompiledModule],
) -> (CompiledModule, Vec<VerificationError>) {
    let module_code_key = module.self_code_key();
    let mut dependency_map = BTreeMap::new();
    for dependency in dependencies {
        let dependency_code_key = dependency.self_code_key();
        if module_code_key != dependency_code_key {
            dependency_map.insert(dependency_code_key, dependency);
        }
    }
    let mut errors = vec![];
    let module_view = ModuleView::new(&module);
    errors.append(&mut verify_struct_kind(&module_view, &dependency_map));
    errors.append(&mut verify_function_visibility_and_type(
        &module_view,
        &dependency_map,
    ));
    errors.append(&mut verify_all_dependencies_provided(
        &module_view,
        &dependency_map,
    ));
    (module, errors)
}

/// Verifying the dependencies of a script follows the same recipe as `VerifiedScript::new`
/// ---convert to a module and invoke verify_module_dependencies. Each dependency of 'script' is
/// looked up in 'dependencies'.  If not found, an error is included in the returned list of errors.
/// If found, usage of types and functions of the dependency in 'script' is checked against the
/// declarations in the found module and mismatch errors are returned.
pub fn verify_script_dependencies(
    script: CompiledScript,
    dependencies: &[CompiledModule],
) -> (CompiledScript, Vec<VerificationError>) {
    let fake_module = script.into_module();
    let (fake_module, errors) = verify_module_dependencies(fake_module, dependencies);
    let script = fake_module.into_script();
    (script, errors)
}

fn verify_all_dependencies_provided(
    module_view: &ModuleView<CompiledModule>,
    dependency_map: &BTreeMap<CodeKey, &CompiledModule>,
) -> Vec<VerificationError> {
    let mut errors = vec![];
    for (idx, module_handle_view) in module_view.module_handles().enumerate() {
        let module_id = module_handle_view.module_code_key();
        if idx != CompiledModule::IMPLEMENTED_MODULE_INDEX as usize
            && !dependency_map.contains_key(&module_id)
        {
            errors.push(VerificationError {
                kind: IndexKind::ModuleHandle,
                idx,
                err: VMStaticViolation::MissingDependency,
            });
        }
    }
    errors
}

fn verify_struct_kind(
    module_view: &ModuleView<CompiledModule>,
    dependency_map: &BTreeMap<CodeKey, &CompiledModule>,
) -> Vec<VerificationError> {
    let mut errors = vec![];
    for (idx, struct_handle_view) in module_view.struct_handles().enumerate() {
        let owner_module_id = struct_handle_view.module_code_key();
        if !dependency_map.contains_key(&owner_module_id) {
            continue;
        }
        let struct_name = struct_handle_view.name();
        let owner_module = &dependency_map[&owner_module_id];
        let owner_module_view = ModuleView::new(*owner_module);
        if let Some(struct_definition_view) = owner_module_view.struct_definition(struct_name) {
            if struct_handle_view.is_resource() != struct_definition_view.is_resource() {
                errors.push(VerificationError {
                    kind: IndexKind::StructHandle,
                    idx,
                    err: VMStaticViolation::TypeMismatch,
                });
            }
        } else {
            errors.push(VerificationError {
                kind: IndexKind::StructHandle,
                idx,
                err: VMStaticViolation::LookupFailed,
            });
        }
    }
    errors
}

fn verify_function_visibility_and_type(
    module_view: &ModuleView<CompiledModule>,
    dependency_map: &BTreeMap<CodeKey, &CompiledModule>,
) -> Vec<VerificationError> {
    let resolver = Resolver::new(module_view.as_inner());
    let mut errors = vec![];
    for (idx, function_handle_view) in module_view.function_handles().enumerate() {
        let owner_module_id = function_handle_view.module_code_key();
        if !dependency_map.contains_key(&owner_module_id) {
            continue;
        }
        let function_name = function_handle_view.name();
        let owner_module = &dependency_map[&owner_module_id];
        let owner_module_view = ModuleView::new(*owner_module);
        if let Some(function_definition_view) = owner_module_view.function_definition(function_name)
        {
            if function_definition_view.is_public() {
                let function_definition_signature = function_definition_view.signature().as_inner();
                match resolver
                    .import_function_signature(owner_module, &function_definition_signature)
                {
                    Ok(imported_function_signature) => {
                        let function_handle_signature = function_handle_view.signature().as_inner();
                        if imported_function_signature != *function_handle_signature {
                            errors.push(VerificationError {
                                kind: IndexKind::FunctionHandle,
                                idx,
                                err: VMStaticViolation::TypeMismatch,
                            });
                        }
                    }
                    Err(err) => {
                        errors.push(VerificationError {
                            kind: IndexKind::FunctionHandle,
                            idx,
                            err,
                        });
                    }
                }
            } else {
                errors.push(VerificationError {
                    kind: IndexKind::FunctionHandle,
                    idx,
                    err: VMStaticViolation::VisibilityMismatch,
                });
            }
        } else {
            errors.push(VerificationError {
                kind: IndexKind::FunctionHandle,
                idx,
                err: VMStaticViolation::LookupFailed,
            });
        }
    }
    errors
}
