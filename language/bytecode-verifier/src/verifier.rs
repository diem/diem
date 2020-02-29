// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module contains the public APIs supported by the bytecode verifier.
use crate::{
    check_duplication::DuplicationChecker, code_unit_verifier::CodeUnitVerifier,
    instantiation_loops::InstantiationLoopChecker, resources::ResourceTransitiveChecker,
    signature::SignatureChecker, struct_defs::RecursiveStructDefChecker,
};
use anyhow::Error;
use libra_types::{
    language_storage::ModuleId,
    vm_error::{StatusCode, VMStatus},
};
use move_vm_types::{
    native_functions::dispatch::NativeFunction, native_structs::dispatch::resolve_native_struct,
};
use std::{collections::BTreeMap, fmt};
use vm::{
    access::{ModuleAccess, ScriptAccess},
    errors::{append_err_info, verification_error},
    file_format::{CompiledModule, CompiledProgram, CompiledScript, SignatureToken},
    resolver::Resolver,
    views::{ModuleView, ViewInternals},
    IndexKind,
};

/// A program that has been verified for internal consistency.
///
/// This includes cross-module checking for the base dependencies.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedProgram<'a> {
    script: VerifiedScript,
    modules: Vec<VerifiedModule>,
    deps: Vec<&'a VerifiedModule>,
}

impl<'a> VerifiedProgram<'a> {
    /// Creates a new `VerifiedProgram` after verifying the provided `CompiledProgram` against
    /// the provided base dependencies.
    ///
    /// On error, returns a list of verification statuses.
    pub fn new(
        program: CompiledProgram,
        deps: impl IntoIterator<Item = &'a VerifiedModule>,
    ) -> Result<Self, Vec<VMStatus>> {
        let deps: Vec<&VerifiedModule> = deps.into_iter().collect();
        // This is done separately to avoid unnecessary codegen due to monomorphization.
        Self::new_impl(program, deps)
    }

    fn new_impl(
        program: CompiledProgram,
        deps: Vec<&'a VerifiedModule>,
    ) -> Result<Self, Vec<VMStatus>> {
        let mut modules = vec![];

        for module in program.modules.into_iter() {
            let module = match VerifiedModule::new(module) {
                Ok(module) => module,
                Err((_, errors)) => {
                    return Err(errors);
                }
            };

            {
                // Verify against any modules compiled earlier as well.
                let deps = deps.iter().copied().chain(&modules);
                let errors = verify_module_dependencies(&module, deps);
                if !errors.is_empty() {
                    return Err(errors);
                }
            }

            modules.push(module);
        }

        let script = match VerifiedScript::new(program.script) {
            Ok(script) => script,
            Err((_, errors)) => {
                return Err(errors);
            }
        };

        {
            let deps = deps.iter().copied().chain(&modules);
            let errors = verify_script_dependencies(&script, deps);
            if !errors.is_empty() {
                return Err(errors);
            }
        }

        Ok(VerifiedProgram {
            script,
            modules,
            deps,
        })
    }

    /// Returns a reference to the script.
    pub fn script(&self) -> &VerifiedScript {
        &self.script
    }

    /// Returns a reference to the modules in this program.
    pub fn modules(&self) -> &[VerifiedModule] {
        &self.modules
    }

    /// Returns the dependencies this program was verified against.
    pub fn deps(&self) -> &[&'a VerifiedModule] {
        &self.deps
    }

    /// Converts this `VerifiedProgram` into a `CompiledProgram` instance.
    ///
    /// Converting back would require re-verifying this program.
    pub fn into_inner(self) -> CompiledProgram {
        CompiledProgram {
            modules: self
                .modules
                .into_iter()
                .map(|module| module.into_inner())
                .collect(),
            script: self.script.into_inner(),
        }
    }
}

impl<'a> fmt::Display for VerifiedProgram<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VerifiedProgram: {{\nModules: [\n")?;
        for m in &self.modules {
            writeln!(f, "{},", m)?;
        }
        // XXX Should this print out dependencies? Trying to avoid that for brevity for now.
        write!(f, "],\nScript: {},\nDependencies: ...}}", self.script)
    }
}

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
            errors.append(&mut InstantiationLoopChecker::new(&module).verify())
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

impl fmt::Display for VerifiedModule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VerifiedModule: {}", self.0)
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

impl fmt::Display for VerifiedScript {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VerifiedScript: {}", self.0)
    }
}

/// This function checks the extra requirements on the signature of the main function of a script.
pub fn verify_main_signature(script: &CompiledScript) -> Vec<VMStatus> {
    let function_handle = &script.function_handle_at(script.main().function);
    let function_signature = &script.function_signature_at(function_handle.signature);
    if !function_signature.return_types.is_empty() {
        return vec![VMStatus::new(StatusCode::INVALID_MAIN_FUNCTION_SIGNATURE)];
    }
    for arg_type in &function_signature.arg_types {
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
                let declared_function_signature =
                    native_function_definition_view.signature().as_inner();
                let expected_function_signature_res =
                    vm_native_function.signature(Some(module_view));
                let expected_function_signature_opt = match expected_function_signature_res {
                    Ok(opt) => opt,
                    Err(e) => {
                        errors.push(e);
                        continue;
                    }
                };
                let matching_signatures = expected_function_signature_opt
                    .map(|e| &e == declared_function_signature)
                    .unwrap_or(false);
                if !matching_signatures {
                    errors.push(verification_error(
                        IndexKind::FunctionHandle,
                        idx,
                        StatusCode::TYPE_MISMATCH,
                    ))
                }
            }
        }
    }
    errors
}

fn verify_native_structs(module_view: &ModuleView<VerifiedModule>) -> Vec<VMStatus> {
    let mut errors = vec![];

    let module_id = module_view.id();
    for (idx, native_struct_definition_view) in module_view
        .structs()
        .enumerate()
        .filter(|sdv| sdv.1.is_native())
    {
        let struct_name = native_struct_definition_view.name();

        match resolve_native_struct(&module_id, struct_name) {
            None => errors.push(verification_error(
                IndexKind::StructHandle,
                idx,
                StatusCode::MISSING_DEPENDENCY,
            )),
            Some(vm_native_struct) => {
                let declared_index = idx as u16;
                let declared_is_nominal_resource =
                    native_struct_definition_view.is_nominal_resource();
                let declared_type_formals = native_struct_definition_view.type_formals();

                let expected_index = vm_native_struct.expected_index.0;
                let expected_is_nominal_resource = vm_native_struct.expected_nominal_resource;
                let expected_type_formals = &vm_native_struct.expected_type_formals;
                if declared_index != expected_index
                    || declared_is_nominal_resource != expected_is_nominal_resource
                    || declared_type_formals != expected_type_formals
                {
                    errors.push(verification_error(
                        IndexKind::StructHandle,
                        idx,
                        StatusCode::TYPE_MISMATCH,
                    ))
                }
            }
        }
    }
    errors
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
                || struct_handle_view.type_formals() != struct_definition_view.type_formals()
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
            continue;
        }
        let function_name = function_handle_view.name();
        let owner_module = dependency_map[&owner_module_id];
        let owner_module_view = ModuleView::new(owner_module);
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
                            errors.push(verification_error(
                                IndexKind::FunctionHandle,
                                idx,
                                StatusCode::TYPE_MISMATCH,
                            ));
                        }
                    }
                    Err(err) => {
                        errors.push(append_err_info(err, IndexKind::FunctionHandle, idx));
                    }
                }
            } else {
                errors.push(verification_error(
                    IndexKind::FunctionHandle,
                    idx,
                    StatusCode::VISIBILITY_MISMATCH,
                ));
            }
        } else {
            errors.push(verification_error(
                IndexKind::FunctionHandle,
                idx,
                StatusCode::LOOKUP_FAILED,
            ));
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
