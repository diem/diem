// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[allow(unused_imports)]
use log::{debug, info, warn};

use anyhow::bail;
use libra_types::transaction::{ArgumentABI, ScriptABI, TypeArgumentABI};
use move_core_types::language_storage::TypeTag;
use serde::{Deserialize, Serialize};
use spec_lang::{
    env::{GlobalEnv, ModuleEnv},
    ty,
};
use std::{collections::BTreeMap, io::Read, path::PathBuf};

/// Options passed into the ABI generator.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AbigenOptions {
    /// Where to find the .mv files of scripts.
    pub compiled_script_directory: String,
    /// In which directory to store output.
    pub output_directory: String,
}

impl Default for AbigenOptions {
    fn default() -> Self {
        Self {
            compiled_script_directory: ".".to_string(),
            output_directory: "abi".to_string(),
        }
    }
}

/// The ABI generator.
pub struct Abigen<'env> {
    /// Options.
    options: &'env AbigenOptions,
    /// Input definitions.
    env: &'env GlobalEnv,
    /// Map from file name to generated script ABI (if any).
    output: BTreeMap<String, ScriptABI>,
}

impl<'env> Abigen<'env> {
    /// Creates a new ABI generator.
    pub fn new(env: &'env GlobalEnv, options: &'env AbigenOptions) -> Self {
        Self {
            options,
            env,
            output: Default::default(),
        }
    }

    /// Returns the result of ABI generation, a vector of pairs of filenames
    /// and JSON content.
    pub fn into_result(mut self) -> Vec<(String, Vec<u8>)> {
        std::mem::take(&mut self.output)
            .into_iter()
            .map(|(path, abi)| {
                let content = lcs::to_bytes(&abi).expect("ABI serialization should not fail");
                (path, content)
            })
            .collect()
    }

    /// Generates ABIs for all script modules in the environment (excluding the dependency set).
    pub fn gen(&mut self) {
        for module in self.env.get_modules() {
            if module.is_script_module() && !module.is_dependency() {
                let mut path = PathBuf::from(&self.options.output_directory);
                path.push(
                    PathBuf::from(module.get_source_path())
                        .with_extension("abi")
                        .file_name()
                        .expect("file name"),
                );

                match self.compute_abi(&module) {
                    Ok(abi) => {
                        self.output.insert(path.to_string_lossy().to_string(), abi);
                    }
                    Err(error) => panic!(
                        "Error while processing script file {:?}: {}",
                        module.get_source_path(),
                        error
                    ),
                }
            }
        }
    }

    /// Compute the ABI of a script module.
    fn compute_abi(&self, module_env: &ModuleEnv<'env>) -> anyhow::Result<ScriptABI> {
        let symbol_pool = module_env.symbol_pool();
        let func = match module_env.get_functions().next() {
            Some(f) => f,
            None => bail!("A script module should define a function."),
        };
        let name = symbol_pool.string(func.get_name()).to_string();
        let doc = func.get_doc().to_string();
        let code = self.load_compiled_bytes(&module_env)?.to_vec();
        let ty_args = func
            .get_named_type_parameters()
            .iter()
            .map(|ty_param| {
                TypeArgumentABI::new(symbol_pool.string(ty_param.0).to_string().to_lowercase())
            })
            .collect();
        let args = func
            .get_parameters()
            .iter()
            .filter_map(
                |param| match self.get_type_tag_skipping_references(&param.1) {
                    Ok(Some(tag)) => Some(Ok(ArgumentABI::new(
                        symbol_pool.string(param.0).to_string(),
                        tag,
                    ))),
                    Ok(None) => None,
                    Err(error) => Some(Err(error)),
                },
            )
            .collect::<anyhow::Result<_>>()?;
        Ok(ScriptABI::new(name, doc, code, ty_args, args))
    }

    fn load_compiled_bytes(&self, module_env: &ModuleEnv<'env>) -> anyhow::Result<Vec<u8>> {
        let mut path = PathBuf::from(&self.options.compiled_script_directory);
        path.push(
            PathBuf::from(module_env.get_source_path())
                .with_extension("mv")
                .file_name()
                .expect("file name"),
        );
        let mut f = match std::fs::File::open(path.clone()) {
            Ok(f) => f,
            Err(error) => bail!("Failed to open compiled file {:?}: {}", path, error),
        };
        let mut bytes = Vec::new();
        f.read_to_end(&mut bytes)?;
        Ok(bytes)
    }

    fn get_type_tag_skipping_references(&self, ty0: &ty::Type) -> anyhow::Result<Option<TypeTag>> {
        use ty::Type::*;
        let tag = match ty0 {
            Primitive(prim) => {
                use ty::PrimitiveType::*;
                match prim {
                    Bool => TypeTag::Bool,
                    U8 => TypeTag::U8,
                    U64 => TypeTag::U64,
                    U128 => TypeTag::U128,
                    Address => TypeTag::Address,
                    Signer => TypeTag::Signer,
                    Num | Range | TypeValue => bail!("Type {:?} is not allowed in scripts.", ty0),
                }
            }
            Reference(_, _) => {
                // Skip references (most likely a `&signer` type)
                return Ok(None);
            }
            Vector(ty) => {
                let tag = self.get_type_tag(ty)?;
                TypeTag::Vector(Box::new(tag))
            }
            Tuple(_)
            | Struct(_, _, _)
            | TypeParameter(_)
            | Fun(_, _)
            | TypeDomain(_)
            | TypeLocal(_)
            | Error
            | Var(_) => bail!("Type {:?} is not allowed in scripts.", ty0),
        };
        Ok(Some(tag))
    }

    fn get_type_tag(&self, ty: &ty::Type) -> anyhow::Result<TypeTag> {
        if let Some(tag) = self.get_type_tag_skipping_references(ty)? {
            return Ok(tag);
        }
        bail!(
            "References such as {:?} are only allowed in the list of parameters.",
            ty
        );
    }
}
