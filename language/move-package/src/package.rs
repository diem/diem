// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use std::collections::BTreeMap;

use bytecode_verifier::{cyclic_dependencies, dependencies, verify_module};
use diem_types::vm_status::StatusCode;
use move_core_types::language_storage::ModuleId;
use vm::{
    access::ModuleAccess,
    errors::{BinaryLoaderResult, Location, PartialVMError, VMResult},
    file_format::CompiledModule,
};

pub struct CompiledPackage {
    name: String,
    modules_with_bytes: Vec<(Vec<u8>, CompiledModule)>,
}

#[derive(Clone)]
pub struct VerifiedPackage {
    name: String,
    modules_with_bytes: Vec<(Vec<u8>, CompiledModule)>,
    dependencies: Vec<VerifiedPackage>,
}

impl CompiledPackage {
    pub fn from_bytes(
        name: String,
        module_bytes: impl IntoIterator<Item = Vec<u8>>,
    ) -> BinaryLoaderResult<Self> {
        let mut modules_with_bytes = vec![];
        for bytes in module_bytes {
            let module = CompiledModule::deserialize(&bytes)?;
            modules_with_bytes.push((bytes, module));
        }
        Ok(Self {
            name,
            modules_with_bytes,
        })
    }

    pub fn from_compilation(
        name: String,
        compiled_modules: impl IntoIterator<Item = CompiledModule>,
    ) -> Result<Self> {
        let mut modules_with_bytes = vec![];
        for module in compiled_modules {
            let mut bytes = vec![];
            module.serialize(&mut bytes)?;
            modules_with_bytes.push((bytes, module));
        }
        Ok(Self {
            name,
            modules_with_bytes,
        })
    }

    pub fn verify(self, dependencies: Vec<VerifiedPackage>) -> VMResult<VerifiedPackage> {
        fn collect_modules<'a>(
            modules: &mut BTreeMap<ModuleId, &'a CompiledModule>,
            package: &'a VerifiedPackage,
        ) -> VMResult<()> {
            for (_, module) in &package.modules_with_bytes {
                if modules.insert(module.self_id(), module).is_some() {
                    return Err(PartialVMError::new(StatusCode::DUPLICATE_MODULE_NAME)
                        .finish(Location::Undefined));
                }
            }
            for dep in &package.dependencies {
                collect_modules(modules, dep)?;
            }
            Ok(())
        }

        let CompiledPackage {
            name,
            modules_with_bytes: compiled_modules_with_bytes,
        } = self;

        // per-module bytecode checking
        let mut verified_modules_with_bytes = vec![];
        for (bytes, module) in compiled_modules_with_bytes {
            verify_module(&module)?;
            verified_modules_with_bytes.push((bytes, module));
        }

        // linking/dependency checking
        let mut all_modules = BTreeMap::new();
        for (_, module) in &verified_modules_with_bytes {
            if all_modules.insert(module.self_id(), module).is_some() {
                return Err(PartialVMError::new(StatusCode::DUPLICATE_MODULE_NAME)
                    .finish(Location::Undefined));
            }
        }
        for dep in &dependencies {
            collect_modules(&mut all_modules, dep)?;
        }

        for (_, module) in &verified_modules_with_bytes {
            let imm_deps = module
                .immediate_module_dependencies()
                .into_iter()
                .map(|module_id| {
                    all_modules.get(&module_id).copied().ok_or_else(|| {
                        PartialVMError::new(StatusCode::MISSING_DEPENDENCY)
                            .finish(Location::Undefined)
                    })
                })
                .collect::<VMResult<Vec<_>>>()?;
            dependencies::verify_module(module, imm_deps.into_iter())?;

            cyclic_dependencies::verify_module(module, |module_id| {
                all_modules
                    .get(module_id)
                    .map(|dep| dep.immediate_module_dependencies())
                    .ok_or_else(|| PartialVMError::new(StatusCode::MISSING_DEPENDENCY))
            })?;
        }

        Ok(VerifiedPackage {
            name,
            modules_with_bytes: verified_modules_with_bytes,
            dependencies,
        })
    }
}
